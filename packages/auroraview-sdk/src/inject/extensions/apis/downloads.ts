/**
 * Chrome Downloads API Polyfill
 */

import type { ChromeEvent } from '../types';
import { EventEmitter } from '../event_emitter';

type NativeApiCaller = (api: string, method: string, params?: Record<string, any>) => Promise<any>;

export type State = 'in_progress' | 'interrupted' | 'complete';
export type InterruptReason =
  | 'FILE_FAILED'
  | 'FILE_ACCESS_DENIED'
  | 'FILE_NO_SPACE'
  | 'FILE_NAME_TOO_LONG'
  | 'FILE_TOO_LARGE'
  | 'FILE_VIRUS_INFECTED'
  | 'FILE_TRANSIENT_ERROR'
  | 'FILE_BLOCKED'
  | 'FILE_SECURITY_CHECK_FAILED'
  | 'FILE_TOO_SHORT'
  | 'FILE_HASH_MISMATCH'
  | 'FILE_SAME_AS_SOURCE'
  | 'NETWORK_FAILED'
  | 'NETWORK_TIMEOUT'
  | 'NETWORK_DISCONNECTED'
  | 'NETWORK_SERVER_DOWN'
  | 'NETWORK_INVALID_REQUEST'
  | 'SERVER_FAILED'
  | 'SERVER_NO_RANGE'
  | 'SERVER_BAD_CONTENT'
  | 'SERVER_UNAUTHORIZED'
  | 'SERVER_CERT_PROBLEM'
  | 'SERVER_FORBIDDEN'
  | 'SERVER_UNREACHABLE'
  | 'SERVER_CONTENT_LENGTH_MISMATCH'
  | 'SERVER_CROSS_ORIGIN_REDIRECT'
  | 'USER_CANCELED'
  | 'USER_SHUTDOWN'
  | 'CRASH';

export interface DownloadItem {
  id: number;
  url: string;
  finalUrl: string;
  referrer: string;
  filename: string;
  incognito: boolean;
  danger: string;
  mime: string;
  startTime: string;
  endTime?: string;
  estimatedEndTime?: string;
  state: State;
  paused: boolean;
  canResume: boolean;
  error?: InterruptReason;
  bytesReceived: number;
  totalBytes: number;
  fileSize: number;
  exists: boolean;
  byExtensionId?: string;
  byExtensionName?: string;
}

export interface DownloadOptions {
  url: string;
  filename?: string;
  conflictAction?: 'uniquify' | 'overwrite' | 'prompt';
  saveAs?: boolean;
  method?: 'GET' | 'POST';
  headers?: { name: string; value: string }[];
  body?: string;
}

export interface DownloadQuery {
  query?: string[];
  startedBefore?: string;
  startedAfter?: string;
  endedBefore?: string;
  endedAfter?: string;
  totalBytesGreater?: number;
  totalBytesLess?: number;
  filenameRegex?: string;
  urlRegex?: string;
  finalUrlRegex?: string;
  limit?: number;
  orderBy?: string[];
  id?: number;
  url?: string;
  finalUrl?: string;
  filename?: string;
  danger?: string;
  mime?: string;
  startTime?: string;
  endTime?: string;
  state?: State;
  paused?: boolean;
  error?: InterruptReason;
  bytesReceived?: number;
  totalBytes?: number;
  fileSize?: number;
  exists?: boolean;
}

export interface DownloadsApi {
  download(options: DownloadOptions): Promise<number>;
  search(query: DownloadQuery): Promise<DownloadItem[]>;
  pause(downloadId: number): Promise<void>;
  resume(downloadId: number): Promise<void>;
  cancel(downloadId: number): Promise<void>;
  getFileIcon(downloadId: number, options?: { size?: number }): Promise<string>;
  open(downloadId: number): Promise<void>;
  show(downloadId: number): Promise<boolean>;
  showDefaultFolder(): void;
  erase(query: DownloadQuery): Promise<number[]>;
  removeFile(downloadId: number): Promise<void>;
  acceptDanger(downloadId: number): Promise<void>;
  setShelfEnabled(enabled: boolean): void;
  setUiOptions(options: { enabled: boolean }): Promise<void>;
  onCreated: ChromeEvent<(downloadItem: DownloadItem) => void>;
  onErased: ChromeEvent<(downloadId: number) => void>;
  onChanged: ChromeEvent<(downloadDelta: { id: number; [key: string]: any }) => void>;
  onDeterminingFilename: ChromeEvent<(downloadItem: DownloadItem, suggest: (suggestion?: { filename?: string; conflictAction?: string }) => void) => void>;
}

export function createDownloadsApi(callNativeApi: NativeApiCaller): DownloadsApi {
  const onCreated = new EventEmitter<[DownloadItem]>();
  const onErased = new EventEmitter<[number]>();
  const onChanged = new EventEmitter<[{ id: number; [key: string]: any }]>();
  const onDeterminingFilename = new EventEmitter<[DownloadItem, (suggestion?: { filename?: string; conflictAction?: string }) => void]>();

  return {
    async download(options: DownloadOptions): Promise<number> {
      return callNativeApi('downloads', 'download', options);
    },

    async search(query: DownloadQuery): Promise<DownloadItem[]> {
      return callNativeApi('downloads', 'search', query);
    },

    async pause(downloadId: number): Promise<void> {
      return callNativeApi('downloads', 'pause', { downloadId });
    },

    async resume(downloadId: number): Promise<void> {
      return callNativeApi('downloads', 'resume', { downloadId });
    },

    async cancel(downloadId: number): Promise<void> {
      return callNativeApi('downloads', 'cancel', { downloadId });
    },

    async getFileIcon(downloadId: number, options?): Promise<string> {
      return callNativeApi('downloads', 'getFileIcon', { downloadId, ...options });
    },

    async open(downloadId: number): Promise<void> {
      return callNativeApi('downloads', 'open', { downloadId });
    },

    async show(downloadId: number): Promise<boolean> {
      return callNativeApi('downloads', 'show', { downloadId });
    },

    showDefaultFolder(): void {
      callNativeApi('downloads', 'showDefaultFolder', {});
    },

    async erase(query: DownloadQuery): Promise<number[]> {
      return callNativeApi('downloads', 'erase', query);
    },

    async removeFile(downloadId: number): Promise<void> {
      return callNativeApi('downloads', 'removeFile', { downloadId });
    },

    async acceptDanger(downloadId: number): Promise<void> {
      return callNativeApi('downloads', 'acceptDanger', { downloadId });
    },

    setShelfEnabled(enabled: boolean): void {
      callNativeApi('downloads', 'setShelfEnabled', { enabled });
    },

    async setUiOptions(options: { enabled: boolean }): Promise<void> {
      return callNativeApi('downloads', 'setUiOptions', options);
    },

    onCreated,
    onErased,
    onChanged,
    onDeterminingFilename,
  };
}
