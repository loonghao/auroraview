/**
 * Chrome History API Polyfill
 */

import type { ChromeEvent } from '../types';
import { EventEmitter } from '../event_emitter';

type NativeApiCaller = (api: string, method: string, params?: Record<string, any>) => Promise<any>;

export interface HistoryItem {
  id: string;
  url?: string;
  title?: string;
  lastVisitTime?: number;
  visitCount?: number;
  typedCount?: number;
}

export interface VisitItem {
  id: string;
  visitId: string;
  visitTime?: number;
  referringVisitId: string;
  transition: string;
  isLocal: boolean;
}

export interface HistoryApi {
  search(query: { text: string; startTime?: number; endTime?: number; maxResults?: number }): Promise<HistoryItem[]>;
  getVisits(details: { url: string }): Promise<VisitItem[]>;
  addUrl(details: { url: string; title?: string; visitTime?: number; transition?: string }): Promise<void>;
  deleteUrl(details: { url: string }): Promise<void>;
  deleteRange(range: { startTime: number; endTime: number }): Promise<void>;
  deleteAll(): Promise<void>;
  onVisited: ChromeEvent<(result: HistoryItem) => void>;
  onVisitRemoved: ChromeEvent<(removed: { allHistory: boolean; urls?: string[] }) => void>;
}

export function createHistoryApi(callNativeApi: NativeApiCaller): HistoryApi {
  const onVisited = new EventEmitter<[HistoryItem]>();
  const onVisitRemoved = new EventEmitter<[{ allHistory: boolean; urls?: string[] }]>();

  return {
    async search(query): Promise<HistoryItem[]> {
      return callNativeApi('history', 'search', query);
    },

    async getVisits(details): Promise<VisitItem[]> {
      return callNativeApi('history', 'getVisits', details);
    },

    async addUrl(details): Promise<void> {
      return callNativeApi('history', 'addUrl', details);
    },

    async deleteUrl(details): Promise<void> {
      return callNativeApi('history', 'deleteUrl', details);
    },

    async deleteRange(range): Promise<void> {
      return callNativeApi('history', 'deleteRange', range);
    },

    async deleteAll(): Promise<void> {
      return callNativeApi('history', 'deleteAll', {});
    },

    onVisited,
    onVisitRemoved,
  };
}
