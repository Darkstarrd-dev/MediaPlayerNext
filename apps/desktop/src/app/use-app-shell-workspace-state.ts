import type {
  ItemDetail,
  ItemListEntry,
  LibraryDetail,
  LibrarySummary,
  ScanStats,
  SidebarNodeSummary,
  TaskProgress,
} from '@mediaplayernext/contracts'
import { useCallback, useRef, useState } from 'react'

export type ItemsPageTransitionState = 'idle' | 'loading-next-page' | 'committing'

export function useAppShellWorkspaceState() {
  const libraryLoadRequestIdRef = useRef(0)
  const itemDetailRequestIdRef = useRef(0)

  const [libraries, setLibraries] = useState<LibrarySummary[]>([])
  const [librariesLoading, setLibrariesLoading] = useState(false)
  const [selectedLibraryId, setSelectedLibraryId] = useState<string | null>(null)
  const [sidebarNodes, setSidebarNodes] = useState<SidebarNodeSummary[]>([])
  const [sidebarNodesLoading, setSidebarNodesLoading] = useState(false)
  const [selectedSidebarNodeId, setSelectedSidebarNodeId] = useState<string | null>(null)
  const [selectedMediaSourceId, setSelectedMediaSourceId] = useState<string | null>(null)
  const [selectedLibraryDetail, setSelectedLibraryDetail] = useState<LibraryDetail | null>(null)
  const [scanStats, setScanStats] = useState<ScanStats | null>(null)
  const [scanSnapshot, setScanSnapshot] = useState<TaskProgress | null>(null)
  const [workspaceRefreshing, setWorkspaceRefreshing] = useState(false)
  const [workspaceHydrated, setWorkspaceHydrated] = useState(false)
  const [workspaceError, setWorkspaceError] = useState<string | null>(null)
  const [items, setItems] = useState<ItemListEntry[]>([])
  const [itemsPageIndex, setItemsPageIndex] = useState(1)
  const [itemsTargetPageIndex, setItemsTargetPageIndex] = useState(1)
  const [itemsHasNextPage, setItemsHasNextPage] = useState(false)
  const [itemsPageTransitionState, setItemsPageTransitionState] =
    useState<ItemsPageTransitionState>('idle')
  const [selectedAssetId, setSelectedAssetId] = useState<string | null>(null)
  const [selectedItemDetail, setSelectedItemDetail] = useState<ItemDetail | null>(null)
  const [itemDetailLoading, setItemDetailLoading] = useState(false)
  const [itemDetailError, setItemDetailError] = useState<string | null>(null)
  const [itemThumbnailUrls, setItemThumbnailUrls] = useState<Record<string, string>>({})

  const clearWorkspaceData = useCallback(() => {
    libraryLoadRequestIdRef.current += 1
    itemDetailRequestIdRef.current += 1
    setSelectedLibraryDetail(null)
    setSidebarNodes([])
    setSelectedSidebarNodeId(null)
    setSelectedMediaSourceId(null)
    setSidebarNodesLoading(false)
    setScanStats(null)
    setScanSnapshot(null)
    setWorkspaceError(null)
    setWorkspaceRefreshing(false)
    setWorkspaceHydrated(false)
    setItems([])
    setItemsPageIndex(1)
    setItemsTargetPageIndex(1)
    setItemsHasNextPage(false)
    setItemsPageTransitionState('idle')
    setSelectedAssetId(null)
    setSelectedItemDetail(null)
    setItemDetailError(null)
    setItemDetailLoading(false)
    setItemThumbnailUrls({})
  }, [])

  return {
    libraryLoadRequestIdRef,
    itemDetailRequestIdRef,
    libraries,
    setLibraries,
    librariesLoading,
    setLibrariesLoading,
    selectedLibraryId,
    setSelectedLibraryId,
    sidebarNodes,
    setSidebarNodes,
    sidebarNodesLoading,
    setSidebarNodesLoading,
    selectedSidebarNodeId,
    setSelectedSidebarNodeId,
    selectedMediaSourceId,
    setSelectedMediaSourceId,
    selectedLibraryDetail,
    setSelectedLibraryDetail,
    scanStats,
    setScanStats,
    scanSnapshot,
    setScanSnapshot,
    workspaceRefreshing,
    setWorkspaceRefreshing,
    workspaceHydrated,
    setWorkspaceHydrated,
    workspaceError,
    setWorkspaceError,
    items,
    setItems,
    itemsPageIndex,
    setItemsPageIndex,
    itemsTargetPageIndex,
    setItemsTargetPageIndex,
    itemsHasNextPage,
    setItemsHasNextPage,
    itemsPageTransitionState,
    setItemsPageTransitionState,
    selectedAssetId,
    setSelectedAssetId,
    selectedItemDetail,
    setSelectedItemDetail,
    itemDetailLoading,
    setItemDetailLoading,
    itemDetailError,
    setItemDetailError,
    itemThumbnailUrls,
    setItemThumbnailUrls,
    clearWorkspaceData,
  }
}
