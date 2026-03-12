param(
  [string]$OutputRoot
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

. (Join-Path $PSScriptRoot "common.ps1")

$context = Initialize-QualityOutputRoot -OutputRoot $OutputRoot -Label "cargo-bloat"
$projectRoot = $context.ProjectRoot
$resolvedOutputRoot = $context.OutputRoot

$baselinePath = Join-Path $projectRoot "config/quality/cargo-bloat-baseline.json"
$summaryPath = Join-Path $resolvedOutputRoot "cargo-bloat-summary.json"
$logPath = Join-Path $resolvedOutputRoot "cargo-bloat.log"
$cargoRunner = Join-Path $projectRoot "scripts/run-cargo-with-msvc.cmd"

if (-not (Test-Path $baselinePath)) {
  throw "cargo bloat baseline missing: $baselinePath"
}

$baseline = Get-Content -Path $baselinePath -Raw -Encoding utf8 | ConvertFrom-Json

function Convert-SizeToBytes {
  param([string]$SizeToken)

  $clean = $SizeToken.Trim()
  if ($clean -match '^(?<value>[0-9]+(?:\.[0-9]+)?)(?<unit>B|KiB|MiB|GiB)$') {
    $value = [double]$Matches.value
    switch ($Matches.unit) {
      "B" { return [long][math]::Round($value) }
      "KiB" { return [long][math]::Round($value * 1024) }
      "MiB" { return [long][math]::Round($value * 1024 * 1024) }
      "GiB" { return [long][math]::Round($value * 1024 * 1024 * 1024) }
    }
  }

  throw "unsupported size token: $SizeToken"
}

$arguments = @(
  "bloat",
  "--release",
  "--manifest-path",
  [string]$baseline.manifestPath,
  "--bin",
  [string]$baseline.bin,
  "--crates",
  "-n",
  "20"
)

$commandResult = Invoke-LoggedCommand -Executable $cargoRunner -Arguments $arguments -WorkingDirectory $projectRoot -LogPath $logPath
if ($commandResult.ExitCode -ne 0) {
  throw "cargo bloat command failed, see $(Get-RepoRelativePath $logPath)"
}

$output = $commandResult.Output
$totalMatch = [regex]::Match($output, '(?m)^[0-9]+(?:\.[0-9]+)?%\s+[0-9]+(?:\.[0-9]+)?%\s+(?<text>[0-9]+(?:\.[0-9]+)?(?:B|KiB|MiB|GiB))\s+\.text section size, the file size is\s+(?<file>[0-9]+(?:\.[0-9]+)?(?:B|KiB|MiB|GiB))$')
if (-not $totalMatch.Success) {
  throw "failed to parse cargo bloat total line"
}

$primaryCrateName = [string]$baseline.primaryCrate
$primaryPattern = '(?m)^(?<filePercent>[0-9]+(?:\.[0-9]+)?)%\s+[0-9]+(?:\.[0-9]+)?%\s+[0-9]+(?:\.[0-9]+)?(?:B|KiB|MiB|GiB)\s+' + [regex]::Escape($primaryCrateName) + '$'
$primaryMatch = [regex]::Match($output, $primaryPattern)

$textBytes = Convert-SizeToBytes $totalMatch.Groups['text'].Value
$fileBytes = Convert-SizeToBytes $totalMatch.Groups['file'].Value
$primaryFilePercent = if ($primaryMatch.Success) { [double]$primaryMatch.Groups['filePercent'].Value } else { $null }

$maxTextBytes = [long]$baseline.maxTextBytes
$maxFileBytes = [long]$baseline.maxFileBytes
$maxPrimaryFilePercent = [double]$baseline.maxPrimaryFilePercent

$textExceeded = $textBytes -gt $maxTextBytes
$fileExceeded = $fileBytes -gt $maxFileBytes
$primaryExceeded = $false
if ($null -ne $primaryFilePercent) {
  $primaryExceeded = $primaryFilePercent -gt $maxPrimaryFilePercent
}

$summary = [pscustomobject]@{
  checkedAt = (Get-Date).ToString("s")
  passed = (-not $textExceeded -and -not $fileExceeded -and -not $primaryExceeded)
  baselinePath = Get-RepoRelativePath $baselinePath
  command = $commandResult.Command
  textBytes = $textBytes
  maxTextBytes = $maxTextBytes
  textExceeded = $textExceeded
  fileBytes = $fileBytes
  maxFileBytes = $maxFileBytes
  fileExceeded = $fileExceeded
  primaryCrate = $primaryCrateName
  primaryFilePercent = $primaryFilePercent
  maxPrimaryFilePercent = $maxPrimaryFilePercent
  primaryExceeded = $primaryExceeded
  logPath = Get-RepoRelativePath $logPath
}

$summary | Write-JsonFile -Path $summaryPath

if (-not $summary.passed) {
  throw "cargo bloat threshold exceeded, see $(Get-RepoRelativePath $summaryPath)"
}
