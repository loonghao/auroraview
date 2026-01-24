import React, { useState, useEffect, useRef, useCallback } from 'react'
import { callMethod, onEvent, whenReady } from '@/utils/ipc'

interface Tab {
  id: string
  title: string
  url: string
}

interface TabsState {
  tabs: Tab[]
  activeTabId: string | null
}

/**
 * Simple Browser UI Component
 * 
 * Basic browser interface with tabs, navigation, and URL bar.
 */
export function Browser() {
  const [state, setState] = useState<TabsState>({ tabs: [], activeTabId: null })
  const [isLoading, setIsLoading] = useState(false)
  const urlBarRef = useRef<HTMLInputElement>(null)
  const iframeRef = useRef<HTMLIFrameElement>(null)

  // Initialize and listen for tab updates
  useEffect(() => {
    whenReady().then(async () => {
      try {
        const data = await callMethod<TabsState>('browser.get_state')
        if (data) setState(data)
      } catch { /* ignore */ }
    })

    const unsub = onEvent('tabs:update', (data) => {
      const tabsData = data as TabsState
      setState({ tabs: tabsData.tabs || [], activeTabId: tabsData.activeTabId })
      updateUrlBar(tabsData)
    })

    return unsub
  }, [])

  // Handle iframe load
  useEffect(() => {
    const iframe = iframeRef.current
    if (!iframe) return

    const handleLoad = () => {
      setIsLoading(false)
      try {
        const url = iframe.contentWindow?.location.href
        if (url && url !== 'about:blank' && urlBarRef.current) {
          urlBarRef.current.value = url
          callMethod('browser.navigate', { url, silent: true })
        }
      } catch { /* cross-origin restriction */ }
    }

    iframe.addEventListener('load', handleLoad)
    return () => iframe.removeEventListener('load', handleLoad)
  }, [])

  const updateUrlBar = useCallback((data: TabsState) => {
    const active = data.tabs.find(t => t.id === data.activeTabId)
    if (active?.url && urlBarRef.current) {
      urlBarRef.current.value = active.url
    }
  }, [])

  const navigate = useCallback((url?: string) => {
    let targetUrl = url || urlBarRef.current?.value.trim()
    if (!targetUrl) return

    // Add protocol if missing
    if (!targetUrl.match(/^https?:\/\//i) && !targetUrl.startsWith('about:')) {
      targetUrl = targetUrl.includes('.') && !targetUrl.includes(' ')
        ? `https://${targetUrl}`
        : `https://www.google.com/search?q=${encodeURIComponent(targetUrl)}`
    }

    setIsLoading(true)
    if (iframeRef.current) {
      iframeRef.current.src = targetUrl
    }
    callMethod('browser.navigate', { url: targetUrl })
  }, [])

  const newTab = () => callMethod('browser.new_tab')
  const closeTab = (id: string) => callMethod('browser.close_tab', { tabId: id })
  const activateTab = (id: string) => {
    const tab = state.tabs.find(t => t.id === id)
    if (tab?.url && iframeRef.current && iframeRef.current.src !== tab.url) {
      setIsLoading(true)
      iframeRef.current.src = tab.url
    }
    callMethod('browser.activate_tab', { tabId: id })
  }

  const goBack = () => {
    try {
      iframeRef.current?.contentWindow?.history.back()
    } catch { /* ignore */ }
    callMethod('browser.go_back')
  }

  const goForward = () => {
    try {
      iframeRef.current?.contentWindow?.history.forward()
    } catch { /* ignore */ }
    callMethod('browser.go_forward')
  }

  const reload = () => {
    setIsLoading(true)
    if (iframeRef.current) {
      iframeRef.current.src = iframeRef.current.src
    }
    callMethod('browser.reload')
  }

  return (
    <div className="h-full flex flex-col">
      {/* Tab Bar */}
      <div className="flex bg-[var(--bg-secondary)] px-2 pt-1 h-[34px] gap-px items-end overflow-x-auto scrollbar-none">
        {state.tabs.map(tab => (
          <div
            key={tab.id}
            onClick={() => activateTab(tab.id)}
            className={`flex items-center px-2.5 h-[30px] rounded-t-md cursor-pointer min-w-[120px] max-w-[200px] text-xs transition-all ${
              tab.id === state.activeTabId
                ? 'bg-[var(--bg-primary)] text-[var(--text-primary)] z-10'
                : 'bg-[var(--bg-tertiary)] text-[var(--text-secondary)] hover:bg-[var(--bg-hover)]'
            }`}
          >
            <span className="flex-1 truncate mr-1.5">{tab.title || 'New Tab'}</span>
            <span
              onClick={(e) => { e.stopPropagation(); closeTab(tab.id) }}
              className="w-4 h-4 flex items-center justify-center rounded opacity-0 hover:opacity-100 hover:bg-[var(--bg-active)] transition-opacity group-hover:opacity-60"
            >
              <CloseIcon />
            </span>
          </div>
        ))}
        <button
          onClick={newTab}
          className="w-7 h-7 flex items-center justify-center text-[var(--text-secondary)] rounded hover:bg-[var(--bg-hover)] hover:text-[var(--text-primary)] mb-0.5 ml-1"
        >
          <PlusIcon />
        </button>
      </div>

      {/* Toolbar */}
      <div className="flex items-center px-2 py-1 gap-1.5 bg-[var(--bg-primary)] border-b border-[var(--border-color)] h-10">
        <NavButton onClick={goBack} title="Back">
          <BackIcon />
        </NavButton>
        <NavButton onClick={goForward} title="Forward">
          <ForwardIcon />
        </NavButton>
        <NavButton onClick={reload} title="Reload">
          <ReloadIcon />
        </NavButton>
        <div className="flex-1 flex items-center bg-[var(--bg-secondary)] border border-[var(--border-color)] rounded-md h-7 px-2.5">
          <input
            ref={urlBarRef}
            className="flex-1 bg-transparent border-none text-[var(--text-primary)] text-[13px] outline-none"
            placeholder="Search or enter address"
            onKeyDown={(e) => e.key === 'Enter' && navigate()}
          />
        </div>
      </div>

      {/* Content */}
      <div className="flex-1 relative bg-[var(--bg-primary)]">
        <div className={`absolute top-0 left-0 right-0 h-0.5 bg-[var(--accent-color)] transition-transform ${isLoading ? 'loading-bar active' : 'scale-x-0'}`} />
        <iframe
          ref={iframeRef}
          className="w-full h-full border-none bg-white"
          src="about:blank"
        />
      </div>
    </div>
  )
}

function NavButton({ children, onClick, title }: { children: React.ReactNode; onClick: () => void; title: string }) {
  return (
    <button
      onClick={onClick}
      title={title}
      className="w-7 h-7 flex items-center justify-center bg-transparent border-none rounded text-[var(--text-primary)] cursor-pointer hover:bg-[var(--bg-hover)]"
    >
      {children}
    </button>
  )
}

function CloseIcon() {
  return (
    <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor">
      <path d="M4.646 4.646a.5.5 0 0 1 .708 0L8 7.293l2.646-2.647a.5.5 0 0 1 .708.708L8.707 8l2.647 2.646a.5.5 0 0 1-.708.708L8 8.707l-2.646 2.647a.5.5 0 0 1-.708-.708L7.293 8 4.646 5.354a.5.5 0 0 1 0-.708z"/>
    </svg>
  )
}

function PlusIcon() {
  return (
    <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
      <path d="M8 4a.5.5 0 0 1 .5.5v3h3a.5.5 0 0 1 0 1h-3v3a.5.5 0 0 1-1 0v-3h-3a.5.5 0 0 1 0-1h3v-3A.5.5 0 0 1 8 4z"/>
    </svg>
  )
}

function BackIcon() {
  return (
    <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
      <path d="M11.354 1.646a.5.5 0 0 1 0 .708L5.707 8l5.647 5.646a.5.5 0 0 1-.708.708l-6-6a.5.5 0 0 1 0-.708l6-6a.5.5 0 0 1 .708 0z"/>
    </svg>
  )
}

function ForwardIcon() {
  return (
    <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
      <path d="M4.646 1.646a.5.5 0 0 1 .708 0l6 6a.5.5 0 0 1 0 .708l-6 6a.5.5 0 0 1-.708-.708L10.293 8 4.646 2.354a.5.5 0 0 1 0-.708z"/>
    </svg>
  )
}

function ReloadIcon() {
  return (
    <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
      <path d="M11.534 7h3.932a.25.25 0 0 1 .192.41l-1.966 2.36a.25.25 0 0 1-.384 0l-1.966-2.36a.25.25 0 0 1 .192-.41zm-1.315 3.693a.5.5 0 0 1 .144.693 5 5 0 1 1-.393-5.326.5.5 0 0 1 .744.669 4 4 0 1 0 .285 4.657l-.78-.693z"/>
    </svg>
  )
}
