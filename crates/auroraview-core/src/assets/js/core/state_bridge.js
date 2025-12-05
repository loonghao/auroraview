/**
 * AuroraView State Bridge
 * 
 * Provides reactive shared state between Python and JavaScript.
 * Inspired by PyWebView's state mechanism.
 * 
 * Usage in JavaScript:
 *   // Read state
 *   const user = window.auroraview.state.user;
 *   
 *   // Write state (auto-syncs to Python)
 *   window.auroraview.state.theme = "dark";
 *   
 *   // Subscribe to changes
 *   window.auroraview.state.onChange((key, value, source) => {
 *     console.log(`${key} changed to ${value} from ${source}`);
 *   });
 */

(function() {
    'use strict';
    
    // Internal state storage
    const _stateData = {};
    const _changeHandlers = [];
    
    /**
     * Notify all change handlers
     */
    function notifyHandlers(key, value, source) {
        _changeHandlers.forEach(handler => {
            try {
                handler(key, value, source);
            } catch (e) {
                console.error('[AuroraView State] Handler error:', e);
            }
        });
    }
    
    /**
     * Send state update to Python
     */
    function sendToPython(key, value) {
        if (window.auroraview && window.auroraview.emit) {
            window.auroraview.emit('__state_update__', { key: key, value: value });
        }
    }
    
    /**
     * Create a reactive proxy for state object
     */
    function createStateProxy() {
        return new Proxy(_stateData, {
            get: function(target, prop) {
                if (prop === 'onChange') {
                    return function(handler) {
                        _changeHandlers.push(handler);
                        return function() {
                            const idx = _changeHandlers.indexOf(handler);
                            if (idx > -1) _changeHandlers.splice(idx, 1);
                        };
                    };
                }
                if (prop === 'offChange') {
                    return function(handler) {
                        const idx = _changeHandlers.indexOf(handler);
                        if (idx > -1) _changeHandlers.splice(idx, 1);
                    };
                }
                if (prop === 'toJSON') {
                    return function() { return Object.assign({}, target); };
                }
                if (prop === 'keys') {
                    return function() { return Object.keys(target); };
                }
                return target[prop];
            },
            set: function(target, prop, value) {
                const oldValue = target[prop];
                target[prop] = value;
                
                // Only sync if value actually changed
                if (oldValue !== value) {
                    sendToPython(prop, value);
                    notifyHandlers(prop, value, 'javascript');
                }
                return true;
            },
            deleteProperty: function(target, prop) {
                if (prop in target) {
                    delete target[prop];
                    sendToPython(prop, undefined);
                    notifyHandlers(prop, undefined, 'javascript');
                }
                return true;
            }
        });
    }
    
    // Create the state proxy
    const stateProxy = createStateProxy();
    
    // Handle sync messages from Python
    function handleStateSync(data) {
        if (!data || typeof data !== 'object') return;
        
        switch (data.type) {
            case 'set':
                _stateData[data.key] = data.value;
                notifyHandlers(data.key, data.value, 'python');
                break;
                
            case 'delete':
                delete _stateData[data.key];
                notifyHandlers(data.key, undefined, 'python');
                break;
                
            case 'batch':
                if (data.data && typeof data.data === 'object') {
                    Object.entries(data.data).forEach(([key, value]) => {
                        _stateData[key] = value;
                        notifyHandlers(key, value, 'python');
                    });
                }
                break;
                
            case 'full':
                // Clear and replace all state
                Object.keys(_stateData).forEach(key => delete _stateData[key]);
                if (data.data && typeof data.data === 'object') {
                    Object.assign(_stateData, data.data);
                    Object.entries(data.data).forEach(([key, value]) => {
                        notifyHandlers(key, value, 'python');
                    });
                }
                break;
                
            case 'clear':
                const keys = Object.keys(_stateData);
                keys.forEach(key => {
                    delete _stateData[key];
                    notifyHandlers(key, undefined, 'python');
                });
                break;
        }
    }
    
    // Register state sync handler
    if (window.auroraview) {
        window.auroraview.state = stateProxy;
        window.auroraview.on('__state_sync__', handleStateSync);
    } else {
        // Wait for auroraview to be available
        Object.defineProperty(window, 'auroraview', {
            configurable: true,
            set: function(val) {
                delete window.auroraview;
                window.auroraview = val;
                window.auroraview.state = stateProxy;
                window.auroraview.on('__state_sync__', handleStateSync);
            }
        });
    }
    
    console.log('[AuroraView] State bridge initialized');
})();

