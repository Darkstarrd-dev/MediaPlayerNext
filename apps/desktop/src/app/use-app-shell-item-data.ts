import type { ItemDetail, ItemListEntry } from '@mediaplayernext/contracts'
import { useEffect } from 'react'
import type { MediaRepository } from '../repositories/media-repository'
import { getErrorMessage } from './app-shell-utils'

interface UseAppShellItemDataParams {
  repository: MediaRepository
  itemDetailRequestIdRef: { current: number }
  selectedAssetId: string | null
  items: ItemListEntry[]
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
    setSelectedItemDetail,
    setItemDetailError,
    setItemDetailLoading,
    setItemThumbnailUrls,
  } = params

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
      setItemThumbnailUrls({})
      return
    }

    let disposed = false
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

    const applyThumbnailUrl = (assetId: string, thumbnailUrl: string) => {
      if (disposed) {
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

    for (const item of items) {
      if (item.thumbnailKey) {
        applyThumbnailUrl(item.assetId, repository.urls.thumbnail(item.thumbnailKey))
        continue
      }

      void repository.thumbnail
        .ensure(item.assetId, 'grid-md')
        .then((ensuredThumbnail) => {
          applyThumbnailUrl(item.assetId, repository.urls.thumbnail(ensuredThumbnail.thumbnailKey))
        })
        .catch(() => {
          // ignore missing thumbnail and keep placeholder
        })
    }

    return () => {
      disposed = true
    }
  }, [items, repository, setItemThumbnailUrls])
}
