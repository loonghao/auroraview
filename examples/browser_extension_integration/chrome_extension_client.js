/**
 * AuroraView Chrome Extension Client Example
 * 
 * This file shows how to integrate your Chrome extension with AuroraView.
 * Copy this code into your extension's content script, side panel, or background script.
 * 
 * Usage in your Chrome extension:
 * 
 * 1. In your side panel or popup:
 *    import { AuroraViewClient } from './auroraview-client.js';
 *    const client = new AuroraViewClient();
 *    await client.connect();
 *    const info = await client.call('get_scene_info');
 * 
 * 2. In content script (with messaging to background):
 *    // Content scripts can't directly use WebSocket to local ports
 *    // Use chrome.runtime.sendMessage to communicate with background script
 */

/**
 * AuroraView Client for Chrome Extensions
 */
class AuroraViewClient {
  constructor(config = {}) {
    this.wsPort = config.wsPort || 49152;
    this.httpPort = config.httpPort || 49153;
    this.host = config.host || '127.0.0.1';
    this.autoReconnect = config.autoReconnect !== false;
    this.reconnectInterval = config.reconnectInterval || 3000;
    this.timeout = config.timeout || 30000;
    
    this.ws = null;
    this.requestId = 0;
    this.pendingRequests = new Map();
    this.eventHandlers = new Map();
    this.reconnectTimer = null;
    this.isConnecting = false;
  }

  get wsUrl() {
    return `ws://${this.host}:${this.wsPort}`;
  }

  get httpUrl() {
    return `http://${this.host}:${this.httpPort}`;
  }

  get isConnected() {
    return this.ws?.readyState === WebSocket.OPEN;
  }

  /**
   * Connect to AuroraView via WebSocket
   */
  async connect() {
    if (this.isConnected) return;
    if (this.isConnecting) {
      return new Promise((resolve, reject) => {
        const check = setInterval(() => {
          if (this.isConnected) {
            clearInterval(check);
            resolve();
          } else if (!this.isConnecting) {
            clearInterval(check);
            reject(new Error('Connection failed'));
          }
        }, 100);
      });
    }

    this.isConnecting = true;

    return new Promise((resolve, reject) => {
      try {
        this.ws = new WebSocket(this.wsUrl);

        this.ws.onopen = () => {
          console.log('[AuroraView] Connected');
          this.isConnecting = false;
          this._emit('connected', {});
          resolve();
        };

        this.ws.onclose = (event) => {
          console.log('[AuroraView] Disconnected:', event.code);
          this.isConnecting = false;
          this._emit('disconnected', { code: event.code });

          // Reject pending requests
          for (const [id, req] of this.pendingRequests) {
            clearTimeout(req.timeout);
            req.reject(new Error('Connection closed'));
          }
          this.pendingRequests.clear();

          // Auto-reconnect
          if (this.autoReconnect && !this.reconnectTimer) {
            this._scheduleReconnect();
          }
        };

        this.ws.onerror = (error) => {
          console.error('[AuroraView] Error:', error);
          this.isConnecting = false;
          reject(new Error('WebSocket connection failed'));
        };

        this.ws.onmessage = (event) => {
          this._handleMessage(event.data);
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
  disconnect() {
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
   * @param {string} action - Handler action name
   * @param {object} data - Data to send
   * @param {object} options - Call options
   * @returns {Promise<any>} Handler response
   */
  async call(action, data = {}, options = {}) {
    if (options.useHttp) {
      return this._callHttp(action, data, options);
    }
    return this._callWs(action, data, options);
  }

  async _callWs(action, data, options) {
    if (!this.isConnected) {
      await this.connect();
    }

    const requestId = `req_${++this.requestId}`;
    const timeout = options.timeout || this.timeout;

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

      this.ws.send(JSON.stringify({ action, data, requestId }));
    });
  }

  async _callHttp(action, data, options) {
    const timeout = options.timeout || this.timeout;
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), timeout);

    try {
      const response = await fetch(`${this.httpUrl}/call`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ action, data }),
        signal: controller.signal,
      });

      clearTimeout(timeoutId);
      const result = await response.json();

      if (!response.ok) {
        throw new Error(result.error || `HTTP ${response.status}`);
      }

      return result.result;
    } catch (error) {
      clearTimeout(timeoutId);
      if (error.name === 'AbortError') {
        throw new Error(`Request timeout: ${action}`);
      }
      throw error;
    }
  }

  /**
   * Register event handler
   * @param {string} event - Event name
   * @param {function} callback - Event callback
   * @returns {function} Unsubscribe function
   */
  on(event, callback) {
    if (!this.eventHandlers.has(event)) {
      this.eventHandlers.set(event, new Set());
    }
    this.eventHandlers.get(event).add(callback);
    return () => this.eventHandlers.get(event)?.delete(callback);
  }

  /**
   * Remove event handler
   */
  off(event, callback) {
    this.eventHandlers.get(event)?.delete(callback);
  }

  _emit(event, data) {
    const handlers = this.eventHandlers.get(event);
    if (handlers) {
      for (const handler of handlers) {
        try {
          handler(data);
        } catch (error) {
          console.error(`[AuroraView] Event handler error:`, error);
        }
      }
    }
  }

  _handleMessage(rawData) {
    try {
      const message = JSON.parse(rawData);

      if (message.type === 'response' && message.requestId) {
        const pending = this.pendingRequests.get(message.requestId);
        if (pending) {
          clearTimeout(pending.timeout);
          this.pendingRequests.delete(message.requestId);
          pending.resolve(message.data);
        }
      } else if (message.type === 'error' && message.requestId) {
        const pending = this.pendingRequests.get(message.requestId);
        if (pending) {
          clearTimeout(pending.timeout);
          this.pendingRequests.delete(message.requestId);
          pending.reject(new Error(message.error || 'Unknown error'));
        }
      } else if (message.type === 'event') {
        this._emit(message.action, message.data);
      }
    } catch (error) {
      console.error('[AuroraView] Parse error:', error);
    }
  }

  _scheduleReconnect() {
    this.reconnectTimer = setTimeout(async () => {
      this.reconnectTimer = null;
      console.log('[AuroraView] Reconnecting...');
      try {
        await this.connect();
      } catch (error) {
        console.error('[AuroraView] Reconnect failed:', error);
        if (this.autoReconnect) {
          this._scheduleReconnect();
        }
      }
    }, this.reconnectInterval);
  }

  /**
   * Check if AuroraView bridge is available
   */
  async checkHealth() {
    try {
      const response = await fetch(`${this.httpUrl}/health`);
      return response.ok;
    } catch {
      return false;
    }
  }

  /**
   * Get bridge info
   */
  async getInfo() {
    const response = await fetch(`${this.httpUrl}/info`);
    return response.json();
  }
}

// =============================================================================
// Example Usage in Chrome Extension
// =============================================================================

/**
 * Example: Initialize client and connect
 */
async function initAuroraView() {
  const client = new AuroraViewClient({
    wsPort: 49152,
    httpPort: 49153,
  });

  // Listen for connection events
  client.on('connected', () => {
    console.log('Connected to AuroraView!');
  });

  client.on('disconnected', () => {
    console.log('Disconnected from AuroraView');
  });

  // Listen for server-pushed events
  client.on('object_created', (data) => {
    console.log('New object created:', data);
  });

  client.on('frame_changed', (data) => {
    console.log('Frame changed:', data);
  });

  // Connect
  try {
    await client.connect();
    return client;
  } catch (error) {
    console.error('Failed to connect:', error);
    return null;
  }
}

/**
 * Example: Call handlers
 */
async function exampleUsage(client) {
  if (!client) return;

  // Get scene info
  const sceneInfo = await client.call('get_scene_info');
  console.log('Scene info:', sceneInfo);

  // List objects
  const objects = await client.call('list_objects');
  console.log('Objects:', objects);

  // Create a new object
  const newObj = await client.call('create_object', {
    type: 'sphere',
    name: 'mySphere'
  });
  console.log('Created:', newObj);

  // Select an object
  const selection = await client.call('select_object', { name: 'pCube1' });
  console.log('Selection:', selection);

  // Set frame
  await client.call('set_frame', { frame: 50 });

  // Ping test
  const pong = await client.call('ping');
  console.log('Ping response:', pong);
}

/**
 * Example: HTTP-only mode (no WebSocket)
 */
async function httpOnlyExample() {
  const client = new AuroraViewClient();

  // Use HTTP for all calls
  const info = await client.call('get_scene_info', {}, { useHttp: true });
  console.log('Scene info (HTTP):', info);
}

// Export for use in Chrome extension
if (typeof module !== 'undefined' && module.exports) {
  module.exports = { AuroraViewClient, initAuroraView, exampleUsage };
}

// For ES modules
export { AuroraViewClient, initAuroraView, exampleUsage, httpOnlyExample };
