/**
 * Chrome Commands API Polyfill
 */

import type { ChromeEvent, Tab } from '../types';
import { EventEmitter } from '../event_emitter';

type NativeApiCaller = (api: string, method: string, params?: Record<string, any>) => Promise<any>;

export interface Command {
  name?: string;
  description?: string;
  shortcut?: string;
}

export interface CommandsApi {
  getAll(): Promise<Command[]>;
  onCommand: ChromeEvent<(command: string, tab?: Tab) => void>;
}

export function createCommandsApi(callNativeApi: NativeApiCaller): CommandsApi {
  const onCommand = new EventEmitter<[string, Tab?]>();

  return {
    async getAll(): Promise<Command[]> {
      return callNativeApi('commands', 'getAll', {});
    },

    onCommand,
  };
}
