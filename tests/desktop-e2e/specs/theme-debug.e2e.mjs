import assert from 'node:assert/strict'
import { openThemeDebugPanel, selectors } from '../utils/app-helpers.mjs'

describe('主题调试面板自动验收', () => {
  it('支持双分页切换、快照导出与根级变量写入', async () => {
    await openThemeDebugPanel()

    const snapshotPage = await $(selectors.themeDebugPageSnapshot)
    await snapshotPage.waitForDisplayed({ timeout: 30000 })

    const exportButton = await $(selectors.themeDebugSnapshotExport)
    await exportButton.waitForDisplayed({ timeout: 30000 })
    await exportButton.click()

    const snapshotTextarea = await $(selectors.themeDebugSnapshotTextarea)
    await browser.waitUntil(
      async () => {
        const text = await snapshotTextarea.getValue()
        return text.includes('"scope": "1.0-2.4-root"')
      },
      {
        timeout: 30000,
        timeoutMsg: 'Timed out waiting for snapshot JSON export content',
      },
    )

    const containerLayerTab = await $(selectors.themeDebugPageButtonContainerLayer)
    await containerLayerTab.waitForDisplayed({ timeout: 30000 })
    await containerLayerTab.click()

    const containerLayerPage = await $(selectors.themeDebugPageContainerLayer)
    await containerLayerPage.waitForDisplayed({ timeout: 30000 })

    const backgroundInput = await $(selectors.themeDebugInputBgAppFill)
    await backgroundInput.waitForDisplayed({ timeout: 30000 })
    await backgroundInput.click()
    await backgroundInput.clearValue()
    await backgroundInput.setValue('#123456')

    await browser.waitUntil(
      async () => {
        const cssValue = await browser.execute(() => {
          const root = document.documentElement
          const inlineValue = root.style.getPropertyValue('--mpx-bg-app-fill').trim()
          const computedValue = window.getComputedStyle(root).getPropertyValue('--mpx-bg-app-fill').trim()
          return inlineValue.length > 0 ? inlineValue : computedValue
        })
        return cssValue === '#123456' || cssValue === 'rgb(18, 52, 86)'
      },
      {
        timeout: 30000,
        timeoutMsg: 'Timed out waiting for bg app fill override to update',
      },
    )

    await browser.keys('Escape')
    await browser.waitUntil(
      async () => !(await $(selectors.themeDebugPanel).isDisplayed().catch(() => false)),
      {
        timeout: 30000,
        timeoutMsg: 'Timed out waiting for theme debug panel to close by Escape',
      },
    )

    const themeDebugPanelVisible = await $(selectors.themeDebugPanel).isDisplayed().catch(() => false)
    assert.equal(themeDebugPanelVisible, false)
  })
})
