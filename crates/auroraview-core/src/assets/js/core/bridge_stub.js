/**
 * AuroraView Bridge Stub - Early Initialization Placeholder
 * 
 * This stub creates a minimal window.auroraview namespace before the full
 * event bridge is loaded. It queues any calls made during this time and
 * replays them once the real bridge is initialized.
 * 
 * Use this in DCC environments where timing of WebView initialization
 * may vary, or when frontend code loads before the Rust bridge injects.
 * 
 * @module bridge_stub
 */

(function() {
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
    var pendingCalls = [];
    var readyCallbacks = [];

    /**
     * Stub implementation of window.auroraview
     * All calls are queued until the real bridge takes over
     */
    window.auroraview = {
        /**
         * Marker to identify this as a stub
         */
        _isStub: true,

        /**
         * Not ready yet - stub is a placeholder
         */
        _ready: false,

        /**
         * Pending calls queue - will be processed by real bridge
         */
        _pendingCalls: pendingCalls,

        /**
         * Stub call() - queues the call for later execution
         */
        call: function(method, params) {
            console.warn('[AuroraView Stub] Queuing call:', method, '(bridge not ready)');
            return new Promise(function(resolve, reject) {
                pendingCalls.push({
                    method: method,
                    params: params,
                    resolve: resolve,
                    reject: reject
                });
            });
        },

        /**
         * Stub send_event() - logs warning and queues event
         */
        send_event: function(event, detail) {
            console.warn('[AuroraView Stub] Event queued (bridge not ready):', event);
            pendingCalls.push({
                type: 'event',
                event: event,
                detail: detail
            });
        },

        /**
         * Stub on() - stores handler for later registration
         */
        on: function(event, handler) {
            console.log('[AuroraView Stub] Handler queued for:', event);
            pendingCalls.push({
                type: 'handler',
                event: event,
                handler: handler
            });
        },

        /**
         * whenReady() - resolves when real bridge initializes
         */
        whenReady: function() {
            return new Promise(function(resolve) {
                if (window.auroraview._ready && !window.auroraview._isStub) {
                    resolve(window.auroraview);
                } else {
                    readyCallbacks.push(resolve);
                }
            });
        },

        /**
         * isReady() - always false for stub
         */
        isReady: function() {
            return false;
        },

        /**
         * Stub _registerApiMethods() - queues for later
         */
        _registerApiMethods: function(namespace, methods) {
            console.log('[AuroraView Stub] API registration queued:', namespace);
            pendingCalls.push({
                type: 'register',
                namespace: namespace,
                methods: methods
            });
        },

        /**
         * Empty api namespace - methods will be populated later
         */
        api: {}
    };

    console.log('[AuroraView Stub] ✓ Bridge stub created, calls will be queued');
})();

