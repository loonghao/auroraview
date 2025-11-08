/**
 * AuroraView Bridge - Qt-Style Signal/Slot System for JavaScript
 * 
 * Provides a robust, Qt-inspired API for bidirectional communication
 * between Python and JavaScript in AuroraView applications.
 * 
 * Features:
 * - Automatic initialization waiting
 * - Pending call queueing
 * - Error handling for each slot
 * - Familiar Qt-style API (emit/connect)
 * 
 * Usage:
 *     const bridge = new AuroraViewBridge();
 *     
 *     // Connect slot to signal (Python → JavaScript)
 *     bridge.connect('node_created', (data) => {
 *         console.log('Node created:', data.name);
 *     });
 *     
 *     // Emit signal to Python (JavaScript → Python)
 *     bridge.emit('create_node', { type: 'Grade' });
 * 
 * @class AuroraViewBridge
 */
class AuroraViewBridge {
    constructor() {
        this.ready = false;
        this.pendingCalls = [];
        this.eventHandlers = new Map();
        this.initTimeout = 10000; // 10 seconds timeout
        this.initStartTime = Date.now();
        this.init();
    }
    
    /**
     * Initialize the bridge connection to native AuroraView
     * Waits for window.auroraview to be available
     * @private
     */
    init() {
        // Check timeout
        if (Date.now() - this.initStartTime > this.initTimeout) {
            console.error('[AuroraView] Bridge initialization timeout - window.auroraview not available');
            return;
        }
        
        // Wait for AuroraView bridge to be ready
        if (window.auroraview && window.auroraview.send_event) {
            this.ready = true;
            console.log('[AuroraView] Bridge ready');
            this.registerEventHandlers();
            this.processPendingCalls();
        } else {
            console.log('[AuroraView] Waiting for bridge...');
            setTimeout(() => this.init(), 50);
        }
    }
    
    /**
     * Register all event handlers with the native bridge
     * @private
     */
    registerEventHandlers() {
        this.eventHandlers.forEach((handlers, signal) => {
            window.auroraview.on(signal, (data) => {
                handlers.forEach(handler => {
                    try {
                        handler(data);
                    } catch (e) {
                        console.error(`[AuroraView] Error in slot for signal '${signal}':`, e);
                    }
                });
            });
        });
    }
    
    /**
     * Connect a slot (callback) to a signal (Qt-style)
     * 
     * @param {string} signal - Signal name to listen for
     * @param {function} slot - Callback function to execute when signal is received
     * 
     * @example
     * bridge.connect('node_created', (data) => {
     *     console.log('Node created:', data.name);
     * });
     */
    connect(signal, slot) {
        if (typeof slot !== 'function') {
            console.error(`[AuroraView] Slot must be a function, got ${typeof slot}`);
            return;
        }
        
        if (!this.eventHandlers.has(signal)) {
            this.eventHandlers.set(signal, []);
            
            // Register with native bridge if already ready
            if (this.ready) {
                window.auroraview.on(signal, (data) => {
                    this.eventHandlers.get(signal).forEach(handler => {
                        try {
                            handler(data);
                        } catch (e) {
                            console.error(`[AuroraView] Error in slot for signal '${signal}':`, e);
                        }
                    });
                });
            }
        }
        
        this.eventHandlers.get(signal).push(slot);
        console.log(`[AuroraView] Connected slot to signal: ${signal}`);
    }
    
    /**
     * Emit a signal to Python (Qt-style)
     * 
     * @param {string} signal - Signal name to emit
     * @param {object} data - Data to send with the signal
     * 
     * @example
     * bridge.emit('create_node', { type: 'Grade' });
     */
    emit(signal, data = {}) {
        if (this.ready) {
            try {
                window.auroraview.send_event(signal, data);
                console.log(`[AuroraView] Emitted signal: ${signal}`, data);
            } catch (e) {
                console.error(`[AuroraView] Error emitting signal '${signal}':`, e);
            }
        } else {
            console.log(`[AuroraView] Queuing signal: ${signal}`);
            this.pendingCalls.push({ signal, data });
        }
    }
    
    /**
     * Process all pending calls that were queued before bridge was ready
     * @private
     */
    processPendingCalls() {
        console.log(`[AuroraView] Processing ${this.pendingCalls.length} pending calls`);
        this.pendingCalls.forEach(({ signal, data }) => {
            this.emit(signal, data);
        });
        this.pendingCalls = [];
    }
    
    /**
     * Check if bridge is ready
     * @returns {boolean} True if bridge is initialized and ready
     */
    isReady() {
        return this.ready;
    }
}

