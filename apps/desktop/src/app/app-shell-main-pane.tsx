import type { ItemListEntry, LibraryDetail } from '@mediaplayernext/contracts'
import type { CSSProperties, WheelEvent as ReactWheelEvent } from 'react'
import { resolvePathLeaf } from './app-shell-utils'
import {
  THUMBNAIL_ZOOM_LEVELS,
  toThumbnailZoomLevel,
  type ThumbnailZoomLevel,
} from './thumbnail-grid-layout'
import type { ItemsPageTransitionState } from './use-app-shell-workspace-state'

interface AppShellMainPaneProps {
  selectedLibraryId: string | null
  selectedLibraryDetail: LibraryDetail | null
  selectedLibraryRootPath: string | null
  thumbnailZoomLevel: ThumbnailZoomLevel
  workspaceHydrated: boolean
  workspaceRefreshing: boolean
  workspaceError: string | null
  items: ItemListEntry[]
  selectedAssetId: string | null
  itemThumbnailUrls: Record<string, string>
  itemGridStyle: CSSProperties
  mainFooterPrimary: string
  mainFooterSecondary: string
  mainFooterPageLabel: string
  mainFooterTransitionLabel: string | null
  pageTransitionState: ItemsPageTransitionState
  itemsHasNextPage: boolean
  itemsPageIndex: number
  setMainGridElement: (element: HTMLDivElement | null) => void
  onThumbnailZoomLevelChange: (value: ThumbnailZoomLevel) => void
  onSelectAsset: (assetId: string) => void
  onThumbnailError: (assetId: string) => void
  onGoPreviousItemsPage: () => void
  onGoNextItemsPage: () => void
  onMainGridWheel: (event: ReactWheelEvent<HTMLDivElement>) => void
}

export function AppShellMainPane({
  selectedLibraryId,
  selectedLibraryDetail,
  selectedLibraryRootPath,
  thumbnailZoomLevel,
  workspaceHydrated,
  workspaceRefreshing,
  workspaceError,
  items,
  selectedAssetId,
  itemThumbnailUrls,
  itemGridStyle,
  mainFooterPrimary,
  mainFooterSecondary,
  mainFooterPageLabel,
  mainFooterTransitionLabel,
  pageTransitionState,
  itemsHasNextPage,
  itemsPageIndex,
  setMainGridElement,
  onThumbnailZoomLevelChange,
  onSelectAsset,
  onThumbnailError,
  onGoPreviousItemsPage,
  onGoNextItemsPage,
  onMainGridWheel,
}: AppShellMainPaneProps) {
  return (
    <section className="app-frame app-main-root" data-slot="fg-main-root">
      <section className="workspace-pane main-pane-frame">
        <header className="workspace-pane-header main-header" data-slot="fg-main-header">
          {selectedLibraryDetail === null ? (
            <div />
          ) : (
            <h2 className="pane-title-single">{resolvePathLeaf(selectedLibraryRootPath ?? '')}</h2>
          )}

          <label className="main-zoom-control" aria-label="缩略图缩放级别">
            <span>缩放</span>
            <select
              className="main-zoom-select"
              value={thumbnailZoomLevel}
              onChange={(event) => onThumbnailZoomLevelChange(toThumbnailZoomLevel(Number(event.target.value)))}
              disabled={selectedLibraryId === null}
            >
              {THUMBNAIL_ZOOM_LEVELS.map((level) => (
                <option key={level} value={level}>
                  {level}
                </option>
              ))}
            </select>
          </label>
        </header>

        <div
          ref={setMainGridElement}
          className="workspace-pane-main main-pane-main"
          data-slot="fg-main-main"
          data-page-transition={pageTransitionState}
          onWheel={onMainGridWheel}
        >
          {selectedLibraryId === null ? (
            <div className="workspace-stage">
              <span className="workspace-label">主工作区</span>
              <strong>等待导入媒体库</strong>
              <p>完成路径登记后，这里会按当前媒体库快照显示条目预览，并与 Sidebar/Metadata 同步刷新。</p>
            </div>
          ) : !workspaceHydrated && workspaceRefreshing ? (
            <div className="workspace-stage">
              <span className="workspace-label">主工作区</span>
              <strong>正在刷新快照</strong>
              <p>正在读取当前媒体库的条目列表、扫描状态与详情摘要。</p>
            </div>
          ) : workspaceError !== null ? (
            <div className="error-text">{workspaceError}</div>
          ) : items.length === 0 ? (
            <div className="workspace-stage">
              <span className="workspace-label">Items</span>
              <strong>当前媒体库暂无条目</strong>
              <p>可以直接开始扫描，或回到导入面板登记新的本地路径。</p>
            </div>
          ) : (
            <div className="item-grid" style={itemGridStyle}>
              {items.map((item) => {
                const isActive = item.assetId === selectedAssetId
                const thumbnailUrl = itemThumbnailUrls[item.assetId] ?? null

                return (
                  <button
                    key={item.assetId}
                    className={`workspace-card-button item-card-button ${isActive ? 'is-active' : ''}`}
                    type="button"
                    aria-pressed={isActive}
                    onClick={() => onSelectAsset(item.assetId)}
                  >
                    {thumbnailUrl === null ? (
                      <div className="item-card-thumbnail item-card-thumbnail-placeholder">
                        <span>{item.sourceKind === 'archive_entry' ? 'Archive' : 'Media'}</span>
                      </div>
                    ) : (
                      <img
                        className="item-card-thumbnail"
                        src={thumbnailUrl}
                        alt=""
                        loading="lazy"
                        onError={() => onThumbnailError(item.assetId)}
                      />
                    )}
                  </button>
                )
              })}
            </div>
          )}
        </div>

        <footer className="workspace-pane-footer main-footer" data-slot="fg-main-footer">
          <div className="main-footer-meta" data-slot="fg-main-footer-meta">
            <span>{mainFooterPrimary}</span>
            <span>{mainFooterSecondary}</span>
          </div>

          <div className="main-footer-pagination" data-slot="fg-main-footer-pagination">
            <button
              className="mpx-btn pane-pagination-btn"
              type="button"
              disabled={selectedLibraryId === null || itemsPageIndex <= 1}
              onClick={onGoPreviousItemsPage}
            >
              Prev
            </button>
            <span>{mainFooterPageLabel}</span>
            <button
              className="mpx-btn pane-pagination-btn"
              type="button"
              disabled={selectedLibraryId === null || !itemsHasNextPage}
              onClick={onGoNextItemsPage}
            >
              Next
            </button>
          </div>
          {mainFooterTransitionLabel === null ? null : (
            <span className="main-footer-transition" data-state={pageTransitionState}>
              {mainFooterTransitionLabel}
            </span>
          )}
        </footer>
      </section>
    </section>
  )
}
