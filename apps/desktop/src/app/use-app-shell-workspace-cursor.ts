import type { WorkspaceCursor } from '@mediaplayernext/contracts'
import { useEffect } from 'react'

interface WorkspaceCursorStore {
  readWorkspaceCursor(): Promise<WorkspaceCursor | null>
  writeWorkspaceCursor(cursor: WorkspaceCursor): Promise<void>
}

interface RefreshWorkspaceOptions {
  preferredLibraryId?: string | null
  preferredSidebarNodeId?: string | null
  preferredMediaSourceId?: string | null
  preferredPageIndex?: number
  preferredAssetId?: string | null
}

interface UseAppShellWorkspaceCursorParams {
  cursorStore: WorkspaceCursorStore
  refreshWorkspace: (options?: RefreshWorkspaceOptions) => Promise<void>
  workspaceHydrated: boolean
  selectedLibraryId: string | null
  selectedSidebarNodeId: string | null
  selectedMediaSourceId: string | null
  itemsPageIndex: number
  selectedAssetId: string | null
}

export function useAppShellWorkspaceCursor(params: UseAppShellWorkspaceCursorParams) {
  const {
    cursorStore,
    refreshWorkspace,
    workspaceHydrated,
    selectedLibraryId,
    selectedSidebarNodeId,
    selectedMediaSourceId,
    itemsPageIndex,
    selectedAssetId,
  } = params

  useEffect(() => {
    let cancelled = false

    const hydrateWorkspace = async (): Promise<void> => {
      try {
        const cursor = await cursorStore.readWorkspaceCursor()
        if (cancelled) {
          return
        }

        const normalizedCursor: WorkspaceCursor = {
          selectedLibraryId: cursor?.selectedLibraryId ?? null,
          selectedSidebarNodeId: cursor?.selectedSidebarNodeId ?? cursor?.selectedNodeId ?? null,
          selectedMediaSourceId: cursor?.selectedMediaSourceId ?? null,
          selectedNodeId: cursor?.selectedNodeId ?? null,
          itemsPageIndex: Math.max(1, cursor?.itemsPageIndex ?? 1),
          selectedAssetId: cursor?.selectedAssetId ?? null,
        }

        await refreshWorkspace({
          preferredLibraryId: normalizedCursor.selectedLibraryId,
          preferredSidebarNodeId: normalizedCursor.selectedSidebarNodeId,
          preferredMediaSourceId: normalizedCursor.selectedMediaSourceId,
          preferredPageIndex: normalizedCursor.itemsPageIndex ?? 1,
          preferredAssetId: normalizedCursor.selectedAssetId,
        })
      } catch {
        if (cancelled) {
          return
        }

        await refreshWorkspace()
      }
    }

    void hydrateWorkspace()

    return () => {
      cancelled = true
    }
  }, [cursorStore, refreshWorkspace])

  useEffect(() => {
    if (!workspaceHydrated) {
      return
    }

    const timer = window.setTimeout(() => {
      void cursorStore
        .writeWorkspaceCursor({
          selectedLibraryId,
          selectedSidebarNodeId,
          selectedMediaSourceId,
          selectedNodeId: selectedSidebarNodeId,
          itemsPageIndex,
          selectedAssetId,
        })
        .catch(() => {
          // ignore workspace cursor persistence failure
        })
    }, 180)

    return () => {
      window.clearTimeout(timer)
    }
  }, [
    cursorStore,
    itemsPageIndex,
    selectedAssetId,
    selectedLibraryId,
    selectedMediaSourceId,
    selectedSidebarNodeId,
    workspaceHydrated,
  ])
}
