/**
 * Chrome Extension API Type Definitions
 * 
 * Type definitions for the Chrome Extension API polyfill.
 */

export interface ChromeEvent<T extends (...args: any[]) => void = (...args: any[]) => void> {
  addListener(callback: T): void;
  removeListener(callback: T): void;
  hasListener(callback: T): boolean;
  hasListeners(): boolean;
  /** @internal */
  _dispatch?(...args: Parameters<T>): void;
}

export interface StorageChange {
  oldValue?: any;
  newValue?: any;
}

export interface StorageArea {
  get(keys?: string | string[] | Record<string, any> | null): Promise<Record<string, any>>;
  set(items: Record<string, any>): Promise<void>;
  remove(keys: string | string[]): Promise<void>;
  clear(): Promise<void>;
  getBytesInUse(keys?: string | string[] | null): Promise<number>;
  setAccessLevel?(accessOptions: { accessLevel: string }): Promise<void>;
  onChanged: ChromeEvent<(changes: Record<string, StorageChange>, areaName: string) => void>;
  QUOTA_BYTES: number;
  QUOTA_BYTES_PER_ITEM: number;
  MAX_ITEMS: number;
  MAX_WRITE_OPERATIONS_PER_HOUR: number;
  MAX_WRITE_OPERATIONS_PER_MINUTE: number;
}

export interface Port {
  name: string;
  sender?: { id: string };
  onMessage: ChromeEvent<(message: any) => void>;
  onDisconnect: ChromeEvent<() => void>;
  postMessage(message: any): void;
  disconnect(): void;
}

export interface Tab {
  id?: number;
  windowId?: number;
  url?: string;
  title?: string;
  active?: boolean;
  pinned?: boolean;
  status?: string;
}

export interface NativeApiParams {
  extensionId: string;
  api: string;
  method: string;
  params: Record<string, any>;
}

export interface ExtensionEventData {
  extensionId: string;
  api: string;
  event: string;
  args?: any[];
}

export interface RuntimeMessageData {
  messageId: string;
  targetExtensionId?: string;
  message: any;
  sender: { id: string };
}
