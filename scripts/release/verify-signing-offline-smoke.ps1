param(
  [string]$OutputRoot,
  [string]$ReleaseRoot
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

. (Join-Path (Split-Path -Parent $PSScriptRoot) "quality\common.ps1")

$projectRoot = Get-QualityProjectRoot
$resolvedReleaseRoot = if ([string]::IsNullOrWhiteSpace($ReleaseRoot)) {
  Join-Path $projectRoot "target\release"
} else {
  $ReleaseRoot
}
$resolvedOutputRoot = if ([string]::IsNullOrWhiteSpace($OutputRoot)) {
  Join-Path $projectRoot ("data\quality-gates\" + (Get-Date -Format "yyyyMMdd-HHmmss") + "\signing-offline-smoke")
} else {
  $OutputRoot
}
if (-not (Test-Path $resolvedOutputRoot)) {
  New-Item -ItemType Directory -Force -Path $resolvedOutputRoot | Out-Null
}

$policyPath = Join-Path $projectRoot "config\quality\release-signing-policy.json"
$pathsConfigPath = Join-Path $projectRoot "config\local.paths.json"
$summaryPath = Join-Path $resolvedOutputRoot "signing-offline-smoke-summary.json"
$smokeLogPath = Join-Path $resolvedOutputRoot "runtime-smoke-check.log"

if (-not (Test-Path $policyPath)) {
  throw "release signing policy missing: $policyPath"
}
if (-not (Test-Path $pathsConfigPath)) {
  throw "local paths config missing: $pathsConfigPath"
}

$policy = Get-Content -Path $policyPath -Raw -Encoding utf8 | ConvertFrom-Json
$localPaths = Get-Content -Path $pathsConfigPath -Raw -Encoding utf8 | ConvertFrom-Json

$missingRuntimePaths = @()
$runtimeArgs = @()
foreach ($key in @($policy.requiredLocalRuntimeKeys)) {
  $value = $localPaths.PSObject.Properties[$key]
  if ($null -eq $value -or [string]::IsNullOrWhiteSpace([string]$value.Value) -or -not (Test-Path ([string]$value.Value))) {
    $missingRuntimePaths += [string]$key
    continue
  }

  switch ($key) {
    "ffmpeg" { $runtimeArgs += @("--ffmpeg-path", [string]$value.Value) }
    "ffprobe" { $runtimeArgs += @("--ffprobe-path", [string]$value.Value) }
    "mpv" { $runtimeArgs += @("--mpv-path", [string]$value.Value) }
    default { }
  }
}

$bundleRoot = Join-Path $resolvedReleaseRoot "bundle"
$installerFiles = @()
if (Test-Path $bundleRoot) {
  $installerFiles = @(Get-ChildItem -Path $bundleRoot -File -Recurse | Where-Object {
      $_.Extension -in @(".exe", ".msi")
    })
}

$signatureStatuses = @($installerFiles | ForEach-Object {
  $signature = Get-AuthenticodeSignature -FilePath $_.FullName
  [pscustomobject]@{
    file = $_.FullName
    status = [string]$signature.Status
  }
})
$invalidSignatureCount = @($signatureStatuses | Where-Object { $_.status -ne "Valid" }).Count

$runtimeSmokeExe = Join-Path $resolvedReleaseRoot "runtime-smoke-check.exe"
$runtimeSmokeResult = if ((Test-Path $runtimeSmokeExe) -and $missingRuntimePaths.Count -eq 0) {
  Invoke-LoggedCommand `
    -Executable $runtimeSmokeExe `
    -Arguments $runtimeArgs `
    -WorkingDirectory $projectRoot `
    -LogPath $smokeLogPath
} else {
  [pscustomobject]@{
    ExitCode = 1
    DurationMs = 0
    LogPath = $smokeLogPath
  }
}

$signingRequired = [bool]$policy.signingRequired

$summary = [pscustomobject]@{
  checkedAt = (Get-Date).ToString("s")
  policyPath = Get-RepoRelativePath $policyPath
  releaseRoot = Get-RepoRelativePath $resolvedReleaseRoot
  signingRequired = $signingRequired
  installerCount = $installerFiles.Count
  invalidSignatureCount = $invalidSignatureCount
  signatureStatuses = @($signatureStatuses | ForEach-Object {
    [pscustomobject]@{
      file = Get-RepoRelativePath $_.file
      status = $_.status
    }
  })
  runtimeSmoke = [pscustomobject]@{
    executable = Get-RepoRelativePath $runtimeSmokeExe
    exitCode = $runtimeSmokeResult.ExitCode
    durationMs = $runtimeSmokeResult.DurationMs
    logPath = Get-RepoRelativePath $smokeLogPath
  }
  missingRuntimePaths = @($missingRuntimePaths)
  passed = (
    $installerFiles.Count -gt 0 -and
    $missingRuntimePaths.Count -eq 0 -and
    $runtimeSmokeResult.ExitCode -eq 0 -and
    ($(if ($signingRequired) { $invalidSignatureCount -eq 0 } else { $true }))
  )
}

$summary | ConvertTo-Json -Depth 10 | Set-Content -Path $summaryPath -Encoding utf8

if (-not $summary.passed) {
  throw "signing/offline smoke verify failed, see $summaryPath"
}

Write-Output "signing/offline smoke verify passed: $summaryPath"
