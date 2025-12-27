/**
 * Chrome Action API Polyfill
 */

import type { ChromeEvent, Tab } from '../types';
import { EventEmitter } from '../event_emitter';

type NativeApiCaller = (api: string, method: string, params?: Record<string, any>) => Promise<any>;

export interface ActionApi {
  setIcon(details: { imageData?: any; path?: string | Record<string, string>; tabId?: number }): Promise<void>;
  setTitle(details: { title: string; tabId?: number }): Promise<void>;
  getTitle(details: { tabId?: number }): Promise<string>;
  setPopup(details: { popup: string; tabId?: number }): Promise<void>;
  getPopup(details: { tabId?: number }): Promise<string>;
  setBadgeText(details: { text: string; tabId?: number }): Promise<void>;
  getBadgeText(details: { tabId?: number }): Promise<string>;
  setBadgeBackgroundColor(details: { color: string | [number, number, number, number]; tabId?: number }): Promise<void>;
  getBadgeBackgroundColor(details: { tabId?: number }): Promise<[number, number, number, number]>;
  setBadgeTextColor(details: { color: string | [number, number, number, number]; tabId?: number }): Promise<void>;
  getBadgeTextColor(details: { tabId?: number }): Promise<[number, number, number, number]>;
  enable(tabId?: number): Promise<void>;
  disable(tabId?: number): Promise<void>;
  isEnabled(details: { tabId?: number }): Promise<boolean>;
  openPopup(options?: { windowId?: number }): Promise<void>;
  onClicked: ChromeEvent<(tab: Tab) => void>;
}

export function createActionApi(callNativeApi: NativeApiCaller): ActionApi {
  const onClicked = new EventEmitter<[Tab]>();

  return {
    async setIcon(details): Promise<void> {
      return callNativeApi('action', 'setIcon', details);
    },

    async setTitle(details): Promise<void> {
      return callNativeApi('action', 'setTitle', details);
    },

    async getTitle(details): Promise<string> {
      return callNativeApi('action', 'getTitle', details);
    },

    async setPopup(details): Promise<void> {
      return callNativeApi('action', 'setPopup', details);
    },

    async getPopup(details): Promise<string> {
      return callNativeApi('action', 'getPopup', details);
    },

    async setBadgeText(details): Promise<void> {
      return callNativeApi('action', 'setBadgeText', details);
    },

    async getBadgeText(details): Promise<string> {
      return callNativeApi('action', 'getBadgeText', details);
    },

    async setBadgeBackgroundColor(details): Promise<void> {
      return callNativeApi('action', 'setBadgeBackgroundColor', details);
    },

    async getBadgeBackgroundColor(details): Promise<[number, number, number, number]> {
      return callNativeApi('action', 'getBadgeBackgroundColor', details);
    },

    async setBadgeTextColor(details): Promise<void> {
      return callNativeApi('action', 'setBadgeTextColor', details);
    },

    async getBadgeTextColor(details): Promise<[number, number, number, number]> {
      return callNativeApi('action', 'getBadgeTextColor', details);
    },

    async enable(tabId?: number): Promise<void> {
      return callNativeApi('action', 'enable', { tabId });
    },

    async disable(tabId?: number): Promise<void> {
      return callNativeApi('action', 'disable', { tabId });
    },

    async isEnabled(details): Promise<boolean> {
      return callNativeApi('action', 'isEnabled', details);
    },

    async openPopup(options): Promise<void> {
      return callNativeApi('action', 'openPopup', options || {});
    },

    onClicked,
  };
}
