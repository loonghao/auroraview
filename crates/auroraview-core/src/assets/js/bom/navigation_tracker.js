/**
 * AuroraView BOM - Navigation and Loading State Tracker
 * 
 * This script tracks navigation state, loading progress, and emits events
 * to the Python backend via IPC.
 * 
 * Events emitted:
 * - navigation_started: { url: string }
 * - navigation_completed: { url: string, success: boolean }
 * - navigation_failed: { url: string, error: string }
 * - load_progress: { url: string, progress: number }
 * - title_changed: { title: string }
 * - url_changed: { url: string }
 * - dom_ready: { url: string }
 * - window_show: {}
 * - window_hide: {}
 * - window_focus: {}
 * - window_blur: {}
 * - window_resize: { width: number, height: number }
 * - fullscreen_changed: { fullscreen: boolean }
 */
(function() {
    'use strict';

    // Helper function to emit events to Python
    function emitEvent(eventName, data) {
        if (window.auroraview && typeof window.auroraview.emit === 'function') {
            window.auroraview.emit(eventName, data);
        } else if (window.ipc && typeof window.ipc.postMessage === 'function') {
            // Fallback to raw IPC
            window.ipc.postMessage(JSON.stringify({
                event: eventName,
                data: data
            }));
        }
    }

    // Helper to emit unified navigation event (new API)
    function emitNavigationEvent(eventType, url, options) {
        options = options || {};
        var event = {
            url: url,
            event_type: eventType,
            success: options.success !== undefined ? options.success : true,
            error: options.error || null,
            progress: options.progress || 0
        };
        emitEvent('navigation', event);
    }

    // Track if forward navigation is possible
    window.__auroraview_can_go_forward = false;

    // Track loading state
    window.__auroraview_is_loading = document.readyState !== 'complete';
    window.__auroraview_load_progress = document.readyState === 'complete' ? 100 : 0;
    window.__auroraview_current_url = location.href;

    // Listen for popstate events to update forward navigation status
    window.addEventListener('popstate', function(event) {
        // After going back, forward should be possible
        window.__auroraview_can_go_forward = true;

        // Emit navigation started event for history navigation
        var newUrl = location.href;
        if (newUrl !== window.__auroraview_current_url) {
            emitEvent('navigation_started', { url: newUrl });
            emitNavigationEvent('start', newUrl);
            window.__auroraview_current_url = newUrl;
        }
    });

    // Reset forward flag on new navigation
    var originalPushState = history.pushState;
    history.pushState = function() {
        window.__auroraview_can_go_forward = false;
        var result = originalPushState.apply(history, arguments);

        // Emit navigation event for pushState
        var newUrl = location.href;
        if (newUrl !== window.__auroraview_current_url) {
            emitEvent('navigation_started', { url: newUrl });
            emitNavigationEvent('start', newUrl);
            window.__auroraview_current_url = newUrl;
        }
        return result;
    };

    var originalReplaceState = history.replaceState;
    history.replaceState = function() {
        var result = originalReplaceState.apply(history, arguments);

        // Emit navigation event for replaceState
        var newUrl = location.href;
        if (newUrl !== window.__auroraview_current_url) {
            emitEvent('navigation_started', { url: newUrl });
            emitNavigationEvent('start', newUrl);
            window.__auroraview_current_url = newUrl;
        }
        return result;
    };

    // Track loading state changes and emit navigation events
    document.addEventListener('readystatechange', function() {
        var url = location.href;
        switch (document.readyState) {
            case 'loading':
                window.__auroraview_is_loading = true;
                window.__auroraview_load_progress = 10;
                emitEvent('load_progress', { url: url, progress: 10 });
                emitNavigationEvent('progress', url, { progress: 10 });
                break;
            case 'interactive':
                window.__auroraview_is_loading = true;
                window.__auroraview_load_progress = 50;
                emitEvent('load_progress', { url: url, progress: 50 });
                emitNavigationEvent('progress', url, { progress: 50 });
                break;
            case 'complete':
                window.__auroraview_is_loading = false;
                window.__auroraview_load_progress = 100;
                emitEvent('load_progress', { url: url, progress: 100 });
                emitNavigationEvent('progress', url, { progress: 100 });
                break;
        }
    });

    // Track page load completion - emit navigation_completed
    window.addEventListener('load', function() {
        window.__auroraview_is_loading = false;
        window.__auroraview_load_progress = 100;
        emitEvent('navigation_completed', {
            url: location.href,
            success: true
        });
        emitNavigationEvent('end', location.href, { success: true });
    });

    // Track navigation start (beforeunload indicates leaving page)
    window.addEventListener('beforeunload', function() {
        window.__auroraview_is_loading = true;
        window.__auroraview_load_progress = 0;
        emitEvent('navigation_started', { url: location.href });
        emitNavigationEvent('start', location.href);
    });

    // Track page errors for navigation_failed
    window.addEventListener('error', function(event) {
        // Only emit for resource loading errors that might indicate navigation failure
        if (event.target && (event.target.tagName === 'SCRIPT' || event.target.tagName === 'LINK')) {
            var errorMsg = 'Resource loading failed: ' + (event.target.src || event.target.href);
            emitEvent('navigation_failed', {
                url: location.href,
                error: errorMsg
            });
            emitNavigationEvent('end', location.href, { success: false, error: errorMsg });
        }
    }, true);

    console.log('[AuroraView BOM] Navigation tracker initialized');
})();

