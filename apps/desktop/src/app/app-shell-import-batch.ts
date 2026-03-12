import type { LibraryDetail } from '@mediaplayernext/contracts'
import type { MediaRepository } from '../repositories/media-repository'
import { getErrorMessage, normalizePathBatch } from './app-shell-utils'
import type { ImportActivity } from './use-app-shell-import-activities'
import type { ImportActionKind } from './use-app-shell-import-controller'

const IMPORT_ACTION_LABELS: Record<Extract<ImportActionKind, 'dropImport' | 'pasteImport'>, string> = {
  dropImport: '拖拽导入',
  pasteImport: '粘贴导入',
}

interface RefreshWorkspaceOptions {
  preferredLibraryId?: string | null
  preferredPageIndex?: number
}

interface RunPathImportBatchParams {
  rawPaths: string[]
  actionKind: Extract<ImportActionKind, 'dropImport' | 'pasteImport'>
  emptyErrorMessage: string
  successSummaryLabel: string
  failureSummaryLabel: string
  repository: MediaRepository
  selectedLibraryId: string | null
  refreshWorkspace: (options?: RefreshWorkspaceOptions) => Promise<void>
  bootstrapScanSnapshot: (libraryId: string) => Promise<void>
  appendImportActivity: (activity: Omit<ImportActivity, 'id' | 'createdAt'>) => string
  updateImportActivity: (
    activityId: string,
    patch: Partial<Omit<ImportActivity, 'id' | 'createdAt'>>,
  ) => void
  setActionBusy: (value: ImportActionKind | null) => void
  setActionError: (value: string | null) => void
  setActionMessage: (value: string | null) => void
}

export async function runPathImportBatch(params: RunPathImportBatchParams) {
  const {
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
  } = params

  const paths = normalizePathBatch(rawPaths)
  if (paths.length === 0) {
    setActionError(emptyErrorMessage)
    return
  }

  const activityId = appendImportActivity({
    title: IMPORT_ACTION_LABELS[actionKind],
    source: actionKind === 'dropImport' ? '拖拽导入' : '粘贴导入',
    status: 'running',
    detail: `正在处理 ${paths.length} 条路径。`,
  })

  setActionBusy(actionKind)
  setActionError(null)
  setActionMessage(null)

  const successLibraries: LibraryDetail[] = []
  const failedPaths: string[] = []

  for (const path of paths) {
    try {
      const createdLibrary = await repository.library.add({ rootPath: path })
      successLibraries.push(createdLibrary)
      void repository.scan
        .start(createdLibrary.id)
        .then(async () => {
          await bootstrapScanSnapshot(createdLibrary.id)
        })
        .catch((error: unknown) => {
          setActionError(`扫描启动失败：${getErrorMessage(error)}`)
        })
    } catch (error) {
      failedPaths.push(`${path}：${getErrorMessage(error)}`)
    }
  }

  try {
    const preferredLibraryId = successLibraries.at(-1)?.id ?? selectedLibraryId
    await refreshWorkspace({ preferredLibraryId, preferredPageIndex: 1 })
  } finally {
    setActionBusy(null)
  }

  if (successLibraries.length > 0) {
    const successSummary = `已处理 ${successLibraries.length} 条${successSummaryLabel}，并刷新主界面快照。`

    if (failedPaths.length > 0) {
      setActionMessage(`${successSummary} 部分路径失败。`)
      setActionError(failedPaths.join('；'))
      updateImportActivity(activityId, {
        status: 'completed',
        detail: `${successSummary} 部分路径失败。`,
      })
    } else {
      setActionMessage(successSummary)
      setActionError(null)
      updateImportActivity(activityId, {
        status: 'completed',
        detail: successSummary,
      })
    }

    return
  }

  const failureDetail = failedPaths.join('；') || failureSummaryLabel
  setActionMessage(null)
  setActionError(failureDetail)
  updateImportActivity(activityId, {
    status: 'failed',
    detail: failureDetail,
  })
}
