$ErrorActionPreference = "Stop"

function Get-MpNextWorkspaceRoot {
  return Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
}

function Get-MpNextConfig {
  $workspaceRoot = Get-MpNextWorkspaceRoot
  $configPath = Join-Path $workspaceRoot "config\local.paths.json"
  if (-not (Test-Path $configPath)) {
    throw "missing config file: $configPath"
  }

  return Get-Content $configPath -Raw | ConvertFrom-Json
}

function Get-MpNextRuntimePath {
  param(
    [Parameter(Mandatory = $true)]
    [string]$Name,
    [string]$Fallback
  )

  $config = Get-MpNextConfig
  $value = $config.$Name
  if ([string]::IsNullOrWhiteSpace($value)) {
    $value = $Fallback
  }
  if ([string]::IsNullOrWhiteSpace($value)) {
    throw "missing runtime path: $Name"
  }

  return $value
}

function New-MpNextBenchContext {
  param(
    [Parameter(Mandatory = $true)]
    [string]$Name,
    [string]$OutputPath
  )

  $workspaceRoot = Get-MpNextWorkspaceRoot
  $stamp = Get-Date -Format "yyyyMMdd-HHmmss"
  if ([string]::IsNullOrWhiteSpace($OutputPath)) {
    $runRoot = Join-Path $workspaceRoot "data\benchmarks\$stamp\$Name"
  } else {
    $runRoot = $OutputPath
  }

  New-Item -ItemType Directory -Force -Path $runRoot | Out-Null
  $cacheRoot = Join-Path $runRoot "cache"
  $thumbCacheRoot = Join-Path $cacheRoot "thumbs"
  $playbackSessionsRoot = Join-Path $cacheRoot "playback\sessions"
  $normalizeRoot = Join-Path $cacheRoot "normalized"
  $subtitleSessionsRoot = Join-Path $cacheRoot "subtitle\sessions"

  foreach ($path in @($cacheRoot, $thumbCacheRoot, $playbackSessionsRoot, $normalizeRoot, $subtitleSessionsRoot)) {
    New-Item -ItemType Directory -Force -Path $path | Out-Null
  }

  return [pscustomobject]@{
    Name = $Name
    Stamp = $stamp
    WorkspaceRoot = $workspaceRoot
    RunRoot = $runRoot
    DbPath = Join-Path $runRoot "mediaplayernext-bench.db"
    ThumbCacheRoot = $thumbCacheRoot
    PlaybackSessionsRoot = $playbackSessionsRoot
    NormalizeRoot = $normalizeRoot
    SubtitleSessionsRoot = $subtitleSessionsRoot
  }
}

function Build-MpNextBackendHarness {
  $workspaceRoot = Get-MpNextWorkspaceRoot
  $cargoRunner = Join-Path $workspaceRoot "scripts\run-cargo-with-msvc.cmd"
  $manifestPath = Join-Path $workspaceRoot "src-tauri\Cargo.toml"
  & $cargoRunner build --manifest-path $manifestPath --bin backend_harness
  if ($LASTEXITCODE -ne 0) {
    throw "failed to build backend_harness"
  }
}

function Get-MpNextBackendHarnessPath {
  $workspaceRoot = Get-MpNextWorkspaceRoot
  $candidates = @(
    (Join-Path $workspaceRoot "target\debug\backend_harness.exe"),
    (Join-Path $workspaceRoot "src-tauri\target\debug\backend_harness.exe")
  )

  foreach ($candidate in $candidates) {
    if (Test-Path $candidate) {
      return $candidate
    }
  }

  throw "backend_harness.exe not found; run build first"
}

function Invoke-MpNextBackendHarness {
  param(
    [Parameter(Mandatory = $true)]
    $Context,
    [Parameter(Mandatory = $true)]
    [string[]]$Arguments,
    [hashtable]$ExtraEnv
  )

  $envMap = @{
    MPNEXT_BACKEND_DB_PATH = $Context.DbPath
    MPNEXT_BACKEND_THUMB_CACHE_ROOT = $Context.ThumbCacheRoot
    MPNEXT_BACKEND_PLAYBACK_SESSIONS_ROOT = $Context.PlaybackSessionsRoot
    MPNEXT_BACKEND_NORMALIZE_ROOT = $Context.NormalizeRoot
  }
  if ($ExtraEnv) {
    foreach ($key in $ExtraEnv.Keys) {
      $envMap[$key] = $ExtraEnv[$key]
    }
  }

  $previous = @{}
  foreach ($key in $envMap.Keys) {
    $previous[$key] = [Environment]::GetEnvironmentVariable($key, "Process")
    [Environment]::SetEnvironmentVariable($key, [string]$envMap[$key], "Process")
  }

  try {
    $exe = Get-MpNextBackendHarnessPath
    $raw = & $exe @Arguments
    if ($LASTEXITCODE -ne 0) {
      throw "backend_harness failed: $($Arguments -join ' ')"
    }

    return ($raw | Out-String) | ConvertFrom-Json
  }
  finally {
    foreach ($key in $previous.Keys) {
      [Environment]::SetEnvironmentVariable($key, $previous[$key], "Process")
    }
  }
}

function Get-MpNextSampleFile {
  param(
    [Parameter(Mandatory = $true)]
    [string]$Root,
    [Parameter(Mandatory = $true)]
    [string[]]$Extensions
  )

  $normalized = $Extensions | ForEach-Object { $_.ToLowerInvariant() }
  $file = Get-ChildItem -Path $Root -Recurse -File |
    Where-Object { $normalized -contains $_.Extension.ToLowerInvariant() } |
    Sort-Object FullName |
    Select-Object -First 1

  if (-not $file) {
    throw "sample file not found under $Root for extensions: $($Extensions -join ', ')"
  }

  return $file.FullName
}

function ConvertTo-MpNextNormalizedPath {
  param(
    [Parameter(Mandatory = $true)]
    [string]$Path
  )

  return $Path.Replace('\', '/').ToLowerInvariant()
}

function Get-MpNextTimingStats {
  param(
    [Parameter(Mandatory = $true)]
    [double[]]$SamplesMs
  )

  if ($SamplesMs.Count -eq 0) {
    throw "timing samples cannot be empty"
  }

  $sorted = @($SamplesMs | Sort-Object)
  $count = $sorted.Count
  if ($count % 2 -eq 1) {
    $median = $sorted[[int]($count / 2)]
  } else {
    $median = ($sorted[($count / 2) - 1] + $sorted[$count / 2]) / 2
  }

  return [pscustomobject]@{
    runs = $count
    averageMs = [math]::Round((($SamplesMs | Measure-Object -Average).Average), 3)
    medianMs = [math]::Round($median, 3)
    minMs = [math]::Round($sorted[0], 3)
    maxMs = [math]::Round($sorted[$count - 1], 3)
  }
}

function Measure-MpNextOperation {
  param(
    [Parameter(Mandatory = $true)]
    [int]$Runs,
    [Parameter(Mandatory = $true)]
    [scriptblock]$Action,
    [scriptblock]$BeforeEach,
    [scriptblock]$AfterEach
  )

  $samples = New-Object System.Collections.Generic.List[double]
  $results = @()

  for ($index = 0; $index -lt $Runs; $index++) {
    if ($BeforeEach) {
      & $BeforeEach $index
    }

    $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
    $result = & $Action $index
    $stopwatch.Stop()
    $samples.Add($stopwatch.Elapsed.TotalMilliseconds)
    $results += $result

    if ($AfterEach) {
      & $AfterEach $index $result
    }
  }

  return [pscustomobject]@{
    samplesMs = @($samples)
    stats = Get-MpNextTimingStats -SamplesMs @($samples)
    lastResult = $results[-1]
  }
}

function Write-MpNextJson {
  param(
    [Parameter(Mandatory = $true)]
    [string]$Path,
    [Parameter(Mandatory = $true)]
    $Value
  )

  $parent = Split-Path -Parent $Path
  if ($parent) {
    New-Item -ItemType Directory -Force -Path $parent | Out-Null
  }

  $Value | ConvertTo-Json -Depth 100 | Set-Content -Encoding utf8 -Path $Path
}
