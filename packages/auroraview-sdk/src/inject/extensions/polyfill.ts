/**
 * Chrome Extension API Polyfill
 *
 * This module creates a complete chrome.* API polyfill for running
 * Chrome extensions inside AuroraView WebView.
 */

import { createNativeApiCaller } from './native_api';
import {
  createStorageApi,
  createRuntimeApi,
  createTabsApi,
  createSidePanelApi,
  createActionApi,
  createScriptingApi,
  createContextMenusApi,
  createNotificationsApi,
  createAlarmsApi,
  createWebRequestApi,
  createI18nApi,
  createExtensionApi,
  createWindowsApi,
  createCommandsApi,
  createPermissionsApi,
  createIdentityApi,
  createDeclarativeNetRequestApi,
  createOffscreenApi,
  createBookmarksApi,
  createHistoryApi,
  createDownloadsApi,
  createCookiesApi,
  createTopSitesApi,
  createOmniboxApi,
  createSearchApi,
  createSessionsApi,
  createTtsApi,
  createBrowsingDataApi,
  createIdleApi,
  createPowerApi,
  createTabGroupsApi,
  createManagementApi,
  createFontSettingsApi,
} from './apis';

export interface PolyfillConfig {
  extensionId: string;
  extensionPath: string;
  manifest?: any;
  messages?: Record<string, { message: string; placeholders?: Record<string, { content: string }> }>;
}

/**
 * Create the chrome.* API polyfill
 */
export function createChromePolyfill(config: PolyfillConfig) {
  const { extensionId, extensionPath, manifest = {}, messages = {} } = config;
  const callNativeApi = createNativeApiCaller(extensionId);

  // Last error tracking
  let lastError: { message: string } | null = null;

  const chrome = {
    // Core APIs
    runtime: createRuntimeApi(callNativeApi, extensionId, manifest, () => lastError, (err) => { lastError = err; }),
    storage: createStorageApi(callNativeApi),
    tabs: createTabsApi(callNativeApi),
    windows: createWindowsApi(callNativeApi),

    // UI APIs
    action: createActionApi(callNativeApi),
    sidePanel: createSidePanelApi(callNativeApi),
    contextMenus: createContextMenusApi(callNativeApi),
    notifications: createNotificationsApi(callNativeApi),
    omnibox: createOmniboxApi(callNativeApi),

    // Content & Scripting
    scripting: createScriptingApi(callNativeApi),
    webRequest: createWebRequestApi(callNativeApi),
    declarativeNetRequest: createDeclarativeNetRequestApi(callNativeApi),

    // Extension utilities
    extension: createExtensionApi(callNativeApi, extensionId, extensionPath),
    i18n: createI18nApi(callNativeApi, messages),
    commands: createCommandsApi(callNativeApi),
    permissions: createPermissionsApi(callNativeApi),
    identity: createIdentityApi(callNativeApi, extensionId),
    management: createManagementApi(callNativeApi),

    // Data & State
    alarms: createAlarmsApi(callNativeApi),
    bookmarks: createBookmarksApi(callNativeApi),
    history: createHistoryApi(callNativeApi),
    downloads: createDownloadsApi(callNativeApi),
    cookies: createCookiesApi(callNativeApi),
    sessions: createSessionsApi(callNativeApi),
    browsingData: createBrowsingDataApi(callNativeApi),
    topSites: createTopSitesApi(callNativeApi),

    // Advanced
    offscreen: createOffscreenApi(callNativeApi),
    search: createSearchApi(callNativeApi),
    tts: createTtsApi(callNativeApi),
    idle: createIdleApi(callNativeApi),
    power: createPowerApi(callNativeApi),
    tabGroups: createTabGroupsApi(callNativeApi),
    fontSettings: createFontSettingsApi(callNativeApi),
  };

  return chrome;
}

/**
 * Install the chrome.* polyfill globally
 */
export function installChromePolyfill(config: PolyfillConfig): void {
  const chrome = createChromePolyfill(config);

  // Install as window.chrome
  (window as any).chrome = chrome;

  // Also install as window.browser for Firefox compatibility
  (window as any).browser = chrome;

  console.log(`[AuroraView] Chrome API polyfill installed for extension: ${config.extensionId}`);
}
