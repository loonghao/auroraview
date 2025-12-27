/**
 * Chrome SidePanel API Polyfill
 */

type NativeApiCaller = (api: string, method: string, params?: Record<string, any>) => Promise<any>;

export interface SidePanelOptions {
  tabId?: number;
  path?: string;
  enabled?: boolean;
}

export interface SidePanelApi {
  setOptions(options: SidePanelOptions): Promise<void>;
  getOptions(options?: { tabId?: number }): Promise<SidePanelOptions>;
  open(options?: { windowId?: number; tabId?: number }): Promise<void>;
  setPanelBehavior(behavior: { openPanelOnActionClick?: boolean }): Promise<void>;
  getPanelBehavior(): Promise<{ openPanelOnActionClick?: boolean }>;
}

export function createSidePanelApi(callNativeApi: NativeApiCaller): SidePanelApi {
  return {
    async setOptions(options: SidePanelOptions): Promise<void> {
      return callNativeApi('sidePanel', 'setOptions', { options });
    },

    async getOptions(options?: { tabId?: number }): Promise<SidePanelOptions> {
      return callNativeApi('sidePanel', 'getOptions', options || {});
    },

    async open(options?: { windowId?: number; tabId?: number }): Promise<void> {
      return callNativeApi('sidePanel', 'open', options || {});
    },

    async setPanelBehavior(behavior: { openPanelOnActionClick?: boolean }): Promise<void> {
      return callNativeApi('sidePanel', 'setPanelBehavior', { behavior });
    },

    async getPanelBehavior(): Promise<{ openPanelOnActionClick?: boolean }> {
      return callNativeApi('sidePanel', 'getPanelBehavior', {});
    },
  };
}
