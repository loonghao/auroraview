/**
 * AuroraView Extension View Management Types
 *
 * Type definitions for Chrome-like extension DevTools separation.
 */

// ============================================
// Extension View Types
// ============================================

/** Extension view type */
export type ExtensionViewType =
  | 'service_worker'
  | 'popup'
  | 'side_panel'
  | 'options'
  | 'devtools_panel';

/** Extension view state */
export type ExtensionViewState =
  | 'not_created'
  | 'creating'
  | 'hidden'
  | 'visible'
  | 'destroying'
  | 'error';

/** Extension view information */
export interface ExtensionViewInfo {
  /** Unique view ID */
  viewId: string;
  /** Extension ID */
  extensionId: string;
  /** View type */
  viewType: ExtensionViewType;
  /** Current state */
  state: ExtensionViewState;
  /** CDP debugging port */
  debugPort: number;
  /** DevTools URL for this view */
  devtoolsUrl: string;
  /** View title */
  title: string;
  /** Whether DevTools is currently open */
  devtoolsOpen: boolean;
}

/** Configuration for creating an extension view */
export interface CreateViewConfig {
  /** Extension ID */
  extensionId: string;
  /** View type */
  viewType: ExtensionViewType;
  /** HTML file path (relative to extension root) */
  htmlPath: string;
  /** Window title */
  title?: string;
  /** Window width */
  width?: number;
  /** Window height */
  height?: number;
  /** Enable DevTools */
  devTools?: boolean;
  /** CDP debugging port (auto-assigned if not specified) */
  debugPort?: number;
  /** Whether the view should be visible on creation */
  visible?: boolean;
  /** Parent window handle (for embedded views) */
  parentHwnd?: number;
}

// ============================================
// CDP Connection Types
// ============================================

/** CDP connection information */
export interface CdpConnectionInfo {
  /** View ID */
  viewId: string;
  /** Host address */
  host: string;
  /** Port number */
  port: number;
  /** WebSocket URL for CDP connection */
  wsUrl: string;
  /** DevTools frontend URL */
  devtoolsFrontendUrl: string;
}

// ============================================
// Extension View Manager API
// ============================================

/** Extension View Manager API interface */
export interface ExtensionViewManagerAPI {
  /**
   * Create a new extension view
   * @param config View configuration
   * @returns View information
   */
  createView(config: CreateViewConfig): Promise<ExtensionViewInfo>;

  /**
   * Get view information by view ID
   * @param viewId View ID
   * @returns View information or null if not found
   */
  getView(viewId: string): Promise<ExtensionViewInfo | null>;

  /**
   * Get all views for an extension
   * @param extensionId Extension ID
   * @returns Array of view information
   */
  getExtensionViews(extensionId: string): Promise<ExtensionViewInfo[]>;

  /**
   * Get all extension views
   * @returns Array of all view information
   */
  getAllViews(): Promise<ExtensionViewInfo[]>;

  /**
   * Open DevTools for a view
   * @param viewId View ID
   */
  openDevtools(viewId: string): Promise<void>;

  /**
   * Close DevTools for a view
   * @param viewId View ID
   */
  closeDevtools(viewId: string): Promise<void>;

  /**
   * Show a view
   * @param viewId View ID
   */
  showView(viewId: string): Promise<void>;

  /**
   * Hide a view
   * @param viewId View ID
   */
  hideView(viewId: string): Promise<void>;

  /**
   * Destroy a view
   * @param viewId View ID
   */
  destroyView(viewId: string): Promise<void>;

  /**
   * Get CDP connection info for a view
   * @param viewId View ID
   * @returns CDP connection information
   */
  getCdpInfo(viewId: string): Promise<CdpConnectionInfo | null>;

  /**
   * Get all CDP connections
   * @returns Array of CDP connection information
   */
  getAllCdpConnections(): Promise<CdpConnectionInfo[]>;
}

// ============================================
// Extension Info Types
// ============================================

/** Extension information */
export interface ExtensionInfo {
  /** Extension ID */
  id: string;
  /** Extension name */
  name: string;
  /** Extension version */
  version: string;
  /** Extension description */
  description: string;
  /** Whether extension is enabled */
  enabled: boolean;
  /** Side panel path (if any) */
  sidePanelPath?: string;
  /** Popup path (if any) */
  popupPath?: string;
  /** Options page path (if any) */
  optionsPage?: string;
  /** Root directory */
  rootDir: string;
  /** Permissions */
  permissions: string[];
  /** Host permissions */
  hostPermissions: string[];
  /** Manifest data */
  manifest?: Record<string, unknown>;
}

export {};
