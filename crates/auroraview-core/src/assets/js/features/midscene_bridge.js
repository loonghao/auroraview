/**
 * Midscene AI Testing Bridge
 *
 * This module provides browser-side utilities for AI-powered UI testing.
 * It integrates with Midscene.js concepts while being optimized for AuroraView.
 *
 * Features:
 * - DOM analysis and element location
 * - Screenshot capture (via canvas)
 * - Element interaction helpers
 * - Page state inspection
 *
 * @module midscene_bridge
 */

(function() {
    'use strict';

    // Prevent double initialization
    if (window.__midscene_bridge__) {
        console.log('[Midscene] Bridge already initialized');
        return;
    }

    var DEBUG = !!window.__AURORAVIEW_DEBUG__;

    function debugLog() {
        if (DEBUG) {
            var args = ['[Midscene]'].concat(Array.prototype.slice.call(arguments));
            console.log.apply(console, args);
        }
    }

    /**
     * Midscene Bridge API
     */
    window.__midscene_bridge__ = {
        version: '1.0.0',
        ready: true,

        // ─────────────────────────────────────────────────────────────────
        // Screenshot Capture
        // ─────────────────────────────────────────────────────────────────

        /**
         * Capture screenshot as base64 PNG
         * Uses html2canvas if available, otherwise returns null
         * @returns {Promise<string|null>} Base64 encoded PNG or null
         */
        captureScreenshot: function() {
            return new Promise(function(resolve) {
                if (typeof html2canvas !== 'undefined') {
                    html2canvas(document.body, {
                        useCORS: true,
                        logging: false,
                        scale: window.devicePixelRatio || 1
                    }).then(function(canvas) {
                        resolve(canvas.toDataURL('image/png').split(',')[1]);
                    }).catch(function(err) {
                        console.warn('[Midscene] Screenshot failed:', err);
                        resolve(null);
                    });
                } else {
                    resolve(null);
                }
            });
        },

        // ─────────────────────────────────────────────────────────────────
        // DOM Analysis
        // ─────────────────────────────────────────────────────────────────

        /**
         * Get simplified DOM structure for AI analysis
         * @param {number} maxDepth - Maximum depth to traverse (default: 10)
         * @returns {Object} Simplified DOM tree
         */
        getSimplifiedDOM: function(maxDepth) {
            maxDepth = maxDepth || 10;

            function walk(node, depth) {
                if (depth > maxDepth) return null;
                if (node.nodeType !== 1) return null;

                var tag = node.tagName.toLowerCase();

                // Skip hidden elements
                var style = window.getComputedStyle(node);
                if (style.display === 'none' || style.visibility === 'hidden') {
                    return null;
                }

                var result = { tag: tag };

                // Important attributes
                if (node.id) result.id = node.id;
                if (node.className && typeof node.className === 'string') {
                    result.class = node.className;
                }

                // Accessibility attributes
                var role = node.getAttribute('role');
                if (role) result.role = role;

                var ariaLabel = node.getAttribute('aria-label');
                if (ariaLabel) result.ariaLabel = ariaLabel;

                var placeholder = node.getAttribute('placeholder');
                if (placeholder) result.placeholder = placeholder;

                var title = node.getAttribute('title');
                if (title) result.title = title;

                // Text content (limited)
                var text = node.textContent;
                if (text && text.trim().length > 0 && text.trim().length < 200) {
                    result.text = text.trim();
                }

                // Value for inputs
                if (tag === 'input' || tag === 'textarea' || tag === 'select') {
                    result.value = node.value || '';
                    result.type = node.type || 'text';
                }

                // Bounding rect
                var rect = node.getBoundingClientRect();
                if (rect.width > 0 && rect.height > 0) {
                    result.rect = {
                        x: Math.round(rect.x),
                        y: Math.round(rect.y),
                        width: Math.round(rect.width),
                        height: Math.round(rect.height)
                    };
                }

                // Interactive state
                if (node.disabled) result.disabled = true;
                if (node.readOnly) result.readOnly = true;
                if (node.checked !== undefined) result.checked = node.checked;

                // Children
                var children = [];
                for (var i = 0; i < node.children.length; i++) {
                    var childResult = walk(node.children[i], depth + 1);
                    if (childResult) children.push(childResult);
                }
                if (children.length > 0) result.children = children;

                return result;
            }

            return walk(document.body, 0);
        },

        /**
         * Get all interactive elements on the page
         * @returns {Array} List of interactive elements with their properties
         */
        getInteractiveElements: function() {
            var selectors = [
                'button',
                'a[href]',
                'input',
                'textarea',
                'select',
                '[role="button"]',
                '[role="link"]',
                '[role="checkbox"]',
                '[role="radio"]',
                '[role="tab"]',
                '[role="menuitem"]',
                '[onclick]',
                '[tabindex]:not([tabindex="-1"])'
            ];

            var elements = document.querySelectorAll(selectors.join(','));
            var results = [];

            for (var i = 0; i < elements.length; i++) {
                var el = elements[i];
                var rect = el.getBoundingClientRect();

                // Skip invisible elements
                if (rect.width === 0 || rect.height === 0) continue;

                var style = window.getComputedStyle(el);
                if (style.display === 'none' || style.visibility === 'hidden') continue;

                results.push({
                    tag: el.tagName.toLowerCase(),
                    id: el.id || null,
                    class: el.className || null,
                    text: (el.textContent || '').trim().substring(0, 100),
                    ariaLabel: el.getAttribute('aria-label'),
                    placeholder: el.getAttribute('placeholder'),
                    role: el.getAttribute('role'),
                    type: el.type || null,
                    disabled: el.disabled || false,
                    rect: {
                        x: Math.round(rect.x),
                        y: Math.round(rect.y),
                        width: Math.round(rect.width),
                        height: Math.round(rect.height),
                        centerX: Math.round(rect.x + rect.width / 2),
                        centerY: Math.round(rect.y + rect.height / 2)
                    }
                });
            }

            return results;
        },

        // ─────────────────────────────────────────────────────────────────
        // Element Location
        // ─────────────────────────────────────────────────────────────────

        /**
         * Get element at specific coordinates
         * @param {number} x - X coordinate
         * @param {number} y - Y coordinate
         * @returns {Object|null} Element info or null
         */
        getElementAt: function(x, y) {
            var el = document.elementFromPoint(x, y);
            if (!el) return null;

            var rect = el.getBoundingClientRect();
            return {
                tag: el.tagName.toLowerCase(),
                id: el.id || null,
                class: el.className || null,
                text: (el.textContent || '').trim().substring(0, 100),
                ariaLabel: el.getAttribute('aria-label'),
                rect: {
                    x: Math.round(rect.x),
                    y: Math.round(rect.y),
                    width: Math.round(rect.width),
                    height: Math.round(rect.height)
                }
            };
        },

        /**
         * Find elements matching a text query
         * @param {string} query - Text to search for
         * @returns {Array} Matching elements
         */
        findByText: function(query) {
            var queryLower = query.toLowerCase();
            var results = [];

            // Use TreeWalker for efficient text node traversal
            var walker = document.createTreeWalker(
                document.body,
                NodeFilter.SHOW_TEXT,
                null,
                false
            );

            var node;
            while ((node = walker.nextNode())) {
                var text = node.textContent.trim();
                if (text && text.toLowerCase().indexOf(queryLower) !== -1) {
                    var parent = node.parentElement;
                    if (parent) {
                        var rect = parent.getBoundingClientRect();
                        if (rect.width > 0 && rect.height > 0) {
                            results.push({
                                tag: parent.tagName.toLowerCase(),
                                id: parent.id || null,
                                class: parent.className || null,
                                text: text.substring(0, 100),
                                rect: {
                                    x: Math.round(rect.x),
                                    y: Math.round(rect.y),
                                    width: Math.round(rect.width),
                                    height: Math.round(rect.height),
                                    centerX: Math.round(rect.x + rect.width / 2),
                                    centerY: Math.round(rect.y + rect.height / 2)
                                }
                            });
                        }
                    }
                }
            }

            return results;
        },

        /**
         * Find element by CSS selector
         * @param {string} selector - CSS selector
         * @returns {Object|null} Element info or null
         */
        findBySelector: function(selector) {
            try {
                var el = document.querySelector(selector);
                if (!el) return null;

                var rect = el.getBoundingClientRect();
                return {
                    tag: el.tagName.toLowerCase(),
                    id: el.id || null,
                    class: el.className || null,
                    text: (el.textContent || '').trim().substring(0, 100),
                    rect: {
                        x: Math.round(rect.x),
                        y: Math.round(rect.y),
                        width: Math.round(rect.width),
                        height: Math.round(rect.height),
                        centerX: Math.round(rect.x + rect.width / 2),
                        centerY: Math.round(rect.y + rect.height / 2)
                    }
                };
            } catch (e) {
                console.warn('[Midscene] Invalid selector:', selector);
                return null;
            }
        },

        // ─────────────────────────────────────────────────────────────────
        // Element Interaction
        // ─────────────────────────────────────────────────────────────────

        /**
         * Click at coordinates
         * @param {number} x - X coordinate
         * @param {number} y - Y coordinate
         * @returns {boolean} Success
         */
        clickAt: function(x, y) {
            var el = document.elementFromPoint(x, y);
            if (el) {
                el.click();
                debugLog('Clicked at', x, y, '->', el.tagName);
                return true;
            }
            return false;
        },

        /**
         * Click element by selector
         * @param {string} selector - CSS selector
         * @returns {boolean} Success
         */
        clickSelector: function(selector) {
            try {
                var el = document.querySelector(selector);
                if (el) {
                    el.click();
                    debugLog('Clicked selector:', selector);
                    return true;
                }
            } catch (e) {
                console.warn('[Midscene] Click failed:', e);
            }
            return false;
        },

        /**
         * Type text into focused element or selector
         * @param {string} text - Text to type
         * @param {string} selector - Optional CSS selector
         * @returns {boolean} Success
         */
        typeText: function(text, selector) {
            var el = selector ? document.querySelector(selector) : document.activeElement;

            if (el && (el.tagName === 'INPUT' || el.tagName === 'TEXTAREA')) {
                el.focus();
                el.value = text;
                el.dispatchEvent(new Event('input', { bubbles: true }));
                el.dispatchEvent(new Event('change', { bubbles: true }));
                debugLog('Typed text into', el.tagName);
                return true;
            }
            return false;
        },

        /**
         * Clear and type text
         * @param {string} text - Text to type
         * @param {string} selector - CSS selector
         * @returns {boolean} Success
         */
        fillText: function(text, selector) {
            var el = document.querySelector(selector);
            if (el && (el.tagName === 'INPUT' || el.tagName === 'TEXTAREA')) {
                el.focus();
                el.value = '';
                el.value = text;
                el.dispatchEvent(new Event('input', { bubbles: true }));
                el.dispatchEvent(new Event('change', { bubbles: true }));
                return true;
            }
            return false;
        },

        /**
         * Focus element by selector
         * @param {string} selector - CSS selector
         * @returns {boolean} Success
         */
        focusElement: function(selector) {
            var el = document.querySelector(selector);
            if (el) {
                el.focus();
                return true;
            }
            return false;
        },

        /**
         * Scroll element into view
         * @param {string} selector - CSS selector
         * @returns {boolean} Success
         */
        scrollIntoView: function(selector) {
            var el = document.querySelector(selector);
            if (el) {
                el.scrollIntoView({ behavior: 'smooth', block: 'center' });
                return true;
            }
            return false;
        },

        /**
         * Scroll page by amount
         * @param {number} deltaX - Horizontal scroll
         * @param {number} deltaY - Vertical scroll
         */
        scrollBy: function(deltaX, deltaY) {
            window.scrollBy({
                left: deltaX,
                top: deltaY,
                behavior: 'smooth'
            });
        },

        // ─────────────────────────────────────────────────────────────────
        // Page State
        // ─────────────────────────────────────────────────────────────────

        /**
         * Get current page info
         * @returns {Object} Page information
         */
        getPageInfo: function() {
            return {
                url: window.location.href,
                title: document.title,
                viewport: {
                    width: window.innerWidth,
                    height: window.innerHeight
                },
                scroll: {
                    x: window.scrollX,
                    y: window.scrollY
                },
                documentSize: {
                    width: document.documentElement.scrollWidth,
                    height: document.documentElement.scrollHeight
                },
                readyState: document.readyState,
                activeElement: document.activeElement ? document.activeElement.tagName.toLowerCase() : null
            };
        },

        /**
         * Get page text content
         * @returns {string} Page text
         */
        getPageText: function() {
            return document.body.innerText || '';
        },

        /**
         * Check if element is visible
         * @param {string} selector - CSS selector
         * @returns {boolean} Visibility
         */
        isVisible: function(selector) {
            try {
                var el = document.querySelector(selector);
                if (!el) return false;

                var rect = el.getBoundingClientRect();
                if (rect.width === 0 || rect.height === 0) return false;

                var style = window.getComputedStyle(el);
                if (style.display === 'none') return false;
                if (style.visibility === 'hidden') return false;
                if (style.opacity === '0') return false;

                return true;
            } catch (e) {
                return false;
            }
        },

        /**
         * Wait for selector to appear
         * @param {string} selector - CSS selector
         * @param {number} timeout - Timeout in ms (default: 5000)
         * @returns {Promise<boolean>} Success
         */
        waitForSelector: function(selector, timeout) {
            timeout = timeout || 5000;
            var start = Date.now();

            return new Promise(function(resolve) {
                function check() {
                    if (document.querySelector(selector)) {
                        resolve(true);
                    } else if (Date.now() - start > timeout) {
                        resolve(false);
                    } else {
                        requestAnimationFrame(check);
                    }
                }
                check();
            });
        },

        /**
         * Wait for text to appear on page
         * @param {string} text - Text to wait for
         * @param {number} timeout - Timeout in ms (default: 5000)
         * @returns {Promise<boolean>} Success
         */
        waitForText: function(text, timeout) {
            timeout = timeout || 5000;
            var start = Date.now();
            var textLower = text.toLowerCase();

            return new Promise(function(resolve) {
                function check() {
                    var pageText = (document.body.innerText || '').toLowerCase();
                    if (pageText.indexOf(textLower) !== -1) {
                        resolve(true);
                    } else if (Date.now() - start > timeout) {
                        resolve(false);
                    } else {
                        requestAnimationFrame(check);
                    }
                }
                check();
            });
        }
    };

    // Expose to auroraview namespace if available
    if (window.auroraview) {
        window.auroraview.midscene = window.__midscene_bridge__;
        debugLog('Attached to window.auroraview.midscene');
    }

    console.log('[Midscene] Bridge initialized v' + window.__midscene_bridge__.version);
})();
