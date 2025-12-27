/**
 * Chrome Bookmarks API Polyfill
 */

import type { ChromeEvent } from '../types';
import { EventEmitter } from '../event_emitter';

type NativeApiCaller = (api: string, method: string, params?: Record<string, any>) => Promise<any>;

export interface BookmarkTreeNode {
  id: string;
  parentId?: string;
  index?: number;
  url?: string;
  title: string;
  dateAdded?: number;
  dateLastUsed?: number;
  dateGroupModified?: number;
  unmodifiable?: 'managed';
  children?: BookmarkTreeNode[];
}

export interface CreateDetails {
  parentId?: string;
  index?: number;
  title?: string;
  url?: string;
}

export interface MoveDestination {
  parentId?: string;
  index?: number;
}

export interface BookmarksApi {
  get(idOrIdList: string | string[]): Promise<BookmarkTreeNode[]>;
  getChildren(id: string): Promise<BookmarkTreeNode[]>;
  getRecent(numberOfItems: number): Promise<BookmarkTreeNode[]>;
  getTree(): Promise<BookmarkTreeNode[]>;
  getSubTree(id: string): Promise<BookmarkTreeNode[]>;
  search(query: string | { query?: string; url?: string; title?: string }): Promise<BookmarkTreeNode[]>;
  create(bookmark: CreateDetails): Promise<BookmarkTreeNode>;
  move(id: string, destination: MoveDestination): Promise<BookmarkTreeNode>;
  update(id: string, changes: { title?: string; url?: string }): Promise<BookmarkTreeNode>;
  remove(id: string): Promise<void>;
  removeTree(id: string): Promise<void>;
  onCreated: ChromeEvent<(id: string, bookmark: BookmarkTreeNode) => void>;
  onRemoved: ChromeEvent<(id: string, removeInfo: { parentId: string; index: number; node: BookmarkTreeNode }) => void>;
  onChanged: ChromeEvent<(id: string, changeInfo: { title: string; url?: string }) => void>;
  onMoved: ChromeEvent<(id: string, moveInfo: { parentId: string; index: number; oldParentId: string; oldIndex: number }) => void>;
  onChildrenReordered: ChromeEvent<(id: string, reorderInfo: { childIds: string[] }) => void>;
  onImportBegan: ChromeEvent<() => void>;
  onImportEnded: ChromeEvent<() => void>;
  MAX_WRITE_OPERATIONS_PER_HOUR: number;
  MAX_SUSTAINED_WRITE_OPERATIONS_PER_MINUTE: number;
}

export function createBookmarksApi(callNativeApi: NativeApiCaller): BookmarksApi {
  const onCreated = new EventEmitter<[string, BookmarkTreeNode]>();
  const onRemoved = new EventEmitter<[string, { parentId: string; index: number; node: BookmarkTreeNode }]>();
  const onChanged = new EventEmitter<[string, { title: string; url?: string }]>();
  const onMoved = new EventEmitter<[string, { parentId: string; index: number; oldParentId: string; oldIndex: number }]>();
  const onChildrenReordered = new EventEmitter<[string, { childIds: string[] }]>();
  const onImportBegan = new EventEmitter<[]>();
  const onImportEnded = new EventEmitter<[]>();

  return {
    async get(idOrIdList): Promise<BookmarkTreeNode[]> {
      return callNativeApi('bookmarks', 'get', { idOrIdList });
    },

    async getChildren(id: string): Promise<BookmarkTreeNode[]> {
      return callNativeApi('bookmarks', 'getChildren', { id });
    },

    async getRecent(numberOfItems: number): Promise<BookmarkTreeNode[]> {
      return callNativeApi('bookmarks', 'getRecent', { numberOfItems });
    },

    async getTree(): Promise<BookmarkTreeNode[]> {
      return callNativeApi('bookmarks', 'getTree', {});
    },

    async getSubTree(id: string): Promise<BookmarkTreeNode[]> {
      return callNativeApi('bookmarks', 'getSubTree', { id });
    },

    async search(query): Promise<BookmarkTreeNode[]> {
      return callNativeApi('bookmarks', 'search', { query });
    },

    async create(bookmark: CreateDetails): Promise<BookmarkTreeNode> {
      return callNativeApi('bookmarks', 'create', { bookmark });
    },

    async move(id: string, destination: MoveDestination): Promise<BookmarkTreeNode> {
      return callNativeApi('bookmarks', 'move', { id, destination });
    },

    async update(id: string, changes): Promise<BookmarkTreeNode> {
      return callNativeApi('bookmarks', 'update', { id, changes });
    },

    async remove(id: string): Promise<void> {
      return callNativeApi('bookmarks', 'remove', { id });
    },

    async removeTree(id: string): Promise<void> {
      return callNativeApi('bookmarks', 'removeTree', { id });
    },

    onCreated,
    onRemoved,
    onChanged,
    onMoved,
    onChildrenReordered,
    onImportBegan,
    onImportEnded,
    MAX_WRITE_OPERATIONS_PER_HOUR: 1000000,
    MAX_SUSTAINED_WRITE_OPERATIONS_PER_MINUTE: 1000000,
  };
}
