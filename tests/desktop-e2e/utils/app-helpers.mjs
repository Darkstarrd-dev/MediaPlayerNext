export const selectors = {
  headerSettingsTrigger: '[data-testid="header-settings-trigger"]',
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
