import type {
  ItemListEntry,
  LibraryDetail,
  ScanStats,
  TaskProgress,
} from '@mediaplayernext/contracts'
import { useCallback, useEffect, useRef } from 'react'
import type { MediaRepository } from '../repositories/media-repository'
import { getErrorMessage, isTaskNotFoundError } from './app-shell-utils'
import { useAppShellBootstrapScanSnapshot } from './use-app-shell-bootstrap-scan-snapshot'
import type { ItemsPageTransitionState } from './use-app-shell-workspace-state'

interface RefreshWorkspaceOptions {
  preferredLibraryId?: string | null
  preferredSidebarNodeId?: string | null
  preferredMediaSourceId?: string | null
  preferredPageIndex?: number
  preferredAssetId?: string | null
}

interface UseAppShellWorkspaceDataParams {
  repository: MediaRepository
  thumbnailPageSize: number
  libraryLoadRequestIdRef: { current: number }
  selectedLibraryId: string | null
  selectedMediaSourceId: string | null
  itemsPageIndex: number
  workspaceHydrated: boolean
  clearWorkspaceData: () => void
  setSelectedLibraryDetail: (value: LibraryDetail | null) => void
  setScanStats: (value: ScanStats | null) => void
  setScanSnapshot: (value: TaskProgress | null) => void
  setWorkspaceRefreshing: (value: boolean) => void
  setWorkspaceHydrated: (value: boolean) => void
  setWorkspaceError: (value: string | null) => void
  setItems: (value: ItemListEntry[]) => void
  setItemsPageIndex: (value: number) => void
  setItemsTargetPageIndex: (value: number) => void
  setItemsHasNextPage: (value: boolean) => void
  setItemsPageTransitionState: (value: ItemsPageTransitionState) => void
  setSelectedAssetId: (updater: string | null | ((current: string | null) => string | null)) => void
  setItemThumbnailUrls: (
    updater: Record<string, string> | ((current: Record<string, string>) => Record<string, string>),
  ) => void
  refreshLibraries: (preferredLibraryId?: string | null) => Promise<string | null>
  refreshSidebarNodes: (
    libraryId: string,
    preferredSidebarNodeId?: string | null,
    preferredMediaSourceId?: string | null,
  ) => Promise<{ selectedSidebarNodeId: string | null; selectedMediaSourceId: string | null }>
}

export function useAppShellWorkspaceData(params: UseAppShellWorkspaceDataParams) {
  const {
    repository,
    thumbnailPageSize,
    libraryLoadRequestIdRef,
    selectedLibraryId,
    selectedMediaSourceId,
    itemsPageIndex,
    workspaceHydrated,
    clearWorkspaceData,
    setSelectedLibraryDetail,
    setScanStats,
    setScanSnapshot,
    setWorkspaceRefreshing,
    setWorkspaceHydrated,
    setWorkspaceError,
    setItems,
    setItemsPageIndex,
    setItemsTargetPageIndex,
    setItemsHasNextPage,
    setItemsPageTransitionState,
    setSelectedAssetId,
    setItemThumbnailUrls,
    refreshLibraries,
    refreshSidebarNodes,
  } = params

  const bootstrapScanSnapshot = useAppShellBootstrapScanSnapshot({
    repository,
    setScanSnapshot,
  })

  const loadLibrarySurface = useCallback(
    async (options: {
      libraryId: string
      mediaSourceId: string | null
      requestedPageIndex: number
      preferredAssetId?: string | null
      includeWorkspaceSummary?: boolean
    }) => {
      const {
        libraryId,
        mediaSourceId,
        requestedPageIndex,
        preferredAssetId,
        includeWorkspaceSummary = false,
      } = options
      const requestId = libraryLoadRequestIdRef.current + 1
      libraryLoadRequestIdRef.current = requestId

      const useReadyCommit = workspaceHydrated && !includeWorkspaceSummary
      if (useReadyCommit) {
        setItemsTargetPageIndex(Math.max(1, requestedPageIndex))
        setItemsPageTransitionState('loading-next-page')
      } else {
        setItemsPageTransitionState('idle')
      }

      setWorkspaceRefreshing(true)
      setWorkspaceError(null)

      try {
        let resolvedPageIndex = Math.max(1, requestedPageIndex)
        const effectivePageSize = Math.max(1, thumbnailPageSize)
        const readItemsPage = (pageIndex: number) =>
          repository.items.list({
            libraryId,
            mediaSourceId: mediaSourceId ?? undefined,
            page: pageIndex,
            pageSize: effectivePageSize,
          })

        let nextItemsPage = await readItemsPage(resolvedPageIndex)
        while (resolvedPageIndex > 1 && nextItemsPage.items.length === 0) {
          resolvedPageIndex -= 1
          nextItemsPage = await readItemsPage(resolvedPageIndex)
        }

        const workspaceSummary = includeWorkspaceSummary
          ? await Promise.all([
              repository.library.get(libraryId),
              repository.scan.stats(libraryId),
              repository.scan.snapshot(libraryId).catch((error: unknown) => {
                if (isTaskNotFoundError(error)) {
                  return null
                }
                throw error
              }),
            ])
          : null

        if (libraryLoadRequestIdRef.current !== requestId) {
          return
        }

        const nextItems = nextItemsPage.items
        if (useReadyCommit) {
          setItemsPageTransitionState('committing')
        }

        if (workspaceSummary !== null) {
          const [detail, stats, snapshot] = workspaceSummary
          setSelectedLibraryDetail(detail)
          setScanStats(stats)
          setScanSnapshot(snapshot)
        }
        setItemsPageIndex(resolvedPageIndex)
        setItemsTargetPageIndex(resolvedPageIndex)
        setItemsHasNextPage(nextItemsPage.hasNextPage)
        setItems(nextItems)
        setWorkspaceHydrated(true)
        setItemThumbnailUrls((current) => {
          const allowedAssetIds = new Set(nextItems.map((item) => item.assetId))
          const next: Record<string, string> = {}
          for (const [assetId, url] of Object.entries(current)) {
            if (allowedAssetIds.has(assetId)) {
              next[assetId] = url
            }
          }
          return next
        })
        setSelectedAssetId((currentAssetId) => {
          if (preferredAssetId && nextItems.some((item) => item.assetId === preferredAssetId)) {
            return preferredAssetId
          }
          if (currentAssetId !== null && nextItems.some((item) => item.assetId === currentAssetId)) {
            return currentAssetId
          }
          return nextItems[0]?.assetId ?? null
        })
        setItemsPageTransitionState('idle')
      } catch (error) {
        if (libraryLoadRequestIdRef.current !== requestId) {
          return
        }

        setItemsPageTransitionState('idle')
        setItemsTargetPageIndex(itemsPageIndex)
        setWorkspaceError(getErrorMessage(error))
        if (!workspaceHydrated) {
          setSelectedLibraryDetail(null)
          setScanStats(null)
          setScanSnapshot(null)
          setItems([])
          setItemsPageIndex(1)
          setItemsTargetPageIndex(1)
          setItemsHasNextPage(false)
          setSelectedAssetId(null)
          setItemThumbnailUrls({})
        }
      } finally {
        if (libraryLoadRequestIdRef.current === requestId) {
          setWorkspaceRefreshing(false)
        }
      }
    },
    [
      libraryLoadRequestIdRef,
      repository,
      setItemThumbnailUrls,
      setItems,
      setItemsHasNextPage,
      setItemsPageIndex,
      setItemsPageTransitionState,
      setItemsTargetPageIndex,
      setScanSnapshot,
      setScanStats,
      setSelectedAssetId,
      setSelectedLibraryDetail,
      setWorkspaceError,
      setWorkspaceHydrated,
      setWorkspaceRefreshing,
      thumbnailPageSize,
      workspaceHydrated,
    ],
  )

  const refreshWorkspace = useCallback(
    async (options?: RefreshWorkspaceOptions) => {
      const nextSelectedLibraryId = await refreshLibraries(options?.preferredLibraryId)
      if (nextSelectedLibraryId === null) {
        clearWorkspaceData()
        return
      }

      const nextSelection = await refreshSidebarNodes(
        nextSelectedLibraryId,
        options?.preferredSidebarNodeId,
        options?.preferredMediaSourceId,
      )
      const nextPageIndex = Math.max(1, options?.preferredPageIndex ?? 1)
      setItemsPageIndex(nextPageIndex)
      setItemsTargetPageIndex(nextPageIndex)
      await loadLibrarySurface({
        libraryId: nextSelectedLibraryId,
        mediaSourceId: nextSelection.selectedMediaSourceId,
        requestedPageIndex: nextPageIndex,
        preferredAssetId: options?.preferredAssetId,
        includeWorkspaceSummary: true,
      })
    },
    [
      clearWorkspaceData,
      loadLibrarySurface,
      refreshLibraries,
      refreshSidebarNodes,
      setItemsPageIndex,
      setItemsTargetPageIndex,
    ],
  )

  const previousPageSizeRef = useRef(thumbnailPageSize)
  useEffect(() => {
    const previousPageSize = previousPageSizeRef.current
    previousPageSizeRef.current = thumbnailPageSize

    if (selectedLibraryId === null || previousPageSize === thumbnailPageSize) {
      return
    }

    void loadLibrarySurface({
      libraryId: selectedLibraryId,
      mediaSourceId: selectedMediaSourceId,
      requestedPageIndex: itemsPageIndex,
      includeWorkspaceSummary: false,
    })
  }, [itemsPageIndex, loadLibrarySurface, selectedLibraryId, selectedMediaSourceId, thumbnailPageSize])

  return {
    bootstrapScanSnapshot,
    loadLibrarySurface,
    refreshSidebarNodes,
    refreshWorkspace,
  }
}
