/**
 * AuroraView Event Bridge - Core JavaScript API
 *
 * This script provides the core event bridge between JavaScript and Python.
 * It is injected at WebView initialization and persists across navigations.
 *
 * @module event_bridge
 */

(function() {
    'use strict';

    console.log('[AuroraView] Initializing event bridge...');

    // Check if already initialized (prevent double initialization)
    if (window.auroraview && window.auroraview._ready) {
        console.log('[AuroraView] Event bridge already initialized, skipping');
        return;
    }

    // Preserve any pending calls from stub if it exists
    var pendingFromStub = (window.auroraview && window.auroraview._pendingCalls)
        ? window.auroraview._pendingCalls.slice()
        : [];

    // Ready callbacks for whenReady() API
    var readyCallbacks = [];

    // Event handlers registry for Python -> JS communication
    const eventHandlers = new Map();

    // Pending call registry for auroraview.call Promise resolution
    let auroraviewCallIdCounter = 0;
    const auroraviewPendingCalls = new Map();

    /**
     * Generate unique call ID for Promise tracking
     * @returns {string} Unique call ID
     */
    function auroraviewGenerateCallId() {
        auroraviewCallIdCounter += 1;
        return 'av_call_' + Date.now() + '_' + auroraviewCallIdCounter;
    }

    /**
     * Handle call_result events coming back from Python (Python -> JS)
     * This is an internal handler called by trigger() for __auroraview_call_result events
     * @param {object} detail - The call result payload
     */
    function handleCallResult(detail) {
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
                const error = new Error(errInfo.message || 'AuroraView call failed');
                if (errInfo.name) error.name = errInfo.name;
                if (Object.prototype.hasOwnProperty.call(errInfo, 'code')) error.code = errInfo.code;
                if (Object.prototype.hasOwnProperty.call(errInfo, 'data')) error.data = errInfo.data;
                pending.reject(error);
            }
        } catch (e) {
            console.error('[AuroraView] Error handling call_result:', e);
        }
    }

    /**
     * Primary AuroraView bridge API
     * Provides low-level communication with Python backend
     */
    window.auroraview = {
        /**
         * High-level call API (JS -> Python, Promise-based)
         * @param {string} method - Python method name
         * @param {*} params - Method parameters
         * @returns {Promise} Promise that resolves with Python method result
         */
        call: function(method, params) {
            console.log('[AuroraView] Calling Python method via auroraview.call:', method, params);
            return new Promise(function(resolve, reject) {
                const id = auroraviewGenerateCallId();
                auroraviewPendingCalls.set(id, { resolve: resolve, reject: reject });

                try {
                    const payload = {
                        type: 'call',
                        id: id,
                        method: method,
                    };
                    if (typeof params !== 'undefined') {
                        payload.params = params;
                    }
                    window.ipc.postMessage(JSON.stringify(payload));
                } catch (e) {
                    console.error('[AuroraView] Failed to send call via IPC:', e);
                    auroraviewPendingCalls.delete(id);
                    reject(e);
                }
            });
        },

        /**
         * Send event to Python (JS -> Python, fire-and-forget)
         * @param {string} event - Event name
         * @param {*} detail - Event data
         */
        send_event: function(event, detail) {
            try {
                const payload = {
                    type: 'event',
                    event: event,
                    detail: detail || {}
                };
                window.ipc.postMessage(JSON.stringify(payload));
                console.log('[AuroraView] Event sent:', event, detail);
            } catch (e) {
                console.error('[AuroraView] Failed to send event:', e);
            }
        },

        /**
         * Register event handler (Python -> JS)
         * @param {string} event - Event name
         * @param {Function} handler - Event handler function
         */
        on: function(event, handler) {
            if (typeof handler !== 'function') {
                console.error('[AuroraView] Handler must be a function');
                return;
            }
            if (!eventHandlers.has(event)) {
                eventHandlers.set(event, []);
            }
            eventHandlers.get(event).push(handler);
            console.log('[AuroraView] Registered handler for event:', event);
        },

        /**
         * Trigger event handlers (called by Python)
         * @param {string} event - Event name
         * @param {*} detail - Event data
         */
        trigger: function(event, detail) {
            // Special handling for internal call_result events
            if (event === '__auroraview_call_result') {
                handleCallResult(detail);
                return;
            }

            const handlers = eventHandlers.get(event);
            if (!handlers || handlers.length === 0) {
                console.warn('[AuroraView] No handlers for event:', event);
                return;
            }
            handlers.forEach(function(handler) {
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
         * Command format: "plugin:<plugin_name>|<command_name>"
         * 
         * @example
         * // Read a file using the fs plugin
         * const content = await auroraview.invoke('plugin:fs|read_file', { path: '/path/to/file.txt' });
         * 
         * // Open a file dialog
         * const file = await auroraview.invoke('plugin:dialog|open_file', { title: 'Select File' });
         * 
         * @param {string} cmd - Plugin command in format "plugin:<plugin>|<command>"
         * @param {object} [args={}] - Command arguments
         * @returns {Promise} Promise that resolves with command result
         */
        invoke: function(cmd, args) {
            console.log('[AuroraView] Invoking plugin command:', cmd, args);
            return new Promise(function(resolve, reject) {
                var id = auroraviewGenerateCallId();
                auroraviewPendingCalls.set(id, { resolve: resolve, reject: reject });

                try {
                    var payload = {
                        type: 'invoke',
                        id: id,
                        cmd: cmd,
                        args: args || {}
                    };
                    window.ipc.postMessage(JSON.stringify(payload));
                } catch (e) {
                    console.error('[AuroraView] Failed to send invoke via IPC:', e);
                    auroraviewPendingCalls.delete(id);
                    reject(e);
                }
            });
        },

        /**
         * Ready state flag - true when bridge is fully initialized
         * Use whenReady() for async checking
         */
        _ready: false,

        /**
         * Pending calls queue - stores calls made before bridge is ready
         * @private
         */
        _pendingCalls: [],

        /**
         * Wait for event bridge to be ready
         * Use this to safely call bridge methods in DCC environments
         * where initialization timing may vary.
         *
         * @example
         * window.auroraview.whenReady().then(function(av) {
         *     av.call('api.myMethod', { param: 'value' });
         * });
         *
         * @returns {Promise} Promise that resolves with window.auroraview when ready
         */
        whenReady: function() {
            return new Promise(function(resolve) {
                if (window.auroraview._ready) {
                    resolve(window.auroraview);
                } else {
                    readyCallbacks.push(resolve);
                }
            });
        },

        /**
         * Check if bridge is ready (synchronous)
         * @returns {boolean} True if bridge is fully initialized
         */
        isReady: function() {
            return window.auroraview._ready === true;
        },

        /**
         * Registry of all bound methods for duplicate detection
         * @private
         */
        _boundMethods: {},

        /**
         * Check if a method is already registered
         * @param {string} fullMethodName - Full method name (e.g., "api.echo")
         * @returns {boolean} True if method is already bound
         */
        isMethodBound: function(fullMethodName) {
            return !!window.auroraview._boundMethods[fullMethodName];
        },

        /**
         * Get list of all bound method names
         * @returns {Array<string>} Array of bound method names
         */
        getBoundMethods: function() {
            return Object.keys(window.auroraview._boundMethods);
        },

        /**
         * Register API methods dynamically
         * This is called by Rust/Python to populate window.auroraview.api
         * @param {string} namespace - Namespace (e.g., "api")
         * @param {Array<string>} methods - Array of method names
         * @param {Object} options - Optional configuration
         * @param {boolean} options.allowRebind - Allow rebinding existing methods (default: true)
         */
        _registerApiMethods: function(namespace, methods, options) {
            if (!namespace || !methods || !Array.isArray(methods)) {
                console.error('[AuroraView] Invalid arguments for _registerApiMethods');
                return;
            }

            var opts = options || {};
            var allowRebind = opts.allowRebind !== false;

            // Create namespace if it doesn't exist
            if (!window.auroraview[namespace]) {
                window.auroraview[namespace] = {};
            }

            // Track registered and skipped methods
            var registeredCount = 0;
            var skippedCount = 0;

            // Create wrapper methods
            for (var i = 0; i < methods.length; i++) {
                var methodName = methods[i];
                var fullMethodName = namespace + '.' + methodName;

                // Check for duplicate registration
                if (window.auroraview._boundMethods[fullMethodName]) {
                    if (!allowRebind) {
                        console.debug('[AuroraView] Skipping already bound method:', fullMethodName);
                        skippedCount++;
                        continue;
                    }
                    console.debug('[AuroraView] Rebinding method:', fullMethodName);
                }

                // Create closure to capture method name
                window.auroraview[namespace][methodName] = (function(fullName) {
                    return function(params) {
                        return window.auroraview.call(fullName, params);
                    };
                })(fullMethodName);

                // Mark as bound
                window.auroraview._boundMethods[fullMethodName] = true;
                registeredCount++;
            }

            if (registeredCount > 0) {
                console.log('[AuroraView] Registered ' + registeredCount + ' methods in window.auroraview.' + namespace);
            }
            if (skippedCount > 0) {
                console.log('[AuroraView] Skipped ' + skippedCount + ' already-bound methods in window.auroraview.' + namespace);
            }
        }
    };

    // Mark bridge as ready
    window.auroraview._ready = true;

    // Process any pending calls from stub
    if (pendingFromStub.length > 0) {
        console.log('[AuroraView] Processing ' + pendingFromStub.length + ' pending calls from stub');
        pendingFromStub.forEach(function(pending) {
            try {
                window.auroraview.call(pending.method, pending.params)
                    .then(pending.resolve)
                    .catch(pending.reject);
            } catch (e) {
                pending.reject(e);
            }
        });
    }

    // Notify all whenReady() waiters
    if (readyCallbacks.length > 0) {
        console.log('[AuroraView] Notifying ' + readyCallbacks.length + ' ready callbacks');
        readyCallbacks.forEach(function(callback) {
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
    // This allows Python to re-register API methods after page navigation
    try {
        window.auroraview.send_event('__auroraview_ready', {
            timestamp: Date.now(),
            url: window.location.href
        });
        console.log('[AuroraView] ✓ Sent __auroraview_ready event to backend');
    } catch (e) {
        console.warn('[AuroraView] Failed to send __auroraview_ready event:', e);
    }
})();
