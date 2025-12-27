/**
 * Chrome Runtime API Polyfill
 *
 * Provides chrome.runtime for extension lifecycle and messaging.
 */

import type { ChromeEvent, Port } from '../types';
import { EventEmitter } from '../event_emitter';

type NativeApiCaller = (api: string, method: string, params?: Record<string, any>) => Promise<any>;

export interface RuntimeApi {
  id: string;
  getURL(path: string): string;
  getManifest(): any;
  getPlatformInfo(): Promise<{ os: string; arch: string; nacl_arch: string }>;
  sendMessage(extensionIdOrMessage: string | any, messageOrOptions?: any, options?: any): Promise<any>;
  connect(extensionIdOrConnectInfo?: string | { name?: string }, connectInfo?: { name?: string }): Port;
  openOptionsPage(): Promise<void>;
  setUninstallURL(url: string): Promise<void>;
  reload(): void;
  requestUpdateCheck(): Promise<[string, any]>;
  getBackgroundPage(callback?: (page: Window | null) => void): Promise<Window | null>;
  getContexts(filter?: any): Promise<any[]>;
  getPackageDirectoryEntry(callback?: (entry: any) => void): void;
  onInstalled: ChromeEvent<(details: { reason: string; previousVersion?: string }) => void>;
  onStartup: ChromeEvent<() => void>;
  onSuspend: ChromeEvent<() => void>;
  onSuspendCanceled: ChromeEvent<() => void>;
  onUpdateAvailable: ChromeEvent<(details: { version: string }) => void>;
  onMessage: ChromeEvent<(message: any, sender: { id: string }, sendResponse: (response: any) => void) => void>;
  onMessageExternal: ChromeEvent<(message: any, sender: { id: string }, sendResponse: (response: any) => void) => void>;
  onConnect: ChromeEvent<(port: Port) => void>;
  onConnectExternal: ChromeEvent<(port: Port) => void>;
  onConnectNative: ChromeEvent<(port: Port) => void>;
  lastError: { message: string } | null;
}

export function createRuntimeApi(
  callNativeApi: NativeApiCaller,
  extensionId: string,
  manifest: any = {},
  getLastError: () => { message: string } | null = () => null,
  _setLastError?: (err: { message: string } | null) => void
): RuntimeApi {
  const onInstalled = new EventEmitter<[{ reason: string; previousVersion?: string }]>();
  const onStartup = new EventEmitter<[]>();
  const onSuspend = new EventEmitter<[]>();
  const onSuspendCanceled = new EventEmitter<[]>();
  const onUpdateAvailable = new EventEmitter<[{ version: string }]>();
  const onMessage = new EventEmitter<[any, { id: string }, (response: any) => void]>();
  const onMessageExternal = new EventEmitter<[any, { id: string }, (response: any) => void]>();
  const onConnect = new EventEmitter<[Port]>();
  const onConnectExternal = new EventEmitter<[Port]>();
  const onConnectNative = new EventEmitter<[Port]>();

  return {
    id: extensionId,

    getURL(path: string): string {
      const cleanPath = path.replace(/^\//, '');
      return `chrome-extension://${extensionId}/${cleanPath}`;
    },

    getManifest(): any {
      return manifest;
    },

    async getPlatformInfo(): Promise<{ os: string; arch: string; nacl_arch: string }> {
      // Detect platform info from browser
      const userAgent = navigator.userAgent.toLowerCase();
      let os = 'win';
      if (userAgent.includes('mac')) os = 'mac';
      else if (userAgent.includes('linux')) os = 'linux';
      else if (userAgent.includes('cros')) os = 'cros';
      else if (userAgent.includes('android')) os = 'android';

      let arch = 'x86-64';
      if (userAgent.includes('arm')) arch = 'arm';
      else if (userAgent.includes('x86')) arch = 'x86-32';

      return { os, arch, nacl_arch: arch };
    },

    async sendMessage(
      extensionIdOrMessage: string | any,
      messageOrOptions?: any,
      options?: any
    ): Promise<any> {
      let targetExtensionId = extensionId;
      let message = extensionIdOrMessage;

      if (typeof extensionIdOrMessage === 'string' && extensionIdOrMessage !== extensionId) {
        targetExtensionId = extensionIdOrMessage;
        message = messageOrOptions;
      } else if (typeof extensionIdOrMessage === 'object') {
        options = messageOrOptions;
        message = extensionIdOrMessage;
      }

      return callNativeApi('runtime', 'sendMessage', {
        extensionId: targetExtensionId,
        message,
        options,
      });
    },

    connect(extensionIdOrConnectInfo?: string | { name?: string }, connectInfo?: { name?: string }): Port {
      let targetExtensionId = extensionId;
      let info = connectInfo;

      if (typeof extensionIdOrConnectInfo === 'object') {
        info = extensionIdOrConnectInfo;
      } else if (typeof extensionIdOrConnectInfo === 'string') {
        targetExtensionId = extensionIdOrConnectInfo;
      }

      const portId = `port_${Date.now()}_${Math.random().toString(36).substring(2, 11)}`;
      const portOnMessage = new EventEmitter<[any]>();
      const portOnDisconnect = new EventEmitter<[]>();

      let connected = true;

      const port: Port = {
        name: info?.name || '',
        sender: { id: extensionId },
        onMessage: portOnMessage,
        onDisconnect: portOnDisconnect,
        postMessage(message: any) {
          if (!connected) {
            throw new Error('Attempting to use a disconnected port object');
          }
          callNativeApi('runtime', 'portPostMessage', { portId, message });
        },
        disconnect() {
          if (connected) {
            connected = false;
            callNativeApi('runtime', 'portDisconnect', { portId });
            portOnDisconnect._dispatch();
          }
        },
      };

      callNativeApi('runtime', 'connect', {
        extensionId: targetExtensionId,
        portId,
        name: port.name,
      });

      return port;
    },

    async openOptionsPage(): Promise<void> {
      return callNativeApi('runtime', 'openOptionsPage', {});
    },

    async setUninstallURL(url: string): Promise<void> {
      return callNativeApi('runtime', 'setUninstallURL', { url });
    },

    reload(): void {
      callNativeApi('runtime', 'reload', {});
    },

    async requestUpdateCheck(): Promise<[string, any]> {
      return callNativeApi('runtime', 'requestUpdateCheck', {});
    },

    getBackgroundPage(callback?: (page: Window | null) => void): Promise<Window | null> {
      if (callback) callback(null);
      return Promise.resolve(null);
    },

    async getContexts(filter?: any): Promise<any[]> {
      return callNativeApi('runtime', 'getContexts', { filter });
    },

    getPackageDirectoryEntry(callback?: (entry: any) => void): void {
      if (callback) callback(null);
    },

    onInstalled,
    onStartup,
    onSuspend,
    onSuspendCanceled,
    onUpdateAvailable,
    onMessage,
    onMessageExternal,
    onConnect,
    onConnectExternal,
    onConnectNative,

    get lastError() {
      return getLastError();
    },
  };
}
