export function buildThumbnailUrl(thumbnailKey: string): string {
  return `thumb://cache/${encodeURIComponent(thumbnailKey)}`
}

export function buildMediaUrl(assetId: string): string {
  return `media://asset/${encodeURIComponent(assetId)}`
}

export function buildArchiveEntryUrl(archiveEntryId: string): string {
  return `archive://entry/${encodeURIComponent(archiveEntryId)}`
}
