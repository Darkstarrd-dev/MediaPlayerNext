param(
  [string]$SampleRoot,
  [int]$Runs = 3,
  [string]$OutputPath
)

$ErrorActionPreference = "Stop"

. (Join-Path $PSScriptRoot "common.ps1")

$context = New-MpNextBenchContext -Name "scan-archive" -OutputPath $OutputPath
if ([string]::IsNullOrWhiteSpace($SampleRoot)) {
  $SampleRoot = Join-Path $context.WorkspaceRoot "docs\fixtures\medium-fixture\generated-placeholder\scan-root"
}
if (-not (Test-Path $SampleRoot)) {
  throw "sample root does not exist: $SampleRoot"
}

Build-MpNextBackendHarness

$library = Invoke-MpNextBackendHarness -Context $context -Arguments @("scan", "add-library", $SampleRoot)
$libraryId = $library.payload.libraryId

$scanFirst = Measure-MpNextOperation -Runs $Runs -Action {
  Invoke-MpNextBackendHarness -Context $context -Arguments @("scan", "run", $libraryId)
} -BeforeEach {
  param($index)
  if ($index -gt 0) {
    Remove-Item -Recurse -Force -ErrorAction SilentlyContinue $context.DbPath
    $freshLibrary = Invoke-MpNextBackendHarness -Context $context -Arguments @("scan", "add-library", $SampleRoot)
    $script:libraryId = $freshLibrary.payload.libraryId
  }
}

$scanRescan = Measure-MpNextOperation -Runs $Runs -Action {
  Invoke-MpNextBackendHarness -Context $context -Arguments @("scan", "resume", $libraryId)
}

$archiveIndex = Measure-MpNextOperation -Runs $Runs -Action {
  Invoke-MpNextBackendHarness -Context $context -Arguments @("archive", "index", $libraryId)
}

$sources = (Invoke-MpNextBackendHarness -Context $context -Arguments @("scan", "diff", $libraryId)).payload
$archiveSource = $null
$archiveEntry = $null
foreach ($source in ($sources | Where-Object { $_.kind -eq "archive" })) {
  $snapshot = (Invoke-MpNextBackendHarness -Context $context -Arguments @("archive", "show", $source.id)).payload
  if ($snapshot.entries.Count -gt 0) {
    $archiveSource = $source
    $archiveEntry = $snapshot.entries | Select-Object -First 1
    break
  }
}

if (-not $archiveSource -or -not $archiveEntry) {
  throw "archive entry sample not found under $SampleRoot"
}

$archiveRead = Measure-MpNextOperation -Runs $Runs -Action {
  Invoke-MpNextBackendHarness -Context $context -Arguments @(
    "archive",
    "read-entry",
    $archiveSource.id,
    $archiveEntry.entryPath
  )
}

$output = [pscustomobject]@{
  benchmark = "scan-archive"
  generatedAt = (Get-Date).ToString("s")
  sampleRoot = $SampleRoot
  outputRoot = $context.RunRoot
  libraryId = $libraryId
  samples = [pscustomobject]@{
    archiveSourceId = $archiveSource.id
    archiveEntryId = $archiveEntry.id
    archiveEntryPath = $archiveEntry.entryPath
  }
  timings = [pscustomobject]@{
    scanFirst = [pscustomobject]@{
      stats = $scanFirst.stats
      samplesMs = $scanFirst.samplesMs
      lastResult = $scanFirst.lastResult.payload
    }
    scanRescan = [pscustomobject]@{
      stats = $scanRescan.stats
      samplesMs = $scanRescan.samplesMs
      lastResult = $scanRescan.lastResult.payload
    }
    archiveIndex = [pscustomobject]@{
      stats = $archiveIndex.stats
      samplesMs = $archiveIndex.samplesMs
      lastResult = $archiveIndex.lastResult.payload
    }
    archiveReadEntry = [pscustomobject]@{
      stats = $archiveRead.stats
      samplesMs = $archiveRead.samplesMs
      lastResult = $archiveRead.lastResult.payload
    }
  }
}

$resultPath = Join-Path $context.RunRoot "scan-archive-benchmark.json"
Write-MpNextJson -Path $resultPath -Value $output
$output | ConvertTo-Json -Depth 100
