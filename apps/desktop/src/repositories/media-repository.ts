import type {
  AddLibraryInput,
  ArchiveEntryDetail,
  ArchiveEntrySummary,
  ArchiveNormalizeResult,
  ItemDetail,
  ItemListEntry,
  ItemsListQuery,
  LibraryDetail,
  LibrarySummary,
  PlaybackSession,
  RuntimeInfo,
  ScanRunResult,
  ScanStats,
  SidebarNodeSummary,
  SetRuntimeStoragePathsInput,
  SubtitleHost,
  SubtitleProgress,
  SubtitleSession,
  TaskProgress,
  ThumbnailEnsureResult,
  ThumbnailProgressEvent,
  ThumbnailProfile,
  WorkspaceCursor,
} from '@mediaplayernext/contracts'
import type {
  RuntimeSmokeCheckInput,
  RuntimeSmokeCheckResult,
} from '../adapters/tauri/commands'

export interface MediaRepository {
  diagnostics: {
    checkRuntimeHealth(input: RuntimeSmokeCheckInput): Promise<RuntimeSmokeCheckResult>
  }
  database: {
    readRuntimeInfo(): Promise<RuntimeInfo>
    setStoragePaths(input: SetRuntimeStoragePathsInput): Promise<RuntimeInfo>
    clear(): Promise<void>
    readWorkspaceCursor(): Promise<WorkspaceCursor | null>
    writeWorkspaceCursor(cursor: WorkspaceCursor): Promise<void>
  }
  library: {
    list(): Promise<LibrarySummary[]>
    add(input: AddLibraryInput): Promise<LibraryDetail>
    get(libraryId: string): Promise<LibraryDetail>
    remove(libraryId: string): Promise<void>
    nodes(libraryId: string): Promise<SidebarNodeSummary[]>
  }
  scan: {
    start(libraryId: string): Promise<ScanRunResult>
    resume(libraryId: string): Promise<ScanRunResult>
    stats(libraryId: string): Promise<ScanStats>
    snapshot(libraryId: string): Promise<TaskProgress>
  }
  items: {
    list(query: ItemsListQuery): Promise<ItemListEntry[]>
    detail(assetId: string): Promise<ItemDetail>
  }
  archive: {
    entries(sourceId: string): Promise<ArchiveEntrySummary[]>
    entryDetail(archiveEntryId: string): Promise<ArchiveEntryDetail>
    normalize(sourceId: string): Promise<ArchiveNormalizeResult>
    normalizeStatus(taskId: string): Promise<TaskProgress>
  }
  thumbnail: {
    ensure(assetId: string, profile: ThumbnailProfile): Promise<ThumbnailEnsureResult>
    get(thumbnailKey: string): Promise<string>
    progress(taskId: string): Promise<ThumbnailProgressEvent>
  }
  playback: {
    open(assetId: string): Promise<PlaybackSession>
    status(sessionId: string): Promise<PlaybackSession>
    seek(sessionId: string, positionMs: number): Promise<PlaybackSession>
  }
  subtitle: {
    ping(): Promise<SubtitleHost>
    health(): Promise<SubtitleHost>
    startSession(assetId?: string): Promise<SubtitleSession>
    stopSession(sessionId: string): Promise<SubtitleSession>
    getProgress(sessionId: string): Promise<SubtitleProgress>
  }
  urls: {
    thumbnail(thumbnailKey: string): string
    media(assetId: string): string
    archiveEntry(archiveEntryId: string): string
  }
}

export interface RepositoryDomainStatus {
  domain: keyof Omit<MediaRepository, 'urls'>
  status: 'ready' | 'planned'
  note: string
}

export interface PageDependencyRow {
  page: string
  interaction: string
  repositoryMethods: string[]
  transport: string[]
  status: 'ready' | 'planned'
}

export const i1DomainStatuses: readonly RepositoryDomainStatus[] = [
  {
    domain: 'diagnostics',
    status: 'ready',
    note: 'runtime_smoke_check 已能通过 repository 调用',
  },
  {
    domain: 'database',
    status: 'ready',
    note: 'readRuntimeInfo/setStoragePaths/clear 已通过 repository 接入 Tauri command',
  },
  {
    domain: 'subtitle',
    status: 'ready',
    note: 'subtitle ping/health/session/progress 已接进 repository',
  },
  {
    domain: 'library',
    status: 'ready',
    note: 'list/add/get/remove 已通过 repository 接入 Tauri command',
  },
  {
    domain: 'scan',
    status: 'ready',
    note: 'start/resume/stats/snapshot 已接入 Tauri command',
  },
  {
    domain: 'items',
    status: 'ready',
    note: 'items list/detail 已接入宿主命令',
  },
  {
    domain: 'archive',
    status: 'ready',
    note: 'archive entries/detail/normalize/status 与 archive:// 已接通',
  },
  {
    domain: 'thumbnail',
    status: 'planned',
    note: 'thumb:// 与 thumbnail.ensure 已可用，但统一 progress 流仍未形成',
  },
  {
    domain: 'playback',
    status: 'ready',
    note: 'open/status/seek 已通过 repository 接入 Tauri command',
  },
] as const

export const i1PageDependencies: readonly PageDependencyRow[] = [
  {
    page: 'SettingsDatabasePage',
    interaction: '读取运行时存储路径并执行 SQL/缩略图目录切换与数据库清除',
    repositoryMethods: ['database.readRuntimeInfo', 'database.setStoragePaths', 'database.clear'],
    transport: ['command'],
    status: 'ready',
  },
  {
    page: 'LibraryPicker',
    interaction: '列出现有媒体库并创建/删除入口',
    repositoryMethods: ['library.list', 'library.add', 'library.remove'],
    transport: ['command'],
    status: 'ready',
  },
  {
    page: 'ScanPanel',
    interaction: '启动扫描、恢复扫描、查看统计与状态',
    repositoryMethods: ['scan.start', 'scan.resume', 'scan.stats', 'scan.snapshot'],
    transport: ['command'],
    status: 'ready',
  },
  {
    page: 'ItemsGridPage',
    interaction: '列表查询并消费缩略图 URL',
    repositoryMethods: ['items.list', 'thumbnail.ensure', 'urls.thumbnail'],
    transport: ['command', 'protocol'],
    status: 'ready',
  },
  {
    page: 'ArchiveViewerPage',
    interaction: '读取 archive entries 并消费 archive:// 图片',
    repositoryMethods: ['archive.entries', 'archive.entryDetail', 'urls.archiveEntry'],
    transport: ['command', 'protocol'],
    status: 'ready',
  },
  {
    page: 'DiagnosticsPanel',
    interaction: '校验 runtimes 与 subtitle host 健康状态',
    repositoryMethods: ['diagnostics.checkRuntimeHealth', 'subtitle.ping', 'subtitle.health'],
    transport: ['command'],
    status: 'ready',
  },
] as const
