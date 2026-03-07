Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Get-QualityProjectRoot {
  return Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
}

function Initialize-QualityOutputRoot {
  param(
    [string]$OutputRoot,
    [string]$Label = "run"
  )

  $projectRoot = Get-QualityProjectRoot
  if ([string]::IsNullOrWhiteSpace($OutputRoot)) {
    $timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
    $resolvedOutputRoot = Join-Path $projectRoot (Join-Path "data\quality-gates\$timestamp" $Label)
  } elseif ([System.IO.Path]::IsPathRooted($OutputRoot)) {
    $resolvedOutputRoot = $OutputRoot
  } else {
    $resolvedOutputRoot = Join-Path $projectRoot $OutputRoot
  }

  New-Item -ItemType Directory -Force -Path $resolvedOutputRoot | Out-Null

  return [pscustomobject]@{
    ProjectRoot = $projectRoot
    OutputRoot = $resolvedOutputRoot
  }
}

function Get-RepoRelativePath {
  param([string]$Path)

  if ([string]::IsNullOrWhiteSpace($Path)) {
    return $Path
  }

  $projectRoot = Get-QualityProjectRoot
  $resolvedProjectRoot = [System.IO.Path]::GetFullPath($projectRoot)
  $resolvedPath = [System.IO.Path]::GetFullPath($Path)

  if ($resolvedPath.StartsWith($resolvedProjectRoot, [System.StringComparison]::OrdinalIgnoreCase)) {
    $relative = $resolvedPath.Substring($resolvedProjectRoot.Length).TrimStart("\\")
    return $relative -replace "\\", "/"
  }

  return $resolvedPath -replace "\\", "/"
}

function Format-QualityCommand {
  param(
    [string]$Executable,
    [string[]]$Arguments = @()
  )

  $parts = @($Executable)
  foreach ($argument in $Arguments) {
    if ($null -eq $argument) {
      continue
    }

    if ($argument -match '[\s"]') {
      $escaped = $argument.Replace('"', '""')
      $parts += '"{0}"' -f $escaped
      continue
    }

    $parts += $argument
  }

  return ($parts -join ' ')
}

function Invoke-LoggedCommand {
  param(
    [string]$Executable,
    [string[]]$Arguments = @(),
    [string]$WorkingDirectory,
    [string]$LogPath
  )

  $commandText = Format-QualityCommand -Executable $Executable -Arguments $Arguments
  $resolvedWorkingDirectory = if ([string]::IsNullOrWhiteSpace($WorkingDirectory)) {
    Get-QualityProjectRoot
  } else {
    $WorkingDirectory
  }

  $previousLocation = Get-Location
  $startedAt = Get-Date
  $outputText = ""
  $exitCode = 0
  $stdoutPath = [System.IO.Path]::ChangeExtension($LogPath, ".stdout.tmp")
  $stderrPath = [System.IO.Path]::ChangeExtension($LogPath, ".stderr.tmp")

  try {
    Set-Location $resolvedWorkingDirectory

    if (Test-Path $stdoutPath) {
      Remove-Item $stdoutPath -Force
    }
    if (Test-Path $stderrPath) {
      Remove-Item $stderrPath -Force
    }

    $startProcessParameters = @{
      FilePath = $Executable
      WorkingDirectory = $resolvedWorkingDirectory
      Wait = $true
      PassThru = $true
      NoNewWindow = $true
      RedirectStandardOutput = $stdoutPath
      RedirectStandardError = $stderrPath
    }
    if ($Arguments.Count -gt 0) {
      $startProcessParameters.ArgumentList = $Arguments
    }

    $process = Start-Process @startProcessParameters

    $exitCode = [int]$process.ExitCode

    $outputParts = @()
    if (Test-Path $stdoutPath) {
      $outputParts += Get-Content $stdoutPath -Raw
    }
    if (Test-Path $stderrPath) {
      $stderrText = Get-Content $stderrPath -Raw
      if (-not [string]::IsNullOrWhiteSpace($stderrText)) {
        $outputParts += $stderrText
      }
    }
    $outputText = ($outputParts -join [Environment]::NewLine).TrimEnd()
  } catch {
    $exitCode = 1
    $outputText = $_ | Out-String
    $outputText = $outputText.TrimEnd()
  } finally {
    $durationMs = [math]::Round(((Get-Date) - $startedAt).TotalMilliseconds, 3)
    if (Test-Path $stdoutPath) {
      Remove-Item $stdoutPath -Force
    }
    if (Test-Path $stderrPath) {
      Remove-Item $stderrPath -Force
    }
    Set-Location $previousLocation
  }

  $logLines = @(
    "command: $commandText",
    "working_directory: $resolvedWorkingDirectory",
    "exit_code: $exitCode",
    "duration_ms: $durationMs",
    ""
  )
  if (-not [string]::IsNullOrWhiteSpace($outputText)) {
    $logLines += $outputText
  }
  Set-Content -Path $LogPath -Value $logLines -Encoding utf8

  return [pscustomobject]@{
    Command = $commandText
    WorkingDirectory = $resolvedWorkingDirectory
    ExitCode = $exitCode
    DurationMs = $durationMs
    LogPath = $LogPath
    Output = $outputText
  }
}

function Write-JsonFile {
  param(
    [string]$Path,
    [Parameter(ValueFromPipeline = $true)]
    $Value
  )

  $json = $Value | ConvertTo-Json -Depth 10
  Set-Content -Path $Path -Value $json -Encoding utf8
}
