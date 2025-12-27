/**
 * Chrome TopSites API Polyfill
 */

type NativeApiCaller = (api: string, method: string, params?: Record<string, any>) => Promise<any>;

export interface MostVisitedURL {
  url: string;
  title: string;
}

export interface TopSitesApi {
  get(): Promise<MostVisitedURL[]>;
}

export function createTopSitesApi(callNativeApi: NativeApiCaller): TopSitesApi {
  return {
    async get(): Promise<MostVisitedURL[]> {
      return callNativeApi('topSites', 'get', {});
    },
  };
}
