import React, { useState, useEffect, useMemo } from 'react'
import { ErrorIcon, LogoContainer } from '@/components/Logo'
import { getUrlParams, copyToClipboard, onEvent } from '@/utils/ipc'

interface StackFrame {
  file: string
  line?: number
  column?: number
  function?: string
  code?: string
  isInternal?: boolean
}

interface ErrorInfo {
  code: string
  title: string
  message: string
  details?: string
  url?: string
  stack?: StackFrame[]
  source?: 'python' | 'rust' | 'javascript' | 'unknown'
  timestamp?: number
  console?: ConsoleMessage[]
}

interface ConsoleMessage {
  level: 'log' | 'warn' | 'error' | 'info' | 'debug'
  message: string
  timestamp: number
}

type TabType = 'overview' | 'stack' | 'console' | 'diagnostics'

/**
 * Next.js-style Error Overlay
 * 
 * Provides detailed error diagnostics for developers including:
 * - Error overview with message and code
 * - Stack trace with source code preview
 * - Console output
 * - System diagnostics
 */
export function ErrorOverlay() {
  const [error, setError] = useState<ErrorInfo>(() => getErrorFromUrl())
  const [activeTab, setActiveTab] = useState<TabType>('overview')
  const [copied, setCopied] = useState(false)
  const [expanded, setExpanded] = useState<Record<number, boolean>>({})

  // Listen for error updates from backend
  useEffect(() => {
    const unsub = onEvent('error_update', (data) => {
      setError(prev => ({ ...prev, ...(data as Partial<ErrorInfo>) }))
    })
    return unsub
  }, [])

  // Parse error from URL params
  function getErrorFromUrl(): ErrorInfo {
    const params = getUrlParams()
    let stack: StackFrame[] | undefined
    let console_output: ConsoleMessage[] | undefined

    try {
      const stackParam = params.get('stack')
      if (stackParam) {
        stack = JSON.parse(decodeURIComponent(stackParam))
      }
    } catch { /* ignore */ }

    try {
      const consoleParam = params.get('console')
      if (consoleParam) {
        console_output = JSON.parse(decodeURIComponent(consoleParam))
      }
    } catch { /* ignore */ }

    return {
      code: params.get('code') || '500',
      title: params.get('title') || 'Internal Server Error',
      message: params.get('message') || 'An unexpected error occurred while processing your request.',
      details: params.get('details') || undefined,
      url: params.get('url') || undefined,
      stack,
      source: (params.get('source') as ErrorInfo['source']) || 'unknown',
      timestamp: params.get('timestamp') ? parseInt(params.get('timestamp')!) : Date.now(),
      console: console_output,
    }
  }

  const handleCopy = async () => {
    const text = formatErrorForCopy(error)
    const success = await copyToClipboard(text)
    if (success) {
      setCopied(true)
      setTimeout(() => setCopied(false), 2000)
    }
  }

  const handleRetry = () => {
    if (error.url) {
      window.location.href = error.url
    } else {
      window.location.reload()
    }
  }

  const sourceLabel = useMemo(() => {
    switch (error.source) {
      case 'python': return 'Python Backend'
      case 'rust': return 'Rust Runtime'
      case 'javascript': return 'JavaScript'
      default: return 'Unknown'
    }
  }, [error.source])

  const tabs: { id: TabType; label: string; count?: number }[] = [
    { id: 'overview', label: 'Overview' },
    { id: 'stack', label: 'Stack Trace', count: error.stack?.length },
    { id: 'console', label: 'Console', count: error.console?.length },
    { id: 'diagnostics', label: 'Diagnostics' },
  ]

  return (
    <div className="aurora-bg error-bg min-h-screen flex flex-col">
      {/* Header */}
      <header className="flex items-center justify-between px-6 py-4 border-b border-white/10">
        <div className="flex items-center gap-3">
          <ErrorIcon size={24} className="animate-float" />
          <span className="text-white/60 text-sm font-medium">AuroraView Error</span>
        </div>
        <div className="flex items-center gap-2">
          <span className="text-xs text-white/40 bg-white/5 px-2 py-1 rounded">
            {sourceLabel}
          </span>
          {error.timestamp && (
            <span className="text-xs text-white/40">
              {new Date(error.timestamp).toLocaleTimeString()}
            </span>
          )}
        </div>
      </header>

      {/* Main content */}
      <main className="flex-1 flex flex-col overflow-hidden">
        {/* Error summary */}
        <div className="px-6 py-8 text-center border-b border-white/10">
          <div className="error-text text-6xl font-bold mb-2">{error.code}</div>
          <h1 className="text-xl font-semibold text-white mb-2">{error.title}</h1>
          <p className="text-white/70 max-w-xl mx-auto">{error.message}</p>
        </div>

        {/* Tabs */}
        <div className="flex border-b border-white/10 px-6">
          {tabs.map(tab => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={`px-4 py-3 text-sm font-medium transition-colors relative ${
                activeTab === tab.id
                  ? 'text-aurora-cyan'
                  : 'text-white/50 hover:text-white/80'
              }`}
            >
              {tab.label}
              {tab.count !== undefined && tab.count > 0 && (
                <span className="ml-1.5 text-xs bg-white/10 px-1.5 py-0.5 rounded-full">
                  {tab.count}
                </span>
              )}
              {activeTab === tab.id && (
                <div className="absolute bottom-0 left-0 right-0 h-0.5 bg-aurora-cyan" />
              )}
            </button>
          ))}
        </div>

        {/* Tab content */}
        <div className="flex-1 overflow-auto p-6">
          {activeTab === 'overview' && (
            <OverviewTab error={error} />
          )}
          {activeTab === 'stack' && (
            <StackTraceTab
              stack={error.stack || []}
              expanded={expanded}
              onToggle={(i) => setExpanded(prev => ({ ...prev, [i]: !prev[i] }))}
            />
          )}
          {activeTab === 'console' && (
            <ConsoleTab messages={error.console || []} />
          )}
          {activeTab === 'diagnostics' && (
            <DiagnosticsTab error={error} />
          )}
        </div>

        {/* Actions */}
        <div className="flex items-center justify-center gap-4 px-6 py-4 border-t border-white/10">
          <button
            onClick={handleCopy}
            className="px-4 py-2 text-sm font-medium text-white/70 bg-white/5 hover:bg-white/10 rounded-lg transition-colors flex items-center gap-2"
          >
            <CopyIcon />
            {copied ? 'Copied!' : 'Copy Error'}
          </button>
          <button
            onClick={handleRetry}
            className="aurora-btn error-btn px-6 py-2 text-sm"
          >
            Try Again
          </button>
        </div>
      </main>

      {/* Footer */}
      <footer className="text-center py-3 text-white/30 text-xs tracking-widest">
        AURORAVIEW
      </footer>
    </div>
  )
}

function OverviewTab({ error }: { error: ErrorInfo }) {
  if (!error.details && !error.url) {
    return (
      <div className="text-center text-white/40 py-12">
        No additional details available
      </div>
    )
  }

  return (
    <div className="space-y-4">
      {error.url && (
        <div className="bg-white/5 rounded-lg p-4">
          <div className="text-xs text-white/40 mb-1">URL</div>
          <div className="text-sm text-white/80 font-mono break-all">{error.url}</div>
        </div>
      )}
      {error.details && (
        <div className="bg-white/5 rounded-lg p-4">
          <div className="text-xs text-white/40 mb-1">Details</div>
          <pre className="text-sm text-white/80 font-mono whitespace-pre-wrap break-words">
            {error.details}
          </pre>
        </div>
      )}
    </div>
  )
}

function StackTraceTab({
  stack,
  expanded,
  onToggle,
}: {
  stack: StackFrame[]
  expanded: Record<number, boolean>
  onToggle: (i: number) => void
}) {
  if (stack.length === 0) {
    return (
      <div className="text-center text-white/40 py-12">
        No stack trace available
      </div>
    )
  }

  return (
    <div className="space-y-2">
      {stack.map((frame, i) => (
        <div
          key={i}
          className={`bg-white/5 rounded-lg overflow-hidden ${
            frame.isInternal ? 'opacity-50' : ''
          }`}
        >
          <button
            onClick={() => onToggle(i)}
            className="w-full px-4 py-3 flex items-center justify-between text-left hover:bg-white/5 transition-colors"
          >
            <div className="flex items-center gap-3 min-w-0">
              <ChevronIcon expanded={expanded[i]} />
              <span className="text-sm text-aurora-cyan font-mono truncate">
                {frame.function || '<anonymous>'}
              </span>
            </div>
            <span className="text-xs text-white/40 font-mono flex-shrink-0 ml-4">
              {frame.file}
              {frame.line && `:${frame.line}`}
              {frame.column && `:${frame.column}`}
            </span>
          </button>
          {expanded[i] && frame.code && (
            <div className="px-4 pb-3">
              <pre className="bg-black/30 rounded p-3 text-xs text-white/70 font-mono overflow-x-auto">
                {frame.code}
              </pre>
            </div>
          )}
        </div>
      ))}
    </div>
  )
}

function ConsoleTab({ messages }: { messages: ConsoleMessage[] }) {
  if (messages.length === 0) {
    return (
      <div className="text-center text-white/40 py-12">
        No console output captured
      </div>
    )
  }

  const levelColors: Record<ConsoleMessage['level'], string> = {
    error: 'text-red-400 bg-red-500/10',
    warn: 'text-yellow-400 bg-yellow-500/10',
    info: 'text-blue-400 bg-blue-500/10',
    log: 'text-white/70 bg-white/5',
    debug: 'text-white/50 bg-white/5',
  }

  return (
    <div className="space-y-1 font-mono text-sm">
      {messages.map((msg, i) => (
        <div key={i} className={`px-3 py-2 rounded ${levelColors[msg.level]}`}>
          <span className="text-xs opacity-50 mr-2">
            {new Date(msg.timestamp).toLocaleTimeString()}
          </span>
          <span className="uppercase text-xs font-medium mr-2 opacity-70">
            [{msg.level}]
          </span>
          <span className="break-words">{msg.message}</span>
        </div>
      ))}
    </div>
  )
}

function DiagnosticsTab({ error }: { error: ErrorInfo }) {
  const diagnostics = useMemo(() => ({
    'Error Code': error.code,
    'Error Type': error.title,
    'Source': error.source || 'unknown',
    'Timestamp': error.timestamp ? new Date(error.timestamp).toISOString() : 'N/A',
    'URL': error.url || 'N/A',
    'Stack Frames': error.stack?.length || 0,
    'Console Messages': error.console?.length || 0,
    'User Agent': navigator.userAgent,
    'Platform': navigator.platform,
    'Window Size': `${window.innerWidth}x${window.innerHeight}`,
  }), [error])

  return (
    <div className="bg-white/5 rounded-lg overflow-hidden">
      {Object.entries(diagnostics).map(([key, value]) => (
        <div key={key} className="flex border-b border-white/5 last:border-b-0">
          <div className="w-40 px-4 py-3 text-xs text-white/40 bg-white/5">{key}</div>
          <div className="flex-1 px-4 py-3 text-sm text-white/80 font-mono break-all">
            {String(value)}
          </div>
        </div>
      ))}
    </div>
  )
}

function formatErrorForCopy(error: ErrorInfo): string {
  const lines = [
    `Error ${error.code}: ${error.title}`,
    error.message,
    '',
  ]

  if (error.url) {
    lines.push(`URL: ${error.url}`)
  }

  if (error.details) {
    lines.push(`Details: ${error.details}`)
  }

  if (error.stack?.length) {
    lines.push('', 'Stack Trace:')
    error.stack.forEach(frame => {
      const loc = [frame.file, frame.line, frame.column].filter(Boolean).join(':')
      lines.push(`  at ${frame.function || '<anonymous>'} (${loc})`)
    })
  }

  if (error.console?.length) {
    lines.push('', 'Console Output:')
    error.console.forEach(msg => {
      lines.push(`  [${msg.level.toUpperCase()}] ${msg.message}`)
    })
  }

  return lines.join('\n')
}

function CopyIcon() {
  return (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
      <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
      <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
    </svg>
  )
}

function ChevronIcon({ expanded }: { expanded?: boolean }) {
  return (
    <svg
      width="16"
      height="16"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      className={`transition-transform ${expanded ? 'rotate-90' : ''}`}
    >
      <polyline points="9 18 15 12 9 6" />
    </svg>
  )
}
