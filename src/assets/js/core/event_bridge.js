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
     */
    window.addEventListener('__auroraview_call_result', function(event) {
        try {
            const detail = event && event.detail ? event.detail : {};
            const id = detail.id;
            
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
    });

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
        api: {}
    };

    console.log('[AuroraView] ✓ Event bridge initialized');
    console.log('[AuroraView] ✓ API: window.auroraview.call() / .send_event() / .on()');
})();
