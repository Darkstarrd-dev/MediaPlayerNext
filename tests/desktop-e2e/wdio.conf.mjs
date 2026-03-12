import fs from 'node:fs'
import path from 'node:path'
import { spawn, spawnSync } from 'node:child_process'
import net from 'node:net'
import {
  applyDesktopE2eEnvironment,
  cleanupDesktopE2eRuntimeContext,
  createDesktopE2eRuntimeContext,
  resolveDesktopE2eRuntimeContextFromEnv,
} from './utils/runtime-context.mjs'
import {
  repoRoot,
  resolveExistingEdgeDriverPath,
  resolveCargoBinaryPath,
  resolvePreferredApplicationPath,
} from './utils/resolve-paths.mjs'

let tauriDriverProcess
let tauriDriverExpectedExit = false
const { desktopE2eRuntimeContext, ownsDesktopE2eRuntimeContext } = initializeDesktopE2eRuntimeContext()

function initializeDesktopE2eRuntimeContext() {
  try {
    return {
      desktopE2eRuntimeContext: resolveDesktopE2eRuntimeContextFromEnv(),
      ownsDesktopE2eRuntimeContext: false,
    }
  } catch {
    const createdContext = createDesktopE2eRuntimeContext()
    applyDesktopE2eEnvironment(createdContext)
    return {
      desktopE2eRuntimeContext: createdContext,
      ownsDesktopE2eRuntimeContext: true,
    }
  }
}

function ensureSuccess(result, label) {
  if (result.status === 0) {
    return
  }

  throw new Error(`${label} failed with exit code ${result.status ?? 'null'}`)
}

function buildDesktopApplication() {
  spawnSync('powershell', ['-NoProfile', '-Command', 'Stop-Process -Name mediaplayernext -Force -ErrorAction SilentlyContinue'], {
    cwd: repoRoot,
    stdio: 'ignore',
    shell: true,
  })
  spawnSync('powershell', ['-NoProfile', '-Command', 'Stop-Process -Name tauri-driver -Force -ErrorAction SilentlyContinue'], {
    cwd: repoRoot,
    stdio: 'ignore',
    shell: true,
  })

  const result = spawnSync(path.resolve(repoRoot, 'scripts', 'run-tauri-build.cmd'), ['--debug', '--no-bundle'], {
    cwd: repoRoot,
    stdio: 'inherit',
    shell: true,
  })

  ensureSuccess(result, 'Tauri desktop build')
}

function startTauriDriver() {
  const tauriDriverBinary = process.env.MPNEXT_TAURI_DRIVER_PATH || resolveCargoBinaryPath('tauri-driver')
  const args = []

  const edgeDriverPath = process.env.MPNEXT_MSEDGEDRIVER_PATH || resolveExistingEdgeDriverPath(fs)
  if (process.platform === 'win32' && edgeDriverPath) {
    args.push('--native-driver', edgeDriverPath)
  }

  tauriDriverExpectedExit = false
  tauriDriverProcess = spawn(tauriDriverBinary, args, {
    cwd: repoRoot,
    stdio: ['ignore', 'inherit', 'inherit'],
  })

  tauriDriverProcess.on('error', (error) => {
    console.error('tauri-driver error:', error)
    process.exit(1)
  })

  tauriDriverProcess.on('exit', (code) => {
    if (!tauriDriverExpectedExit && code !== 0) {
      console.error('tauri-driver exited unexpectedly with code:', code)
      process.exit(1)
    }
  })
}

function waitForDriverReady({
  host,
  port,
  timeoutMs = 15000,
  intervalMs = 120,
}) {
  const startedAt = Date.now()

  return new Promise((resolve, reject) => {
    const tryConnect = () => {
      const socket = net.connect({ host, port })

      socket.once('connect', () => {
        socket.destroy()
        resolve()
      })

      socket.once('error', () => {
        socket.destroy()
        if (Date.now() - startedAt >= timeoutMs) {
          reject(new Error(`tauri-driver is not ready on ${host}:${port} within ${timeoutMs}ms`))
          return
        }

        setTimeout(tryConnect, intervalMs)
      })
    }

    tryConnect()
  })
}

function closeTauriDriver() {
  tauriDriverExpectedExit = true
  tauriDriverProcess?.kill()
  tauriDriverProcess = undefined
}

function registerShutdownHooks() {
  const cleanup = () => {
    closeTauriDriver()
  }

  process.on('exit', cleanup)
  process.on('SIGINT', cleanup)
  process.on('SIGTERM', cleanup)
  process.on('SIGHUP', cleanup)
  process.on('SIGBREAK', cleanup)
}

registerShutdownHooks()

export const config = {
  runner: 'local',
  hostname: '127.0.0.1',
  port: 4444,
  path: '/',
  specs: ['./specs/**/*.e2e.mjs'],
  maxInstances: 1,
  capabilities: [
    {
      maxInstances: 1,
      'tauri:options': {
        application: resolvePreferredApplicationPath(fs),
        webviewOptions: {},
      },
    },
  ],
  reporters: ['spec'],
  framework: 'mocha',
  mochaOpts: {
    ui: 'bdd',
    timeout: 120000,
  },
  waitforTimeout: 30000,
  connectionRetryTimeout: 120000,
  connectionRetryCount: 1,
  onPrepare: async () => {
    buildDesktopApplication()
    startTauriDriver()
    await waitForDriverReady({ host: '127.0.0.1', port: 4444 })
  },
  onComplete: () => {
    closeTauriDriver()
    if (ownsDesktopE2eRuntimeContext) {
      cleanupDesktopE2eRuntimeContext(desktopE2eRuntimeContext)
    }
  },
}
