import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import { closeImportTaskPanelIfOpen, openImportTaskPanel, selectors } from '../utils/app-helpers.mjs'
import { resolveDesktopE2eRuntimeContextFromEnv } from '../utils/runtime-context.mjs'

const runtimeContext = resolveDesktopE2eRuntimeContextFromEnv()

describe('Sidebar.main 节点语义验收', () => {
  it('目录节点只更新选中态，媒体源节点驱动 Main 作用域', async () => {
    const fixtureRoot = path.join(runtimeContext.sessionRoot, 'fixtures', 'sidebar-main')
    seedSidebarFixture(fixtureRoot)

    await openImportTaskPanel()

    const pathInput = await $(selectors.importRootPathInput)
    await pathInput.waitForDisplayed({ timeout: 30000 })
    await pathInput.setValue(fixtureRoot)

    const addAndScanButton = await $(selectors.importAddAndScan)
    await addAndScanButton.waitForEnabled({ timeout: 30000 })
    await addAndScanButton.click()

    await browser.waitUntil(
      async () => {
        const nodes = await $$(selectors.sidebarTreeNode)
        for (const node of nodes) {
          if ((await node.getAttribute('data-node-type')) === 'media_source') {
            return true
          }
        }

        return false
      },
      {
        timeout: 60000,
        timeoutMsg: 'Timed out waiting for Sidebar media source nodes after import scan',
      },
    )

    await closeImportTaskPanelIfOpen()

    const { mediaNodeId, folderNodeId } = await pickSidebarNodesForAssertion()
    assert.ok(mediaNodeId, 'should find at least one media source node')
    assert.ok(folderNodeId, 'should find one folder node with direct media child')

    const mediaNode = await $(`${selectors.sidebarTreeNode}[data-node-id="${mediaNodeId}"]`)
    const folderNode = await $(`${selectors.sidebarTreeNode}[data-node-id="${folderNodeId}"]`)

    const workspaceRoot = await $(selectors.workspaceRoot)
    await workspaceRoot.waitForDisplayed({ timeout: 30000 })

    const mediaSourceId = await mediaNode.getAttribute('data-media-source-id')
    assert.ok(mediaSourceId && mediaSourceId.length > 0, 'media node should carry mediaSourceId')

    await mediaNode.click()
    await browser.waitUntil(
      async () => (await workspaceRoot.getAttribute('data-active-media-source-id')) === mediaSourceId,
      {
        timeout: 30000,
        timeoutMsg: 'Timed out waiting for media node selection to drive main scope',
      },
    )

    const activeMediaSourceIdBeforeFolderClick = await workspaceRoot.getAttribute(
      'data-active-media-source-id',
    )
    await folderNode.click()

    await browser.waitUntil(
      async () =>
        (await workspaceRoot.getAttribute('data-active-sidebar-node-id')) ===
        (await folderNode.getAttribute('data-node-id')),
      {
        timeout: 30000,
        timeoutMsg: 'Timed out waiting for folder node to become active selection',
      },
    )

    const activeMediaSourceIdAfterFolderClick = await workspaceRoot.getAttribute(
      'data-active-media-source-id',
    )
    assert.equal(
      activeMediaSourceIdAfterFolderClick,
      activeMediaSourceIdBeforeFolderClick,
      'folder selection should not change active media source scope',
    )
  })
})

async function pickSidebarNodesForAssertion() {
  const snapshots = await browser.execute((nodeSelector) => {
    return Array.from(document.querySelectorAll(nodeSelector)).map((node) => ({
      nodeId: node.getAttribute('data-node-id') ?? '',
      nodeType: node.getAttribute('data-node-type') ?? '',
      hasDirectMediaChild: node.getAttribute('data-has-direct-media-child') === '1',
    }))
  }, selectors.sidebarTreeNode)

  const mediaNode = snapshots.find((node) => node.nodeType === 'media_source')
  const folderNode = snapshots.find(
    (node) => node.nodeType === 'folder' && node.hasDirectMediaChild,
  )

  return {
    mediaNodeId: mediaNode?.nodeId ?? null,
    folderNodeId: folderNode?.nodeId ?? null,
  }
}

function seedSidebarFixture(rootPath) {
  const pngBytes = Buffer.from(
    'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/w8AAgMBgUYhV0QAAAAASUVORK5CYII=',
    'base64',
  )

  const files = [
    path.join(rootPath, 'group-a', '001.png'),
    path.join(rootPath, 'group-b', '001.png'),
  ]

  for (const filePath of files) {
    fs.mkdirSync(path.dirname(filePath), { recursive: true })
    fs.writeFileSync(filePath, pngBytes)
  }
}
