param(
  [string]$OutputRoot
)

$ErrorActionPreference = "Stop"

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

function Resolve-PathCandidate {
  param(
    [string]$Label,
    [string]$EnvName,
    [string]$ConfigValue,
    [string]$DefaultPath,
    [string]$CommandName,
    [switch]$AllowCommandLookup
  )

  $envValue = [Environment]::GetEnvironmentVariable($EnvName)
  if (-not [string]::IsNullOrWhiteSpace($envValue)) {
    return [pscustomobject]@{
      label = $Label
      source = "env"
      value = $envValue
      exists = Test-Path $envValue
    }
  }

  if (-not [string]::IsNullOrWhiteSpace($ConfigValue)) {
    return [pscustomobject]@{
      label = $Label
      source = "config"
      value = $ConfigValue
      exists = Test-Path $ConfigValue
    }
  }

  if ($AllowCommandLookup -and -not [string]::IsNullOrWhiteSpace($CommandName)) {
    $command = Get-Command $CommandName -ErrorAction SilentlyContinue
    if ($null -ne $command) {
      return [pscustomobject]@{
        label = $Label
        source = "path"
        value = $command.Source
        exists = Test-Path $command.Source
      }
    }
  }

  return [pscustomobject]@{
    label = $Label
    source = "default"
    value = $DefaultPath
    exists = $(if ([string]::IsNullOrWhiteSpace($DefaultPath)) { $false } else { Test-Path $DefaultPath })
  }
}

function New-RelativePath {
  param(
    [string]$ProjectRoot,
    [string]$PathValue
  )

  if ([string]::IsNullOrWhiteSpace($PathValue)) {
    return $PathValue
  }

  $absolute = [System.IO.Path]::GetFullPath($PathValue)
  if ($absolute.StartsWith($ProjectRoot, [System.StringComparison]::OrdinalIgnoreCase)) {
    return $absolute.Substring($ProjectRoot.Length).TrimStart('\\')
  }

  return $absolute
}

$projectRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$configPath = Join-Path $projectRoot "config\local.paths.json"
$defaultOutputRoot = Join-Path $projectRoot ("data\resource-paths\" + (Get-Date -Format "yyyyMMdd-HHmmss"))
$resolvedOutputRoot = if ([string]::IsNullOrWhiteSpace($OutputRoot)) { $defaultOutputRoot } else { $OutputRoot }

if (-not (Test-Path $resolvedOutputRoot)) {
  New-Item -ItemType Directory -Path $resolvedOutputRoot -Force | Out-Null
}

$config = $null
if (Test-Path $configPath) {
  $config = Get-Content $configPath -Raw | ConvertFrom-Json
}

$sidecarEntryDefault = Join-Path $projectRoot "apps\subtitle-sidecar\dist\src\index.js"
$subtitleSessionsDefault = Join-Path $projectRoot "data\cache\subtitle\sessions"
$backendDbDefault = Join-Path $projectRoot "data\mediaplayernext-dev.db"
$thumbCacheDefault = Join-Path $projectRoot "data\cache\thumbs"
$playbackSessionsDefault = Join-Path $projectRoot "data\cache\playback\sessions"
$normalizeRootDefault = Join-Path $projectRoot "data\cache\normalized"

$checks = @(
  (Resolve-PathCandidate -Label "ffmpeg" -EnvName "MPNEXT_RUNTIME_FFMPEG_PATH" -ConfigValue (Get-ConfigValue $config "ffmpeg") -DefaultPath "" -CommandName "ffmpeg" -AllowCommandLookup),
  (Resolve-PathCandidate -Label "ffprobe" -EnvName "MPNEXT_RUNTIME_FFPROBE_PATH" -ConfigValue (Get-ConfigValue $config "ffprobe") -DefaultPath "C:/Tools/ffmpeg/bin/ffprobe.exe" -CommandName "ffprobe" -AllowCommandLookup),
  (Resolve-PathCandidate -Label "mpv" -EnvName "MPNEXT_RUNTIME_MPV_PATH" -ConfigValue (Get-ConfigValue $config "mpv") -DefaultPath "C:/mpv/mpv.exe" -CommandName "mpv" -AllowCommandLookup),
  (Resolve-PathCandidate -Label "sevenz" -EnvName "MPNEXT_RUNTIME_SEVENVZ_PATH" -ConfigValue (Get-ConfigValue $config "sevenz") -DefaultPath "C:/Program Files/7-Zip/7z.exe" -CommandName "7z" -AllowCommandLookup),
  (Resolve-PathCandidate -Label "subtitleNode" -EnvName "MPNEXT_SUBTITLE_NODE_PATH" -ConfigValue (Get-ConfigValue $config "node") -DefaultPath "" -CommandName "node" -AllowCommandLookup),
  (Resolve-PathCandidate -Label "subtitleEntry" -EnvName "MPNEXT_SUBTITLE_ENTRY_PATH" -ConfigValue $null -DefaultPath $sidecarEntryDefault -CommandName ""),
  (Resolve-PathCandidate -Label "subtitleSessionsRoot" -EnvName "MPNEXT_SUBTITLE_SESSIONS_ROOT" -ConfigValue $null -DefaultPath $subtitleSessionsDefault -CommandName ""),
  (Resolve-PathCandidate -Label "backendDbPath" -EnvName "MPNEXT_BACKEND_DB_PATH" -ConfigValue $null -DefaultPath $backendDbDefault -CommandName ""),
  (Resolve-PathCandidate -Label "backendThumbCacheRoot" -EnvName "MPNEXT_BACKEND_THUMB_CACHE_ROOT" -ConfigValue $null -DefaultPath $thumbCacheDefault -CommandName ""),
  (Resolve-PathCandidate -Label "backendPlaybackSessionsRoot" -EnvName "MPNEXT_BACKEND_PLAYBACK_SESSIONS_ROOT" -ConfigValue $null -DefaultPath $playbackSessionsDefault -CommandName ""),
  (Resolve-PathCandidate -Label "backendNormalizeRoot" -EnvName "MPNEXT_BACKEND_NORMALIZE_ROOT" -ConfigValue $null -DefaultPath $normalizeRootDefault -CommandName "")
)

$requiredMissing = @(
  $checks | Where-Object {
    $_.label -in @("ffmpeg", "ffprobe", "mpv", "sevenz", "subtitleNode", "subtitleEntry") -and -not $_.exists
  }
)

$summary = [pscustomobject]@{
  checkedAt = (Get-Date).ToString("s")
  configPath = $(if (Test-Path $configPath) { "config/local.paths.json" } else { $null })
  passed = ($requiredMissing.Count -eq 0)
  requiredMissing = @($requiredMissing | ForEach-Object {
    [pscustomobject]@{
      label = $_.label
      source = $_.source
      value = New-RelativePath -ProjectRoot $projectRoot -PathValue $_.value
    }
  })
  resolved = @($checks | ForEach-Object {
    [pscustomobject]@{
      label = $_.label
      source = $_.source
      value = New-RelativePath -ProjectRoot $projectRoot -PathValue $_.value
      exists = $_.exists
    }
  })
  packagedStrategy = [pscustomobject]@{
    node = "env override first, then PATH node; bundled node not landed yet"
    sidecarEntry = "env override first, then future bundled resource path"
    runtimes = "env override first; bundled ffmpeg/ffprobe/mpv/7z path still pending"
    migrations = "embedded in media-db crate; no external path lookup"
  }
}

$summaryPath = Join-Path $resolvedOutputRoot "resource-paths-summary.json"
$summary | ConvertTo-Json -Depth 6 | Set-Content -Path $summaryPath -Encoding utf8

if (-not $summary.passed) {
  throw "resource path verification failed, see $summaryPath"
}

Write-Output "resource path verification passed: $summaryPath"
