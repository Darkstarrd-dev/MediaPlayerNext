param(
  [string]$OutputRoot
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

. (Join-Path $PSScriptRoot "common.ps1")

$context = Initialize-QualityOutputRoot -OutputRoot $OutputRoot -Label "release-verify"
$projectRoot = $context.ProjectRoot
$resolvedOutputRoot = $context.OutputRoot

$sidecarBuildLogPath = Join-Path $resolvedOutputRoot "subtitle-sidecar-build.log"
$tauriBuildLogPath = Join-Path $resolvedOutputRoot "tauri-build.log"
$summaryPath = Join-Path $resolvedOutputRoot "release-verify-summary.json"

$tauriConfigPath = Join-Path $projectRoot "src-tauri\tauri.conf.json"
$capabilitiesRoot = Join-Path $projectRoot "src-tauri\capabilities"
$sidecarEntryPath = Join-Path $projectRoot "apps\subtitle-sidecar\dist\src\index.js"
$bundleRoot = Join-Path $projectRoot "src-tauri\target\release\bundle"

$tauriConfig = Get-Content $tauriConfigPath -Raw | ConvertFrom-Json

$sidecarBuildResult = Invoke-LoggedCommand `
  -Executable "npm.cmd" `
  -Arguments @("run", "build", "--workspace", "@mediaplayernext/subtitle-sidecar") `
  -WorkingDirectory $projectRoot `
  -LogPath $sidecarBuildLogPath

$iconPaths = @()
if ($null -ne $tauriConfig.bundle -and $null -ne $tauriConfig.bundle.icon) {
  foreach ($icon in $tauriConfig.bundle.icon) {
    $iconPaths += Join-Path (Join-Path $projectRoot "src-tauri") $icon
  }
}

$missingIcons = @()
foreach ($iconPath in $iconPaths) {
  if (-not (Test-Path $iconPath)) {
    $missingIcons += Get-RepoRelativePath $iconPath
  }
}

$tauriBuildResult = Invoke-LoggedCommand `
  -Executable (Join-Path $projectRoot "scripts\run-tauri-build.cmd") `
  -Arguments @() `
  -WorkingDirectory $projectRoot `
  -LogPath $tauriBuildLogPath

$capabilityFiles = @()
if (Test-Path $capabilitiesRoot) {
  $capabilityFiles = @(Get-ChildItem -Path $capabilitiesRoot -Filter *.json -Recurse | ForEach-Object {
    Get-RepoRelativePath $_.FullName
  })
}

$bundleTargets = @()
if (Test-Path $bundleRoot) {
  $bundleTargets = @(Get-ChildItem -Path $bundleRoot -Directory | ForEach-Object {
    $_.Name
  })
}

$bundlePropertyNames = if ($null -ne $tauriConfig.bundle) {
  @($tauriConfig.bundle.PSObject.Properties.Name)
} else {
  @()
}

$resourceCount = if ($bundlePropertyNames -contains "resources") {
  @($tauriConfig.bundle.resources).Count
} else {
  0
}

$externalBinCount = if ($bundlePropertyNames -contains "externalBin") {
  @($tauriConfig.bundle.externalBin).Count
} else {
  0
}

$summary = [pscustomobject]@{
  checkedAt = (Get-Date).ToString("s")
  passed = (
    $sidecarBuildResult.ExitCode -eq 0 -and
    $tauriBuildResult.ExitCode -eq 0 -and
    (Test-Path $sidecarEntryPath) -and
    $missingIcons.Count -eq 0 -and
    $capabilityFiles.Count -gt 0 -and
    $bundleTargets.Count -gt 0
  )
  sidecarBuild = [pscustomobject]@{
    exitCode = $sidecarBuildResult.ExitCode
    durationMs = $sidecarBuildResult.DurationMs
    logPath = Get-RepoRelativePath $sidecarBuildLogPath
  }
  tauriBuild = [pscustomobject]@{
    exitCode = $tauriBuildResult.ExitCode
    durationMs = $tauriBuildResult.DurationMs
    logPath = Get-RepoRelativePath $tauriBuildLogPath
  }
  sidecarEntryPath = Get-RepoRelativePath $sidecarEntryPath
  sidecarEntryExists = (Test-Path $sidecarEntryPath)
  bundleRoot = Get-RepoRelativePath $bundleRoot
  bundleTargets = @($bundleTargets)
  capabilityFiles = @($capabilityFiles)
  missingIcons = @($missingIcons)
  declaredResourceCount = $resourceCount
  declaredExternalBinCount = $externalBinCount
}

$summary | Write-JsonFile -Path $summaryPath

if (-not $summary.passed) {
  throw "release verify failed, see $(Get-RepoRelativePath $summaryPath)"
}
