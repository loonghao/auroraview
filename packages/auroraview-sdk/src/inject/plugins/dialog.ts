/**
 * AuroraView Dialog Plugin API
 *
 * Provides native file/folder dialog capabilities accessible from JavaScript.
 *
 * @module dialog
 */

import { invokePlugin, initPlugin } from './utils';

interface FileFilter {
  name: string;
  extensions: string[];
}

interface OpenOptions {
  title?: string;
  defaultPath?: string;
  filters?: FileFilter[];
}

interface SaveOptions extends OpenOptions {
  defaultName?: string;
}

interface MessageOptions {
  message: string;
  title?: string;
  level?: 'info' | 'warning' | 'error';
  buttons?: 'ok' | 'ok_cancel' | 'yes_no' | 'yes_no_cancel';
}

(function () {
  'use strict';

  const dialog = {
    /**
     * Open a single file picker dialog
     */
    async openFile(options?: OpenOptions): Promise<{ path: string | null; cancelled: boolean }> {
      return invokePlugin('dialog', 'open_file', options || {});
    },

    /**
     * Open a multiple file picker dialog
     */
    async openFiles(options?: OpenOptions): Promise<{ paths: string[]; cancelled: boolean }> {
      return invokePlugin('dialog', 'open_files', options || {});
    },

    /**
     * Open a folder picker dialog
     */
    async openFolder(options?: OpenOptions): Promise<{ path: string | null; cancelled: boolean }> {
      return invokePlugin('dialog', 'open_folder', options || {});
    },

    /**
     * Open a multiple folder picker dialog
     */
    async openFolders(options?: OpenOptions): Promise<{ paths: string[]; cancelled: boolean }> {
      return invokePlugin('dialog', 'open_folders', options || {});
    },

    /**
     * Open a save file dialog
     */
    async saveFile(options?: SaveOptions): Promise<{ path: string | null; cancelled: boolean }> {
      return invokePlugin('dialog', 'save_file', options || {});
    },

    /**
     * Show a message dialog
     */
    async message(options: MessageOptions): Promise<{ response: string }> {
      return invokePlugin('dialog', 'message', options);
    },

    /**
     * Show a confirmation dialog
     */
    async confirm(options: { message: string; title?: string }): Promise<{ confirmed: boolean }> {
      return invokePlugin('dialog', 'confirm', options);
    },

    /**
     * Show an info message
     */
    async info(message: string, title?: string): Promise<{ response: string }> {
      return this.message({ message, title, level: 'info' });
    },

    /**
     * Show a warning message
     */
    async warning(message: string, title?: string): Promise<{ response: string }> {
      return this.message({ message, title, level: 'warning' });
    },

    /**
     * Show an error message
     */
    async error(message: string, title?: string): Promise<{ response: string }> {
      return this.message({ message, title, level: 'error' });
    },

    /**
     * Show a yes/no question dialog
     */
    async ask(message: string, title?: string): Promise<boolean> {
      const result = await this.confirm({ message, title });
      return result.confirmed;
    },
  };

  initPlugin('dialog', dialog);
})();
