import type { ThumbnailZoomLevel } from './thumbnail-grid-layout'

export const DEFAULT_VIEWPORT_WIDTH = 1280
export const DEFAULT_SETTINGS_BACKDROP_OPACITY = 18
export const DEFAULT_LAYOUT_GAP_SCALE_COEFF = 1
export const DEFAULT_PANE_INNER_GAP_SCALE_COEFF = 1
export const DEFAULT_PANE_STACK_GAP_SCALE_COEFF = 1
export const DEFAULT_SPLITTER_WIDTH_SCALE_COEFF = 1
export const DEFAULT_SIDEBAR_WIDTH_PX = 300
export const DEFAULT_META_WIDTH_PX = 340
export const DEFAULT_THUMBNAIL_ZOOM_LEVEL: ThumbnailZoomLevel = 4
export const THUMBNAIL_GRID_GAP_PX = 14
export const THUMBNAIL_GRID_MIN_CELL_PX = 96

export const SETTINGS_STORAGE_KEYS = {
  settingsBackdropOpacity: 'mpnext.ui.settingsBackdropOpacity',
  layoutGapScaleCoeff: 'mpnext.ui.layoutGapScaleCoeff',
  paneInnerGapScaleCoeff: 'mpnext.ui.paneInnerGapScaleCoeff',
  paneStackGapScaleCoeff: 'mpnext.ui.paneStackGapScaleCoeff',
  splitterWidthScaleCoeff: 'mpnext.ui.splitterWidthScaleCoeff',
  sidebarWidthPx: 'mpnext.ui.sidebarWidthPx',
  metaWidthPx: 'mpnext.ui.metaWidthPx',
  thumbnailZoomLevel: 'mpnext.ui.thumbnailZoomLevel',
} as const
