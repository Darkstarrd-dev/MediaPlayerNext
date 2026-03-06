$ErrorActionPreference = "Stop"

$projectRoot = "C:\opencode\MediaPlayerNext"
$configPath = Join-Path $projectRoot "config\local.paths.json"

if (-not (Test-Path $configPath)) {
  throw "missing config file: $configPath"
}

$config = Get-Content $configPath -Raw | ConvertFrom-Json
$cargoRunner = "C:\opencode\MediaPlayerNext\scripts\run-cargo-with-msvc.cmd"

if (-not (Test-Path $cargoRunner)) {
  throw "missing cargo runner: $cargoRunner"
}

& $cargoRunner run --manifest-path "C:\opencode\MediaPlayerNext\src-tauri\Cargo.toml" --bin runtime-smoke-check -- --ffmpeg-path $config.ffmpeg --mpv-path $config.mpv
