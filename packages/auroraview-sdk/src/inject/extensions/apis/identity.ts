/**
 * Chrome Identity API Polyfill
 */

import type { ChromeEvent } from '../types';
import { EventEmitter } from '../event_emitter';

type NativeApiCaller = (api: string, method: string, params?: Record<string, any>) => Promise<any>;

export interface TokenDetails {
  interactive?: boolean;
  account?: { id: string };
  scopes?: string[];
}

export interface ProfileUserInfo {
  email: string;
  id: string;
}

export interface AccountInfo {
  id: string;
}

export interface WebAuthFlowDetails {
  url: string;
  interactive?: boolean;
}

export interface IdentityApi {
  getAuthToken(details?: TokenDetails): Promise<{ token: string; grantedScopes?: string[] }>;
  removeCachedAuthToken(details: { token: string }): Promise<void>;
  clearAllCachedAuthTokens(): Promise<void>;
  getProfileUserInfo(details?: { accountStatus?: string }): Promise<ProfileUserInfo>;
  getAccounts(): Promise<AccountInfo[]>;
  launchWebAuthFlow(details: WebAuthFlowDetails): Promise<string>;
  getRedirectURL(path?: string): string;
  onSignInChanged: ChromeEvent<(account: AccountInfo, signedIn: boolean) => void>;
}

export function createIdentityApi(callNativeApi: NativeApiCaller, extensionId: string): IdentityApi {
  const onSignInChanged = new EventEmitter<[AccountInfo, boolean]>();

  return {
    async getAuthToken(details?: TokenDetails): Promise<{ token: string; grantedScopes?: string[] }> {
      return callNativeApi('identity', 'getAuthToken', details || {});
    },

    async removeCachedAuthToken(details: { token: string }): Promise<void> {
      return callNativeApi('identity', 'removeCachedAuthToken', details);
    },

    async clearAllCachedAuthTokens(): Promise<void> {
      return callNativeApi('identity', 'clearAllCachedAuthTokens', {});
    },

    async getProfileUserInfo(details?: { accountStatus?: string }): Promise<ProfileUserInfo> {
      return callNativeApi('identity', 'getProfileUserInfo', details || {});
    },

    async getAccounts(): Promise<AccountInfo[]> {
      return callNativeApi('identity', 'getAccounts', {});
    },

    async launchWebAuthFlow(details: WebAuthFlowDetails): Promise<string> {
      return callNativeApi('identity', 'launchWebAuthFlow', details);
    },

    getRedirectURL(path?: string): string {
      const base = `https://${extensionId}.chromiumapp.org`;
      return path ? `${base}/${path}` : `${base}/`;
    },

    onSignInChanged,
  };
}
