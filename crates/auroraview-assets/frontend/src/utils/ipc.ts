/**
 * AuroraView IPC utilities
 * 
 * These utilities help communicate with the Rust backend
 */

declare global {
  interface Window {
    auroraview?: {
      call: (method: string, params?: unknown) => Promise<unknown>
      on: (event: string, handler: (data: unknown) => void) => () => void
      send_event: (event: string, data?: unknown) => void
      whenReady?: () => Promise<void>
      _ready?: boolean
    }
    ipc?: {
      postMessage: (message: string) => void
    }
    __loadingReadySent?: boolean
    __backendReady?: boolean
  }
}

/**
 * Send an event to the Rust backend
 */
export function sendEvent(event: string, data: unknown = {}) {
  // Method 1: Use auroraview.send_event if available
  if (window.auroraview?.send_event) {
    try {
      window.auroraview.send_event(event, data)
      console.log(`[IPC] Sent via auroraview.send_event: ${event}`)
      return true
    } catch (e) {
      console.warn(`[IPC] Failed to send via auroraview.send_event:`, e)
    }
  }

  // Method 2: Direct IPC postMessage (fallback)
  if (window.ipc?.postMessage) {
    try {
      const payload = JSON.stringify({
        type: 'event',
        event,
        detail: data,
      })
      window.ipc.postMessage(payload)
      console.log(`[IPC] Sent via window.ipc.postMessage: ${event}`)
      return true
    } catch (e) {
      console.warn(`[IPC] Failed to send via window.ipc.postMessage:`, e)
    }
  }

  console.error(`[IPC] Could not send event: ${event} - no IPC available`)
  return false
}

/**
 * Call a method on the Rust backend
 */
export async function callMethod<T = unknown>(method: string, params?: unknown): Promise<T | undefined> {
  if (window.auroraview?.call) {
    try {
      return await window.auroraview.call(method, params) as T
    } catch (e) {
      console.error(`[IPC] Call failed: ${method}`, e)
      throw e
    }
  }
  console.error(`[IPC] auroraview.call not available`)
  return undefined
}

/**
 * Subscribe to an event from the Rust backend
 */
export function onEvent(event: string, handler: (data: unknown) => void): () => void {
  if (window.auroraview?.on) {
    return window.auroraview.on(event, handler)
  }

  // Fallback: use window custom events
  const wrappedHandler = (e: CustomEvent) => handler(e.detail)
  window.addEventListener(`auroraview:${event}` as keyof WindowEventMap, wrappedHandler as EventListener)
  return () => window.removeEventListener(`auroraview:${event}` as keyof WindowEventMap, wrappedHandler as EventListener)
}

/**
 * Wait for AuroraView bridge to be ready
 */
export function whenReady(): Promise<void> {
  return new Promise((resolve) => {
    if (window.auroraview?._ready) {
      resolve()
      return
    }

    if (window.auroraview?.whenReady) {
      window.auroraview.whenReady().then(resolve)
      return
    }

    window.addEventListener('auroraviewready', () => resolve(), { once: true })

    // Timeout fallback
    setTimeout(resolve, 1000)
  })
}

/**
 * Copy text to clipboard
 */
export async function copyToClipboard(text: string): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(text)
    return true
  } catch {
    // Fallback for older browsers
    const textarea = document.createElement('textarea')
    textarea.value = text
    textarea.style.position = 'fixed'
    textarea.style.opacity = '0'
    document.body.appendChild(textarea)
    textarea.select()
    try {
      document.execCommand('copy')
      return true
    } catch {
      return false
    } finally {
      document.body.removeChild(textarea)
    }
  }
}

/**
 * Parse URL search params
 */
export function getUrlParams(): URLSearchParams {
  return new URLSearchParams(window.location.search)
}
