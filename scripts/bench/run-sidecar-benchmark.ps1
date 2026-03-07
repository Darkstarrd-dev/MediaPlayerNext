param(
  [int]$Runs = 5,
  [string]$OutputPath
)

$ErrorActionPreference = "Stop"

. (Join-Path $PSScriptRoot "common.ps1")

$context = New-MpNextBenchContext -Name "sidecar" -OutputPath $OutputPath
Build-MpNextBackendHarness

$entryPath = Join-Path $context.WorkspaceRoot "apps\subtitle-sidecar\dist\src\index.js"
if (-not (Test-Path $entryPath)) {
  throw "subtitle sidecar entry missing: $entryPath"
}

$ping = Measure-MpNextOperation -Runs $Runs -Action {
  Invoke-MpNextBackendHarness -Context $context -Arguments @("subtitle", "ping") -ExtraEnv @{
    MPNEXT_SUBTITLE_SESSIONS_ROOT = $context.SubtitleSessionsRoot
  }
}

$health = Measure-MpNextOperation -Runs $Runs -Action {
  Invoke-MpNextBackendHarness -Context $context -Arguments @("subtitle", "health") -ExtraEnv @{
    MPNEXT_SUBTITLE_SESSIONS_ROOT = $context.SubtitleSessionsRoot
  }
}

$retryScriptPath = Join-Path $context.RunRoot "retry-sidecar.mjs"
$attemptFilePath = Join-Path $context.RunRoot "retry-sidecar.attempt.txt"
$retryScript = @'
import { createInterface } from "node:readline";
import { readFile, writeFile } from "node:fs/promises";

const attemptFile = new URL("./retry-sidecar.attempt.txt", import.meta.url);
const currentAttempt = await readFile(attemptFile, "utf8").catch(() => "0");
if (currentAttempt.trim() === "0") {
  await writeFile(attemptFile, "1");
  process.stderr.write("intentional first boot crash");
  process.exit(2);
}

const rl = createInterface({ input: process.stdin, crlfDelay: Infinity });
for await (const line of rl) {
  if (!line.trim()) {
    continue;
  }

  const request = JSON.parse(line);
  process.stdout.write(`${JSON.stringify({
    id: request.id,
    type: "response",
    ok: true,
    payload: {
      service: "subtitle-sidecar",
      protocolVersion: "b8-v1",
      transport: "stdio",
      nodeVersion: process.version,
      sharpVersion: "0.34.4",
      uptimeMs: 1,
      activeSessions: 0,
      sessionsRoot: "sessions",
    },
  })}\n`);
}
'@
Set-Content -Encoding utf8 -Path $retryScriptPath -Value $retryScript

$restart = Measure-MpNextOperation -Runs $Runs -BeforeEach {
  if (Test-Path $attemptFilePath) {
    Remove-Item -Force $attemptFilePath
  }
} -Action {
  Invoke-MpNextBackendHarness -Context $context -Arguments @("subtitle", "health") -ExtraEnv @{
    MPNEXT_SUBTITLE_ENTRY_PATH = $retryScriptPath
    MPNEXT_SUBTITLE_SESSIONS_ROOT = $context.SubtitleSessionsRoot
  }
}

$output = [pscustomobject]@{
  benchmark = "sidecar"
  generatedAt = (Get-Date).ToString("s")
  outputRoot = $context.RunRoot
  entryPath = $entryPath
  timings = [pscustomobject]@{
    ping = [pscustomobject]@{
      stats = $ping.stats
      samplesMs = $ping.samplesMs
      lastResult = $ping.lastResult.payload
    }
    health = [pscustomobject]@{
      stats = $health.stats
      samplesMs = $health.samplesMs
      lastResult = $health.lastResult.payload
    }
    restart = [pscustomobject]@{
      stats = $restart.stats
      samplesMs = $restart.samplesMs
      lastResult = $restart.lastResult.payload
    }
  }
}

$resultPath = Join-Path $context.RunRoot "sidecar-benchmark.json"
Write-MpNextJson -Path $resultPath -Value $output
$output | ConvertTo-Json -Depth 100
