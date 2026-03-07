$ErrorActionPreference = "Stop"

$projectRoot = Split-Path -Parent $PSScriptRoot
$configPath = Join-Path $projectRoot "config\local.paths.json"

if (-not (Test-Path $configPath)) {
  throw "missing config file: $configPath"
}

$config = Get-Content $configPath -Raw | ConvertFrom-Json
$cargoRunner = Join-Path $projectRoot "scripts\run-cargo-with-msvc.cmd"

if (-not (Test-Path $cargoRunner)) {
  throw "missing cargo runner: $cargoRunner"
}

$cargoManifestPath = Join-Path $projectRoot "src-tauri\Cargo.toml"

& $cargoRunner run --manifest-path $cargoManifestPath --bin runtime-smoke-check -- --ffmpeg-path $config.ffmpeg --ffprobe-path $config.ffprobe --mpv-path $config.mpv
