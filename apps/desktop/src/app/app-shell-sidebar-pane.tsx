import type { LibrarySummary, SidebarNodeSummary } from '@mediaplayernext/contracts'
import { resolvePathLeaf } from './app-shell-utils'

interface AppShellSidebarPaneProps {
  selectedLibrarySummary: LibrarySummary | null
  selectedLibraryId: string | null
  libraries: LibrarySummary[]
  librariesLoading: boolean
  sidebarNodesLoading: boolean
  sidebarNodes: SidebarNodeSummary[]
  selectedSidebarNodeId: string | null
  sidebarFooterText: string
  onLibrarySelect: (libraryId: string) => void
  onSidebarNodeSelect: (nodeId: string) => void
}

export function AppShellSidebarPane({
  selectedLibrarySummary,
  selectedLibraryId,
  libraries,
  librariesLoading,
  sidebarNodesLoading,
  sidebarNodes,
  selectedSidebarNodeId,
  sidebarFooterText,
  onLibrarySelect,
  onSidebarNodeSelect,
}: AppShellSidebarPaneProps) {
  return (
    <aside className="app-frame app-sidebar-root" data-slot="fg-sidebar-root">
      <section className="workspace-pane sidebar-frame">
        <header className="workspace-pane-header sidebar-header" data-slot="fg-sidebar-header">
          <div className="pane-title-stack sidebar-title-stack">
            <h2>直属节点</h2>
            <span className="sidebar-selected-library">
              {selectedLibrarySummary === null
                ? '未选择媒体库'
                : resolvePathLeaf(selectedLibrarySummary.rootPath)}
            </span>
          </div>

          <label className="sidebar-library-select-wrap" aria-label="切换媒体库">
            <span>媒体库</span>
            <select
              className="sidebar-library-select"
              value={selectedLibraryId ?? ''}
              onChange={(event) => {
                const nextLibraryId = event.target.value.trim()
                if (nextLibraryId.length === 0) {
                  return
                }

                onLibrarySelect(nextLibraryId)
              }}
              disabled={librariesLoading || libraries.length === 0}
            >
              {libraries.map((library) => (
                <option key={library.id} value={library.id}>
                  {resolvePathLeaf(library.rootPath)}
                </option>
              ))}
            </select>
          </label>
        </header>

        <div className="workspace-pane-main sidebar-main-shell" data-slot="fg-sidebar-main">
          {selectedLibraryId === null ? (
            <section className="workspace-stage compact">
              <span className="workspace-label">Sidebar</span>
              <strong>等待导入媒体库</strong>
              <p>导入后将显示当前媒体库的直属节点列表。</p>
            </section>
          ) : sidebarNodesLoading && sidebarNodes.length === 0 ? (
            <section className="workspace-stage compact">
              <span className="workspace-label">Sidebar</span>
              <strong>正在读取直属节点</strong>
              <p>当前媒体库的首层节点正在同步，请稍候。</p>
            </section>
          ) : sidebarNodes.length === 0 ? (
            <section className="workspace-stage compact">
              <span className="workspace-label">Sidebar</span>
              <strong>当前媒体库暂无直属节点</strong>
              <p>可以先执行扫描，或继续导入新的本地路径。</p>
            </section>
          ) : (
            <div className="sidebar-tree" role="tree" aria-label="直属媒体节点列表">
              {sidebarNodes.map((node) => {
                const isActive = node.nodeId === selectedSidebarNodeId

                return (
                  <button
                    key={node.nodeId}
                    className={`sidebar-tree-node ${isActive ? 'is-active' : ''}`}
                    type="button"
                    role="treeitem"
                    aria-selected={isActive}
                    data-testid="sidebar-tree-node"
                    data-node-id={node.nodeId}
                    data-node-type={node.nodeType}
                    data-media-source-id={node.mediaSourceId ?? ''}
                    data-has-direct-media-child={node.hasDirectMediaChild ? '1' : '0'}
                    style={{ paddingInlineStart: `${14 + node.depth * 14}px` }}
                    onClick={() => onSidebarNodeSelect(node.nodeId)}
                  >
                    <span className="sidebar-tree-node-rail" aria-hidden="true" />
                    <span className="sidebar-tree-node-dot" aria-hidden="true" />
                    <span className="sidebar-tree-node-copy">
                      <strong>{node.label}</strong>
                      <span>
                        {node.nodeType === 'media_source'
                          ? `${node.sourceType ?? 'source'} · ${node.itemCount ?? 0} 项`
                          : node.hasDirectMediaChild
                            ? '包含直属媒体节点'
                            : '路径节点'}
                      </span>
                    </span>
                  </button>
                )
              })}
            </div>
          )}
        </div>

        <footer className="workspace-pane-footer sidebar-footer" data-slot="fg-sidebar-footer">
          <span>{sidebarFooterText}</span>
        </footer>
      </section>
    </aside>
  )
}
