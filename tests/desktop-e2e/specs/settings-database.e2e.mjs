import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import {
  openDatabaseSettingsPage,
  queueDirectorySelection,
  readDatabasePaths,
  selectors,
  waitForReloadToSettle,
  waitForText,
} from '../utils/app-helpers.mjs'
import { resolveDesktopE2eRuntimeContextFromEnv } from '../utils/runtime-context.mjs'

const runtimeContext = resolveDesktopE2eRuntimeContextFromEnv()

describe('数据库管理自动验收', () => {
  it('读取当前路径，并在取消清除时保持状态不变', async () => {
    await openDatabaseSettingsPage()

    await waitForText(selectors.databaseSqlPath, runtimeContext.initialDatabasePath)
    await waitForText(selectors.databaseThumbnailPath, runtimeContext.initialThumbnailCacheDir)

    const initialPaths = await readDatabasePaths()
    assert.equal(path.normalize(initialPaths.databasePath), path.normalize(runtimeContext.initialDatabasePath))
    assert.equal(
      path.normalize(initialPaths.thumbnailCachePath),
      path.normalize(runtimeContext.initialThumbnailCacheDir),
    )
    assert.equal(fs.existsSync(runtimeContext.runtimeStorageConfigPath), true)

    const clearButton = await $(selectors.databaseClearButton)
    await clearButton.click()

    const clearDialog = await $(selectors.databaseClearDialog)
    await clearDialog.waitForDisplayed({ timeout: 30000 })

    const cancelButton = await $(selectors.databaseClearCancel)
    await cancelButton.click()

    await browser.waitUntil(async () => !(await $(selectors.databaseClearDialog).isExisting()), {
      timeout: 30000,
      timeoutMsg: 'Timed out waiting for clear dialog to close after cancel',
    })

    const afterCancelPaths = await readDatabasePaths()
    assert.deepEqual(afterCancelPaths, initialPaths)
    assert.equal(fs.existsSync(runtimeContext.runtimeStorageConfigPath), true)
  })

  it('通过测试替身切换 SQL 与缩略图目录，并更新 runtime storage config', async () => {
    await openDatabaseSettingsPage()

    writeFile(runtimeContext.initialDatabasePath, 'db-before-switch')
    writeFile(`${runtimeContext.initialDatabasePath}-wal`, 'wal-before-switch')
    writeFile(`${runtimeContext.initialDatabasePath}-shm`, 'shm-before-switch')
    writeFile(path.join(runtimeContext.initialThumbnailCacheDir, 'keep.txt'), 'thumb-before-switch')

    await queueDirectorySelection('选择 SQL 目录', runtimeContext.switchedDatabaseDir)
    const selectSqlButton = await $(selectors.databaseSelectSqlDir)
    await selectSqlButton.click()

    await waitForText(selectors.databaseSqlPath, runtimeContext.switchedDatabasePath)
    const configAfterDatabaseSwitch = readRuntimeStorageConfig()
    assert.equal(
      path.normalize(configAfterDatabaseSwitch.database_dir),
      path.normalize(runtimeContext.switchedDatabaseDir),
    )
    assert.equal(fs.existsSync(runtimeContext.initialDatabasePath), false)
    assert.equal(fs.existsSync(`${runtimeContext.initialDatabasePath}-wal`), false)
    assert.equal(fs.existsSync(`${runtimeContext.initialDatabasePath}-shm`), false)
    assert.equal(fs.existsSync(runtimeContext.switchedDatabasePath), true)
    assert.equal(fs.existsSync(`${runtimeContext.switchedDatabasePath}-wal`), true)
    assert.equal(fs.existsSync(`${runtimeContext.switchedDatabasePath}-shm`), true)

    await queueDirectorySelection('选择缩略图目录', runtimeContext.switchedThumbnailCacheDir)
    const selectThumbnailButton = await $(selectors.databaseSelectThumbnailDir)
    await selectThumbnailButton.click()

    await waitForText(selectors.databaseThumbnailPath, runtimeContext.switchedThumbnailCacheDir)
    const configAfterThumbnailSwitch = readRuntimeStorageConfig()
    assert.equal(
      path.normalize(configAfterThumbnailSwitch.thumbnail_cache_dir),
      path.normalize(runtimeContext.switchedThumbnailCacheDir),
    )
    assert.equal(fs.existsSync(runtimeContext.switchedThumbnailCacheDir), true)
    assert.equal(fs.existsSync(path.join(runtimeContext.initialThumbnailCacheDir, 'keep.txt')), true)
  })

  it('确认清除后删除 runtime 数据并回落到隔离默认路径', async () => {
    await openDatabaseSettingsPage()

    const normalizeMarkerPath = path.join(runtimeContext.normalizeRoot, 'normalize.txt')
    const playbackMarkerPath = path.join(runtimeContext.playbackSessionsRoot, 'session.txt')

    writeFile(runtimeContext.switchedDatabasePath, 'db-before-clear')
    writeFile(`${runtimeContext.switchedDatabasePath}-wal`, 'wal-before-clear')
    writeFile(`${runtimeContext.switchedDatabasePath}-shm`, 'shm-before-clear')
    writeFile(path.join(runtimeContext.switchedThumbnailCacheDir, 'thumb.txt'), 'thumb-before-clear')
    writeFile(normalizeMarkerPath, 'normalize-before-clear')
    writeFile(playbackMarkerPath, 'playback-before-clear')

    const clearButton = await $(selectors.databaseClearButton)
    await clearButton.click()

    const confirmButton = await $(selectors.databaseClearConfirm)
    await confirmButton.waitForDisplayed({ timeout: 30000 })
    await confirmButton.click()

    await waitForReloadToSettle()
    await openDatabaseSettingsPage()

    const pathsAfterClear = await readDatabasePaths()
    assert.equal(path.normalize(pathsAfterClear.databasePath), path.normalize(runtimeContext.defaultDatabasePath))
    assert.equal(
      path.normalize(pathsAfterClear.thumbnailCachePath),
      path.normalize(runtimeContext.defaultThumbnailCachePath),
    )
    assert.equal(fs.existsSync(runtimeContext.runtimeStorageConfigPath), false)
    assert.equal(fs.existsSync(runtimeContext.switchedDatabasePath), false)
    assert.equal(fs.existsSync(`${runtimeContext.switchedDatabasePath}-wal`), false)
    assert.equal(fs.existsSync(`${runtimeContext.switchedDatabasePath}-shm`), false)
    assert.equal(fs.existsSync(runtimeContext.switchedThumbnailCacheDir), false)
    assert.equal(fs.existsSync(normalizeMarkerPath), false)
    assert.equal(fs.existsSync(playbackMarkerPath), false)
  })
})

function readRuntimeStorageConfig() {
  return JSON.parse(fs.readFileSync(runtimeContext.runtimeStorageConfigPath, 'utf8'))
}

function writeFile(filePath, content) {
  fs.mkdirSync(path.dirname(filePath), { recursive: true })
  fs.writeFileSync(filePath, content)
}
