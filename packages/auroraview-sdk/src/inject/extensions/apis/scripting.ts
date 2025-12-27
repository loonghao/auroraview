/**
 * Chrome Scripting API Polyfill
 */

type NativeApiCaller = (api: string, method: string, params?: Record<string, any>) => Promise<any>;

export interface InjectionTarget {
  tabId: number;
  allFrames?: boolean;
  frameIds?: number[];
  documentIds?: string[];
}

export interface ScriptInjection {
  target: InjectionTarget;
  files?: string[];
  func?: (...args: any[]) => any;
  args?: any[];
  world?: 'ISOLATED' | 'MAIN';
  injectImmediately?: boolean;
}

export interface CSSInjection {
  target: InjectionTarget;
  files?: string[];
  css?: string;
  origin?: 'USER' | 'AUTHOR';
}

export interface InjectionResult {
  documentId: string;
  frameId: number;
  result?: any;
  error?: Error;
}

export interface RegisteredContentScript {
  id: string;
  matches?: string[];
  excludeMatches?: string[];
  css?: string[];
  js?: string[];
  allFrames?: boolean;
  matchOriginAsFallback?: boolean;
  runAt?: 'document_start' | 'document_end' | 'document_idle';
  world?: 'ISOLATED' | 'MAIN';
  persistAcrossSessions?: boolean;
}

export interface ScriptingApi {
  executeScript(injection: ScriptInjection): Promise<InjectionResult[]>;
  insertCSS(injection: CSSInjection): Promise<void>;
  removeCSS(injection: CSSInjection): Promise<void>;
  registerContentScripts(scripts: RegisteredContentScript[]): Promise<void>;
  unregisterContentScripts(filter?: { ids?: string[] }): Promise<void>;
  getRegisteredContentScripts(filter?: { ids?: string[] }): Promise<RegisteredContentScript[]>;
  updateContentScripts(scripts: RegisteredContentScript[]): Promise<void>;
}

export function createScriptingApi(callNativeApi: NativeApiCaller): ScriptingApi {
  return {
    async executeScript(injection: ScriptInjection): Promise<InjectionResult[]> {
      // Convert function to string if provided
      const params: any = { ...injection };
      if (injection.func) {
        params.func = injection.func.toString();
      }
      return callNativeApi('scripting', 'executeScript', params);
    },

    async insertCSS(injection: CSSInjection): Promise<void> {
      return callNativeApi('scripting', 'insertCSS', injection);
    },

    async removeCSS(injection: CSSInjection): Promise<void> {
      return callNativeApi('scripting', 'removeCSS', injection);
    },

    async registerContentScripts(scripts: RegisteredContentScript[]): Promise<void> {
      return callNativeApi('scripting', 'registerContentScripts', { scripts });
    },

    async unregisterContentScripts(filter?: { ids?: string[] }): Promise<void> {
      return callNativeApi('scripting', 'unregisterContentScripts', filter || {});
    },

    async getRegisteredContentScripts(filter?: { ids?: string[] }): Promise<RegisteredContentScript[]> {
      return callNativeApi('scripting', 'getRegisteredContentScripts', filter || {});
    },

    async updateContentScripts(scripts: RegisteredContentScript[]): Promise<void> {
      return callNativeApi('scripting', 'updateContentScripts', { scripts });
    },
  };
}
