/**
 * Chrome Idle API Polyfill
 */

import type { ChromeEvent } from '../types';
import { EventEmitter } from '../event_emitter';

type NativeApiCaller = (api: string, method: string, params?: Record<string, any>) => Promise<any>;

export type IdleState = 'active' | 'idle' | 'locked';

export interface IdleApi {
  queryState(detectionIntervalInSeconds: number): Promise<IdleState>;
  setDetectionInterval(intervalInSeconds: number): void;
  getAutoLockDelay(): Promise<number>;
  onStateChanged: ChromeEvent<(newState: IdleState) => void>;
}

export function createIdleApi(callNativeApi: NativeApiCaller): IdleApi {
  const onStateChanged = new EventEmitter<[IdleState]>();

  return {
    async queryState(detectionIntervalInSeconds: number): Promise<IdleState> {
      return callNativeApi('idle', 'queryState', { detectionIntervalInSeconds });
    },

    setDetectionInterval(intervalInSeconds: number): void {
      callNativeApi('idle', 'setDetectionInterval', { intervalInSeconds });
    },

    async getAutoLockDelay(): Promise<number> {
      return callNativeApi('idle', 'getAutoLockDelay', {});
    },

    onStateChanged,
  };
}
