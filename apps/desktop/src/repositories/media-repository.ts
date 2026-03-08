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
  ScanRunResult,
  ScanStats,
  SubtitleHost,
  SubtitleProgress,
  SubtitleSession,
  TaskProgress,
  ThumbnailEnsureResult,
  ThumbnailProgressEvent,
  ThumbnailProfile,
} from '@mediaplayernext/contracts'
import type {
  RuntimeSmokeCheckInput,
  RuntimeSmokeCheckResult,
} from '../adapters/tauri/commands'

export interface MediaRepository {
  diagnostics: {
    checkRuntimeHealth(input: RuntimeSmokeCheckInput): Promise<RuntimeSmokeCheckResult>
  }
  library: {
    list(): Promise<LibrarySummary[]>
    add(input: AddLibraryInput): Promise<LibraryDetail>
    get(libraryId: string): Promise<LibraryDetail>
    remove(libraryId: string): Promise<void>
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
    domain: 'subtitle',
    status: 'ready',
    note: 'subtitle ping/health/session/progress 已接进 repository',
  },
  {
    domain: 'library',
    status: 'planned',
    note: 'contracts 已就位，等待 src-tauri command wiring',
  },
  {
    domain: 'scan',
    status: 'planned',
    note: '交互已稳定，等待 command/channel 接线',
  },
  {
    domain: 'items',
    status: 'planned',
    note: '等待 items list/detail 的宿主入口',
  },
  {
    domain: 'archive',
    status: 'planned',
    note: 'archive:// 已可消费，meta/list command 尚未接线',
  },
  {
    domain: 'thumbnail',
    status: 'planned',
    note: 'thumb:// 已可消费，ensure/progress command 尚未接线',
  },
  {
    domain: 'playback',
    status: 'planned',
    note: 'contracts 已有，当前仍未注册 playback command',
  },
] as const

export const i1PageDependencies: readonly PageDependencyRow[] = [
  {
    page: 'LibraryPicker',
    interaction: '列出现有媒体库并创建/删除入口',
    repositoryMethods: ['library.list', 'library.add', 'library.remove'],
    transport: ['command', 'event'],
    status: 'planned',
  },
  {
    page: 'ScanPanel',
    interaction: '启动扫描、恢复扫描、查看统计与状态',
    repositoryMethods: ['scan.start', 'scan.resume', 'scan.stats', 'scan.snapshot'],
    transport: ['command', 'channel'],
    status: 'planned',
  },
  {
    page: 'ItemsGridPage',
    interaction: '列表查询并消费缩略图 URL',
    repositoryMethods: ['items.list', 'thumbnail.ensure', 'urls.thumbnail'],
    transport: ['command', 'protocol'],
    status: 'planned',
  },
  {
    page: 'ArchiveViewerPage',
    interaction: '读取 archive entries 并消费 archive:// 图片',
    repositoryMethods: ['archive.entries', 'archive.entryDetail', 'urls.archiveEntry'],
    transport: ['command', 'protocol'],
    status: 'planned',
  },
  {
    page: 'DiagnosticsPanel',
    interaction: '校验 runtimes 与 subtitle host 健康状态',
    repositoryMethods: ['diagnostics.checkRuntimeHealth', 'subtitle.ping', 'subtitle.health'],
    transport: ['command'],
    status: 'ready',
  },
] as const
