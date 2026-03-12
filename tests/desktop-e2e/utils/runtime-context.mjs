import fs from 'node:fs'
import os from 'node:os'
import path from 'node:path'

const ENV_KEYS = {
  sessionRoot: 'MPNEXT_E2E_SESSION_ROOT',
  runtimeStorageConfigPath: 'MPNEXT_RUNTIME_STORAGE_CONFIG_PATH',
  runtimeDefaultDataDir: 'MPNEXT_RUNTIME_DEFAULT_DATA_DIR',
  runtimeDefaultCacheDir: 'MPNEXT_RUNTIME_DEFAULT_CACHE_DIR',
  normalizeRoot: 'MPNEXT_BACKEND_NORMALIZE_ROOT',
  playbackSessionsRoot: 'MPNEXT_BACKEND_PLAYBACK_SESSIONS_ROOT',
}

const E2E_DATABASE_FILE_NAME = 'mediaplayernext.db'

function normalizeSlashes(targetPath) {
  return targetPath.replaceAll('/', path.sep)
}

export function createDesktopE2eRuntimeContext() {
  const sessionRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'mpnext-desktop-e2e-'))
  const runtimeStorageConfigPath = path.join(sessionRoot, 'runtime', 'runtime-storage-paths.json')
  const runtimeDefaultDataDir = path.join(sessionRoot, 'default-data')
  const runtimeDefaultCacheDir = path.join(sessionRoot, 'default-cache')
  const defaultDatabasePath = path.join(runtimeDefaultDataDir, E2E_DATABASE_FILE_NAME)
  const defaultThumbnailCachePath = path.join(runtimeDefaultCacheDir, 'thumbs')
  const normalizeRoot = path.join(runtimeDefaultCacheDir, 'normalized')
  const playbackSessionsRoot = path.join(runtimeDefaultCacheDir, 'playback', 'sessions')
  const initialDatabaseDir = path.join(sessionRoot, 'storage', 'initial-sql')
  const initialThumbnailCacheDir = path.join(sessionRoot, 'storage', 'initial-thumbs')
  const switchedDatabaseDir = path.join(sessionRoot, 'storage', 'switched-sql')
  const switchedThumbnailCacheDir = path.join(sessionRoot, 'storage', 'switched-thumbs')

  fs.mkdirSync(path.dirname(runtimeStorageConfigPath), { recursive: true })
  fs.mkdirSync(runtimeDefaultDataDir, { recursive: true })
  fs.mkdirSync(runtimeDefaultCacheDir, { recursive: true })
  fs.mkdirSync(initialDatabaseDir, { recursive: true })
  fs.mkdirSync(initialThumbnailCacheDir, { recursive: true })
  fs.mkdirSync(switchedDatabaseDir, { recursive: true })
  fs.mkdirSync(switchedThumbnailCacheDir, { recursive: true })

  const config = {
    database_dir: normalizeSlashes(initialDatabaseDir),
    thumbnail_cache_dir: normalizeSlashes(initialThumbnailCacheDir),
  }

  fs.writeFileSync(runtimeStorageConfigPath, JSON.stringify(config, null, 2))

  return {
    sessionRoot,
    runtimeStorageConfigPath,
    runtimeDefaultDataDir,
    runtimeDefaultCacheDir,
    defaultDatabasePath,
    defaultThumbnailCachePath,
    normalizeRoot,
    playbackSessionsRoot,
    initialDatabaseDir,
    initialDatabasePath: path.join(initialDatabaseDir, E2E_DATABASE_FILE_NAME),
    initialThumbnailCacheDir,
    switchedDatabaseDir,
    switchedDatabasePath: path.join(switchedDatabaseDir, E2E_DATABASE_FILE_NAME),
    switchedThumbnailCacheDir,
  }
}

export function applyDesktopE2eEnvironment(context) {
  process.env[ENV_KEYS.sessionRoot] = context.sessionRoot
  process.env[ENV_KEYS.runtimeStorageConfigPath] = context.runtimeStorageConfigPath
  process.env[ENV_KEYS.runtimeDefaultDataDir] = context.runtimeDefaultDataDir
  process.env[ENV_KEYS.runtimeDefaultCacheDir] = context.runtimeDefaultCacheDir
  process.env[ENV_KEYS.normalizeRoot] = context.normalizeRoot
  process.env[ENV_KEYS.playbackSessionsRoot] = context.playbackSessionsRoot
}

export function resolveDesktopE2eRuntimeContextFromEnv() {
  const context = {
    sessionRoot: process.env[ENV_KEYS.sessionRoot],
    runtimeStorageConfigPath: process.env[ENV_KEYS.runtimeStorageConfigPath],
    runtimeDefaultDataDir: process.env[ENV_KEYS.runtimeDefaultDataDir],
    runtimeDefaultCacheDir: process.env[ENV_KEYS.runtimeDefaultCacheDir],
    normalizeRoot: process.env[ENV_KEYS.normalizeRoot],
    playbackSessionsRoot: process.env[ENV_KEYS.playbackSessionsRoot],
  }

  if (Object.values(context).some((value) => typeof value !== 'string' || value.length === 0)) {
    throw new Error('Desktop E2E runtime context is not available in process env')
  }

  return {
    sessionRoot: context.sessionRoot,
    runtimeStorageConfigPath: context.runtimeStorageConfigPath,
    runtimeDefaultDataDir: context.runtimeDefaultDataDir,
    runtimeDefaultCacheDir: context.runtimeDefaultCacheDir,
    defaultDatabasePath: path.join(context.runtimeDefaultDataDir, E2E_DATABASE_FILE_NAME),
    defaultThumbnailCachePath: path.join(context.runtimeDefaultCacheDir, 'thumbs'),
    normalizeRoot: context.normalizeRoot,
    playbackSessionsRoot: context.playbackSessionsRoot,
    initialDatabaseDir: path.join(context.sessionRoot, 'storage', 'initial-sql'),
    initialDatabasePath: path.join(context.sessionRoot, 'storage', 'initial-sql', E2E_DATABASE_FILE_NAME),
    initialThumbnailCacheDir: path.join(context.sessionRoot, 'storage', 'initial-thumbs'),
    switchedDatabaseDir: path.join(context.sessionRoot, 'storage', 'switched-sql'),
    switchedDatabasePath: path.join(context.sessionRoot, 'storage', 'switched-sql', E2E_DATABASE_FILE_NAME),
    switchedThumbnailCacheDir: path.join(context.sessionRoot, 'storage', 'switched-thumbs'),
  }
}

export function cleanupDesktopE2eRuntimeContext(context) {
  if (process.env.MPNEXT_E2E_KEEP_RUNTIME === '1') {
    return
  }

  fs.rmSync(context.sessionRoot, { recursive: true, force: true })
}
