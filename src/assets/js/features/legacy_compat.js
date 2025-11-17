/**
 * AuroraView Legacy Compatibility Layer
 * 
 * Provides backward compatibility with older AuroraView API.
 * Creates the legacy `window.aurora` alias.
 * 
 * @module legacy_compat
 */

(function() {
    'use strict';
    
    console.log('[AuroraView] Initializing legacy compatibility layer...');

    /**
     * Legacy AuroraView class wrapper
     */
    window.AuroraView = function() {
        this.ready = true;
        this.version = '2.0';
    };

    window.AuroraView.prototype = {
        /**
         * Legacy call method - delegates to window.auroraview.call
         */
        call: function(method, params) {
            return window.auroraview.call(method, params);
        },

        /**
         * Legacy send_event method - delegates to window.auroraview.send_event
         */
        send_event: function(event, detail) {
            return window.auroraview.send_event(event, detail);
        },

        /**
         * Legacy on method - delegates to window.auroraview.on
         */
        on: function(event, handler) {
            return window.auroraview.on(event, handler);
        },

        /**
         * Check if AuroraView is ready
         */
        isReady: function() {
            return this.ready;
        }
    };

    // Create legacy instance: window.aurora
    window.aurora = new window.AuroraView();

    // Intercept CustomEvent dispatch for backward compatibility
    const originalDispatchEvent = window.dispatchEvent;
    window.dispatchEvent = function(event) {
        if (event instanceof CustomEvent && event.type.startsWith('__auroraview_')) {
            // Let AuroraView handle its own events
            return originalDispatchEvent.call(window, event);
        }
        return originalDispatchEvent.call(window, event);
    };

    console.log('[AuroraView] ✓ Legacy compatibility layer initialized');
    console.log('[AuroraView] ✓ Legacy API: window.aurora (alias for window.auroraview)');
})();
