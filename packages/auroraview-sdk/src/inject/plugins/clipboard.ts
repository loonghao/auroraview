/**
 * AuroraView Clipboard Plugin API
 *
 * Provides system clipboard access from JavaScript.
 *
 * @module clipboard
 */

import { invokePlugin, initPlugin } from './utils';

(function () {
  'use strict';

  const clipboard = {
    /**
     * Read text from clipboard
     */
    async readText(): Promise<string> {
      const result = await invokePlugin<{ text: string }>('clipboard', 'read_text', {});
      return result.text || '';
    },

    /**
     * Write text to clipboard
     */
    async writeText(text: string): Promise<void> {
      return invokePlugin('clipboard', 'write_text', { text });
    },

    /**
     * Clear clipboard contents
     */
    async clear(): Promise<void> {
      return invokePlugin('clipboard', 'clear', {});
    },

    /**
     * Check if clipboard has text content
     */
    async hasText(): Promise<boolean> {
      const result = await invokePlugin<{ hasText: boolean }>('clipboard', 'has_text', {});
      return result.hasText || false;
    },

    /**
     * Read image from clipboard as base64
     */
    async readImage(): Promise<string | null> {
      try {
        const result = await invokePlugin<{ image: string | null }>('clipboard', 'read_image', {});
        return result.image || null;
      } catch {
        // Image read might not be supported
        return null;
      }
    },

    /**
     * Write image to clipboard from base64
     */
    async writeImage(base64: string): Promise<void> {
      return invokePlugin('clipboard', 'write_image', { image: base64 });
    },
  };

  initPlugin('clipboard', clipboard);
})();
