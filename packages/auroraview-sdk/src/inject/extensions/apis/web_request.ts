/**
 * Chrome WebRequest API Polyfill
 */

import { createWebRequestEvent } from '../event_emitter';

type NativeApiCaller = (api: string, method: string, params?: Record<string, any>) => Promise<any>;

export interface RequestFilter {
  urls: string[];
  types?: ResourceType[];
  tabId?: number;
  windowId?: number;
}

export type ResourceType =
  | 'main_frame'
  | 'sub_frame'
  | 'stylesheet'
  | 'script'
  | 'image'
  | 'font'
  | 'object'
  | 'xmlhttprequest'
  | 'ping'
  | 'csp_report'
  | 'media'
  | 'websocket'
  | 'webbundle'
  | 'other';

export interface WebRequestDetails {
  requestId: string;
  url: string;
  method: string;
  frameId: number;
  parentFrameId: number;
  tabId: number;
  type: ResourceType;
  timeStamp: number;
  initiator?: string;
  documentId?: string;
  documentLifecycle?: string;
  frameType?: string;
  parentDocumentId?: string;
}

export interface BlockingResponse {
  cancel?: boolean;
  redirectUrl?: string;
  requestHeaders?: { name: string; value: string }[];
  responseHeaders?: { name: string; value: string }[];
  authCredentials?: { username: string; password: string };
}

export interface WebRequestApi {
  onBeforeRequest: ReturnType<typeof createWebRequestEvent>;
  onBeforeSendHeaders: ReturnType<typeof createWebRequestEvent>;
  onSendHeaders: ReturnType<typeof createWebRequestEvent>;
  onHeadersReceived: ReturnType<typeof createWebRequestEvent>;
  onAuthRequired: ReturnType<typeof createWebRequestEvent>;
  onResponseStarted: ReturnType<typeof createWebRequestEvent>;
  onBeforeRedirect: ReturnType<typeof createWebRequestEvent>;
  onCompleted: ReturnType<typeof createWebRequestEvent>;
  onErrorOccurred: ReturnType<typeof createWebRequestEvent>;
  handlerBehaviorChanged(): Promise<void>;
  MAX_HANDLER_BEHAVIOR_CHANGED_CALLS_PER_10_MINUTES: number;
}

export function createWebRequestApi(callNativeApi: NativeApiCaller): WebRequestApi {
  return {
    onBeforeRequest: createWebRequestEvent('onBeforeRequest', callNativeApi),
    onBeforeSendHeaders: createWebRequestEvent('onBeforeSendHeaders', callNativeApi),
    onSendHeaders: createWebRequestEvent('onSendHeaders', callNativeApi),
    onHeadersReceived: createWebRequestEvent('onHeadersReceived', callNativeApi),
    onAuthRequired: createWebRequestEvent('onAuthRequired', callNativeApi),
    onResponseStarted: createWebRequestEvent('onResponseStarted', callNativeApi),
    onBeforeRedirect: createWebRequestEvent('onBeforeRedirect', callNativeApi),
    onCompleted: createWebRequestEvent('onCompleted', callNativeApi),
    onErrorOccurred: createWebRequestEvent('onErrorOccurred', callNativeApi),

    async handlerBehaviorChanged(): Promise<void> {
      return callNativeApi('webRequest', 'handlerBehaviorChanged', {});
    },

    MAX_HANDLER_BEHAVIOR_CHANGED_CALLS_PER_10_MINUTES: 20,
  };
}
