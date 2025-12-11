/**
 * AuroraView Shell Plugin API
 * 
 * Provides shell command execution and file/URL opening from JavaScript.
 * 
 * Usage:
 *   await auroraview.shell.open('https://example.com');
 *   await auroraview.shell.openPath('/path/to/file.txt');
 *   await auroraview.shell.showInFolder('/path/to/file.txt');
 *   const result = await auroraview.shell.execute('echo', ['hello']);
 *   const home = await auroraview.shell.getEnv('HOME');
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
     * Shell API
     */
    const shell = {
        /**
         * Open a URL in the default browser
         * @param {string} url - URL to open
         * @returns {Promise<void>}
         */
        async open(url) {
            return invokePlugin('shell', 'open', { path: url });
        },
        
        /**
         * Open a file/folder with the default application
         * @param {string} path - Path to open
         * @returns {Promise<void>}
         */
        async openPath(path) {
            return invokePlugin('shell', 'open_path', { path });
        },
        
        /**
         * Show a file in its parent folder (reveal in file manager)
         * @param {string} path - Path to reveal
         * @returns {Promise<void>}
         */
        async showInFolder(path) {
            return invokePlugin('shell', 'show_in_folder', { path });
        },
        
        /**
         * Execute a command and wait for result
         * @param {string} command - Command to execute
         * @param {string[]} [args=[]] - Command arguments
         * @param {object} [options] - Execution options
         * @param {string} [options.cwd] - Working directory
         * @param {object} [options.env] - Environment variables
         * @returns {Promise<{code: number|null, stdout: string, stderr: string}>}
         */
        async execute(command, args, options) {
            return invokePlugin('shell', 'execute', {
                command,
                args: args || [],
                cwd: options?.cwd,
                env: options?.env
            });
        },
        
        /**
         * Spawn a detached process (doesn't wait for completion)
         * @param {string} command - Command to spawn
         * @param {string[]} [args=[]] - Command arguments
         * @param {object} [options] - Spawn options
         * @param {string} [options.cwd] - Working directory
         * @param {object} [options.env] - Environment variables
         * @returns {Promise<{success: boolean, pid: number}>} Process ID
         */
        async spawn(command, args, options) {
            return invokePlugin('shell', 'spawn', {
                command,
                args: args || [],
                cwd: options?.cwd,
                env: options?.env
            });
        },
        
        /**
         * Find an executable in PATH
         * @param {string} command - Command name to find
         * @returns {Promise<string|null>} Full path or null if not found
         */
        async which(command) {
            const result = await invokePlugin('shell', 'which', { command });
            return result.path || null;
        },
        
        /**
         * Get environment variable
         * @param {string} name - Variable name
         * @returns {Promise<string|null>} Variable value or null
         */
        async getEnv(name) {
            const result = await invokePlugin('shell', 'get_env', { name });
            return result.value || null;
        },
        
        /**
         * Get all environment variables
         * @returns {Promise<object>} Environment variables map
         */
        async getEnvAll() {
            const result = await invokePlugin('shell', 'get_env_all', {});
            return result.env || {};
        }
    };
    
    // Attach to auroraview object
    function attachToAuroraView() {
        if (window.auroraview) {
            window.auroraview.shell = shell;
            console.log('[AuroraView] Shell plugin initialized');
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
