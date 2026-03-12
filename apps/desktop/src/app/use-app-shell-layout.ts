import type { CSSProperties } from 'react'
import { useCallback, useMemo, useState } from 'react'
import {
  clampNumber,
  readSessionNumber,
  resolveSpacingPx,
  resolveWorkspaceWidths,
} from './app-shell-utils'
import {
  DEFAULT_LAYOUT_GAP_SCALE_COEFF,
  DEFAULT_META_WIDTH_PX,
  DEFAULT_PANE_INNER_GAP_SCALE_COEFF,
  DEFAULT_PANE_STACK_GAP_SCALE_COEFF,
  DEFAULT_SETTINGS_BACKDROP_OPACITY,
  DEFAULT_SIDEBAR_WIDTH_PX,
  DEFAULT_SPLITTER_WIDTH_SCALE_COEFF,
  DEFAULT_THUMBNAIL_ZOOM_LEVEL,
  DEFAULT_VIEWPORT_WIDTH,
  SETTINGS_STORAGE_KEYS,
  THUMBNAIL_GRID_GAP_PX,
  THUMBNAIL_GRID_MIN_CELL_PX,
} from './app-shell-layout-constants'
import {
  computeThumbnailGridLayout,
  THUMBNAIL_ZOOM_LEVELS,
  toThumbnailZoomLevel,
  type ThumbnailZoomLevel,
} from './thumbnail-grid-layout'
import { useAppShellLayoutEffects } from './use-app-shell-layout-effects'

export type DragTarget = 'left' | 'right'

export interface DragState {
  target: DragTarget
  startX: number
  startSidebarWidthPx: number
  startMetaWidthPx: number
}

export function useAppShellLayout() {
  const [mainGridElement, setMainGridElement] = useState<HTMLDivElement | null>(null)
  const [viewportWidth, setViewportWidth] = useState(DEFAULT_VIEWPORT_WIDTH)
  const [settingsBackdropOpacity, setSettingsBackdropOpacity] = useState(() =>
    readSessionNumber(
      SETTINGS_STORAGE_KEYS.settingsBackdropOpacity,
      DEFAULT_SETTINGS_BACKDROP_OPACITY,
      0,
      100,
    ),
  )
  const [layoutGapScaleCoeff, setLayoutGapScaleCoeff] = useState(() =>
    readSessionNumber(
      SETTINGS_STORAGE_KEYS.layoutGapScaleCoeff,
      DEFAULT_LAYOUT_GAP_SCALE_COEFF,
      0,
      3,
    ),
  )
  const [paneInnerGapScaleCoeff, setPaneInnerGapScaleCoeff] = useState(() =>
    readSessionNumber(
      SETTINGS_STORAGE_KEYS.paneInnerGapScaleCoeff,
      DEFAULT_PANE_INNER_GAP_SCALE_COEFF,
      0,
      2,
    ),
  )
  const [paneStackGapScaleCoeff, setPaneStackGapScaleCoeff] = useState(() =>
    readSessionNumber(
      SETTINGS_STORAGE_KEYS.paneStackGapScaleCoeff,
      DEFAULT_PANE_STACK_GAP_SCALE_COEFF,
      0,
      2,
    ),
  )
  const [splitterWidthScaleCoeff, setSplitterWidthScaleCoeff] = useState(() =>
    readSessionNumber(
      SETTINGS_STORAGE_KEYS.splitterWidthScaleCoeff,
      DEFAULT_SPLITTER_WIDTH_SCALE_COEFF,
      0.5,
      2,
    ),
  )
  const [sidebarWidthPx, setSidebarWidthPx] = useState(() =>
    readSessionNumber(
      SETTINGS_STORAGE_KEYS.sidebarWidthPx,
      DEFAULT_SIDEBAR_WIDTH_PX,
      160,
      640,
    ),
  )
  const [metaWidthPx, setMetaWidthPx] = useState(() =>
    readSessionNumber(SETTINGS_STORAGE_KEYS.metaWidthPx, DEFAULT_META_WIDTH_PX, 200, 720),
  )
  const [dragState, setDragState] = useState<DragState | null>(null)
  const [mainGridSize, setMainGridSize] = useState({ width: 960, height: 640 })
  const [thumbnailZoomLevel, setThumbnailZoomLevel] = useState<ThumbnailZoomLevel>(() =>
    toThumbnailZoomLevel(
      readSessionNumber(
        SETTINGS_STORAGE_KEYS.thumbnailZoomLevel,
        DEFAULT_THUMBNAIL_ZOOM_LEVEL,
        THUMBNAIL_ZOOM_LEVELS[0],
        THUMBNAIL_ZOOM_LEVELS[THUMBNAIL_ZOOM_LEVELS.length - 1],
      ),
    ),
  )

  const layoutPreview = useMemo(() => {
    const normalizedLayoutGapScaleCoeff = clampNumber(layoutGapScaleCoeff, 0, 3)
    const normalizedPaneInnerGapScaleCoeff = clampNumber(paneInnerGapScaleCoeff, 0, 2)
    const normalizedPaneStackGapScaleCoeff = clampNumber(paneStackGapScaleCoeff, 0, 2)
    const normalizedSplitterWidthScaleCoeff = clampNumber(splitterWidthScaleCoeff, 0.5, 2)
    const layoutGapPx = resolveSpacingPx(viewportWidth, normalizedLayoutGapScaleCoeff)
    const paneInnerPaddingPx = resolveSpacingPx(viewportWidth, normalizedPaneInnerGapScaleCoeff)
    const paneStackGapPx = Math.max(
      0,
      Math.round(paneInnerPaddingPx * 0.75 * normalizedPaneStackGapScaleCoeff),
    )
    const splitterWidthPx = Math.max(0, Math.round(layoutGapPx * normalizedSplitterWidthScaleCoeff))
    const paneHeaderHeightPx = Math.max(68, Math.round(paneInnerPaddingPx * 3.2))
    const paneFooterHeightPx = Math.max(48, Math.round(paneInnerPaddingPx * 2.2))

    return {
      layoutGapPx,
      paneInnerPaddingPx,
      paneStackGapPx,
      paneHeaderHeightPx,
      paneFooterHeightPx,
      splitterWidthPx,
      normalizedLayoutGapScaleCoeff,
      normalizedPaneInnerGapScaleCoeff,
      normalizedPaneStackGapScaleCoeff,
      normalizedSplitterWidthScaleCoeff,
    }
  }, [
    layoutGapScaleCoeff,
    paneInnerGapScaleCoeff,
    paneStackGapScaleCoeff,
    splitterWidthScaleCoeff,
    viewportWidth,
  ])

  const workspaceLayout = useMemo(
    () =>
      resolveWorkspaceWidths(
        viewportWidth,
        layoutPreview.layoutGapPx,
        layoutPreview.splitterWidthPx,
        sidebarWidthPx,
        metaWidthPx,
      ),
    [layoutPreview.layoutGapPx, layoutPreview.splitterWidthPx, metaWidthPx, sidebarWidthPx, viewportWidth],
  )

  const workspaceStyle = useMemo(
    () =>
      ({
        '--app-sidebar-width-px': `${workspaceLayout.sidebarWidthPx}px`,
        '--app-meta-width-px': `${workspaceLayout.metaWidthPx}px`,
      }) as CSSProperties,
    [workspaceLayout.metaWidthPx, workspaceLayout.sidebarWidthPx],
  )

  const thumbnailGridLayout = useMemo(
    () =>
      computeThumbnailGridLayout({
        containerWidth: mainGridSize.width,
        containerHeight: mainGridSize.height,
        zoomLevel: thumbnailZoomLevel,
        gapPx: THUMBNAIL_GRID_GAP_PX,
        minCellSizePx: THUMBNAIL_GRID_MIN_CELL_PX,
      }),
    [mainGridSize.height, mainGridSize.width, thumbnailZoomLevel],
  )

  const itemGridStyle = useMemo(
    () =>
      ({
        gridTemplateColumns: `repeat(${thumbnailGridLayout.columns}, minmax(0, ${thumbnailGridLayout.cellSizePx}px))`,
        gap: `${thumbnailGridLayout.gapPx}px`,
      }) as CSSProperties,
    [thumbnailGridLayout.cellSizePx, thumbnailGridLayout.columns, thumbnailGridLayout.gapPx],
  )

  const beginSplitterDrag = useCallback(
    (target: DragTarget, startX: number) => {
      setDragState({
        target,
        startX,
        startSidebarWidthPx: workspaceLayout.sidebarWidthPx,
        startMetaWidthPx: workspaceLayout.metaWidthPx,
      })
    },
    [workspaceLayout.metaWidthPx, workspaceLayout.sidebarWidthPx],
  )

  useAppShellLayoutEffects({
    settingsBackdropOpacity,
    layoutGapScaleCoeff,
    paneInnerGapScaleCoeff,
    paneStackGapScaleCoeff,
    splitterWidthScaleCoeff,
    thumbnailZoomLevel,
    layoutPreview,
    workspaceLayout,
    dragState,
    setViewportWidth,
    setSidebarWidthPx,
    setMetaWidthPx,
    setDragState,
    mainGridElement,
    setMainGridSize,
  })

  return {
    dragState,
    beginSplitterDrag,
    itemGridStyle,
    layoutPreview,
    setMainGridElement,
    setSettingsBackdropOpacity,
    setLayoutGapScaleCoeff,
    setPaneInnerGapScaleCoeff,
    setPaneStackGapScaleCoeff,
    setSplitterWidthScaleCoeff,
    settingsBackdropOpacity,
    layoutGapScaleCoeff,
    paneInnerGapScaleCoeff,
    paneStackGapScaleCoeff,
    splitterWidthScaleCoeff,
    thumbnailGridLayout,
    thumbnailZoomLevel,
    setThumbnailZoomLevel,
    workspaceLayout,
    workspaceStyle,
  }
}
