/**
 * AuroraView Event Bridge - Core JavaScript API
 *
 * This script provides the core event bridge between JavaScript and Python.
 * It is injected at WebView initialization and persists across navigations.
 *
 * @module event_bridge
 */

(function () {
  'use strict';

  console.log('[AuroraView] Initializing event bridge...');

  // Check if already initialized (prevent double initialization)
  if (window.auroraview && window.auroraview._ready) {
    console.log('[AuroraView] Event bridge already initialized, skipping');
    return;
  }

  // Preserve any pending calls from stub if it exists
  const pendingFromStub =
    window.auroraview && window.auroraview._pendingCalls
      ? window.auroraview._pendingCalls.slice()
      : [];

  // Ready callbacks for whenReady() API
  let readyCallbacks: Array<(av: typeof window.auroraview) => void> = [];

  // Event handlers registry for Python -> JS communication
  const eventHandlers = new Map<string, Set<(data: unknown) => void>>();

  // Pending call registry for auroraview.call Promise resolution
  let auroraviewCallIdCounter = 0;
  const auroraviewPendingCalls = new Map<
    string,
    { resolve: (value: unknown) => void; reject: (error: Error) => void }
  >();

  /**
   * Generate unique call ID for Promise tracking
   */
  function auroraviewGenerateCallId(): string {
    auroraviewCallIdCounter += 1;
    return 'av_call_' + Date.now() + '_' + auroraviewCallIdCounter;
  }

  /**
   * Handle call_result events coming back from Python (Python -> JS)
   */
  function handleCallResult(detail: {
    id?: string;
    ok?: boolean;
    result?: unknown;
    error?: { name?: string; message?: string; code?: string | number; data?: unknown };
  }): void {
    try {
      const id = detail && detail.id;

      if (!id) {
        console.warn('[AuroraView] call_result without id:', detail);
        return;
      }

      const pending = auroraviewPendingCalls.get(id);
      if (!pending) {
        console.warn('[AuroraView] No pending call for id:', id);
        return;
      }

      auroraviewPendingCalls.delete(id);

      if (detail.ok) {
        pending.resolve(detail.result);
      } else {
        const errInfo = detail.error || {};
        const error = new Error(errInfo.message || 'AuroraView call failed') as Error & {
          code?: string | number;
          data?: unknown;
        };
        if (errInfo.name) error.name = errInfo.name;
        if (errInfo.code !== undefined) error.code = errInfo.code;
        if (errInfo.data !== undefined) error.data = errInfo.data;
        pending.reject(error);
      }
    } catch (e) {
      console.error('[AuroraView] Error handling call_result:', e);
    }
  }

  /**
   * Primary AuroraView bridge API
   */
  window.auroraview = {
    /**
     * High-level call API (JS -> Python, Promise-based)
     */
    call: function <T = unknown>(method: string, params?: unknown): Promise<T> {
      console.log('[AuroraView] Calling Python method via auroraview.call:', method, params);
      return new Promise(function (resolve, reject) {
        const id = auroraviewGenerateCallId();
        auroraviewPendingCalls.set(id, {
          resolve: resolve as (value: unknown) => void,
          reject: reject,
        });

        try {
          const payload: { type: string; id: string; method: string; params?: unknown } = {
            type: 'call',
            id: id,
            method: method,
          };
          if (typeof params !== 'undefined') {
            payload.params = params;
          }
          window.ipc!.postMessage(JSON.stringify(payload));
        } catch (e) {
          console.error('[AuroraView] Failed to send call via IPC:', e);
          auroraviewPendingCalls.delete(id);
          reject(e);
        }
      });
    },

    /**
     * Send event to Python (JS -> Python, fire-and-forget)
     */
    send_event: function (event: string, detail?: unknown): void {
      try {
        const payload = {
          type: 'event',
          event: event,
          detail: detail || {},
        };
        window.ipc!.postMessage(JSON.stringify(payload));
        console.log('[AuroraView] Event sent:', event, detail);
      } catch (e) {
        console.error('[AuroraView] Failed to send event:', e);
      }
    },

    /**
     * Register event handler (Python -> JS)
     * @returns Unsubscribe function
     */
    on: function (event: string, handler: (data: unknown) => void): () => void {
      if (typeof handler !== 'function') {
        console.error('[AuroraView] Handler must be a function');
        return () => {};
      }
      if (!eventHandlers.has(event)) {
        eventHandlers.set(event, new Set());
      }
      const handlers = eventHandlers.get(event)!;
      handlers.add(handler);
      console.log('[AuroraView] Registered handler for event:', event);

      // Return unsubscribe function
      return () => {
        handlers.delete(handler);
        if (handlers.size === 0) {
          eventHandlers.delete(event);
        }
        console.log('[AuroraView] Unregistered handler for event:', event);
      };
    },

    /**
     * Remove event handler
     */
    off: function (event: string, handler?: (data: unknown) => void): void {
      if (handler) {
        eventHandlers.get(event)?.delete(handler);
      } else {
        eventHandlers.delete(event);
      }
    },

    /**
     * Trigger event handlers (called by Python)
     */
    trigger: function (event: string, detail?: unknown): void {
      // Special handling for internal call_result events
      if (event === '__auroraview_call_result') {
        handleCallResult(detail as Parameters<typeof handleCallResult>[0]);
        return;
      }

      const handlers = eventHandlers.get(event);
      if (!handlers || handlers.size === 0) {
        console.warn('[AuroraView] No handlers for event:', event);
        return;
      }
      handlers.forEach(function (handler) {
        try {
          handler(detail);
        } catch (e) {
          console.error('[AuroraView] Error in event handler:', e);
        }
      });
    },

    /**
     * Namespace for API methods (populated by Python)
     */
    api: {},

    /**
     * Invoke a plugin command (JS -> Python, Promise-based)
     */
    invoke: function <T = unknown>(cmd: string, args?: Record<string, unknown>): Promise<T> {
      console.log('[AuroraView] Invoking plugin command:', cmd, args);
      return new Promise(function (resolve, reject) {
        const id = auroraviewGenerateCallId();
        auroraviewPendingCalls.set(id, {
          resolve: resolve as (value: unknown) => void,
          reject: reject,
        });

        try {
          const payload = {
            type: 'invoke',
            id: id,
            cmd: cmd,
            args: args || {},
          };
          window.ipc!.postMessage(JSON.stringify(payload));
        } catch (e) {
          console.error('[AuroraView] Failed to send invoke via IPC:', e);
          auroraviewPendingCalls.delete(id);
          reject(e);
        }
      });
    },

    /**
     * Ready state flag
     */
    _ready: false,

    /**
     * Pending calls queue
     */
    _pendingCalls: [],

    /**
     * Wait for event bridge to be ready
     */
    whenReady: function (): Promise<typeof window.auroraview> {
      return new Promise(function (resolve) {
        if (window.auroraview!._ready) {
          resolve(window.auroraview);
        } else {
          readyCallbacks.push(resolve);
        }
      });
    },

    /**
     * Check if bridge is ready (synchronous)
     */
    isReady: function (): boolean {
      return window.auroraview!._ready === true;
    },

    /**
     * Registry of all bound methods
     */
    _boundMethods: {},

    /**
     * Check if a method is already registered
     */
    isMethodBound: function (fullMethodName: string): boolean {
      return !!window.auroraview!._boundMethods[fullMethodName];
    },

    /**
     * Get list of all bound method names
     */
    getBoundMethods: function (): string[] {
      return Object.keys(window.auroraview!._boundMethods);
    },

    /**
     * Register API methods dynamically
     */
    _registerApiMethods: function (
      namespace: string,
      methods: string[],
      options?: { allowRebind?: boolean }
    ): void {
      if (!namespace || !methods || !Array.isArray(methods)) {
        console.error('[AuroraView] Invalid arguments for _registerApiMethods');
        return;
      }

      const opts = options || {};
      const allowRebind = opts.allowRebind !== false;

      // Create namespace if it doesn't exist
      if (!(window.auroraview as Record<string, unknown>)[namespace]) {
        (window.auroraview as Record<string, unknown>)[namespace] = {};
      }

      let registeredCount = 0;
      let skippedCount = 0;

      for (let i = 0; i < methods.length; i++) {
        const methodName = methods[i];
        const fullMethodName = namespace + '.' + methodName;

        if (window.auroraview!._boundMethods[fullMethodName]) {
          if (!allowRebind) {
            console.debug('[AuroraView] Skipping already bound method:', fullMethodName);
            skippedCount++;
            continue;
          }
          console.debug('[AuroraView] Rebinding method:', fullMethodName);
        }

        // Create closure to capture method name
        ((window.auroraview as Record<string, Record<string, unknown>>)[namespace] as Record<
          string,
          (params?: unknown) => Promise<unknown>
        >)[methodName] = (function (fullName: string) {
          return function (params?: unknown): Promise<unknown> {
            return window.auroraview!.call(fullName, params);
          };
        })(fullMethodName);

        window.auroraview!._boundMethods[fullMethodName] = true;
        registeredCount++;
      }

      if (registeredCount > 0) {
        console.log(
          '[AuroraView] Registered ' + registeredCount + ' methods in window.auroraview.' + namespace
        );
      }
      if (skippedCount > 0) {
        console.log(
          '[AuroraView] Skipped ' + skippedCount + ' already-bound methods in window.auroraview.' + namespace
        );
      }
    },
  };

  // Mark bridge as ready
  window.auroraview._ready = true;

  // Process any pending calls from stub
  if (pendingFromStub.length > 0) {
    console.log('[AuroraView] Processing ' + pendingFromStub.length + ' pending calls from stub');
    pendingFromStub.forEach(function (pending: {
      type?: string;
      method?: string;
      params?: unknown;
      event?: string;
      detail?: unknown;
      handler?: (data: unknown) => void;
      namespace?: string;
      methods?: string[];
      resolve?: (value: unknown) => void;
      reject?: (error: Error) => void;
    }) {
      try {
        if (pending.type === 'event' && pending.event) {
          window.auroraview!.send_event(pending.event, pending.detail);
        } else if (pending.type === 'handler' && pending.event && pending.handler) {
          window.auroraview!.on(pending.event, pending.handler);
        } else if (pending.type === 'register' && pending.namespace && pending.methods) {
          window.auroraview!._registerApiMethods!(pending.namespace, pending.methods);
        } else if (pending.method && pending.resolve && pending.reject) {
          window.auroraview!.call(pending.method, pending.params)
            .then(pending.resolve)
            .catch(pending.reject);
        }
      } catch (e) {
        if (pending.reject) {
          pending.reject(e as Error);
        }
      }
    });
  }

  // Notify all whenReady() waiters
  if (readyCallbacks.length > 0) {
    console.log('[AuroraView] Notifying ' + readyCallbacks.length + ' ready callbacks');
    readyCallbacks.forEach(function (callback) {
      try {
        callback(window.auroraview);
      } catch (e) {
        console.error('[AuroraView] Error in ready callback:', e);
      }
    });
    readyCallbacks = [];
  }

  console.log('[AuroraView] ✓ Event bridge initialized');
  console.log('[AuroraView] ✓ API: window.auroraview.call() / .send_event() / .on() / .whenReady()');

  // Emit __auroraview_ready event to Python backend
  try {
    window.auroraview.send_event('__auroraview_ready', {
      timestamp: Date.now(),
      url: window.location.href,
    });
    console.log('[AuroraView] ✓ Sent __auroraview_ready event to backend');
  } catch (e) {
    console.warn('[AuroraView] Failed to send __auroraview_ready event:', e);
  }
})();
