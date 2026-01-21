/**
 * AuroraView SDK Window Management API
 *
 * Provides JavaScript API for creating and managing WebView windows.
 *
 * @module window
 */

import type { Unsubscribe, WindowEventType, WindowEventData } from './core/types';

// ============================================
// Window Types
// ============================================

/**
 * Options for creating a new window
 */
export interface WindowOptions {
  /** Unique label/identifier for the window */
  label?: string;
  /** URL to load in the window */
  url?: string;
  /** HTML content to load (alternative to url) */
  html?: string;
  /** Window title */
  title?: string;
  /** Window width in pixels */
  width?: number;
  /** Window height in pixels */
  height?: number;
  /** Window x position */
  x?: number;
  /** Window y position */
  y?: number;
  /** Center the window on screen */
  center?: boolean;
  /** Make window resizable */
  resizable?: boolean;
  /** Make window frameless (no title bar) */
  frameless?: boolean;
  /** Make window transparent */
  transparent?: boolean;
  /** Keep window always on top */
  alwaysOnTop?: boolean;
  /** Start window minimized */
  minimized?: boolean;
  /** Start window maximized */
  maximized?: boolean;
  /** Start window in fullscreen */
  fullscreen?: boolean;
  /** Enable dev tools (debug mode) */
  devtools?: boolean;
}

/**
 * Window position
 */
export interface WindowPosition {
  x: number;
  y: number;
}

/**
 * Window size
 */
export interface WindowSize {
  width: number;
  height: number;
}

/**
 * Window bounds (position + size)
 */
export interface WindowBounds extends WindowPosition, WindowSize {}

/**
 * Window state information
 */
export interface WindowState {
  /** Window label/id */
  label: string;
  /** Whether window is visible */
  visible: boolean;
  /** Whether window is focused */
  focused: boolean;
  /** Whether window is minimized */
  minimized: boolean;
  /** Whether window is maximized */
  maximized: boolean;
  /** Whether window is fullscreen */
  fullscreen: boolean;
  /** Current window bounds */
  bounds: WindowBounds;
}

/**
 * Window event handler type
 */
export type WindowEventHandler = (data: WindowEventData) => void;

// ============================================
// Window Class
// ============================================

/**
 * Window management class for controlling WebView windows.
 *
 * @example
 * ```typescript
 * import { Window } from '@auroraview/sdk';
 *
 * // Create a new window
 * const win = await Window.create({
 *   label: 'settings',
 *   url: '/settings.html',
 *   title: 'Settings',
 *   width: 520,
 *   height: 650,
 *   center: true,
 * });
 *
 * // Control the window
 * await win.show();
 * await win.setSize(800, 600);
 * await win.center();
 *
 * // Get current window
 * const current = Window.getCurrent();
 * current.startDrag();
 * ```
 */
export class Window {
  private _label: string;
  private _eventHandlers: Map<WindowEventType, Set<WindowEventHandler>>;

  /**
   * Create a Window instance (internal use - use Window.create() instead)
   */
  private constructor(label: string) {
    this._label = label;
    this._eventHandlers = new Map();
    this._setupEventListeners();
  }

  /**
   * Get the window label/identifier
   */
  get label(): string {
    return this._label;
  }

  // ============================================
  // Static Factory Methods
  // ============================================

  /**
   * Create a new window.
   *
   * @param options - Window creation options
   * @returns Promise that resolves to the new Window instance
   *
   * @example
   * ```typescript
   * const win = await Window.create({
   *   label: 'settings',
   *   url: '/settings.html',
   *   title: 'Settings',
   *   width: 520,
   *   height: 650,
   * });
   * ```
   */
  static async create(options: WindowOptions): Promise<Window> {
    const bridge = window.auroraview;
    if (!bridge) {
      throw new Error('AuroraView bridge not available');
    }

    const result = await bridge.call<{ label: string }>('window.create', options);
    return new Window(result.label);
  }

  /**
   * Get the current window instance.
   *
   * @returns The current Window instance
   *
   * @example
   * ```typescript
   * const current = Window.getCurrent();
   * await current.setTitle('New Title');
   * ```
   */
  static getCurrent(): Window {
    // Get current window label from global
    const w = window as { __AURORAVIEW_WINDOW_LABEL__?: string };
    const label = w.__AURORAVIEW_WINDOW_LABEL__ || 'main';
    return new Window(label);
  }

  /**
   * Get a window by its label.
   *
   * @param label - The window label to find
   * @returns Promise that resolves to the Window, or null if not found
   *
   * @example
   * ```typescript
   * const settings = await Window.getByLabel('settings');
   * if (settings) {
   *   await settings.focus();
   * }
   * ```
   */
  static async getByLabel(label: string): Promise<Window | null> {
    const bridge = window.auroraview;
    if (!bridge) {
      return null;
    }

    try {
      const result = await bridge.call<{ exists: boolean }>('window.exists', { label });
      if (result.exists) {
        return new Window(label);
      }
    } catch {
      // Window not found
    }

    return null;
  }

  /**
   * Get all open windows.
   *
   * @returns Promise that resolves to an array of all Window instances
   *
   * @example
   * ```typescript
   * const windows = await Window.getAll();
   * for (const win of windows) {
   *   console.log(win.label);
   * }
   * ```
   */
  static async getAll(): Promise<Window[]> {
    const bridge = window.auroraview;
    if (!bridge) {
      return [];
    }

    try {
      const result = await bridge.call<{ labels: string[] }>('window.list');
      return result.labels.map((label) => new Window(label));
    } catch {
      return [];
    }
  }

  /**
   * Get count of all open windows.
   *
   * @returns Promise that resolves to the number of open windows
   */
  static async count(): Promise<number> {
    const bridge = window.auroraview;
    if (!bridge) {
      return 0;
    }

    try {
      const result = await bridge.call<{ count: number }>('window.count');
      return result.count;
    } catch {
      return 0;
    }
  }

  // ============================================
  // Instance Methods
  // ============================================

  /**
   * Show the window.
   */
  async show(): Promise<void> {
    await this._call('window.show');
  }

  /**
   * Hide the window.
   */
  async hide(): Promise<void> {
    await this._call('window.hide');
  }

  /**
   * Close the window.
   */
  async close(): Promise<void> {
    await this._call('window.close');
    this._cleanup();
  }

  /**
   * Focus the window (bring to front).
   */
  async focus(): Promise<void> {
    await this._call('window.focus');
  }

  /**
   * Minimize the window.
   */
  async minimize(): Promise<void> {
    await this._call('window.minimize');
  }

  /**
   * Maximize the window.
   */
  async maximize(): Promise<void> {
    await this._call('window.maximize');
  }

  /**
   * Restore the window from minimized/maximized state.
   */
  async restore(): Promise<void> {
    await this._call('window.restore');
  }

  /**
   * Toggle fullscreen mode.
   */
  async toggleFullscreen(): Promise<void> {
    await this._call('window.toggleFullscreen');
  }

  /**
   * Set the window title.
   *
   * @param title - The new window title
   */
  async setTitle(title: string): Promise<void> {
    await this._call('window.setTitle', { title });
  }

  /**
   * Set the window position.
   *
   * @param x - X coordinate
   * @param y - Y coordinate
   */
  async setPosition(x: number, y: number): Promise<void> {
    await this._call('window.setPosition', { x, y });
  }

  /**
   * Set the window size.
   *
   * @param width - Width in pixels
   * @param height - Height in pixels
   */
  async setSize(width: number, height: number): Promise<void> {
    await this._call('window.setSize', { width, height });
  }

  /**
   * Set the minimum window size.
   *
   * @param width - Minimum width
   * @param height - Minimum height
   */
  async setMinSize(width: number, height: number): Promise<void> {
    await this._call('window.setMinSize', { width, height });
  }

  /**
   * Set the maximum window size.
   *
   * @param width - Maximum width
   * @param height - Maximum height
   */
  async setMaxSize(width: number, height: number): Promise<void> {
    await this._call('window.setMaxSize', { width, height });
  }

  /**
   * Center the window on screen.
   */
  async center(): Promise<void> {
    await this._call('window.center');
  }

  /**
   * Set whether the window is always on top.
   *
   * @param alwaysOnTop - Whether to keep window on top
   */
  async setAlwaysOnTop(alwaysOnTop: boolean): Promise<void> {
    await this._call('window.setAlwaysOnTop', { alwaysOnTop });
  }

  /**
   * Set whether the window is resizable.
   *
   * @param resizable - Whether window can be resized
   */
  async setResizable(resizable: boolean): Promise<void> {
    await this._call('window.setResizable', { resizable });
  }

  /**
   * Get the current window position.
   *
   * @returns Promise that resolves to the window position
   */
  async getPosition(): Promise<WindowPosition> {
    return await this._call<WindowPosition>('window.getPosition');
  }

  /**
   * Get the current window size.
   *
   * @returns Promise that resolves to the window size
   */
  async getSize(): Promise<WindowSize> {
    return await this._call<WindowSize>('window.getSize');
  }

  /**
   * Get the current window bounds (position + size).
   *
   * @returns Promise that resolves to the window bounds
   */
  async getBounds(): Promise<WindowBounds> {
    return await this._call<WindowBounds>('window.getBounds');
  }

  /**
   * Get the current window state.
   *
   * @returns Promise that resolves to the window state
   */
  async getState(): Promise<WindowState> {
    return await this._call<WindowState>('window.getState');
  }

  /**
   * Check if the window is visible.
   */
  async isVisible(): Promise<boolean> {
    const result = await this._call<{ visible: boolean }>('window.isVisible');
    return result.visible;
  }

  /**
   * Check if the window is focused.
   */
  async isFocused(): Promise<boolean> {
    const result = await this._call<{ focused: boolean }>('window.isFocused');
    return result.focused;
  }

  /**
   * Check if the window is minimized.
   */
  async isMinimized(): Promise<boolean> {
    const result = await this._call<{ minimized: boolean }>('window.isMinimized');
    return result.minimized;
  }

  /**
   * Check if the window is maximized.
   */
  async isMaximized(): Promise<boolean> {
    const result = await this._call<{ maximized: boolean }>('window.isMaximized');
    return result.maximized;
  }

  /**
   * Start native window drag (for frameless windows).
   * Call this on mousedown in drag regions.
   */
  startDrag(): void {
    const bridge = window.auroraview;
    if (bridge?.startDrag) {
      bridge.startDrag();
    }
  }

  /**
   * Navigate to a URL.
   *
   * @param url - URL to navigate to
   */
  async navigate(url: string): Promise<void> {
    await this._call('window.navigate', { url });
  }

  /**
   * Load HTML content.
   *
   * @param html - HTML content to load
   */
  async loadHtml(html: string): Promise<void> {
    await this._call('window.loadHtml', { html });
  }

  /**
   * Execute JavaScript in the window.
   *
   * @param script - JavaScript code to execute
   * @returns Promise that resolves to the result
   */
  async eval<T = unknown>(script: string): Promise<T> {
    return await this._call<T>('window.eval', { script });
  }

  /**
   * Emit an event to this window.
   *
   * @param event - Event name
   * @param data - Event data
   */
  async emit(event: string, data?: unknown): Promise<void> {
    await this._call('window.emit', { event, data });
  }

  // ============================================
  // Event Handling
  // ============================================

  /**
   * Subscribe to a window event.
   *
   * @param event - Event type to subscribe to
   * @param handler - Event handler function
   * @returns Unsubscribe function
   *
   * @example
   * ```typescript
   * const unsubscribe = win.on('resized', (data) => {
   *   console.log('Window resized:', data.width, data.height);
   * });
   *
   * // Later: unsubscribe();
   * ```
   */
  on(event: WindowEventType, handler: WindowEventHandler): Unsubscribe {
    let handlers = this._eventHandlers.get(event);
    if (!handlers) {
      handlers = new Set();
      this._eventHandlers.set(event, handlers);
    }
    handlers.add(handler);

    return () => {
      handlers?.delete(handler);
    };
  }

  /**
   * Subscribe to a window event once.
   *
   * @param event - Event type to subscribe to
   * @param handler - Event handler function
   * @returns Unsubscribe function
   */
  once(event: WindowEventType, handler: WindowEventHandler): Unsubscribe {
    const wrappedHandler: WindowEventHandler = (data) => {
      this._eventHandlers.get(event)?.delete(wrappedHandler);
      handler(data);
    };

    return this.on(event, wrappedHandler);
  }

  /**
   * Unsubscribe from a window event.
   *
   * @param event - Event type
   * @param handler - Handler to remove (optional - removes all if not provided)
   */
  off(event: WindowEventType, handler?: WindowEventHandler): void {
    if (handler) {
      this._eventHandlers.get(event)?.delete(handler);
    } else {
      this._eventHandlers.delete(event);
    }
  }

  // ============================================
  // Private Methods
  // ============================================

  /**
   * Call a window method on the backend.
   */
  private async _call<T = void>(method: string, args?: Record<string, unknown>): Promise<T> {
    const bridge = window.auroraview;
    if (!bridge) {
      throw new Error('AuroraView bridge not available');
    }

    return await bridge.call<T>(method, { label: this._label, ...args });
  }

  /**
   * Set up event listeners for window events.
   */
  private _setupEventListeners(): void {
    const bridge = window.auroraview;
    if (!bridge) return;

    // Listen for window events
    const eventPrefix = `window:${this._label}:`;

    bridge.on(`${eventPrefix}shown`, () => this._triggerEvent('shown'));
    bridge.on(`${eventPrefix}hidden`, () => this._triggerEvent('hidden'));
    bridge.on(`${eventPrefix}focused`, () => this._triggerEvent('focused'));
    bridge.on(`${eventPrefix}blurred`, () => this._triggerEvent('blurred'));
    bridge.on(`${eventPrefix}resized`, (data) => this._triggerEvent('resized', data as WindowEventData));
    bridge.on(`${eventPrefix}moved`, (data) => this._triggerEvent('moved', data as WindowEventData));
    bridge.on(`${eventPrefix}minimized`, () => this._triggerEvent('minimized'));
    bridge.on(`${eventPrefix}maximized`, () => this._triggerEvent('maximized'));
    bridge.on(`${eventPrefix}restored`, () => this._triggerEvent('restored'));
    bridge.on(`${eventPrefix}closing`, () => this._triggerEvent('closing'));
    bridge.on(`${eventPrefix}closed`, () => this._triggerEvent('closed'));
  }

  /**
   * Trigger a window event to all handlers.
   */
  private _triggerEvent(type: WindowEventType, data?: Partial<WindowEventData>): void {
    const eventData: WindowEventData = {
      type,
      timestamp: Date.now(),
      ...data,
    };

    const handlers = this._eventHandlers.get(type);
    if (handlers) {
      handlers.forEach((handler) => {
        try {
          handler(eventData);
        } catch (e) {
          console.error(`[Window] Error in ${type} handler:`, e);
        }
      });
    }
  }

  /**
   * Clean up event handlers.
   */
  private _cleanup(): void {
    this._eventHandlers.clear();
  }
}

// ============================================
// Convenience Functions
// ============================================

/**
 * Get the current window.
 *
 * @returns The current Window instance
 */
export function getCurrentWindow(): Window {
  return Window.getCurrent();
}

/**
 * Create a new window.
 *
 * @param options - Window creation options
 * @returns Promise that resolves to the new Window instance
 */
export function createWindow(options: WindowOptions): Promise<Window> {
  return Window.create(options);
}

/**
 * Close the current window.
 */
export async function closeCurrentWindow(): Promise<void> {
  await Window.getCurrent().close();
}

/**
 * Start native drag on the current window.
 * Convenience function for frameless windows.
 */
export function startDrag(): void {
  Window.getCurrent().startDrag();
}
