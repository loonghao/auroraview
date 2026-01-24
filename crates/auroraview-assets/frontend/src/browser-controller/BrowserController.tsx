import React, { useState, useEffect, useRef, useCallback } from 'react'
import { callMethod, onEvent, whenReady, sendEvent } from '@/utils/ipc'

interface Tab {
  id: string
  title: string
  url: string
  favicon?: string
  loading?: boolean
  pinned?: boolean
  muted?: boolean
  playing?: boolean
}

interface TabsState {
  tabs: Tab[]
  activeTabId: string | null
}

/**
 * Full-featured Browser Controller (Edge-like)
 * 
 * Provides a complete browser interface with:
 * - Tabbed browsing with favicon support
 * - Window controls for frameless mode
 * - Navigation toolbar (back, forward, reload, home)
 * - URL bar with loading indicator
 * - Dark/light theme support
 */
export function BrowserController() {
  const [state, setState] = useState<TabsState>({ tabs: [], activeTabId: null })
  const [isLoading, setIsLoading] = useState(false)
  const [canGoBack, setCanGoBack] = useState(false)
  const [canGoForward, setCanGoForward] = useState(false)
  const urlBarRef = useRef<HTMLInputElement>(null)
  const [urlFocused, setUrlFocused] = useState(false)
  const [frameless, setFrameless] = useState(false)

  // Initialize and listen for events
  useEffect(() => {
    whenReady().then(async () => {
      try {
        const data = await callMethod<TabsState>('browser.get_state')
        if (data) {
          setState(data)
          updateUrlBar(data)
        }
        // Check if we're in frameless mode
        const isFrameless = await callMethod<boolean>('window.is_frameless')
        setFrameless(!!isFrameless)
      } catch { /* ignore */ }
    })

    const unsubTabs = onEvent('tabs:update', (data) => {
      const tabsData = data as TabsState
      setState({ tabs: tabsData.tabs || [], activeTabId: tabsData.activeTabId })
      updateUrlBar(tabsData)
    })

    const unsubNav = onEvent('navigation:update', (data) => {
      const navData = data as { canGoBack?: boolean; canGoForward?: boolean; loading?: boolean }
      if (navData.canGoBack !== undefined) setCanGoBack(navData.canGoBack)
      if (navData.canGoForward !== undefined) setCanGoForward(navData.canGoForward)
      if (navData.loading !== undefined) setIsLoading(navData.loading)
    })

    return () => {
      unsubTabs()
      unsubNav()
    }
  }, [])

  const updateUrlBar = useCallback((data: TabsState) => {
    const active = data.tabs.find(t => t.id === data.activeTabId)
    if (active?.url && urlBarRef.current && !urlFocused) {
      urlBarRef.current.value = active.url
    }
  }, [urlFocused])

  // Navigation functions
  const navigate = useCallback((url?: string) => {
    let targetUrl = url || urlBarRef.current?.value.trim()
    if (!targetUrl) return

    if (!targetUrl.match(/^https?:\/\//i) && !targetUrl.startsWith('about:')) {
      targetUrl = targetUrl.includes('.') && !targetUrl.includes(' ')
        ? `https://${targetUrl}`
        : `https://www.google.com/search?q=${encodeURIComponent(targetUrl)}`
    }

    setIsLoading(true)
    callMethod('browser.navigate', { url: targetUrl })
  }, [])

  const goBack = () => callMethod('browser.go_back')
  const goForward = () => callMethod('browser.go_forward')
  const reload = () => {
    setIsLoading(true)
    callMethod('browser.reload')
  }
  const stopLoading = () => {
    setIsLoading(false)
    callMethod('browser.stop')
  }
  const goHome = () => callMethod('browser.go_home')

  // Tab functions
  const newTab = () => callMethod('browser.new_tab')
  const closeTab = (id: string) => callMethod('browser.close_tab', { tabId: id })
  const activateTab = (id: string) => callMethod('browser.activate_tab', { tabId: id })
  const pinTab = (id: string) => callMethod('browser.pin_tab', { tabId: id })
  const toggleMute = (id: string) => callMethod('browser.toggle_mute', { tabId: id })

  // Window controls
  const minimize = () => sendEvent('window:minimize', {})
  const maximize = () => sendEvent('window:maximize', {})
  const close = () => sendEvent('window:close', {})

  const activeTab = state.tabs.find(t => t.id === state.activeTabId)

  return (
    <div className="h-full flex flex-col bg-[var(--bg-secondary)]">
      {/* Tab Bar */}
      <div className="tab-bar">
        {state.tabs.map(tab => (
          <div
            key={tab.id}
            className={`tab ${tab.id === state.activeTabId ? 'active' : ''} ${tab.pinned ? 'pinned' : ''}`}
            onClick={() => activateTab(tab.id)}
            onDoubleClick={() => pinTab(tab.id)}
            title={tab.url}
          >
            <TabFavicon favicon={tab.favicon} loading={tab.loading} />
            <span className="tab-title">{tab.title || 'New Tab'}</span>
            {tab.playing && (
              <span
                className={`tab-audio ${tab.muted ? 'muted' : ''}`}
                onClick={(e) => { e.stopPropagation(); toggleMute(tab.id) }}
                title={tab.muted ? 'Unmute tab' : 'Mute tab'}
              >
                {tab.muted ? <MutedIcon /> : <AudioIcon />}
              </span>
            )}
            <span
              className="tab-close"
              onClick={(e) => { e.stopPropagation(); closeTab(tab.id) }}
              title="Close tab"
            >
              <CloseIcon />
            </span>
          </div>
        ))}
        <button className="new-tab-btn" onClick={newTab} title="New tab">
          <PlusIcon />
        </button>

        {/* Window Controls for frameless */}
        {frameless && (
          <div className="window-controls">
            <button className="window-btn minimize" onClick={minimize} title="Minimize">
              <MinimizeIcon />
            </button>
            <button className="window-btn maximize" onClick={maximize} title="Maximize">
              <MaximizeIcon />
            </button>
            <button className="window-btn close" onClick={close} title="Close">
              <CloseIcon />
            </button>
          </div>
        )}
      </div>

      {/* Toolbar */}
      <div className="toolbar">
        <NavButton onClick={goBack} disabled={!canGoBack} title="Back (Alt+Left)">
          <BackIcon />
        </NavButton>
        <NavButton onClick={goForward} disabled={!canGoForward} title="Forward (Alt+Right)">
          <ForwardIcon />
        </NavButton>
        <NavButton onClick={isLoading ? stopLoading : reload} title={isLoading ? 'Stop' : 'Reload (F5)'}>
          {isLoading ? <StopIcon /> : <ReloadIcon />}
        </NavButton>
        <NavButton onClick={goHome} title="Home">
          <HomeIcon />
        </NavButton>

        {/* URL Bar */}
        <div className={`url-bar ${urlFocused ? 'focused' : ''}`}>
          <div className="url-bar-icon">
            {activeTab?.url?.startsWith('https://') ? <LockIcon /> : <GlobeIcon />}
          </div>
          <input
            ref={urlBarRef}
            className="url-input"
            placeholder="Search or enter address"
            onKeyDown={(e) => e.key === 'Enter' && navigate()}
            onFocus={() => {
              setUrlFocused(true)
              urlBarRef.current?.select()
            }}
            onBlur={() => setUrlFocused(false)}
          />
          {isLoading && <div className="url-loading" />}
        </div>

        {/* Extension area (placeholder) */}
        <div className="toolbar-actions">
          <NavButton onClick={() => callMethod('browser.show_downloads')} title="Downloads">
            <DownloadIcon />
          </NavButton>
          <NavButton onClick={() => callMethod('browser.show_menu')} title="Settings and more">
            <MenuIcon />
          </NavButton>
        </div>
      </div>

      {/* Loading indicator */}
      <div className={`loading-bar ${isLoading ? 'active' : ''}`} />
    </div>
  )
}

function TabFavicon({ favicon, loading }: { favicon?: string; loading?: boolean }) {
  if (loading) {
    return (
      <div className="tab-favicon">
        <div className="tab-favicon-spinner" />
      </div>
    )
  }
  if (favicon) {
    return (
      <div className="tab-favicon">
        <img src={favicon} alt="" onError={(e) => (e.currentTarget.style.display = 'none')} />
      </div>
    )
  }
  return (
    <div className="tab-favicon">
      <GlobeIcon />
    </div>
  )
}

function NavButton({ children, onClick, disabled, title }: {
  children: React.ReactNode
  onClick: () => void
  disabled?: boolean
  title: string
}) {
  return (
    <button
      className={`nav-btn ${disabled ? 'disabled' : ''}`}
      onClick={onClick}
      disabled={disabled}
      title={title}
    >
      {children}
    </button>
  )
}

// Icons
function CloseIcon() {
  return (
    <svg viewBox="0 0 16 16" fill="currentColor">
      <path d="M4.646 4.646a.5.5 0 0 1 .708 0L8 7.293l2.646-2.647a.5.5 0 0 1 .708.708L8.707 8l2.647 2.646a.5.5 0 0 1-.708.708L8 8.707l-2.646 2.647a.5.5 0 0 1-.708-.708L7.293 8 4.646 5.354a.5.5 0 0 1 0-.708z"/>
    </svg>
  )
}

function PlusIcon() {
  return (
    <svg viewBox="0 0 16 16" fill="currentColor">
      <path d="M8 4a.5.5 0 0 1 .5.5v3h3a.5.5 0 0 1 0 1h-3v3a.5.5 0 0 1-1 0v-3h-3a.5.5 0 0 1 0-1h3v-3A.5.5 0 0 1 8 4z"/>
    </svg>
  )
}

function BackIcon() {
  return (
    <svg viewBox="0 0 16 16" fill="currentColor">
      <path d="M11.354 1.646a.5.5 0 0 1 0 .708L5.707 8l5.647 5.646a.5.5 0 0 1-.708.708l-6-6a.5.5 0 0 1 0-.708l6-6a.5.5 0 0 1 .708 0z"/>
    </svg>
  )
}

function ForwardIcon() {
  return (
    <svg viewBox="0 0 16 16" fill="currentColor">
      <path d="M4.646 1.646a.5.5 0 0 1 .708 0l6 6a.5.5 0 0 1 0 .708l-6 6a.5.5 0 0 1-.708-.708L10.293 8 4.646 2.354a.5.5 0 0 1 0-.708z"/>
    </svg>
  )
}

function ReloadIcon() {
  return (
    <svg viewBox="0 0 16 16" fill="currentColor">
      <path d="M11.534 7h3.932a.25.25 0 0 1 .192.41l-1.966 2.36a.25.25 0 0 1-.384 0l-1.966-2.36a.25.25 0 0 1 .192-.41zm-1.315 3.693a.5.5 0 0 1 .144.693 5 5 0 1 1-.393-5.326.5.5 0 0 1 .744.669 4 4 0 1 0 .285 4.657l-.78-.693z"/>
    </svg>
  )
}

function StopIcon() {
  return (
    <svg viewBox="0 0 16 16" fill="currentColor">
      <path d="M5 3.5h6A1.5 1.5 0 0 1 12.5 5v6a1.5 1.5 0 0 1-1.5 1.5H5A1.5 1.5 0 0 1 3.5 11V5A1.5 1.5 0 0 1 5 3.5z"/>
    </svg>
  )
}

function HomeIcon() {
  return (
    <svg viewBox="0 0 16 16" fill="currentColor">
      <path d="M8.707 1.5a1 1 0 0 0-1.414 0L.646 8.146a.5.5 0 0 0 .708.708L2 8.207V13.5A1.5 1.5 0 0 0 3.5 15h9a1.5 1.5 0 0 0 1.5-1.5V8.207l.646.647a.5.5 0 0 0 .708-.708L8.707 1.5ZM3 8.207l5-5 5 5V13.5a.5.5 0 0 1-.5.5h-9a.5.5 0 0 1-.5-.5V8.207z"/>
    </svg>
  )
}

function LockIcon() {
  return (
    <svg viewBox="0 0 16 16" fill="currentColor">
      <path d="M8 1a2 2 0 0 1 2 2v4H6V3a2 2 0 0 1 2-2zm3 6V3a3 3 0 0 0-6 0v4a2 2 0 0 0-2 2v5a2 2 0 0 0 2 2h6a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2z"/>
    </svg>
  )
}

function GlobeIcon() {
  return (
    <svg viewBox="0 0 16 16" fill="currentColor">
      <path d="M0 8a8 8 0 1 1 16 0A8 8 0 0 1 0 8zm7.5-6.923c-.67.204-1.335.82-1.887 1.855A7.97 7.97 0 0 0 5.145 4H7.5V1.077zM4.09 4a9.267 9.267 0 0 1 .64-1.539 6.7 6.7 0 0 1 .597-.933A7.025 7.025 0 0 0 2.255 4H4.09zm-.582 3.5c.03-.877.138-1.718.312-2.5H1.674a6.958 6.958 0 0 0-.656 2.5h2.49z"/>
    </svg>
  )
}

function AudioIcon() {
  return (
    <svg viewBox="0 0 16 16" fill="currentColor">
      <path d="M11.536 14.01A8.473 8.473 0 0 0 14.026 8a8.473 8.473 0 0 0-2.49-6.01l-.708.707A7.476 7.476 0 0 1 13.025 8c0 2.071-.84 3.946-2.197 5.303l.708.707z"/>
      <path d="M10.121 12.596A6.48 6.48 0 0 0 12.025 8a6.48 6.48 0 0 0-1.904-4.596l-.707.707A5.483 5.483 0 0 1 11.025 8a5.483 5.483 0 0 1-1.61 3.89l.706.706z"/>
      <path d="M8.707 11.182A4.486 4.486 0 0 0 10.025 8a4.486 4.486 0 0 0-1.318-3.182L8 5.525A3.489 3.489 0 0 1 9.025 8 3.49 3.49 0 0 1 8 10.475l.707.707zM6.717 3.55A.5.5 0 0 1 7 4v8a.5.5 0 0 1-.812.39L3.825 10.5H1.5A.5.5 0 0 1 1 10V6a.5.5 0 0 1 .5-.5h2.325l2.363-1.89a.5.5 0 0 1 .529-.06z"/>
    </svg>
  )
}

function MutedIcon() {
  return (
    <svg viewBox="0 0 16 16" fill="currentColor">
      <path d="M6.717 3.55A.5.5 0 0 1 7 4v8a.5.5 0 0 1-.812.39L3.825 10.5H1.5A.5.5 0 0 1 1 10V6a.5.5 0 0 1 .5-.5h2.325l2.363-1.89a.5.5 0 0 1 .529-.06zm7.137 2.096a.5.5 0 0 1 0 .708L12.207 8l1.647 1.646a.5.5 0 0 1-.708.708L11.5 8.707l-1.646 1.647a.5.5 0 0 1-.708-.708L10.793 8 9.146 6.354a.5.5 0 1 1 .708-.708L11.5 7.293l1.646-1.647a.5.5 0 0 1 .708 0z"/>
    </svg>
  )
}

function MinimizeIcon() {
  return (
    <svg viewBox="0 0 16 16" fill="currentColor">
      <path d="M4 8a.5.5 0 0 1 .5-.5h7a.5.5 0 0 1 0 1h-7A.5.5 0 0 1 4 8z"/>
    </svg>
  )
}

function MaximizeIcon() {
  return (
    <svg viewBox="0 0 16 16" fill="currentColor">
      <path d="M3 3v10h10V3H3zM2 3a1 1 0 0 1 1-1h10a1 1 0 0 1 1 1v10a1 1 0 0 1-1 1H3a1 1 0 0 1-1-1V3z"/>
    </svg>
  )
}

function DownloadIcon() {
  return (
    <svg viewBox="0 0 16 16" fill="currentColor">
      <path d="M.5 9.9a.5.5 0 0 1 .5.5v2.5a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1v-2.5a.5.5 0 0 1 1 0v2.5a2 2 0 0 1-2 2H2a2 2 0 0 1-2-2v-2.5a.5.5 0 0 1 .5-.5z"/>
      <path d="M7.646 11.854a.5.5 0 0 0 .708 0l3-3a.5.5 0 0 0-.708-.708L8.5 10.293V1.5a.5.5 0 0 0-1 0v8.793L5.354 8.146a.5.5 0 1 0-.708.708l3 3z"/>
    </svg>
  )
}

function MenuIcon() {
  return (
    <svg viewBox="0 0 16 16" fill="currentColor">
      <path d="M9.5 13a1.5 1.5 0 1 1-3 0 1.5 1.5 0 0 1 3 0zm0-5a1.5 1.5 0 1 1-3 0 1.5 1.5 0 0 1 3 0zm0-5a1.5 1.5 0 1 1-3 0 1.5 1.5 0 0 1 3 0z"/>
    </svg>
  )
}
