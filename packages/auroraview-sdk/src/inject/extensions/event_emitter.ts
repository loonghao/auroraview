/**
 * Chrome Event Emitter
 * 
 * Provides Chrome-compatible event handling for extension APIs.
 */

type Listener<T extends any[] = any[]> = (...args: T) => void;

export class EventEmitter<T extends any[] = any[]> {
  private _listeners: Listener<T>[] = [];
  private _onceListeners: Listener<T>[] = [];

  addListener(callback: Listener<T>): void {
    if (typeof callback === 'function') {
      this._listeners.push(callback);
    }
  }

  removeListener(callback: Listener<T>): void {
    const idx = this._listeners.indexOf(callback);
    if (idx >= 0) {
      this._listeners.splice(idx, 1);
    }
  }

  hasListener(callback: Listener<T>): boolean {
    return this._listeners.includes(callback);
  }

  hasListeners(): boolean {
    return this._listeners.length > 0;
  }

  addOnceListener(callback: Listener<T>): void {
    if (typeof callback === 'function') {
      this._onceListeners.push(callback);
    }
  }

  /** @internal */
  _dispatch(...args: T): void {
    // Call regular listeners
    for (const listener of this._listeners) {
      try {
        listener(...args);
      } catch (e) {
        console.error('[Chrome API] Event listener error:', e);
      }
    }
    // Call and remove once listeners
    const once = this._onceListeners.splice(0);
    for (const listener of once) {
      try {
        listener(...args);
      } catch (e) {
        console.error('[Chrome API] Once listener error:', e);
      }
    }
  }
}

/**
 * Create a web request event with filter support
 */
export function createWebRequestEvent(eventName: string, callNativeApi: Function) {
  interface ListenerEntry {
    callback: Function;
    filter?: any;
    extraInfoSpec?: string[];
  }

  const listeners: ListenerEntry[] = [];

  return {
    _listeners: listeners,
    addListener(callback: Function, filter?: any, extraInfoSpec?: string[]) {
      listeners.push({ callback, filter, extraInfoSpec });
      callNativeApi('webRequest', 'addListener', {
        event: eventName,
        filter,
        extraInfoSpec,
      });
    },
    removeListener(callback: Function) {
      const idx = listeners.findIndex((l) => l.callback === callback);
      if (idx >= 0) {
        listeners.splice(idx, 1);
        callNativeApi('webRequest', 'removeListener', { event: eventName });
      }
    },
    hasListener(callback: Function) {
      return listeners.some((l) => l.callback === callback);
    },
    hasListeners() {
      return listeners.length > 0;
    },
  };
}
