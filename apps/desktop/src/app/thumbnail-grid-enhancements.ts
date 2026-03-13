import type { ThumbnailProfile } from '@mediaplayernext/contracts'

export const PAGE_WHEEL_DELTA_THRESHOLD_PX = 96
export const PAGE_WHEEL_SETTLE_MS = 160
export const GAP_SNAP_MIN_REMAINDER_PX = 8
export const GAP_SNAP_MIN_ADJUST_PX = 12
export const GAP_SNAP_EXPAND_BUFFER_PX = 2

interface GapSnapTargetWidthInput {
  containerWidth: number
  gridUsedWidth: number
  cellSizePx: number
  gapPx: number
  minMainWidthPx: number
  maxMainWidthPx: number
}

interface GapSnapPaneWidthsInput {
  availableWidth: number
  targetMainWidthPx: number
  currentSidebarWidthPx: number
  currentMetaWidthPx: number
}

interface GapSnapPaneWidthsResult {
  sidebarWidthPx: number
  metaWidthPx: number
}

export function resolveThumbnailProfileForGrid(
  cellSizePx: number,
  devicePixelRatio =
    typeof window === 'undefined' ? 1 : Math.max(1, window.devicePixelRatio ?? 1),
): ThumbnailProfile {
  const normalizedCellSize = Math.max(48, Math.round(cellSizePx))
  const normalizedDpr = Math.max(1, Math.min(3, devicePixelRatio))
  const targetEdge = Math.max(120, Math.min(1440, Math.ceil(normalizedCellSize * normalizedDpr)))

  if (targetEdge <= 240) {
    return 'grid-sm'
  }
  if (targetEdge <= 480) {
    return 'grid-md'
  }
  if (targetEdge <= 960) {
    return 'detail-md'
  }
  return 'detail-lg'
}

export function computeGapSnapTargetWidth(input: GapSnapTargetWidthInput): number | null {
  const containerWidth = Math.max(0, Math.round(input.containerWidth))
  const gridUsedWidth = Math.max(1, Math.round(input.gridUsedWidth))
  const cellSizePx = Math.max(48, Math.round(input.cellSizePx))
  const gapPx = Math.max(0, Math.round(input.gapPx))
  const minMainWidthPx = Math.max(1, Math.round(input.minMainWidthPx))
  const maxMainWidthPx = Math.max(minMainWidthPx, Math.round(input.maxMainWidthPx))

  if (containerWidth <= 0) {
    return null
  }

  const rightGap = containerWidth - gridUsedWidth
  if (Math.abs(rightGap) < GAP_SNAP_MIN_REMAINDER_PX) {
    return null
  }

  let mainDelta = -rightGap
  if (rightGap > 0) {
    const halfCell = cellSizePx * 0.5
    if (rightGap > halfCell) {
      const cellSpan = cellSizePx + gapPx
      mainDelta = cellSpan - rightGap + GAP_SNAP_EXPAND_BUFFER_PX
    }
  }

  const targetMainWidthPx = clampWidth(containerWidth + mainDelta, minMainWidthPx, maxMainWidthPx)
  if (Math.abs(targetMainWidthPx - containerWidth) < GAP_SNAP_MIN_ADJUST_PX) {
    return null
  }

  return targetMainWidthPx
}

export function resolveGapSnapPaneWidths(
  input: GapSnapPaneWidthsInput,
): GapSnapPaneWidthsResult | null {
  const availableWidth = Math.max(0, Math.round(input.availableWidth))
  if (availableWidth <= 0) {
    return null
  }

  const minSidebarWidthPx = Math.min(220, Math.max(160, Math.round(availableWidth * 0.22)))
  const minMetaWidthPx = Math.min(280, Math.max(200, Math.round(availableWidth * 0.24)))
  const minMainWidthPx = Math.min(420, Math.max(280, Math.round(availableWidth * 0.34)))
  const maxMainWidthPx = Math.max(minMainWidthPx, availableWidth - minSidebarWidthPx - minMetaWidthPx)
  const targetMainWidthPx = clampWidth(input.targetMainWidthPx, minMainWidthPx, maxMainWidthPx)
  const sideAndMetaTotal = Math.max(0, availableWidth - targetMainWidthPx)

  const maxSidebarWidthPx = Math.max(minSidebarWidthPx, availableWidth - minMetaWidthPx - minMainWidthPx)
  const maxMetaWidthPx = Math.max(minMetaWidthPx, availableWidth - minSidebarWidthPx - minMainWidthPx)

  let sidebarWidthPx = clampWidth(
    Math.round(input.currentSidebarWidthPx),
    minSidebarWidthPx,
    maxSidebarWidthPx,
  )
  let metaWidthPx = clampWidth(
    Math.round(input.currentMetaWidthPx),
    minMetaWidthPx,
    maxMetaWidthPx,
  )

  const currentMainWidthPx = Math.max(0, availableWidth - sidebarWidthPx - metaWidthPx)
  let mainDelta = targetMainWidthPx - currentMainWidthPx

  if (Math.abs(mainDelta) < GAP_SNAP_MIN_ADJUST_PX) {
    return null
  }

  if (mainDelta > 0) {
    const metaShrink = Math.min(mainDelta, metaWidthPx - minMetaWidthPx)
    metaWidthPx -= metaShrink
    mainDelta -= metaShrink

    const sidebarShrink = Math.min(mainDelta, sidebarWidthPx - minSidebarWidthPx)
    sidebarWidthPx -= sidebarShrink
    mainDelta -= sidebarShrink
  } else {
    const growthNeeded = -mainDelta
    const metaGrowth = Math.min(growthNeeded, maxMetaWidthPx - metaWidthPx)
    metaWidthPx += metaGrowth
    mainDelta += metaGrowth

    const sidebarGrowth = Math.min(-mainDelta, maxSidebarWidthPx - sidebarWidthPx)
    sidebarWidthPx += sidebarGrowth
    mainDelta += sidebarGrowth
  }

  if (Math.abs(mainDelta) >= GAP_SNAP_MIN_ADJUST_PX) {
    return null
  }

  const maxSideAndMetaTotal = maxSidebarWidthPx + maxMetaWidthPx
  if (sideAndMetaTotal > maxSideAndMetaTotal) {
    return null
  }

  const finalMainWidthPx = availableWidth - sidebarWidthPx - metaWidthPx
  if (Math.abs(finalMainWidthPx - targetMainWidthPx) > GAP_SNAP_MIN_ADJUST_PX) {
    return null
  }

  return {
    sidebarWidthPx,
    metaWidthPx,
  }
}

function clampWidth(value: number, min: number, max: number): number {
  return Math.max(min, Math.min(max, Math.round(value)))
}
