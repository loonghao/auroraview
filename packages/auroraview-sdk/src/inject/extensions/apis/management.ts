/**
 * Chrome Management API Polyfill
 */

import type { ChromeEvent } from '../types';
import { EventEmitter } from '../event_emitter';

type NativeApiCaller = (api: string, method: string, params?: Record<string, any>) => Promise<any>;

export type ExtensionType = 'extension' | 'hosted_app' | 'packaged_app' | 'legacy_packaged_app' | 'theme' | 'login_screen_extension';
export type LaunchType = 'OPEN_AS_REGULAR_TAB' | 'OPEN_AS_PINNED_TAB' | 'OPEN_AS_WINDOW' | 'OPEN_FULL_SCREEN';

export interface ExtensionInfo {
  id: string;
  name: string;
  shortName: string;
  description: string;
  version: string;
  versionName?: string;
  mayDisable: boolean;
  mayEnable?: boolean;
  enabled: boolean;
  disabledReason?: 'unknown' | 'permissions_increase';
  isApp: boolean;
  type: ExtensionType;
  appLaunchUrl?: string;
  homepageUrl?: string;
  updateUrl?: string;
  offlineEnabled: boolean;
  optionsUrl: string;
  icons?: { size: number; url: string }[];
  permissions: string[];
  hostPermissions: string[];
  installType: 'admin' | 'development' | 'normal' | 'sideload' | 'other';
  launchType?: LaunchType;
  availableLaunchTypes?: LaunchType[];
}

export interface ManagementApi {
  getAll(): Promise<ExtensionInfo[]>;
  get(id: string): Promise<ExtensionInfo>;
  getSelf(): Promise<ExtensionInfo>;
  getPermissionWarningsById(id: string): Promise<string[]>;
  getPermissionWarningsByManifest(manifestStr: string): Promise<string[]>;
  setEnabled(id: string, enabled: boolean): Promise<void>;
  uninstall(id: string, options?: { showConfirmDialog?: boolean }): Promise<void>;
  uninstallSelf(options?: { showConfirmDialog?: boolean }): Promise<void>;
  launchApp(id: string): Promise<void>;
  createAppShortcut(id: string): Promise<void>;
  setLaunchType(id: string, launchType: LaunchType): Promise<void>;
  generateAppForLink(url: string, title: string): Promise<ExtensionInfo>;
  onInstalled: ChromeEvent<(info: ExtensionInfo) => void>;
  onUninstalled: ChromeEvent<(id: string) => void>;
  onEnabled: ChromeEvent<(info: ExtensionInfo) => void>;
  onDisabled: ChromeEvent<(info: ExtensionInfo) => void>;
}

export function createManagementApi(callNativeApi: NativeApiCaller): ManagementApi {
  const onInstalled = new EventEmitter<[ExtensionInfo]>();
  const onUninstalled = new EventEmitter<[string]>();
  const onEnabled = new EventEmitter<[ExtensionInfo]>();
  const onDisabled = new EventEmitter<[ExtensionInfo]>();

  return {
    async getAll(): Promise<ExtensionInfo[]> {
      return callNativeApi('management', 'getAll', {});
    },
    async get(id: string): Promise<ExtensionInfo> {
      return callNativeApi('management', 'get', { id });
    },
    async getSelf(): Promise<ExtensionInfo> {
      return callNativeApi('management', 'getSelf', {});
    },
    async getPermissionWarningsById(id: string): Promise<string[]> {
      return callNativeApi('management', 'getPermissionWarningsById', { id });
    },
    async getPermissionWarningsByManifest(manifestStr: string): Promise<string[]> {
      return callNativeApi('management', 'getPermissionWarningsByManifest', { manifestStr });
    },
    async setEnabled(id: string, enabled: boolean): Promise<void> {
      return callNativeApi('management', 'setEnabled', { id, enabled });
    },
    async uninstall(id: string, options?): Promise<void> {
      return callNativeApi('management', 'uninstall', { id, ...options });
    },
    async uninstallSelf(options?): Promise<void> {
      return callNativeApi('management', 'uninstallSelf', options || {});
    },
    async launchApp(id: string): Promise<void> {
      return callNativeApi('management', 'launchApp', { id });
    },
    async createAppShortcut(id: string): Promise<void> {
      return callNativeApi('management', 'createAppShortcut', { id });
    },
    async setLaunchType(id: string, launchType: LaunchType): Promise<void> {
      return callNativeApi('management', 'setLaunchType', { id, launchType });
    },
    async generateAppForLink(url: string, title: string): Promise<ExtensionInfo> {
      return callNativeApi('management', 'generateAppForLink', { url, title });
    },
    onInstalled,
    onUninstalled,
    onEnabled,
    onDisabled,
  };
}
