/**
 * AuroraView Clipboard Plugin API
 * 
 * Provides system clipboard access from JavaScript.
 * 
 * Usage:
 *   const text = await auroraview.clipboard.readText();
 *   await auroraview.clipboard.writeText('Hello World');
 *   await auroraview.clipboard.clear();
 */

(function() {
    'use strict';
    
    /**
     * Invoke a plugin command
     * @param {string} plugin - Plugin name
     * @param {string} command - Command name
     * @param {object} args - Command arguments
     * @returns {Promise} Promise that resolves with command result
     */
    async function invokePlugin(plugin, command, args) {
        if (!window.auroraview || !window.auroraview.invoke) {
            throw new Error('AuroraView bridge not available');
        }
        
        const result = await window.auroraview.invoke(`plugin:${plugin}|${command}`, args || {});
        
        if (result && result.success === false) {
            const error = new Error(result.error || 'Unknown error');
            error.code = result.code || 'UNKNOWN';
            throw error;
        }
        
        return result;
    }
    
    /**
     * Clipboard API
     */
    const clipboard = {
        /**
         * Read text from clipboard
         * @returns {Promise<string>} Clipboard text content
         */
        async readText() {
            const result = await invokePlugin('clipboard', 'read_text', {});
            return result.text || '';
        },
        
        /**
         * Write text to clipboard
         * @param {string} text - Text to write
         * @returns {Promise<void>}
         */
        async writeText(text) {
            return invokePlugin('clipboard', 'write_text', { text });
        },
        
        /**
         * Clear clipboard contents
         * @returns {Promise<void>}
         */
        async clear() {
            return invokePlugin('clipboard', 'clear', {});
        },
        
        /**
         * Check if clipboard has text content
         * @returns {Promise<boolean>}
         */
        async hasText() {
            const result = await invokePlugin('clipboard', 'has_text', {});
            return result.hasText || false;
        },
        
        /**
         * Read image from clipboard as base64
         * @returns {Promise<string|null>} Base64 encoded image or null
         */
        async readImage() {
            try {
                const result = await invokePlugin('clipboard', 'read_image', {});
                return result.image || null;
            } catch (e) {
                // Image read might not be supported
                return null;
            }
        },
        
        /**
         * Write image to clipboard from base64
         * @param {string} base64 - Base64 encoded image
         * @returns {Promise<void>}
         */
        async writeImage(base64) {
            return invokePlugin('clipboard', 'write_image', { image: base64 });
        }
    };
    
    // Attach to auroraview object
    function attachToAuroraView() {
        if (window.auroraview) {
            window.auroraview.clipboard = clipboard;
            console.log('[AuroraView] Clipboard plugin initialized');
        }
    }
    
    // Initialization logic
    if (window.auroraview) {
        attachToAuroraView();
    } else {
        // Wait for auroraview to be available
        const observer = setInterval(function() {
            if (window.auroraview) {
                clearInterval(observer);
                attachToAuroraView();
            }
        }, 10);
        
        // Stop trying after 5 seconds
        setTimeout(function() { clearInterval(observer); }, 5000);
    }
})();
