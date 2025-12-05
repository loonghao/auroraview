/**
 * AuroraView BOM - DOM and Window Events Tracker
 * 
 * This script tracks DOM events such as title changes, URL changes,
 * and various window events (visibility, focus, resize, fullscreen).
 * 
 * Events emitted:
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
            window.ipc.postMessage(JSON.stringify({
                event: eventName,
                data: data
            }));
        }
    }

    // Track title changes using MutationObserver
    var lastTitle = document.title;
    var titleObserver = new MutationObserver(function(mutations) {
        if (document.title !== lastTitle) {
            lastTitle = document.title;
            emitEvent('title_changed', { title: document.title });
        }
    });

    // Observe title element for changes
    var titleElement = document.querySelector('title');
    if (titleElement) {
        titleObserver.observe(titleElement, { subtree: true, characterData: true, childList: true });
    }

    // Also observe head for new title elements
    if (document.head) {
        titleObserver.observe(document.head, { childList: true });
    }

    // Track URL changes (for SPA navigation)
    var lastUrl = location.href;
    setInterval(function() {
        if (location.href !== lastUrl) {
            lastUrl = location.href;
            emitEvent('url_changed', { url: location.href });
        }
    }, 100);

    // Emit dom_ready when DOM is interactive
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', function() {
            emitEvent('dom_ready', { url: location.href });
        });
    } else {
        // DOM already ready
        emitEvent('dom_ready', { url: location.href });
    }

    // Window visibility events
    document.addEventListener('visibilitychange', function() {
        if (document.hidden) {
            emitEvent('window_hide', {});
        } else {
            emitEvent('window_show', {});
        }
    });

    // Window focus/blur events
    window.addEventListener('focus', function() {
        emitEvent('window_focus', {});
    });
    window.addEventListener('blur', function() {
        emitEvent('window_blur', {});
    });

    // Window resize event (debounced)
    var resizeTimeout;
    window.addEventListener('resize', function() {
        clearTimeout(resizeTimeout);
        resizeTimeout = setTimeout(function() {
            emitEvent('window_resize', {
                width: window.innerWidth,
                height: window.innerHeight
            });
        }, 100);
    });

    // Fullscreen change event
    document.addEventListener('fullscreenchange', function() {
        emitEvent('fullscreen_changed', {
            fullscreen: !!document.fullscreenElement
        });
    });
    // Webkit prefix for Safari
    document.addEventListener('webkitfullscreenchange', function() {
        emitEvent('fullscreen_changed', {
            fullscreen: !!document.webkitFullscreenElement
        });
    });

    console.log('[AuroraView BOM] DOM events tracker initialized');
})();

