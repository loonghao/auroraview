/**
 * Chrome FontSettings API Polyfill
 */

import type { ChromeEvent } from '../types';
import { EventEmitter } from '../event_emitter';

type NativeApiCaller = (api: string, method: string, params?: Record<string, any>) => Promise<any>;

export type GenericFamily = 'standard' | 'sansserif' | 'serif' | 'fixed' | 'cursive' | 'fantasy' | 'math';
export type ScriptCode = string;

export interface FontName {
  fontId: string;
  displayName: string;
}

export interface FontSettingsApi {
  clearFont(details: { genericFamily: GenericFamily; script?: ScriptCode }): Promise<void>;
  getFont(details: { genericFamily: GenericFamily; script?: ScriptCode }): Promise<{ fontId: string; levelOfControl: string }>;
  setFont(details: { genericFamily: GenericFamily; fontId: string; script?: ScriptCode }): Promise<void>;
  getFontList(): Promise<FontName[]>;
  clearDefaultFontSize(): Promise<void>;
  getDefaultFontSize(): Promise<{ pixelSize: number; levelOfControl: string }>;
  setDefaultFontSize(details: { pixelSize: number }): Promise<void>;
  clearDefaultFixedFontSize(): Promise<void>;
  getDefaultFixedFontSize(): Promise<{ pixelSize: number; levelOfControl: string }>;
  setDefaultFixedFontSize(details: { pixelSize: number }): Promise<void>;
  clearMinimumFontSize(): Promise<void>;
  getMinimumFontSize(): Promise<{ pixelSize: number; levelOfControl: string }>;
  setMinimumFontSize(details: { pixelSize: number }): Promise<void>;
  onFontChanged: ChromeEvent<(details: { fontId: string; genericFamily: GenericFamily; script?: ScriptCode; levelOfControl: string }) => void>;
  onDefaultFontSizeChanged: ChromeEvent<(details: { pixelSize: number; levelOfControl: string }) => void>;
  onDefaultFixedFontSizeChanged: ChromeEvent<(details: { pixelSize: number; levelOfControl: string }) => void>;
  onMinimumFontSizeChanged: ChromeEvent<(details: { pixelSize: number; levelOfControl: string }) => void>;
}

export function createFontSettingsApi(callNativeApi: NativeApiCaller): FontSettingsApi {
  const onFontChanged = new EventEmitter<[{ fontId: string; genericFamily: GenericFamily; script?: ScriptCode; levelOfControl: string }]>();
  const onDefaultFontSizeChanged = new EventEmitter<[{ pixelSize: number; levelOfControl: string }]>();
  const onDefaultFixedFontSizeChanged = new EventEmitter<[{ pixelSize: number; levelOfControl: string }]>();
  const onMinimumFontSizeChanged = new EventEmitter<[{ pixelSize: number; levelOfControl: string }]>();

  return {
    async clearFont(details): Promise<void> {
      return callNativeApi('fontSettings', 'clearFont', details);
    },
    async getFont(details): Promise<{ fontId: string; levelOfControl: string }> {
      return callNativeApi('fontSettings', 'getFont', details);
    },
    async setFont(details): Promise<void> {
      return callNativeApi('fontSettings', 'setFont', details);
    },
    async getFontList(): Promise<FontName[]> {
      return callNativeApi('fontSettings', 'getFontList', {});
    },
    async clearDefaultFontSize(): Promise<void> {
      return callNativeApi('fontSettings', 'clearDefaultFontSize', {});
    },
    async getDefaultFontSize(): Promise<{ pixelSize: number; levelOfControl: string }> {
      return callNativeApi('fontSettings', 'getDefaultFontSize', {});
    },
    async setDefaultFontSize(details): Promise<void> {
      return callNativeApi('fontSettings', 'setDefaultFontSize', details);
    },
    async clearDefaultFixedFontSize(): Promise<void> {
      return callNativeApi('fontSettings', 'clearDefaultFixedFontSize', {});
    },
    async getDefaultFixedFontSize(): Promise<{ pixelSize: number; levelOfControl: string }> {
      return callNativeApi('fontSettings', 'getDefaultFixedFontSize', {});
    },
    async setDefaultFixedFontSize(details): Promise<void> {
      return callNativeApi('fontSettings', 'setDefaultFixedFontSize', details);
    },
    async clearMinimumFontSize(): Promise<void> {
      return callNativeApi('fontSettings', 'clearMinimumFontSize', {});
    },
    async getMinimumFontSize(): Promise<{ pixelSize: number; levelOfControl: string }> {
      return callNativeApi('fontSettings', 'getMinimumFontSize', {});
    },
    async setMinimumFontSize(details): Promise<void> {
      return callNativeApi('fontSettings', 'setMinimumFontSize', details);
    },
    onFontChanged,
    onDefaultFontSizeChanged,
    onDefaultFixedFontSizeChanged,
    onMinimumFontSizeChanged,
  };
}
