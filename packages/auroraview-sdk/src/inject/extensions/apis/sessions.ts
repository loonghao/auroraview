/**
 * Chrome Sessions API Polyfill
 */

import type { ChromeEvent, Tab } from '../types';
import { EventEmitter } from '../event_emitter';
import type { Window } from './windows';

type NativeApiCaller = (api: string, method: string, params?: Record<string, any>) => Promise<any>;

export interface Session {
  lastModified: number;
  tab?: Tab;
  window?: Window;
}

export interface Device {
  deviceName: string;
  sessions: Session[];
}

export interface SessionsApi {
  getRecentlyClosed(filter?: { maxResults?: number }): Promise<Session[]>;
  getDevices(filter?: { maxResults?: number }): Promise<Device[]>;
  restore(sessionId?: string): Promise<Session>;
  onChanged: ChromeEvent<() => void>;
  MAX_SESSION_RESULTS: number;
}

export function createSessionsApi(callNativeApi: NativeApiCaller): SessionsApi {
  const onChanged = new EventEmitter<[]>();

  return {
    async getRecentlyClosed(filter?): Promise<Session[]> {
      return callNativeApi('sessions', 'getRecentlyClosed', filter || {});
    },

    async getDevices(filter?): Promise<Device[]> {
      return callNativeApi('sessions', 'getDevices', filter || {});
    },

    async restore(sessionId?: string): Promise<Session> {
      return callNativeApi('sessions', 'restore', { sessionId });
    },

    onChanged,
    MAX_SESSION_RESULTS: 25,
  };
}
