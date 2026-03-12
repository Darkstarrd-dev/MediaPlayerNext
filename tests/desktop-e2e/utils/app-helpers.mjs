export const selectors = {
  headerLogoTrigger: '[data-testid="header-logo-trigger"]',
  headerSettingsTrigger: '[data-testid="header-settings-trigger"]',
  headerThemeDebugTrigger: '[data-testid="header-theme-debug-trigger"]',
  importTaskPanel: '[data-testid="import-task-panel"]',
  importRootPathInput: '[data-testid="import-root-path-input"]',
  importAddAndScan: '[data-testid="import-add-and-scan"]',
  workspaceRoot: '[data-testid="workspace-root"]',
  sidebarTreeNode: '[data-testid="sidebar-tree-node"]',
  settingsPanel: '[data-testid="settings-panel"]',
  settingsPageDatabase: '[data-testid="settings-page-database"]',
  settingsPageDatabaseBody: '[data-testid="settings-page-database-body"]',
  databaseSqlPath: '[data-testid="database-sql-path"]',
  databaseThumbnailPath: '[data-testid="database-thumbnail-path"]',
  databaseSelectSqlDir: '[data-testid="database-select-sql-dir"]',
  databaseSelectThumbnailDir: '[data-testid="database-select-thumbnail-dir"]',
  databaseClearButton: '[data-testid="database-clear-button"]',
  databaseActionMessage: '[data-testid="database-action-message"]',
  databaseClearDialog: '[data-testid="database-clear-dialog"]',
  databaseClearCancel: '[data-testid="database-clear-cancel"]',
  databaseClearConfirm: '[data-testid="database-clear-confirm"]',
  themeDebugPanel: '[data-testid="theme-debug-panel"]',
  themeDebugPageSnapshot: '[data-testid="theme-debug-page-snapshot"]',
  themeDebugPageContainerLayer: '[data-testid="theme-debug-page-container-layer"]',
  themeDebugPageButtonSnapshot: '[data-testid="theme-debug-tab-snapshot"]',
  themeDebugPageButtonContainerLayer: '[data-testid="theme-debug-tab-containerLayer"]',
  themeDebugSnapshotExport: '[data-testid="theme-debug-snapshot-export"]',
  themeDebugSnapshotApply: '[data-testid="theme-debug-snapshot-apply"]',
  themeDebugSnapshotTextarea: '[data-testid="theme-debug-snapshot-textarea"]',
  themeDebugLayoutGapScale: '[data-testid="theme-debug-layout-gap-scale"]',
  themeDebugSplitterWidthScale: '[data-testid="theme-debug-splitter-width-scale"]',
  themeDebugInputBgAppFill: '[data-testid="theme-debug-input-mpx-bg-app-fill"]',
}

export async function openImportTaskPanel() {
  const trigger = await $(selectors.headerLogoTrigger)
  await trigger.waitForDisplayed({ timeout: 30000 })
  await trigger.click()

  const panel = await $(selectors.importTaskPanel)
  await panel.waitForDisplayed({ timeout: 30000 })
}

export async function closeImportTaskPanelIfOpen() {
  const panel = await $(selectors.importTaskPanel)
  const panelVisible = await panel.isDisplayed().catch(() => false)

  if (!panelVisible) {
    return
  }

  await browser.keys('Escape')
  const closedByEscape = await browser
    .waitUntil(async () => !(await panel.isDisplayed().catch(() => false)), {
      timeout: 2000,
      timeoutMsg: 'ImportTaskPanel still open after Escape',
      interval: 100,
    })
    .then(() => true)
    .catch(() => false)

  if (!closedByEscape) {
    await browser.execute(() => {
      const overlay = document.querySelector('[data-overlay-close="import-task-panel"]')
      if (overlay instanceof HTMLElement) {
        overlay.click()
      }
    })
  }

  await browser.waitUntil(async () => !(await panel.isDisplayed().catch(() => false)), {
    timeout: 30000,
    timeoutMsg: 'Timed out waiting for ImportTaskPanel to close',
  })
}

export async function openDatabaseSettingsPage() {
  const settingsTrigger = await $(selectors.headerSettingsTrigger)
  await settingsTrigger.waitForDisplayed({ timeout: 30000 })

  const settingsPanel = await $(selectors.settingsPanel)
  const panelVisible = await settingsPanel.isDisplayed().catch(() => false)
  if (!panelVisible) {
    await settingsTrigger.click()
    await settingsPanel.waitForDisplayed({ timeout: 30000 })
  }

  const databasePageButton = await $(selectors.settingsPageDatabase)
  await databasePageButton.waitForDisplayed({ timeout: 30000 })

  if ((await databasePageButton.getAttribute('aria-pressed')) !== 'true') {
    await databasePageButton.click()
  }

  const databasePageBody = await $(selectors.settingsPageDatabaseBody)
  await databasePageBody.waitForDisplayed({ timeout: 30000 })
}

export async function openThemeDebugPanel() {
  const trigger = await $(selectors.headerThemeDebugTrigger)
  await trigger.waitForDisplayed({ timeout: 30000 })

  const panel = await $(selectors.themeDebugPanel)
  const panelVisible = await panel.isDisplayed().catch(() => false)
  if (!panelVisible) {
    await trigger.click()
    await panel.waitForDisplayed({ timeout: 30000 })
  }
}

export async function readDatabasePaths() {
  const databasePath = await readTrimmedText(selectors.databaseSqlPath)
  const thumbnailCachePath = await readTrimmedText(selectors.databaseThumbnailPath)

  return {
    databasePath,
    thumbnailCachePath,
  }
}

export async function queueDirectorySelection(title, targetPath) {
  await browser.execute(
    (selectionTitle, selectionPath) => {
      window.__MPNEXT_E2E__ ??= { enabled: true, directorySelections: [] }
      window.__MPNEXT_E2E__.enabled = true
      window.__MPNEXT_E2E__.directorySelections ??= []
      window.__MPNEXT_E2E__.directorySelections.push({
        title: selectionTitle,
        path: selectionPath,
      })
    },
    title,
    targetPath,
  )
}

export async function waitForText(selector, expectedText) {
  await browser.waitUntil(async () => (await readTrimmedText(selector)) === expectedText, {
    timeout: 30000,
    timeoutMsg: `Timed out waiting for ${selector} to equal ${expectedText}`,
  })
}

export async function waitForReloadToSettle() {
  await browser.waitUntil(
    async () => {
      const settingsTrigger = await $(selectors.headerSettingsTrigger)
      return settingsTrigger.isDisplayed().catch(() => false)
    },
    {
      timeout: 30000,
      timeoutMsg: 'Timed out waiting for desktop shell to become interactive after reload',
    },
  )
}

async function readTrimmedText(selector) {
  const element = await $(selector)
  await element.waitForDisplayed({ timeout: 30000 })
  return (await element.getText()).trim()
}
