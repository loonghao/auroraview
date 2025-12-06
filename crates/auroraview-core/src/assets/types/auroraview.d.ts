/**
 * AuroraView TypeScript Type Definitions
 *
 * This file provides TypeScript type definitions for the AuroraView JavaScript API.
 * Include this file in your TypeScript project to get full type support.
 *
 * Usage:
 *   // In your tsconfig.json, add to "include":
 *   // "node_modules/auroraview/types/auroraview.d.ts"
 *
 *   // Or reference directly:
 *   /// <reference path="./auroraview.d.ts" />
 *
 *   // Then use the API with full type support:
 *   const result = await window.auroraview.invoke("greet", { name: "Alice" });
 */

declare global {
  interface Window {
    auroraview: AuroraViewAPI;
  }
}

/**
 * Main AuroraView API interface
 */
export interface AuroraViewAPI {
  // ============================================
  // Ready State (for DCC environments)
  // ============================================

  /**
   * Internal ready state flag
   * True when bridge is fully initialized
   */
  readonly _ready: boolean;

  /**
   * Internal stub marker
   * True if this is a stub placeholder (bridge not yet loaded)
   */
  readonly _isStub?: boolean;

  /**
   * Check if bridge is ready (synchronous)
   * @returns True if bridge is fully initialized
   */
  isReady(): boolean;

  /**
   * Wait for event bridge to be ready
   * Use this in DCC environments where initialization timing may vary.
   *
   * @example
   * window.auroraview.whenReady().then(function(av) {
   *     av.call('api.myMethod', { param: 'value' });
   * });
   *
   * @returns Promise that resolves with window.auroraview when ready
   */
  whenReady(): Promise<AuroraViewAPI>;

  // ============================================
  // Event System
  // ============================================

  /**
   * Send an event to Python backend
   * @param event - Event name
   * @param data - Event data (optional)
   */
  emit(event: string, data?: unknown): void;

  /**
   * Alias for emit (deprecated, use emit instead)
   */
  send_event(event: string, data?: unknown): void;

  /**
   * Register an event handler
   * @param event - Event name to listen for
   * @param handler - Callback function
   * @returns Unsubscribe function
   */
  on(event: string, handler: (data: unknown) => void): () => void;

  /**
   * Remove an event handler
   * @param event - Event name
   * @param handler - Handler to remove
   */
  off(event: string, handler: (data: unknown) => void): void;

  // ============================================
  // State System (PyWebView-inspired)
  // ============================================

  /**
   * Reactive shared state between Python and JavaScript
   *
   * @example
   * // Read state
   * const theme = window.auroraview.state.theme;
   *
   * // Write state (auto-syncs to Python)
   * window.auroraview.state.theme = "dark";
   *
   * // Subscribe to changes
   * window.auroraview.state.onChange((key, value, source) => {
   *   console.log(`${key} changed to ${value} from ${source}`);
   * });
   */
  state: AuroraViewState;

  // ============================================
  // Command System (Tauri-inspired)
  // ============================================

  /**
   * Invoke a Python command (RPC-style)
   *
   * @param command - Command name registered in Python
   * @param args - Command arguments
   * @returns Promise resolving to command result
   *
   * @example
   * const result = await window.auroraview.invoke("greet", { name: "Alice" });
   * console.log(result); // "Hello, Alice!"
   */
  invoke<T = unknown>(command: string, args?: Record<string, unknown>): Promise<T>;

  // ============================================
  // Channel System (Tauri-inspired)
  // ============================================

  /**
   * Get or create a streaming channel
   *
   * @param channelId - Channel identifier
   * @returns Channel instance for receiving streamed data
   *
   * @example
   * const channel = window.auroraview.channel("progress");
   * channel.onMessage((data) => console.log("Progress:", data));
   * channel.onClose(() => console.log("Done!"));
   */
  channel(channelId: string): AuroraViewChannel;

  // ============================================
  // API Mode (DCC/QtWebView)
  // ============================================

  /**
   * Direct API access for DCC integration mode
   * Available when using QtWebView with AuroraView wrapper
   */
  api?: AuroraViewDirectAPI;
}

/**
 * Reactive state proxy interface
 */
export interface AuroraViewState {
  /**
   * Get/set any state value by key
   */
  [key: string]: unknown;

  /**
   * Subscribe to state changes
   * @param handler - Called when any state value changes
   * @returns Unsubscribe function
   */
  onChange(handler: StateChangeHandler): () => void;

  /**
   * Unsubscribe from state changes
   * @param handler - Handler to remove
   */
  offChange(handler: StateChangeHandler): void;

  /**
   * Get all state as a plain object
   */
  toJSON(): Record<string, unknown>;

  /**
   * Get all state keys
   */
  keys(): string[];
}

/**
 * State change handler function type
 */
export type StateChangeHandler = (
  key: string,
  value: unknown,
  source: "python" | "javascript"
) => void;

/**
 * Streaming channel interface
 */
export interface AuroraViewChannel {
  /**
   * Channel identifier
   */
  readonly id: string;

  /**
   * Whether the channel is closed
   */
  readonly isClosed: boolean;

  /**
   * Register a message handler
   * @param handler - Called with each message
   * @returns Unsubscribe function
   */
  onMessage<T = unknown>(handler: (data: T) => void): () => void;

  /**
   * Register a close handler
   * @param handler - Called when channel closes
   * @returns Unsubscribe function
   */
  onClose(handler: () => void): () => void;
}

/**
 * Direct API interface for DCC integration mode
 * Methods are dynamically bound from Python API object
 */
export interface AuroraViewDirectAPI {
  /**
   * Call any registered API method
   * Methods are defined by the Python API object passed to AuroraView
   */
  [methodName: string]: <T = unknown>(params?: unknown) => Promise<T>;
}

// ============================================
// Common Types
// ============================================

/**
 * Window event data
 */
export interface WindowEventData {
  type: WindowEventType;
  x?: number;
  y?: number;
  width?: number;
  height?: number;
  timestamp?: number;
}

/**
 * Window event types
 */
export type WindowEventType =
  | "shown"
  | "hidden"
  | "focused"
  | "blurred"
  | "resized"
  | "moved"
  | "minimized"
  | "maximized"
  | "restored"
  | "closing"
  | "closed";

/**
 * IPC message structure
 */
export interface IPCMessage {
  type: string;
  data?: unknown;
  id?: string;
}

/**
 * Command invoke request
 */
export interface InvokeRequest {
  id: string;
  command: string;
  args: Record<string, unknown>;
}

/**
 * Command invoke response
 */
export interface InvokeResponse<T = unknown> {
  id: string;
  result?: T;
  error?: CommandErrorInfo;
}

/**
 * Command error information
 */
export interface CommandErrorInfo {
  code: CommandErrorCode;
  message: string;
  details?: Record<string, unknown>;
}

/**
 * Command error codes
 */
export type CommandErrorCode =
  | "UNKNOWN"
  | "INTERNAL"
  | "INVALID_DATA"
  | "MISSING_COMMAND"
  | "COMMAND_NOT_FOUND"
  | "INVALID_ARGUMENTS"
  | "MISSING_ARGUMENT"
  | "TYPE_ERROR"
  | "EXECUTION_ERROR"
  | "TIMEOUT"
  | "CANCELLED"
  | "PERMISSION_DENIED";

/**
 * CommandError class thrown by invoke()
 */
export declare class CommandError extends Error {
  readonly code: CommandErrorCode;
  readonly details: Record<string, unknown>;
  constructor(code: CommandErrorCode, message: string, details?: Record<string, unknown>);
}

/**
 * State sync message types
 */
export interface StateSyncMessage {
  type: "set" | "delete" | "batch" | "full" | "clear";
  key?: string;
  value?: unknown;
  data?: Record<string, unknown>;
}

/**
 * Channel message types
 */
export interface ChannelMessage<T = unknown> {
  channel_id: string;
  data?: T;
}

// Export for module usage
export {};

