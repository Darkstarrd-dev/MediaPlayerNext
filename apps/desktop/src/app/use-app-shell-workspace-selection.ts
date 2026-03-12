import type { LibrarySummary, SidebarNodeSummary } from '@mediaplayernext/contracts'
import { useCallback } from 'react'
import type { MediaRepository } from '../repositories/media-repository'
import { getErrorMessage, resolveLibrarySelection } from './app-shell-utils'
import { normalizeSidebarImageNodes } from './sidebar-main-image-tree'

export interface SidebarSelection {
  selectedSidebarNodeId: string | null
  selectedMediaSourceId: string | null
}

interface UseAppShellWorkspaceSelectionParams {
  repository: MediaRepository
  clearWorkspaceData: () => void
  setLibraries: (value: LibrarySummary[]) => void
  setLibrariesLoading: (value: boolean) => void
  setSelectedLibraryId: (value: string | null) => void
  setSidebarNodes: (value: SidebarNodeSummary[]) => void
  setSidebarNodesLoading: (value: boolean) => void
  setSelectedSidebarNodeId: (value: string | null) => void
  setSelectedMediaSourceId: (value: string | null) => void
  setWorkspaceError: (value: string | null) => void
}

function isMediaSourceNode(node: SidebarNodeSummary): boolean {
  return node.nodeType === 'media_source' && typeof node.mediaSourceId === 'string'
}

function resolveSidebarSelection(
  nodes: SidebarNodeSummary[],
  preferredSidebarNodeId?: string | null,
  preferredMediaSourceId?: string | null,
): SidebarSelection {
  if (preferredSidebarNodeId !== undefined && preferredSidebarNodeId !== null) {
    const matchedNode = nodes.find((node) => node.nodeId === preferredSidebarNodeId)
    if (matchedNode) {
      return {
        selectedSidebarNodeId: matchedNode.nodeId,
        selectedMediaSourceId: isMediaSourceNode(matchedNode) ? matchedNode.mediaSourceId ?? null : null,
      }
    }
  }

  if (preferredMediaSourceId !== undefined && preferredMediaSourceId !== null) {
    const matchedNode = nodes.find((node) => node.mediaSourceId === preferredMediaSourceId)
    if (matchedNode) {
      return {
        selectedSidebarNodeId: matchedNode.nodeId,
        selectedMediaSourceId: matchedNode.mediaSourceId ?? null,
      }
    }
  }

  const firstMediaNode = nodes.find((node) => isMediaSourceNode(node))
  if (firstMediaNode) {
    return {
      selectedSidebarNodeId: firstMediaNode.nodeId,
      selectedMediaSourceId: firstMediaNode.mediaSourceId ?? null,
    }
  }

  return {
    selectedSidebarNodeId: nodes[0]?.nodeId ?? null,
    selectedMediaSourceId: null,
  }
}

export function useAppShellWorkspaceSelection(params: UseAppShellWorkspaceSelectionParams) {
  const {
    repository,
    clearWorkspaceData,
    setLibraries,
    setLibrariesLoading,
    setSelectedLibraryId,
    setSidebarNodes,
    setSidebarNodesLoading,
    setSelectedSidebarNodeId,
    setSelectedMediaSourceId,
    setWorkspaceError,
  } = params

  const refreshLibraries = useCallback(
    async (preferredLibraryId?: string | null): Promise<string | null> => {
      setLibrariesLoading(true)
      try {
        const nextLibraries = await repository.library.list()
        const nextSelectedLibraryId = resolveLibrarySelection(nextLibraries, preferredLibraryId)
        setLibraries(nextLibraries)
        setSelectedLibraryId(nextSelectedLibraryId)
        return nextSelectedLibraryId
      } catch (error) {
        setLibraries([])
        setSelectedLibraryId(null)
        setWorkspaceError(getErrorMessage(error))
        clearWorkspaceData()
        return null
      } finally {
        setLibrariesLoading(false)
      }
    },
    [clearWorkspaceData, repository, setLibraries, setLibrariesLoading, setSelectedLibraryId, setWorkspaceError],
  )

  const refreshSidebarNodes = useCallback(
    async (
      libraryId: string,
      preferredSidebarNodeId?: string | null,
      preferredMediaSourceId?: string | null,
    ): Promise<SidebarSelection> => {
      setSidebarNodesLoading(true)
      setSidebarNodes([])
      setSelectedSidebarNodeId(null)
      setSelectedMediaSourceId(null)

      try {
        const rawNodes = await repository.library.nodes(libraryId)
        const nextNodes = normalizeSidebarImageNodes(rawNodes)
        const nextSelection = resolveSidebarSelection(
          nextNodes,
          preferredSidebarNodeId,
          preferredMediaSourceId,
        )
        setSidebarNodes(nextNodes)
        setSelectedSidebarNodeId(nextSelection.selectedSidebarNodeId)
        setSelectedMediaSourceId(nextSelection.selectedMediaSourceId)
        return nextSelection
      } catch (error) {
        setSidebarNodes([])
        setSelectedSidebarNodeId(null)
        setSelectedMediaSourceId(null)
        setWorkspaceError(getErrorMessage(error))
        return { selectedSidebarNodeId: null, selectedMediaSourceId: null }
      } finally {
        setSidebarNodesLoading(false)
      }
    },
    [
      repository,
      setSelectedMediaSourceId,
      setSelectedSidebarNodeId,
      setSidebarNodes,
      setSidebarNodesLoading,
      setWorkspaceError,
    ],
  )

  return {
    refreshLibraries,
    refreshSidebarNodes,
  }
}
