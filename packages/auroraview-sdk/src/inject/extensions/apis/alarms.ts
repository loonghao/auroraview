/**
 * Chrome Alarms API Polyfill
 */

import type { ChromeEvent } from '../types';
import { EventEmitter } from '../event_emitter';

type NativeApiCaller = (api: string, method: string, params?: Record<string, any>) => Promise<any>;

export interface AlarmInfo {
  when?: number;
  delayInMinutes?: number;
  periodInMinutes?: number;
}

export interface Alarm {
  name: string;
  scheduledTime: number;
  periodInMinutes?: number;
}

export interface AlarmsApi {
  create(name?: string, alarmInfo?: AlarmInfo): Promise<void>;
  get(name?: string): Promise<Alarm | undefined>;
  getAll(): Promise<Alarm[]>;
  clear(name?: string): Promise<boolean>;
  clearAll(): Promise<boolean>;
  onAlarm: ChromeEvent<(alarm: Alarm) => void>;
}

export function createAlarmsApi(callNativeApi: NativeApiCaller): AlarmsApi {
  const onAlarm = new EventEmitter<[Alarm]>();

  return {
    async create(name?: string, alarmInfo?: AlarmInfo): Promise<void> {
      return callNativeApi('alarms', 'create', { name, alarmInfo });
    },

    async get(name?: string): Promise<Alarm | undefined> {
      return callNativeApi('alarms', 'get', { name });
    },

    async getAll(): Promise<Alarm[]> {
      return callNativeApi('alarms', 'getAll', {});
    },

    async clear(name?: string): Promise<boolean> {
      return callNativeApi('alarms', 'clear', { name });
    },

    async clearAll(): Promise<boolean> {
      return callNativeApi('alarms', 'clearAll', {});
    },

    onAlarm,
  };
}
