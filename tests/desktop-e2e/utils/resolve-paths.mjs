import os from 'node:os'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const currentDirectory = fileURLToPath(new URL('.', import.meta.url))

export const desktopE2eRoot = path.resolve(currentDirectory, '..')
export const repoRoot = path.resolve(desktopE2eRoot, '..', '..')
export const tauriManifestPath = path.resolve(repoRoot, 'src-tauri', 'Cargo.toml')

export function resolveApplicationCandidates() {
  return [
    path.resolve(repoRoot, 'target', 'debug', executableName('mediaplayernext')),
    path.resolve(repoRoot, 'src-tauri', 'target', 'debug', executableName('mediaplayernext')),
  ]
}

export function resolveEdgeDriverCandidates() {
  return [
    path.resolve(repoRoot, executableName('msedgedriver')),
    path.resolve(repoRoot, 'tests', 'desktop-e2e', executableName('msedgedriver')),
    path.resolve(os.homedir(), '.cargo', 'bin', executableName('msedgedriver')),
  ]
}

export function resolveExistingApplicationPath(fsModule) {
  const candidate = resolveApplicationCandidates().find((entry) => fsModule.existsSync(entry))
  return candidate ?? null
}

export function resolvePreferredApplicationPath(fsModule) {
  const existing = resolveExistingApplicationPath(fsModule)
  if (existing !== null) {
    return existing
  }

  return resolveApplicationCandidates()[1]
}

export function resolveExistingEdgeDriverPath(fsModule) {
  const candidate = resolveEdgeDriverCandidates().find((entry) => fsModule.existsSync(entry))
  return candidate ?? null
}

export function resolveCargoBinaryPath(binaryName) {
  const executable = executableName(binaryName)
  return path.resolve(os.homedir(), '.cargo', 'bin', executable)
}

export function executableName(baseName) {
  return process.platform === 'win32' ? `${baseName}.exe` : baseName
}
