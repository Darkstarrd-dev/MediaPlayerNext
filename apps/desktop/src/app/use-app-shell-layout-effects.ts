import { useEffect } from 'react'
import { clampNumber } from './app-shell-utils'
import { SETTINGS_STORAGE_KEYS } from './app-shell-layout-constants'

interface LayoutPreview {
  layoutGapPx: number
  paneInnerPaddingPx: number
  paneStackGapPx: number
  paneHeaderHeightPx: number
  paneFooterHeightPx: number
  splitterWidthPx: number
  normalizedLayoutGapScaleCoeff: number
  normalizedPaneInnerGapScaleCoeff: number
  normalizedPaneStackGapScaleCoeff: number
  normalizedSplitterWidthScaleCoeff: number
}

interface WorkspaceLayout {
  sidebarWidthPx: number
  metaWidthPx: number
}

interface DragState {
  target: 'left' | 'right'
  startX: number
  startSidebarWidthPx: number
  startMetaWidthPx: number
}

interface UseAppShellLayoutEffectsParams {
  settingsBackdropOpacity: number
  layoutGapScaleCoeff: number
  paneInnerGapScaleCoeff: number
  paneStackGapScaleCoeff: number
  splitterWidthScaleCoeff: number
  thumbnailZoomLevel: number
  layoutPreview: LayoutPreview
  workspaceLayout: WorkspaceLayout
  dragState: DragState | null
  setViewportWidth: (value: number) => void
  setSidebarWidthPx: (value: number) => void
  setMetaWidthPx: (value: number) => void
  setDragState: (value: DragState | null) => void
  mainGridElement: HTMLDivElement | null
  setMainGridSize: (updater: (current: { width: number; height: number }) => { width: number; height: number }) => void
}

export function useAppShellLayoutEffects(params: UseAppShellLayoutEffectsParams) {
  const {
    settingsBackdropOpacity,
    layoutGapScaleCoeff,
    paneInnerGapScaleCoeff,
    paneStackGapScaleCoeff,
    splitterWidthScaleCoeff,
    thumbnailZoomLevel,
    layoutPreview,
    workspaceLayout,
    dragState,
    setViewportWidth,
    setSidebarWidthPx,
    setMetaWidthPx,
    setDragState,
    mainGridElement,
    setMainGridSize,
  } = params

  useEffect(() => {
    if (typeof window === 'undefined') {
      return
    }

    const updateViewportWidth = (): void => {
      setViewportWidth(window.innerWidth)
    }

    updateViewportWidth()
    window.addEventListener('resize', updateViewportWidth)

    return () => {
      window.removeEventListener('resize', updateViewportWidth)
    }
  }, [setViewportWidth])

  useEffect(() => {
    const root = document.documentElement

    root.style.setProperty(
      '--mpx-settings-backdrop-opacity',
      `${clampNumber(settingsBackdropOpacity, 0, 100).toFixed(0)}%`,
    )
    root.style.setProperty('--mpx-layout-gap-scale', layoutPreview.normalizedLayoutGapScaleCoeff.toFixed(2))
    root.style.setProperty('--mpx-layout-gap-px', `${layoutPreview.layoutGapPx}px`)
    root.style.setProperty('--mpx-layout-padding', `${layoutPreview.layoutGapPx}px`)
    root.style.setProperty(
      '--mpx-header-floating-gap',
      `${layoutPreview.layoutGapPx}px ${layoutPreview.layoutGapPx}px 0px`,
    )
    root.style.setProperty(
      '--mpx-pane-inner-gap-scale',
      layoutPreview.normalizedPaneInnerGapScaleCoeff.toFixed(2),
    )
    root.style.setProperty('--mpx-pane-inner-padding-px', `${layoutPreview.paneInnerPaddingPx}px`)
    root.style.setProperty(
      '--mpx-pane-stack-gap-scale',
      layoutPreview.normalizedPaneStackGapScaleCoeff.toFixed(2),
    )
    root.style.setProperty('--mpx-pane-stack-gap-px', `${layoutPreview.paneStackGapPx}px`)
    root.style.setProperty('--mpx-pane-section-gap-px', `${layoutPreview.paneStackGapPx}px`)
    root.style.setProperty('--mpx-pane-header-height-px', `${layoutPreview.paneHeaderHeightPx}px`)
    root.style.setProperty('--mpx-pane-footer-height-px', `${layoutPreview.paneFooterHeightPx}px`)
    root.style.setProperty(
      '--mpx-splitter-width-scale',
      layoutPreview.normalizedSplitterWidthScaleCoeff.toFixed(2),
    )
    root.style.setProperty('--mpx-splitter-width', `${layoutPreview.splitterWidthPx}px`)
  }, [layoutPreview, settingsBackdropOpacity])

  useEffect(() => {
    if (typeof window === 'undefined') {
      return
    }

    window.sessionStorage.setItem(
      SETTINGS_STORAGE_KEYS.settingsBackdropOpacity,
      settingsBackdropOpacity.toString(),
    )
    window.sessionStorage.setItem(
      SETTINGS_STORAGE_KEYS.layoutGapScaleCoeff,
      layoutGapScaleCoeff.toString(),
    )
    window.sessionStorage.setItem(
      SETTINGS_STORAGE_KEYS.paneInnerGapScaleCoeff,
      paneInnerGapScaleCoeff.toString(),
    )
    window.sessionStorage.setItem(
      SETTINGS_STORAGE_KEYS.paneStackGapScaleCoeff,
      paneStackGapScaleCoeff.toString(),
    )
    window.sessionStorage.setItem(
      SETTINGS_STORAGE_KEYS.splitterWidthScaleCoeff,
      splitterWidthScaleCoeff.toString(),
    )
    window.sessionStorage.setItem(
      SETTINGS_STORAGE_KEYS.sidebarWidthPx,
      workspaceLayout.sidebarWidthPx.toString(),
    )
    window.sessionStorage.setItem(SETTINGS_STORAGE_KEYS.metaWidthPx, workspaceLayout.metaWidthPx.toString())
    window.sessionStorage.setItem(SETTINGS_STORAGE_KEYS.thumbnailZoomLevel, thumbnailZoomLevel.toString())
  }, [
    layoutGapScaleCoeff,
    paneInnerGapScaleCoeff,
    paneStackGapScaleCoeff,
    settingsBackdropOpacity,
    splitterWidthScaleCoeff,
    thumbnailZoomLevel,
    workspaceLayout.metaWidthPx,
    workspaceLayout.sidebarWidthPx,
  ])

  useEffect(() => {
    if (!dragState) {
      return
    }

    const previousUserSelect = document.body.style.userSelect
    const previousCursor = document.body.style.cursor
    document.body.style.userSelect = 'none'
    document.body.style.cursor = 'col-resize'

    const handlePointerMove = (event: PointerEvent): void => {
      const deltaX = event.clientX - dragState.startX

      if (dragState.target === 'left') {
        setSidebarWidthPx(dragState.startSidebarWidthPx + deltaX)
        return
      }

      setMetaWidthPx(dragState.startMetaWidthPx - deltaX)
    }

    const handlePointerUp = (): void => {
      setDragState(null)
    }

    window.addEventListener('pointermove', handlePointerMove)
    window.addEventListener('pointerup', handlePointerUp)

    return () => {
      document.body.style.userSelect = previousUserSelect
      document.body.style.cursor = previousCursor
      window.removeEventListener('pointermove', handlePointerMove)
      window.removeEventListener('pointerup', handlePointerUp)
    }
  }, [dragState, setDragState, setMetaWidthPx, setSidebarWidthPx])

  useEffect(() => {
    if (mainGridElement === null) {
      return
    }

    const updateGridSize = (width: number, height: number) => {
      const nextWidth = Math.max(0, Math.round(width))
      const nextHeight = Math.max(0, Math.round(height))

      setMainGridSize((current) => {
        if (current.width === nextWidth && current.height === nextHeight) {
          return current
        }

        return {
          width: nextWidth,
          height: nextHeight,
        }
      })
    }

    const initialRect = mainGridElement.getBoundingClientRect()
    updateGridSize(initialRect.width, initialRect.height)

    const observer = new ResizeObserver((entries) => {
      const entry = entries[0]
      if (!entry) {
        return
      }

      updateGridSize(entry.contentRect.width, entry.contentRect.height)
    })

    observer.observe(mainGridElement)
    return () => observer.disconnect()
  }, [mainGridElement, setMainGridSize])
}
