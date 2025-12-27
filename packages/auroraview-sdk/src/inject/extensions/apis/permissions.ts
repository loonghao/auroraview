/**
 * Chrome Permissions API Polyfill
 */

import type { ChromeEvent } from '../types';
import { EventEmitter } from '../event_emitter';

type NativeApiCaller = (api: string, method: string, params?: Record<string, any>) => Promise<any>;

export interface Permissions {
  permissions?: string[];
  origins?: string[];
}

export interface PermissionsApi {
  getAll(): Promise<Permissions>;
  contains(permissions: Permissions): Promise<boolean>;
  request(permissions: Permissions): Promise<boolean>;
  remove(permissions: Permissions): Promise<boolean>;
  onAdded: ChromeEvent<(permissions: Permissions) => void>;
  onRemoved: ChromeEvent<(permissions: Permissions) => void>;
}

export function createPermissionsApi(callNativeApi: NativeApiCaller): PermissionsApi {
  const onAdded = new EventEmitter<[Permissions]>();
  const onRemoved = new EventEmitter<[Permissions]>();

  return {
    async getAll(): Promise<Permissions> {
      return callNativeApi('permissions', 'getAll', {});
    },

    async contains(permissions: Permissions): Promise<boolean> {
      return callNativeApi('permissions', 'contains', { permissions });
    },

    async request(permissions: Permissions): Promise<boolean> {
      return callNativeApi('permissions', 'request', { permissions });
    },

    async remove(permissions: Permissions): Promise<boolean> {
      return callNativeApi('permissions', 'remove', { permissions });
    },

    onAdded,
    onRemoved,
  };
}
