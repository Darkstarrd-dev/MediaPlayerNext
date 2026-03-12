export const THUMBNAIL_ZOOM_LEVELS = [1, 2, 3, 4, 5, 6, 7] as const

export type ThumbnailZoomLevel = (typeof THUMBNAIL_ZOOM_LEVELS)[number]

export interface ThumbnailGridLayoutInput {
  containerWidth: number
  containerHeight: number
  zoomLevel: ThumbnailZoomLevel
  gapPx: number
  minCellSizePx: number
}

export interface ThumbnailGridLayoutResult {
  columns: number
  rows: number
  cellSizePx: number
  pageSize: number
  gapPx: number
}

const FALLBACK_COLUMNS = 3
const FALLBACK_CELL_SIZE_PX = 160

export function toThumbnailZoomLevel(value: number): ThumbnailZoomLevel {
  if (value <= 1) {
    return 1
  }

  if (value >= 7) {
    return 7
  }

  return Math.round(value) as ThumbnailZoomLevel
}

export function computeThumbnailGridLayout(input: ThumbnailGridLayoutInput): ThumbnailGridLayoutResult {
  const width = Number.isFinite(input.containerWidth) ? Math.max(0, input.containerWidth) : 0
  const height = Number.isFinite(input.containerHeight) ? Math.max(0, input.containerHeight) : 0
  const gapPx = Number.isFinite(input.gapPx) ? Math.max(0, Math.round(input.gapPx)) : 0
  const minCellSizePx = Number.isFinite(input.minCellSizePx)
    ? Math.max(48, Math.round(input.minCellSizePx))
    : 96

  if (width < 1 || height < 1) {
    const rows = input.zoomLevel
    return {
      columns: FALLBACK_COLUMNS,
      rows,
      cellSizePx: FALLBACK_CELL_SIZE_PX,
      pageSize: FALLBACK_COLUMNS * rows,
      gapPx,
    }
  }

  const targetRows = input.zoomLevel
  const maxRowsByHeight = Math.max(1, Math.floor((height + gapPx) / (minCellSizePx + gapPx)))
  const rows = Math.max(1, Math.min(targetRows, maxRowsByHeight))
  const maxCellByHeight = (height - gapPx * (rows - 1)) / rows
  const minColumns = Math.max(1, Math.ceil((width + gapPx) / (Math.max(1, maxCellByHeight) + gapPx)))
  const rawCellSize = (width - gapPx * (minColumns - 1)) / minColumns
  const cellSizePx = Math.max(48, Math.floor(Math.min(rawCellSize, maxCellByHeight)))
  const columns = Math.max(1, minColumns)

  return {
    columns,
    rows,
    cellSizePx,
    pageSize: Math.max(1, columns * rows),
    gapPx,
  }
}
