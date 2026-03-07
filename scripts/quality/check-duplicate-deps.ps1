param(
  [string]$OutputRoot
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

. (Join-Path $PSScriptRoot "common.ps1")

$context = Initialize-QualityOutputRoot -OutputRoot $OutputRoot -Label "duplicate-deps"
$projectRoot = $context.ProjectRoot
$resolvedOutputRoot = $context.OutputRoot

$cargoRunner = Join-Path $projectRoot "scripts\run-cargo-with-msvc.cmd"
$logPath = Join-Path $resolvedOutputRoot "duplicate-deps.log"
$summaryPath = Join-Path $resolvedOutputRoot "duplicate-deps-summary.json"

$commandResult = Invoke-LoggedCommand `
  -Executable $cargoRunner `
  -Arguments @("tree", "-d", "--workspace", "--charset", "ascii") `
  -WorkingDirectory $projectRoot `
  -LogPath $logPath

$duplicateHeaders = @()
if ($commandResult.ExitCode -eq 0 -and -not [string]::IsNullOrWhiteSpace($commandResult.Output)) {
  $matches = [regex]::Matches($commandResult.Output, '(?m)^[A-Za-z0-9_.+-]+ v[0-9][^\r\n]*$')
  foreach ($match in $matches) {
    $duplicateHeaders += $match.Value
  }
  $duplicateHeaders = @($duplicateHeaders | Sort-Object -Unique)
}

$summary = [pscustomobject]@{
  checkedAt = (Get-Date).ToString("s")
  command = $commandResult.Command
  passed = ($commandResult.ExitCode -eq 0 -and $duplicateHeaders.Count -eq 0)
  exitCode = $commandResult.ExitCode
  durationMs = $commandResult.DurationMs
  duplicateCount = $duplicateHeaders.Count
  duplicates = @($duplicateHeaders)
  logPath = Get-RepoRelativePath $logPath
}

$summary | Write-JsonFile -Path $summaryPath

if (-not $summary.passed) {
  if ($commandResult.ExitCode -ne 0) {
    throw "duplicate deps check failed, see $($summary.logPath)"
  }

  throw "duplicate deps found: $($duplicateHeaders.Count), see $($summary.logPath)"
}
