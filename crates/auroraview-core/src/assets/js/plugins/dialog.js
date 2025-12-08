/**
 * AuroraView Dialog Plugin API
 * 
 * Provides native file/folder dialog capabilities accessible from JavaScript.
 * 
 * Usage:
 *   const file = await auroraview.dialog.openFile({ title: 'Select a file' });
 *   const folder = await auroraview.dialog.openFolder({ title: 'Select a folder' });
 *   const savePath = await auroraview.dialog.saveFile({ defaultName: 'document.txt' });
 *   const confirmed = await auroraview.dialog.confirm({ message: 'Are you sure?' });
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
     * Dialog API
     */
    const dialog = {
        /**
         * Open a single file picker dialog
         * @param {object} [options] - Dialog options
         * @param {string} [options.title] - Dialog title
         * @param {string} [options.defaultPath] - Default directory
         * @param {Array<{name: string, extensions: string[]}>} [options.filters] - File filters
         * @returns {Promise<{path: string|null, cancelled: boolean}>} Selected file path or null if cancelled
         */
        async openFile(options) {
            return invokePlugin('dialog', 'open_file', options || {});
        },
        
        /**
         * Open a multiple file picker dialog
         * @param {object} [options] - Dialog options
         * @param {string} [options.title] - Dialog title
         * @param {string} [options.defaultPath] - Default directory
         * @param {Array<{name: string, extensions: string[]}>} [options.filters] - File filters
         * @returns {Promise<{paths: string[], cancelled: boolean}>} Selected file paths or empty array if cancelled
         */
        async openFiles(options) {
            return invokePlugin('dialog', 'open_files', options || {});
        },
        
        /**
         * Open a folder picker dialog
         * @param {object} [options] - Dialog options
         * @param {string} [options.title] - Dialog title
         * @param {string} [options.defaultPath] - Default directory
         * @returns {Promise<{path: string|null, cancelled: boolean}>} Selected folder path or null if cancelled
         */
        async openFolder(options) {
            return invokePlugin('dialog', 'open_folder', options || {});
        },
        
        /**
         * Open a multiple folder picker dialog
         * @param {object} [options] - Dialog options
         * @param {string} [options.title] - Dialog title
         * @param {string} [options.defaultPath] - Default directory
         * @returns {Promise<{paths: string[], cancelled: boolean}>} Selected folder paths or empty array if cancelled
         */
        async openFolders(options) {
            return invokePlugin('dialog', 'open_folders', options || {});
        },
        
        /**
         * Open a save file dialog
         * @param {object} [options] - Dialog options
         * @param {string} [options.title] - Dialog title
         * @param {string} [options.defaultPath] - Default directory
         * @param {string} [options.defaultName] - Default file name
         * @param {Array<{name: string, extensions: string[]}>} [options.filters] - File filters
         * @returns {Promise<{path: string|null, cancelled: boolean}>} Selected save path or null if cancelled
         */
        async saveFile(options) {
            return invokePlugin('dialog', 'save_file', options || {});
        },
        
        /**
         * Show a message dialog
         * @param {object} options - Dialog options
         * @param {string} options.message - Message content
         * @param {string} [options.title] - Dialog title
         * @param {string} [options.level='info'] - Message level: 'info', 'warning', 'error'
         * @param {string} [options.buttons='ok'] - Button type: 'ok', 'ok_cancel', 'yes_no', 'yes_no_cancel'
         * @returns {Promise<{response: string}>} User response: 'ok', 'cancel', 'yes', 'no'
         */
        async message(options) {
            return invokePlugin('dialog', 'message', options);
        },
        
        /**
         * Show a confirmation dialog
         * @param {object} options - Dialog options
         * @param {string} options.message - Confirmation message
         * @param {string} [options.title] - Dialog title
         * @returns {Promise<{confirmed: boolean}>} Whether user confirmed
         */
        async confirm(options) {
            return invokePlugin('dialog', 'confirm', options);
        },
        
        /**
         * Show an info message
         * @param {string} message - Message content
         * @param {string} [title] - Dialog title
         * @returns {Promise<{response: string}>}
         */
        async info(message, title) {
            return this.message({ message, title, level: 'info' });
        },
        
        /**
         * Show a warning message
         * @param {string} message - Message content
         * @param {string} [title] - Dialog title
         * @returns {Promise<{response: string}>}
         */
        async warning(message, title) {
            return this.message({ message, title, level: 'warning' });
        },
        
        /**
         * Show an error message
         * @param {string} message - Message content
         * @param {string} [title] - Dialog title
         * @returns {Promise<{response: string}>}
         */
        async error(message, title) {
            return this.message({ message, title, level: 'error' });
        },
        
        /**
         * Show a yes/no question dialog
         * @param {string} message - Question message
         * @param {string} [title] - Dialog title
         * @returns {Promise<boolean>} True if user clicked Yes
         */
        async ask(message, title) {
            const result = await this.confirm({ message, title });
            return result.confirmed;
        }
    };
    
    // Attach to auroraview object
    function attachToAuroraView() {
        if (window.auroraview) {
            window.auroraview.dialog = dialog;
            console.log('[AuroraView] Dialog plugin initialized');
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
