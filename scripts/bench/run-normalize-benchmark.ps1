param(
  [string]$SampleRoot,
  [int]$Runs = 3,
  [string]$OutputPath
)

$ErrorActionPreference = "Stop"

. (Join-Path $PSScriptRoot "common.ps1")

$context = New-MpNextBenchContext -Name "normalize" -OutputPath $OutputPath
if ([string]::IsNullOrWhiteSpace($SampleRoot)) {
  $SampleRoot = Join-Path $context.WorkspaceRoot "docs\fixtures\small-fixture\generated-placeholder"
}
if (-not (Test-Path $SampleRoot)) {
  throw "sample root does not exist: $SampleRoot"
}

$sevenzPath = Get-MpNextRuntimePath -Name "sevenz" -Fallback "C:\Program Files\7-Zip\7z.exe"
if (-not (Test-Path $sevenzPath)) {
  throw "7z runtime not found: $sevenzPath"
}

Build-MpNextBackendHarness

$seedDir = Join-Path $context.RunRoot "seed-input"
New-Item -ItemType Directory -Force -Path $seedDir | Out-Null
$seedFiles = Get-ChildItem -Path $SampleRoot -Recurse -File |
  Where-Object { @(".jpg", ".jpeg", ".png", ".webp") -contains $_.Extension.ToLowerInvariant() } |
  Sort-Object FullName |
  Select-Object -First 4
if ($seedFiles.Count -lt 1) {
  throw "no image samples available for normalize benchmark"
}
foreach ($file in $seedFiles) {
  Copy-Item -Force $file.FullName (Join-Path $seedDir $file.Name)
}

$archivePath = Join-Path $context.RunRoot "benchmark-input.7z"
& $sevenzPath a -t7z $archivePath "$seedDir\*" | Out-Null
if ($LASTEXITCODE -ne 0 -or -not (Test-Path $archivePath)) {
  throw "failed to create benchmark .7z archive: $archivePath"
}

$libraryRoot = Join-Path $context.RunRoot "library"
New-Item -ItemType Directory -Force -Path $libraryRoot | Out-Null
$libraryArchive = Join-Path $libraryRoot "benchmark-input.7z"
Copy-Item -Force $archivePath $libraryArchive

$library = Invoke-MpNextBackendHarness -Context $context -Arguments @("scan", "add-library", $libraryRoot)
$libraryId = $library.payload.libraryId
[void](Invoke-MpNextBackendHarness -Context $context -Arguments @("scan", "run", $libraryId))

$sources = (Invoke-MpNextBackendHarness -Context $context -Arguments @("scan", "diff", $libraryId)).payload
$normalizedArchivePath = ConvertTo-MpNextNormalizedPath -Path $libraryArchive
$archiveSource = $sources | Where-Object { $_.normalizedPath -eq $normalizedArchivePath } | Select-Object -First 1
if (-not $archiveSource) {
  throw "normalize benchmark source not found: $libraryArchive"
}

$normalize = Measure-MpNextOperation -Runs $Runs -BeforeEach {
  if (Test-Path $context.NormalizeRoot) {
    Remove-Item -Recurse -Force $context.NormalizeRoot
  }
  New-Item -ItemType Directory -Force -Path $context.NormalizeRoot | Out-Null
} -Action {
  Invoke-MpNextBackendHarness -Context $context -Arguments @("archive", "normalize", $archiveSource.id)
}

$output = [pscustomobject]@{
  benchmark = "normalize"
  generatedAt = (Get-Date).ToString("s")
  outputRoot = $context.RunRoot
  sevenzPath = $sevenzPath
  sampleArchive = [pscustomobject]@{
    sourcePath = $libraryArchive
    sourceId = $archiveSource.id
    seedFiles = @($seedFiles | ForEach-Object { $_.FullName })
  }
  timings = [pscustomobject]@{
    normalize = [pscustomobject]@{
      stats = $normalize.stats
      samplesMs = $normalize.samplesMs
      lastResult = $normalize.lastResult.payload
    }
  }
}

$resultPath = Join-Path $context.RunRoot "normalize-benchmark.json"
Write-MpNextJson -Path $resultPath -Value $output
$output | ConvertTo-Json -Depth 100
