import type { RuntimeInfo } from '@mediaplayernext/contracts'
import { useCallback, useEffect, useMemo, useState } from 'react'
import type { MediaRepository } from '../repositories/media-repository'
import { getErrorMessage } from './app-shell-utils'
import type { AppShellDatabaseActionKind, AppShellSettingsPage } from './app-shell-settings-types'

const DATABASE_ACTION_LABELS = {
  pickDatabaseDir: '选择 SQL 目录',
  pickThumbnailDir: '选择缩略图目录',
  clearDatabase: '清除数据库',
} as const

interface UseAppShellDatabaseSettingsParams {
  repository: MediaRepository
  settingsOpen: boolean
  settingsPage: AppShellSettingsPage
  pickSingleDirectory: (title: string) => Promise<string | null>
}

export function useAppShellDatabaseSettings(params: UseAppShellDatabaseSettingsParams) {
  const { repository, settingsOpen, settingsPage, pickSingleDirectory } = params

  const [clearDatabaseDialogOpen, setClearDatabaseDialogOpen] = useState(false)
  const [runtimeInfo, setRuntimeInfo] = useState<RuntimeInfo | null>(null)
  const [runtimeInfoLoading, setRuntimeInfoLoading] = useState(false)
  const [runtimeInfoError, setRuntimeInfoError] = useState<string | null>(null)
  const [databaseActionBusy, setDatabaseActionBusy] = useState<AppShellDatabaseActionKind | null>(null)
  const [databaseActionMessage, setDatabaseActionMessage] = useState<string | null>(null)
  const [databaseActionError, setDatabaseActionError] = useState<string | null>(null)

  const refreshRuntimeInfo = useCallback(async () => {
    setRuntimeInfoLoading(true)
    setRuntimeInfoError(null)

    try {
      const nextRuntimeInfo = await repository.database.readRuntimeInfo()
      setRuntimeInfo(nextRuntimeInfo)
    } catch (error) {
      setRuntimeInfo(null)
      setRuntimeInfoError(getErrorMessage(error))
    } finally {
      setRuntimeInfoLoading(false)
    }
  }, [repository])

  useEffect(() => {
    if (!settingsOpen || settingsPage !== 'database') {
      return
    }

    void refreshRuntimeInfo()
  }, [refreshRuntimeInfo, settingsOpen, settingsPage])

  useEffect(() => {
    if (settingsOpen) {
      return
    }

    setClearDatabaseDialogOpen(false)
  }, [settingsOpen])

  useEffect(() => {
    if (!clearDatabaseDialogOpen) {
      return
    }

    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key !== 'Escape' || databaseActionBusy === 'clearDatabase') {
        return
      }

      setClearDatabaseDialogOpen(false)
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [clearDatabaseDialogOpen, databaseActionBusy])

  const handlePickDatabaseDirectory = useCallback(async () => {
    setRuntimeInfoError(null)
    setDatabaseActionError(null)

    try {
      const nextPath = await pickSingleDirectory('选择 SQL 目录')
      if (nextPath === null) {
        return
      }

      setDatabaseActionBusy('pickDatabaseDir')
      setDatabaseActionMessage(null)
      const nextRuntimeInfo = await repository.database.setStoragePaths({ databaseDir: nextPath })
      setRuntimeInfo(nextRuntimeInfo)
      setDatabaseActionMessage('SQL 目录已保存。')
    } catch (error) {
      setDatabaseActionError(getErrorMessage(error))
    } finally {
      setDatabaseActionBusy(null)
    }
  }, [pickSingleDirectory, repository])

  const handlePickThumbnailDirectory = useCallback(async () => {
    setRuntimeInfoError(null)
    setDatabaseActionError(null)

    try {
      const nextPath = await pickSingleDirectory('选择缩略图目录')
      if (nextPath === null) {
        return
      }

      setDatabaseActionBusy('pickThumbnailDir')
      setDatabaseActionMessage(null)
      const nextRuntimeInfo = await repository.database.setStoragePaths({
        thumbnailCacheDir: nextPath,
      })
      setRuntimeInfo(nextRuntimeInfo)
      setDatabaseActionMessage('缩略图目录已保存。')
    } catch (error) {
      setDatabaseActionError(getErrorMessage(error))
    } finally {
      setDatabaseActionBusy(null)
    }
  }, [pickSingleDirectory, repository])

  const handleRequestClearDatabase = useCallback(() => {
    if (databaseActionBusy !== null) {
      return
    }

    setDatabaseActionError(null)
    setDatabaseActionMessage(null)
    setClearDatabaseDialogOpen(true)
  }, [databaseActionBusy])

  const handleCloseClearDatabaseDialog = useCallback(() => {
    if (databaseActionBusy === 'clearDatabase') {
      return
    }

    setClearDatabaseDialogOpen(false)
  }, [databaseActionBusy])

  const handleConfirmClearDatabase = useCallback(async () => {
    setRuntimeInfoError(null)
    setDatabaseActionBusy('clearDatabase')
    setDatabaseActionError(null)
    setDatabaseActionMessage(null)

    try {
      await repository.database.clear()
      setDatabaseActionMessage('已清除数据库，正在重新加载。')
      setClearDatabaseDialogOpen(false)
      window.location.reload()
    } catch (error) {
      setDatabaseActionError(getErrorMessage(error))
    } finally {
      setDatabaseActionBusy(null)
    }
  }, [repository])

  const databasePendingLabel = useMemo(
    () => (databaseActionBusy === null ? null : DATABASE_ACTION_LABELS[databaseActionBusy]),
    [databaseActionBusy],
  )

  const runtimeInfoDatabasePath = runtimeInfoLoading
    ? '正在读取当前 SQL 路径...'
    : runtimeInfo?.databasePath ?? '未读取'

  const runtimeInfoThumbnailCachePath = runtimeInfoLoading
    ? '正在读取当前缩略图目录...'
    : runtimeInfo?.thumbnailCachePath ?? '未读取'

  return {
    clearDatabaseDialogOpen,
    runtimeInfoError,
    databaseActionError,
    databaseActionMessage,
    databasePendingLabel,
    runtimeInfoDatabasePath,
    runtimeInfoThumbnailCachePath,
    databaseActionBusy,
    handleRequestClearDatabase,
    handleCloseClearDatabaseDialog,
    handlePickDatabaseDirectory,
    handlePickThumbnailDirectory,
    handleConfirmClearDatabase,
  }
}
