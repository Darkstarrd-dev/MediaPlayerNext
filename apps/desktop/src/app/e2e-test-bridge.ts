interface E2eDirectorySelection {
  title: string
  path: string | null
}

interface MpnextE2eBridgeState {
  enabled?: boolean
  directorySelections?: E2eDirectorySelection[]
}

declare global {
  interface Window {
    __MPNEXT_E2E__?: MpnextE2eBridgeState
  }
}

export function consumeE2eDirectorySelection(title: string): { handled: boolean; path: string | null } {
  if (typeof window === 'undefined') {
    return { handled: false, path: null }
  }

  const bridge = window.__MPNEXT_E2E__
  if (bridge?.enabled !== true) {
    return { handled: false, path: null }
  }

  const selections = bridge.directorySelections ?? []
  const selectionIndex = selections.findIndex((entry) => entry.title === title || entry.title === '*')
  if (selectionIndex < 0) {
    return { handled: false, path: null }
  }

  const [selection] = selections.splice(selectionIndex, 1)
  return {
    handled: true,
    path: selection?.path ?? null,
  }
}

export {}
