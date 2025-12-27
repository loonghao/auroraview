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
