/**
 * AuroraView Browser Extension Client
 *
 * This module provides a client for Chrome/Firefox extensions to communicate
 * with AuroraView Python applications via WebSocket and HTTP.
 *
 * @example
 * ```typescript
 * import { AuroraViewClient } from '@auroraview/sdk/browser-extension';
 *
 * const client = new AuroraViewClient({
 *   wsPort: 9001,
 *   httpPort: 9002,
 * });
 *
 * // Connect to AuroraView
 * await client.connect();
 *
 * // Call a handler
 * const result = await client.call('get_scene_info', { format: 'json' });
 * console.log(result);
 *
 * // Listen for events
 * client.on('scene_updated', (data) => {
 *   console.log('Scene updated:', data);
 * });
 * ```
 */

export interface AuroraViewClientConfig {
  /** WebSocket server port (default: 9001) */
  wsPort?: number;
  /** HTTP server port (default: 9002) */
  httpPort?: number;
  /** Server host (default: "127.0.0.1") */
  host?: string;
  /** Auto-reconnect on disconnect (default: true) */
  autoReconnect?: boolean;
  /** Reconnect interval in ms (default: 3000) */
  reconnectInterval?: number;
  /** Request timeout in ms (default: 30000) */
  timeout?: number;
}

export interface CallOptions {
  /** Request timeout in ms */
  timeout?: number;
  /** Use HTTP instead of WebSocket */
  useHttp?: boolean;
}

type EventCallback = (data: any) => void;

interface PendingRequest {
  resolve: (value: any) => void;
  reject: (error: Error) => void;
  timeout: ReturnType<typeof setTimeout>;
}

/**
 * AuroraView Browser Extension Client
 *
 * Provides WebSocket and HTTP communication with AuroraView Python bridge.
 */
export class AuroraViewClient {
  private config: Required<AuroraViewClientConfig>;
  private ws: WebSocket | null = null;
  private requestId = 0;
  private pendingRequests = new Map<string, PendingRequest>();
  private eventHandlers = new Map<string, Set<EventCallback>>();
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  private isConnecting = false;

  constructor(config: AuroraViewClientConfig = {}) {
    this.config = {
      wsPort: config.wsPort ?? 9001,
      httpPort: config.httpPort ?? 9002,
      host: config.host ?? "127.0.0.1",
      autoReconnect: config.autoReconnect ?? true,
      reconnectInterval: config.reconnectInterval ?? 3000,
      timeout: config.timeout ?? 30000,
    };
  }

  /**
   * Get WebSocket URL
   */
  get wsUrl(): string {
    return `ws://${this.config.host}:${this.config.wsPort}`;
  }

  /**
   * Get HTTP base URL
   */
  get httpUrl(): string {
    return `http://${this.config.host}:${this.config.httpPort}`;
  }

  /**
   * Check if connected via WebSocket
   */
  get isConnected(): boolean {
    return this.ws?.readyState === WebSocket.OPEN;
  }

  /**
   * Connect to AuroraView via WebSocket
   */
  async connect(): Promise<void> {
    if (this.isConnected) {
      return;
    }

    if (this.isConnecting) {
      // Wait for existing connection attempt
      return new Promise((resolve, reject) => {
        const checkConnection = setInterval(() => {
          if (this.isConnected) {
            clearInterval(checkConnection);
            resolve();
          } else if (!this.isConnecting) {
            clearInterval(checkConnection);
            reject(new Error("Connection failed"));
          }
        }, 100);
      });
    }

    this.isConnecting = true;

    return new Promise((resolve, reject) => {
      try {
        this.ws = new WebSocket(this.wsUrl);

        this.ws.onopen = () => {
          console.log("[AuroraView] WebSocket connected");
          this.isConnecting = false;
          this.emit("connected", {});
          resolve();
        };

        this.ws.onclose = (event) => {
          console.log("[AuroraView] WebSocket closed:", event.code, event.reason);
          this.isConnecting = false;
          this.emit("disconnected", { code: event.code, reason: event.reason });

          // Reject pending requests
          for (const [, request] of this.pendingRequests) {
            clearTimeout(request.timeout);
            request.reject(new Error("Connection closed"));
          }
          this.pendingRequests.clear();

          // Auto-reconnect
          if (this.config.autoReconnect && !this.reconnectTimer) {
            this.scheduleReconnect();
          }
        };

        this.ws.onerror = (error) => {
          console.error("[AuroraView] WebSocket error:", error);
          this.isConnecting = false;
          reject(new Error("WebSocket connection failed"));
        };

        this.ws.onmessage = (event) => {
          this.handleMessage(event.data);
        };
      } catch (error) {
        this.isConnecting = false;
        reject(error);
      }
    });
  }

  /**
   * Disconnect from AuroraView
   */
  disconnect(): void {
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }

    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }

  /**
   * Call a handler on AuroraView
   *
   * @param action - Handler action name
   * @param data - Data to send
   * @param options - Call options
   * @returns Handler response
   */
  async call<T = any>(
    action: string,
    data: Record<string, any> = {},
    options: CallOptions = {}
  ): Promise<T> {
    if (options.useHttp) {
      return this.callHttp(action, data, options);
    }
    return this.callWs(action, data, options);
  }

  /**
   * Call via WebSocket
   */
  private async callWs<T = any>(
    action: string,
    data: Record<string, any>,
    options: CallOptions
  ): Promise<T> {
    if (!this.isConnected) {
      await this.connect();
    }

    const requestId = `req_${++this.requestId}`;
    const timeout = options.timeout ?? this.config.timeout;

    return new Promise((resolve, reject) => {
      const timeoutHandle = setTimeout(() => {
        this.pendingRequests.delete(requestId);
        reject(new Error(`Request timeout: ${action}`));
      }, timeout);

      this.pendingRequests.set(requestId, {
        resolve,
        reject,
        timeout: timeoutHandle,
      });

      const message = JSON.stringify({
        action,
        data,
        requestId,
      });

      this.ws!.send(message);
    });
  }

  /**
   * Call via HTTP
   */
  private async callHttp<T = any>(
    action: string,
    data: Record<string, any>,
    options: CallOptions
  ): Promise<T> {
    const timeout = options.timeout ?? this.config.timeout;

    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), timeout);

    try {
      const response = await fetch(`${this.httpUrl}/call`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({ action, data }),
        signal: controller.signal,
      });

      clearTimeout(timeoutId);

      const result = await response.json();

      if (!response.ok) {
        throw new Error(result.error || `HTTP ${response.status}`);
      }

      return result.result as T;
    } catch (error) {
      clearTimeout(timeoutId);
      if (error instanceof Error && error.name === "AbortError") {
        throw new Error(`Request timeout: ${action}`);
      }
      throw error;
    }
  }

  /**
   * Register event handler
   *
   * @param event - Event name
   * @param callback - Event callback
   * @returns Unsubscribe function
   */
  on(event: string, callback: EventCallback): () => void {
    if (!this.eventHandlers.has(event)) {
      this.eventHandlers.set(event, new Set());
    }
    this.eventHandlers.get(event)!.add(callback);

    return () => {
      this.eventHandlers.get(event)?.delete(callback);
    };
  }

  /**
   * Remove event handler
   */
  off(event: string, callback: EventCallback): void {
    this.eventHandlers.get(event)?.delete(callback);
  }

  /**
   * Emit event locally
   */
  private emit(event: string, data: any): void {
    const handlers = this.eventHandlers.get(event);
    if (handlers) {
      for (const handler of handlers) {
        try {
          handler(data);
        } catch (error) {
          console.error(`[AuroraView] Event handler error (${event}):`, error);
        }
      }
    }
  }

  /**
   * Handle incoming WebSocket message
   */
  private handleMessage(rawData: string): void {
    try {
      const message = JSON.parse(rawData);

      if (message.type === "response" && message.requestId) {
        // Response to a request
        const pending = this.pendingRequests.get(message.requestId);
        if (pending) {
          clearTimeout(pending.timeout);
          this.pendingRequests.delete(message.requestId);
          pending.resolve(message.data);
        }
      } else if (message.type === "error" && message.requestId) {
        // Error response
        const pending = this.pendingRequests.get(message.requestId);
        if (pending) {
          clearTimeout(pending.timeout);
          this.pendingRequests.delete(message.requestId);
          pending.reject(new Error(message.error || "Unknown error"));
        }
      } else if (message.type === "event") {
        // Server-pushed event
        this.emit(message.action, message.data);
      }
    } catch (error) {
      console.error("[AuroraView] Failed to parse message:", error);
    }
  }

  /**
   * Schedule reconnection
   */
  private scheduleReconnect(): void {
    this.reconnectTimer = setTimeout(async () => {
      this.reconnectTimer = null;
      console.log("[AuroraView] Attempting to reconnect...");
      try {
        await this.connect();
      } catch (error) {
        console.error("[AuroraView] Reconnection failed:", error);
        if (this.config.autoReconnect) {
          this.scheduleReconnect();
        }
      }
    }, this.config.reconnectInterval);
  }

  /**
   * Check if AuroraView bridge is available
   */
  async checkHealth(): Promise<boolean> {
    try {
      const response = await fetch(`${this.httpUrl}/health`, {
        method: "GET",
      });
      return response.ok;
    } catch {
      return false;
    }
  }

  /**
   * Get bridge info
   */
  async getInfo(): Promise<{
    name: string;
    version: string;
    ws_url: string;
    capabilities: string[];
    handlers: string[];
  }> {
    const response = await fetch(`${this.httpUrl}/info`);
    return response.json();
  }
}

/**
 * Create a pre-configured client instance
 */
export function createAuroraViewClient(
  config?: AuroraViewClientConfig
): AuroraViewClient {
  return new AuroraViewClient(config);
}

/**
 * Singleton client for simple usage
 */
let defaultClient: AuroraViewClient | null = null;

export function getDefaultClient(): AuroraViewClient {
  if (!defaultClient) {
    defaultClient = new AuroraViewClient();
  }
  return defaultClient;
}
