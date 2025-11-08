import type { IPCEventHandler } from '../types'

/**
 * Maya IPC communication composable
 *
 * This composable provides a bridge between the Vue frontend and Maya backend
 * using AuroraView's IPC system.
 *
 * AuroraView uses CustomEvent for IPC:
 * - Send to Python: window.dispatchEvent(new CustomEvent(eventName, { detail: data }))
 * - Receive from Python: window.addEventListener(eventName, (e) => { ... e.detail ... })
 */
export function useMayaIPC() {
  // Event handlers registry
  const eventHandlers = new Map<string, Set<IPCEventHandler>>()

  /**
   * Send a message to Maya using AuroraView's CustomEvent system
   */
  const sendToMaya = (event: string, data: Record<string, unknown>) => {
    console.log('[MayaIPC] Sending event:', event, data)

    // AuroraView uses CustomEvent for IPC
    const customEvent = new CustomEvent(event, {
      detail: data,
    })

    window.dispatchEvent(customEvent)
  }

  /**
   * Register an event handler for Maya events
   */
  const onMayaEvent = (event: string, handler: IPCEventHandler) => {
    if (!eventHandlers.has(event)) {
      eventHandlers.set(event, new Set())

      // Register global event listener for CustomEvent
      const eventListener = (e: Event) => {
        const customEvent = e as CustomEvent
        console.log('[MayaIPC] Received event:', event, customEvent.detail)

        const handlers = eventHandlers.get(event)
        if (handlers) {
          handlers.forEach((h) => h(customEvent.detail))
        }
      }

      window.addEventListener(event, eventListener)
    }

    eventHandlers.get(event)!.add(handler)
  }

  /**
   * Unregister an event handler
   */
  const offMayaEvent = (event: string, handler: IPCEventHandler) => {
    const handlers = eventHandlers.get(event)
    if (handlers) {
      handlers.delete(handler)
      if (handlers.size === 0) {
        eventHandlers.delete(event)
      }
    }
  }

  return {
    sendToMaya,
    onMayaEvent,
    offMayaEvent,
  }
}

