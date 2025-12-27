/**
 * Chrome ContextMenus API Polyfill
 */

import type { ChromeEvent, Tab } from '../types';
import { EventEmitter } from '../event_emitter';

type NativeApiCaller = (api: string, method: string, params?: Record<string, any>) => Promise<any>;

export type ContextType =
  | 'all'
  | 'page'
  | 'frame'
  | 'selection'
  | 'link'
  | 'editable'
  | 'image'
  | 'video'
  | 'audio'
  | 'launcher'
  | 'browser_action'
  | 'page_action'
  | 'action';

export type ItemType = 'normal' | 'checkbox' | 'radio' | 'separator';

export interface CreateProperties {
  id?: string;
  type?: ItemType;
  title?: string;
  checked?: boolean;
  contexts?: ContextType[];
  visible?: boolean;
  onclick?: (info: OnClickData, tab?: Tab) => void;
  parentId?: string | number;
  documentUrlPatterns?: string[];
  targetUrlPatterns?: string[];
  enabled?: boolean;
}

export interface UpdateProperties {
  type?: ItemType;
  title?: string;
  checked?: boolean;
  contexts?: ContextType[];
  visible?: boolean;
  onclick?: (info: OnClickData, tab?: Tab) => void;
  parentId?: string | number;
  documentUrlPatterns?: string[];
  targetUrlPatterns?: string[];
  enabled?: boolean;
}

export interface OnClickData {
  menuItemId: string | number;
  parentMenuItemId?: string | number;
  mediaType?: string;
  linkUrl?: string;
  srcUrl?: string;
  pageUrl?: string;
  frameUrl?: string;
  frameId?: number;
  selectionText?: string;
  editable: boolean;
  wasChecked?: boolean;
  checked?: boolean;
}

export interface ContextMenusApi {
  create(createProperties: CreateProperties, callback?: () => void): string | number;
  update(id: string | number, updateProperties: UpdateProperties): Promise<void>;
  remove(menuItemId: string | number): Promise<void>;
  removeAll(): Promise<void>;
  onClicked: ChromeEvent<(info: OnClickData, tab?: Tab) => void>;
  ACTION_MENU_TOP_LEVEL_LIMIT: number;
}

export function createContextMenusApi(callNativeApi: NativeApiCaller): ContextMenusApi {
  const onClicked = new EventEmitter<[OnClickData, Tab?]>();
  let menuIdCounter = 0;

  return {
    create(createProperties: CreateProperties, callback?: () => void): string | number {
      const id = createProperties.id ?? `menu_${++menuIdCounter}`;
      callNativeApi('contextMenus', 'create', { ...createProperties, id }).then(() => {
        callback?.();
      });
      return id;
    },

    async update(id: string | number, updateProperties: UpdateProperties): Promise<void> {
      return callNativeApi('contextMenus', 'update', { id, ...updateProperties });
    },

    async remove(menuItemId: string | number): Promise<void> {
      return callNativeApi('contextMenus', 'remove', { menuItemId });
    },

    async removeAll(): Promise<void> {
      return callNativeApi('contextMenus', 'removeAll', {});
    },

    onClicked,
    ACTION_MENU_TOP_LEVEL_LIMIT: 6,
  };
}
