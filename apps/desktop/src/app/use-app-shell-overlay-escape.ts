import { useEffect } from 'react'

interface UseAppShellOverlayEscapeParams {
  settingsOpen: boolean
  themeDebugOpen: boolean
  importTaskPanelOpen: boolean
  setSettingsOpen: (value: boolean | ((current: boolean) => boolean)) => void
  setThemeDebugOpen: (value: boolean | ((current: boolean) => boolean)) => void
  setImportTaskPanelOpen: (value: boolean | ((current: boolean) => boolean)) => void
}

export function useAppShellOverlayEscape(params: UseAppShellOverlayEscapeParams) {
  const {
    settingsOpen,
    themeDebugOpen,
    importTaskPanelOpen,
    setSettingsOpen,
    setThemeDebugOpen,
    setImportTaskPanelOpen,
  } = params

  useEffect(() => {
    if (!settingsOpen && !themeDebugOpen && !importTaskPanelOpen) {
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

      if (themeDebugOpen) {
        setThemeDebugOpen(false)
        return
      }

      setImportTaskPanelOpen(false)
    }

    window.addEventListener('keydown', handleEscape)
    return () => window.removeEventListener('keydown', handleEscape)
  }, [
    importTaskPanelOpen,
    setImportTaskPanelOpen,
    setSettingsOpen,
    setThemeDebugOpen,
    settingsOpen,
    themeDebugOpen,
  ])
}
