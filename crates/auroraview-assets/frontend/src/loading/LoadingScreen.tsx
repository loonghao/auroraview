import React, { useCallback, useEffect, useState, useRef } from 'react'
import { AuroraLogo, LogoContainer } from '@/components/Logo'
import { sendEvent, whenReady, onEvent } from '@/utils/ipc'

interface Step {
  id: string
  text: string
  status: 'pending' | 'active' | 'completed'
}

interface BackendError {
  message: string
  source: string
  timestamp: number
}

interface LoadingState {
  progress: number // -1 for indeterminate
  text: string
  steps: Step[]
  errors: BackendError[]
}

/**
 * Loading Screen Component
 * 
 * Displays an Aurora-themed loading animation with progress indication.
 * Communicates with the Rust backend via IPC.
 */
export function LoadingScreen() {
  const [state, setState] = useState<LoadingState>({
    progress: -1,
    text: 'Preparing your experience',
    steps: [],
    errors: [],
  })
  const [isReady, setIsReady] = useState(false)
  const startTimeRef = useRef(Date.now())
  const readySentRef = useRef(false)

  // Expose global API for Rust to call
  useEffect(() => {
    const api = {
      setProgress: (percent: number) => {
        setState(s => ({ ...s, progress: percent }))
      },
      setText: (text: string) => {
        setState(s => ({ ...s, text }))
      },
      setStep: (id: string, text: string, status: Step['status']) => {
        setState(s => {
          const steps = [...s.steps]
          const existing = steps.findIndex(step => step.id === id)
          if (existing >= 0) {
            steps[existing] = { id, text, status }
          } else {
            steps.push({ id, text, status })
          }
          return { ...s, steps }
        })
      },
      clearSteps: () => {
        setState(s => ({ ...s, steps: [] }))
      },
      update: (options: Partial<LoadingState>) => {
        setState(s => ({ ...s, ...options }))
      },
      addError: (message: string, source = 'unknown') => {
        setState(s => {
          const errors = [...s.errors, { message, source, timestamp: Date.now() }]
          // Keep only last 50 errors
          return { ...s, errors: errors.slice(-50) }
        })
      },
      clearErrors: () => {
        setState(s => ({ ...s, errors: [] }))
      },
      getState: () => state,
      getDiagnostics: () => ({
        elapsed_ms: Date.now() - startTimeRef.current,
        backend_ready: isReady,
        auroraview_available: !!window.auroraview,
        loading_state: state,
      }),
      isReady: () => isReady,
      getElapsedTime: () => Date.now() - startTimeRef.current,
    }

    // @ts-expect-error - expose to window
    window.auroraLoading = api
    // @ts-expect-error - alias
    window.AuroraLoading = api

    return () => {
      // @ts-expect-error - cleanup
      delete window.auroraLoading
      // @ts-expect-error - cleanup
      delete window.AuroraLoading
    }
  }, [state, isReady])

  // Send loading_screen_ready event
  const sendLoadingReady = useCallback(() => {
    if (readySentRef.current) return
    readySentRef.current = true
    sendEvent('loading_screen_ready', {})
  }, [])

  // Initialize and listen for events
  useEffect(() => {
    whenReady().then(sendLoadingReady)

    // Timeout fallback
    const timeout = setTimeout(sendLoadingReady, 1000)

    // Listen for loading updates
    const unsubUpdate = onEvent('loading_update', (data) => {
      setState(s => ({ ...s, ...(data as Partial<LoadingState>) }))
    })

    // Listen for backend_ready
    const unsubReady = onEvent('backend_ready', () => {
      setIsReady(true)
      setState(s => ({ ...s, progress: 100, text: 'Ready! Loading application...' }))
      
      // Navigate after short delay
      setTimeout(() => {
        sendEvent('navigate_to_app', {})
      }, 300)
    })

    return () => {
      clearTimeout(timeout)
      unsubUpdate()
      unsubReady()
    }
  }, [sendLoadingReady])

  // Diagnostic logging
  useEffect(() => {
    const interval = setInterval(() => {
      if (!isReady) {
        console.log('[Loading:Diagnostic]', {
          elapsed_ms: Date.now() - startTimeRef.current,
          backend_ready: isReady,
        })
      }
    }, 5000)

    // Timeout warning
    const timeout = setTimeout(() => {
      if (!isReady) {
        console.error('[Loading:Timeout] Backend not ready after 30s')
        setState(s => ({ ...s, text: 'Loading timeout - Backend not responding', progress: -1 }))
      }
    }, 30000)

    return () => {
      clearInterval(interval)
      clearTimeout(timeout)
    }
  }, [isReady])

  const { progress, text, steps, errors } = state
  const isIndeterminate = progress < 0

  return (
    <div className="aurora-bg min-h-screen flex items-center justify-center">
      <div className="text-center text-white z-10 relative">
        {/* Logo with rotating ring */}
        <LogoContainer className="mx-auto mb-8">
          <AuroraLogo className="animate-float drop-shadow-[0_0_20px_rgba(98,216,243,0.5)]" size={80} />
        </LogoContainer>

        {/* Title */}
        <h1 className="text-3xl font-bold mb-2 tracking-widest aurora-text uppercase animate-shimmer">
          AuroraView
        </h1>

        {/* Status text */}
        <p className="text-sm text-white/80 font-medium tracking-wide min-h-[20px] transition-opacity duration-300">
          {text}
        </p>

        {/* Progress bar */}
        <div className="w-[280px] h-1 bg-white/10 rounded-full mt-6 mx-auto overflow-hidden relative">
          <div
            className={`h-full bg-gradient-to-r from-aurora-cyan via-aurora-purple to-aurora-pink rounded-full transition-all duration-300 ${
              isIndeterminate ? 'w-[30%] progress-indeterminate' : ''
            }`}
            style={!isIndeterminate ? { width: `${Math.min(100, Math.max(0, progress))}%` } : undefined}
          />
        </div>

        {/* Progress percentage */}
        {!isIndeterminate && (
          <p className="text-xs text-white/60 mt-2">{Math.round(progress)}%</p>
        )}

        {/* Steps */}
        {steps.length > 0 && (
          <div className="mt-4 flex flex-col items-center gap-1">
            {steps.map(step => (
              <div
                key={step.id}
                className={`text-xs flex items-center gap-1.5 transition-colors ${
                  step.status === 'completed' ? 'text-aurora-cyan' :
                  step.status === 'active' ? 'text-white/90' :
                  'text-white/50'
                }`}
              >
                <span className="w-3.5 h-3.5 flex items-center justify-center">
                  {step.status === 'completed' ? (
                    <span className="text-aurora-cyan">✓</span>
                  ) : step.status === 'active' ? (
                    <span className="w-3 h-3 border-2 border-white/20 border-t-aurora-cyan rounded-full animate-spin" />
                  ) : (
                    <span className="opacity-30">○</span>
                  )}
                </span>
                <span>{step.text}</span>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Error panel */}
      {errors.length > 0 && (
        <div className="fixed bottom-5 left-5 right-5 max-h-[200px] bg-red-500/15 border border-red-500/40 rounded-lg p-3 overflow-y-auto font-mono text-xs text-red-300 z-50 backdrop-blur-sm">
          <div className="flex justify-between items-center mb-2 pb-2 border-b border-red-500/30">
            <span className="font-semibold text-red-400 text-sm">Backend Errors</span>
            <span className="bg-red-500/30 px-2 py-0.5 rounded-full text-[11px]">{errors.length}</span>
          </div>
          {errors.map((err, i) => (
            <div key={i} className="py-1 border-b border-red-500/15 last:border-b-0 break-words">
              <span className="text-yellow-400 text-[10px] mr-1.5">[{err.source}]</span>
              {err.message}
            </div>
          ))}
        </div>
      )}
    </div>
  )
}
