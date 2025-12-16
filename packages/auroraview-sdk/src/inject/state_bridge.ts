/**
 * AuroraView State Bridge
 *
 * Provides reactive shared state between Python and JavaScript.
 * Inspired by PyWebView's state mechanism.
 *
 * @module state_bridge
 */

(function () {
  'use strict';

  type StateChangeHandler = (key: string, value: unknown, source: 'python' | 'javascript') => void;

  // Internal state storage
  const _stateData: Record<string, unknown> = {};
  const _changeHandlers: StateChangeHandler[] = [];

  /**
   * Notify all change handlers
   */
  function notifyHandlers(key: string, value: unknown, source: 'python' | 'javascript'): void {
    _changeHandlers.forEach((handler) => {
      try {
        handler(key, value, source);
      } catch (e) {
        console.error('[AuroraView State] Handler error:', e);
      }
    });
  }

  /**
   * Send state update to Python
   */
  function sendToPython(key: string, value: unknown): void {
    if (window.auroraview && window.auroraview.send_event) {
      window.auroraview.send_event('__state_update__', { key: key, value: value });
    }
  }

  /**
   * Create a reactive proxy for state object
   */
  function createStateProxy(): Record<string, unknown> & {
    onChange: (handler: StateChangeHandler) => () => void;
    offChange: (handler: StateChangeHandler) => void;
    toJSON: () => Record<string, unknown>;
    keys: () => string[];
  } {
    return new Proxy(_stateData, {
      get: function (target, prop: string) {
        if (prop === 'onChange') {
          return function (handler: StateChangeHandler): () => void {
            _changeHandlers.push(handler);
            return function () {
              const idx = _changeHandlers.indexOf(handler);
              if (idx > -1) _changeHandlers.splice(idx, 1);
            };
          };
        }
        if (prop === 'offChange') {
          return function (handler: StateChangeHandler): void {
            const idx = _changeHandlers.indexOf(handler);
            if (idx > -1) _changeHandlers.splice(idx, 1);
          };
        }
        if (prop === 'toJSON') {
          return function (): Record<string, unknown> {
            return Object.assign({}, target);
          };
        }
        if (prop === 'keys') {
          return function (): string[] {
            return Object.keys(target);
          };
        }
        return target[prop];
      },
      set: function (target, prop: string, value: unknown): boolean {
        const oldValue = target[prop];
        target[prop] = value;

        // Only sync if value actually changed
        if (oldValue !== value) {
          sendToPython(prop, value);
          notifyHandlers(prop, value, 'javascript');
        }
        return true;
      },
      deleteProperty: function (target, prop: string): boolean {
        if (prop in target) {
          delete target[prop];
          sendToPython(prop, undefined);
          notifyHandlers(prop, undefined, 'javascript');
        }
        return true;
      },
    }) as ReturnType<typeof createStateProxy>;
  }

  // Create the state proxy
  const stateProxy = createStateProxy();

  // Handle sync messages from Python
  function handleStateSync(data: {
    type: 'set' | 'delete' | 'batch' | 'full' | 'clear';
    key?: string;
    value?: unknown;
    data?: Record<string, unknown>;
  }): void {
    if (!data || typeof data !== 'object') return;

    switch (data.type) {
      case 'set':
        if (data.key) {
          _stateData[data.key] = data.value;
          notifyHandlers(data.key, data.value, 'python');
        }
        break;

      case 'delete':
        if (data.key) {
          delete _stateData[data.key];
          notifyHandlers(data.key, undefined, 'python');
        }
        break;

      case 'batch':
        if (data.data && typeof data.data === 'object') {
          Object.entries(data.data).forEach(([key, value]) => {
            _stateData[key] = value;
            notifyHandlers(key, value, 'python');
          });
        }
        break;

      case 'full':
        // Clear and replace all state
        Object.keys(_stateData).forEach((key) => delete _stateData[key]);
        if (data.data && typeof data.data === 'object') {
          Object.assign(_stateData, data.data);
          Object.entries(data.data).forEach(([key, value]) => {
            notifyHandlers(key, value, 'python');
          });
        }
        break;

      case 'clear':
        const keys = Object.keys(_stateData);
        keys.forEach((key) => {
          delete _stateData[key];
          notifyHandlers(key, undefined, 'python');
        });
        break;
    }
  }

  // Register state sync handler
  if (window.auroraview) {
    (window.auroraview as Record<string, unknown>).state = stateProxy;
    window.auroraview.on('__state_sync__', handleStateSync);
  } else {
    // Wait for auroraview to be available
    Object.defineProperty(window, 'auroraview', {
      configurable: true,
      set: function (val) {
        delete (window as { auroraview?: unknown }).auroraview;
        window.auroraview = val;
        (window.auroraview as Record<string, unknown>).state = stateProxy;
        window.auroraview!.on('__state_sync__', handleStateSync);
      },
    });
  }

  console.log('[AuroraView] State bridge initialized');
})();
