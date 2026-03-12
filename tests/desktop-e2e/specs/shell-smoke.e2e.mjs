describe('MediaPlayerNext desktop shell smoke', () => {
  it('opens settings and shows the database management page', async () => {
    const settingsTrigger = await $('.header-settings-trigger')
    await settingsTrigger.waitForDisplayed({ timeout: 30000 })
    await settingsTrigger.click()

    const settingsPanel = await $('.settings-panel')
    await settingsPanel.waitForDisplayed({ timeout: 30000 })

    const databaseTab = await $('button=数据库管理')
    await databaseTab.waitForDisplayed({ timeout: 30000 })
    await databaseTab.click()

    const databaseHeading = await $('h2=数据库管理')
    await databaseHeading.waitForDisplayed({ timeout: 30000 })

    const clearDatabaseButton = await $('button=清除数据库')
    await clearDatabaseButton.waitForDisplayed({ timeout: 30000 })
  })
})
