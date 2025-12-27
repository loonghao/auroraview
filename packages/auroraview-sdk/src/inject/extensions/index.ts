/**
 * Chrome Extension Polyfill Module
 *
 * This module provides Chrome Extension API polyfill for running
 * Chrome extensions inside AuroraView WebView.
 */

export { createChromePolyfill, installChromePolyfill, type PolyfillConfig } from './polyfill';
export { EventEmitter, createWebRequestEvent } from './event_emitter';
export { callNativeApi, createNativeApiCaller, promisify } from './native_api';
export * from './types';
export * from './apis';
