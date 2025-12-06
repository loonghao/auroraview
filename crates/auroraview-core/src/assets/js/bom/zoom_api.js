/**
 * AuroraView BOM - Zoom API Utilities
 * 
 * This script provides zoom control and zoom level tracking.
 */
(function() {
    'use strict';

    // Track current zoom level
    window.__auroraview_zoom_level = 1.0;

    /**
     * Set zoom level
     * @param {number} scale - Zoom scale factor (1.0 = 100%, 1.5 = 150%, etc.)
     */
    window.__auroraview_setZoom = function(scale) {
        if (typeof scale !== 'number' || scale <= 0) {
            console.error('[AuroraView BOM] Invalid zoom scale:', scale);
            return;
        }
        
        document.body.style.zoom = String(scale);
        window.__auroraview_zoom_level = scale;
        console.log('[AuroraView BOM] Zoom set to', scale);
    };

    /**
     * Get current zoom level
     * @returns {number}
     */
    window.__auroraview_getZoom = function() {
        return window.__auroraview_zoom_level;
    };

    /**
     * Zoom in by a step (default 0.1)
     * @param {number} [step=0.1] - Zoom step increment
     */
    window.__auroraview_zoomIn = function(step) {
        step = step || 0.1;
        var newZoom = Math.min(window.__auroraview_zoom_level + step, 5.0);
        window.__auroraview_setZoom(newZoom);
    };

    /**
     * Zoom out by a step (default 0.1)
     * @param {number} [step=0.1] - Zoom step decrement
     */
    window.__auroraview_zoomOut = function(step) {
        step = step || 0.1;
        var newZoom = Math.max(window.__auroraview_zoom_level - step, 0.1);
        window.__auroraview_setZoom(newZoom);
    };

    /**
     * Reset zoom to 100%
     */
    window.__auroraview_resetZoom = function() {
        window.__auroraview_setZoom(1.0);
    };

    console.log('[AuroraView BOM] Zoom API initialized');
})();

