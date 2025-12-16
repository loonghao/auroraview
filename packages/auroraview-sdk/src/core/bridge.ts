/**
 * AuroraView SDK Bridge Client
 *
 * Provides a type-safe wrapper around the native bridge API.
 */

import { EventEmitter, getGlobalEmitter } from './events';
import type {
  EventHandler,
  Unsubscribe,
  AuroraViewBridge,
  FileSystemAPI,
  DialogAPI,
  ClipboardAPI,
  ShellAPI,
  AuroraViewState,
} from './types';

/**
 * AuroraView client interface
 */
export interface AuroraViewClient {
  /** Call a Python method (RPC-style) */
  call<T = unknown>(method: string, params?: unknown): Promise<T>;

  /** Invoke a plugin command */
  invoke<T = unknown>(cmd: string, args?: Record<string, unknown>): Promise<T>;

  /** Send an event to Python (fire-and-forget) */
  emit(event: string, data?: unknown): void;

  /** Subscribe to an event from Python */
  on<T = unknown>(event: string, handler: EventHandler<T>): Unsubscribe;

  /** Subscribe to an event once */
  once<T = unknown>(event: string, handler: EventHandler<T>): Unsubscribe;

  /** Unsubscribe from an event */
  off(event: string, handler?: EventHandler): void;

  /** Check if bridge is ready */
  isReady(): boolean;

  /** Wait for bridge to be ready */
  whenReady(): Promise<AuroraViewClient>;

  /** Get the raw bridge object */
  getRawBridge(): AuroraViewBridge | undefined;

  /** File system API */
  readonly fs: FileSystemAPI | undefined;

  /** Dialog API */
  readonly dialog: DialogAPI | undefined;

  /** Clipboard API */
  readonly clipboard: ClipboardAPI | undefined;

  /** Shell API */
  readonly shell: ShellAPI | undefined;

  /** Shared state */
  readonly state: AuroraViewState | undefined;
}

/**
 * Internal client implementation
 */
class AuroraViewClientImpl implements AuroraViewClient {
  private events: EventEmitter;
  private interceptInstalled = false;

  constructor() {
    this.events = getGlobalEmitter();
    this.installTriggerIntercept();
  }

  /**
   * Install intercept on window.auroraview.trigger to forward events
   */
  private installTriggerIntercept(): void {
    if (this.interceptInstalled) return;
    if (typeof window === 'undefined') return;

    const install = () => {
      const bridge = window.auroraview;
      if (!bridge) return;

      const originalTrigger = bridge.trigger;
      bridge.trigger = (event: string, detail?: unknown) => {
        // Call original trigger first
        originalTrigger?.call(bridge, event, detail);
        // Forward to our event system
        this.events.emit(event, detail);
      };

      this.interceptInstalled = true;
    };

    // Try to install immediately
    if (window.auroraview) {
      install();
    } else {
      // Wait for bridge to be available
      const checkInterval = setInterval(() => {
        if (window.auroraview) {
          clearInterval(checkInterval);
          install();
        }
      }, 10);

      // Stop checking after 10 seconds
      setTimeout(() => clearInterval(checkInterval), 10000);
    }
  }

  call<T = unknown>(method: string, params?: unknown): Promise<T> {
    const bridge = window.auroraview;
    if (!bridge) {
      return Promise.reject(new Error('AuroraView bridge not available'));
    }
    return bridge.call<T>(method, params);
  }

  invoke<T = unknown>(cmd: string, args?: Record<string, unknown>): Promise<T> {
    const bridge = window.auroraview;
    if (!bridge) {
      return Promise.reject(new Error('AuroraView bridge not available'));
    }
    return bridge.invoke<T>(cmd, args);
  }

  emit(event: string, data?: unknown): void {
    const bridge = window.auroraview;
    if (bridge) {
      bridge.send_event(event, data);
    }
  }

  on<T = unknown>(event: string, handler: EventHandler<T>): Unsubscribe {
    return this.events.on(event, handler);
  }

  once<T = unknown>(event: string, handler: EventHandler<T>): Unsubscribe {
    return this.events.once(event, handler);
  }

  off(event: string, handler?: EventHandler): void {
    this.events.off(event, handler);
  }

  isReady(): boolean {
    return window.auroraview?._ready === true;
  }

  whenReady(): Promise<AuroraViewClient> {
    return new Promise((resolve) => {
      if (this.isReady()) {
        resolve(this);
      } else if (window.auroraview) {
        window.auroraview.whenReady().then(() => {
          this.installTriggerIntercept();
          resolve(this);
        });
      } else {
        // Wait for bridge to appear
        const checkInterval = setInterval(() => {
          if (window.auroraview) {
            clearInterval(checkInterval);
            window.auroraview.whenReady().then(() => {
              this.installTriggerIntercept();
              resolve(this);
            });
          }
        }, 10);

        // Timeout after 30 seconds
        setTimeout(() => {
          clearInterval(checkInterval);
          resolve(this);
        }, 30000);
      }
    });
  }

  getRawBridge(): AuroraViewBridge | undefined {
    return window.auroraview;
  }

  get fs(): FileSystemAPI | undefined {
    return window.auroraview?.fs;
  }

  get dialog(): DialogAPI | undefined {
    return window.auroraview?.dialog;
  }

  get clipboard(): ClipboardAPI | undefined {
    return window.auroraview?.clipboard;
  }

  get shell(): ShellAPI | undefined {
    return window.auroraview?.shell;
  }

  get state(): AuroraViewState | undefined {
    return window.auroraview?.state;
  }
}

/** Singleton client instance */
let clientInstance: AuroraViewClient | null = null;

/**
 * Create or get the AuroraView client instance
 */
export function createAuroraView(): AuroraViewClient {
  if (!clientInstance) {
    clientInstance = new AuroraViewClientImpl();
  }
  return clientInstance;
}

/**
 * Get the AuroraView client instance (alias for createAuroraView)
 */
export function getAuroraView(): AuroraViewClient {
  return createAuroraView();
}
