/**
 * AuroraView Bridge Stub - Early Initialization Placeholder
 *
 * This stub creates a minimal window.auroraview namespace before the full
 * event bridge is loaded. It queues any calls made during this time and
 * replays them once the real bridge is initialized.
 *
 * @module bridge_stub
 */

(function () {
  'use strict';

  // Skip if bridge is already fully initialized
  if (window.auroraview && window.auroraview._ready) {
    console.log('[AuroraView Stub] Bridge already initialized, skipping stub');
    return;
  }

  // Skip if stub already created
  if (window.auroraview && window.auroraview._isStub) {
    console.log('[AuroraView Stub] Stub already exists');
    return;
  }

  console.log('[AuroraView Stub] Creating bridge stub...');

  // Queue for pending calls
  const pendingCalls: Array<{
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
  }> = [];

  const readyCallbacks: Array<(av: typeof window.auroraview) => void> = [];

  /**
   * Stub implementation of window.auroraview
   */
  window.auroraview = {
    _isStub: true,
    _ready: false,
    _pendingCalls: pendingCalls,
    _boundMethods: {},

    call: function <T = unknown>(method: string, params?: unknown): Promise<T> {
      console.warn('[AuroraView Stub] Queuing call:', method, '(bridge not ready)');
      return new Promise(function (resolve, reject) {
        pendingCalls.push({
          method: method,
          params: params,
          resolve: resolve as (value: unknown) => void,
          reject: reject,
        });
      });
    },

    send_event: function (event: string, detail?: unknown): void {
      console.warn('[AuroraView Stub] Event queued (bridge not ready):', event);
      pendingCalls.push({
        type: 'event',
        event: event,
        detail: detail,
      });
    },

    on: function (event: string, handler: (data: unknown) => void): () => void {
      console.log('[AuroraView Stub] Handler queued for:', event);
      pendingCalls.push({
        type: 'handler',
        event: event,
        handler: handler,
      });
      // Return a no-op unsubscribe (real one will be set up when bridge loads)
      return () => {};
    },

    off: function (_event: string, _handler?: (data: unknown) => void): void {
      // No-op in stub mode
    },

    trigger: function (_event: string, _detail?: unknown): void {
      // No-op in stub mode
    },

    invoke: function <T = unknown>(cmd: string, args?: Record<string, unknown>): Promise<T> {
      console.warn('[AuroraView Stub] Queuing invoke:', cmd, '(bridge not ready)');
      return new Promise(function (resolve, reject) {
        pendingCalls.push({
          type: 'invoke',
          method: cmd,
          params: args,
          resolve: resolve as (value: unknown) => void,
          reject: reject,
        });
      });
    },

    whenReady: function (): Promise<typeof window.auroraview> {
      return new Promise(function (resolve) {
        if (window.auroraview!._ready && !window.auroraview!._isStub) {
          resolve(window.auroraview);
        } else {
          readyCallbacks.push(resolve);
        }
      });
    },

    isReady: function (): boolean {
      return false;
    },

    _registerApiMethods: function (namespace: string, methods: string[]): void {
      console.log('[AuroraView Stub] API registration queued:', namespace);
      pendingCalls.push({
        type: 'register',
        namespace: namespace,
        methods: methods,
      });
    },

    api: {},
  };

  console.log('[AuroraView Stub] ✓ Bridge stub created, calls will be queued');
})();
