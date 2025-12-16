/**
 * AuroraView File System Plugin API
 *
 * Provides native file system operations accessible from JavaScript.
 *
 * @module fs
 */

import { invokePlugin, initPlugin } from './utils';

(function () {
  'use strict';

  const fs = {
    /**
     * Read a file as text
     */
    async readFile(path: string, encoding?: string): Promise<string> {
      return invokePlugin('fs', 'read_file', { path, encoding });
    },

    /**
     * Read a file as binary (base64 encoded)
     */
    async readFileBinary(path: string): Promise<string> {
      return invokePlugin('fs', 'read_file_binary', { path });
    },

    /**
     * Read a file as ArrayBuffer
     */
    async readFileBuffer(path: string): Promise<ArrayBuffer> {
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
     */
    async writeFile(path: string, contents: string, append?: boolean): Promise<void> {
      return invokePlugin('fs', 'write_file', {
        path,
        contents,
        append: append || false,
      });
    },

    /**
     * Write binary data to a file
     */
    async writeFileBinary(
      path: string,
      contents: ArrayBuffer | Uint8Array,
      append?: boolean
    ): Promise<void> {
      let bytes: number[];
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
        append: append || false,
      });
    },

    /**
     * Read directory contents
     */
    async readDir(
      path: string,
      recursive?: boolean
    ): Promise<
      Array<{
        name: string;
        path: string;
        isDir: boolean;
        isFile: boolean;
        size?: number;
        modified?: number;
      }>
    > {
      return invokePlugin('fs', 'read_dir', {
        path,
        recursive: recursive || false,
      });
    },

    /**
     * Create a directory
     */
    async createDir(path: string, recursive?: boolean): Promise<void> {
      return invokePlugin('fs', 'create_dir', {
        path,
        recursive: recursive !== false,
      });
    },

    /**
     * Remove a file or directory
     */
    async remove(path: string, recursive?: boolean): Promise<void> {
      return invokePlugin('fs', 'remove', {
        path,
        recursive: recursive || false,
      });
    },

    /**
     * Copy a file or directory
     */
    async copy(from: string, to: string): Promise<void> {
      return invokePlugin('fs', 'copy', { from, to });
    },

    /**
     * Rename/move a file or directory
     */
    async rename(from: string, to: string): Promise<void> {
      return invokePlugin('fs', 'rename', { from, to });
    },

    /**
     * Check if a path exists
     */
    async exists(path: string): Promise<boolean> {
      const result = await invokePlugin<{ exists: boolean }>('fs', 'exists', { path });
      return result && result.exists;
    },

    /**
     * Get file or directory statistics
     */
    async stat(path: string): Promise<{
      size: number;
      isDir: boolean;
      isFile: boolean;
      isSymlink: boolean;
      modified?: number;
      created?: number;
      accessed?: number;
      readonly?: boolean;
    }> {
      return invokePlugin('fs', 'stat', { path });
    },
  };

  initPlugin('fs', fs);
})();
