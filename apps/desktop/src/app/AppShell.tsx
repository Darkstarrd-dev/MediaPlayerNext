import { useMemo, useState } from 'react'
import type {
  PlaybackSession,
  SubtitleHost,
  SubtitleProgress,
  SubtitleSession,
} from '@mediaplayernext/contracts'
import {
  i1DomainStatuses,
  i1PageDependencies,
} from '../repositories/media-repository'
import type {
  RuntimeSmokeCheckInput,
  RuntimeSmokeCheckResult,
} from '../adapters/tauri/commands'
import { useMediaRepository } from './use-media-repository'

const DEFAULT_RUNTIME_PATHS: RuntimeSmokeCheckInput = {
  ffmpegPath: 'C:/Tools/ffmpeg-7.1.1-essentials_build/bin/ffmpeg.exe',
  ffprobePath: 'C:/Tools/ffmpeg-7.1.1-essentials_build/bin/ffprobe.exe',
  mpvPath: 'Z:/Playground/CurrentWorking/mpv/mpv.exe',
}

export function AppShell() {
  const repository = useMediaRepository()
  const [runtimePaths, setRuntimePaths] = useState<RuntimeSmokeCheckInput>(DEFAULT_RUNTIME_PATHS)
  const [runtimeResult, setRuntimeResult] = useState<RuntimeSmokeCheckResult | null>(null)
  const [runtimeError, setRuntimeError] = useState('')
  const [runtimeLoading, setRuntimeLoading] = useState(false)

  const [assetId, setAssetId] = useState('asset_001')
  const [sessionId, setSessionId] = useState('')
  const [playbackSessionId, setPlaybackSessionId] = useState('')
  const [playbackPositionMs, setPlaybackPositionMs] = useState('0')
  const [subtitleHost, setSubtitleHost] = useState<SubtitleHost | null>(null)
  const [subtitleSession, setSubtitleSession] = useState<SubtitleSession | null>(null)
  const [subtitleProgress, setSubtitleProgress] = useState<SubtitleProgress | null>(null)
  const [playbackSession, setPlaybackSession] = useState<PlaybackSession | null>(null)
  const [subtitleError, setSubtitleError] = useState('')
  const [subtitleLoading, setSubtitleLoading] = useState(false)
  const [playbackError, setPlaybackError] = useState('')
  const [playbackLoading, setPlaybackLoading] = useState(false)

  const protocolPreview = useMemo(
    () => ({
      thumbnail: repository.urls.thumbnail('thumb_001'),
      media: repository.urls.media(assetId || 'asset_001'),
      archive: repository.urls.archiveEntry('archive_entry_001'),
    }),
    [assetId, repository],
  )

  async function handleRuntimeCheck(): Promise<void> {
    setRuntimeLoading(true)
    setRuntimeError('')
    try {
      const result = await repository.diagnostics.checkRuntimeHealth(runtimePaths)
      setRuntimeResult(result)
    } catch (error) {
      setRuntimeResult(null)
      setRuntimeError(String(error))
    } finally {
      setRuntimeLoading(false)
    }
  }

  async function runSubtitleAction(action: () => Promise<void>): Promise<void> {
    setSubtitleLoading(true)
    setSubtitleError('')
    try {
      await action()
    } catch (error) {
      setSubtitleError(String(error))
    } finally {
      setSubtitleLoading(false)
    }
  }

  async function runPlaybackAction(action: () => Promise<void>): Promise<void> {
    setPlaybackLoading(true)
    setPlaybackError('')
    try {
      await action()
    } catch (error) {
      setPlaybackError(String(error))
    } finally {
      setPlaybackLoading(false)
    }
  }

  async function handleSubtitlePing(): Promise<void> {
    await runSubtitleAction(async () => {
      const host = await repository.subtitle.ping()
      setSubtitleHost(host)
    })
  }

  async function handleSubtitleHealth(): Promise<void> {
    await runSubtitleAction(async () => {
      const host = await repository.subtitle.health()
      setSubtitleHost(host)
    })
  }

  async function handleSubtitleStart(): Promise<void> {
    await runSubtitleAction(async () => {
      const nextSession = await repository.subtitle.startSession(assetId || undefined)
      setSubtitleSession(nextSession)
      setSessionId(nextSession.sessionId)
      setSubtitleProgress(null)
    })
  }

  async function handleSubtitleProgress(): Promise<void> {
    if (!sessionId) {
      setSubtitleError('请先输入 sessionId，或先启动 subtitle session')
      return
    }

    await runSubtitleAction(async () => {
      const nextProgress = await repository.subtitle.getProgress(sessionId)
      setSubtitleProgress(nextProgress)
    })
  }

  async function handleSubtitleStop(): Promise<void> {
    if (!sessionId) {
      setSubtitleError('请先输入 sessionId，或先启动 subtitle session')
      return
    }

    await runSubtitleAction(async () => {
      const nextSession = await repository.subtitle.stopSession(sessionId)
      setSubtitleSession(nextSession)
      setSessionId(nextSession.sessionId)
    })
  }

  async function handlePlaybackOpen(): Promise<void> {
    await runPlaybackAction(async () => {
      const nextSession = await repository.playback.open(assetId)
      setPlaybackSession(nextSession)
      setPlaybackSessionId(nextSession.sessionId)
    })
  }

  async function handlePlaybackStatus(): Promise<void> {
    if (!playbackSessionId) {
      setPlaybackError('请先输入 playback sessionId，或先打开 playback session')
      return
    }

    await runPlaybackAction(async () => {
      const nextSession = await repository.playback.status(playbackSessionId)
      setPlaybackSession(nextSession)
      setPlaybackSessionId(nextSession.sessionId)
    })
  }

  async function handlePlaybackSeek(): Promise<void> {
    if (!playbackSessionId) {
      setPlaybackError('请先输入 playback sessionId，或先打开 playback session')
      return
    }

    const positionMs = Number.parseInt(playbackPositionMs, 10)
    if (!Number.isFinite(positionMs) || positionMs < 0) {
      setPlaybackError('positionMs 必须是大于等于 0 的整数')
      return
    }

    await runPlaybackAction(async () => {
      const nextSession = await repository.playback.seek(playbackSessionId, positionMs)
      setPlaybackSession(nextSession)
      setPlaybackSessionId(nextSession.sessionId)
    })
  }

  return (
    <main className="shell">
      <section className="hero panel panel-hero">
        <div>
          <span className="badge">I1 Repository Shell</span>
          <h1>先冻结 repository / adapter，再等待 theme 收口</h1>
          <p className="lead">
            这个桌面壳不迁 theme，只把已经确认的交互关系收口成 repository、adapter
            与页面依赖矩阵，避免后续 React 组件继续散落 `invoke(...)`。
          </p>
        </div>
        <div className="hero-grid">
          {i1DomainStatuses.map((item) => (
            <article key={item.domain} className="status-card">
              <span className={`status-dot status-${item.status}`} />
              <strong>{item.domain}</strong>
              <p>{item.note}</p>
            </article>
          ))}
        </div>
      </section>

      <section className="workspace-grid">
        <section className="panel">
          <div className="panel-heading">
            <div>
              <span className="section-kicker">P6-3 补件</span>
              <h2>页面依赖矩阵</h2>
            </div>
          </div>
          <div className="matrix-table">
            <div className="matrix-head">页面/模块</div>
            <div className="matrix-head">交互关系</div>
            <div className="matrix-head">repository 方法</div>
            <div className="matrix-head">transport</div>
            <div className="matrix-head">状态</div>
            {i1PageDependencies.map((row) => (
              <div key={row.page} className="matrix-row">
                <div className="matrix-cell matrix-page">{row.page}</div>
                <div className="matrix-cell">{row.interaction}</div>
                <div className="matrix-cell">{row.repositoryMethods.join(' / ')}</div>
                <div className="matrix-cell">{row.transport.join(' / ')}</div>
                <div className="matrix-cell">
                  <span className={`pill pill-${row.status}`}>{row.status}</span>
                </div>
              </div>
            ))}
          </div>
        </section>

        <section className="panel stack-panel">
          <div className="panel-heading">
            <div>
              <span className="section-kicker">已接线能力</span>
              <h2>宿主诊断与 subtitle</h2>
            </div>
          </div>

          <section className="subpanel">
            <h3>Runtime Smoke Check</h3>
            <label className="field" htmlFor="ffmpeg-path">
              <span>ffmpegPath</span>
              <input
                id="ffmpeg-path"
                value={runtimePaths.ffmpegPath}
                onChange={(event) =>
                  setRuntimePaths((current) => ({ ...current, ffmpegPath: event.target.value }))
                }
              />
            </label>
            <label className="field" htmlFor="ffprobe-path">
              <span>ffprobePath</span>
              <input
                id="ffprobe-path"
                value={runtimePaths.ffprobePath}
                onChange={(event) =>
                  setRuntimePaths((current) => ({ ...current, ffprobePath: event.target.value }))
                }
              />
            </label>
            <label className="field" htmlFor="mpv-path">
              <span>mpvPath</span>
              <input
                id="mpv-path"
                value={runtimePaths.mpvPath}
                onChange={(event) =>
                  setRuntimePaths((current) => ({ ...current, mpvPath: event.target.value }))
                }
              />
            </label>
            <div className="actions">
              <button type="button" onClick={() => void handleRuntimeCheck()} disabled={runtimeLoading}>
                {runtimeLoading ? '校验中...' : '执行 runtime_smoke_check'}
              </button>
            </div>
            {runtimeError ? <p className="error-text">{runtimeError}</p> : null}
            {runtimeResult ? (
              <dl className="kv-list">
                <div>
                  <dt>sqlite</dt>
                  <dd>{runtimeResult.sqliteVersion}</dd>
                </div>
                <div>
                  <dt>ffmpeg</dt>
                  <dd>{runtimeResult.ffmpegFirstLine}</dd>
                </div>
                <div>
                  <dt>ffprobe</dt>
                  <dd>{runtimeResult.ffprobeFirstLine}</dd>
                </div>
                <div>
                  <dt>mpv</dt>
                  <dd>{runtimeResult.mpvFirstLine}</dd>
                </div>
              </dl>
            ) : null}
          </section>

          <section className="subpanel">
            <h3>Subtitle Host</h3>
            <label className="field" htmlFor="subtitle-asset-id">
              <span>assetId</span>
              <input
                id="subtitle-asset-id"
                value={assetId}
                onChange={(event) => setAssetId(event.target.value)}
              />
            </label>
            <label className="field" htmlFor="subtitle-session-id">
              <span>sessionId</span>
              <input
                id="subtitle-session-id"
                value={sessionId}
                onChange={(event) => setSessionId(event.target.value)}
              />
            </label>
            <div className="actions actions-wrap">
              <button type="button" onClick={() => void handleSubtitlePing()} disabled={subtitleLoading}>
                ping
              </button>
              <button type="button" onClick={() => void handleSubtitleHealth()} disabled={subtitleLoading}>
                health
              </button>
              <button type="button" onClick={() => void handleSubtitleStart()} disabled={subtitleLoading}>
                startSession
              </button>
              <button type="button" onClick={() => void handleSubtitleProgress()} disabled={subtitleLoading}>
                getProgress
              </button>
              <button type="button" onClick={() => void handleSubtitleStop()} disabled={subtitleLoading}>
                stopSession
              </button>
            </div>
            {subtitleError ? <p className="error-text">{subtitleError}</p> : null}
            <div className="result-grid">
              <article className="result-card">
                <span className="result-label">Host</span>
                <code>{formatSubtitleHost(subtitleHost)}</code>
              </article>
              <article className="result-card">
                <span className="result-label">Session</span>
                <code>{formatSubtitleSession(subtitleSession)}</code>
              </article>
              <article className="result-card">
                <span className="result-label">Progress</span>
                <code>{formatSubtitleProgress(subtitleProgress)}</code>
              </article>
            </div>
          </section>

          <section className="subpanel">
            <h3>Playback Host</h3>
            <label className="field" htmlFor="playback-asset-id">
              <span>assetId</span>
              <input
                id="playback-asset-id"
                value={assetId}
                onChange={(event) => setAssetId(event.target.value)}
              />
            </label>
            <label className="field" htmlFor="playback-session-id">
              <span>sessionId</span>
              <input
                id="playback-session-id"
                value={playbackSessionId}
                onChange={(event) => setPlaybackSessionId(event.target.value)}
              />
            </label>
            <label className="field" htmlFor="playback-position-ms">
              <span>positionMs</span>
              <input
                id="playback-position-ms"
                value={playbackPositionMs}
                onChange={(event) => setPlaybackPositionMs(event.target.value)}
              />
            </label>
            <div className="actions actions-wrap">
              <button type="button" onClick={() => void handlePlaybackOpen()} disabled={playbackLoading}>
                open
              </button>
              <button type="button" onClick={() => void handlePlaybackStatus()} disabled={playbackLoading}>
                status
              </button>
              <button type="button" onClick={() => void handlePlaybackSeek()} disabled={playbackLoading}>
                seek
              </button>
            </div>
            {playbackError ? <p className="error-text">{playbackError}</p> : null}
            <article className="result-card">
              <span className="result-label">Playback</span>
              <code>{formatPlaybackSession(playbackSession)}</code>
            </article>
          </section>
        </section>

        <section className="panel">
          <div className="panel-heading">
            <div>
              <span className="section-kicker">协议统一入口</span>
              <h2>Protocol URL builders</h2>
            </div>
          </div>
          <div className="result-grid">
            <article className="result-card">
              <span className="result-label">thumb://</span>
              <code>{protocolPreview.thumbnail}</code>
            </article>
            <article className="result-card">
              <span className="result-label">media://</span>
              <code>{protocolPreview.media}</code>
            </article>
            <article className="result-card">
              <span className="result-label">archive://</span>
              <code>{protocolPreview.archive}</code>
            </article>
          </div>
          <p className="muted-note">
            组件层后续只能从 repository/adapter 取 URL，不能重新散写协议字符串。
          </p>
        </section>
      </section>
    </main>
  )
}

function formatSubtitleHost(host: SubtitleHost | null): string {
  if (host === null) {
    return '尚未调用'
  }

  return JSON.stringify(
    {
      executable: host.executable,
      running: host.running,
      restartCount: host.restartCount,
      nodeVersion: host.health?.nodeVersion,
      activeSessions: host.health?.activeSessions,
    },
    null,
    2,
  )
}

function formatSubtitleSession(session: SubtitleSession | null): string {
  if (session === null) {
    return '尚未调用'
  }

  return JSON.stringify(session, null, 2)
}

function formatSubtitleProgress(progress: SubtitleProgress | null): string {
  if (progress === null) {
    return '尚未调用'
  }

  return JSON.stringify(progress, null, 2)
}

function formatPlaybackSession(session: PlaybackSession | null): string {
  if (session === null) {
    return '尚未调用'
  }

  return JSON.stringify(session, null, 2)
}
