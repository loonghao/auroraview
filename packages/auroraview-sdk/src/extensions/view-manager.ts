/**
 * AuroraView Extension View Manager
 *
 * Provides Chrome-like DevTools separation for extension views.
 * Each extension view (Service Worker, Popup, Side Panel) can have
 * its own independent DevTools window.
 *
 * @example
 * ```typescript
 * import { ExtensionViewManager } from '@aspect/auroraview-sdk/extensions';
 *
 * const viewManager = new ExtensionViewManager();
 *
 * // Create a side panel view
 * const view = await viewManager.createView({
 *   extensionId: 'my-extension',
 *   viewType: 'side_panel',
 *   htmlPath: 'sidepanel.html',
 *   title: 'My Extension Panel',
 * });
 *
 * // Open DevTools for this view
 * await viewManager.openDevtools(view.viewId);
 *
 * // Get CDP connection info for external tools
 * const cdp = await viewManager.getCdpInfo(view.viewId);
 * console.log(`Connect Playwright to: ${cdp.wsUrl}`);
 * ```
 */

import type {
  CreateViewConfig,
  CdpConnectionInfo,
  ExtensionViewInfo,
  ExtensionViewManagerAPI,
} from './types';

/**
 * Extension View Manager
 *
 * Manages independent WebView instances for each extension view,
 * providing Chrome-like DevTools separation.
 */
export class ExtensionViewManager implements ExtensionViewManagerAPI {
  private invoke: <T = unknown>(
    cmd: string,
    args?: Record<string, unknown>
  ) => Promise<T>;

  /**
   * Create a new ExtensionViewManager
   * @param invoker Function to invoke plugin commands
   */
  constructor(
    invoker?: <T = unknown>(
      cmd: string,
      args?: Record<string, unknown>
    ) => Promise<T>
  ) {
    // Use provided invoker or get from window.auroraview
    this.invoke =
      invoker ||
      ((cmd, args) => {
        if (window.auroraview?.invoke) {
          return window.auroraview.invoke(cmd, args);
        }
        return Promise.reject(new Error('AuroraView bridge not available'));
      });
  }

  /**
   * Create a new extension view
   */
  async createView(config: CreateViewConfig): Promise<ExtensionViewInfo> {
    return this.invoke<ExtensionViewInfo>('plugin:extensions|create_view', {
      extensionId: config.extensionId,
      viewType: config.viewType,
      htmlPath: config.htmlPath,
      title: config.title,
      width: config.width,
      height: config.height,
      devTools: config.devTools,
      debugPort: config.debugPort,
      visible: config.visible,
      parentHwnd: config.parentHwnd,
    });
  }

  /**
   * Get view information by view ID
   */
  async getView(viewId: string): Promise<ExtensionViewInfo | null> {
    try {
      return await this.invoke<ExtensionViewInfo>('plugin:extensions|get_view', {
        viewId,
      });
    } catch {
      return null;
    }
  }

  /**
   * Get all views for an extension
   */
  async getExtensionViews(extensionId: string): Promise<ExtensionViewInfo[]> {
    return this.invoke<ExtensionViewInfo[]>(
      'plugin:extensions|get_extension_views',
      { extensionId }
    );
  }

  /**
   * Get all extension views
   */
  async getAllViews(): Promise<ExtensionViewInfo[]> {
    return this.invoke<ExtensionViewInfo[]>('plugin:extensions|get_all_views');
  }

  /**
   * Open DevTools for a view
   */
  async openDevtools(viewId: string): Promise<void> {
    await this.invoke('plugin:extensions|open_devtools', { viewId });
  }

  /**
   * Close DevTools for a view
   */
  async closeDevtools(viewId: string): Promise<void> {
    await this.invoke('plugin:extensions|close_devtools', { viewId });
  }

  /**
   * Show a view
   */
  async showView(viewId: string): Promise<void> {
    await this.invoke('plugin:extensions|show_view', { viewId });
  }

  /**
   * Hide a view
   */
  async hideView(viewId: string): Promise<void> {
    await this.invoke('plugin:extensions|hide_view', { viewId });
  }

  /**
   * Destroy a view
   */
  async destroyView(viewId: string): Promise<void> {
    await this.invoke('plugin:extensions|destroy_view', { viewId });
  }

  /**
   * Get CDP connection info for a view
   */
  async getCdpInfo(viewId: string): Promise<CdpConnectionInfo | null> {
    try {
      return await this.invoke<CdpConnectionInfo>(
        'plugin:extensions|get_cdp_info',
        { viewId }
      );
    } catch {
      return null;
    }
  }

  /**
   * Get all CDP connections
   */
  async getAllCdpConnections(): Promise<CdpConnectionInfo[]> {
    return this.invoke<CdpConnectionInfo[]>(
      'plugin:extensions|get_all_cdp_connections'
    );
  }
}

/**
 * Get the global ExtensionViewManager instance
 */
let globalViewManager: ExtensionViewManager | null = null;

export function getExtensionViewManager(): ExtensionViewManager {
  if (!globalViewManager) {
    globalViewManager = new ExtensionViewManager();
  }
  return globalViewManager;
}

export default ExtensionViewManager;
