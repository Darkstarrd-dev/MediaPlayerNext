import { useMemo } from 'react'
import { AppShell } from './app/AppShell'
import { MediaRepositoryProvider } from './app/MediaRepositoryProvider'
import { createTauriMediaRepository } from './repositories/tauri-media-repository'
import './App.css'
import './styles/panels.css'
import './styles/workspace.css'

function App() {
  const repository = useMemo(() => createTauriMediaRepository(), [])

  return (
    <MediaRepositoryProvider repository={repository}>
      <AppShell />
    </MediaRepositoryProvider>
  )
}

export default App
