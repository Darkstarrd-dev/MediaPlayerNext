import type {
  ItemDetail,
  ItemListEntry,
  ThumbnailProfile,
} from '@mediaplayernext/contracts'
import { useEffect, useRef } from 'react'
import type { MediaRepository } from '../repositories/media-repository'
import { getErrorMessage } from './app-shell-utils'

const THUMBNAIL_ENSURE_MAX_CONCURRENT = 6
const THUMBNAIL_WARMUP_RADIUS = 1
const THUMBNAIL_WARMUP_MAX_CONCURRENT = 2

interface UseAppShellItemDataParams {
  repository: MediaRepository
  itemDetailRequestIdRef: { current: number }
  selectedAssetId: string | null
  items: ItemListEntry[]
  selectedLibraryId: string | null
  selectedMediaSourceId: string | null
  itemsPageIndex: number
  itemsHasNextPage: boolean
  thumbnailPageSize: number
  thumbnailProfile: ThumbnailProfile
  setSelectedItemDetail: (value: ItemDetail | null) => void
  setItemDetailError: (value: string | null) => void
  setItemDetailLoading: (value: boolean) => void
  setItemThumbnailUrls: (
    updater: Record<string, string> | ((current: Record<string, string>) => Record<string, string>),
  ) => void
}

export function useAppShellItemData(params: UseAppShellItemDataParams) {
  const {
    repository,
    itemDetailRequestIdRef,
    selectedAssetId,
    items,
    selectedLibraryId,
    selectedMediaSourceId,
    itemsPageIndex,
    itemsHasNextPage,
    thumbnailPageSize,
    thumbnailProfile,
    setSelectedItemDetail,
    setItemDetailError,
    setItemDetailLoading,
    setItemThumbnailUrls,
  } = params
  const thumbnailEnsureRequestIdRef = useRef(0)
  const thumbnailEnsureInFlightRef = useRef(new Map<string, Promise<string>>())

  useEffect(() => {
    if (selectedAssetId === null) {
      itemDetailRequestIdRef.current += 1
      setSelectedItemDetail(null)
      setItemDetailError(null)
      setItemDetailLoading(false)
      return
    }

    const requestId = itemDetailRequestIdRef.current + 1
    itemDetailRequestIdRef.current = requestId
    setItemDetailLoading(true)
    setItemDetailError(null)

    void repository.items
      .detail(selectedAssetId)
      .then((detail) => {
        if (itemDetailRequestIdRef.current !== requestId) {
          return
        }

        setSelectedItemDetail(detail)
      })
      .catch((error: unknown) => {
        if (itemDetailRequestIdRef.current !== requestId) {
          return
        }

        setSelectedItemDetail(null)
        setItemDetailError(getErrorMessage(error))
      })
      .finally(() => {
        if (itemDetailRequestIdRef.current === requestId) {
          setItemDetailLoading(false)
        }
      })
  }, [
    itemDetailRequestIdRef,
    repository,
    selectedAssetId,
    setItemDetailError,
    setItemDetailLoading,
    setSelectedItemDetail,
  ])

  useEffect(() => {
    if (items.length === 0) {
      thumbnailEnsureRequestIdRef.current += 1
      setItemThumbnailUrls({})
      return
    }

    const requestId = thumbnailEnsureRequestIdRef.current + 1
    thumbnailEnsureRequestIdRef.current = requestId
    const visibleAssetIds = new Set(items.map((item) => item.assetId))

    setItemThumbnailUrls((current) => {
      const next: Record<string, string> = {}
      for (const [assetId, url] of Object.entries(current)) {
        if (visibleAssetIds.has(assetId)) {
          next[assetId] = url
        }
      }
      return next
    })

    const isCurrentRequest = () => thumbnailEnsureRequestIdRef.current === requestId

    const applyThumbnailUrl = (assetId: string, thumbnailUrl: string) => {
      if (!isCurrentRequest()) {
        return
      }

      setItemThumbnailUrls((current) => {
        if (current[assetId] === thumbnailUrl) {
          return current
        }

        return {
          ...current,
          [assetId]: thumbnailUrl,
        }
      })
    }

    const resolveThumbnailKey = (assetId: string): Promise<string> => {
      const inflightKey = `${assetId}:${thumbnailProfile}`
      const inFlight = thumbnailEnsureInFlightRef.current.get(inflightKey)
      if (inFlight) {
        return inFlight
      }

      const request = repository.thumbnail
        .ensure(assetId, thumbnailProfile)
        .then((ensuredThumbnail) => ensuredThumbnail.thumbnailKey)
        .finally(() => {
          const currentRequest = thumbnailEnsureInFlightRef.current.get(inflightKey)
          if (currentRequest === request) {
            thumbnailEnsureInFlightRef.current.delete(inflightKey)
          }
        })

      thumbnailEnsureInFlightRef.current.set(inflightKey, request)
      return request
    }

    const ensureAssetIds = async (assetIds: string[], maxConcurrent: number, applyThumbnail: boolean) => {
      const workerCount = Math.min(Math.max(1, maxConcurrent), assetIds.length)
      const workers = Array.from(
        {
          length: workerCount,
        },
        async () => {
          while (assetIds.length > 0 && isCurrentRequest()) {
            const assetId = assetIds.shift()
            if (!assetId) {
              return
            }

            try {
              const thumbnailKey = await resolveThumbnailKey(assetId)
              if (applyThumbnail) {
                applyThumbnailUrl(assetId, repository.urls.thumbnail(thumbnailKey))
              }
            } catch {
              // ignore missing thumbnail and keep placeholder
            }
          }
        },
      )

      await Promise.all(workers)
    }

    const pendingVisibleAssetIds = items
      .filter((item) => item.thumbnailKey == null || item.thumbnailKey.length === 0)
      .map((item) => item.assetId)

    for (const item of items) {
      if (item.thumbnailKey) {
        applyThumbnailUrl(item.assetId, repository.urls.thumbnail(item.thumbnailKey))
      }
    }

    const warmupAdjacentPages = async () => {
      if (!isCurrentRequest() || selectedLibraryId === null || THUMBNAIL_WARMUP_RADIUS <= 0) {
        return
      }

      const effectivePageSize = Math.max(1, thumbnailPageSize)
      const targetPageIndexes: number[] = []

      for (let offset = 1; offset <= THUMBNAIL_WARMUP_RADIUS; offset += 1) {
        const previousPageIndex = itemsPageIndex - offset
        if (previousPageIndex >= 1) {
          targetPageIndexes.push(previousPageIndex)
        }

        const nextPageIndex = itemsPageIndex + offset
        if (offset === 1 && itemsHasNextPage) {
          targetPageIndexes.push(nextPageIndex)
        }
      }

      if (targetPageIndexes.length === 0) {
        return
      }

      const warmupAssetIds = new Set<string>()
      for (const pageIndex of targetPageIndexes) {
        if (!isCurrentRequest()) {
          return
        }

        try {
          const warmupPage = await repository.items.list({
            libraryId: selectedLibraryId,
            mediaSourceId: selectedMediaSourceId ?? undefined,
            thumbnailProfile,
            page: pageIndex,
            pageSize: effectivePageSize,
          })

          for (const item of warmupPage.items) {
            if (item.thumbnailKey && item.thumbnailKey.length > 0) {
              continue
            }
            if (visibleAssetIds.has(item.assetId)) {
              continue
            }
            warmupAssetIds.add(item.assetId)
          }
        } catch {
          return
        }
      }

      if (!isCurrentRequest() || warmupAssetIds.size === 0) {
        return
      }

      await ensureAssetIds([...warmupAssetIds], THUMBNAIL_WARMUP_MAX_CONCURRENT, false)
    }

    void (async () => {
      await ensureAssetIds(pendingVisibleAssetIds, THUMBNAIL_ENSURE_MAX_CONCURRENT, true)
      await warmupAdjacentPages()
    })()

    return () => {
      if (thumbnailEnsureRequestIdRef.current === requestId) {
        thumbnailEnsureRequestIdRef.current += 1
      }
    }
  }, [
    items,
    itemsHasNextPage,
    itemsPageIndex,
    repository,
    selectedLibraryId,
    selectedMediaSourceId,
    setItemThumbnailUrls,
    thumbnailPageSize,
    thumbnailProfile,
  ])
}
