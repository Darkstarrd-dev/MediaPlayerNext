import type { ThumbnailProfile } from '@mediaplayernext/contracts'

export const PAGE_WHEEL_DELTA_THRESHOLD_PX = 96
export const PAGE_WHEEL_SETTLE_MS = 160
export const GAP_SNAP_MIN_REMAINDER_PX = 36
export const GAP_SNAP_MIN_ADJUST_PX = 12

interface GapSnapTargetWidthInput {
  containerWidth: number
  columns: number
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
  const columns = Math.max(1, Math.round(input.columns))
  const cellSizePx = Math.max(48, Math.round(input.cellSizePx))
  const gapPx = Math.max(0, Math.round(input.gapPx))
  const minMainWidthPx = Math.max(1, Math.round(input.minMainWidthPx))
  const maxMainWidthPx = Math.max(minMainWidthPx, Math.round(input.maxMainWidthPx))

  if (containerWidth <= 0) {
    return null
  }

  const usedWidth = columns * cellSizePx + Math.max(0, columns - 1) * gapPx
  const remainder = Math.max(0, containerWidth - usedWidth)
  if (remainder < GAP_SNAP_MIN_REMAINDER_PX) {
    return null
  }

  const previousColumnsWidth = Math.max(cellSizePx, usedWidth - (cellSizePx + gapPx))
  const nextColumnsWidth = usedWidth + cellSizePx + gapPx

  const candidateWidths = [
    clampWidth(usedWidth, minMainWidthPx, maxMainWidthPx),
    clampWidth(previousColumnsWidth, minMainWidthPx, maxMainWidthPx),
    clampWidth(nextColumnsWidth, minMainWidthPx, maxMainWidthPx),
  ]

  let bestTarget: number | null = null
  let bestDistance = Number.POSITIVE_INFINITY
  for (const candidate of candidateWidths) {
    const distance = Math.abs(candidate - containerWidth)
    if (distance < bestDistance) {
      bestDistance = distance
      bestTarget = candidate
    }
  }

  if (bestTarget === null || Math.abs(bestTarget - containerWidth) < GAP_SNAP_MIN_ADJUST_PX) {
    return null
  }

  return bestTarget
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

  const currentSidebarWidthPx = Math.max(minSidebarWidthPx, Math.round(input.currentSidebarWidthPx))
  const currentMetaWidthPx = Math.max(minMetaWidthPx, Math.round(input.currentMetaWidthPx))
  const currentSum = Math.max(1, currentSidebarWidthPx + currentMetaWidthPx)
  const sidebarRatio = currentSidebarWidthPx / currentSum

  let sidebarWidthPx = clampWidth(
    Math.round(sideAndMetaTotal * sidebarRatio),
    minSidebarWidthPx,
    maxSidebarWidthPx,
  )
  let metaWidthPx = clampWidth(
    sideAndMetaTotal - sidebarWidthPx,
    minMetaWidthPx,
    maxMetaWidthPx,
  )
  sidebarWidthPx = clampWidth(sideAndMetaTotal - metaWidthPx, minSidebarWidthPx, maxSidebarWidthPx)

  const targetCombinedWidth = sideAndMetaTotal
  let combinedWidth = sidebarWidthPx + metaWidthPx
  if (combinedWidth < targetCombinedWidth) {
    const growthNeeded = targetCombinedWidth - combinedWidth
    const metaGrowth = Math.min(growthNeeded, maxMetaWidthPx - metaWidthPx)
    metaWidthPx += metaGrowth
    combinedWidth += metaGrowth
    const sidebarGrowth = Math.min(targetCombinedWidth - combinedWidth, maxSidebarWidthPx - sidebarWidthPx)
    sidebarWidthPx += sidebarGrowth
    combinedWidth += sidebarGrowth
  } else if (combinedWidth > targetCombinedWidth) {
    const shrinkNeeded = combinedWidth - targetCombinedWidth
    const metaShrink = Math.min(shrinkNeeded, metaWidthPx - minMetaWidthPx)
    metaWidthPx -= metaShrink
    combinedWidth -= metaShrink
    const sidebarShrink = Math.min(combinedWidth - targetCombinedWidth, sidebarWidthPx - minSidebarWidthPx)
    sidebarWidthPx -= sidebarShrink
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
