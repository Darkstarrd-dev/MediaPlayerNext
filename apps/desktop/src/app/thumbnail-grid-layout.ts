export const THUMBNAIL_ZOOM_LEVELS = [1, 2, 3, 4, 5, 6, 7] as const

export type ThumbnailZoomLevel = (typeof THUMBNAIL_ZOOM_LEVELS)[number]

export interface ThumbnailGridLayoutInput {
  containerWidth: number
  containerHeight: number
  zoomLevel: ThumbnailZoomLevel
  gapPx: number
  minCellSizePx: number
  cardChromePx?: number
}

export interface ThumbnailGridLayoutResult {
  columns: number
  rows: number
  cellSizePx: number
  cellWidth: number
  mediaHeight: number
  pageSize: number
  gapPx: number
  baseGapPx: number
  renderGapPx: number
  idealGridWidth: number
  idealGridHeight: number
  zoomLevel: number
  zoomLevelCount: number
  zoomValue: number
  cardChromePx: number
}

const MIN_DIM_PX = 36
const FALLBACK_COLUMNS = 3
const FALLBACK_CELL_SIZE_PX = 160
const DEFAULT_CARD_CHROME_PX = 2

function floorPx(value: number): number {
  return Math.floor(value)
}

function clampInteger(value: number, min: number, max: number): number {
  return Math.max(min, Math.min(max, Math.round(value)))
}

function pickClosestCols(canvasWidth: number, cellSize: number, gapPx: number): number {
  const approxColumns = Math.max(1, Math.round((canvasWidth + gapPx) / (cellSize + gapPx)))
  let bestColumns = 1
  let bestDistance = Number.POSITIVE_INFINITY
  let bestOverflows = true

  for (let columns = Math.max(1, approxColumns - 2); columns <= approxColumns + 2; columns += 1) {
    const usedWidth = columns * cellSize + (columns - 1) * gapPx
    const distance = Math.abs(usedWidth - canvasWidth)
    const overflows = usedWidth > canvasWidth + 0.5

    if (bestOverflows && !overflows) {
      bestColumns = columns
      bestDistance = distance
      bestOverflows = false
      continue
    }

    if (!bestOverflows && overflows) {
      continue
    }

    if (distance < bestDistance - 0.001) {
      bestColumns = columns
      bestDistance = distance
      bestOverflows = overflows
      continue
    }

    if (Math.abs(distance - bestDistance) <= 0.001) {
      if (!overflows && columns > bestColumns) {
        bestColumns = columns
      } else if (overflows && columns < bestColumns) {
        bestColumns = columns
      }
    }
  }

  return bestColumns
}

export function computeRenderGap(input: {
  gridWidth: number
  columns: number
  cellWidth: number
  baseGapPx: number
}): number {
  const columns = Math.max(1, Math.round(input.columns))
  const gridWidth = Math.max(0, input.gridWidth)
  const cellWidth = Math.max(1, input.cellWidth)
  const baseGapPx = Math.max(0, input.baseGapPx)

  if (columns <= 1) {
    return baseGapPx
  }

  const totalGapSpace = gridWidth - columns * cellWidth
  if (totalGapSpace <= 0) {
    return baseGapPx
  }

  const perGap = totalGapSpace / (columns - 1)
  const maxPerGap = baseGapPx + Math.max(2, cellWidth * 0.08)
  return perGap <= maxPerGap ? perGap : baseGapPx
}

export function resolveThumbnailCardChromePx(): number {
  if (typeof window === 'undefined' || typeof document === 'undefined') {
    return DEFAULT_CARD_CHROME_PX
  }

  const itemCard = document.querySelector<HTMLElement>('.item-card-button')
  if (itemCard) {
    const styles = window.getComputedStyle(itemCard)
    const paddingLeft = Number.parseFloat(styles.paddingLeft) || 0
    const paddingRight = Number.parseFloat(styles.paddingRight) || 0
    const borderLeft = Number.parseFloat(styles.borderLeftWidth) || 0
    const borderRight = Number.parseFloat(styles.borderRightWidth) || 0
    const chrome = paddingLeft + paddingRight + borderLeft + borderRight
    if (Number.isFinite(chrome)) {
      return Math.max(0, floorPx(chrome))
    }
  }

  const rootStyles = window.getComputedStyle(document.documentElement)
  const borderWidthRaw = rootStyles.getPropertyValue('--mpx-card-border-width').trim()
  const borderWidth = Number.parseFloat(borderWidthRaw)
  if (Number.isFinite(borderWidth)) {
    return Math.max(0, floorPx(borderWidth * 2))
  }

  return DEFAULT_CARD_CHROME_PX
}

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
  const width = Number.isFinite(input.containerWidth) ? Math.max(0, floorPx(input.containerWidth)) : 0
  const height = Number.isFinite(input.containerHeight) ? Math.max(0, floorPx(input.containerHeight)) : 0
  const baseGapPx = Number.isFinite(input.gapPx) ? Math.max(0, Math.round(input.gapPx)) : 0
  const cardChromePx = Number.isFinite(input.cardChromePx)
    ? Math.max(0, floorPx(input.cardChromePx ?? DEFAULT_CARD_CHROME_PX))
    : DEFAULT_CARD_CHROME_PX
  const minCellSizePx = Number.isFinite(input.minCellSizePx)
    ? Math.max(48, Math.round(input.minCellSizePx))
    : 96
  const targetRows = clampInteger(input.zoomLevel, 1, THUMBNAIL_ZOOM_LEVELS.length)

  if (width < 1 || height < 1) {
    const rows = targetRows
    return {
      columns: FALLBACK_COLUMNS,
      rows,
      cellSizePx: FALLBACK_CELL_SIZE_PX,
      cellWidth: FALLBACK_CELL_SIZE_PX,
      mediaHeight: Math.max(MIN_DIM_PX, FALLBACK_CELL_SIZE_PX - cardChromePx),
      pageSize: FALLBACK_COLUMNS * rows,
      gapPx: baseGapPx,
      baseGapPx,
      renderGapPx: baseGapPx,
      idealGridWidth: FALLBACK_COLUMNS * FALLBACK_CELL_SIZE_PX + (FALLBACK_COLUMNS - 1) * baseGapPx,
      idealGridHeight: rows * FALLBACK_CELL_SIZE_PX + (rows - 1) * baseGapPx,
      zoomLevel: rows,
      zoomLevelCount: THUMBNAIL_ZOOM_LEVELS.length,
      zoomValue: Math.max(MIN_DIM_PX, FALLBACK_CELL_SIZE_PX - cardChromePx),
      cardChromePx,
    }
  }

  let rows = targetRows
  while (rows > 1) {
    const mediaByHeight = floorPx((height - (rows - 1) * baseGapPx) / rows - cardChromePx)
    if (mediaByHeight >= MIN_DIM_PX && mediaByHeight + cardChromePx >= minCellSizePx) {
      break
    }
    rows -= 1
  }

  const mediaByHeight = floorPx((height - (rows - 1) * baseGapPx) / rows - cardChromePx)
  const mediaByWidth = floorPx(width - cardChromePx)
  const mediaHeight = Math.max(MIN_DIM_PX, floorPx(Math.min(mediaByHeight, mediaByWidth)))
  const cellWidth = mediaHeight + cardChromePx
  const columns = pickClosestCols(width, cellWidth, baseGapPx)
  const idealGridWidth = columns * cellWidth + Math.max(0, columns - 1) * baseGapPx
  const idealGridHeight = rows * (mediaHeight + cardChromePx) + Math.max(0, rows - 1) * baseGapPx
  const renderGapPx = computeRenderGap({
    gridWidth: width,
    columns,
    cellWidth,
    baseGapPx,
  })
  const gapPx = Math.max(0, renderGapPx)

  return {
    columns,
    rows,
    cellSizePx: floorPx(cellWidth),
    cellWidth: floorPx(cellWidth),
    mediaHeight: floorPx(mediaHeight),
    pageSize: Math.max(1, columns * rows),
    gapPx,
    baseGapPx,
    renderGapPx: gapPx,
    idealGridWidth: floorPx(idealGridWidth),
    idealGridHeight: floorPx(idealGridHeight),
    zoomLevel: rows,
    zoomLevelCount: THUMBNAIL_ZOOM_LEVELS.length,
    zoomValue: floorPx(mediaHeight),
    cardChromePx,
  }
}
