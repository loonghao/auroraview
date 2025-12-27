/**
 * Chrome Power API Polyfill
 */

type NativeApiCaller = (api: string, method: string, params?: Record<string, any>) => Promise<any>;

export type Level = 'system' | 'display';

export interface PowerApi {
  requestKeepAwake(level: Level): void;
  releaseKeepAwake(): void;
  reportActivity(): Promise<void>;
}

export function createPowerApi(callNativeApi: NativeApiCaller): PowerApi {
  return {
    requestKeepAwake(level: Level): void {
      callNativeApi('power', 'requestKeepAwake', { level });
    },

    releaseKeepAwake(): void {
      callNativeApi('power', 'releaseKeepAwake', {});
    },

    async reportActivity(): Promise<void> {
      return callNativeApi('power', 'reportActivity', {});
    },
  };
}
