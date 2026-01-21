/**
 * AuroraView SDK
 *
 * Framework-agnostic SDK for AuroraView WebView bridge.
 *
 * @packageDocumentation
 */

// Core exports
export { createAuroraView, getAuroraView } from './core/bridge';
export type { AuroraViewClient } from './core/bridge';

export { EventEmitter, getGlobalEmitter } from './core/events';

// Feature detection exports
export {
  hasFeature,
  hasFeatures,
  getAvailableFeatures,
  getEnvironment,
  isAuroraView,
  isDCC,
  isStandalone,
  isPacked,
  waitForFeature,
  Features,
} from './features';
export type {
  FeatureName,
  Platform,
  RuntimeMode,
  DCCHost,
  Environment,
} from './features';

// Window management exports
export {
  Window,
  getCurrentWindow,
  createWindow,
  closeCurrentWindow,
  startDrag,
} from './window';
export type {
  WindowOptions,
  WindowPosition,
  WindowSize,
  WindowBounds,
  WindowState,
  WindowEventHandler,
} from './window';

// Plugin exports
export { interactive } from './inject/plugins/interactive';
export type { InteractiveRegion, InteractiveConfig } from './inject/plugins/interactive';

// Type exports
export type {
  // Event types
  EventHandler,
  Unsubscribe,
  StateChangeHandler,

  // IPC types
  IPCMessage,
  CallMessage,
  EventMessage,
  InvokeMessage,
  CallResult,
  CallErrorInfo,

  // Plugin types
  PluginResult,
  DirEntry,
  FileStat,
  FileFilter,
  OpenFileOptions,
  OpenFileResult,
  OpenFilesResult,
  SaveFileOptions,
  MessageOptions,
  MessageResult,
  ConfirmResult,
  ExecuteOptions,
  ExecuteResult,
  SpawnResult,

  // Process types
  ProcessOutput,
  ProcessExit,

  // Window types
  WindowEventType,
  WindowEventData,

  // Bridge types
  AuroraViewBridge,
  AuroraViewState,
  FileSystemAPI,
  DialogAPI,
  ClipboardAPI,
  ShellAPI,
} from './core/types';
