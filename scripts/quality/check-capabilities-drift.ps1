param(
  [string]$OutputRoot,
  [switch]$UpdateBaseline
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

. (Join-Path $PSScriptRoot "common.ps1")

$context = Initialize-QualityOutputRoot -OutputRoot $OutputRoot -Label "capabilities-drift"
$projectRoot = $context.ProjectRoot
$resolvedOutputRoot = $context.OutputRoot

$baselinePath = Join-Path $projectRoot "config/quality/capabilities-baseline.json"
$summaryPath = Join-Path $resolvedOutputRoot "capabilities-drift-summary.json"
$logPath = Join-Path $resolvedOutputRoot "capabilities-drift.log"

function Normalize-Capability {
  param(
    [string]$Path,
    [pscustomobject]$Content
  )

  return [pscustomobject]@{
    path = Get-RepoRelativePath $Path
    identifier = if ($null -eq $Content.identifier) { "" } else { [string]$Content.identifier }
    windows = @($Content.windows | ForEach-Object { [string]$_ } | Sort-Object)
    permissions = @($Content.permissions | ForEach-Object { [string]$_ } | Sort-Object)
  }
}

$capabilityFiles = Get-ChildItem -Path (Join-Path $projectRoot "src-tauri/capabilities") -Filter "*.json" -File -Recurse |
  Sort-Object FullName

$currentCapabilities = @()
foreach ($file in $capabilityFiles) {
  $raw = Get-Content -Path $file.FullName -Raw -Encoding utf8
  $parsed = $raw | ConvertFrom-Json
  $currentCapabilities += Normalize-Capability -Path $file.FullName -Content $parsed
}

$current = [pscustomobject]@{
  fileCount = $currentCapabilities.Count
  capabilities = $currentCapabilities
}

if ($UpdateBaseline) {
  $baseline = [pscustomobject]@{
    updatedAt = (Get-Date).ToString("s")
    fileCount = $current.fileCount
    capabilities = $current.capabilities
  }
  $baseline | Write-JsonFile -Path $baselinePath
}

$baselineExists = Test-Path $baselinePath
$baselineMatched = $false
$baseline = $null
$missingInBaseline = @()
$missingInCurrent = @()
$changed = @()

if ($baselineExists) {
  $baseline = Get-Content -Path $baselinePath -Raw -Encoding utf8 | ConvertFrom-Json
  $baselineMap = @{}
  foreach ($item in $baseline.capabilities) {
    $baselineMap[$item.path] = $item
  }

  $currentMap = @{}
  foreach ($item in $current.capabilities) {
    $currentMap[$item.path] = $item
  }

  foreach ($path in $baselineMap.Keys) {
    if (-not $currentMap.ContainsKey($path)) {
      $missingInCurrent += $path
      continue
    }

    $baselineItemJson = ($baselineMap[$path] | ConvertTo-Json -Depth 10 -Compress)
    $currentItemJson = ($currentMap[$path] | ConvertTo-Json -Depth 10 -Compress)
    if ($baselineItemJson -ne $currentItemJson) {
      $changed += $path
    }
  }

  foreach ($path in $currentMap.Keys) {
    if (-not $baselineMap.ContainsKey($path)) {
      $missingInBaseline += $path
    }
  }

  $baselineMatched = ($missingInBaseline.Count -eq 0 -and $missingInCurrent.Count -eq 0 -and $changed.Count -eq 0)
}

$summary = [pscustomobject]@{
  checkedAt = (Get-Date).ToString("s")
  passed = ($baselineExists -and $baselineMatched)
  updateBaseline = [bool]$UpdateBaseline
  baselinePath = Get-RepoRelativePath $baselinePath
  baselineExists = $baselineExists
  baselineMatched = $baselineMatched
  scannedCapabilityFileCount = $current.fileCount
  missingInBaseline = @($missingInBaseline | Sort-Object)
  missingInCurrent = @($missingInCurrent | Sort-Object)
  changed = @($changed | Sort-Object)
  current = $current
}

$summary | Write-JsonFile -Path $summaryPath

$logLines = @(
  "baselinePath: $(Get-RepoRelativePath $baselinePath)",
  "baselineExists: $baselineExists",
  "baselineMatched: $baselineMatched",
  "scannedCapabilityFileCount: $($current.fileCount)",
  "missingInBaseline: $($missingInBaseline -join ', ')",
  "missingInCurrent: $($missingInCurrent -join ', ')",
  "changed: $($changed -join ', ')",
  "summaryPath: $(Get-RepoRelativePath $summaryPath)"
)
Set-Content -Path $logPath -Value $logLines -Encoding utf8

if (-not $summary.passed) {
  Write-Error "capabilities drift check failed"
  exit 1
}
