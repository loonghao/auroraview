/**
 * Chrome Omnibox API Polyfill
 */

import type { ChromeEvent } from '../types';
import { EventEmitter } from '../event_emitter';

type NativeApiCaller = (api: string, method: string, params?: Record<string, any>) => Promise<any>;

export interface SuggestResult {
  content: string;
  description: string;
  deletable?: boolean;
}

export type OnInputEnteredDisposition = 'currentTab' | 'newForegroundTab' | 'newBackgroundTab';

export interface OmniboxApi {
  setDefaultSuggestion(suggestion: { description: string }): void;
  onInputStarted: ChromeEvent<() => void>;
  onInputChanged: ChromeEvent<(text: string, suggest: (suggestResults: SuggestResult[]) => void) => void>;
  onInputEntered: ChromeEvent<(text: string, disposition: OnInputEnteredDisposition) => void>;
  onInputCancelled: ChromeEvent<() => void>;
  onDeleteSuggestion: ChromeEvent<(text: string) => void>;
}

export function createOmniboxApi(callNativeApi: NativeApiCaller): OmniboxApi {
  const onInputStarted = new EventEmitter<[]>();
  const onInputChanged = new EventEmitter<[string, (suggestResults: SuggestResult[]) => void]>();
  const onInputEntered = new EventEmitter<[string, OnInputEnteredDisposition]>();
  const onInputCancelled = new EventEmitter<[]>();
  const onDeleteSuggestion = new EventEmitter<[string]>();

  return {
    setDefaultSuggestion(suggestion: { description: string }): void {
      callNativeApi('omnibox', 'setDefaultSuggestion', suggestion);
    },

    onInputStarted,
    onInputChanged,
    onInputEntered,
    onInputCancelled,
    onDeleteSuggestion,
  };
}
