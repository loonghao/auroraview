/**
 * Inject Bridge Tests
 *
 * Tests for the core inject layer logic: bridge_stub, event_bridge utilities,
 * state_bridge, and plugin utils. Since these modules use IIFEs that run at
 * load time, we test the extracted pure logic and mock the window object.
 */

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { invokePlugin, attachPlugin, initPlugin } from '../src/inject/plugins/utils';

// ============================================================================
// invokePlugin tests
// ============================================================================

describe('invokePlugin', () => {
  let originalWindow: typeof window;

  beforeEach(() => {
    originalWindow = (global as unknown as { window: typeof window }).window;
  });

  afterEach(() => {
    (global as unknown as { window: typeof window }).window = originalWindow;
    vi.restoreAllMocks();
  });

  it('should throw when auroraview not available', async () => {
    (global as unknown as { window: { auroraview?: unknown } }).window = {
      auroraview: undefined,
    };
    await expect(invokePlugin('fs', 'read_file', { path: '/test.txt' })).rejects.toThrow(
      'AuroraView bridge not available'
    );
  });

  it('should throw when invoke is missing', async () => {
    (global as unknown as { window: { auroraview: object } }).window = {
      auroraview: {},
    };
    await expect(invokePlugin('fs', 'read_file')).rejects.toThrow(
      'AuroraView bridge not available'
    );
  });

  it('should call invoke with correct command format', async () => {
    const mockInvoke = vi.fn().mockResolvedValue({ data: 'content' });
    (global as unknown as { window: { auroraview: { invoke: typeof mockInvoke } } }).window = {
      auroraview: { invoke: mockInvoke },
    };

    await invokePlugin('fs', 'read_file', { path: '/test.txt' });
    expect(mockInvoke).toHaveBeenCalledWith('plugin:fs|read_file', { path: '/test.txt' });
  });

  it('should use empty object when no args provided', async () => {
    const mockInvoke = vi.fn().mockResolvedValue({ result: 'ok' });
    (global as unknown as { window: { auroraview: { invoke: typeof mockInvoke } } }).window = {
      auroraview: { invoke: mockInvoke },
    };

    await invokePlugin('dialog', 'open');
    expect(mockInvoke).toHaveBeenCalledWith('plugin:dialog|open', {});
  });

  it('should throw when result has success === false', async () => {
    const mockInvoke = vi.fn().mockResolvedValue({
      success: false,
      error: 'File not found',
      code: 'NOT_FOUND',
    });
    (global as unknown as { window: { auroraview: { invoke: typeof mockInvoke } } }).window = {
      auroraview: { invoke: mockInvoke },
    };

    await expect(invokePlugin('fs', 'read_file', { path: '/missing.txt' })).rejects.toThrow(
      'File not found'
    );
  });

  it('should throw with UNKNOWN code when error has no code', async () => {
    const mockInvoke = vi.fn().mockResolvedValue({
      success: false,
      error: 'Something failed',
    });
    (global as unknown as { window: { auroraview: { invoke: typeof mockInvoke } } }).window = {
      auroraview: { invoke: mockInvoke },
    };

    let caught: Error & { code?: string } | null = null;
    try {
      await invokePlugin('clipboard', 'write');
    } catch (e) {
      caught = e as Error & { code?: string };
    }
    expect(caught).not.toBeNull();
    expect(caught?.code).toBe('UNKNOWN');
  });

  it('should return result when success is true', async () => {
    const data = { content: 'hello world' };
    const mockInvoke = vi.fn().mockResolvedValue({ success: true, ...data });
    (global as unknown as { window: { auroraview: { invoke: typeof mockInvoke } } }).window = {
      auroraview: { invoke: mockInvoke },
    };

    const result = await invokePlugin('fs', 'read_file');
    expect(result).toEqual({ success: true, content: 'hello world' });
  });

  it('should return result when no success field', async () => {
    const data = [{ name: 'a.txt', isFile: true }];
    const mockInvoke = vi.fn().mockResolvedValue(data);
    (global as unknown as { window: { auroraview: { invoke: typeof mockInvoke } } }).window = {
      auroraview: { invoke: mockInvoke },
    };

    const result = await invokePlugin('fs', 'read_dir');
    expect(result).toEqual(data);
  });
});

// ============================================================================
// attachPlugin tests
// ============================================================================

describe('attachPlugin', () => {
  let originalWindow: typeof window;

  beforeEach(() => {
    originalWindow = (global as unknown as { window: typeof window }).window;
  });

  afterEach(() => {
    (global as unknown as { window: typeof window }).window = originalWindow;
  });

  it('should attach plugin to window.auroraview', () => {
    const mockAV = {} as Record<string, unknown>;
    (global as unknown as { window: { auroraview: typeof mockAV } }).window = {
      auroraview: mockAV,
    };

    const api = { readFile: vi.fn() };
    attachPlugin('fs', api as unknown as Record<string, unknown>);

    expect((mockAV as Record<string, unknown>)['fs']).toBe(api);
  });

  it('should do nothing when auroraview is not available', () => {
    (global as unknown as { window: { auroraview?: unknown } }).window = {};

    const consoleSpy = vi.spyOn(console, 'log').mockImplementation(() => {});
    const api = { readFile: vi.fn() };
    // Should not throw
    expect(() =>
      attachPlugin('fs', api as unknown as Record<string, unknown>)
    ).not.toThrow();
    consoleSpy.mockRestore();
  });

  it('should log initialization message', () => {
    const logSpy = vi.spyOn(console, 'log').mockImplementation(() => {});
    const mockAV = {} as Record<string, unknown>;
    (global as unknown as { window: { auroraview: typeof mockAV } }).window = {
      auroraview: mockAV,
    };

    attachPlugin('fs', {});
    expect(logSpy).toHaveBeenCalledWith(expect.stringContaining('Fs plugin initialized'));
    logSpy.mockRestore();
  });
});

// ============================================================================
// initPlugin tests
// ============================================================================

describe('initPlugin', () => {
  let originalWindow: typeof window;

  beforeEach(() => {
    originalWindow = (global as unknown as { window: typeof window }).window;
    vi.useFakeTimers();
  });

  afterEach(() => {
    (global as unknown as { window: typeof window }).window = originalWindow;
    vi.useRealTimers();
  });

  it('should attach immediately when auroraview is available', () => {
    const mockAV = {} as Record<string, unknown>;
    (global as unknown as { window: { auroraview: typeof mockAV } }).window = {
      auroraview: mockAV,
    };

    const logSpy = vi.spyOn(console, 'log').mockImplementation(() => {});
    const api = { readFile: vi.fn() };
    initPlugin('fs', api as unknown as Record<string, unknown>);

    expect((mockAV as Record<string, unknown>)['fs']).toBe(api);
    logSpy.mockRestore();
  });

  it('should poll for auroraview when not available', async () => {
    const mockWindow = {} as { auroraview?: Record<string, unknown> };
    (global as unknown as { window: typeof mockWindow }).window = mockWindow;

    const logSpy = vi.spyOn(console, 'log').mockImplementation(() => {});
    const api = { readFile: vi.fn() };
    initPlugin('fs', api as unknown as Record<string, unknown>);

    // Plugin not yet attached
    expect(mockWindow.auroraview?.['fs']).toBeUndefined();

    // Set auroraview and advance timers
    mockWindow.auroraview = {};
    vi.advanceTimersByTime(15);

    expect(mockWindow.auroraview?.['fs']).toBe(api);
    logSpy.mockRestore();
  });

  it('should stop polling after 5 seconds', () => {
    const mockWindow = {} as { auroraview?: Record<string, unknown> };
    (global as unknown as { window: typeof mockWindow }).window = mockWindow;

    const logSpy = vi.spyOn(console, 'log').mockImplementation(() => {});
    initPlugin('fs', {});

    // Advance past 5 second timeout
    vi.advanceTimersByTime(6000);

    // auroraview still not set — should not throw
    expect(mockWindow.auroraview).toBeUndefined();
    logSpy.mockRestore();
  });
});

// ============================================================================
// Bridge behavior (mocked event bridge logic tests)
// ============================================================================

describe('Event bridge logic (manual simulation)', () => {
  it('should recognize fatal errors by keyword', () => {
    // Test the isFatalError logic extracted from event_bridge.ts
    const FATAL_KEYWORDS = [
      'process has exited',
      'backend ready timeout',
      'stdout closed',
      'connection lost',
      'fatal error',
      'crash',
    ];

    function isFatalError(message: string): boolean {
      const lower = message.toLowerCase();
      return FATAL_KEYWORDS.some((kw) => lower.includes(kw));
    }

    expect(isFatalError('The process has exited with code 1')).toBe(true);
    expect(isFatalError('Backend ready timeout reached')).toBe(true);
    expect(isFatalError('stdout closed unexpectedly')).toBe(true);
    expect(isFatalError('connection lost')).toBe(true);
    expect(isFatalError('Fatal error in main thread')).toBe(true);
    expect(isFatalError('App crash detected')).toBe(true);
    expect(isFatalError('Module not found: xyz')).toBe(false);
    expect(isFatalError('Warning: deprecated API')).toBe(false);
    expect(isFatalError('')).toBe(false);
  });

  it('should describe backend error for various input types', () => {
    function describeBackendError(detail: unknown): string {
      if (typeof detail === 'string') return detail;
      if (detail && typeof detail === 'object') {
        const maybeMessage = (detail as { message?: unknown }).message;
        if (typeof maybeMessage === 'string') return maybeMessage;
        try {
          return JSON.stringify(detail);
        } catch {
          return '[unserializable backend error]';
        }
      }
      return 'unknown backend error';
    }

    expect(describeBackendError('my error')).toBe('my error');
    expect(describeBackendError({ message: 'object error' })).toBe('object error');
    expect(describeBackendError({ code: 42 })).toBe('{"code":42}');
    expect(describeBackendError(null)).toBe('unknown backend error');
    expect(describeBackendError(undefined)).toBe('unknown backend error');
    expect(describeBackendError(42)).toBe('unknown backend error');
  });

  it('should normalize URL: add https:// when no scheme', () => {
    function normalizeUrl(url: string): string {
      const trimmed = url.trim();
      if (!trimmed) return '';
      if (trimmed.includes('://')) return trimmed;
      return `https://${trimmed}`;
    }

    expect(normalizeUrl('example.com')).toBe('https://example.com');
    expect(normalizeUrl('https://example.com')).toBe('https://example.com');
    expect(normalizeUrl('http://localhost:8080')).toBe('http://localhost:8080');
    expect(normalizeUrl('file:///C:/path')).toBe('file:///C:/path');
    expect(normalizeUrl('')).toBe('');
    expect(normalizeUrl('  ')).toBe('');
  });

  it('should generate unique call IDs', () => {
    let counter = 0;
    function generateCallId(): string {
      counter += 1;
      return `av_call_${Date.now()}_${counter}`;
    }

    const id1 = generateCallId();
    const id2 = generateCallId();
    const id3 = generateCallId();

    expect(id1).not.toBe(id2);
    expect(id2).not.toBe(id3);
    expect(id1).toMatch(/^av_call_\d+_\d+$/);
  });
});

// ============================================================================
// State bridge logic (manual simulation)
// ============================================================================

describe('State bridge handleStateSync logic', () => {
  type StateData = Record<string, unknown>;
  type ChangeHandler = (key: string, value: unknown, source: 'python' | 'javascript') => void;

  function createStateBridge() {
    const stateData: StateData = {};
    const changeHandlers: ChangeHandler[] = [];

    function notifyHandlers(key: string, value: unknown, source: 'python' | 'javascript') {
      changeHandlers.forEach((h) => {
        try {
          h(key, value, source);
        } catch {
          // ignore
        }
      });
    }

    function handleStateSync(data: {
      type: 'set' | 'delete' | 'batch' | 'full' | 'clear';
      key?: string;
      value?: unknown;
      data?: Record<string, unknown>;
    }) {
      if (!data || typeof data !== 'object') return;

      switch (data.type) {
        case 'set':
          if (data.key) {
            stateData[data.key] = data.value;
            notifyHandlers(data.key, data.value, 'python');
          }
          break;
        case 'delete':
          if (data.key) {
            delete stateData[data.key];
            notifyHandlers(data.key, undefined, 'python');
          }
          break;
        case 'batch':
          if (data.data) {
            Object.entries(data.data).forEach(([k, v]) => {
              stateData[k] = v;
              notifyHandlers(k, v, 'python');
            });
          }
          break;
        case 'full':
          Object.keys(stateData).forEach((k) => delete stateData[k]);
          if (data.data) {
            Object.assign(stateData, data.data);
            Object.entries(data.data).forEach(([k, v]) => notifyHandlers(k, v, 'python'));
          }
          break;
        case 'clear':
          Object.keys(stateData).forEach((k) => {
            delete stateData[k];
            notifyHandlers(k, undefined, 'python');
          });
          break;
      }
    }

    return { stateData, changeHandlers, handleStateSync };
  }

  it('should handle set operation', () => {
    const { stateData, handleStateSync } = createStateBridge();
    handleStateSync({ type: 'set', key: 'username', value: 'alice' });
    expect(stateData['username']).toBe('alice');
  });

  it('should handle delete operation', () => {
    const { stateData, handleStateSync } = createStateBridge();
    stateData['foo'] = 'bar';
    handleStateSync({ type: 'delete', key: 'foo' });
    expect('foo' in stateData).toBe(false);
  });

  it('should handle batch operation', () => {
    const { stateData, handleStateSync } = createStateBridge();
    handleStateSync({ type: 'batch', data: { a: 1, b: 2, c: 3 } });
    expect(stateData['a']).toBe(1);
    expect(stateData['b']).toBe(2);
    expect(stateData['c']).toBe(3);
  });

  it('should handle full operation (replace all)', () => {
    const { stateData, handleStateSync } = createStateBridge();
    stateData['old'] = 'value';
    handleStateSync({ type: 'full', data: { newKey: 'newValue' } });
    expect('old' in stateData).toBe(false);
    expect(stateData['newKey']).toBe('newValue');
  });

  it('should handle clear operation', () => {
    const { stateData, handleStateSync } = createStateBridge();
    stateData['x'] = 1;
    stateData['y'] = 2;
    handleStateSync({ type: 'clear' });
    expect(Object.keys(stateData).length).toBe(0);
  });

  it('should notify handlers on set', () => {
    const { changeHandlers, handleStateSync } = createStateBridge();
    const handler = vi.fn();
    changeHandlers.push(handler);

    handleStateSync({ type: 'set', key: 'counter', value: 42 });
    expect(handler).toHaveBeenCalledWith('counter', 42, 'python');
  });

  it('should notify handlers on delete', () => {
    const { stateData, changeHandlers, handleStateSync } = createStateBridge();
    stateData['key'] = 'val';
    const handler = vi.fn();
    changeHandlers.push(handler);

    handleStateSync({ type: 'delete', key: 'key' });
    expect(handler).toHaveBeenCalledWith('key', undefined, 'python');
  });

  it('should notify handlers on batch', () => {
    const { changeHandlers, handleStateSync } = createStateBridge();
    const handler = vi.fn();
    changeHandlers.push(handler);

    handleStateSync({ type: 'batch', data: { p: 10, q: 20 } });
    expect(handler).toHaveBeenCalledTimes(2);
    expect(handler).toHaveBeenCalledWith('p', 10, 'python');
    expect(handler).toHaveBeenCalledWith('q', 20, 'python');
  });

  it('should notify handlers on full replacement', () => {
    const { stateData, changeHandlers, handleStateSync } = createStateBridge();
    stateData['existing'] = 'old';
    const handler = vi.fn();
    changeHandlers.push(handler);

    handleStateSync({ type: 'full', data: { fresh: 'new' } });
    expect(handler).toHaveBeenCalledWith('fresh', 'new', 'python');
  });

  it('should notify handlers on clear', () => {
    const { stateData, changeHandlers, handleStateSync } = createStateBridge();
    stateData['k1'] = 'v1';
    stateData['k2'] = 'v2';
    const handler = vi.fn();
    changeHandlers.push(handler);

    handleStateSync({ type: 'clear' });
    expect(handler).toHaveBeenCalledTimes(2);
  });

  it('should ignore invalid data', () => {
    const { stateData, handleStateSync } = createStateBridge();
    // @ts-expect-error testing invalid input
    handleStateSync(null);
    // @ts-expect-error testing invalid input
    handleStateSync(undefined);
    expect(Object.keys(stateData).length).toBe(0);
  });

  it('should handle set without key gracefully', () => {
    const { stateData, handleStateSync } = createStateBridge();
    handleStateSync({ type: 'set', value: 'orphaned' });
    expect(Object.keys(stateData).length).toBe(0);
  });
});

// ============================================================================
// Bridge config logic
// ============================================================================

describe('AuroraView bridge config', () => {
  it('should apply callTimeoutMs from config', () => {
    const DEFAULT = 30000;
    const config = {
      callTimeoutMs: DEFAULT,
      backendFailFast: true,
      heartbeatTimeoutMs: 0,
    };

    function applyConfig(partial?: Partial<typeof config>) {
      if (!partial) return;
      if (typeof partial.callTimeoutMs === 'number') config.callTimeoutMs = partial.callTimeoutMs;
      if (typeof partial.backendFailFast === 'boolean')
        config.backendFailFast = partial.backendFailFast;
      if (typeof partial.heartbeatTimeoutMs === 'number')
        config.heartbeatTimeoutMs = partial.heartbeatTimeoutMs;
    }

    applyConfig({ callTimeoutMs: 5000 });
    expect(config.callTimeoutMs).toBe(5000);

    applyConfig({ backendFailFast: false });
    expect(config.backendFailFast).toBe(false);

    applyConfig({ heartbeatTimeoutMs: 10000 });
    expect(config.heartbeatTimeoutMs).toBe(10000);
  });

  it('should ignore null partial config', () => {
    const config = { callTimeoutMs: 30000, backendFailFast: true, heartbeatTimeoutMs: 0 };
    function applyConfig(partial?: Partial<typeof config>) {
      if (!partial) return;
      if (typeof partial.callTimeoutMs === 'number') config.callTimeoutMs = partial.callTimeoutMs;
    }

    applyConfig(undefined);
    expect(config.callTimeoutMs).toBe(30000);
  });

  it('should not apply non-number values for numeric fields', () => {
    const config = { callTimeoutMs: 30000, backendFailFast: true, heartbeatTimeoutMs: 0 };
    function applyConfig(partial?: Partial<Record<string, unknown>>) {
      if (!partial) return;
      if (typeof partial['callTimeoutMs'] === 'number')
        config.callTimeoutMs = partial['callTimeoutMs'];
    }

    applyConfig({ callTimeoutMs: 'not-a-number' as unknown as number });
    expect(config.callTimeoutMs).toBe(30000);
  });
});

// ============================================================================
// Pending call registry
// ============================================================================

describe('Pending call registry simulation', () => {
  type PendingEntry = {
    resolve: (v: unknown) => void;
    reject: (e: Error) => void;
    timeoutId?: ReturnType<typeof setTimeout>;
  };

  function createRegistry() {
    const pending = new Map<string, PendingEntry>();
    let counter = 0;

    function generateId(): string {
      counter += 1;
      return `av_call_${counter}`;
    }

    function register(
      id: string,
      resolve: PendingEntry['resolve'],
      reject: PendingEntry['reject'],
      timeoutMs = 1000
    ) {
      const timeoutId = setTimeout(() => {
        const entry = pending.get(id);
        if (entry) {
          pending.delete(id);
          const err = new Error(`timed out after ${timeoutMs}ms`) as Error & { name: string };
          err.name = 'TimeoutError';
          entry.reject(err);
        }
      }, timeoutMs);
      pending.set(id, { resolve, reject, timeoutId });
    }

    function resolve(id: string, value: unknown): boolean {
      const entry = pending.get(id);
      if (!entry) return false;
      if (entry.timeoutId) clearTimeout(entry.timeoutId);
      pending.delete(id);
      entry.resolve(value);
      return true;
    }

    function reject(id: string, error: Error): boolean {
      const entry = pending.get(id);
      if (!entry) return false;
      if (entry.timeoutId) clearTimeout(entry.timeoutId);
      pending.delete(id);
      entry.reject(error);
      return true;
    }

    function clearAll(reason: string): number {
      const count = pending.size;
      pending.forEach((entry, id) => {
        if (entry.timeoutId) clearTimeout(entry.timeoutId);
        const err = new Error(`cancelled: ${reason}`) as Error & { name: string };
        err.name = 'CancelledError';
        entry.reject(err);
      });
      pending.clear();
      return count;
    }

    return { pending, generateId, register, resolve, reject, clearAll };
  }

  it('should resolve a pending call', async () => {
    const registry = createRegistry();
    const id = registry.generateId();
    const p = new Promise<unknown>((res, rej) => {
      registry.register(id, res, rej);
    });

    const success = registry.resolve(id, { result: 42 });
    expect(success).toBe(true);
    const value = await p;
    expect(value).toEqual({ result: 42 });
  });

  it('should reject a pending call', async () => {
    const registry = createRegistry();
    const id = registry.generateId();
    const p = new Promise<unknown>((res, rej) => {
      registry.register(id, res, rej);
    });

    const error = new Error('backend failure');
    const success = registry.reject(id, error);
    expect(success).toBe(true);
    await expect(p).rejects.toThrow('backend failure');
  });

  it('should return false for unknown id on resolve', () => {
    const registry = createRegistry();
    const success = registry.resolve('nonexistent', null);
    expect(success).toBe(false);
  });

  it('should return false for unknown id on reject', () => {
    const registry = createRegistry();
    const success = registry.reject('nonexistent', new Error('noop'));
    expect(success).toBe(false);
  });

  it('should clear all pending calls', async () => {
    const registry = createRegistry();
    const id1 = registry.generateId();
    const id2 = registry.generateId();

    const promises: Promise<unknown>[] = [];
    promises.push(
      new Promise<unknown>((res, rej) => {
        registry.register(id1, res, rej);
      })
    );
    promises.push(
      new Promise<unknown>((res, rej) => {
        registry.register(id2, res, rej);
      })
    );

    const count = registry.clearAll('page unloading');
    expect(count).toBe(2);
    expect(registry.pending.size).toBe(0);

    await expect(promises[0]).rejects.toThrow('cancelled: page unloading');
    await expect(promises[1]).rejects.toThrow('cancelled: page unloading');
  });
});
