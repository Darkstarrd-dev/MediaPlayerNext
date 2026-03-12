import { readText as readClipboardText } from '@tauri-apps/plugin-clipboard-manager'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { hasFiles as clipboardHasFiles, readFiles as readClipboardFiles } from 'tauri-plugin-clipboard-x-api'
import { useEffect, useRef } from 'react'
import { getErrorMessage, isEditablePasteTarget, normalizePathBatch, parseClipboardPaths } from './app-shell-utils'

interface UseAppShellImportListenersParams {
  handleDropImport: (rawPaths: string[]) => Promise<void>
  handlePasteImport: (rawText: string) => Promise<void>
  setDropImportActive: (value: boolean) => void
  setImportTaskPanelOpen: (value: boolean | ((open: boolean) => boolean) ) => void
  setActionError: (value: string | null) => void
}

export function useAppShellImportListeners(params: UseAppShellImportListenersParams) {
  const { handleDropImport, handlePasteImport, setDropImportActive, setImportTaskPanelOpen, setActionError } = params
  const handleDropImportRef = useRef<(paths: string[]) => Promise<void>>(async () => undefined)
  const handlePasteImportRef = useRef<(text: string) => Promise<void>>(async () => undefined)

  useEffect(() => {
    handleDropImportRef.current = handleDropImport
  }, [handleDropImport])

  useEffect(() => {
    handlePasteImportRef.current = handlePasteImport
  }, [handlePasteImport])

  useEffect(() => {
    let cancelled = false
    let cleanup: (() => void) | null = null

    void getCurrentWindow()
      .onDragDropEvent((event) => {
        if (cancelled) {
          return
        }

        if (event.payload.type === 'enter' || event.payload.type === 'over') {
          setDropImportActive(true)
          return
        }

        if (event.payload.type === 'leave') {
          setDropImportActive(false)
          return
        }

        setDropImportActive(false)
        setImportTaskPanelOpen(true)
        void handleDropImportRef.current(event.payload.paths)
      })
      .then((unlisten) => {
        if (cancelled) {
          void unlisten()
          return
        }

        cleanup = unlisten
      })
      .catch((error: unknown) => {
        setActionError(`拖拽监听初始化失败：${getErrorMessage(error)}`)
      })

    return () => {
      cancelled = true
      cleanup?.()
    }
  }, [setActionError, setDropImportActive, setImportTaskPanelOpen])

  useEffect(() => {
    const handlePaste = (event: ClipboardEvent): void => {
      if (isEditablePasteTarget(event.target)) {
        return
      }

      const clipboardText =
        event.clipboardData?.getData('text/plain') || event.clipboardData?.getData('text/uri-list') || ''

      const processPaste = async (): Promise<void> => {
        const nativeFilePaths = await clipboardHasFiles()
          .then(async (hasFiles) => {
            if (!hasFiles) {
              return []
            }

            const result = await readClipboardFiles()
            return normalizePathBatch(result.paths)
          })
          .catch(() => [])

        if (nativeFilePaths.length > 0) {
          event.preventDefault()
          setImportTaskPanelOpen(true)
          await handlePasteImportRef.current(nativeFilePaths.join('\n'))
          return
        }

        const fallbackText = clipboardText.length > 0 ? clipboardText : await readClipboardText().catch(() => '')
        const parsedPaths = parseClipboardPaths(fallbackText)
        if (parsedPaths.length === 0) {
          return
        }

        event.preventDefault()
        setImportTaskPanelOpen(true)
        await handlePasteImportRef.current(fallbackText)
      }

      void processPaste()
    }

    window.addEventListener('paste', handlePaste)
    return () => {
      window.removeEventListener('paste', handlePaste)
    }
  }, [setImportTaskPanelOpen])
}
