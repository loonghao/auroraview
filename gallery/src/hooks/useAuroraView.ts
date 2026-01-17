/**
 * AuroraView React Hooks for Gallery
 *
 * This file re-exports hooks from @auroraview/sdk and adds Gallery-specific
 * API wrappers for sample management.
 * 
 * Design: Uses Rust native plugins for extension management and Chrome API compatibility.
 * All extension APIs now go through `plugin:extensions|*` commands.
 */

import { useState, useEffect, useCallback } from 'react';
import {
  useAuroraView as useAuroraViewBase,
  useProcessEvents as useProcessEventsBase,
  type ProcessOutput,
  type ProcessExit,
} from '@auroraview/sdk/react';

// Re-export types from SDK
export type { ProcessOutput, ProcessExit } from '@auroraview/sdk/react';

// Gallery-specific types
export interface RunOptions {
  showConsole?: boolean;
}

export interface ApiResult {
  ok: boolean;
  error?: string;
  pid?: number;
  mode?: string;
  message?: string;
}

export interface Sample {
  id: string;
  title: string;
  category: string;
  description: string;
  icon: string;
  source_file: string;
  tags?: string[];
}

export interface DependencyInfo {
  requirements: string[];
  missing: string[];
  needs_install: boolean;
}

export interface DependencyCheckResult extends ApiResult {
  sample_id?: string;
  requirements?: string[];
  missing?: string[];
  needs_install?: boolean;
}

export interface DependencyInstallResult extends ApiResult {
  packages?: string[];
  already_installed?: boolean;
}

export interface DependencyProgress {
  sample_id: string;
  package?: string;
  index?: number;
  total?: number;
  message?: string;
  line?: string;
  phase?: 'starting' | 'installing' | 'complete';
  success?: boolean;
}

export interface DependencyComplete {
  sample_id: string;
  installed: string[];
  message: string;
}

export interface DependencyError {
  sample_id: string;
  package?: string;
  failed?: string[];
  error: string;
  cancelled?: boolean;
}


export interface Category {
  title: string;
  icon: string;
  description: string;
}

export interface ProcessInfo {
  pid: number;
  sampleId: string;
  title: string;
  startTime: number;
}

// Extension types - aligned with Rust ExtensionInfo
export interface ExtensionInfo {
  id: string;
  name: string;
  version: string;
  description: string;
  enabled: boolean;
  sidePanelPath?: string;
  popupPath?: string;
  optionsPage?: string;
  rootDir: string;
  permissions: string[];
  hostPermissions: string[];
  manifest?: Record<string, unknown>;
  installType?: 'admin' | 'development' | 'normal' | 'sideload' | 'other';
  homepageUrl?: string;
  mayDisable?: boolean;
  mayEnable?: boolean;
}

// Installed extension for UI display
export interface InstalledExtension {
  id: string;
  name: string;
  version: string;
  description: string;
  path: string;
  hasSidePanel?: boolean;
  sidePanelPath?: string;
  hasPopup?: boolean;
  popupPath?: string;
  optionsUrl?: string;
  installType?: 'admin' | 'development' | 'normal' | 'sideload' | 'other';
  homepageUrl?: string;
  permissions?: string[];
  hostPermissions?: string[];
}

// Side panel state
export interface SidePanelState {
  isOpen: boolean;
  path?: string;
  options?: {
    path?: string;
    enabled?: boolean;
  };
}

// Action state (toolbar button)
export interface ActionState {
  title?: string;
  badgeText?: string;
  badgeBackgroundColor?: string;
  badgeTextColor?: string;
  popup?: string;
  enabled: boolean;
  icon?: unknown;
}

// Browser extension bridge status (for legacy Python backend)
export interface BrowserExtensionStatus {
  enabled: boolean;
  wsPort: number;
  httpPort: number;
  connectedClients: number;
  isRunning: boolean;
}

// ============================================
// Features API Types
// ============================================

export interface Bookmark {
  id: string;
  url: string;
  title: string;
  favicon?: string;
  folderId?: string;
  createdAt?: string;
}

export interface BookmarkFolder {
  id: string;
  name: string;
  parentId?: string;
  bookmarks: Bookmark[];
}

export interface HistoryEntry {
  id: string;
  url: string;
  title: string;
  visitCount: number;
  lastVisit: string;
  favicon?: string;
}

export interface DownloadItem {
  id: string;
  url: string;
  filename: string;
  path?: string;
  size?: number;
  downloadedBytes: number;
  state: 'pending' | 'downloading' | 'paused' | 'completed' | 'failed' | 'cancelled';
  error?: string;
  startedAt?: string;
  completedAt?: string;
  speed?: number;
}

export interface FeaturesResult<T = unknown> {
  ok: boolean;
  error?: string;
  [key: string]: T | boolean | string | undefined;
}

/**
 * Gallery-specific hook that wraps SDK's useAuroraView with Gallery API methods
 */
export function useAuroraView() {
  const { client, isReady } = useAuroraViewBase();

  // ============================================
  // Sample Management APIs (Python backend)
  // ============================================

  const getSource = useCallback(async (sampleId: string): Promise<string> => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    return client.call<string>('api.get_source', { sample_id: sampleId });
  }, [client]);

  const runSample = useCallback(async (sampleId: string, options?: RunOptions): Promise<ApiResult> => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    return client.call<ApiResult>('api.run_sample', {
      sample_id: sampleId,
      show_console: options?.showConsole ?? false,
    });
  }, [client]);

  const getSamples = useCallback(async (): Promise<Sample[]> => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    return client.call<Sample[]>('api.get_samples');
  }, [client]);

  const getCategories = useCallback(async (): Promise<Record<string, Category>> => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    return client.call<Record<string, Category>>('api.get_categories');
  }, [client]);

  const openUrl = useCallback(async (url: string): Promise<ApiResult> => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    return client.call<ApiResult>('api.open_url', { url });
  }, [client]);

  const openInWebView = useCallback((url: string, title?: string) => {
    const windowName = title ?? 'AuroraView';
    const features = 'width=1024,height=768,menubar=no,toolbar=no,location=yes,status=no';
    window.open(url, windowName, features);
  }, []);

  const killProcess = useCallback(async (pid: number): Promise<ApiResult> => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    return client.call<ApiResult>('api.kill_process', { pid });
  }, [client]);

  const sendToProcess = useCallback(async (pid: number, data: string) => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    return client.call('api.send_to_process', { pid, data });
  }, [client]);

  const listProcesses = useCallback(async () => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    return client.call('api.list_processes');
  }, [client]);

  // ============================================
  // Dependency Management APIs
  // ============================================

  /**
   * Check if a sample has missing dependencies
   */
  const checkDependencies = useCallback(async (sampleId: string): Promise<DependencyCheckResult> => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    return client.call<DependencyCheckResult>('api.check_dependencies', { sample_id: sampleId });
  }, [client]);

  /**
   * Install missing dependencies for a sample.
   * Progress will be reported via events: dep:start, dep:progress, dep:complete, dep:error
   */
  const installDependencies = useCallback(async (sampleId: string): Promise<DependencyInstallResult> => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    console.log(`[useAuroraView] Starting dependency installation for sample_id=${sampleId}`);
    try {
      const result = await client.call<DependencyInstallResult>('api.install_dependencies', { sample_id: sampleId });
      console.log(`[useAuroraView] Dependency installation API call result:`, result);
      return result;
    } catch (error) {
      console.error(`[useAuroraView] Dependency installation API call failed:`, error);
      throw error;
    }
  }, [client]);

  /**
   * Cancel ongoing dependency installation
   */
  const cancelInstallation = useCallback(async (): Promise<ApiResult> => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    console.log(`[useAuroraView] Cancelling dependency installation`);
    try {
      const result = await client.call<ApiResult>('api.cancel_installation');
      console.log(`[useAuroraView] Cancel installation API call result:`, result);
      return result;
    } catch (error) {
      console.error(`[useAuroraView] Cancel installation API call failed:`, error);
      throw error;
    }
  }, [client]);

  /**
   * Get dependency info for all samples that have requirements
   */

  const getAllSampleDependencies = useCallback(async (): Promise<{
    ok: boolean;
    samples: Record<string, DependencyInfo>;
  }> => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    return client.call<{ ok: boolean; samples: Record<string, DependencyInfo> }>('api.get_all_sample_dependencies');
  }, [client]);

  // ============================================
  // Extension Management APIs (Rust native)
  // ============================================

  /**
   * List all loaded extensions from Rust plugin
   * Note: Uses invoke() for plugin commands, not call() which is for Python API methods
   */
  const listExtensions = useCallback(async (): Promise<ExtensionInfo[]> => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    try {
      const result = await client.invoke<ExtensionInfo[]>('plugin:extensions|list_extensions', {});
      return result || [];
    } catch (e) {
      console.error('[useAuroraView:listExtensions] Error:', e);
      return [];
    }
  }, [client]);

  /**
   * Get details about a specific extension
   */
  const getExtension = useCallback(async (extensionId: string): Promise<ExtensionInfo | null> => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    try {
      return await client.invoke<ExtensionInfo>('plugin:extensions|get_extension', {
        extensionId,
      });
    } catch (e) {
      console.error('[useAuroraView:getExtension] Error:', e);
      return null;
    }
  }, [client]);

  /**
   * Get polyfill script for an extension
   */
  const getPolyfill = useCallback(async (extensionId: string): Promise<{ polyfill: string; wxtShim: string } | null> => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    try {
      return await client.invoke<{ polyfill: string; wxtShim: string }>('plugin:extensions|get_polyfill', {
        extensionId,
      });
    } catch (e) {
      console.error('[useAuroraView:getPolyfill] Error:', e);
      return null;
    }
  }, [client]);

  /**
   * Get side panel HTML content for an extension
   */
  const getSidePanel = useCallback(async (extensionId: string): Promise<{ html: string; path: string } | null> => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    try {
      return await client.invoke<{ html: string; path: string }>('plugin:extensions|get_side_panel', {
        extensionId,
      });
    } catch (e) {
      console.error('[useAuroraView:getSidePanel] Error:', e);
      return null;
    }
  }, [client]);

  /**
   * Open side panel for an extension
   */
  const openSidePanel = useCallback(async (extensionId: string): Promise<boolean> => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    try {
      const result = await client.invoke<{ success: boolean }>('plugin:extensions|open_side_panel', {
        extensionId,
      });
      return result?.success ?? false;
    } catch (e) {
      console.error('[useAuroraView:openSidePanel] Error:', e);
      return false;
    }
  }, [client]);

  /**
   * Close side panel for an extension
   */
  const closeSidePanel = useCallback(async (extensionId: string): Promise<boolean> => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    try {
      const result = await client.invoke<{ success: boolean }>('plugin:extensions|close_side_panel', {
        extensionId,
      });
      return result?.success ?? false;
    } catch (e) {
      console.error('[useAuroraView:closeSidePanel] Error:', e);
      return false;
    }
  }, [client]);

  /**
   * Get side panel state for an extension
   */
  const getSidePanelState = useCallback(async (extensionId: string): Promise<SidePanelState | null> => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    try {
      return await client.invoke<SidePanelState>('plugin:extensions|get_side_panel_state', {
        extensionId,
      });
    } catch (e) {
      console.error('[useAuroraView:getSidePanelState] Error:', e);
      return null;
    }
  }, [client]);

  /**
   * Call a Chrome Extension API through the Rust plugin
   */
  const callExtensionApi = useCallback(async <T = unknown>(
    extensionId: string,
    api: string,
    method: string,
    params: Record<string, unknown> = {}
  ): Promise<T> => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    return client.invoke<T>('plugin:extensions|api_call', {
      extensionId,
      api,
      method,
      params,
    });
  }, [client]);

  /**
   * Dispatch an event to extension listeners
   */
  const dispatchExtensionEvent = useCallback(async (
    extensionId: string,
    api: string,
    event: string,
    args: unknown[] = []
  ): Promise<boolean> => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    try {
      const result = await client.invoke<{ success: boolean }>('plugin:extensions|dispatch_event', {
        extensionId,
        api,
        event,
        args,
      });
      return result?.success ?? false;
    } catch (e) {
      console.error('[useAuroraView:dispatchExtensionEvent] Error:', e);
      return false;
    }
  }, [client]);

  // ============================================
  // Chrome Management API (Rust native)
  // ============================================

  /**
   * Get all installed extensions via chrome.management API
   */
  const managementGetAll = useCallback(async (): Promise<ExtensionInfo[]> => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    try {
      const result = await client.invoke<ExtensionInfo[]>('plugin:extensions|api_call', {
        extensionId: 'auroraview-host',
        api: 'management',
        method: 'getAll',
        params: {},
      });
      return result || [];
    } catch (e) {
      console.error('[useAuroraView:managementGetAll] Error:', e);
      return [];
    }
  }, [client]);

  /**
   * Get extension info by ID via chrome.management API
   */
  const managementGet = useCallback(async (extensionId: string): Promise<ExtensionInfo | null> => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    try {
      return await client.invoke<ExtensionInfo>('plugin:extensions|api_call', {
        extensionId: 'auroraview-host',
        api: 'management',
        method: 'get',
        params: { id: extensionId },
      });
    } catch (e) {
      console.error('[useAuroraView:managementGet] Error:', e);
      return null;
    }
  }, [client]);

  /**
   * Enable/disable extension via chrome.management API
   */
  const managementSetEnabled = useCallback(async (extensionId: string, enabled: boolean): Promise<boolean> => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    try {
      await client.invoke('plugin:extensions|api_call', {
        extensionId: 'auroraview-host',
        api: 'management',
        method: 'setEnabled',
        params: { id: extensionId, enabled },
      });
      return true;
    } catch (e) {
      console.error('[useAuroraView:managementSetEnabled] Error:', e);
      return false;
    }
  }, [client]);

  /**
   * Uninstall extension via chrome.management API
   */
  const managementUninstall = useCallback(async (extensionId: string, showConfirmDialog = true): Promise<boolean> => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    try {
      await client.invoke('plugin:extensions|api_call', {
        extensionId: 'auroraview-host',
        api: 'management',
        method: 'uninstall',
        params: { id: extensionId, options: { showConfirmDialog } },
      });
      return true;
    } catch (e) {
      console.error('[useAuroraView:managementUninstall] Error:', e);
      return false;
    }
  }, [client]);

  /**
   * Get permission warnings for an extension
   */
  const managementGetPermissionWarnings = useCallback(async (extensionId: string): Promise<string[]> => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    try {
      const result = await client.invoke<string[]>('plugin:extensions|api_call', {
        extensionId: 'auroraview-host',
        api: 'management',
        method: 'getPermissionWarningsById',
        params: { id: extensionId },
      });
      return result || [];
    } catch (e) {
      console.error('[useAuroraView:managementGetPermissionWarnings] Error:', e);
      return [];
    }
  }, [client]);

  // ============================================
  // Extension Installation APIs (Python backend - for file system operations)
  // ============================================

  /**
   * Install extension to WebView2's extensions directory
   */
  const installToWebView = useCallback(async (path: string, name?: string): Promise<ApiResult & { requiresRestart?: boolean; extensionsDir?: string }> => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    try {
      const result = await client.call<{
        ok: boolean;
        success?: boolean;
        id?: string;
        name?: string;
        version?: string;
        path?: string;
        extensionsDir?: string;
        message?: string;
        error?: string;
        requiresRestart?: boolean;
      }>('api.install_to_webview', { path, name });
      
      return {
        ok: result.ok || result.success || false,
        message: result.message,
        error: result.error,
        requiresRestart: result.requiresRestart,
        extensionsDir: result.extensionsDir,
      };
    } catch (e) {
      return {
        ok: false,
        error: String(e),
      };
    }
  }, [client]);

  /**
   * List extensions installed in WebView2's extensions directory
   */
  const listWebViewExtensions = useCallback(async (): Promise<{
    ok: boolean;
    extensions: InstalledExtension[];
    extensionsDir: string;
    count: number;
  }> => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    try {
      return await client.call<{
        ok: boolean;
        extensions: InstalledExtension[];
        extensionsDir: string;
        count: number;
      }>('api.list_webview_extensions', {});
    } catch {
      return { ok: false, extensions: [], extensionsDir: '', count: 0 };
    }
  }, [client]);

  /**
   * Remove extension from WebView2's extensions directory
   */
  const removeWebViewExtension = useCallback(async (id: string): Promise<ApiResult & { requiresRestart?: boolean }> => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    try {
      const result = await client.call<{
        ok: boolean;
        success?: boolean;
        id?: string;
        message?: string;
        error?: string;
        requiresRestart?: boolean;
      }>('api.remove_webview_extension', { id });
      
      return {
        ok: result.ok || result.success || false,
        message: result.message,
        error: result.error,
        requiresRestart: result.requiresRestart,
      };
    } catch (e) {
      return {
        ok: false,
        error: String(e),
      };
    }
  }, [client]);

  /**
   * Open WebView2 extensions directory in file explorer
   */
  const openExtensionsDir = useCallback(async (): Promise<ApiResult> => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    try {
      const result = await client.call<{
        ok: boolean;
        success?: boolean;
        path?: string;
        error?: string;
      }>('api.open_extensions_dir', {});
      
      return {
        ok: result.ok || result.success || false,
        message: result.path ? `Opened: ${result.path}` : undefined,
        error: result.error,
      };
    } catch (e) {
      return {
        ok: false,
        error: String(e),
      };
    }
  }, [client]);

  /**
   * Restart the application (for applying extension changes)
   */
  const restartApp = useCallback(async (): Promise<ApiResult> => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    try {
      const result = await client.invoke<{
        ok: boolean;
        success?: boolean;
        message?: string;
        error?: string;
      }>('plugin:shell|restart_app', {});
      
      return {
        ok: result.ok || result.success || false,
        message: result.message,
        error: result.error,
      };
    } catch (e) {
      return {
        ok: false,
        error: String(e),
      };
    }
  }, [client]);

  // ============================================
  // Legacy Browser Extension Bridge APIs (Python backend)
  // These are kept for backward compatibility but deprecated
  // ============================================

  const startExtensionBridge = useCallback(async (wsPort: number = 49152, httpPort: number = 49153): Promise<ApiResult> => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    return client.call<ApiResult>('api.start_extension_bridge', { ws_port: wsPort, http_port: httpPort });
  }, [client]);

  const stopExtensionBridge = useCallback(async (): Promise<ApiResult> => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    return client.call<ApiResult>('api.stop_extension_bridge');
  }, [client]);

  const getExtensionStatus = useCallback(async (): Promise<BrowserExtensionStatus> => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    return client.call<BrowserExtensionStatus>('api.get_extension_status');
  }, [client]);

  const broadcastToExtensions = useCallback(async (action: string, data: unknown): Promise<ApiResult> => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    return client.call<ApiResult>('api.broadcast_to_extensions', { event: action, data });
  }, [client]);

  const installExtension = useCallback(async (path: string, browser: 'chrome' | 'firefox' = 'chrome'): Promise<ApiResult> => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    try {
      const result = await client.call<{ ok: boolean; success?: boolean; path?: string; browser?: string; message?: string; error?: string; isFolder?: boolean }>(
        'api.install_extension',
        { path, browser }
      );
      
      return {
        ok: result.ok || result.success || false,
        message: result.message,
        error: result.error,
      };
    } catch (e) {
      return {
        ok: false,
        error: String(e),
      };
    }
  }, [client]);

  const getExtensionInfo = useCallback(async (browser: 'chrome' | 'firefox' = 'chrome') => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    try {
      return await client.invoke<{
        downloadUrl: string;
        extensionId: string;
        browser: string;
        instructions: string;
        localPath: string | null;
      }>('plugin:browser_bridge|get_extension', { browser });
    } catch {
      return null;
    }
  }, [client]);

  // ============================================
  // Features APIs (Python backend)
  // ============================================

  // Bookmarks
  const getBookmarks = useCallback(async (): Promise<{
    ok: boolean;
    bookmarks: Bookmark[];
    folders: BookmarkFolder[];
  }> => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    try {
      return await client.call('features.get_bookmarks');
    } catch {
      return { ok: false, bookmarks: [], folders: [] };
    }
  }, [client]);

  const addBookmark = useCallback(async (url: string, title: string, folderId?: string): Promise<{
    ok: boolean;
    bookmark?: Bookmark;
    error?: string;
  }> => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    return client.call('features.add_bookmark', { url, title, folder_id: folderId });
  }, [client]);

  const removeBookmark = useCallback(async (bookmarkId: string): Promise<{ ok: boolean }> => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    return client.call('features.remove_bookmark', { bookmark_id: bookmarkId });
  }, [client]);

  const isBookmarked = useCallback(async (url: string): Promise<{
    ok: boolean;
    bookmarked: boolean;
    bookmark?: Bookmark;
  }> => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    try {
      return await client.call('features.is_bookmarked', { url });
    } catch {
      return { ok: false, bookmarked: false };
    }
  }, [client]);

  // History
  const getHistory = useCallback(async (limit = 100): Promise<{
    ok: boolean;
    entries: HistoryEntry[];
    total: number;
  }> => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    try {
      return await client.call('features.get_history', { limit });
    } catch {
      return { ok: false, entries: [], total: 0 };
    }
  }, [client]);

  const addHistory = useCallback(async (url: string, title: string): Promise<{
    ok: boolean;
    entry?: HistoryEntry;
  }> => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    return client.call('features.add_history', { url, title });
  }, [client]);

  const removeHistory = useCallback(async (entryId: string): Promise<{ ok: boolean }> => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    return client.call('features.remove_history', { entry_id: entryId });
  }, [client]);

  const clearHistory = useCallback(async (): Promise<{ ok: boolean }> => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    return client.call('features.clear_history');
  }, [client]);

  const searchHistory = useCallback(async (query: string, limit = 50): Promise<{
    ok: boolean;
    entries: HistoryEntry[];
  }> => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    try {
      return await client.call('features.search_history', { query, limit });
    } catch {
      return { ok: false, entries: [] };
    }
  }, [client]);

  // Downloads
  const getDownloads = useCallback(async (): Promise<{
    ok: boolean;
    downloads: DownloadItem[];
  }> => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    try {
      return await client.call('features.get_downloads');
    } catch {
      return { ok: false, downloads: [] };
    }
  }, [client]);

  const pauseDownload = useCallback(async (downloadId: string): Promise<{ ok: boolean }> => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    return client.call('features.pause_download', { download_id: downloadId });
  }, [client]);

  const resumeDownload = useCallback(async (downloadId: string): Promise<{ ok: boolean }> => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    return client.call('features.resume_download', { download_id: downloadId });
  }, [client]);

  const cancelDownload = useCallback(async (downloadId: string): Promise<{ ok: boolean }> => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    return client.call('features.cancel_download', { download_id: downloadId });
  }, [client]);

  const retryDownload = useCallback(async (downloadId: string): Promise<{ ok: boolean }> => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    return client.call('features.retry_download', { download_id: downloadId });
  }, [client]);

  const removeDownload = useCallback(async (downloadId: string): Promise<{ ok: boolean }> => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    return client.call('features.remove_download', { download_id: downloadId });
  }, [client]);

  const clearCompletedDownloads = useCallback(async (): Promise<{ ok: boolean }> => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    return client.call('features.clear_completed_downloads');
  }, [client]);

  // Settings
  const getSettings = useCallback(async (): Promise<{
    ok: boolean;
    settings: Record<string, unknown>;
  }> => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    try {
      return await client.call('features.get_settings');
    } catch {
      return { ok: false, settings: {} };
    }
  }, [client]);

  const getSetting = useCallback(async (key: string): Promise<{
    ok: boolean;
    key: string;
    value: unknown;
  }> => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    return client.call('features.get_setting', { key });
  }, [client]);

  const setSetting = useCallback(async (key: string, value: unknown): Promise<{ ok: boolean }> => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    return client.call('features.set_setting', { key, value });
  }, [client]);

  const resetSettings = useCallback(async (): Promise<{ ok: boolean }> => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    return client.call('features.reset_settings');
  }, [client]);

  // Notifications
  const showNotification = useCallback(async (
    title: string,
    message: string,
    type: 'info' | 'success' | 'warning' | 'error' = 'info',
    timeout?: number
  ): Promise<{ ok: boolean; id?: string }> => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    return client.call('features.show_notification', { title, message, type, timeout });
  }, [client]);

  const dismissNotification = useCallback(async (notificationId: string): Promise<{ ok: boolean }> => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    return client.call('features.dismiss_notification', { notification_id: notificationId });
  }, [client]);

  const getNotifications = useCallback(async (): Promise<{
    ok: boolean;
    notifications: Array<{ id: string; title: string; message: string; type: string }>;
  }> => {
    if (!client) {
      throw new Error('AuroraView not ready');
    }
    try {
      return await client.call('features.get_notifications');
    } catch {
      return { ok: false, notifications: [] };
    }
  }, [client]);

  return {
    isReady,
    // Sample Management
    getSource,
    runSample,
    getSamples,
    getCategories,
    openUrl,
    openInWebView,
    killProcess,
    sendToProcess,
    listProcesses,
    // Dependency Management
    checkDependencies,
    installDependencies,
    cancelInstallation,
    getAllSampleDependencies,

    // Extension Management (Rust native)
    listExtensions,
    getExtension,
    getPolyfill,
    getSidePanel,
    openSidePanel,
    closeSidePanel,
    getSidePanelState,
    callExtensionApi,
    dispatchExtensionEvent,
    // Chrome Management API
    managementGetAll,
    managementGet,
    managementSetEnabled,
    managementUninstall,
    managementGetPermissionWarnings,
    // Extension Installation (Python backend)
    installToWebView,
    listWebViewExtensions,
    removeWebViewExtension,
    openExtensionsDir,
    restartApp,
    // Legacy Browser Extension APIs (deprecated)
    startExtensionBridge,
    stopExtensionBridge,
    getExtensionStatus,
    broadcastToExtensions,
    installExtension,
    getExtensionInfo,
    // Features APIs
    // Bookmarks
    getBookmarks,
    addBookmark,
    removeBookmark,
    isBookmarked,
    // History
    getHistory,
    addHistory,
    removeHistory,
    clearHistory,
    searchHistory,
    // Downloads
    getDownloads,
    pauseDownload,
    resumeDownload,
    cancelDownload,
    retryDownload,
    removeDownload,
    clearCompletedDownloads,
    // Settings
    getSettings,
    getSetting,
    setSetting,
    resetSettings,
    // Notifications
    showNotification,
    dismissNotification,
    getNotifications,
  };
}

/**
 * Re-export useProcessEvents from SDK with Gallery-compatible interface
 */
export function useProcessEvents(options?: {
  onStdout?: (data: ProcessOutput) => void;
  onStderr?: (data: ProcessOutput) => void;
  onExit?: (data: ProcessExit) => void;
}) {
  const [isSubscribed, setIsSubscribed] = useState(false);

  useProcessEventsBase({
    onStdout: options?.onStdout,
    onStderr: options?.onStderr,
    onExit: options?.onExit,
  });

  useEffect(() => {
    setIsSubscribed(true);
    return () => setIsSubscribed(false);
  }, []);

  return { isSubscribed };
}
