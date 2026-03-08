$ErrorActionPreference = "Stop"

$projectRoot = Split-Path -Parent $PSScriptRoot
$configPath = Join-Path $projectRoot "config\local.paths.json"

function Get-ConfigValue {
  param(
    [object]$Config,
    [string]$PropertyName
  )

  if ($null -eq $Config) {
    return $null
  }

  $property = $Config.PSObject.Properties[$PropertyName]
  if ($null -eq $property) {
    return $null
  }

  return $property.Value
}

function Resolve-RequiredPath {
  param(
    [string]$Label,
    [string]$EnvName,
    [string]$ConfigValue
  )

  $envValue = [Environment]::GetEnvironmentVariable($EnvName)
  if (-not [string]::IsNullOrWhiteSpace($envValue)) {
    return $envValue
  }

  if (-not [string]::IsNullOrWhiteSpace($ConfigValue)) {
    return $ConfigValue
  }

  throw "missing runtime path for $Label (set $EnvName or config/local.paths.json)"
}

$config = $null
if (Test-Path $configPath) {
  $config = Get-Content $configPath -Raw | ConvertFrom-Json
}

$cargoRunner = Join-Path $projectRoot "scripts\run-cargo-with-msvc.cmd"

if (-not (Test-Path $cargoRunner)) {
  throw "missing cargo runner: $cargoRunner"
}

$cargoManifestPath = Join-Path $projectRoot "src-tauri\Cargo.toml"

$ffmpegPath = Resolve-RequiredPath -Label "ffmpeg" -EnvName "MPNEXT_RUNTIME_FFMPEG_PATH" -ConfigValue (Get-ConfigValue -Config $config -PropertyName "ffmpeg")
$ffprobePath = Resolve-RequiredPath -Label "ffprobe" -EnvName "MPNEXT_RUNTIME_FFPROBE_PATH" -ConfigValue (Get-ConfigValue -Config $config -PropertyName "ffprobe")
$mpvPath = Resolve-RequiredPath -Label "mpv" -EnvName "MPNEXT_RUNTIME_MPV_PATH" -ConfigValue (Get-ConfigValue -Config $config -PropertyName "mpv")

& $cargoRunner run --manifest-path $cargoManifestPath --bin runtime-smoke-check -- --ffmpeg-path $ffmpegPath --ffprobe-path $ffprobePath --mpv-path $mpvPath
