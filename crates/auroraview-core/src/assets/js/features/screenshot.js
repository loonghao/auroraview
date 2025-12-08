/**
 * AuroraView Screenshot Module
 *
 * Provides screenshot capture functionality using html2canvas.
 * This module is injected into the WebView and provides methods for:
 * - Full page screenshots
 * - Element screenshots
 * - Viewport screenshots
 *
 * The screenshot data is sent back to Python via IPC.
 */
(function() {
    'use strict';

    // html2canvas CDN URL (v1.4.1 - stable)
    var HTML2CANVAS_CDN = 'https://cdnjs.cloudflare.com/ajax/libs/html2canvas/1.4.1/html2canvas.min.js';

    // Screenshot state
    var html2canvasLoaded = false;
    var html2canvasLoading = false;
    var loadCallbacks = [];

    /**
     * Load html2canvas library dynamically
     */
    function loadHtml2Canvas(callback) {
        if (html2canvasLoaded) {
            callback(null);
            return;
        }

        loadCallbacks.push(callback);

        if (html2canvasLoading) {
            return;
        }

        html2canvasLoading = true;

        var script = document.createElement('script');
        script.src = HTML2CANVAS_CDN;
        script.async = true;

        script.onload = function() {
            html2canvasLoaded = true;
            html2canvasLoading = false;
            loadCallbacks.forEach(function(cb) { cb(null); });
            loadCallbacks = [];
        };

        script.onerror = function() {
            html2canvasLoading = false;
            var error = new Error('Failed to load html2canvas from CDN');
            loadCallbacks.forEach(function(cb) { cb(error); });
            loadCallbacks = [];
        };

        document.head.appendChild(script);
    }

    /**
     * Capture screenshot of the page or element
     *
     * @param {Object} options - Screenshot options
     * @param {string} options.selector - CSS selector for element (null for full page)
     * @param {boolean} options.fullPage - Capture full scrollable page
     * @param {Object} options.clip - Clip region {x, y, width, height}
     * @param {string} options.format - Image format: 'png' or 'jpeg'
     * @param {number} options.quality - JPEG quality (0-1)
     * @param {number} options.scale - Scale factor
     * @param {string} options.callbackId - Callback ID for async response
     */
    function captureScreenshot(options) {
        options = options || {};
        var callbackId = options.callbackId || 0;
        var format = options.format || 'png';
        var quality = options.quality || 0.92;
        var scale = options.scale || window.devicePixelRatio || 1;

        loadHtml2Canvas(function(error) {
            if (error) {
                sendResult(callbackId, null, error.message);
                return;
            }

            var target = document.body;
            if (options.selector) {
                target = document.querySelector(options.selector);
                if (!target) {
                    sendResult(callbackId, null, 'Element not found: ' + options.selector);
                    return;
                }
            }

            var html2canvasOptions = {
                scale: scale,
                useCORS: true,
                allowTaint: true,
                logging: false,
                backgroundColor: null
            };

            // Full page screenshot
            if (options.fullPage) {
                html2canvasOptions.windowWidth = document.documentElement.scrollWidth;
                html2canvasOptions.windowHeight = document.documentElement.scrollHeight;
                html2canvasOptions.x = 0;
                html2canvasOptions.y = 0;
                html2canvasOptions.scrollX = 0;
                html2canvasOptions.scrollY = 0;
            }

            // Clip region
            if (options.clip) {
                html2canvasOptions.x = options.clip.x || 0;
                html2canvasOptions.y = options.clip.y || 0;
                html2canvasOptions.width = options.clip.width;
                html2canvasOptions.height = options.clip.height;
            }

            window.html2canvas(target, html2canvasOptions).then(function(canvas) {
                var mimeType = format === 'jpeg' ? 'image/jpeg' : 'image/png';
                var dataUrl = canvas.toDataURL(mimeType, quality);

                // Extract base64 data (remove data:image/xxx;base64, prefix)
                var base64Data = dataUrl.split(',')[1];

                sendResult(callbackId, {
                    data: base64Data,
                    format: format,
                    width: canvas.width,
                    height: canvas.height
                }, null);
            }).catch(function(err) {
                sendResult(callbackId, null, err.message || 'Screenshot capture failed');
            });
        });
    }

    /**
     * Send screenshot result back to Python via IPC
     */
    function sendResult(callbackId, result, error) {
        var payload = {
            type: 'screenshot_result',
            callback_id: callbackId,
            result: result,
            error: error
        };

        try {
            if (window.ipc && window.ipc.postMessage) {
                window.ipc.postMessage(JSON.stringify(payload));
            } else if (window.webkit && window.webkit.messageHandlers && window.webkit.messageHandlers.ipc) {
                window.webkit.messageHandlers.ipc.postMessage(JSON.stringify(payload));
            } else {
                console.error('[AuroraView] No IPC channel available for screenshot result');
            }
        } catch (e) {
            console.error('[AuroraView] Failed to send screenshot result:', e);
        }
    }

    // Expose screenshot API on window.auroraview
    if (!window.auroraview) {
        window.auroraview = {};
    }

    window.auroraview._screenshot = {
        capture: captureScreenshot,
        isLoaded: function() { return html2canvasLoaded; },
        preload: function() { loadHtml2Canvas(function() {}); }
    };

    // Also expose as a direct method for convenience
    window.auroraview.screenshot = function(options) {
        return new Promise(function(resolve, reject) {
            var callbackId = Date.now() + '_' + Math.random().toString(36).substr(2, 9);

            // Set up one-time listener for result
            var handler = function(event) {
                var data = event.detail || event.data;
                if (data && data.type === 'screenshot_result' && data.callback_id === callbackId) {
                    window.removeEventListener('auroraview_screenshot_result', handler);
                    if (data.error) {
                        reject(new Error(data.error));
                    } else {
                        resolve(data.result);
                    }
                }
            };

            window.addEventListener('auroraview_screenshot_result', handler);

            // Capture with callback
            options = options || {};
            options.callbackId = callbackId;
            captureScreenshot(options);
        });
    };

})();
