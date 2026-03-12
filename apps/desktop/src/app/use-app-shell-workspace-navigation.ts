import type { SidebarNodeSummary } from '@mediaplayernext/contracts'
import { useCallback } from 'react'

interface UseAppShellWorkspaceNavigationParams {
  selectedLibraryId: string | null
  sidebarNodes: SidebarNodeSummary[]
  refreshWorkspace: (options?: { preferredLibraryId?: string | null; preferredPageIndex?: number }) => Promise<void>
  loadLibrarySurface: (options: {
    libraryId: string
    mediaSourceId: string | null
    requestedPageIndex: number
    preferredAssetId?: string | null
    includeWorkspaceSummary?: boolean
  }) => Promise<void>
  setSelectedSidebarNodeId: (value: string | null) => void
  setSelectedMediaSourceId: (value: string | null) => void
}

export function useAppShellWorkspaceNavigation(params: UseAppShellWorkspaceNavigationParams) {
  const {
    selectedLibraryId,
    sidebarNodes,
    refreshWorkspace,
    loadLibrarySurface,
    setSelectedSidebarNodeId,
    setSelectedMediaSourceId,
  } = params

  const handleLibrarySelect = useCallback(
    (libraryId: string) => {
      void refreshWorkspace({ preferredLibraryId: libraryId, preferredPageIndex: 1 })
    },
    [refreshWorkspace],
  )

  const handleSidebarNodeSelect = useCallback(
    (nodeId: string) => {
      if (selectedLibraryId === null) {
        return
      }

      const matchedNode = sidebarNodes.find((node) => node.nodeId === nodeId) ?? null
      if (matchedNode === null) {
        return
      }

      if (matchedNode.nodeType !== 'media_source') {
        setSelectedSidebarNodeId(nodeId)
        return
      }

      const nextMediaSourceId = matchedNode.mediaSourceId ?? null
      setSelectedSidebarNodeId(nodeId)
      setSelectedMediaSourceId(nextMediaSourceId)
      void loadLibrarySurface({
        libraryId: selectedLibraryId,
        mediaSourceId: nextMediaSourceId,
        requestedPageIndex: 1,
        includeWorkspaceSummary: false,
      })
    },
    [loadLibrarySurface, selectedLibraryId, setSelectedMediaSourceId, setSelectedSidebarNodeId, sidebarNodes],
  )

  return {
    handleLibrarySelect,
    handleSidebarNodeSelect,
  }
}
