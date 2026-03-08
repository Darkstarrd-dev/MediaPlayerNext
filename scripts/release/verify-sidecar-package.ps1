param(
  [string]$OutputRoot,
  [string]$ReleaseRoot
)

$ErrorActionPreference = "Stop"

function New-RelativePath {
  param(
    [string]$ProjectRoot,
    [string]$PathValue
  )

  if ([string]::IsNullOrWhiteSpace($PathValue)) {
    return $PathValue
  }

  $absolute = [System.IO.Path]::GetFullPath($PathValue)
  if ($absolute.StartsWith($ProjectRoot, [System.StringComparison]::OrdinalIgnoreCase)) {
    return $absolute.Substring($ProjectRoot.Length).TrimStart('\\')
  }

  return $absolute
}

function Get-BundleResourceDeclarations {
  param([object]$Resources)

  if ($null -eq $Resources) {
    return @()
  }

  if ($Resources -is [System.Array]) {
    return @($Resources | ForEach-Object {
      [pscustomobject]@{
        source = [string]$_
        destination = $null
      }
    })
  }

  return @($Resources.PSObject.Properties | ForEach-Object {
    [pscustomobject]@{
      source = $_.Name
      destination = [string]$_.Value
    }
  })
}

$projectRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$resolvedReleaseRoot = if ([string]::IsNullOrWhiteSpace($ReleaseRoot)) {
  Join-Path $projectRoot "target\release"
} else {
  $ReleaseRoot
}
$defaultOutputRoot = Join-Path $projectRoot ("data\release-sidecar\" + (Get-Date -Format "yyyyMMdd-HHmmss"))
$resolvedOutputRoot = if ([string]::IsNullOrWhiteSpace($OutputRoot)) { $defaultOutputRoot } else { $OutputRoot }

if (-not (Test-Path $resolvedOutputRoot)) {
  New-Item -ItemType Directory -Path $resolvedOutputRoot -Force | Out-Null
}

$tauriConfigPath = Join-Path $projectRoot "src-tauri\tauri.conf.json"
$sourceEntryPath = Join-Path $projectRoot "apps\subtitle-sidecar\dist\src\index.js"
$expectedResourceSource = "../apps/subtitle-sidecar/dist/src/**/*"
$expectedResourceDestination = "sidecar/"
$expectedBundledRelativePath = "sidecar/index.js"

$tauriConfig = Get-Content $tauriConfigPath -Raw | ConvertFrom-Json
$resourceDeclarations = Get-BundleResourceDeclarations -Resources $tauriConfig.bundle.resources
$hasExpectedResourceMapping = @($resourceDeclarations | Where-Object {
  $_.source -eq $expectedResourceSource -and $_.destination -eq $expectedResourceDestination
}).Count -gt 0

$stagedEntryMatches = @()
if (Test-Path $resolvedReleaseRoot) {
  $stagedEntryMatches = @(Get-ChildItem -Path $resolvedReleaseRoot -Filter index.js -File -Recurse | Where-Object {
    $_.FullName -match '[\\/]sidecar[\\/]index\.js$'
  } | ForEach-Object {
    New-RelativePath -ProjectRoot $projectRoot -PathValue $_.FullName
  })
}

$summary = [pscustomobject]@{
  checkedAt = (Get-Date).ToString("s")
  tauriConfigPath = "src-tauri/tauri.conf.json"
  releaseRoot = New-RelativePath -ProjectRoot $projectRoot -PathValue $resolvedReleaseRoot
  sourceEntryPath = New-RelativePath -ProjectRoot $projectRoot -PathValue $sourceEntryPath
  sourceEntryExists = (Test-Path $sourceEntryPath)
  expectedResourceSource = $expectedResourceSource
  expectedResourceDestination = $expectedResourceDestination
  expectedBundledRelativePath = $expectedBundledRelativePath
  declaredResourceMappings = @($resourceDeclarations)
  hasExpectedResourceMapping = $hasExpectedResourceMapping
  stagedEntryMatches = @($stagedEntryMatches)
  stagedEntryFound = ($stagedEntryMatches.Count -gt 0)
  passed = (
    (Test-Path $sourceEntryPath) -and
    $hasExpectedResourceMapping -and
    $stagedEntryMatches.Count -gt 0
  )
}

$summaryPath = Join-Path $resolvedOutputRoot "sidecar-package-summary.json"
$summary | ConvertTo-Json -Depth 8 | Set-Content -Path $summaryPath -Encoding utf8

if (-not $summary.passed) {
  throw "sidecar package verification failed, see $summaryPath"
}

Write-Output "sidecar package verification passed: $summaryPath"
