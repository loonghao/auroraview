/**
 * AuroraView SDK Core Types
 *
 * Type definitions for the AuroraView bridge API.
 */

// ============================================
// Event System Types
// ============================================

/** Event handler function */
export type EventHandler<T = unknown> = (data: T) => void;

/** Unsubscribe function returned by event subscriptions */
export type Unsubscribe = () => void;

/** State change handler function */
export type StateChangeHandler = (
  key: string,
  value: unknown,
  source: 'python' | 'javascript'
) => void;

// ============================================
// IPC Types
// ============================================

/** IPC message payload types */
export type IPCMessageType = 'call' | 'event' | 'invoke';

/** Base IPC message structure */
export interface IPCMessage {
  type: IPCMessageType;
  id?: string;
}

/** Call message (JS -> Python RPC) */
export interface CallMessage extends IPCMessage {
  type: 'call';
  id: string;
  method: string;
  params?: unknown;
}

/** Event message (JS -> Python fire-and-forget) */
export interface EventMessage extends IPCMessage {
  type: 'event';
  event: string;
  detail?: unknown;
}

/** Invoke message (JS -> Python plugin command) */
export interface InvokeMessage extends IPCMessage {
  type: 'invoke';
  id: string;
  cmd: string;
  args: Record<string, unknown>;
}

/** Call result from Python */
export interface CallResult<T = unknown> {
  id: string;
  ok: boolean;
  result?: T;
  error?: CallErrorInfo;
}

/** Error information from failed calls */
export interface CallErrorInfo {
  name?: string;
  message: string;
  code?: string | number;
  data?: unknown;
}

// ============================================
// Pending Call Types
// ============================================

/** Pending call entry for Promise tracking */
export interface PendingCall<T = unknown> {
  resolve: (value: T) => void;
  reject: (error: Error) => void;
}

// ============================================
// State Types
// ============================================

/** State sync message from Python */
export interface StateSyncMessage {
  type: 'set' | 'delete' | 'batch' | 'full' | 'clear';
  key?: string;
  value?: unknown;
  data?: Record<string, unknown>;
}

// ============================================
// Plugin Types
// ============================================

/** Plugin invoke result */
export interface PluginResult<T = unknown> {
  success?: boolean;
  error?: string;
  code?: string;
  [key: string]: T | boolean | string | undefined;
}

// ============================================
// File System Plugin Types
// ============================================

/** Directory entry from readDir */
export interface DirEntry {
  name: string;
  path: string;
  isDir: boolean;
  isFile: boolean;
  size?: number;
  modified?: number;
  created?: number;
}

/** File stat result */
export interface FileStat {
  size: number;
  isDir: boolean;
  isFile: boolean;
  isSymlink: boolean;
  modified?: number;
  created?: number;
  accessed?: number;
  readonly?: boolean;
}

// ============================================
// Dialog Plugin Types
// ============================================

/** File filter for dialogs */
export interface FileFilter {
  name: string;
  extensions: string[];
}

/** Open file dialog options */
export interface OpenFileOptions {
  title?: string;
  defaultPath?: string;
  filters?: FileFilter[];
}

/** Open file result */
export interface OpenFileResult {
  path: string | null;
  cancelled: boolean;
}

/** Open files result */
export interface OpenFilesResult {
  paths: string[];
  cancelled: boolean;
}

/** Save file dialog options */
export interface SaveFileOptions {
  title?: string;
  defaultPath?: string;
  defaultName?: string;
  filters?: FileFilter[];
}

/** Message dialog options */
export interface MessageOptions {
  message: string;
  title?: string;
  level?: 'info' | 'warning' | 'error';
  buttons?: 'ok' | 'ok_cancel' | 'yes_no' | 'yes_no_cancel';
}

/** Message dialog result */
export interface MessageResult {
  response: 'ok' | 'cancel' | 'yes' | 'no';
}

/** Confirm dialog result */
export interface ConfirmResult {
  confirmed: boolean;
}

// ============================================
// Shell Plugin Types
// ============================================

/** Shell execute options */
export interface ExecuteOptions {
  cwd?: string;
  env?: Record<string, string>;
}

/** Shell execute result */
export interface ExecuteResult {
  code: number | null;
  stdout: string;
  stderr: string;
}

/** Shell spawn result */
export interface SpawnResult {
  success: boolean;
  pid: number;
}

// ============================================
// Process Event Types
// ============================================

/** Process output event data */
export interface ProcessOutput {
  pid: number;
  data: string;
}

/** Process exit event data */
export interface ProcessExit {
  pid: number;
  code: number | null;
}

// ============================================
// Window Types
// ============================================

/** Window event types */
export type WindowEventType =
  | 'shown'
  | 'hidden'
  | 'focused'
  | 'blurred'
  | 'resized'
  | 'moved'
  | 'minimized'
  | 'maximized'
  | 'restored'
  | 'closing'
  | 'closed';

/** Window event data */
export interface WindowEventData {
  type: WindowEventType;
  x?: number;
  y?: number;
  width?: number;
  height?: number;
  timestamp?: number;
}

// ============================================
// Global Window Extension
// ============================================

declare global {
  interface Window {
    auroraview?: AuroraViewBridge;
    ipc?: {
      postMessage(message: string): void;
    };
  }
}

/** Raw bridge interface exposed on window.auroraview */
export interface AuroraViewBridge {
  _ready: boolean;
  _isStub?: boolean;
  _pendingCalls: unknown[];
  _boundMethods: Record<string, boolean>;

  call<T = unknown>(method: string, params?: unknown): Promise<T>;
  invoke<T = unknown>(cmd: string, args?: Record<string, unknown>): Promise<T>;
  send_event(event: string, detail?: unknown): void;
  on(event: string, handler: EventHandler): Unsubscribe;
  off?(event: string, handler?: EventHandler): void;
  trigger(event: string, detail?: unknown): void;
  whenReady(): Promise<AuroraViewBridge>;
  isReady(): boolean;
  isMethodBound?(fullMethodName: string): boolean;
  getBoundMethods?(): string[];
  _registerApiMethods?(
    namespace: string,
    methods: string[],
    options?: { allowRebind?: boolean }
  ): void;

  /** Start native window drag (for frameless windows) */
  startDrag?(): void;

  api?: Record<string, (params?: unknown) => Promise<unknown>>;
  state?: AuroraViewState;
  fs?: FileSystemAPI;
  dialog?: DialogAPI;
  clipboard?: ClipboardAPI;
  shell?: ShellAPI;
}

/** State proxy interface */
export interface AuroraViewState {
  [key: string]: unknown;
  onChange(handler: StateChangeHandler): Unsubscribe;
  offChange(handler: StateChangeHandler): void;
  toJSON(): Record<string, unknown>;
  keys(): string[];
}

/** File system API interface */
export interface FileSystemAPI {
  readFile(path: string, encoding?: string): Promise<string>;
  readFileBinary(path: string): Promise<string>;
  readFileBuffer(path: string): Promise<ArrayBuffer>;
  writeFile(path: string, contents: string, append?: boolean): Promise<void>;
  writeFileBinary(
    path: string,
    contents: ArrayBuffer | Uint8Array,
    append?: boolean
  ): Promise<void>;
  readDir(path: string, recursive?: boolean): Promise<DirEntry[]>;
  createDir(path: string, recursive?: boolean): Promise<void>;
  remove(path: string, recursive?: boolean): Promise<void>;
  copy(from: string, to: string): Promise<void>;
  rename(from: string, to: string): Promise<void>;
  exists(path: string): Promise<boolean>;
  stat(path: string): Promise<FileStat>;
}

/** Dialog API interface */
export interface DialogAPI {
  openFile(options?: OpenFileOptions): Promise<OpenFileResult>;
  openFiles(options?: OpenFileOptions): Promise<OpenFilesResult>;
  openFolder(options?: OpenFileOptions): Promise<OpenFileResult>;
  openFolders(options?: OpenFileOptions): Promise<OpenFilesResult>;
  saveFile(options?: SaveFileOptions): Promise<OpenFileResult>;
  message(options: MessageOptions): Promise<MessageResult>;
  confirm(options: { message: string; title?: string }): Promise<ConfirmResult>;
  info(message: string, title?: string): Promise<MessageResult>;
  warning(message: string, title?: string): Promise<MessageResult>;
  error(message: string, title?: string): Promise<MessageResult>;
  ask(message: string, title?: string): Promise<boolean>;
}

/** Clipboard API interface */
export interface ClipboardAPI {
  readText(): Promise<string>;
  writeText(text: string): Promise<void>;
  clear(): Promise<void>;
  hasText(): Promise<boolean>;
  readImage(): Promise<string | null>;
  writeImage(base64: string): Promise<void>;
}

/** Shell API interface */
export interface ShellAPI {
  open(url: string): Promise<void>;
  openPath(path: string): Promise<void>;
  showInFolder(path: string): Promise<void>;
  execute(
    command: string,
    args?: string[],
    options?: ExecuteOptions
  ): Promise<ExecuteResult>;
  spawn(
    command: string,
    args?: string[],
    options?: ExecuteOptions
  ): Promise<SpawnResult>;
  which(command: string): Promise<string | null>;
  getEnv(name: string): Promise<string | null>;
  getEnvAll(): Promise<Record<string, string>>;
}

export {};
