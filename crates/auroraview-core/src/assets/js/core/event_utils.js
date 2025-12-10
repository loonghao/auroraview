/**
 * AuroraView Event Utilities
 *
 * Provides utility functions for event handling including debounce and throttle.
 *
 * @module event_utils
 */

(function() {
    'use strict';

    // Ensure auroraview namespace exists
    if (!window.auroraview) {
        window.auroraview = {};
    }

    /**
     * Event utilities namespace
     */
    window.auroraview.utils = window.auroraview.utils || {};

    /**
     * Creates a debounced function that delays invoking func until after wait
     * milliseconds have elapsed since the last time the debounced function was invoked.
     *
     * @param {Function} func - The function to debounce
     * @param {number} wait - The number of milliseconds to delay
     * @param {Object} [options] - Options object
     * @param {boolean} [options.leading=false] - Invoke on the leading edge
     * @param {boolean} [options.trailing=true] - Invoke on the trailing edge
     * @returns {Function} The debounced function with cancel() and flush() methods
     *
     * @example
     * // Debounce resize handler
     * var debouncedResize = auroraview.utils.debounce(function() {
     *     console.log('Resized!');
     * }, 250);
     * window.addEventListener('resize', debouncedResize);
     *
     * // Cancel pending invocation
     * debouncedResize.cancel();
     */
    window.auroraview.utils.debounce = function(func, wait, options) {
        var timeout = null;
        var lastArgs = null;
        var lastThis = null;
        var result;
        var lastCallTime;
        var lastInvokeTime = 0;

        options = options || {};
        var leading = options.leading === true;
        var trailing = options.trailing !== false;
        var maxWait = options.maxWait;
        var maxing = typeof maxWait === 'number';

        if (maxing) {
            maxWait = Math.max(maxWait, wait);
        }

        function invokeFunc(time) {
            var args = lastArgs;
            var thisArg = lastThis;
            lastArgs = lastThis = null;
            lastInvokeTime = time;
            result = func.apply(thisArg, args);
            return result;
        }

        function leadingEdge(time) {
            lastInvokeTime = time;
            timeout = setTimeout(timerExpired, wait);
            return leading ? invokeFunc(time) : result;
        }

        function remainingWait(time) {
            var timeSinceLastCall = time - lastCallTime;
            var timeSinceLastInvoke = time - lastInvokeTime;
            var timeWaiting = wait - timeSinceLastCall;

            return maxing
                ? Math.min(timeWaiting, maxWait - timeSinceLastInvoke)
                : timeWaiting;
        }

        function shouldInvoke(time) {
            var timeSinceLastCall = time - lastCallTime;
            var timeSinceLastInvoke = time - lastInvokeTime;

            return (
                lastCallTime === undefined ||
                timeSinceLastCall >= wait ||
                timeSinceLastCall < 0 ||
                (maxing && timeSinceLastInvoke >= maxWait)
            );
        }

        function timerExpired() {
            var time = Date.now();
            if (shouldInvoke(time)) {
                return trailingEdge(time);
            }
            timeout = setTimeout(timerExpired, remainingWait(time));
        }

        function trailingEdge(time) {
            timeout = null;
            if (trailing && lastArgs) {
                return invokeFunc(time);
            }
            lastArgs = lastThis = null;
            return result;
        }

        function cancel() {
            if (timeout !== null) {
                clearTimeout(timeout);
            }
            lastInvokeTime = 0;
            lastArgs = lastCallTime = lastThis = timeout = null;
        }

        function flush() {
            return timeout === null ? result : trailingEdge(Date.now());
        }

        function pending() {
            return timeout !== null;
        }

        function debounced() {
            var time = Date.now();
            var isInvoking = shouldInvoke(time);

            lastArgs = arguments;
            lastThis = this;
            lastCallTime = time;

            if (isInvoking) {
                if (timeout === null) {
                    return leadingEdge(lastCallTime);
                }
                if (maxing) {
                    clearTimeout(timeout);
                    timeout = setTimeout(timerExpired, wait);
                    return invokeFunc(lastCallTime);
                }
            }
            if (timeout === null) {
                timeout = setTimeout(timerExpired, wait);
            }
            return result;
        }

        debounced.cancel = cancel;
        debounced.flush = flush;
        debounced.pending = pending;

        return debounced;
    };

    /**
     * Creates a throttled function that only invokes func at most once per every
     * wait milliseconds.
     *
     * @param {Function} func - The function to throttle
     * @param {number} wait - The number of milliseconds to throttle invocations to
     * @param {Object} [options] - Options object
     * @param {boolean} [options.leading=true] - Invoke on the leading edge
     * @param {boolean} [options.trailing=true] - Invoke on the trailing edge
     * @returns {Function} The throttled function with cancel() and flush() methods
     *
     * @example
     * // Throttle scroll handler
     * var throttledScroll = auroraview.utils.throttle(function() {
     *     console.log('Scrolled!');
     * }, 100);
     * window.addEventListener('scroll', throttledScroll);
     */
    window.auroraview.utils.throttle = function(func, wait, options) {
        options = options || {};
        var leading = options.leading !== false;
        var trailing = options.trailing !== false;

        return window.auroraview.utils.debounce(func, wait, {
            leading: leading,
            trailing: trailing,
            maxWait: wait
        });
    };

    /**
     * Creates a function that is restricted to invoking func once.
     * Repeat calls to the function return the value of the first invocation.
     *
     * @param {Function} func - The function to restrict
     * @returns {Function} The restricted function
     *
     * @example
     * var initialize = auroraview.utils.once(function() {
     *     console.log('Initialized!');
     *     return { ready: true };
     * });
     * initialize(); // logs 'Initialized!' and returns { ready: true }
     * initialize(); // returns { ready: true } without logging
     */
    window.auroraview.utils.once = function(func) {
        var called = false;
        var result;

        return function() {
            if (!called) {
                called = true;
                result = func.apply(this, arguments);
            }
            return result;
        };
    };

    /**
     * Wraps an event handler with debounce
     *
     * @param {string} event - Event name
     * @param {Function} handler - Event handler
     * @param {number} wait - Debounce wait time in ms
     * @returns {Function} Unsubscribe function
     */
    window.auroraview.onDebounced = function(event, handler, wait) {
        var debouncedHandler = window.auroraview.utils.debounce(handler, wait);
        window.auroraview.on(event, debouncedHandler);

        return function() {
            debouncedHandler.cancel();
            // Note: auroraview.off would need to be implemented for full cleanup
        };
    };

    /**
     * Wraps an event handler with throttle
     *
     * @param {string} event - Event name
     * @param {Function} handler - Event handler
     * @param {number} wait - Throttle wait time in ms
     * @returns {Function} Unsubscribe function
     */
    window.auroraview.onThrottled = function(event, handler, wait) {
        var throttledHandler = window.auroraview.utils.throttle(handler, wait);
        window.auroraview.on(event, throttledHandler);

        return function() {
            throttledHandler.cancel();
        };
    };

    console.log('[AuroraView] Event utilities initialized (debounce, throttle, once)');
})();
