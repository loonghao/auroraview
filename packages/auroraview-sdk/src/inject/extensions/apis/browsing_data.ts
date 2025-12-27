/**
 * Chrome BrowsingData API Polyfill
 */

type NativeApiCaller = (api: string, method: string, params?: Record<string, any>) => Promise<any>;

export interface RemovalOptions {
  since?: number;
  origins?: string[];
  excludeOrigins?: string[];
}

export interface DataTypeSet {
  appcache?: boolean;
  cache?: boolean;
  cacheStorage?: boolean;
  cookies?: boolean;
  downloads?: boolean;
  fileSystems?: boolean;
  formData?: boolean;
  history?: boolean;
  indexedDB?: boolean;
  localStorage?: boolean;
  passwords?: boolean;
  serviceWorkers?: boolean;
  webSQL?: boolean;
}

export interface SettingsResult {
  options: RemovalOptions;
  dataToRemove: DataTypeSet;
  dataRemovalPermitted: DataTypeSet;
}

export interface BrowsingDataApi {
  settings(): Promise<SettingsResult>;
  remove(options: RemovalOptions, dataToRemove: DataTypeSet): Promise<void>;
  removeAppcache(options: RemovalOptions): Promise<void>;
  removeCache(options: RemovalOptions): Promise<void>;
  removeCacheStorage(options: RemovalOptions): Promise<void>;
  removeCookies(options: RemovalOptions): Promise<void>;
  removeDownloads(options: RemovalOptions): Promise<void>;
  removeFileSystems(options: RemovalOptions): Promise<void>;
  removeFormData(options: RemovalOptions): Promise<void>;
  removeHistory(options: RemovalOptions): Promise<void>;
  removeIndexedDB(options: RemovalOptions): Promise<void>;
  removeLocalStorage(options: RemovalOptions): Promise<void>;
  removePasswords(options: RemovalOptions): Promise<void>;
  removeServiceWorkers(options: RemovalOptions): Promise<void>;
  removeWebSQL(options: RemovalOptions): Promise<void>;
}

export function createBrowsingDataApi(callNativeApi: NativeApiCaller): BrowsingDataApi {
  return {
    async settings(): Promise<SettingsResult> {
      return callNativeApi('browsingData', 'settings', {});
    },

    async remove(options: RemovalOptions, dataToRemove: DataTypeSet): Promise<void> {
      return callNativeApi('browsingData', 'remove', { options, dataToRemove });
    },

    async removeAppcache(options: RemovalOptions): Promise<void> {
      return callNativeApi('browsingData', 'removeAppcache', options);
    },

    async removeCache(options: RemovalOptions): Promise<void> {
      return callNativeApi('browsingData', 'removeCache', options);
    },

    async removeCacheStorage(options: RemovalOptions): Promise<void> {
      return callNativeApi('browsingData', 'removeCacheStorage', options);
    },

    async removeCookies(options: RemovalOptions): Promise<void> {
      return callNativeApi('browsingData', 'removeCookies', options);
    },

    async removeDownloads(options: RemovalOptions): Promise<void> {
      return callNativeApi('browsingData', 'removeDownloads', options);
    },

    async removeFileSystems(options: RemovalOptions): Promise<void> {
      return callNativeApi('browsingData', 'removeFileSystems', options);
    },

    async removeFormData(options: RemovalOptions): Promise<void> {
      return callNativeApi('browsingData', 'removeFormData', options);
    },

    async removeHistory(options: RemovalOptions): Promise<void> {
      return callNativeApi('browsingData', 'removeHistory', options);
    },

    async removeIndexedDB(options: RemovalOptions): Promise<void> {
      return callNativeApi('browsingData', 'removeIndexedDB', options);
    },

    async removeLocalStorage(options: RemovalOptions): Promise<void> {
      return callNativeApi('browsingData', 'removeLocalStorage', options);
    },

    async removePasswords(options: RemovalOptions): Promise<void> {
      return callNativeApi('browsingData', 'removePasswords', options);
    },

    async removeServiceWorkers(options: RemovalOptions): Promise<void> {
      return callNativeApi('browsingData', 'removeServiceWorkers', options);
    },

    async removeWebSQL(options: RemovalOptions): Promise<void> {
      return callNativeApi('browsingData', 'removeWebSQL', options);
    },
  };
}
