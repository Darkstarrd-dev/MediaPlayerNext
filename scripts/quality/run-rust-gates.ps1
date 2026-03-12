param(
  [string]$OutputRoot,
  [ValidateSet("legacy", "fast", "standard", "heavy")]
  [string]$Layer = "legacy"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

. (Join-Path $PSScriptRoot "common.ps1")

$outputLabel = if ($Layer -eq "legacy") {
  "rust-gates"
} else {
  "rust-gates-$Layer"
}

$context = Initialize-QualityOutputRoot -OutputRoot $OutputRoot -Label $outputLabel
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

function Build-ProfileConfig {
  param([string]$ProfileName)

  switch ($ProfileName) {
    "fast" {
      return [pscustomobject]@{
        runFmt = $true
        runClippy = $false
        runCheck = $true
        nextestMode = "none"
        runCoverage = $false
        runDeny = $false
        runAudit = $false
        runUdeps = $false
        runDebtDelta = $true
        runDuplicateDeps = $false
        runForbiddenEdges = $true
        runTauriBuild = $false
      }
    }
    "standard" {
      return [pscustomobject]@{
        runFmt = $true
        runClippy = $true
        runCheck = $true
        nextestMode = "x1"
        runCoverage = $false
        runDeny = $false
        runAudit = $false
        runUdeps = $false
        runDebtDelta = $true
        runDuplicateDeps = $true
        runForbiddenEdges = $true
        runTauriBuild = $false
      }
    }
    "heavy" {
      return [pscustomobject]@{
        runFmt = $true
        runClippy = $true
        runCheck = $true
        nextestMode = "x3"
        runCoverage = $true
        runDeny = $true
        runAudit = $true
        runUdeps = $true
        runDebtDelta = $true
        runDuplicateDeps = $true
        runForbiddenEdges = $true
        runTauriBuild = $false
      }
    }
    default {
      return [pscustomobject]@{
        runFmt = $true
        runClippy = $true
        runCheck = $true
        nextestMode = "x3"
        runCoverage = $true
        runDeny = $true
        runAudit = $true
        runUdeps = $true
        runDebtDelta = $true
        runDuplicateDeps = $true
        runForbiddenEdges = $true
        runTauriBuild = $true
      }
    }
  }
}

function Invoke-NextestByMode {
  param(
    [string]$Mode,
    [string]$CargoRunnerPath
  )

  $nextestResults = @()

  switch ($Mode) {
    "none" {
      return @($nextestResults)
    }
    "x1" {
      $nextestResults += Invoke-DirectGate -Name "nextest-run" -Priority "P0" -Executable $CargoRunnerPath -Arguments @("nextest", "run", "--workspace", "--all-features") -LogFileName "p0-nextest-run.log" -Details "nextest standard run 1/1"
      return @($nextestResults)
    }
    "x3" {
      $singleRuns = @()
      for ($index = 1; $index -le 3; $index += 1) {
        $singleRuns += Invoke-DirectGate -Name "nextest-run-$index" -Priority "P0" -Executable $CargoRunnerPath -Arguments @("nextest", "run", "--workspace", "--all-features") -LogFileName "p0-nextest-run-$index.log" -Details "nextest flaky replay run $index/3"
      }

      $nextestResults += $singleRuns

      $nextestPassedCount = @($singleRuns | Where-Object { $_.passed }).Count
      $nextestDurationMs = [math]::Round((@($singleRuns | Measure-Object -Property durationMs -Sum).Sum), 3)
      $nextestArtifacts = @()
      for ($index = 1; $index -le 3; $index += 1) {
        $nextestArtifacts += Join-Path $resolvedOutputRoot "p0-nextest-run-$index.log"
      }

      $nextestResults += Add-GateResult `
        -Name "nextest-flaky-x3" `
        -Priority "P0" `
        -Passed ($nextestPassedCount -eq 3) `
        -Command "aggregate" `
        -ExitCode $(if ($nextestPassedCount -eq 3) { 0 } else { 1 }) `
        -DurationMs $nextestDurationMs `
        -LogPath (Join-Path $resolvedOutputRoot "p0-nextest-run-1.log") `
        -Details "$nextestPassedCount/3 passed" `
        -Artifacts $nextestArtifacts

      return @($nextestResults)
    }
    default {
      throw "unsupported nextest mode: $Mode"
    }
  }
}

$gitCommitLogPath = Join-Path $resolvedOutputRoot "baseline-git-commit.log"
$gitStatusLogPath = Join-Path $resolvedOutputRoot "baseline-git-status.log"

$gitCommitResult = Invoke-LoggedCommand -Executable "git" -Arguments @("rev-parse", "--short", "HEAD") -WorkingDirectory $projectRoot -LogPath $gitCommitLogPath
$gitStatusResult = Invoke-LoggedCommand -Executable "git" -Arguments @("status", "--short") -WorkingDirectory $projectRoot -LogPath $gitStatusLogPath

$commit = $gitCommitResult.Output.Trim()
$worktreeStatus = if ([string]::IsNullOrWhiteSpace($gitStatusResult.Output)) { "clean" } else { "dirty" }

$profileConfig = Build-ProfileConfig -ProfileName $Layer

$localAdvisoryDbPath = $null
$advisoryDbRoot = Join-Path $env:USERPROFILE ".cargo\advisory-dbs"
if (Test-Path $advisoryDbRoot) {
  $localAdvisoryDbPath = Get-ChildItem -Path $advisoryDbRoot -Directory -Filter "advisory-db-*" |
    Sort-Object LastWriteTime -Descending |
    Select-Object -First 1 |
    ForEach-Object { $_.FullName }
}

$gateResults = @()
if ($profileConfig.runFmt) {
  $gateResults += Invoke-DirectGate -Name "fmt" -Priority "P0" -Executable $cargoRunner -Arguments @("fmt", "--all", "--check") -LogFileName "p0-fmt.log" -Details "Rust workspace format gate"
}

if ($profileConfig.runClippy) {
  $gateResults += Invoke-DirectGate -Name "clippy" -Priority "P0" -Executable $cargoRunner -Arguments @("clippy", "--workspace", "--all-targets", "--all-features", "--", "-D", "warnings") -LogFileName "p0-clippy.log" -Details "Rust workspace lint gate"
}

if ($profileConfig.runCheck) {
  $gateResults += Invoke-DirectGate -Name "check" -Priority "P0" -Executable $cargoRunner -Arguments @("check", "--workspace", "--all-targets", "--locked") -LogFileName "p0-check.log" -Details "Rust workspace compile gate"
}

if ($profileConfig.nextestMode -ne "none") {
  $gateResults += @(Invoke-NextestByMode -Mode $profileConfig.nextestMode -CargoRunnerPath $cargoRunner)
}

if ($profileConfig.runCoverage) {
  $gateResults += Invoke-PowerShellGate -Name "coverage" -Priority "P0" -ScriptPath (Join-Path $projectRoot "scripts\quality\run-coverage.ps1") -LogFileName "p0-coverage-wrapper.log" -SummaryFileName "coverage-summary.json" -Details "cargo llvm-cov + nextest coverage gate"
}

if ($profileConfig.runDeny) {
  $denyArguments = @("deny", "check", "advisories", "licenses", "bans", "sources")
  if (-not [string]::IsNullOrWhiteSpace($localAdvisoryDbPath)) {
    $denyArguments += "--disable-fetch"
  }

  $gateResults += Invoke-DirectGate -Name "deny" -Priority "P0" -Executable $cargoRunner -Arguments $denyArguments -LogFileName "p0-deny.log" -Details "cargo deny security and license gate"
}

if ($profileConfig.runAudit) {
  $auditArguments = @("audit")
  if (-not [string]::IsNullOrWhiteSpace($localAdvisoryDbPath)) {
    $auditArguments += @("--no-fetch", "--db", $localAdvisoryDbPath)
  }

  $gateResults += Invoke-DirectGate -Name "audit" -Priority "P0" -Executable $cargoRunner -Arguments $auditArguments -LogFileName "p0-audit.log" -Details "cargo audit RustSec gate"
}

if ($profileConfig.runUdeps) {
  $gateResults += Invoke-DirectGate -Name "udeps" -Priority "P2" -Executable $cargoRunner -Arguments @("+nightly", "udeps", "--workspace", "--all-targets") -LogFileName "p2-udeps.log" -Details "unused dependency gate"
}

if ($profileConfig.runDebtDelta) {
  $gateResults += Invoke-PowerShellGate -Name "debt-delta" -Priority "P2" -ScriptPath (Join-Path $projectRoot "scripts\quality\check-debt-delta.ps1") -LogFileName "p2-debt-delta-wrapper.log" -SummaryFileName "debt-delta-summary.json" -Details "Rust debt baseline-delta gate"
}

if ($profileConfig.runDuplicateDeps) {
  $gateResults += Invoke-PowerShellGate -Name "duplicate-deps" -Priority "P2" -ScriptPath (Join-Path $projectRoot "scripts\quality\check-duplicate-deps.ps1") -LogFileName "p2-duplicate-deps-wrapper.log" -SummaryFileName "duplicate-deps-summary.json" -Details "duplicate dependency gate"
}

if ($profileConfig.runForbiddenEdges) {
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
}

if ($profileConfig.runTauriBuild) {
  $gateResults += Invoke-PowerShellGate -Name "tauri-build" -Priority "P1" -ScriptPath (Join-Path $projectRoot "scripts\quality\run-release-verify.ps1") -LogFileName "p1-release-verify-wrapper.log" -SummaryFileName "release-verify-summary.json" -Details "release build and bundle verification gate"
}

$passedCount = @($gateResults | Where-Object { $_.passed }).Count
$failedCount = $gateResults.Count - $passedCount

$summary = [pscustomobject]@{
  generatedAt = (Get-Date).ToString("s")
  layer = $Layer
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
