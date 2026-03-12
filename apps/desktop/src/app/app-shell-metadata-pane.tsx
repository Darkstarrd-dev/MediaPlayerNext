import type { ItemDetail, LibraryDetail } from '@mediaplayernext/contracts'
import { formatDateTime, resolveItemDisplayLabel, resolveItemLocation, resolvePathLeaf } from './app-shell-utils'

interface AppShellMetadataPaneProps {
  selectedLibraryId: string | null
  selectedLibraryDetail: LibraryDetail | null
  selectedItemDetail: ItemDetail | null
  scanStateLabel: string
  scanStateData: string
  scanSummary: string
  scanActiveSourceCount: number
  scanMissingSourceCount: number
  importBusy: boolean
  actionPendingLabel: string | null
  workspaceError: string | null
  itemDetailLoading: boolean
  itemDetailError: string | null
}

export function AppShellMetadataPane({
  selectedLibraryId,
  selectedLibraryDetail,
  selectedItemDetail,
  scanStateLabel,
  scanStateData,
  scanSummary,
  scanActiveSourceCount,
  scanMissingSourceCount,
  importBusy,
  actionPendingLabel,
  workspaceError,
  itemDetailLoading,
  itemDetailError,
}: AppShellMetadataPaneProps) {
  const metadataCaption = selectedItemDetail
    ? resolveItemLocation(selectedItemDetail)
    : selectedLibraryDetail?.rootPath ?? 'Caption / 摘要预留，后续根据选中项显示真实内容。'

  return (
    <aside className="app-frame app-meta-root" data-slot="fg-meta-root">
      <section className="workspace-pane metadata-frame">
        <header className="workspace-pane-header metadata-header" data-slot="fg-meta-header">
          {selectedLibraryId === null ? (
            <div />
          ) : (
            <div className="pane-title-stack metadata-header-title">
              <h2>{resolveItemDisplayLabel(selectedItemDetail) || resolvePathLeaf(selectedLibraryDetail?.rootPath ?? '')}</h2>
              {selectedItemDetail === null ? null : (
                <p className="pane-title-caption">{resolveItemLocation(selectedItemDetail)}</p>
              )}
            </div>
          )}

          <div className="workspace-pane-actions metadata-header-g3">
            <span className="status-pill" data-state={scanStateData}>
              {importBusy && actionPendingLabel !== null ? `${actionPendingLabel} / ${scanStateLabel}` : scanStateLabel}
            </span>
          </div>
        </header>

        <div className="workspace-pane-main metadata-main" data-slot="fg-meta-main">
          {selectedLibraryId === null ? (
            <div className="workspace-stage compact">
              <span className="workspace-label">详情区</span>
              <strong>等待可用上下文</strong>
              <p>选择媒体库后，这里会显示当前库、扫描状态和当前条目的最小摘要。</p>
            </div>
          ) : (
            <div className="metadata-stack">
              {workspaceError === null ? null : <div className="error-text">{workspaceError}</div>}

              <section className="pane-section">
                <div className="panel-heading pane-section-heading">
                  <div>
                    <span className="workspace-label">Library</span>
                    <h3>当前媒体库</h3>
                  </div>
                </div>

                <dl className="kv-list">
                  <div>
                    <dt>Root Path</dt>
                    <dd>{selectedLibraryDetail?.rootPath ?? '未读取'}</dd>
                  </div>
                  <div>
                    <dt>Library Type</dt>
                    <dd>{selectedLibraryDetail?.libraryType ?? '未读取'}</dd>
                  </div>
                  <div>
                    <dt>Scan Mode</dt>
                    <dd>{selectedLibraryDetail?.scanMode ?? '未读取'}</dd>
                  </div>
                  <div>
                    <dt>Updated At</dt>
                    <dd>{selectedLibraryDetail === null ? '未读取' : formatDateTime(selectedLibraryDetail.updatedAt)}</dd>
                  </div>
                </dl>
              </section>

              <section className="pane-section">
                <div className="panel-heading pane-section-heading">
                  <div>
                    <span className="workspace-label">Scan Snapshot</span>
                    <h3>扫描摘要</h3>
                  </div>
                </div>

                <dl className="kv-list">
                  <div>
                    <dt>Task State</dt>
                    <dd>{scanStateLabel}</dd>
                  </div>
                  <div>
                    <dt>Progress</dt>
                    <dd>{scanSummary}</dd>
                  </div>
                  <div>
                    <dt>Active Sources</dt>
                    <dd>{scanActiveSourceCount}</dd>
                  </div>
                  <div>
                    <dt>Missing Sources</dt>
                    <dd>{scanMissingSourceCount}</dd>
                  </div>
                </dl>
              </section>

              <section className="pane-section">
                <div className="panel-heading pane-section-heading">
                  <div>
                    <span className="workspace-label">Current Item</span>
                    <h3>当前条目</h3>
                  </div>
                </div>

                {itemDetailLoading ? (
                  <div className="workspace-stage compact">
                    <span className="workspace-label">Item</span>
                    <strong>正在读取条目详情</strong>
                    <p>当前选中的条目详情正在从 repository 拉取。</p>
                  </div>
                ) : itemDetailError !== null ? (
                  <div className="error-text">{itemDetailError}</div>
                ) : selectedItemDetail === null ? (
                  <div className="workspace-stage compact">
                    <span className="workspace-label">Item</span>
                    <strong>当前没有可显示条目</strong>
                    <p>扫描后如果产生可浏览条目，Metadata 会在这里显示当前选中项摘要。</p>
                  </div>
                ) : (
                  <dl className="kv-list">
                    <div>
                      <dt>Asset ID</dt>
                      <dd>{selectedItemDetail.assetId}</dd>
                    </div>
                    <div>
                      <dt>MIME</dt>
                      <dd>{selectedItemDetail.mime}</dd>
                    </div>
                    <div>
                      <dt>Source Kind</dt>
                      <dd>{selectedItemDetail.sourceKind}</dd>
                    </div>
                    <div>
                      <dt>Location</dt>
                      <dd>{resolveItemLocation(selectedItemDetail)}</dd>
                    </div>
                  </dl>
                )}
              </section>
            </div>
          )}
        </div>

        <footer className="workspace-pane-footer metadata-footer" data-slot="fg-meta-footer">
          <div className="metadata-image-caption" data-slot="fg-meta-footer-caption">
            {metadataCaption}
          </div>
        </footer>
      </section>
    </aside>
  )
}
