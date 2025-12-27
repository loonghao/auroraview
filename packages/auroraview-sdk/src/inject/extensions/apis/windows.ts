/**
 * Chrome Windows API Polyfill
 */

import type { ChromeEvent, Tab } from '../types';
import { EventEmitter } from '../event_emitter';

type NativeApiCaller = (api: string, method: string, params?: Record<string, any>) => Promise<any>;

export type WindowType = 'normal' | 'popup' | 'panel' | 'app' | 'devtools';
export type WindowState = 'normal' | 'minimized' | 'maximized' | 'fullscreen' | 'locked-fullscreen';

export interface Window {
  id?: number;
  focused: boolean;
  top?: number;
  left?: number;
  width?: number;
  height?: number;
  tabs?: Tab[];
  incognito: boolean;
  type?: WindowType;
  state?: WindowState;
  alwaysOnTop: boolean;
  sessionId?: string;
}

export interface CreateData {
  url?: string | string[];
  tabId?: number;
  left?: number;
  top?: number;
  width?: number;
  height?: number;
  focused?: boolean;
  incognito?: boolean;
  type?: WindowType;
  state?: WindowState;
  setSelfAsOpener?: boolean;
}

export interface UpdateInfo {
  left?: number;
  top?: number;
  width?: number;
  height?: number;
  focused?: boolean;
  drawAttention?: boolean;
  state?: WindowState;
}

export interface WindowsApi {
  get(windowId: number, queryOptions?: { populate?: boolean; windowTypes?: WindowType[] }): Promise<Window>;
  getCurrent(queryOptions?: { populate?: boolean; windowTypes?: WindowType[] }): Promise<Window>;
  getLastFocused(queryOptions?: { populate?: boolean; windowTypes?: WindowType[] }): Promise<Window>;
  getAll(queryOptions?: { populate?: boolean; windowTypes?: WindowType[] }): Promise<Window[]>;
  create(createData?: CreateData): Promise<Window>;
  update(windowId: number, updateInfo: UpdateInfo): Promise<Window>;
  remove(windowId: number): Promise<void>;
  onCreated: ChromeEvent<(window: Window) => void>;
  onRemoved: ChromeEvent<(windowId: number) => void>;
  onFocusChanged: ChromeEvent<(windowId: number) => void>;
  onBoundsChanged: ChromeEvent<(window: Window) => void>;
  WINDOW_ID_NONE: number;
  WINDOW_ID_CURRENT: number;
}

export function createWindowsApi(callNativeApi: NativeApiCaller): WindowsApi {
  const onCreated = new EventEmitter<[Window]>();
  const onRemoved = new EventEmitter<[number]>();
  const onFocusChanged = new EventEmitter<[number]>();
  const onBoundsChanged = new EventEmitter<[Window]>();

  return {
    async get(windowId: number, queryOptions?): Promise<Window> {
      return callNativeApi('windows', 'get', { windowId, ...queryOptions });
    },

    async getCurrent(queryOptions?): Promise<Window> {
      return callNativeApi('windows', 'getCurrent', queryOptions || {});
    },

    async getLastFocused(queryOptions?): Promise<Window> {
      return callNativeApi('windows', 'getLastFocused', queryOptions || {});
    },

    async getAll(queryOptions?): Promise<Window[]> {
      return callNativeApi('windows', 'getAll', queryOptions || {});
    },

    async create(createData?: CreateData): Promise<Window> {
      return callNativeApi('windows', 'create', createData || {});
    },

    async update(windowId: number, updateInfo: UpdateInfo): Promise<Window> {
      return callNativeApi('windows', 'update', { windowId, ...updateInfo });
    },

    async remove(windowId: number): Promise<void> {
      return callNativeApi('windows', 'remove', { windowId });
    },

    onCreated,
    onRemoved,
    onFocusChanged,
    onBoundsChanged,
    WINDOW_ID_NONE: -1,
    WINDOW_ID_CURRENT: -2,
  };
}
