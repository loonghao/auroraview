/**
 * Chrome Cookies API Polyfill
 */

import type { ChromeEvent } from '../types';
import { EventEmitter } from '../event_emitter';

type NativeApiCaller = (api: string, method: string, params?: Record<string, any>) => Promise<any>;

export type SameSiteStatus = 'no_restriction' | 'lax' | 'strict' | 'unspecified';

export interface Cookie {
  name: string;
  value: string;
  domain: string;
  hostOnly: boolean;
  path: string;
  secure: boolean;
  httpOnly: boolean;
  sameSite: SameSiteStatus;
  session: boolean;
  expirationDate?: number;
  storeId: string;
}

export interface CookieStore {
  id: string;
  tabIds: number[];
}

export interface CookieDetails {
  url: string;
  name: string;
  storeId?: string;
}

export interface SetDetails {
  url: string;
  name?: string;
  value?: string;
  domain?: string;
  path?: string;
  secure?: boolean;
  httpOnly?: boolean;
  sameSite?: SameSiteStatus;
  expirationDate?: number;
  storeId?: string;
}

export interface CookiesApi {
  get(details: CookieDetails): Promise<Cookie | null>;
  getAll(details: { url?: string; name?: string; domain?: string; path?: string; secure?: boolean; session?: boolean; storeId?: string }): Promise<Cookie[]>;
  set(details: SetDetails): Promise<Cookie | null>;
  remove(details: CookieDetails): Promise<{ url: string; name: string; storeId: string } | null>;
  getAllCookieStores(): Promise<CookieStore[]>;
  onChanged: ChromeEvent<(changeInfo: { removed: boolean; cookie: Cookie; cause: string }) => void>;
}

export function createCookiesApi(callNativeApi: NativeApiCaller): CookiesApi {
  const onChanged = new EventEmitter<[{ removed: boolean; cookie: Cookie; cause: string }]>();

  return {
    async get(details: CookieDetails): Promise<Cookie | null> {
      return callNativeApi('cookies', 'get', details);
    },

    async getAll(details): Promise<Cookie[]> {
      return callNativeApi('cookies', 'getAll', details);
    },

    async set(details: SetDetails): Promise<Cookie | null> {
      return callNativeApi('cookies', 'set', details);
    },

    async remove(details: CookieDetails): Promise<{ url: string; name: string; storeId: string } | null> {
      return callNativeApi('cookies', 'remove', details);
    },

    async getAllCookieStores(): Promise<CookieStore[]> {
      return callNativeApi('cookies', 'getAllCookieStores', {});
    },

    onChanged,
  };
}
