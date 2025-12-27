/**
 * Chrome Notifications API Polyfill
 */

import type { ChromeEvent } from '../types';
import { EventEmitter } from '../event_emitter';

type NativeApiCaller = (api: string, method: string, params?: Record<string, any>) => Promise<any>;

export type TemplateType = 'basic' | 'image' | 'list' | 'progress';

export interface NotificationButton {
  title: string;
  iconUrl?: string;
}

export interface NotificationItem {
  title: string;
  message: string;
}

export interface NotificationOptions {
  type?: TemplateType;
  iconUrl?: string;
  appIconMaskUrl?: string;
  title?: string;
  message?: string;
  contextMessage?: string;
  priority?: number;
  eventTime?: number;
  buttons?: NotificationButton[];
  imageUrl?: string;
  items?: NotificationItem[];
  progress?: number;
  isClickable?: boolean;
  requireInteraction?: boolean;
  silent?: boolean;
}

export interface NotificationsApi {
  create(notificationId: string | undefined, options: NotificationOptions): Promise<string>;
  update(notificationId: string, options: NotificationOptions): Promise<boolean>;
  clear(notificationId: string): Promise<boolean>;
  getAll(): Promise<Record<string, boolean>>;
  getPermissionLevel(): Promise<'granted' | 'denied'>;
  onClicked: ChromeEvent<(notificationId: string) => void>;
  onButtonClicked: ChromeEvent<(notificationId: string, buttonIndex: number) => void>;
  onClosed: ChromeEvent<(notificationId: string, byUser: boolean) => void>;
  onPermissionLevelChanged: ChromeEvent<(level: string) => void>;
  onShowSettings: ChromeEvent<() => void>;
}

export function createNotificationsApi(callNativeApi: NativeApiCaller): NotificationsApi {
  const onClicked = new EventEmitter<[string]>();
  const onButtonClicked = new EventEmitter<[string, number]>();
  const onClosed = new EventEmitter<[string, boolean]>();
  const onPermissionLevelChanged = new EventEmitter<[string]>();
  const onShowSettings = new EventEmitter<[]>();

  return {
    async create(notificationId: string | undefined, options: NotificationOptions): Promise<string> {
      return callNativeApi('notifications', 'create', { notificationId, options });
    },

    async update(notificationId: string, options: NotificationOptions): Promise<boolean> {
      return callNativeApi('notifications', 'update', { notificationId, options });
    },

    async clear(notificationId: string): Promise<boolean> {
      return callNativeApi('notifications', 'clear', { notificationId });
    },

    async getAll(): Promise<Record<string, boolean>> {
      return callNativeApi('notifications', 'getAll', {});
    },

    async getPermissionLevel(): Promise<'granted' | 'denied'> {
      return callNativeApi('notifications', 'getPermissionLevel', {});
    },

    onClicked,
    onButtonClicked,
    onClosed,
    onPermissionLevelChanged,
    onShowSettings,
  };
}
