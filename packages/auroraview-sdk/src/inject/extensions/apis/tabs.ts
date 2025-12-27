/**
 * Chrome Tabs API
 * 
 * Provides chrome.tabs for tab management.
 */

import { EventEmitter } from '../event_emitter';
import type { Tab } from '../types';

type CallNativeApi = (api: string, method: string, params?: Record<string, any>) => Promise<any>;

export function createTabsApi(callNativeApi: CallNativeApi) {
  return {
    TAB_ID_NONE: -1,

    async query(queryInfo?: Record<string, any>): Promise<Tab[]> {
      return callNativeApi('tabs', 'query', queryInfo || {});
    },

    async get(tabId: number): Promise<Tab> {
      return callNativeApi('tabs', 'get', { tabId });
    },

    async getCurrent(): Promise<Tab | null> {
      const tabs = await callNativeApi('tabs', 'query', { active: true, currentWindow: true });
      return tabs[0] || null;
    },

    async create(createProperties: Record<string, any>): Promise<Tab> {
      return callNativeApi('tabs', 'create', createProperties);
    },

    async update(tabIdOrProps: number | Record<string, any>, updateProperties?: Record<string, any>): Promise<Tab> {
      if (typeof tabIdOrProps === 'object') {
        return callNativeApi('tabs', 'update', tabIdOrProps);
      }
      return callNativeApi('tabs', 'update', { tabId: tabIdOrProps, ...updateProperties });
    },

    async remove(tabIds: number | number[]): Promise<void> {
      const ids = Array.isArray(tabIds) ? tabIds : [tabIds];
      return callNativeApi('tabs', 'remove', { tabIds: ids });
    },

    async reload(tabIdOrProps?: number | Record<string, any>, reloadProperties?: Record<string, any>): Promise<void> {
      if (typeof tabIdOrProps === 'object') {
        return callNativeApi('tabs', 'reload', tabIdOrProps);
      }
      return callNativeApi('tabs', 'reload', { tabId: tabIdOrProps, ...reloadProperties });
    },

    async duplicate(tabId: number): Promise<Tab> {
      return callNativeApi('tabs', 'duplicate', { tabId });
    },

    async sendMessage(tabId: number, message: any, options?: Record<string, any>): Promise<any> {
      return callNativeApi('tabs', 'sendMessage', { tabId, message, options });
    },

    async captureVisibleTab(windowIdOrOptions?: number | Record<string, any>, options?: Record<string, any>): Promise<string> {
      if (typeof windowIdOrOptions === 'object') {
        return callNativeApi('tabs', 'captureVisibleTab', windowIdOrOptions);
      }
      return callNativeApi('tabs', 'captureVisibleTab', { windowId: windowIdOrOptions, options });
    },

    async executeScript(tabIdOrDetails: number | Record<string, any>, details?: Record<string, any>): Promise<any[]> {
      if (typeof tabIdOrDetails === 'object') {
        return callNativeApi('tabs', 'executeScript', tabIdOrDetails);
      }
      return callNativeApi('tabs', 'executeScript', { tabId: tabIdOrDetails, ...details });
    },

    async insertCSS(tabIdOrDetails: number | Record<string, any>, details?: Record<string, any>): Promise<void> {
      if (typeof tabIdOrDetails === 'object') {
        return callNativeApi('tabs', 'insertCSS', tabIdOrDetails);
      }
      return callNativeApi('tabs', 'insertCSS', { tabId: tabIdOrDetails, ...details });
    },

    async removeCSS(tabIdOrDetails: number | Record<string, any>, details?: Record<string, any>): Promise<void> {
      if (typeof tabIdOrDetails === 'object') {
        return callNativeApi('tabs', 'removeCSS', tabIdOrDetails);
      }
      return callNativeApi('tabs', 'removeCSS', { tabId: tabIdOrDetails, ...details });
    },

    async setZoom(tabIdOrFactor: number, zoomFactor?: number): Promise<void> {
      if (typeof zoomFactor === 'undefined') {
        return callNativeApi('tabs', 'setZoom', { zoomFactor: tabIdOrFactor });
      }
      return callNativeApi('tabs', 'setZoom', { tabId: tabIdOrFactor, zoomFactor });
    },

    async getZoom(tabId?: number): Promise<number> {
      return callNativeApi('tabs', 'getZoom', { tabId });
    },

    async group(options: Record<string, any>): Promise<number> {
      return callNativeApi('tabs', 'group', options);
    },

    async ungroup(tabIds: number | number[]): Promise<void> {
      return callNativeApi('tabs', 'ungroup', { tabIds: Array.isArray(tabIds) ? tabIds : [tabIds] });
    },

    // Events
    onCreated: new EventEmitter<[Tab]>(),
    onUpdated: new EventEmitter<[number, Record<string, any>, Tab]>(),
    onMoved: new EventEmitter<[number, { windowId: number; fromIndex: number; toIndex: number }]>(),
    onActivated: new EventEmitter<[{ tabId: number; windowId: number }]>(),
    onHighlighted: new EventEmitter<[{ windowId: number; tabIds: number[] }]>(),
    onDetached: new EventEmitter<[number, { oldWindowId: number; oldPosition: number }]>(),
    onAttached: new EventEmitter<[number, { newWindowId: number; newPosition: number }]>(),
    onRemoved: new EventEmitter<[number, { windowId: number; isWindowClosing: boolean }]>(),
    onReplaced: new EventEmitter<[number, number]>(),
    onZoomChange: new EventEmitter<[{ tabId: number; oldZoomFactor: number; newZoomFactor: number; zoomSettings: any }]>(),
  };
}
