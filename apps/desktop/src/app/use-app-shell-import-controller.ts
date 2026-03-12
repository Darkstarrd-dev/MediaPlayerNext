import { useCallback, useState } from 'react'
import type { MediaRepository } from '../repositories/media-repository'
import { runPathImportBatch } from './app-shell-import-batch'
import { getErrorMessage, parseClipboardPaths } from './app-shell-utils'
import { useAppShellDirectoryPicker } from './use-app-shell-directory-picker'
import type { ImportActivity } from './use-app-shell-import-activities'
import { useAppShellImportListeners } from './use-app-shell-import-listeners'

const ACTION_LABELS = {
  addLibrary: '登记媒体库',
  addAndScan: '登记并扫描',
  dropImport: '拖拽导入',
  pasteImport: '粘贴导入',
} as const

export type ImportActionKind = keyof typeof ACTION_LABELS

interface RefreshWorkspaceOptions {
  preferredLibraryId?: string | null
  preferredPageIndex?: number
}

interface UseAppShellImportControllerParams {
  repository: MediaRepository
  selectedLibraryId: string | null
  refreshWorkspace: (options?: RefreshWorkspaceOptions) => Promise<void>
  bootstrapScanSnapshot: (libraryId: string) => Promise<void>
  appendImportActivity: (activity: Omit<ImportActivity, 'id' | 'createdAt'>) => string
  updateImportActivity: (
    activityId: string,
    patch: Partial<Omit<ImportActivity, 'id' | 'createdAt'>>,
  ) => void
  setImportTaskPanelOpen: (value: boolean | ((open: boolean) => boolean)) => void
}

export function useAppShellImportController(params: UseAppShellImportControllerParams) {
  const {
    repository,
    selectedLibraryId,
    refreshWorkspace,
    bootstrapScanSnapshot,
    appendImportActivity,
    updateImportActivity,
    setImportTaskPanelOpen,
  } = params

  const { pickSingleDirectory } = useAppShellDirectoryPicker()
  const [importRootPath, setImportRootPath] = useState('')
  const [dropImportActive, setDropImportActive] = useState(false)
  const [actionBusy, setActionBusy] = useState<ImportActionKind | null>(null)
  const [actionMessage, setActionMessage] = useState<string | null>(null)
  const [actionError, setActionError] = useState<string | null>(null)

  const runPathImport = useCallback(
    async (
      rawPaths: string[],
      actionKind: Extract<ImportActionKind, 'dropImport' | 'pasteImport'>,
      emptyErrorMessage: string,
      successSummaryLabel: string,
      failureSummaryLabel: string,
    ) => {
      await runPathImportBatch({
        rawPaths,
        actionKind,
        emptyErrorMessage,
        successSummaryLabel,
        failureSummaryLabel,
        repository,
        selectedLibraryId,
        refreshWorkspace,
        bootstrapScanSnapshot,
        appendImportActivity,
        updateImportActivity,
        setActionBusy,
        setActionError,
        setActionMessage,
      })
    },
    [
      appendImportActivity,
      bootstrapScanSnapshot,
      refreshWorkspace,
      repository,
      selectedLibraryId,
      setActionBusy,
      setActionError,
      setActionMessage,
      updateImportActivity,
    ],
  )

  const handleAddLibrary = useCallback(
    async (shouldScanAfterAdd: boolean) => {
      const rootPath = importRootPath.trim()

      if (rootPath.length === 0) {
        setActionError('请先输入要登记的本地路径。')
        return
      }

      const activityId = appendImportActivity({
        title: shouldScanAfterAdd ? '登记并扫描' : '登记媒体库',
        source: '手动路径',
        status: 'running',
        detail: shouldScanAfterAdd ? `正在登记并扫描：${rootPath}` : `正在登记媒体库：${rootPath}`,
      })

      setActionBusy(shouldScanAfterAdd ? 'addAndScan' : 'addLibrary')
      setActionError(null)

      try {
        const createdLibrary = await repository.library.add({ rootPath })

        if (shouldScanAfterAdd) {
          setActionMessage(`已登记并启动扫描：${createdLibrary.rootPath}`)
          updateImportActivity(activityId, {
            status: 'completed',
            detail: `已登记并启动扫描：${createdLibrary.rootPath}`,
          })
          void repository.scan
            .start(createdLibrary.id)
            .then(async () => {
              await bootstrapScanSnapshot(createdLibrary.id)
            })
            .catch((error: unknown) => {
              setActionError(`扫描启动失败：${getErrorMessage(error)}`)
            })
        } else {
          setActionMessage(`已登记媒体库：${createdLibrary.rootPath}`)
          updateImportActivity(activityId, {
            status: 'completed',
            detail: `已登记媒体库：${createdLibrary.rootPath}`,
          })
        }

        setImportRootPath('')
        await refreshWorkspace({
          preferredLibraryId: createdLibrary.id,
          preferredPageIndex: 1,
        })
      } catch (error) {
        setActionError(getErrorMessage(error))
        updateImportActivity(activityId, {
          status: 'failed',
          detail: `登记失败：${getErrorMessage(error)}`,
        })
      } finally {
        setActionBusy(null)
      }
    },
    [
      appendImportActivity,
      bootstrapScanSnapshot,
      importRootPath,
      refreshWorkspace,
      repository,
      updateImportActivity,
    ],
  )

  const handleDropImport = useCallback(
    async (rawPaths: string[]) => {
      await runPathImport(rawPaths, 'dropImport', '未从拖拽事件中解析到可用路径。', '拖拽路径', '拖拽导入失败。')
    },
    [runPathImport],
  )

  const handlePasteImport = useCallback(
    async (rawText: string) => {
      await runPathImport(
        parseClipboardPaths(rawText),
        'pasteImport',
        '未从剪贴板解析到可用路径。',
        '粘贴路径',
        '粘贴导入失败。',
      )
    },
    [runPathImport],
  )

  useAppShellImportListeners({
    handleDropImport,
    handlePasteImport,
    setDropImportActive,
    setImportTaskPanelOpen,
    setActionError,
  })

  const handlePickDirectory = useCallback(async () => {
    setActionError(null)

    try {
      const nextPath = await pickSingleDirectory('选择媒体库目录')
      if (nextPath === null) {
        return
      }

      setImportRootPath(nextPath)
    } catch (error) {
      setActionError(`系统文件夹选择器不可用：${getErrorMessage(error)}`)
    }
  }, [pickSingleDirectory])

  return {
    actionBusy,
    actionError,
    actionMessage,
    dropImportActive,
    handleAddLibrary,
    handleDropImport,
    handlePasteImport,
    handlePickDirectory,
    importRootPath,
    pickSingleDirectory,
    setActionError,
    setActionMessage,
    setImportRootPath,
  }
}
