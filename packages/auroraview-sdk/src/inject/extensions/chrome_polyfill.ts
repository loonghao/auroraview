/**
 * Chrome Extension Polyfill Entry Point
 *
 * This file is the entry point for building the Chrome Extension polyfill
 * that gets injected into extension iframes in AuroraView WebView.
 *
 * Build output: crates/auroraview-extensions/src/assets/js/chrome_polyfill.js
 */

import { installChromePolyfill, type PolyfillConfig } from './polyfill';

declare global {
  interface Window {
    __AURORAVIEW__?: boolean;
    __EXTENSION_ID__?: string;
    __EXTENSION_PATH__?: string;
    __EXTENSION_MANIFEST__?: any;
    __EXTENSION_MESSAGES__?: Record<string, { message: string; placeholders?: Record<string, { content: string }> }>;
  }
}

// Auto-install polyfill when loaded in extension context
(function () {
  'use strict';

  // Only install in AuroraView extension context
  if (!window.__AURORAVIEW__ || !window.__EXTENSION_ID__) {
    return;
  }

  const config: PolyfillConfig = {
    extensionId: window.__EXTENSION_ID__,
    extensionPath: window.__EXTENSION_PATH__ || '',
    manifest: window.__EXTENSION_MANIFEST__ || {},
    messages: window.__EXTENSION_MESSAGES__ || {},
  };

  installChromePolyfill(config);
})();
