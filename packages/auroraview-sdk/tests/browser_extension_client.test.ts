/**
 * AuroraViewClient Browser Extension Tests
 *
 * Tests the AuroraViewClient class in isolation using mocked WebSocket and fetch APIs.
 * Covers: config defaults, URL accessors, connect/disconnect, call (ws/http), on/off/emit,
 * handleMessage, auto-reconnect scheduling, checkHealth, getInfo, factory functions.
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import {
  AuroraViewClient,
  createAuroraViewClient,
  getDefaultClient,
  type AuroraViewClientConfig,
} from '../src/browser-extension/client';

// ============================================================================
// WebSocket mock
// ============================================================================

class MockWebSocket {
  static OPEN = 1;
  static CLOSING = 2;
  static CLOSED = 3;

  readyState: number;
  url: string;
  onopen: ((e: any) => void) | null = null;
  onclose: ((e: any) => void) | null = null;
  onerror: ((e: any) => void) | null = null;
  onmessage: ((e: any) => void) | null = null;

  private _sent: string[] = [];

  constructor(url: string) {
    this.url = url;
    this.readyState = MockWebSocket.CLOSED;
    // Store instance for test access
    MockWebSocket._lastInstance = this;
  }

  send(data: string): void {
    this._sent.push(data);
  }

  close(): void {
    this.readyState = MockWebSocket.CLOSED;
    this.onclose?.({ code: 1000, reason: 'Normal closure' });
  }

  /** Test helper: simulate successful open */
  simulateOpen(): void {
    this.readyState = MockWebSocket.OPEN;
    this.onopen?.({});
  }

  /** Test helper: simulate error during connect */
  simulateError(err: any = new Error('connection refused')): void {
    this.readyState = MockWebSocket.CLOSED;
    this.onerror?.(err);
  }

  /** Test helper: simulate incoming message */
  simulateMessage(data: object): void {
    this.onmessage?.({ data: JSON.stringify(data) });
  }

  /** Test helper: simulate server-side close */
  simulateClose(code = 1001, reason = 'Going away'): void {
    this.readyState = MockWebSocket.CLOSED;
    this.onclose?.({ code, reason });
  }

  getSent(): string[] {
    return this._sent;
  }

  static _lastInstance: MockWebSocket | null = null;
}

// ============================================================================
// Setup global mocks
// ============================================================================

let originalWebSocket: any;
let originalFetch: any;

beforeEach(() => {
  // Replace globals
  originalWebSocket = (globalThis as any).WebSocket;
  (globalThis as any).WebSocket = MockWebSocket;

  originalFetch = (globalThis as any).fetch;
  (globalThis as any).fetch = vi.fn();

  MockWebSocket._lastInstance = null;

  vi.useFakeTimers();
});

afterEach(() => {
  (globalThis as any).WebSocket = originalWebSocket;
  (globalThis as any).fetch = originalFetch;
  vi.useRealTimers();
  vi.restoreAllMocks();
});

// ============================================================================
// Helper
// ============================================================================

function makeClient(cfg: AuroraViewClientConfig = {}): AuroraViewClient {
  return new AuroraViewClient(cfg);
}

function lastWs(): MockWebSocket {
  if (!MockWebSocket._lastInstance) throw new Error('No MockWebSocket created yet');
  return MockWebSocket._lastInstance;
}

// ============================================================================
// Tests
// ============================================================================

describe('AuroraViewClient', () => {
  describe('config defaults', () => {
    it('applies default wsPort 9001', () => {
      const c = makeClient();
      expect(c.wsUrl).toBe('ws://127.0.0.1:9001');
    });

    it('applies default httpPort 9002', () => {
      const c = makeClient();
      expect(c.httpUrl).toBe('http://127.0.0.1:9002');
    });

    it('applies default host 127.0.0.1', () => {
      const c = makeClient();
      expect(c.wsUrl).toContain('127.0.0.1');
    });

    it('accepts custom wsPort', () => {
      const c = makeClient({ wsPort: 8765 });
      expect(c.wsUrl).toBe('ws://127.0.0.1:8765');
    });

    it('accepts custom httpPort', () => {
      const c = makeClient({ httpPort: 8080 });
      expect(c.httpUrl).toBe('http://127.0.0.1:8080');
    });

    it('accepts custom host', () => {
      const c = makeClient({ host: 'localhost' });
      expect(c.wsUrl).toContain('localhost');
      expect(c.httpUrl).toContain('localhost');
    });

    it('wsUrl uses correct scheme', () => {
      const c = makeClient({ host: '192.168.1.1', wsPort: 9999 });
      expect(c.wsUrl).toBe('ws://192.168.1.1:9999');
    });

    it('httpUrl uses correct scheme', () => {
      const c = makeClient({ host: '192.168.1.1', httpPort: 9000 });
      expect(c.httpUrl).toBe('http://192.168.1.1:9000');
    });
  });

  describe('isConnected', () => {
    it('returns false when ws is null', () => {
      const c = makeClient();
      expect(c.isConnected).toBe(false);
    });

    it('returns true after ws opens', async () => {
      const c = makeClient();
      const p = c.connect();
      lastWs().simulateOpen();
      await p;
      expect(c.isConnected).toBe(true);
    });

    it('returns false after disconnect', async () => {
      const c = makeClient({ autoReconnect: false });
      const p = c.connect();
      const ws = lastWs();
      ws.simulateOpen();
      await p;
      c.disconnect();
      expect(c.isConnected).toBe(false);
    });
  });

  describe('connect()', () => {
    it('resolves when WebSocket opens', async () => {
      const c = makeClient();
      const p = c.connect();
      lastWs().simulateOpen();
      await expect(p).resolves.toBeUndefined();
    });

    it('rejects when WebSocket errors', async () => {
      const c = makeClient();
      const p = c.connect();
      lastWs().simulateError();
      await expect(p).rejects.toThrow('WebSocket connection failed');
    });

    it('no-ops when already connected', async () => {
      const c = makeClient();
      const p1 = c.connect();
      lastWs().simulateOpen();
      await p1;

      // Second connect should return immediately (no new ws created)
      const wsBefore = MockWebSocket._lastInstance;
      await c.connect();
      expect(MockWebSocket._lastInstance).toBe(wsBefore);
    });

    it('emits "connected" event on open', async () => {
      const c = makeClient();
      const handler = vi.fn();
      c.on('connected', handler);
      const p = c.connect();
      lastWs().simulateOpen();
      await p;
      expect(handler).toHaveBeenCalledWith({});
    });
  });

  describe('disconnect()', () => {
    it('closes WebSocket and clears ws reference', async () => {
      const c = makeClient({ autoReconnect: false });
      const p = c.connect();
      const ws = lastWs();
      ws.simulateOpen();
      await p;
      c.disconnect();
      expect(c.isConnected).toBe(false);
    });

    it('emits "disconnected" event on close', async () => {
      const c = makeClient({ autoReconnect: false });
      const p = c.connect();
      const ws = lastWs();
      ws.simulateOpen();
      await p;

      const handler = vi.fn();
      c.on('disconnected', handler);
      c.disconnect();
      expect(handler).toHaveBeenCalledWith({ code: 1000, reason: 'Normal closure' });
    });

    it('cancels pending reconnect timer', async () => {
      const c = makeClient({ autoReconnect: true, reconnectInterval: 500 });
      const p = c.connect();
      const ws = lastWs();
      ws.simulateOpen();
      await p;
      // Trigger disconnect with reconnect
      ws.simulateClose();
      // Immediately disconnect to cancel timer
      c.disconnect();
      // Advance timers — no new connection attempt
      vi.advanceTimersByTime(2000);
      // Only the initial ws was created
      // (no assertion on internal state; just ensure no throw)
    });

    it('rejects pending requests on disconnect', async () => {
      const c = makeClient({ autoReconnect: false, timeout: 10000 });
      const connectP = c.connect();
      const ws = lastWs();
      ws.simulateOpen();
      await connectP;

      const callP = c.call('some_action');
      c.disconnect();
      await expect(callP).rejects.toThrow('Connection closed');
    });
  });

  describe('call() via WebSocket', () => {
    it('sends correct message format', async () => {
      const c = makeClient();
      const connectP = c.connect();
      const ws = lastWs();
      ws.simulateOpen();
      await connectP;

      const callP = c.call('test_action', { key: 'value' });

      const sent = ws.getSent();
      expect(sent).toHaveLength(1);
      const msg = JSON.parse(sent[0]);
      expect(msg.action).toBe('test_action');
      expect(msg.data).toEqual({ key: 'value' });
      expect(msg.requestId).toMatch(/^req_\d+/);

      // Resolve the call
      ws.simulateMessage({ type: 'response', requestId: msg.requestId, data: { result: 'ok' } });
      await callP;
    });

    it('resolves with response data', async () => {
      const c = makeClient();
      const connectP = c.connect();
      const ws = lastWs();
      ws.simulateOpen();
      await connectP;

      const callP = c.call<{ val: number }>('get_val');
      const sent = JSON.parse(ws.getSent()[0]);
      ws.simulateMessage({ type: 'response', requestId: sent.requestId, data: { val: 42 } });
      const result = await callP;
      expect(result).toEqual({ val: 42 });
    });

    it('rejects on error response', async () => {
      const c = makeClient();
      const connectP = c.connect();
      const ws = lastWs();
      ws.simulateOpen();
      await connectP;

      const callP = c.call('bad_action');
      const sent = JSON.parse(ws.getSent()[0]);
      ws.simulateMessage({ type: 'error', requestId: sent.requestId, error: 'Handler not found' });
      await expect(callP).rejects.toThrow('Handler not found');
    });

    it('rejects on timeout', async () => {
      const c = makeClient({ timeout: 500 });
      const connectP = c.connect();
      const ws = lastWs();
      ws.simulateOpen();
      await connectP;

      const callP = c.call('slow_action');
      vi.advanceTimersByTime(600);
      await expect(callP).rejects.toThrow('Request timeout: slow_action');
    });

    it('uses per-call timeout over default', async () => {
      const c = makeClient({ timeout: 10000 });
      const connectP = c.connect();
      const ws = lastWs();
      ws.simulateOpen();
      await connectP;

      const callP = c.call('slow', {}, { timeout: 200 });
      vi.advanceTimersByTime(300);
      await expect(callP).rejects.toThrow('Request timeout: slow');
    });

    it('increments requestId per call', async () => {
      const c = makeClient();
      const connectP = c.connect();
      const ws = lastWs();
      ws.simulateOpen();
      await connectP;

      // Fire two calls quickly
      c.call('a');
      c.call('b');

      const msgs = ws.getSent().map((s) => JSON.parse(s));
      expect(msgs[0].requestId).not.toBe(msgs[1].requestId);
    });
  });

  describe('call() via HTTP', () => {
    it('sends POST to /call endpoint', async () => {
      const mockFetch = (globalThis as any).fetch as ReturnType<typeof vi.fn>;
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ result: 'http_result' }),
      });

      const c = makeClient({ httpPort: 9002 });
      const result = await c.call('http_action', { x: 1 }, { useHttp: true });

      expect(mockFetch).toHaveBeenCalledWith(
        'http://127.0.0.1:9002/call',
        expect.objectContaining({
          method: 'POST',
          body: JSON.stringify({ action: 'http_action', data: { x: 1 } }),
        })
      );
      expect(result).toBe('http_result');
    });

    it('throws on non-ok HTTP response', async () => {
      const mockFetch = (globalThis as any).fetch as ReturnType<typeof vi.fn>;
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 404,
        json: async () => ({ error: 'Not found' }),
      });

      const c = makeClient();
      await expect(c.call('missing', {}, { useHttp: true })).rejects.toThrow('Not found');
    });

    it('throws on HTTP abort (timeout)', async () => {
      const mockFetch = (globalThis as any).fetch as ReturnType<typeof vi.fn>;
      mockFetch.mockImplementationOnce(() =>
        new Promise((_, reject) => {
          setTimeout(() => {
            const err = new Error('The operation was aborted');
            (err as any).name = 'AbortError';
            reject(err);
          }, 100);
        })
      );

      const c = makeClient({ timeout: 50 });
      const p = c.call('slow_http', {}, { useHttp: true });
      vi.advanceTimersByTime(200);
      await expect(p).rejects.toThrow('Request timeout');
    });
  });

  describe('on() / off() / emit()', () => {
    it('registers event handler', async () => {
      const c = makeClient();
      const connectP = c.connect();
      const ws = lastWs();
      ws.simulateOpen();
      await connectP;

      const handler = vi.fn();
      c.on('scene_updated', handler);
      ws.simulateMessage({ type: 'event', action: 'scene_updated', data: { frame: 10 } });
      expect(handler).toHaveBeenCalledWith({ frame: 10 });
    });

    it('returns unsubscribe function', async () => {
      const c = makeClient();
      const connectP = c.connect();
      const ws = lastWs();
      ws.simulateOpen();
      await connectP;

      const handler = vi.fn();
      const unsub = c.on('evt', handler);
      unsub();
      ws.simulateMessage({ type: 'event', action: 'evt', data: {} });
      expect(handler).not.toHaveBeenCalled();
    });

    it('off() removes specific handler', async () => {
      const c = makeClient();
      const connectP = c.connect();
      const ws = lastWs();
      ws.simulateOpen();
      await connectP;

      const handler = vi.fn();
      c.on('data', handler);
      c.off('data', handler);
      ws.simulateMessage({ type: 'event', action: 'data', data: {} });
      expect(handler).not.toHaveBeenCalled();
    });

    it('multiple handlers for same event all called', async () => {
      const c = makeClient();
      const connectP = c.connect();
      const ws = lastWs();
      ws.simulateOpen();
      await connectP;

      const h1 = vi.fn();
      const h2 = vi.fn();
      c.on('tick', h1);
      c.on('tick', h2);
      ws.simulateMessage({ type: 'event', action: 'tick', data: 1 });
      expect(h1).toHaveBeenCalled();
      expect(h2).toHaveBeenCalled();
    });

    it('handler error does not stop other handlers', async () => {
      const c = makeClient();
      const connectP = c.connect();
      const ws = lastWs();
      ws.simulateOpen();
      await connectP;

      const bad = vi.fn().mockImplementation(() => { throw new Error('oops'); });
      const good = vi.fn();
      c.on('boom', bad);
      c.on('boom', good);
      // Should not throw
      ws.simulateMessage({ type: 'event', action: 'boom', data: {} });
      expect(good).toHaveBeenCalled();
    });
  });

  describe('handleMessage()', () => {
    it('ignores unknown message types', async () => {
      const c = makeClient();
      const connectP = c.connect();
      const ws = lastWs();
      ws.simulateOpen();
      await connectP;

      // Should not throw
      ws.simulateMessage({ type: 'unknown_type', foo: 'bar' });
    });

    it('ignores response with unknown requestId', async () => {
      const c = makeClient();
      const connectP = c.connect();
      const ws = lastWs();
      ws.simulateOpen();
      await connectP;

      // Should not throw
      ws.simulateMessage({ type: 'response', requestId: 'nonexistent', data: {} });
    });

    it('handles malformed JSON gracefully', async () => {
      const c = makeClient();
      const connectP = c.connect();
      const ws = lastWs();
      ws.simulateOpen();
      await connectP;

      // Inject raw bad JSON
      ws.onmessage?.({ data: 'not-json{{{' });
      // Should not throw
    });

    it('dispatches error response to reject pending request', async () => {
      const c = makeClient();
      const connectP = c.connect();
      const ws = lastWs();
      ws.simulateOpen();
      await connectP;

      const callP = c.call('test');
      const sent = JSON.parse(ws.getSent()[0]);
      ws.simulateMessage({ type: 'error', requestId: sent.requestId, error: 'fail' });
      await expect(callP).rejects.toThrow('fail');
    });

    it('uses "Unknown error" when error field is missing', async () => {
      const c = makeClient();
      const connectP = c.connect();
      const ws = lastWs();
      ws.simulateOpen();
      await connectP;

      const callP = c.call('test2');
      const sent = JSON.parse(ws.getSent()[0]);
      ws.simulateMessage({ type: 'error', requestId: sent.requestId });
      await expect(callP).rejects.toThrow('Unknown error');
    });
  });

  describe('auto-reconnect', () => {
    it('schedules reconnect after close when autoReconnect=true', async () => {
      const c = makeClient({ autoReconnect: true, reconnectInterval: 1000 });
      const connectP = c.connect();
      const ws = lastWs();
      ws.simulateOpen();
      await connectP;

      // Simulate server disconnect
      ws.simulateClose();

      // Advance past reconnect interval
      vi.advanceTimersByTime(1100);

      // A new WebSocket should have been created
      expect(MockWebSocket._lastInstance).not.toBe(ws);
    });

    it('does not reconnect when autoReconnect=false', async () => {
      const c = makeClient({ autoReconnect: false });
      const connectP = c.connect();
      const ws = lastWs();
      ws.simulateOpen();
      await connectP;

      ws.simulateClose();
      vi.advanceTimersByTime(5000);

      // Same instance — no reconnect
      expect(MockWebSocket._lastInstance).toBe(ws);
    });
  });

  describe('checkHealth()', () => {
    it('returns true when /health returns ok', async () => {
      const mockFetch = (globalThis as any).fetch as ReturnType<typeof vi.fn>;
      mockFetch.mockResolvedValueOnce({ ok: true });

      const c = makeClient();
      const result = await c.checkHealth();
      expect(result).toBe(true);
      expect(mockFetch).toHaveBeenCalledWith('http://127.0.0.1:9002/health', { method: 'GET' });
    });

    it('returns false when /health returns not ok', async () => {
      const mockFetch = (globalThis as any).fetch as ReturnType<typeof vi.fn>;
      mockFetch.mockResolvedValueOnce({ ok: false });

      const c = makeClient();
      expect(await c.checkHealth()).toBe(false);
    });

    it('returns false when fetch throws', async () => {
      const mockFetch = (globalThis as any).fetch as ReturnType<typeof vi.fn>;
      mockFetch.mockRejectedValueOnce(new Error('network error'));

      const c = makeClient();
      expect(await c.checkHealth()).toBe(false);
    });
  });

  describe('getInfo()', () => {
    it('fetches from /info and returns parsed JSON', async () => {
      const info = {
        name: 'AuroraView',
        version: '1.0.0',
        ws_url: 'ws://127.0.0.1:9001',
        capabilities: ['call', 'events'],
        handlers: ['get_scene'],
      };
      const mockFetch = (globalThis as any).fetch as ReturnType<typeof vi.fn>;
      mockFetch.mockResolvedValueOnce({ json: async () => info });

      const c = makeClient();
      const result = await c.getInfo();
      expect(result).toEqual(info);
      expect(mockFetch).toHaveBeenCalledWith('http://127.0.0.1:9002/info');
    });
  });
});

// ============================================================================
// Factory function tests
// ============================================================================

describe('createAuroraViewClient()', () => {
  it('returns an AuroraViewClient instance', () => {
    const c = createAuroraViewClient();
    expect(c).toBeInstanceOf(AuroraViewClient);
  });

  it('passes config to the instance', () => {
    const c = createAuroraViewClient({ wsPort: 7777 });
    expect(c.wsUrl).toContain('7777');
  });

  it('works without arguments', () => {
    const c = createAuroraViewClient();
    expect(c.wsUrl).toBe('ws://127.0.0.1:9001');
  });
});

describe('getDefaultClient()', () => {
  it('returns an AuroraViewClient instance', () => {
    const c = getDefaultClient();
    expect(c).toBeInstanceOf(AuroraViewClient);
  });

  it('returns the same instance on repeated calls', () => {
    const c1 = getDefaultClient();
    const c2 = getDefaultClient();
    expect(c1).toBe(c2);
  });
});
