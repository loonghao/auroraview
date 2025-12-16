/**
 * AuroraView SDK Event System
 *
 * Provides a type-safe event emitter with proper unsubscribe support.
 */

import type { EventHandler, Unsubscribe } from './types';

/**
 * Event emitter with unsubscribe support
 */
export class EventEmitter {
  private handlers = new Map<string, Set<EventHandler>>();

  /**
   * Subscribe to an event
   * @param event - Event name
   * @param handler - Event handler function
   * @returns Unsubscribe function
   */
  on<T = unknown>(event: string, handler: EventHandler<T>): Unsubscribe {
    if (!this.handlers.has(event)) {
      this.handlers.set(event, new Set());
    }
    const handlers = this.handlers.get(event)!;
    handlers.add(handler as EventHandler);

    // Return unsubscribe function
    return () => {
      handlers.delete(handler as EventHandler);
      if (handlers.size === 0) {
        this.handlers.delete(event);
      }
    };
  }

  /**
   * Subscribe to an event once
   * @param event - Event name
   * @param handler - Event handler function
   * @returns Unsubscribe function
   */
  once<T = unknown>(event: string, handler: EventHandler<T>): Unsubscribe {
    const wrapper: EventHandler<T> = (data) => {
      unsubscribe();
      handler(data);
    };
    const unsubscribe = this.on(event, wrapper);
    return unsubscribe;
  }

  /**
   * Emit an event to all handlers
   * @param event - Event name
   * @param data - Event data
   */
  emit<T = unknown>(event: string, data: T): void {
    const handlers = this.handlers.get(event);
    if (!handlers) return;

    handlers.forEach((handler) => {
      try {
        handler(data);
      } catch (e) {
        console.error(`[AuroraView] Error in event handler for "${event}":`, e);
      }
    });
  }

  /**
   * Remove event handler(s)
   * @param event - Event name
   * @param handler - Optional specific handler to remove
   */
  off(event: string, handler?: EventHandler): void {
    if (handler) {
      this.handlers.get(event)?.delete(handler);
    } else {
      this.handlers.delete(event);
    }
  }

  /**
   * Check if event has handlers
   * @param event - Event name
   */
  hasHandlers(event: string): boolean {
    const handlers = this.handlers.get(event);
    return handlers !== undefined && handlers.size > 0;
  }

  /**
   * Get handler count for an event
   * @param event - Event name
   */
  handlerCount(event: string): number {
    return this.handlers.get(event)?.size ?? 0;
  }

  /**
   * Remove all handlers
   */
  clear(): void {
    this.handlers.clear();
  }
}

/** Singleton event emitter instance */
let globalEmitter: EventEmitter | null = null;

/**
 * Get the global event emitter instance
 */
export function getGlobalEmitter(): EventEmitter {
  if (!globalEmitter) {
    globalEmitter = new EventEmitter();
  }
  return globalEmitter;
}
