/**
 * Chrome Storage API
 * 
 * Provides chrome.storage.local, chrome.storage.sync, chrome.storage.session
 */

import { EventEmitter } from '../event_emitter';
import type { StorageArea, StorageChange } from '../types';

type CallNativeApi = (api: string, method: string, params?: Record<string, any>) => Promise<any>;

function createStorageArea(
  areaName: string,
  callNativeApi: CallNativeApi,
  globalOnChanged: EventEmitter<[Record<string, StorageChange>, string]>
): StorageArea {
  const onChanged = new EventEmitter<[Record<string, StorageChange>, string]>();

  const area: StorageArea = {
    async get(keys) {
      let keyList: string[] | null = null;
      let defaults: Record<string, any> = {};

      if (keys === null || keys === undefined) {
        keyList = null;
      } else if (typeof keys === 'string') {
        keyList = [keys];
      } else if (Array.isArray(keys)) {
        keyList = keys;
      } else if (typeof keys === 'object') {
        keyList = Object.keys(keys);
        defaults = keys;
      }

      const result = await callNativeApi('storage', 'get', { area: areaName, keys: keyList });
      return { ...defaults, ...result };
    },

    async set(items) {
      const oldValues = await area.get(Object.keys(items));
      await callNativeApi('storage', 'set', { area: areaName, items });

      // Trigger onChanged event
      const changes: Record<string, StorageChange> = {};
      for (const [key, newValue] of Object.entries(items)) {
        changes[key] = {
          oldValue: oldValues[key],
          newValue: newValue,
        };
      }
      globalOnChanged._dispatch(changes, areaName);
      onChanged._dispatch(changes, areaName);
    },

    async remove(keys) {
      const keyList = Array.isArray(keys) ? keys : [keys];
      const oldValues = await area.get(keyList);
      await callNativeApi('storage', 'remove', { area: areaName, keys: keyList });

      // Trigger onChanged event
      const changes: Record<string, StorageChange> = {};
      for (const key of keyList) {
        if (key in oldValues) {
          changes[key] = { oldValue: oldValues[key] };
        }
      }
      if (Object.keys(changes).length > 0) {
        globalOnChanged._dispatch(changes, areaName);
        onChanged._dispatch(changes, areaName);
      }
    },

    async clear() {
      const oldValues = await area.get(null);
      await callNativeApi('storage', 'clear', { area: areaName });

      // Trigger onChanged event
      const changes: Record<string, StorageChange> = {};
      for (const [key, value] of Object.entries(oldValues)) {
        changes[key] = { oldValue: value };
      }
      if (Object.keys(changes).length > 0) {
        globalOnChanged._dispatch(changes, areaName);
        onChanged._dispatch(changes, areaName);
      }
    },

    async getBytesInUse(keys) {
      const keyList = keys ? (Array.isArray(keys) ? keys : [keys]) : null;
      return callNativeApi('storage', 'getBytesInUse', { area: areaName, keys: keyList });
    },

    async setAccessLevel(accessOptions) {
      return callNativeApi('storage', 'setAccessLevel', { area: areaName, accessOptions });
    },

    onChanged,

    // Storage quotas
    QUOTA_BYTES: areaName === 'sync' ? 102400 : 10485760,
    QUOTA_BYTES_PER_ITEM: areaName === 'sync' ? 8192 : 10485760,
    MAX_ITEMS: areaName === 'sync' ? 512 : 10000,
    MAX_WRITE_OPERATIONS_PER_HOUR: areaName === 'sync' ? 1800 : 10000,
    MAX_WRITE_OPERATIONS_PER_MINUTE: areaName === 'sync' ? 120 : 1000,
  };

  return area;
}

export function createStorageApi(callNativeApi: CallNativeApi) {
  const onChanged = new EventEmitter<[Record<string, StorageChange>, string]>();

  return {
    local: createStorageArea('local', callNativeApi, onChanged),
    sync: createStorageArea('sync', callNativeApi, onChanged),
    session: createStorageArea('session', callNativeApi, onChanged),
    managed: createStorageArea('managed', callNativeApi, onChanged),
    onChanged,
  };
}
