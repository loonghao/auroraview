/**
 * AuroraView Debug Interceptor
 *
 * This script intercepts console logs and network requests for debugging purposes.
 * It is designed to be injected into WebView pages to enable MCP debugging tools.
 *
 * @module debug_interceptor
 */

(function () {
  'use strict';

  // Avoid double initialization
  const w = window as unknown as Record<string, unknown>;
  if (w.__auroraview_debug_interceptor_installed) {
    return;
  }
  w.__auroraview_debug_interceptor_installed = true;

  // Configuration
  const MAX_LOGS = 1000;
  const MAX_REQUESTS = 500;

  // Storage for intercepted data
  interface ConsoleLogEntry {
    level: string;
    text: string;
    timestamp: number;
    source?: string;
    line?: number;
    stack?: string;
  }

  interface NetworkRequestEntry {
    url: string;
    method: string;
    status: number;
    type: string;
    size: number;
    time: number;
    startTime: number;
    headers?: Record<string, string>;
    body?: string;
    response?: string;
  }

  // Initialize storage arrays
  w.__auroraview_console_logs = [] as ConsoleLogEntry[];
  w.__auroraview_network_requests = [] as NetworkRequestEntry[];
  w.__auroraview_errors = [] as Array<{ message: string; source: string; line: number; col: number; timestamp: number }>;

  const consoleLogs = w.__auroraview_console_logs as ConsoleLogEntry[];
  const networkRequests = w.__auroraview_network_requests as NetworkRequestEntry[];
  const errors = w.__auroraview_errors as Array<{ message: string; source: string; line: number; col: number; timestamp: number }>;

  // ==================== Console Interceptor ====================

  const originalConsole = {
    log: console.log.bind(console),
    warn: console.warn.bind(console),
    error: console.error.bind(console),
    info: console.info.bind(console),
    debug: console.debug.bind(console),
  };

  function formatArgs(args: unknown[]): string {
    return args.map(arg => {
      if (arg === null) return 'null';
      if (arg === undefined) return 'undefined';
      if (typeof arg === 'object') {
        try {
          return JSON.stringify(arg, null, 2);
        } catch {
          return String(arg);
        }
      }
      return String(arg);
    }).join(' ');
  }

  function getStackInfo(): { source?: string; line?: number; stack?: string } {
    try {
      const stack = new Error().stack;
      if (!stack) return {};

      const lines = stack.split('\n');
      // Skip Error, interceptor frames
      for (let i = 3; i < lines.length; i++) {
        const line = lines[i];
        if (!line.includes('debug_interceptor') && !line.includes('<anonymous>')) {
          const match = line.match(/at\s+(?:.*?\s+)?\(?(.+?):(\d+):\d+\)?/);
          if (match) {
            return {
              source: match[1],
              line: parseInt(match[2], 10),
              stack: lines.slice(i).join('\n'),
            };
          }
        }
      }
    } catch {
      // Ignore stack parsing errors
    }
    return {};
  }

  function interceptConsole(level: keyof typeof originalConsole): void {
    console[level] = function (...args: unknown[]): void {
      // Call original
      originalConsole[level](...args);

      // Store log entry
      const entry: ConsoleLogEntry = {
        level,
        text: formatArgs(args),
        timestamp: Date.now(),
        ...getStackInfo(),
      };

      consoleLogs.push(entry);

      // Trim if exceeds max
      if (consoleLogs.length > MAX_LOGS) {
        consoleLogs.splice(0, consoleLogs.length - MAX_LOGS);
      }
    };
  }

  // Install console interceptors
  interceptConsole('log');
  interceptConsole('warn');
  interceptConsole('error');
  interceptConsole('info');
  interceptConsole('debug');

  // ==================== Error Interceptor ====================

  window.addEventListener('error', (event) => {
    errors.push({
      message: event.message,
      source: event.filename || '',
      line: event.lineno || 0,
      col: event.colno || 0,
      timestamp: Date.now(),
    });

    // Trim if exceeds max
    if (errors.length > MAX_LOGS) {
      errors.splice(0, errors.length - MAX_LOGS);
    }
  });

  window.addEventListener('unhandledrejection', (event) => {
    const reason = event.reason;
    errors.push({
      message: reason instanceof Error ? reason.message : String(reason),
      source: 'Promise',
      line: 0,
      col: 0,
      timestamp: Date.now(),
    });
  });

  // ==================== Network Interceptor ====================

  // Intercept fetch
  const originalFetch = window.fetch.bind(window);

  window.fetch = async function (input: RequestInfo | URL, init?: RequestInit): Promise<Response> {
    const startTime = Date.now();
    const url = typeof input === 'string' ? input : input instanceof URL ? input.href : input.url;
    const method = init?.method || 'GET';

    try {
      const response = await originalFetch(input, init);
      const endTime = Date.now();

      // Clone response to read body without consuming it
      const clonedResponse = response.clone();
      let responseText = '';
      try {
        responseText = await clonedResponse.text();
        if (responseText.length > 10000) {
          responseText = responseText.substring(0, 10000) + '... (truncated)';
        }
      } catch {
        // Ignore body read errors
      }

      const entry: NetworkRequestEntry = {
        url,
        method,
        status: response.status,
        type: 'fetch',
        size: parseInt(response.headers.get('content-length') || '0', 10) || responseText.length,
        time: endTime - startTime,
        startTime,
        response: responseText,
      };

      networkRequests.push(entry);

      // Trim if exceeds max
      if (networkRequests.length > MAX_REQUESTS) {
        networkRequests.splice(0, networkRequests.length - MAX_REQUESTS);
      }

      return response;
    } catch (error) {
      const endTime = Date.now();

      networkRequests.push({
        url,
        method,
        status: 0,
        type: 'fetch',
        size: 0,
        time: endTime - startTime,
        startTime,
        response: error instanceof Error ? error.message : String(error),
      });

      throw error;
    }
  };

  // Intercept XMLHttpRequest
  const XHROpen = XMLHttpRequest.prototype.open;
  const XHRSend = XMLHttpRequest.prototype.send;

  XMLHttpRequest.prototype.open = function (
    method: string,
    url: string | URL,
    async?: boolean,
    username?: string | null,
    password?: string | null
  ): void {
    (this as XMLHttpRequest & { _auroraview_method?: string; _auroraview_url?: string })._auroraview_method = method;
    (this as XMLHttpRequest & { _auroraview_url?: string })._auroraview_url = typeof url === 'string' ? url : url.href;
    return XHROpen.call(this, method, url, async ?? true, username, password);
  };

  XMLHttpRequest.prototype.send = function (body?: Document | XMLHttpRequestBodyInit | null): void {
    const xhr = this as XMLHttpRequest & { _auroraview_method?: string; _auroraview_url?: string; _auroraview_startTime?: number };
    xhr._auroraview_startTime = Date.now();

    this.addEventListener('loadend', function () {
      const endTime = Date.now();
      const entry: NetworkRequestEntry = {
        url: xhr._auroraview_url || '',
        method: xhr._auroraview_method || 'GET',
        status: xhr.status,
        type: 'xhr',
        size: parseInt(xhr.getResponseHeader('content-length') || '0', 10) || (xhr.responseText?.length || 0),
        time: endTime - (xhr._auroraview_startTime || endTime),
        startTime: xhr._auroraview_startTime || endTime,
      };

      networkRequests.push(entry);

      // Trim if exceeds max
      if (networkRequests.length > MAX_REQUESTS) {
        networkRequests.splice(0, networkRequests.length - MAX_REQUESTS);
      }
    });

    return XHRSend.call(this, body);
  };

  // ==================== Debug API ====================

  // Expose debug utilities on window.auroraview
  if (window.auroraview) {
    (window.auroraview as unknown as Record<string, unknown>).debug = {
      /**
       * Get all console logs
       */
      getLogs: (level?: string, limit?: number): ConsoleLogEntry[] => {
        let logs = consoleLogs;
        if (level) {
          logs = logs.filter(log => log.level === level);
        }
        if (limit && limit > 0) {
          logs = logs.slice(-limit);
        }
        return logs;
      },

      /**
       * Get all network requests
       */
      getRequests: (urlPattern?: string, method?: string): NetworkRequestEntry[] => {
        let requests = networkRequests;
        if (urlPattern) {
          const regex = new RegExp(urlPattern);
          requests = requests.filter(r => regex.test(r.url));
        }
        if (method) {
          requests = requests.filter(r => r.method.toUpperCase() === method.toUpperCase());
        }
        return requests;
      },

      /**
       * Get all errors
       */
      getErrors: (): typeof errors => {
        return errors;
      },

      /**
       * Clear all logs
       */
      clearLogs: (): void => {
        consoleLogs.length = 0;
      },

      /**
       * Clear all requests
       */
      clearRequests: (): void => {
        networkRequests.length = 0;
      },

      /**
       * Clear all errors
       */
      clearErrors: (): void => {
        errors.length = 0;
      },

      /**
       * Clear all debug data
       */
      clearAll: (): void => {
        consoleLogs.length = 0;
        networkRequests.length = 0;
        errors.length = 0;
      },

      /**
       * Get debug stats
       */
      getStats: (): { logs: number; requests: number; errors: number } => {
        return {
          logs: consoleLogs.length,
          requests: networkRequests.length,
          errors: errors.length,
        };
      },
    };
  }

  // Store version info
  w.__auroraview_debug_version = '1.0.0';

  console.log('[AuroraView] Debug interceptor installed');
})();
