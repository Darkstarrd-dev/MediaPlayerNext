param(
  [string]$OutputRoot
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

. (Join-Path $PSScriptRoot "common.ps1")

$context = Initialize-QualityOutputRoot -OutputRoot $OutputRoot -Label "coverage"
$projectRoot = $context.ProjectRoot
$resolvedOutputRoot = $context.OutputRoot

$cargoRunner = Join-Path $projectRoot "scripts\run-cargo-with-msvc.cmd"
$logPath = Join-Path $resolvedOutputRoot "coverage.log"
$lcovPath = Join-Path $resolvedOutputRoot "lcov.info"
$summaryPath = Join-Path $resolvedOutputRoot "coverage-summary.json"

$commandResult = Invoke-LoggedCommand `
  -Executable $cargoRunner `
  -Arguments @(
    "llvm-cov",
    "nextest",
    "--workspace",
    "--all-features",
    "--lcov",
    "--output-path",
    $lcovPath
  ) `
  -WorkingDirectory $projectRoot `
  -LogPath $logPath

$lineFound = 0
$lineHit = 0
$branchFound = 0
$branchHit = 0

if (Test-Path $lcovPath) {
  $lcovContent = Get-Content $lcovPath
  foreach ($line in $lcovContent) {
    if ($line -match '^LF:(\d+)$') {
      $lineFound += [int]$Matches[1]
      continue
    }

    if ($line -match '^LH:(\d+)$') {
      $lineHit += [int]$Matches[1]
      continue
    }

    if ($line -match '^BRF:(\d+)$') {
      $branchFound += [int]$Matches[1]
      continue
    }

    if ($line -match '^BRH:(\d+)$') {
      $branchHit += [int]$Matches[1]
    }
  }
}

$lineCoveragePercent = if ($lineFound -gt 0) {
  [math]::Round(($lineHit / $lineFound) * 100, 2)
} else {
  $null
}

$branchCoveragePercent = if ($branchFound -gt 0) {
  [math]::Round(($branchHit / $branchFound) * 100, 2)
} else {
  $null
}

$summary = [pscustomobject]@{
  checkedAt = (Get-Date).ToString("s")
  command = $commandResult.Command
  passed = ($commandResult.ExitCode -eq 0 -and (Test-Path $lcovPath))
  exitCode = $commandResult.ExitCode
  durationMs = $commandResult.DurationMs
  lcovPath = Get-RepoRelativePath $lcovPath
  lineFound = $lineFound
  lineHit = $lineHit
  lineCoveragePercent = $lineCoveragePercent
  branchFound = $branchFound
  branchHit = $branchHit
  branchCoveragePercent = $branchCoveragePercent
  logPath = Get-RepoRelativePath $logPath
}

$summary | Write-JsonFile -Path $summaryPath

if (-not $summary.passed) {
  throw "coverage run failed, see $($summary.logPath)"
}
