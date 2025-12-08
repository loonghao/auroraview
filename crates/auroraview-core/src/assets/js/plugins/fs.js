/**
 * AuroraView File System Plugin API
 * 
 * Provides native file system operations accessible from JavaScript.
 * All paths are validated against the configured scope for security.
 * 
 * Usage:
 *   const content = await auroraview.fs.readFile('/path/to/file.txt');
 *   await auroraview.fs.writeFile('/path/to/file.txt', 'Hello World');
 *   const files = await auroraview.fs.readDir('/path/to/dir');
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
     * File System API
     */
    const fs = {
        /**
         * Read a file as text
         * @param {string} path - File path
         * @param {string} [encoding='utf-8'] - Text encoding
         * @returns {Promise<string>} File content
         */
        async readFile(path, encoding) {
            return invokePlugin('fs', 'read_file', { path, encoding });
        },
        
        /**
         * Read a file as binary (base64 encoded)
         * @param {string} path - File path
         * @returns {Promise<string>} Base64 encoded content
         */
        async readFileBinary(path) {
            return invokePlugin('fs', 'read_file_binary', { path });
        },
        
        /**
         * Read a file as ArrayBuffer
         * @param {string} path - File path
         * @returns {Promise<ArrayBuffer>} File content as ArrayBuffer
         */
        async readFileBuffer(path) {
            const base64 = await this.readFileBinary(path);
            const binary = atob(base64);
            const bytes = new Uint8Array(binary.length);
            for (let i = 0; i < binary.length; i++) {
                bytes[i] = binary.charCodeAt(i);
            }
            return bytes.buffer;
        },
        
        /**
         * Write text to a file
         * @param {string} path - File path
         * @param {string} contents - Content to write
         * @param {boolean} [append=false] - Append instead of overwrite
         */
        async writeFile(path, contents, append) {
            return invokePlugin('fs', 'write_file', { 
                path, 
                contents, 
                append: append || false 
            });
        },
        
        /**
         * Write binary data to a file
         * @param {string} path - File path
         * @param {ArrayBuffer|Uint8Array} contents - Binary content
         * @param {boolean} [append=false] - Append instead of overwrite
         */
        async writeFileBinary(path, contents, append) {
            let bytes;
            if (contents instanceof ArrayBuffer) {
                bytes = Array.from(new Uint8Array(contents));
            } else if (contents instanceof Uint8Array) {
                bytes = Array.from(contents);
            } else {
                throw new Error('contents must be ArrayBuffer or Uint8Array');
            }
            
            return invokePlugin('fs', 'write_file_binary', { 
                path, 
                contents: bytes, 
                append: append || false 
            });
        },
        
        /**
         * Read directory contents
         * @param {string} path - Directory path
         * @param {boolean} [recursive=false] - Read recursively
         * @returns {Promise<Array>} Array of directory entries
         */
        async readDir(path, recursive) {
            return invokePlugin('fs', 'read_dir', { 
                path, 
                recursive: recursive || false 
            });
        },
        
        /**
         * Create a directory
         * @param {string} path - Directory path
         * @param {boolean} [recursive=true] - Create parent directories
         */
        async createDir(path, recursive) {
            return invokePlugin('fs', 'create_dir', { 
                path, 
                recursive: recursive !== false 
            });
        },
        
        /**
         * Remove a file or directory
         * @param {string} path - Path to remove
         * @param {boolean} [recursive=false] - Remove recursively
         */
        async remove(path, recursive) {
            return invokePlugin('fs', 'remove', { 
                path, 
                recursive: recursive || false 
            });
        },
        
        /**
         * Copy a file or directory
         * @param {string} from - Source path
         * @param {string} to - Destination path
         */
        async copy(from, to) {
            return invokePlugin('fs', 'copy', { from, to });
        },
        
        /**
         * Rename/move a file or directory
         * @param {string} from - Source path
         * @param {string} to - Destination path
         */
        async rename(from, to) {
            return invokePlugin('fs', 'rename', { from, to });
        },

        /**
         * Check if a path exists
         * @param {string} path - Path to check
         * @returns {Promise<boolean>} True if path exists
         */
        async exists(path) {
            const result = await invokePlugin('fs', 'exists', { path });
            return result && result.exists;
        },

        /**
         * Get file or directory statistics
         * @param {string} path - Path to stat
         * @returns {Promise<object>} File statistics
         */
        async stat(path) {
            return invokePlugin('fs', 'stat', { path });
        }
    };
    
    // Attach to auroraview object
    function attachToAuroraView() {
        if (window.auroraview) {
            window.auroraview.fs = fs;
            console.log('[AuroraView] File system plugin initialized');
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
