/**
 * AuroraView Channel Bridge
 * 
 * Provides streaming channel support for receiving data from Python.
 * Inspired by Tauri's Channel API.
 * 
 * Usage in JavaScript:
 *   // Subscribe to a channel
 *   const channel = window.auroraview.channel("my_channel");
 *   channel.onMessage((data) => console.log("Received:", data));
 *   channel.onClose(() => console.log("Channel closed"));
 *   
 *   // Or use with invoke
 *   const result = await auroraview.invoke("stream_data", {});
 *   // result.channel_id contains the channel ID
 */

(function() {
    'use strict';
    
    // Active channels
    const _channels = new Map();
    
    /**
     * Channel class for receiving streamed data
     */
    class Channel {
        constructor(id) {
            this.id = id;
            this._messageHandlers = [];
            this._closeHandlers = [];
            this._closed = false;
            this._buffer = [];
        }
        
        /**
         * Register a message handler
         * @param {function} handler - Called with each message
         * @returns {function} Unsubscribe function
         */
        onMessage(handler) {
            this._messageHandlers.push(handler);
            
            // Flush buffer
            this._buffer.forEach(data => {
                try { handler(data); } catch (e) { console.error(e); }
            });
            this._buffer = [];
            
            return () => {
                const idx = this._messageHandlers.indexOf(handler);
                if (idx > -1) this._messageHandlers.splice(idx, 1);
            };
        }
        
        /**
         * Register a close handler
         * @param {function} handler - Called when channel closes
         * @returns {function} Unsubscribe function
         */
        onClose(handler) {
            if (this._closed) {
                try { handler(); } catch (e) { console.error(e); }
                return () => {};
            }
            
            this._closeHandlers.push(handler);
            return () => {
                const idx = this._closeHandlers.indexOf(handler);
                if (idx > -1) this._closeHandlers.splice(idx, 1);
            };
        }
        
        /**
         * Check if channel is closed
         */
        get isClosed() {
            return this._closed;
        }
        
        /**
         * Internal: Handle incoming message
         */
        _handleMessage(data) {
            if (this._messageHandlers.length === 0) {
                this._buffer.push(data);
            } else {
                this._messageHandlers.forEach(handler => {
                    try { handler(data); } catch (e) { console.error(e); }
                });
            }
        }
        
        /**
         * Internal: Handle channel close
         */
        _handleClose() {
            this._closed = true;
            this._closeHandlers.forEach(handler => {
                try { handler(); } catch (e) { console.error(e); }
            });
            this._messageHandlers = [];
            this._closeHandlers = [];
        }
    }
    
    /**
     * Get or create a channel by ID
     */
    function getChannel(channelId) {
        if (!_channels.has(channelId)) {
            _channels.set(channelId, new Channel(channelId));
        }
        return _channels.get(channelId);
    }
    
    /**
     * Handle channel open event from Python
     */
    function handleChannelOpen(data) {
        if (!data || !data.channel_id) return;
        const channel = getChannel(data.channel_id);
        console.log('[AuroraView] Channel opened:', data.channel_id);
    }
    
    /**
     * Handle channel message from Python
     */
    function handleChannelMessage(data) {
        if (!data || !data.channel_id) return;
        const channel = getChannel(data.channel_id);
        channel._handleMessage(data.data);
    }
    
    /**
     * Handle channel close from Python
     */
    function handleChannelClose(data) {
        if (!data || !data.channel_id) return;
        const channel = _channels.get(data.channel_id);
        if (channel) {
            channel._handleClose();
            _channels.delete(data.channel_id);
            console.log('[AuroraView] Channel closed:', data.channel_id);
        }
    }
    
    // Attach to auroraview object
    function attachToAuroraView() {
        if (window.auroraview) {
            window.auroraview.channel = getChannel;
            window.auroraview.on('__channel_open__', handleChannelOpen);
            window.auroraview.on('__channel_message__', handleChannelMessage);
            window.auroraview.on('__channel_close__', handleChannelClose);
            console.log('[AuroraView] Channel bridge initialized');
        }
    }
    
    // Try to attach immediately or wait
    if (window.auroraview) {
        attachToAuroraView();
    } else {
        const check = setInterval(() => {
            if (window.auroraview) {
                clearInterval(check);
                attachToAuroraView();
            }
        }, 10);
        setTimeout(() => clearInterval(check), 5000);
    }
})();

