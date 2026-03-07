param(
  [string]$OutputRoot
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

. (Join-Path $PSScriptRoot "common.ps1")

$context = Initialize-QualityOutputRoot -OutputRoot $OutputRoot -Label "rust-gates"
$projectRoot = $context.ProjectRoot
$resolvedOutputRoot = $context.OutputRoot

$cargoRunner = Join-Path $projectRoot "scripts\run-cargo-with-msvc.cmd"
$cargoExe = Join-Path $env:USERPROFILE ".cargo\bin\cargo.exe"
$nodeScriptPath = Join-Path $projectRoot "scripts\quality\check-forbidden-edges.mjs"

function Add-GateResult {
  param(
    [string]$Name,
    [string]$Priority,
    [bool]$Passed,
    [string]$Command,
    [int]$ExitCode,
    [double]$DurationMs,
    [string]$LogPath,
    [string]$Details,
    [string[]]$Artifacts = @()
  )

  return [pscustomobject]@{
    name = $Name
    priority = $Priority
    passed = $Passed
    command = $Command
    exitCode = $ExitCode
    durationMs = $DurationMs
    logPath = Get-RepoRelativePath $LogPath
    details = $Details
    artifacts = @($Artifacts | Where-Object { -not [string]::IsNullOrWhiteSpace($_) } | ForEach-Object {
      Get-RepoRelativePath $_
    })
  }
}

function Invoke-DirectGate {
  param(
    [string]$Name,
    [string]$Priority,
    [string]$Executable,
    [string[]]$Arguments,
    [string]$LogFileName,
    [string]$Details
  )

  $logPath = Join-Path $resolvedOutputRoot $LogFileName
  $commandResult = Invoke-LoggedCommand -Executable $Executable -Arguments $Arguments -WorkingDirectory $projectRoot -LogPath $logPath

  return Add-GateResult `
    -Name $Name `
    -Priority $Priority `
    -Passed ($commandResult.ExitCode -eq 0) `
    -Command $commandResult.Command `
    -ExitCode $commandResult.ExitCode `
    -DurationMs $commandResult.DurationMs `
    -LogPath $logPath `
    -Details $Details
}

function Invoke-PowerShellGate {
  param(
    [string]$Name,
    [string]$Priority,
    [string]$ScriptPath,
    [string]$LogFileName,
    [string]$SummaryFileName,
    [string]$Details
  )

  $logPath = Join-Path $resolvedOutputRoot $LogFileName
  $summaryPath = Join-Path $resolvedOutputRoot $SummaryFileName
  $commandResult = Invoke-LoggedCommand `
    -Executable "powershell" `
    -Arguments @(
      "-NoProfile",
      "-ExecutionPolicy",
      "Bypass",
      "-File",
      $ScriptPath,
      "-OutputRoot",
      $resolvedOutputRoot
    ) `
    -WorkingDirectory $projectRoot `
    -LogPath $logPath

  return Add-GateResult `
    -Name $Name `
    -Priority $Priority `
    -Passed ($commandResult.ExitCode -eq 0) `
    -Command $commandResult.Command `
    -ExitCode $commandResult.ExitCode `
    -DurationMs $commandResult.DurationMs `
    -LogPath $logPath `
    -Details $Details `
    -Artifacts @($summaryPath)
}

$gitCommitLogPath = Join-Path $resolvedOutputRoot "baseline-git-commit.log"
$gitStatusLogPath = Join-Path $resolvedOutputRoot "baseline-git-status.log"

$gitCommitResult = Invoke-LoggedCommand -Executable "git" -Arguments @("rev-parse", "--short", "HEAD") -WorkingDirectory $projectRoot -LogPath $gitCommitLogPath
$gitStatusResult = Invoke-LoggedCommand -Executable "git" -Arguments @("status", "--short") -WorkingDirectory $projectRoot -LogPath $gitStatusLogPath

$commit = $gitCommitResult.Output.Trim()
$worktreeStatus = if ([string]::IsNullOrWhiteSpace($gitStatusResult.Output)) { "clean" } else { "dirty" }

$gateResults = @()
$gateResults += Invoke-DirectGate -Name "fmt" -Priority "P0" -Executable $cargoRunner -Arguments @("fmt", "--all", "--check") -LogFileName "p0-fmt.log" -Details "Rust workspace format gate"
$gateResults += Invoke-DirectGate -Name "clippy" -Priority "P0" -Executable $cargoRunner -Arguments @("clippy", "--workspace", "--all-targets", "--all-features", "--", "-D", "warnings") -LogFileName "p0-clippy.log" -Details "Rust workspace lint gate"
$gateResults += Invoke-DirectGate -Name "check" -Priority "P0" -Executable $cargoRunner -Arguments @("check", "--workspace", "--all-targets", "--locked") -LogFileName "p0-check.log" -Details "Rust workspace compile gate"

$nextestRuns = @()
for ($index = 1; $index -le 3; $index += 1) {
  $nextestRuns += Invoke-DirectGate -Name "nextest-run-$index" -Priority "P0" -Executable $cargoRunner -Arguments @("nextest", "run", "--workspace", "--all-features") -LogFileName "p0-nextest-run-$index.log" -Details "nextest flaky replay run $index/3"
}
$gateResults += $nextestRuns

$nextestPassedCount = @($nextestRuns | Where-Object { $_.passed }).Count
$nextestDurationMs = [math]::Round((@($nextestRuns | Measure-Object -Property durationMs -Sum).Sum), 3)
$nextestArtifacts = @()
for ($index = 1; $index -le 3; $index += 1) {
  $nextestArtifacts += Join-Path $resolvedOutputRoot "p0-nextest-run-$index.log"
}
$gateResults += Add-GateResult `
  -Name "nextest-flaky-x3" `
  -Priority "P0" `
  -Passed ($nextestPassedCount -eq 3) `
  -Command "aggregate" `
  -ExitCode $(if ($nextestPassedCount -eq 3) { 0 } else { 1 }) `
  -DurationMs $nextestDurationMs `
  -LogPath (Join-Path $resolvedOutputRoot "p0-nextest-run-1.log") `
  -Details "$nextestPassedCount/3 passed" `
  -Artifacts $nextestArtifacts

$gateResults += Invoke-PowerShellGate -Name "coverage" -Priority "P0" -ScriptPath (Join-Path $projectRoot "scripts\quality\run-coverage.ps1") -LogFileName "p0-coverage-wrapper.log" -SummaryFileName "coverage-summary.json" -Details "cargo llvm-cov + nextest coverage gate"
$gateResults += Invoke-DirectGate -Name "deny" -Priority "P0" -Executable $cargoRunner -Arguments @("deny", "check", "advisories", "licenses", "bans", "sources") -LogFileName "p0-deny.log" -Details "cargo deny security and license gate"
$gateResults += Invoke-DirectGate -Name "audit" -Priority "P0" -Executable $cargoRunner -Arguments @("audit") -LogFileName "p0-audit.log" -Details "cargo audit RustSec gate"
$gateResults += Invoke-DirectGate -Name "udeps" -Priority "P2" -Executable $cargoRunner -Arguments @("+nightly", "udeps", "--workspace", "--all-targets") -LogFileName "p2-udeps.log" -Details "unused dependency gate"
$gateResults += Invoke-PowerShellGate -Name "duplicate-deps" -Priority "P2" -ScriptPath (Join-Path $projectRoot "scripts\quality\check-duplicate-deps.ps1") -LogFileName "p2-duplicate-deps-wrapper.log" -SummaryFileName "duplicate-deps-summary.json" -Details "duplicate dependency gate"

$metadataLogPath = Join-Path $resolvedOutputRoot "p1-cargo-metadata.log"
$metadataPath = Join-Path $resolvedOutputRoot "cargo-metadata.json"
$metadataCommandResult = Invoke-LoggedCommand -Executable $cargoExe -Arguments @("metadata", "--format-version", "1") -WorkingDirectory $projectRoot -LogPath $metadataLogPath
if ($metadataCommandResult.ExitCode -eq 0) {
  $utf8NoBom = New-Object System.Text.UTF8Encoding($false)
  [System.IO.File]::WriteAllText($metadataPath, $metadataCommandResult.Output, $utf8NoBom)
}
$forbiddenSummaryPath = Join-Path $resolvedOutputRoot "forbidden-edges-summary.json"
$forbiddenLogPath = Join-Path $resolvedOutputRoot "p1-forbidden-edges.log"
if ($metadataCommandResult.ExitCode -eq 0) {
  $forbiddenCommandResult = Invoke-LoggedCommand `
    -Executable "node" `
    -Arguments @($nodeScriptPath, "--metadata", $metadataPath, "--summary", $forbiddenSummaryPath) `
    -WorkingDirectory $projectRoot `
    -LogPath $forbiddenLogPath
  $gateResults += Add-GateResult `
    -Name "forbidden-edges" `
    -Priority "P1" `
    -Passed ($forbiddenCommandResult.ExitCode -eq 0) `
    -Command $forbiddenCommandResult.Command `
    -ExitCode $forbiddenCommandResult.ExitCode `
    -DurationMs $forbiddenCommandResult.DurationMs `
    -LogPath $forbiddenLogPath `
    -Details "workspace dependency direction gate" `
    -Artifacts @($metadataPath, $forbiddenSummaryPath)
} else {
  $gateResults += Add-GateResult `
    -Name "forbidden-edges" `
    -Priority "P1" `
    -Passed $false `
    -Command $metadataCommandResult.Command `
    -ExitCode $metadataCommandResult.ExitCode `
    -DurationMs $metadataCommandResult.DurationMs `
    -LogPath $metadataLogPath `
    -Details "cargo metadata failed before forbidden edge check"
}

$gateResults += Invoke-PowerShellGate -Name "tauri-build" -Priority "P1" -ScriptPath (Join-Path $projectRoot "scripts\quality\run-release-verify.ps1") -LogFileName "p1-release-verify-wrapper.log" -SummaryFileName "release-verify-summary.json" -Details "release build and bundle verification gate"

$passedCount = @($gateResults | Where-Object { $_.passed }).Count
$failedCount = $gateResults.Count - $passedCount

$summary = [pscustomobject]@{
  generatedAt = (Get-Date).ToString("s")
  commit = $commit
  worktreeStatus = $worktreeStatus
  outputRoot = Get-RepoRelativePath $resolvedOutputRoot
  baseline = [pscustomobject]@{
    gitCommitLog = Get-RepoRelativePath $gitCommitLogPath
    gitStatusLog = Get-RepoRelativePath $gitStatusLogPath
  }
  totals = [pscustomobject]@{
    gateCount = $gateResults.Count
    passedCount = $passedCount
    failedCount = $failedCount
  }
  gates = $gateResults
}

$summaryPath = Join-Path $resolvedOutputRoot "quality-gates-summary.json"
$summary | Write-JsonFile -Path $summaryPath

$failedGateNames = @($gateResults | Where-Object { -not $_.passed } | ForEach-Object { $_.name })
if ($failedGateNames.Count -gt 0) {
  Write-Error ("quality gates failed: " + ($failedGateNames -join ", "))
  exit 1
}
