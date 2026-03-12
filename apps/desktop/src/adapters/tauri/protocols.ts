function shouldUseWindowsLocalhostBridge(): boolean {
  if (typeof window === 'undefined') {
    return false
  }

  const userAgentDataPlatform =
    (window.navigator as Navigator & { userAgentData?: { platform?: string } }).userAgentData?.platform ?? ''
  const navigatorPlatform = window.navigator.platform ?? ''
  const userAgent = window.navigator.userAgent ?? ''
  return /windows/i.test(`${userAgentDataPlatform} ${navigatorPlatform} ${userAgent}`)
}

function buildProtocolUrl(scheme: string, host: string, value: string): string {
  const encodedValue = encodeURIComponent(value)

  if (shouldUseWindowsLocalhostBridge()) {
    return `http://${scheme}.localhost/${host}/${encodedValue}`
  }

  return `${scheme}://${host}/${encodedValue}`
}

export function buildThumbnailUrl(thumbnailKey: string): string {
  return buildProtocolUrl('thumb', 'cache', thumbnailKey)
}

export function buildMediaUrl(assetId: string): string {
  return buildProtocolUrl('media', 'asset', assetId)
}

export function buildArchiveEntryUrl(archiveEntryId: string): string {
  return buildProtocolUrl('archive', 'entry', archiveEntryId)
}
