/**
 * Chrome TTS (Text-to-Speech) API Polyfill
 */

import type { ChromeEvent } from '../types';
import { EventEmitter } from '../event_emitter';

type NativeApiCaller = (api: string, method: string, params?: Record<string, any>) => Promise<any>;

export type EventType = 'start' | 'end' | 'word' | 'sentence' | 'marker' | 'interrupted' | 'cancelled' | 'error' | 'pause' | 'resume';

export interface TtsEvent {
  type: EventType;
  charIndex?: number;
  errorMessage?: string;
  length?: number;
}

export interface TtsVoice {
  voiceName?: string;
  lang?: string;
  remote?: boolean;
  extensionId?: string;
  eventTypes?: EventType[];
}

export interface SpeakOptions {
  enqueue?: boolean;
  voiceName?: string;
  extensionId?: string;
  lang?: string;
  rate?: number;
  pitch?: number;
  volume?: number;
  requiredEventTypes?: EventType[];
  desiredEventTypes?: EventType[];
  onEvent?: (event: TtsEvent) => void;
}

export interface TtsApi {
  speak(utterance: string, options?: SpeakOptions): Promise<void>;
  stop(): void;
  pause(): void;
  resume(): void;
  isSpeaking(): Promise<boolean>;
  getVoices(): Promise<TtsVoice[]>;
  onVoicesChanged: ChromeEvent<() => void>;
}

export function createTtsApi(callNativeApi: NativeApiCaller): TtsApi {
  const onVoicesChanged = new EventEmitter<[]>();

  return {
    async speak(utterance: string, options?: SpeakOptions): Promise<void> {
      // Use browser's native speech synthesis as fallback
      if ('speechSynthesis' in window) {
        return new Promise((resolve, reject) => {
          const speech = new SpeechSynthesisUtterance(utterance);
          if (options?.lang) speech.lang = options.lang;
          if (options?.rate) speech.rate = options.rate;
          if (options?.pitch) speech.pitch = options.pitch;
          if (options?.volume) speech.volume = options.volume;

          speech.onend = () => {
            options?.onEvent?.({ type: 'end' });
            resolve();
          };
          speech.onerror = (e) => {
            options?.onEvent?.({ type: 'error', errorMessage: e.error });
            reject(new Error(e.error));
          };
          speech.onstart = () => options?.onEvent?.({ type: 'start' });
          speech.onpause = () => options?.onEvent?.({ type: 'pause' });
          speech.onresume = () => options?.onEvent?.({ type: 'resume' });

          window.speechSynthesis.speak(speech);
        });
      }
      return callNativeApi('tts', 'speak', { utterance, options });
    },

    stop(): void {
      if ('speechSynthesis' in window) {
        window.speechSynthesis.cancel();
      } else {
        callNativeApi('tts', 'stop', {});
      }
    },

    pause(): void {
      if ('speechSynthesis' in window) {
        window.speechSynthesis.pause();
      } else {
        callNativeApi('tts', 'pause', {});
      }
    },

    resume(): void {
      if ('speechSynthesis' in window) {
        window.speechSynthesis.resume();
      } else {
        callNativeApi('tts', 'resume', {});
      }
    },

    async isSpeaking(): Promise<boolean> {
      if ('speechSynthesis' in window) {
        return window.speechSynthesis.speaking;
      }
      return callNativeApi('tts', 'isSpeaking', {});
    },

    async getVoices(): Promise<TtsVoice[]> {
      if ('speechSynthesis' in window) {
        return window.speechSynthesis.getVoices().map((v) => ({
          voiceName: v.name,
          lang: v.lang,
          remote: v.localService === false,
        }));
      }
      return callNativeApi('tts', 'getVoices', {});
    },

    onVoicesChanged,
  };
}
