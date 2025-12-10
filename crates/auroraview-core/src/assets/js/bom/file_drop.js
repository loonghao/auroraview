/**
 * AuroraView File Drop Handler
 *
 * Handles file drag and drop events and sends file information to Python.
 * Supports both single and multiple file drops.
 *
 * Events emitted to Python:
 * - file_drop_hover: When files are dragged over the window
 * - file_drop: When files are dropped
 * - file_drop_cancelled: When drag operation is cancelled
 *
 * @module file_drop
 */

(function() {
    'use strict';

    // Track drag state
    var dragCounter = 0;

    /**
     * Extract file information from DataTransfer
     * @param {DataTransfer} dataTransfer
     * @returns {Array} Array of file info objects
     */
    function extractFileInfo(dataTransfer) {
        var files = [];

        if (dataTransfer.files && dataTransfer.files.length > 0) {
            for (var i = 0; i < dataTransfer.files.length; i++) {
                var file = dataTransfer.files[i];
                files.push({
                    name: file.name,
                    size: file.size,
                    type: file.type || 'application/octet-stream',
                    lastModified: file.lastModified
                });
            }
        }

        return files;
    }

    /**
     * Extract paths from DataTransfer items (if available)
     * Note: For security reasons, browsers don't expose full paths
     * The actual file paths are handled by the native layer
     * @param {DataTransfer} dataTransfer
     * @returns {Array} Array of path strings (may be empty in browser context)
     */
    function extractPaths(dataTransfer) {
        var paths = [];

        // Try to get paths from items (WebView2 may provide this)
        if (dataTransfer.items) {
            for (var i = 0; i < dataTransfer.items.length; i++) {
                var item = dataTransfer.items[i];
                if (item.kind === 'file') {
                    var file = item.getAsFile();
                    if (file && file.path) {
                        paths.push(file.path);
                    } else if (file && file.name) {
                        // Fallback to name if path not available
                        paths.push(file.name);
                    }
                }
            }
        }

        return paths;
    }

    /**
     * Get drop position relative to the window
     * @param {DragEvent} event
     * @returns {Object} Position object with x, y coordinates
     */
    function getDropPosition(event) {
        return {
            x: event.clientX,
            y: event.clientY,
            screenX: event.screenX,
            screenY: event.screenY
        };
    }

    /**
     * Send file drop event to Python
     * @param {string} eventName
     * @param {Object} data
     */
    function sendDropEvent(eventName, data) {
        if (window.auroraview && window.auroraview.send_event) {
            window.auroraview.send_event(eventName, data);
        } else {
            console.warn('[AuroraView] File drop: bridge not ready, event not sent:', eventName);
        }
    }

    // Prevent default drag behaviors
    document.addEventListener('dragenter', function(e) {
        e.preventDefault();
        e.stopPropagation();
        dragCounter++;

        if (dragCounter === 1) {
            // First entry - notify hover start
            var files = extractFileInfo(e.dataTransfer);
            sendDropEvent('file_drop_hover', {
                hovering: true,
                files: files,
                position: getDropPosition(e)
            });
        }
    }, false);

    document.addEventListener('dragleave', function(e) {
        e.preventDefault();
        e.stopPropagation();
        dragCounter--;

        if (dragCounter === 0) {
            // Left the window - notify hover end
            sendDropEvent('file_drop_cancelled', {
                hovering: false,
                reason: 'left_window'
            });
        }
    }, false);

    document.addEventListener('dragover', function(e) {
        e.preventDefault();
        e.stopPropagation();

        // Set drop effect
        if (e.dataTransfer) {
            e.dataTransfer.dropEffect = 'copy';
        }
    }, false);

    document.addEventListener('drop', function(e) {
        e.preventDefault();
        e.stopPropagation();
        dragCounter = 0;

        var files = extractFileInfo(e.dataTransfer);
        var paths = extractPaths(e.dataTransfer);

        if (files.length > 0) {
            sendDropEvent('file_drop', {
                files: files,
                paths: paths,
                position: getDropPosition(e),
                timestamp: Date.now()
            });
        } else {
            sendDropEvent('file_drop_cancelled', {
                hovering: false,
                reason: 'no_files'
            });
        }
    }, false);

    // Also handle paste events for file paste support
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
                sendDropEvent('file_paste', {
                    files: files,
                    timestamp: Date.now()
                });
            }
        }
    }, false);

    console.log('[AuroraView] File drop handler initialized');
})();
