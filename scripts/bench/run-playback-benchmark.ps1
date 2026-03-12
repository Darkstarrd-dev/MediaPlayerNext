param(
  [string]$SampleRoot,
  [int]$Runs = 5,
  [string]$OutputPath
)

$ErrorActionPreference = "Stop"

. (Join-Path $PSScriptRoot "common.ps1")

function Invoke-MpNextNativeCapture {
  param(
    [Parameter(Mandatory = $true)]
    [string]$FilePath,
    [Parameter(Mandatory = $true)]
    [string[]]$ArgumentList
  )

  $previousPreference = $global:ErrorActionPreference
  $global:ErrorActionPreference = "Continue"

  try {
    $raw = & $FilePath @ArgumentList 2>&1
    $exitCode = $LASTEXITCODE
    return [pscustomobject]@{
      ExitCode = $exitCode
      Output = (($raw | ForEach-Object { $_.ToString() }) -join [Environment]::NewLine)
    }
  }
  finally {
    $global:ErrorActionPreference = $previousPreference
  }
}

$context = New-MpNextBenchContext -Name "playback" -OutputPath $OutputPath
if ([string]::IsNullOrWhiteSpace($SampleRoot)) {
  $SampleRoot = Join-Path $context.WorkspaceRoot "docs\fixtures\small-fixture\generated-placeholder"
}
if (-not (Test-Path $SampleRoot)) {
  throw "sample root does not exist: $SampleRoot"
}

$ffprobePath = Get-MpNextRuntimePath -Name "ffprobe" -Fallback "C:\Tools\ffmpeg\bin\ffprobe.exe"
$ffmpegPath = Get-MpNextRuntimePath -Name "ffmpeg" -Fallback "C:\Tools\ffmpeg\bin\ffmpeg.exe"
$mpvPath = Get-MpNextRuntimePath -Name "mpv" -Fallback "C:\mpv\mpv.exe"
foreach ($runtimePath in @($ffprobePath, $ffmpegPath, $mpvPath)) {
  if (-not (Test-Path $runtimePath)) {
    throw "runtime not found: $runtimePath"
  }
}

$videoSample = Get-MpNextSampleFile -Root $SampleRoot -Extensions @(".mp4")
$framePath = Join-Path $context.RunRoot "frame.png"

$ffprobe = Measure-MpNextOperation -Runs $Runs -Action {
  $result = Invoke-MpNextNativeCapture -FilePath $ffprobePath -ArgumentList @("-v", "quiet", "-print_format", "json", "-show_streams", "-show_format", $videoSample)
  if ($result.ExitCode -ne 0) {
    throw "ffprobe failed for $videoSample"
  }

  return ($result.Output | ConvertFrom-Json)
}

$ffmpeg = Measure-MpNextOperation -Runs $Runs -BeforeEach {
  if (Test-Path $framePath) {
    Remove-Item -Force $framePath
  }
} -Action {
  $result = Invoke-MpNextNativeCapture -FilePath $ffmpegPath -ArgumentList @("-v", "quiet", "-y", "-ss", "0", "-i", $videoSample, "-frames:v", "1", "-update", "1", $framePath)
  if ($result.ExitCode -ne 0) {
    throw "ffmpeg extract frame failed for $videoSample"
  }

  return [pscustomobject]@{
    progress = $result.Output
    outputExists = (Test-Path $framePath)
    outputBytes = if (Test-Path $framePath) { (Get-Item $framePath).Length } else { 0 }
  }
}

$mpv = Measure-MpNextOperation -Runs $Runs -Action {
  $result = Invoke-MpNextNativeCapture -FilePath $mpvPath -ArgumentList @("--no-config", "--really-quiet", "--vo=null", "--ao=null", "--frames=1", $videoSample)

  return [pscustomobject]@{
    exitCode = $result.ExitCode
    output = $result.Output
  }
}

$output = [pscustomobject]@{
  benchmark = "playback"
  generatedAt = (Get-Date).ToString("s")
  outputRoot = $context.RunRoot
  runtimes = [pscustomobject]@{
    ffprobe = $ffprobePath
    ffmpeg = $ffmpegPath
    mpv = $mpvPath
  }
  sample = [pscustomobject]@{
    videoPath = $videoSample
  }
  timings = [pscustomobject]@{
    ffprobe = [pscustomobject]@{
      stats = $ffprobe.stats
      samplesMs = $ffprobe.samplesMs
      lastResult = $ffprobe.lastResult
    }
    ffmpeg = [pscustomobject]@{
      stats = $ffmpeg.stats
      samplesMs = $ffmpeg.samplesMs
      lastResult = $ffmpeg.lastResult
    }
    mpvStartup = [pscustomobject]@{
      stats = $mpv.stats
      samplesMs = $mpv.samplesMs
      lastResult = $mpv.lastResult
    }
  }
}

$resultPath = Join-Path $context.RunRoot "playback-benchmark.json"
Write-MpNextJson -Path $resultPath -Value $output
$output | ConvertTo-Json -Depth 100
