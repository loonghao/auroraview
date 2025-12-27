/**
 * Chrome DeclarativeNetRequest API Polyfill
 */

import type { ChromeEvent } from '../types';
import { EventEmitter } from '../event_emitter';

type NativeApiCaller = (api: string, method: string, params?: Record<string, any>) => Promise<any>;

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
  | 'webtransport'
  | 'webbundle'
  | 'other';

export type RuleActionType = 'block' | 'redirect' | 'allow' | 'upgradeScheme' | 'modifyHeaders' | 'allowAllRequests';

export interface RuleCondition {
  urlFilter?: string;
  regexFilter?: string;
  isUrlFilterCaseSensitive?: boolean;
  initiatorDomains?: string[];
  excludedInitiatorDomains?: string[];
  requestDomains?: string[];
  excludedRequestDomains?: string[];
  resourceTypes?: ResourceType[];
  excludedResourceTypes?: ResourceType[];
  requestMethods?: string[];
  excludedRequestMethods?: string[];
  domainType?: 'firstParty' | 'thirdParty';
  tabIds?: number[];
  excludedTabIds?: number[];
}

export interface RuleAction {
  type: RuleActionType;
  redirect?: { url?: string; extensionPath?: string; transform?: any; regexSubstitution?: string };
  requestHeaders?: { header: string; operation: string; value?: string }[];
  responseHeaders?: { header: string; operation: string; value?: string }[];
}

export interface Rule {
  id: number;
  priority?: number;
  condition: RuleCondition;
  action: RuleAction;
}

export interface UpdateRuleOptions {
  removeRuleIds?: number[];
  addRules?: Rule[];
}

export interface DeclarativeNetRequestApi {
  updateDynamicRules(options: UpdateRuleOptions): Promise<void>;
  getDynamicRules(filter?: { ruleIds?: number[] }): Promise<Rule[]>;
  updateSessionRules(options: UpdateRuleOptions): Promise<void>;
  getSessionRules(filter?: { ruleIds?: number[] }): Promise<Rule[]>;
  updateEnabledRulesets(options: { disableRulesetIds?: string[]; enableRulesetIds?: string[] }): Promise<void>;
  getEnabledRulesets(): Promise<string[]>;
  getAvailableStaticRuleCount(): Promise<number>;
  getMatchedRules(filter?: { tabId?: number; minTimeStamp?: number }): Promise<{ rulesMatchedInfo: any[] }>;
  setExtensionActionOptions(options: { displayActionCountAsBadgeText?: boolean; tabUpdate?: any }): Promise<void>;
  isRegexSupported(regexOptions: { regex: string; isCaseSensitive?: boolean; requireCapturing?: boolean }): Promise<{ isSupported: boolean; reason?: string }>;
  onRuleMatchedDebug: ChromeEvent<(info: any) => void>;
  MAX_NUMBER_OF_DYNAMIC_AND_SESSION_RULES: number;
  MAX_NUMBER_OF_ENABLED_STATIC_RULESETS: number;
  GETMATCHEDRULES_QUOTA_INTERVAL: number;
  MAX_GETMATCHEDRULES_CALLS_PER_INTERVAL: number;
  DYNAMIC_RULESET_ID: string;
  SESSION_RULESET_ID: string;
  GUARANTEED_MINIMUM_STATIC_RULES: number;
  MAX_NUMBER_OF_REGEX_RULES: number;
}

export function createDeclarativeNetRequestApi(callNativeApi: NativeApiCaller): DeclarativeNetRequestApi {
  const onRuleMatchedDebug = new EventEmitter<[any]>();

  return {
    async updateDynamicRules(options: UpdateRuleOptions): Promise<void> {
      return callNativeApi('declarativeNetRequest', 'updateDynamicRules', options);
    },

    async getDynamicRules(filter?): Promise<Rule[]> {
      return callNativeApi('declarativeNetRequest', 'getDynamicRules', filter || {});
    },

    async updateSessionRules(options: UpdateRuleOptions): Promise<void> {
      return callNativeApi('declarativeNetRequest', 'updateSessionRules', options);
    },

    async getSessionRules(filter?): Promise<Rule[]> {
      return callNativeApi('declarativeNetRequest', 'getSessionRules', filter || {});
    },

    async updateEnabledRulesets(options): Promise<void> {
      return callNativeApi('declarativeNetRequest', 'updateEnabledRulesets', options);
    },

    async getEnabledRulesets(): Promise<string[]> {
      return callNativeApi('declarativeNetRequest', 'getEnabledRulesets', {});
    },

    async getAvailableStaticRuleCount(): Promise<number> {
      return callNativeApi('declarativeNetRequest', 'getAvailableStaticRuleCount', {});
    },

    async getMatchedRules(filter?): Promise<{ rulesMatchedInfo: any[] }> {
      return callNativeApi('declarativeNetRequest', 'getMatchedRules', filter || {});
    },

    async setExtensionActionOptions(options): Promise<void> {
      return callNativeApi('declarativeNetRequest', 'setExtensionActionOptions', options);
    },

    async isRegexSupported(regexOptions): Promise<{ isSupported: boolean; reason?: string }> {
      return callNativeApi('declarativeNetRequest', 'isRegexSupported', regexOptions);
    },

    onRuleMatchedDebug,
    MAX_NUMBER_OF_DYNAMIC_AND_SESSION_RULES: 5000,
    MAX_NUMBER_OF_ENABLED_STATIC_RULESETS: 50,
    GETMATCHEDRULES_QUOTA_INTERVAL: 10,
    MAX_GETMATCHEDRULES_CALLS_PER_INTERVAL: 20,
    DYNAMIC_RULESET_ID: '_dynamic',
    SESSION_RULESET_ID: '_session',
    GUARANTEED_MINIMUM_STATIC_RULES: 30000,
    MAX_NUMBER_OF_REGEX_RULES: 1000,
  };
}
