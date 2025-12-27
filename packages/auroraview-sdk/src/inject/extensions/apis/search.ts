/**
 * Chrome Search API Polyfill
 */

type NativeApiCaller = (api: string, method: string, params?: Record<string, any>) => Promise<any>;

export type Disposition = 'CURRENT_TAB' | 'NEW_TAB' | 'NEW_WINDOW';

export interface QueryInfo {
  text: string;
  disposition?: Disposition;
  tabId?: number;
}

export interface SearchApi {
  query(queryInfo: QueryInfo): Promise<void>;
}

export function createSearchApi(callNativeApi: NativeApiCaller): SearchApi {
  return {
    async query(queryInfo: QueryInfo): Promise<void> {
      return callNativeApi('search', 'query', queryInfo);
    },
  };
}
