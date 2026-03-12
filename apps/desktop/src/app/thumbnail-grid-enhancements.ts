import type { ThumbnailProfile } from '@mediaplayernext/contracts'

export const PAGE_WHEEL_DELTA_THRESHOLD_PX = 96
export const PAGE_WHEEL_SETTLE_MS = 160

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
