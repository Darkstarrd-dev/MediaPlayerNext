import { open as openDialog } from '@tauri-apps/plugin-dialog'
import { useCallback } from 'react'
import { consumeE2eDirectorySelection } from './e2e-test-bridge'

export function useAppShellDirectoryPicker() {
  const pickSingleDirectory = useCallback(async (title: string): Promise<string | null> => {
    const e2eSelection = consumeE2eDirectorySelection(title)
    if (e2eSelection.handled) {
      return e2eSelection.path
    }

    const selection = await openDialog({
      directory: true,
      multiple: false,
      title,
    })

    if (selection === null) {
      return null
    }

    const nextPath = Array.isArray(selection) ? selection[0] : selection
    return typeof nextPath === 'string' && nextPath.trim().length > 0 ? nextPath : null
  }, [])

  return {
    pickSingleDirectory,
  }
}
