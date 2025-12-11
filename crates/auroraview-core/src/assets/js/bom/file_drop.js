/**
 * AuroraView File Paste Handler
 *
 * Handles file paste events from clipboard.
 * Note: File drag-drop is handled natively by Rust/wry for full path access.
 *
 * Events emitted to Python:
 * - file_paste: When files are pasted from clipboard
 *
 * Native events (handled by Rust):
 * - file_drop_hover: When files are dragged over the window (with full paths)
 * - file_drop: When files are dropped (with full paths)
 * - file_drop_cancelled: When drag operation is cancelled
 *
 * @module file_drop
 */

(function() {
    'use strict';

    /**
     * Send file event to Python via IPC
     * @param {string} eventName
     * @param {Object} data
     */
    function sendFileEvent(eventName, data) {
        if (window.auroraview && window.auroraview.send_event) {
            window.auroraview.send_event(eventName, data);
        } else {
            console.warn('[AuroraView] File handler: bridge not ready, event not sent:', eventName);
        }
    }

    // Handle paste events for file paste support
    // Note: Clipboard paste cannot provide full file paths due to browser security
    document.addEventListener('paste', function(e) {
        if (e.clipboardData && e.clipboardData.files && e.clipboardData.files.length > 0) {
            var files = [];
            for (var i = 0; i < e.clipboardData.files.length; i++) {
                var file = e.clipboardData.files[i];
                files.push({
                    name: file.name,
                    size: file.size,
                    type: file.type || 'application/octet-stream',
                    lastModified: file.lastModified
                });
            }

            if (files.length > 0) {
                sendFileEvent('file_paste', {
                    files: files,
                    timestamp: Date.now()
                });
            }
        }
    }, false);

    console.log('[AuroraView] File paste handler initialized (drag-drop handled natively)');
})();
