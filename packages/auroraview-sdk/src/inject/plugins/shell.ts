/**
 * AuroraView Shell Plugin API
 *
 * Provides shell command execution and file/URL opening from JavaScript.
 *
 * @module shell
 */

import { invokePlugin, initPlugin } from './utils';

interface ExecuteOptions {
  cwd?: string;
  env?: Record<string, string>;
}

interface ExecuteResult {
  code: number | null;
  stdout: string;
  stderr: string;
}

interface SpawnResult {
  success: boolean;
  pid: number;
}

(function () {
  'use strict';

  const shell = {
    /**
     * Open a URL in the default browser
     */
    async open(url: string): Promise<void> {
      return invokePlugin('shell', 'open', { path: url });
    },

    /**
     * Open a file/folder with the default application
     */
    async openPath(path: string): Promise<void> {
      return invokePlugin('shell', 'open_path', { path });
    },

    /**
     * Show a file in its parent folder (reveal in file manager)
     */
    async showInFolder(path: string): Promise<void> {
      return invokePlugin('shell', 'show_in_folder', { path });
    },

    /**
     * Execute a command and wait for result
     */
    async execute(
      command: string,
      args?: string[],
      options?: ExecuteOptions
    ): Promise<ExecuteResult> {
      return invokePlugin('shell', 'execute', {
        command,
        args: args || [],
        cwd: options?.cwd,
        env: options?.env,
      });
    },

    /**
     * Spawn a detached process (doesn't wait for completion)
     */
    async spawn(
      command: string,
      args?: string[],
      options?: ExecuteOptions
    ): Promise<SpawnResult> {
      return invokePlugin('shell', 'spawn', {
        command,
        args: args || [],
        cwd: options?.cwd,
        env: options?.env,
      });
    },

    /**
     * Find an executable in PATH
     */
    async which(command: string): Promise<string | null> {
      const result = await invokePlugin<{ path: string | null }>('shell', 'which', { command });
      return result.path || null;
    },

    /**
     * Get environment variable
     */
    async getEnv(name: string): Promise<string | null> {
      const result = await invokePlugin<{ value: string | null }>('shell', 'get_env', { name });
      return result.value || null;
    },

    /**
     * Get all environment variables
     */
    async getEnvAll(): Promise<Record<string, string>> {
      const result = await invokePlugin<{ env: Record<string, string> }>('shell', 'get_env_all', {});
      return result.env || {};
    },
  };

  initPlugin('shell', shell);
})();
