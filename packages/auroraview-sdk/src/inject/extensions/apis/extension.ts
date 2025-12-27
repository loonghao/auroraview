/**
 * Chrome Extension API Polyfill
 */

type NativeApiCaller = (api: string, method: string, params?: Record<string, any>) => Promise<any>;

export interface ExtensionApi {
  getURL(path: string): string;
  getBackgroundPage(): Promise<Window | null>;
  getViews(fetchProperties?: { type?: string; windowId?: number; tabId?: number }): Window[];
  isAllowedIncognitoAccess(): Promise<boolean>;
  isAllowedFileSchemeAccess(): Promise<boolean>;
  setUpdateUrlData(data: string): void;
  inIncognitoContext: boolean;
}

export function createExtensionApi(
  callNativeApi: NativeApiCaller,
  extensionId: string,
  extensionPath: string
): ExtensionApi {
  return {
    getURL(path: string): string {
      // Return file:// URL for local extension
      const cleanPath = path.startsWith('/') ? path.slice(1) : path;
      return `file:///${extensionPath.replace(/\\/g, '/')}/${cleanPath}`;
    },

    async getBackgroundPage(): Promise<Window | null> {
      // Not supported in WebView environment
      return null;
    },

    getViews(_fetchProperties?: { type?: string; windowId?: number; tabId?: number }): Window[] {
      // Return current window only
      return [window];
    },

    async isAllowedIncognitoAccess(): Promise<boolean> {
      return false;
    },

    async isAllowedFileSchemeAccess(): Promise<boolean> {
      return true;
    },

    setUpdateUrlData(_data: string): void {
      // No-op in WebView environment
    },

    inIncognitoContext: false,
  };
}
