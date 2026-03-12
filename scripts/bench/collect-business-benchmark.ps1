param(
  [string]$SampleRoot,
  [int]$Runs = 3,
  [string]$OutputPath,
  [string]$LatestPath,
  [string]$HistoryPath,
  [string]$MachineId
)

$ErrorActionPreference = "Stop"

. (Join-Path $PSScriptRoot "common.ps1")

$workspaceRoot = Get-MpNextWorkspaceRoot
$stamp = Get-Date -Format "yyyyMMdd-HHmmss"
$runRoot = if ([string]::IsNullOrWhiteSpace($OutputPath)) {
  Join-Path $workspaceRoot "data\benchmarks\$stamp\business-benchmark"
} else {
  $OutputPath
}
New-Item -ItemType Directory -Force -Path $runRoot | Out-Null

$latestOutputPath = if ([string]::IsNullOrWhiteSpace($LatestPath)) {
  Join-Path $workspaceRoot "docs\benchmarks\business-benchmark-latest.json"
} else {
  $LatestPath
}

$historyOutputPath = if ([string]::IsNullOrWhiteSpace($HistoryPath)) {
  Join-Path $workspaceRoot "docs\benchmarks\business-benchmark-history.json"
} else {
  $HistoryPath
}

if ([string]::IsNullOrWhiteSpace($MachineId)) {
  $MachineId = $env:COMPUTERNAME
}

if ([string]::IsNullOrWhiteSpace($SampleRoot)) {
  $SampleRoot = Join-Path $workspaceRoot "docs\fixtures\medium-fixture\generated-placeholder\scan-root"
}

$scanArchiveOutput = Join-Path $runRoot "scan-archive"
$thumbnailOutput = Join-Path $runRoot "thumbnail"
$playbackOutput = Join-Path $runRoot "playback"
$sidecarOutput = Join-Path $runRoot "sidecar"

& (Join-Path $PSScriptRoot "run-scan-archive-benchmark.ps1") `
  -SampleRoot $SampleRoot `
  -Runs $Runs `
  -OutputPath $scanArchiveOutput | Out-Null

& (Join-Path $PSScriptRoot "run-thumbnail-benchmark.ps1") `
  -SampleRoot $SampleRoot `
  -Runs $Runs `
  -OutputPath $thumbnailOutput | Out-Null

& (Join-Path $PSScriptRoot "run-playback-benchmark.ps1") `
  -SampleRoot $SampleRoot `
  -Runs $Runs `
  -OutputPath $playbackOutput | Out-Null

& (Join-Path $PSScriptRoot "run-sidecar-benchmark.ps1") `
  -Runs $Runs `
  -OutputPath $sidecarOutput | Out-Null

$scanArchivePath = Join-Path $scanArchiveOutput "scan-archive-benchmark.json"
$thumbnailPath = Join-Path $thumbnailOutput "thumbnail-benchmark.json"
$playbackPath = Join-Path $playbackOutput "playback-benchmark.json"
$sidecarPath = Join-Path $sidecarOutput "sidecar-benchmark.json"

$scanArchive = Get-Content -Path $scanArchivePath -Raw -Encoding utf8 | ConvertFrom-Json
$thumbnail = Get-Content -Path $thumbnailPath -Raw -Encoding utf8 | ConvertFrom-Json
$playback = Get-Content -Path $playbackPath -Raw -Encoding utf8 | ConvertFrom-Json
$sidecar = Get-Content -Path $sidecarPath -Raw -Encoding utf8 | ConvertFrom-Json

$latest = [pscustomobject]@{
  runId = $stamp
  recordedAt = (Get-Date).ToString("s")
  machine = $MachineId
  runs = $Runs
  sampleRoot = $SampleRoot
  metrics = [pscustomobject]@{
    scan = [pscustomobject]@{
      firstScanMs = [double]$scanArchive.timings.scanFirst.stats.medianMs
      rescanMs = [double]$scanArchive.timings.scanRescan.stats.medianMs
    }
    archive = [pscustomobject]@{
      indexMs = [double]$scanArchive.timings.archiveIndex.stats.medianMs
      entryReadMs = [double]$scanArchive.timings.archiveReadEntry.stats.medianMs
    }
    thumbnail = [pscustomobject]@{
      ensureMs = [double]$thumbnail.timings.fileCold.stats.medianMs
    }
    playback = [pscustomobject]@{
      probeMs = [double]$playback.timings.ffprobe.stats.medianMs
      sessionOpenMs = [double]$playback.timings.mpvStartup.stats.medianMs
    }
    sidecar = [pscustomobject]@{
      pingMs = [double]$sidecar.timings.ping.stats.medianMs
      healthMs = [double]$sidecar.timings.health.stats.medianMs
    }
  }
  artifacts = [pscustomobject]@{
    scanArchive = (Resolve-Path $scanArchivePath).Path
    thumbnail = (Resolve-Path $thumbnailPath).Path
    playback = (Resolve-Path $playbackPath).Path
    sidecar = (Resolve-Path $sidecarPath).Path
  }
}

Write-MpNextJson -Path $latestOutputPath -Value $latest

$history = if (Test-Path $historyOutputPath) {
  Get-Content -Path $historyOutputPath -Raw -Encoding utf8 | ConvertFrom-Json
} else {
  [pscustomobject]@{
    updatedAt = ""
    entries = @()
  }
}

$entries = @($history.entries)
$entries += $latest
$history = [pscustomobject]@{
  updatedAt = (Get-Date).ToString("s")
  entries = @($entries)
}
Write-MpNextJson -Path $historyOutputPath -Value $history

$collectionPath = Join-Path $runRoot "business-benchmark-collection.json"
Write-MpNextJson -Path $collectionPath -Value $latest

$latest | ConvertTo-Json -Depth 100
