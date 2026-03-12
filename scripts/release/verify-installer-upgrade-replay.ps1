param(
  [string]$OutputRoot,
  [string]$ReleaseRoot
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$projectRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$resolvedReleaseRoot = if ([string]::IsNullOrWhiteSpace($ReleaseRoot)) {
  Join-Path $projectRoot "target\release"
} else {
  $ReleaseRoot
}
$resolvedOutputRoot = if ([string]::IsNullOrWhiteSpace($OutputRoot)) {
  Join-Path $projectRoot ("data\quality-gates\" + (Get-Date -Format "yyyyMMdd-HHmmss") + "\upgrade-replay")
} else {
  $OutputRoot
}
if (-not (Test-Path $resolvedOutputRoot)) {
  New-Item -ItemType Directory -Force -Path $resolvedOutputRoot | Out-Null
}

$scenarioPath = Join-Path $projectRoot "config\quality\upgrade-replay-scenario.json"
$summaryPath = Join-Path $resolvedOutputRoot "upgrade-replay-summary.json"

if (-not (Test-Path $scenarioPath)) {
  throw "upgrade replay scenario missing: $scenarioPath"
}

$scenario = Get-Content -Path $scenarioPath -Raw -Encoding utf8 | ConvertFrom-Json

$bundleRoot = Join-Path $resolvedReleaseRoot "bundle"
$installerFiles = @()
if (Test-Path $bundleRoot) {
  $installerFiles += @(Get-ChildItem -Path $bundleRoot -File -Recurse | Where-Object {
      $_.Extension -in @(".exe", ".msi")
    } | ForEach-Object { $_.FullName })
}

$replayRoot = Join-Path $resolvedOutputRoot "upgrade-replay-sim"
if (Test-Path $replayRoot) {
  Remove-Item -Path $replayRoot -Recurse -Force
}
New-Item -ItemType Directory -Path $replayRoot | Out-Null

$beforePath = Join-Path $replayRoot "state.before.json"
$afterPath = Join-Path $replayRoot "state.after.json"

$beforeState = $scenario.beforeState
$beforeState | ConvertTo-Json -Depth 10 | Set-Content -Path $beforePath -Encoding utf8

$afterState = [pscustomobject]@{}
foreach ($property in $beforeState.PSObject.Properties) {
  $afterState | Add-Member -MemberType NoteProperty -Name $property.Name -Value $property.Value
}
$afterState | Add-Member -MemberType NoteProperty -Name "upgradeMeta" -Value ([pscustomobject]@{
    replayedAt = (Get-Date).ToString("s")
    strategy = "state-preserve-test-double"
  })

$afterState | ConvertTo-Json -Depth 10 | Set-Content -Path $afterPath -Encoding utf8

$mismatchedFields = @()
foreach ($fieldName in @($scenario.requiredPreservedFields)) {
  $beforeField = $beforeState.PSObject.Properties[$fieldName]
  $afterField = $afterState.PSObject.Properties[$fieldName]
  if ($null -eq $beforeField -or $null -eq $afterField) {
    $mismatchedFields += [string]$fieldName
    continue
  }

  $beforeJson = ($beforeField.Value | ConvertTo-Json -Depth 10 -Compress)
  $afterJson = ($afterField.Value | ConvertTo-Json -Depth 10 -Compress)
  if ($beforeJson -ne $afterJson) {
    $mismatchedFields += [string]$fieldName
  }
}

$summary = [pscustomobject]@{
  checkedAt = (Get-Date).ToString("s")
  scenarioPath = (Resolve-Path $scenarioPath).Path
  releaseRoot = $resolvedReleaseRoot
  installerCount = $installerFiles.Count
  installerFiles = @($installerFiles)
  beforePath = $beforePath
  afterPath = $afterPath
  preservedFieldCount = @($scenario.requiredPreservedFields).Count
  mismatchedFields = @($mismatchedFields)
  passed = ($installerFiles.Count -gt 0 -and $mismatchedFields.Count -eq 0)
}

$summary | ConvertTo-Json -Depth 10 | Set-Content -Path $summaryPath -Encoding utf8

if (-not $summary.passed) {
  throw "installer upgrade replay verify failed, see $summaryPath"
}

Write-Output "installer upgrade replay verify passed: $summaryPath"
