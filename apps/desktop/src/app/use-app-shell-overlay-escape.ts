import { useEffect } from 'react'

interface UseAppShellOverlayEscapeParams {
  settingsOpen: boolean
  importTaskPanelOpen: boolean
  setSettingsOpen: (value: boolean | ((current: boolean) => boolean)) => void
  setImportTaskPanelOpen: (value: boolean | ((current: boolean) => boolean)) => void
}

export function useAppShellOverlayEscape(params: UseAppShellOverlayEscapeParams) {
  const { settingsOpen, importTaskPanelOpen, setSettingsOpen, setImportTaskPanelOpen } = params

  useEffect(() => {
    if (!settingsOpen && !importTaskPanelOpen) {
      return
    }

    const handleEscape = (event: KeyboardEvent): void => {
      if (event.key !== 'Escape') {
        return
      }

      if (settingsOpen) {
        setSettingsOpen(false)
        return
      }

      setImportTaskPanelOpen(false)
    }

    window.addEventListener('keydown', handleEscape)
    return () => window.removeEventListener('keydown', handleEscape)
  }, [importTaskPanelOpen, setImportTaskPanelOpen, setSettingsOpen, settingsOpen])
}
