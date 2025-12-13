/**
 * AuroraView Command Bridge
 * 
 * Provides RPC-style command invocation from JavaScript to Python.
 * Inspired by Tauri's invoke system.
 * 
 * Usage in JavaScript:
 *   // Invoke a Python command
 *   const result = await window.auroraview.invoke("greet", {name: "Alice"});
 *   console.log(result);  // "Hello, Alice!"
 *   
 *   // With error handling
 *   try {
 *     const data = await window.auroraview.invoke("fetch_data", {id: 123});
 *   } catch (error) {
 *     console.error("Command failed:", error);
 *   }
 */

(function() {
    'use strict';
    
    // Pending invoke calls
    const _pendingInvokes = new Map();
    let _invokeIdCounter = 0;
    
    /**
     * Generate unique invoke ID
     */
    function generateInvokeId() {
        return `invoke_${++_invokeIdCounter}_${Date.now()}`;
    }
    
    /**
     * Invoke a Python command
     * @param {string} command - Command name
     * @param {object} args - Command arguments
     * @returns {Promise} Promise that resolves with command result
     */
    function invoke(command, args) {
        return new Promise(function(resolve, reject) {
            const invokeId = generateInvokeId();
            
            // Store pending invoke
            _pendingInvokes.set(invokeId, { resolve: resolve, reject: reject });
            
            // Send invoke request to Python
            if (window.auroraview && window.auroraview.emit) {
                window.auroraview.emit('__invoke__', {
                    id: invokeId,
                    command: command,
                    args: args || {}
                });
            } else {
                reject(new Error('AuroraView bridge not available'));
                _pendingInvokes.delete(invokeId);
            }
            
            // Timeout after 30 seconds
            setTimeout(function() {
                if (_pendingInvokes.has(invokeId)) {
                    _pendingInvokes.delete(invokeId);
                    reject(new Error('Invoke timeout: ' + command));
                }
            }, 30000);
        });
    }
    
    /**
     * Custom error class for command failures
     */
    class CommandError extends Error {
        constructor(code, message, details) {
            super(message);
            this.name = 'CommandError';
            this.code = code;
            this.details = details || {};
        }
    }

    /**
     * Handle invoke response from Python
     */
    function handleInvokeResponse(data) {
        if (!data || typeof data !== 'object') return;

        const invokeId = data.id;
        const pending = _pendingInvokes.get(invokeId);

        if (!pending) {
            console.warn('[AuroraView] Unknown invoke response:', invokeId);
            return;
        }

        _pendingInvokes.delete(invokeId);

        if (data.error) {
            // Handle structured error from Python
            if (typeof data.error === 'object') {
                pending.reject(new CommandError(
                    data.error.code || 'UNKNOWN',
                    data.error.message || 'Unknown error',
                    data.error.details
                ));
            } else {
                // Legacy string error
                pending.reject(new Error(data.error));
            }
        } else {
            pending.resolve(data.result);
        }
    }
    
    /**
     * Handle command registration notification
     */
    function handleCommandRegistered(data) {
        if (!data || typeof data !== 'object') return;
        console.log('[AuroraView] Command registered:', data.name, 'params:', data.params);
    }
    
    // Attach to auroraview object
    function attachToAuroraView() {
        if (window.auroraview) {
            // Only set invoke if not already defined (event_bridge.js defines the correct one)
            // The invoke in event_bridge.js uses window.ipc.postMessage for plugin commands
            // This invoke uses emit('__invoke__') for Python command invocation
            // They serve different purposes, so we use a different name for this one
            if (!window.auroraview.invokeCommand) {
                window.auroraview.invokeCommand = invoke;
            }
            window.auroraview.on('__invoke_response__', handleInvokeResponse);
            window.auroraview.on('__command_registered__', handleCommandRegistered);
            console.log('[AuroraView] Command bridge initialized');
        }
    }
    
    // Try to attach immediately or wait
    if (window.auroraview) {
        attachToAuroraView();
    } else {
        // Wait for auroraview to be available
        const originalDescriptor = Object.getOwnPropertyDescriptor(window, 'auroraview');
        if (!originalDescriptor || originalDescriptor.configurable) {
            let _auroraview = null;
            Object.defineProperty(window, 'auroraview', {
                configurable: true,
                get: function() { return _auroraview; },
                set: function(val) {
                    _auroraview = val;
                    attachToAuroraView();
                }
            });
        }
    }
})();

