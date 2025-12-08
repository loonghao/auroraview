/**
 * AuroraView Network Interception Module
 *
 * Provides network request interception and mocking capabilities.
 * This module intercepts fetch and XMLHttpRequest to enable:
 * - Request interception
 * - Response mocking
 * - Network monitoring
 *
 * Note: This is a JavaScript-level interception, not a true network proxy.
 * It intercepts fetch() and XMLHttpRequest calls made by the page.
 */
(function() {
    'use strict';

    // Store original implementations
    var originalFetch = window.fetch;
    var originalXHROpen = XMLHttpRequest.prototype.open;
    var originalXHRSend = XMLHttpRequest.prototype.send;

    // Route handlers: Array of {pattern, handler}
    var routes = [];

    // Request log for monitoring
    var requestLog = [];
    var maxLogSize = 1000;

    /**
     * Check if URL matches a pattern
     * Supports glob patterns: * (any chars), ** (any path segments)
     */
    function matchPattern(url, pattern) {
        if (typeof pattern === 'string') {
            // Convert glob pattern to regex
            var regexStr = pattern
                .replace(/[.+^${}()|[\]\\]/g, '\\$&')  // Escape special chars
                .replace(/\*\*/g, '{{GLOBSTAR}}')      // Temp placeholder for **
                .replace(/\*/g, '[^/]*')               // * matches non-slash chars
                .replace(/\{\{GLOBSTAR\}\}/g, '.*');   // ** matches anything
            var regex = new RegExp('^' + regexStr + '$');
            return regex.test(url);
        } else if (pattern instanceof RegExp) {
            return pattern.test(url);
        }
        return false;
    }

    /**
     * Find matching route for URL
     */
    function findRoute(url, method) {
        for (var i = 0; i < routes.length; i++) {
            var route = routes[i];
            if (matchPattern(url, route.pattern)) {
                if (!route.method || route.method === method) {
                    return route;
                }
            }
        }
        return null;
    }

    /**
     * Create a mock Response object
     */
    function createMockResponse(options) {
        options = options || {};
        var status = options.status || 200;
        var statusText = options.statusText || 'OK';
        var headers = new Headers(options.headers || {});
        var body = options.body || '';

        if (options.json !== undefined) {
            body = JSON.stringify(options.json);
            if (!headers.has('Content-Type')) {
                headers.set('Content-Type', 'application/json');
            }
        }

        return new Response(body, {
            status: status,
            statusText: statusText,
            headers: headers
        });
    }

    /**
     * Log a request
     */
    function logRequest(entry) {
        requestLog.push(entry);
        if (requestLog.length > maxLogSize) {
            requestLog.shift();
        }

        // Notify Python via IPC
        try {
            var payload = {
                type: 'network_request',
                request: entry
            };
            if (window.ipc && window.ipc.postMessage) {
                window.ipc.postMessage(JSON.stringify(payload));
            }
        } catch (e) {
            // Ignore IPC errors
        }
    }

    /**
     * Intercepted fetch implementation
     */
    function interceptedFetch(input, init) {
        var url = typeof input === 'string' ? input : input.url;
        var method = (init && init.method) || 'GET';

        var requestEntry = {
            url: url,
            method: method,
            headers: (init && init.headers) || {},
            body: (init && init.body) || null,
            timestamp: Date.now(),
            type: 'fetch'
        };

        var route = findRoute(url, method);

        if (route && route.handler) {
            // Create route context
            var routeContext = {
                request: {
                    url: url,
                    method: method,
                    headers: requestEntry.headers,
                    postData: requestEntry.body
                },
                fulfill: function(options) {
                    requestEntry.mocked = true;
                    requestEntry.response = {
                        status: options.status || 200,
                        mocked: true
                    };
                    logRequest(requestEntry);
                    return Promise.resolve(createMockResponse(options));
                },
                continue_: function(overrides) {
                    overrides = overrides || {};
                    var newUrl = overrides.url || url;
                    var newInit = Object.assign({}, init || {});
                    if (overrides.method) newInit.method = overrides.method;
                    if (overrides.headers) newInit.headers = overrides.headers;
                    if (overrides.postData) newInit.body = overrides.postData;

                    requestEntry.continued = true;
                    logRequest(requestEntry);
                    return originalFetch(newUrl, newInit);
                },
                abort: function() {
                    requestEntry.aborted = true;
                    logRequest(requestEntry);
                    return Promise.reject(new Error('Request aborted'));
                }
            };

            try {
                var result = route.handler(routeContext);
                if (result && typeof result.then === 'function') {
                    return result;
                }
                // If handler didn't return a promise, continue the request
                return routeContext.continue_();
            } catch (e) {
                console.error('[AuroraView] Route handler error:', e);
                return routeContext.continue_();
            }
        }

        // No matching route, proceed normally
        logRequest(requestEntry);
        return originalFetch(input, init).then(function(response) {
            requestEntry.response = {
                status: response.status,
                statusText: response.statusText
            };
            return response;
        });
    }

    /**
     * Add a route handler
     */
    function addRoute(pattern, handler, options) {
        options = options || {};
        routes.push({
            pattern: pattern,
            handler: handler,
            method: options.method || null
        });
    }

    /**
     * Remove a route handler
     */
    function removeRoute(pattern, handler) {
        routes = routes.filter(function(route) {
            if (handler) {
                return !(route.pattern === pattern && route.handler === handler);
            }
            return route.pattern !== pattern;
        });
    }

    /**
     * Clear all routes
     */
    function clearRoutes() {
        routes = [];
    }

    /**
     * Get request log
     */
    function getRequestLog() {
        return requestLog.slice();
    }

    /**
     * Clear request log
     */
    function clearRequestLog() {
        requestLog = [];
    }

    /**
     * Enable interception
     */
    function enable() {
        window.fetch = interceptedFetch;
        // TODO: Also intercept XMLHttpRequest for older code
    }

    /**
     * Disable interception
     */
    function disable() {
        window.fetch = originalFetch;
        XMLHttpRequest.prototype.open = originalXHROpen;
        XMLHttpRequest.prototype.send = originalXHRSend;
    }

    // Expose network interception API on window.auroraview
    if (!window.auroraview) {
        window.auroraview = {};
    }

    window.auroraview._network = {
        addRoute: addRoute,
        removeRoute: removeRoute,
        clearRoutes: clearRoutes,
        getRequestLog: getRequestLog,
        clearRequestLog: clearRequestLog,
        enable: enable,
        disable: disable
    };

    // Enable by default for testing
    enable();

})();
