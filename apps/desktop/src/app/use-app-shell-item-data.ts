import type { ItemDetail, ItemListEntry } from '@mediaplayernext/contracts'
import { useEffect, useRef } from 'react'
import type { MediaRepository } from '../repositories/media-repository'
import { getErrorMessage } from './app-shell-utils'
import { resolveThumbnailProfileForGrid } from './thumbnail-grid-enhancements'

const THUMBNAIL_ENSURE_MAX_CONCURRENT = 2

interface UseAppShellItemDataParams {
  repository: MediaRepository
  itemDetailRequestIdRef: { current: number }
  selectedAssetId: string | null
  items: ItemListEntry[]
  thumbnailCellSizePx: number
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
    thumbnailCellSizePx,
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
    const thumbnailProfile = resolveThumbnailProfileForGrid(thumbnailCellSizePx)
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

    const pendingAssetIds = items
      .filter(
        (item) =>
          item.thumbnailKey == null ||
          item.thumbnailKey.length === 0 ||
          thumbnailProfile !== 'grid-md',
      )
      .map((item) => item.assetId)

    const workerCount = Math.min(THUMBNAIL_ENSURE_MAX_CONCURRENT, pendingAssetIds.length)
    const workers = Array.from(
      {
        length: workerCount,
      },
      async () => {
        while (pendingAssetIds.length > 0 && isCurrentRequest()) {
          const assetId = pendingAssetIds.shift()
          if (!assetId) {
            return
          }

          try {
            const thumbnailKey = await resolveThumbnailKey(assetId)
            applyThumbnailUrl(assetId, repository.urls.thumbnail(thumbnailKey))
          } catch {
            // ignore missing thumbnail and keep placeholder
          }
        }
      },
    )

    for (const item of items) {
      if (item.thumbnailKey) {
        applyThumbnailUrl(item.assetId, repository.urls.thumbnail(item.thumbnailKey))
      }
    }

    void Promise.all(workers)

    return () => {
      if (thumbnailEnsureRequestIdRef.current === requestId) {
        thumbnailEnsureRequestIdRef.current += 1
      }
    }
  }, [items, repository, setItemThumbnailUrls, thumbnailCellSizePx])
}
