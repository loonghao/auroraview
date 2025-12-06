/**
 * AuroraView BOM - Navigation API Utilities
 * 
 * This script provides navigation-related utilities and state checking functions.
 */
(function() {
    'use strict';

    /**
     * Check if can navigate back in history
     * @returns {boolean}
     */
    window.__auroraview_canGoBack = function() {
        return history.length > 1;
    };

    /**
     * Check if can navigate forward in history
     * Note: This is a heuristic as browser API doesn't expose forward history directly
     * @returns {boolean}
     */
    window.__auroraview_canGoForward = function() {
        return window.__auroraview_can_go_forward === true;
    };

    /**
     * Check if page is currently loading
     * @returns {boolean}
     */
    window.__auroraview_isLoading = function() {
        return window.__auroraview_is_loading === true;
    };

    /**
     * Get current load progress (0-100)
     * @returns {number}
     */
    window.__auroraview_getLoadProgress = function() {
        if (window.__auroraview_load_progress !== undefined) {
            return window.__auroraview_load_progress;
        }
        return document.readyState === 'complete' ? 100 : 0;
    };

    /**
     * Get current URL
     * @returns {string}
     */
    window.__auroraview_getCurrentUrl = function() {
        window.__auroraview_current_url = location.href;
        return location.href;
    };

    /**
     * Navigate to a URL
     * @param {string} url - The URL to navigate to
     */
    window.__auroraview_navigateTo = function(url) {
        location.href = url;
    };

    /**
     * Navigate back in history
     */
    window.__auroraview_goBack = function() {
        history.back();
    };

    /**
     * Navigate forward in history
     */
    window.__auroraview_goForward = function() {
        history.forward();
    };

    /**
     * Reload current page
     * @param {boolean} [force=false] - If true, reload from server; otherwise use cache
     */
    window.__auroraview_reload = function(force) {
        if (force) {
            location.reload(true);
        } else {
            location.reload();
        }
    };

    /**
     * Stop loading current page
     */
    window.__auroraview_stop = function() {
        window.stop();
    };

    console.log('[AuroraView BOM] Navigation API initialized');
})();

