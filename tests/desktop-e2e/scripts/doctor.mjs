import fs from 'node:fs'
import { spawnSync } from 'node:child_process'
import {
  resolveEdgeDriverCandidates,
  resolveExistingEdgeDriverPath,
  resolveApplicationCandidates,
  resolveCargoBinaryPath,
  tauriManifestPath,
} from '../utils/resolve-paths.mjs'

function resolveCommandStatus(command, explicitPath) {
  if (explicitPath) {
    return {
      status: fs.existsSync(explicitPath) ? 'ok' : 'missing',
      location: explicitPath,
      source: 'env',
    }
  }

  const whereCommand = process.platform === 'win32' ? 'where' : 'which'
  const lookup = spawnSync(whereCommand, [command], { stdio: 'pipe', encoding: 'utf8', shell: true })

  if (lookup.status === 0) {
    const firstLine = lookup.stdout.split(/\r?\n/).find((line) => line.trim().length > 0) ?? command
    return {
      status: 'ok',
      location: firstLine,
      source: 'path',
    }
  }

  return {
    status: 'missing',
    location: explicitPath ?? command,
    source: 'path',
  }
}

const tauriDriverPath = process.env.MPNEXT_TAURI_DRIVER_PATH || resolveCargoBinaryPath('tauri-driver')
const edgeDriverPath = process.env.MPNEXT_MSEDGEDRIVER_PATH || resolveExistingEdgeDriverPath(fs)

const report = {
  tauriManifestPath,
  applicationCandidates: resolveApplicationCandidates(),
  edgeDriverCandidates: resolveEdgeDriverCandidates(),
  tauriDriver: resolveCommandStatus('tauri-driver', tauriDriverPath),
  msedgedriver: resolveCommandStatus('msedgedriver', edgeDriverPath),
}

console.log(JSON.stringify(report, null, 2))

const hasBlockingError = report.tauriDriver.status !== 'ok' || report.msedgedriver.status !== 'ok'
if (hasBlockingError) {
  process.exitCode = 1
}
