param(
  [string]$OutputRoot,
  [switch]$UpdateBaseline
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

. (Join-Path $PSScriptRoot "common.ps1")

$context = Initialize-QualityOutputRoot -OutputRoot $OutputRoot -Label "binary-size"
$projectRoot = $context.ProjectRoot
$resolvedOutputRoot = $context.OutputRoot

$baselinePath = Join-Path $projectRoot "config/quality/binary-size-baseline.json"
$summaryPath = Join-Path $resolvedOutputRoot "binary-size-summary.json"
$logPath = Join-Path $resolvedOutputRoot "binary-size.log"

if (-not (Test-Path $baselinePath)) {
  throw "binary size baseline missing: $baselinePath"
}

$baseline = Get-Content -Path $baselinePath -Raw -Encoding utf8 | ConvertFrom-Json
$entries = @($baseline.artifacts)

$results = @()
foreach ($entry in $entries) {
  $relativePath = [string]$entry.path
  $maxBytes = [long]$entry.maxBytes
  $artifactPath = Join-Path $projectRoot $relativePath

  $exists = Test-Path $artifactPath
  $sizeBytes = if ($exists) { (Get-Item $artifactPath).Length } else { $null }
  $overMax = if ($exists) { [long]$sizeBytes -gt $maxBytes } else { $true }

  $results += [pscustomobject]@{
    path = $relativePath.Replace("\", "/")
    exists = $exists
    sizeBytes = $sizeBytes
    maxBytes = $maxBytes
    overMax = $overMax
  }
}

if ($UpdateBaseline) {
  $updatedArtifacts = @($results | ForEach-Object {
    [pscustomobject]@{
      path = $_.path
      maxBytes = $_.maxBytes
      baselineBytes = $_.sizeBytes
    }
  })

  $nextBaseline = [pscustomobject]@{
    updatedAt = (Get-Date).ToString("s")
    artifacts = $updatedArtifacts
  }
  $nextBaseline | Write-JsonFile -Path $baselinePath
}

$failed = @($results | Where-Object { -not $_.exists -or $_.overMax })

$summary = [pscustomobject]@{
  checkedAt = (Get-Date).ToString("s")
  passed = ($failed.Count -eq 0)
  updateBaseline = [bool]$UpdateBaseline
  baselinePath = Get-RepoRelativePath $baselinePath
  artifactCount = $results.Count
  failedCount = $failed.Count
  failedArtifacts = @($failed)
  artifacts = @($results)
}

$summary | Write-JsonFile -Path $summaryPath

$logLines = @(
  "baselinePath: $(Get-RepoRelativePath $baselinePath)",
  "artifactCount: $($results.Count)",
  "failedCount: $($failed.Count)",
  "summaryPath: $(Get-RepoRelativePath $summaryPath)"
)
Set-Content -Path $logPath -Value $logLines -Encoding utf8

if (-not $summary.passed) {
  Write-Error "binary size check failed"
  exit 1
}
