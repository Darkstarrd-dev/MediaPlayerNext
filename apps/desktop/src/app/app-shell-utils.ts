import type {
  ItemDetail,
  LibrarySummary,
  SidebarNodeSummary,
  TaskProgress,
} from '@mediaplayernext/contracts'

export function clampNumber(value: number, min: number, max: number): number {
  return Math.max(min, Math.min(max, value))
}

export function resolveSpacingPx(viewportWidth: number, scaleCoeff: number): number {
  return Math.max(0, Math.round(Math.max(0, viewportWidth) * 0.01 * scaleCoeff))
}

export function readSessionNumber(key: string, fallback: number, min: number, max: number): number {
  if (typeof window === 'undefined') {
    return fallback
  }

  const rawValue = window.sessionStorage.getItem(key)
  if (rawValue === null) {
    return fallback
  }

  const parsedValue = Number.parseFloat(rawValue)
  if (!Number.isFinite(parsedValue)) {
    return fallback
  }

  return clampNumber(parsedValue, min, max)
}

export function resolveWorkspaceWidths(
  viewportWidth: number,
  layoutPaddingPx: number,
  splitterWidthPx: number,
  preferredSidebarWidthPx: number,
  preferredMetaWidthPx: number,
) {
  const availableWidth = Math.max(0, viewportWidth - layoutPaddingPx * 2 - splitterWidthPx * 2)
  const minSidebarWidthPx = Math.min(220, Math.max(160, Math.round(availableWidth * 0.22)))
  const minMetaWidthPx = Math.min(280, Math.max(200, Math.round(availableWidth * 0.24)))
  const minMainWidthPx = Math.min(420, Math.max(280, Math.round(availableWidth * 0.34)))

  const maxSidebarWidthPx = Math.max(
    minSidebarWidthPx,
    availableWidth - minMetaWidthPx - minMainWidthPx,
  )
  const sidebarWidthPx = clampNumber(
    preferredSidebarWidthPx,
    minSidebarWidthPx,
    maxSidebarWidthPx,
  )

  const maxMetaWidthPx = Math.max(minMetaWidthPx, availableWidth - sidebarWidthPx - minMainWidthPx)
  const metaWidthPx = clampNumber(preferredMetaWidthPx, minMetaWidthPx, maxMetaWidthPx)

  const stabilizedSidebarWidthPx = clampNumber(
    sidebarWidthPx,
    minSidebarWidthPx,
    Math.max(minSidebarWidthPx, availableWidth - metaWidthPx - minMainWidthPx),
  )

  return {
    availableWidth,
    sidebarWidthPx: stabilizedSidebarWidthPx,
    metaWidthPx,
    mainWidthPx: Math.max(0, availableWidth - stabilizedSidebarWidthPx - metaWidthPx),
  }
}

export function getErrorMessage(error: unknown): string {
  if (error instanceof Error && error.message.trim().length > 0) {
    return error.message
  }

  return '发生了未识别错误'
}

export function isTaskNotFoundError(error: unknown): boolean {
  return getErrorMessage(error).includes('task not found')
}

export function formatDateTime(value: string): string {
  const timestamp = Date.parse(value)
  if (Number.isNaN(timestamp)) {
    return value
  }

  return new Intl.DateTimeFormat('zh-CN', {
    hour12: false,
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  }).format(timestamp)
}

export function formatTaskStateLabel(state: TaskProgress['state']): string {
  const labelMap: Record<TaskProgress['state'], string> = {
    queued: '排队中',
    running: '运行中',
    completed: '已完成',
    failed: '失败',
    cancelled: '已取消',
  }

  return labelMap[state]
}

export function isActiveTaskProgress(task: TaskProgress | null): boolean {
  if (task === null) {
    return false
  }

  return task.state === 'queued' || task.state === 'running'
}

export function resolvePathLeaf(path: string): string {
  const normalizedPath = path.replace(/[\\/]+$/, '')
  const segments = normalizedPath.split(/[\\/]/).filter((segment) => segment.length > 0)
  return segments[segments.length - 1] ?? path
}

export function resolveItemLocation(item: ItemDetail | null): string {
  if (item === null) {
    return '当前未选中条目'
  }

  return item.filePath ?? item.entryPath ?? item.archivePath ?? '当前条目没有可显示路径'
}

export function resolveItemDisplayLabel(item: ItemDetail | null): string {
  if (item === null) {
    return ''
  }

  return resolvePathLeaf(resolveItemLocation(item))
}

export function resolveLibrarySelection(
  libraries: LibrarySummary[],
  preferredLibraryId?: string | null,
): string | null {
  if (preferredLibraryId !== undefined && preferredLibraryId !== null) {
    const matchedLibrary = libraries.find((library) => library.id === preferredLibraryId)
    if (matchedLibrary) {
      return matchedLibrary.id
    }
  }

  return libraries[0]?.id ?? null
}

export function resolveSidebarNodeSelection(
  nodes: SidebarNodeSummary[],
  preferredNodeId?: string | null,
): string | null {
  if (preferredNodeId !== undefined && preferredNodeId !== null) {
    const matchedNode = nodes.find((node) => node.nodeId === preferredNodeId)
    if (matchedNode) {
      return matchedNode.nodeId
    }
  }

  return nodes[0]?.nodeId ?? null
}

export function normalizePathBatch(paths: string[]): string[] {
  const normalized = paths
    .map((path) => path.trim())
    .filter((path) => path.length > 0)

  return [...new Set(normalized)]
}

export function normalizeClipboardPathToken(token: string): string | null {
  const trimmedToken = token.trim().replace(/^"|"$/g, '')

  if (trimmedToken.length === 0) {
    return null
  }

  if (trimmedToken.startsWith('file://')) {
    try {
      const url = new URL(trimmedToken)
      return decodeURIComponent(url.pathname.replace(/^\//, '').replace(/\//g, '\\'))
    } catch {
      return null
    }
  }

  if (/^[a-zA-Z]:[\\/]/.test(trimmedToken) || /^\\\\/.test(trimmedToken)) {
    return trimmedToken
  }

  return null
}

export function parseClipboardPaths(rawText: string): string[] {
  const candidateTokens = rawText
    .split(/\r?\n/)
    .map((token) => normalizeClipboardPathToken(token))
    .filter((token): token is string => token !== null)

  return normalizePathBatch(candidateTokens)
}

export function isEditablePasteTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) {
    return false
  }

  const tagName = target.tagName.toLowerCase()
  return tagName === 'input' || tagName === 'textarea' || target.isContentEditable
}
