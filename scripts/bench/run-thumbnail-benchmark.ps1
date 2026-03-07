param(
  [string]$SampleRoot,
  [int]$Runs = 5,
  [string]$OutputPath
)

$ErrorActionPreference = "Stop"

. (Join-Path $PSScriptRoot "common.ps1")

$context = New-MpNextBenchContext -Name "thumbnail" -OutputPath $OutputPath
if ([string]::IsNullOrWhiteSpace($SampleRoot)) {
  $SampleRoot = Join-Path $context.WorkspaceRoot "docs\fixtures\medium-fixture\generated-placeholder\scan-root"
}
if (-not (Test-Path $SampleRoot)) {
  throw "sample root does not exist: $SampleRoot"
}

Build-MpNextBackendHarness

$fileSample = Get-MpNextSampleFile -Root $SampleRoot -Extensions @(".jpg", ".jpeg", ".png", ".webp")
$library = Invoke-MpNextBackendHarness -Context $context -Arguments @("scan", "add-library", $SampleRoot)
$libraryId = $library.payload.libraryId
[void](Invoke-MpNextBackendHarness -Context $context -Arguments @("scan", "run", $libraryId))
[void](Invoke-MpNextBackendHarness -Context $context -Arguments @("archive", "index", $libraryId))
[void](Invoke-MpNextBackendHarness -Context $context -Arguments @("asset", "ensure", $libraryId))

$sources = (Invoke-MpNextBackendHarness -Context $context -Arguments @("scan", "diff", $libraryId)).payload
$assets = (Invoke-MpNextBackendHarness -Context $context -Arguments @("asset", "snapshot", $libraryId)).payload

$normalizedFileSample = ConvertTo-MpNextNormalizedPath -Path $fileSample
$fileSource = $sources | Where-Object { $_.normalizedPath -eq $normalizedFileSample } | Select-Object -First 1
if (-not $fileSource) {
  throw "file source not found in scan snapshot: $fileSample"
}
$fileAsset = $assets | Where-Object {
  $_.sourceKind -eq "file" -and $_.sourceId -eq $fileSource.id
} | Select-Object -First 1
if (-not $fileAsset) {
  throw "file asset not found for source: $($fileSource.id)"
}

$archiveSource = $null
$archiveSnapshot = $null
$archiveEntry = $null
foreach ($candidate in ($sources | Where-Object { $_.kind -eq "archive" })) {
  $snapshot = (Invoke-MpNextBackendHarness -Context $context -Arguments @("archive", "show", $candidate.id)).payload
  if ($snapshot.entries.Count -gt 0) {
    $archiveSource = $candidate
    $archiveSnapshot = $snapshot
    $archiveEntry = $snapshot.entries | Select-Object -First 1
    break
  }
}
if (-not $archiveEntry) {
  throw "archive entry benchmark sample not found under $SampleRoot"
}
$archiveAsset = $assets | Where-Object {
  $_.sourceKind -eq "archive_entry" -and $_.sourceRefId -eq $archiveEntry.id
} | Select-Object -First 1
if (-not $archiveAsset) {
  throw "archive asset not found for entry: $($archiveEntry.id)"
}

$seedFile = Invoke-MpNextBackendHarness -Context $context -Arguments @("thumbnail", "ensure", $fileAsset.assetId, "grid-sm")
if (Test-Path $seedFile.payload.diskPath) {
  Remove-Item -Force $seedFile.payload.diskPath
}
$fileCold = Measure-MpNextOperation -Runs $Runs -Action {
  Invoke-MpNextBackendHarness -Context $context -Arguments @("thumbnail", "ensure", $fileAsset.assetId, "grid-sm")
} -AfterEach {
  param($index, $result)
  if (Test-Path $result.payload.diskPath) {
    Remove-Item -Force $result.payload.diskPath
  }
}

$fileHotSeed = Invoke-MpNextBackendHarness -Context $context -Arguments @("thumbnail", "ensure", $fileAsset.assetId, "grid-sm")
$fileHot = Measure-MpNextOperation -Runs $Runs -Action {
  Invoke-MpNextBackendHarness -Context $context -Arguments @("thumbnail", "ensure", $fileAsset.assetId, "grid-sm")
}

$seedArchive = Invoke-MpNextBackendHarness -Context $context -Arguments @("thumbnail", "ensure", $archiveAsset.assetId, "grid-sm")
if (Test-Path $seedArchive.payload.diskPath) {
  Remove-Item -Force $seedArchive.payload.diskPath
}
$archiveCold = Measure-MpNextOperation -Runs $Runs -Action {
  Invoke-MpNextBackendHarness -Context $context -Arguments @("thumbnail", "ensure", $archiveAsset.assetId, "grid-sm")
} -AfterEach {
  param($index, $result)
  if (Test-Path $result.payload.diskPath) {
    Remove-Item -Force $result.payload.diskPath
  }
}

$output = [pscustomobject]@{
  benchmark = "thumbnail"
  generatedAt = (Get-Date).ToString("s")
  sampleRoot = $SampleRoot
  outputRoot = $context.RunRoot
  libraryId = $libraryId
  samples = [pscustomobject]@{
    file = [pscustomobject]@{
      path = $fileSample
      sourceId = $fileSource.id
      assetId = $fileAsset.assetId
    }
    archiveEntry = [pscustomobject]@{
      archivePath = $archiveSource.normalizedPath
      sourceId = $archiveSource.id
      archiveEntryId = $archiveEntry.id
      assetId = $archiveAsset.assetId
      entryPath = $archiveEntry.entryPath
    }
  }
  timings = [pscustomobject]@{
    fileCold = [pscustomobject]@{
      stats = $fileCold.stats
      samplesMs = $fileCold.samplesMs
      lastResult = $fileCold.lastResult.payload
    }
    fileHot = [pscustomobject]@{
      stats = $fileHot.stats
      samplesMs = $fileHot.samplesMs
      lastResult = $fileHot.lastResult.payload
      seededResult = $fileHotSeed.payload
    }
    archiveCold = [pscustomobject]@{
      stats = $archiveCold.stats
      samplesMs = $archiveCold.samplesMs
      lastResult = $archiveCold.lastResult.payload
    }
  }
}

$resultPath = Join-Path $context.RunRoot "thumbnail-benchmark.json"
Write-MpNextJson -Path $resultPath -Value $output
$output | ConvertTo-Json -Depth 100
