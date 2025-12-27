/**
 * Chrome TabGroups API Polyfill
 */

import type { ChromeEvent } from '../types';
import { EventEmitter } from '../event_emitter';

type NativeApiCaller = (api: string, method: string, params?: Record<string, any>) => Promise<any>;

export type Color = 'grey' | 'blue' | 'red' | 'yellow' | 'green' | 'pink' | 'purple' | 'cyan' | 'orange';

export interface TabGroup {
  id: number;
  collapsed: boolean;
  color: Color;
  title?: string;
  windowId: number;
}

export interface QueryInfo {
  collapsed?: boolean;
  color?: Color;
  title?: string;
  windowId?: number;
}

export interface UpdateProperties {
  collapsed?: boolean;
  color?: Color;
  title?: string;
}

export interface MoveProperties {
  windowId?: number;
  index: number;
}

export interface TabGroupsApi {
  get(groupId: number): Promise<TabGroup>;
  query(queryInfo: QueryInfo): Promise<TabGroup[]>;
  update(groupId: number, updateProperties: UpdateProperties): Promise<TabGroup>;
  move(groupId: number, moveProperties: MoveProperties): Promise<TabGroup>;
  onCreated: ChromeEvent<(group: TabGroup) => void>;
  onUpdated: ChromeEvent<(group: TabGroup) => void>;
  onMoved: ChromeEvent<(group: TabGroup) => void>;
  onRemoved: ChromeEvent<(group: TabGroup) => void>;
  TAB_GROUP_ID_NONE: number;
  Color: Record<Color, Color>;
}

export function createTabGroupsApi(callNativeApi: NativeApiCaller): TabGroupsApi {
  const onCreated = new EventEmitter<[TabGroup]>();
  const onUpdated = new EventEmitter<[TabGroup]>();
  const onMoved = new EventEmitter<[TabGroup]>();
  const onRemoved = new EventEmitter<[TabGroup]>();

  const colors: Record<Color, Color> = {
    grey: 'grey',
    blue: 'blue',
    red: 'red',
    yellow: 'yellow',
    green: 'green',
    pink: 'pink',
    purple: 'purple',
    cyan: 'cyan',
    orange: 'orange',
  };

  return {
    async get(groupId: number): Promise<TabGroup> {
      return callNativeApi('tabGroups', 'get', { groupId });
    },

    async query(queryInfo: QueryInfo): Promise<TabGroup[]> {
      return callNativeApi('tabGroups', 'query', queryInfo);
    },

    async update(groupId: number, updateProperties: UpdateProperties): Promise<TabGroup> {
      return callNativeApi('tabGroups', 'update', { groupId, ...updateProperties });
    },

    async move(groupId: number, moveProperties: MoveProperties): Promise<TabGroup> {
      return callNativeApi('tabGroups', 'move', { groupId, ...moveProperties });
    },

    onCreated,
    onUpdated,
    onMoved,
    onRemoved,
    TAB_GROUP_ID_NONE: -1,
    Color: colors,
  };
}
