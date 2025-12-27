/**
 * Chrome Offscreen API Polyfill
 */

type NativeApiCaller = (api: string, method: string, params?: Record<string, any>) => Promise<any>;

export type Reason =
  | 'TESTING'
  | 'AUDIO_PLAYBACK'
  | 'IFRAME_SCRIPTING'
  | 'DOM_SCRAPING'
  | 'BLOBS'
  | 'DOM_PARSER'
  | 'USER_MEDIA'
  | 'DISPLAY_MEDIA'
  | 'WEB_RTC'
  | 'CLIPBOARD'
  | 'LOCAL_STORAGE'
  | 'WORKERS'
  | 'BATTERY_STATUS'
  | 'MATCH_MEDIA'
  | 'GEOLOCATION';

export interface CreateParameters {
  url: string;
  reasons: Reason[];
  justification: string;
}

export interface OffscreenApi {
  createDocument(parameters: CreateParameters): Promise<void>;
  closeDocument(): Promise<void>;
  hasDocument(): Promise<boolean>;
  Reason: Record<Reason, Reason>;
}

export function createOffscreenApi(callNativeApi: NativeApiCaller): OffscreenApi {
  const reasons: Record<Reason, Reason> = {
    TESTING: 'TESTING',
    AUDIO_PLAYBACK: 'AUDIO_PLAYBACK',
    IFRAME_SCRIPTING: 'IFRAME_SCRIPTING',
    DOM_SCRAPING: 'DOM_SCRAPING',
    BLOBS: 'BLOBS',
    DOM_PARSER: 'DOM_PARSER',
    USER_MEDIA: 'USER_MEDIA',
    DISPLAY_MEDIA: 'DISPLAY_MEDIA',
    WEB_RTC: 'WEB_RTC',
    CLIPBOARD: 'CLIPBOARD',
    LOCAL_STORAGE: 'LOCAL_STORAGE',
    WORKERS: 'WORKERS',
    BATTERY_STATUS: 'BATTERY_STATUS',
    MATCH_MEDIA: 'MATCH_MEDIA',
    GEOLOCATION: 'GEOLOCATION',
  };

  return {
    async createDocument(parameters: CreateParameters): Promise<void> {
      return callNativeApi('offscreen', 'createDocument', parameters);
    },

    async closeDocument(): Promise<void> {
      return callNativeApi('offscreen', 'closeDocument', {});
    },

    async hasDocument(): Promise<boolean> {
      return callNativeApi('offscreen', 'hasDocument', {});
    },

    Reason: reasons,
  };
}
