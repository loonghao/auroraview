/**
 * AuroraTest Callback Bridge
 *
 * This module provides a callback mechanism for AuroraTest framework
 * to receive JavaScript evaluation results asynchronously.
 *
 * @module test_callback
 */

(function() {
    'use strict';

    // Prevent double initialization
    if (window.__auroratest_callback) {
        console.log('[AuroraTest] Callback bridge already initialized');
        return;
    }

    /**
     * Send JavaScript evaluation result back to Python
     *
     * This function is called by Page.evaluate() to return results
     * from JavaScript execution to the Python test framework.
     *
     * @param {string} callbackId - Unique identifier for the callback
     * @param {string} resultJson - JSON-encoded result object
     */
    window.__auroratest_callback = function(callbackId, resultJson) {
        try {
            // Send result via IPC to Python
            var payload = {
                type: 'auroratest_callback',
                callback_id: callbackId,
                result: resultJson
            };
            window.ipc.postMessage(JSON.stringify(payload));
        } catch (e) {
            console.error('[AuroraTest] Failed to send callback result:', e);
        }
    };

    console.log('[AuroraTest] Callback bridge initialized');
})();
