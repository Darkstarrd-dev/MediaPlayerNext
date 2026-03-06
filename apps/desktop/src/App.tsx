import { invoke } from '@tauri-apps/api/core'
import { useState } from 'react'
import './App.css'

function App() {
  const [name, setName] = useState('MediaPlayerNext')
  const [message, setMessage] = useState('尚未调用 Tauri command')
  const [loading, setLoading] = useState(false)

  async function handleInvokeGreet(): Promise<void> {
    setLoading(true)
    try {
      const result = await invoke<string>('greet', { name })
      setMessage(result)
    } catch (error) {
      setMessage(`调用失败：${String(error)}`)
    } finally {
      setLoading(false)
    }
  }

  return (
    <main className="shell">
      <section className="panel">
        <span className="badge">Tauri Command Demo</span>
        <h1>React 已经能调用 Rust 宿主</h1>
        <p className="lead">
          这个页面用于验证 `apps/desktop` 与 `src-tauri` 的最小桥接链路已经打通。
        </p>

        <label className="field" htmlFor="greet-name">
          <span>发送给 Rust command 的名称</span>
          <input
            id="greet-name"
            value={name}
            onChange={(event) => setName(event.target.value)}
            placeholder="输入任意名称"
          />
        </label>

        <div className="actions">
          <button type="button" onClick={() => void handleInvokeGreet()} disabled={loading}>
            {loading ? '调用中...' : '调用 greet command'}
          </button>
        </div>

        <div className="result">
          <span className="result-label">Rust 返回结果</span>
          <code>{message}</code>
        </div>
      </section>
    </main>
  )
}

export default App
