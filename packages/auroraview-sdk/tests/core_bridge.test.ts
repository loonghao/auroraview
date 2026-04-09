/**
 * AuroraView SDK core/bridge.ts Tests
 *
 * Tests the AuroraViewClientImpl (via createAuroraView/getAuroraView) and
 * the _resetClientInstance helper:
 * - call / invoke / emit
 * - on / once / off event forwarding
 * - isReady / whenReady
 * - getRawBridge / setConfig
 * - fs/dialog/clipboard/shell/state accessors
 * - installTriggerIntercept (bridge wrapping, double-wrap prevention)
 * - singleton semantics + reset
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import {
  createAuroraView,
  getAuroraView,
  _resetClientInstance,
} from '../src/core/bridge';

// ============================================================================
// Mock helpers
// ============================================================================

type AnyRecord = Record<string, any>;

let mockBridgeCall: ReturnType<typeof vi.fn>;
let mockBridgeInvoke: ReturnType<typeof vi.fn>;
let mockBridgeSendEvent: ReturnType<typeof vi.fn>;
let mockBridgeOn: ReturnType<typeof vi.fn>;
let mockBridgeWhenReady: ReturnType<typeof vi.fn>;
let mockBridgeTrigger: ReturnType<typeof vi.fn>;
let mockBridgeSetConfig: ReturnType<typeof vi.fn>;

function makeMockBridge(overrides: AnyRecord = {}): AnyRecord {
  mockBridgeCall = vi.fn().mockResolvedValue({ result: 'ok' });
  mockBridgeInvoke = vi.fn().mockResolvedValue({ success: true });
  mockBridgeSendEvent = vi.fn();
  mockBridgeOn = vi.fn().mockReturnValue(() => {});
  mockBridgeWhenReady = vi.fn().mockResolvedValue(undefined);
  mockBridgeTrigger = vi.fn();
  mockBridgeSetConfig = vi.fn();

  return {
    _ready: true,
    _isStub: false,
    _pendingCalls: [],
    _boundMethods: {},
    call: mockBridgeCall,
    invoke: mockBridgeInvoke,
    send_event: mockBridgeSendEvent,
    on: mockBridgeOn,
    trigger: mockBridgeTrigger,
    whenReady: mockBridgeWhenReady,
    isReady: () => true,
    setConfig: mockBridgeSetConfig,
    fs: { readFile: vi.fn() },
    dialog: { openFile: vi.fn() },
    clipboard: { readText: vi.fn() },
    shell: { open: vi.fn() },
    state: { onChange: vi.fn() },
    ...overrides,
  };
}

function installBridge(bridge: AnyRecord | undefined): void {
  (globalThis as AnyRecord).window = {
    ...((globalThis as AnyRecord).window ?? {}),
    auroraview: bridge,
    addEventListener: vi.fn(),
  };
}

function removeBridge(): void {
  (globalThis as AnyRecord).window = {
    ...((globalThis as AnyRecord).window ?? {}),
    auroraview: undefined,
    addEventListener: vi.fn(),
  };
}

// ============================================================================
// Setup / teardown
// ============================================================================

beforeEach(() => {
  _resetClientInstance();
  vi.useFakeTimers();
});

afterEach(() => {
  _resetClientInstance();
  removeBridge();
  vi.useRealTimers();
  vi.clearAllMocks();
});

// ============================================================================
// Singleton semantics
// ============================================================================

describe('Singleton semantics', () => {
  it('createAuroraView returns same instance', () => {
    installBridge(makeMockBridge());
    const c1 = createAuroraView();
    const c2 = createAuroraView();
    expect(c1).toBe(c2);
  });

  it('getAuroraView returns same instance as createAuroraView', () => {
    installBridge(makeMockBridge());
    const c1 = createAuroraView();
    const c2 = getAuroraView();
    expect(c1).toBe(c2);
  });

  it('_resetClientInstance creates a fresh instance', () => {
    installBridge(makeMockBridge());
    const c1 = createAuroraView();
    _resetClientInstance();
    installBridge(makeMockBridge());
    const c2 = createAuroraView();
    expect(c1).not.toBe(c2);
  });
});

// ============================================================================
// call()
// ============================================================================

describe('AuroraViewClient.call', () => {
  it('delegates to bridge.call with method and params', async () => {
    installBridge(makeMockBridge());
    const client = createAuroraView();
    await client.call('my.method', { x: 1 });
    expect(mockBridgeCall).toHaveBeenCalledWith('my.method', { x: 1 });
  });

  it('rejects when bridge is unavailable', async () => {
    removeBridge();
    const client = createAuroraView();
    await expect(client.call('m')).rejects.toThrow('AuroraView bridge not available');
  });

  it('returns resolved value from bridge', async () => {
    mockBridgeCall = vi.fn().mockResolvedValue('hello');
    installBridge(makeMockBridge({ call: mockBridgeCall }));
    const client = createAuroraView();
    const result = await client.call<string>('greet');
    expect(result).toBe('hello');
  });

  it('call with no params passes undefined', async () => {
    installBridge(makeMockBridge());
    const client = createAuroraView();
    await client.call('no.params');
    expect(mockBridgeCall).toHaveBeenCalledWith('no.params', undefined);
  });
});

// ============================================================================
// invoke()
// ============================================================================

describe('AuroraViewClient.invoke', () => {
  it('delegates to bridge.invoke with cmd and args', async () => {
    installBridge(makeMockBridge());
    const client = createAuroraView();
    await client.invoke('plugin.cmd', { key: 'val' });
    expect(mockBridgeInvoke).toHaveBeenCalledWith('plugin.cmd', { key: 'val' });
  });

  it('rejects when bridge is unavailable', async () => {
    removeBridge();
    const client = createAuroraView();
    await expect(client.invoke('cmd')).rejects.toThrow();
  });
});

// ============================================================================
// emit()
// ============================================================================

describe('AuroraViewClient.emit', () => {
  it('calls bridge.send_event with event and data', () => {
    installBridge(makeMockBridge());
    const client = createAuroraView();
    client.emit('my.event', { payload: 42 });
    expect(mockBridgeSendEvent).toHaveBeenCalledWith('my.event', { payload: 42 });
  });

  it('does nothing when bridge is unavailable', () => {
    removeBridge();
    const client = createAuroraView();
    expect(() => client.emit('ev', null)).not.toThrow();
  });

  it('emit with no data passes undefined', () => {
    installBridge(makeMockBridge());
    const client = createAuroraView();
    client.emit('ev');
    expect(mockBridgeSendEvent).toHaveBeenCalledWith('ev', undefined);
  });
});

// ============================================================================
// on / once / off event forwarding
// ============================================================================

describe('AuroraViewClient.on / once / off', () => {
  it('on subscribes via internal emitter and returns unsubscribe', () => {
    installBridge(makeMockBridge());
    const client = createAuroraView();
    const handler = vi.fn();
    const unsub = client.on('ev', handler);
    expect(typeof unsub).toBe('function');
  });

  it('once subscribes once via internal emitter', () => {
    installBridge(makeMockBridge());
    const client = createAuroraView();
    const handler = vi.fn();
    const unsub = client.once('ev', handler);
    expect(typeof unsub).toBe('function');
  });

  it('off removes event handler', () => {
    installBridge(makeMockBridge());
    const client = createAuroraView();
    const handler = vi.fn();
    client.on('ev', handler);
    client.off('ev', handler);
    // Emit via trigger intercept
    const bridge = (globalThis as AnyRecord).window.auroraview;
    bridge.trigger?.('ev', 'data');
    expect(handler).not.toHaveBeenCalled();
  });

  it('off without handler removes all handlers', () => {
    installBridge(makeMockBridge());
    const client = createAuroraView();
    const h1 = vi.fn();
    const h2 = vi.fn();
    client.on('ev', h1);
    client.on('ev', h2);
    client.off('ev');
    const bridge = (globalThis as AnyRecord).window.auroraview;
    bridge.trigger?.('ev', 'data');
    expect(h1).not.toHaveBeenCalled();
    expect(h2).not.toHaveBeenCalled();
  });
});

// ============================================================================
// isReady
// ============================================================================

describe('AuroraViewClient.isReady', () => {
  it('returns true when bridge._ready is true', () => {
    installBridge(makeMockBridge({ _ready: true }));
    const client = createAuroraView();
    expect(client.isReady()).toBe(true);
  });

  it('returns false when bridge is absent', () => {
    removeBridge();
    const client = createAuroraView();
    expect(client.isReady()).toBe(false);
  });

  it('returns false when bridge._ready is false', () => {
    installBridge(makeMockBridge({ _ready: false }));
    const client = createAuroraView();
    expect(client.isReady()).toBe(false);
  });
});

// ============================================================================
// whenReady
// ============================================================================

describe('AuroraViewClient.whenReady', () => {
  it('resolves immediately when bridge is ready', async () => {
    installBridge(makeMockBridge({ _ready: true, whenReady: vi.fn().mockResolvedValue(undefined) }));
    const client = createAuroraView();
    const resolved = await client.whenReady();
    expect(resolved).toBe(client);
  });

  it('resolves even when bridge not present (timeout path)', async () => {
    removeBridge();
    const client = createAuroraView();
    const promise = client.whenReady();
    // Fast-forward timers to trigger timeout resolve
    vi.advanceTimersByTime(30001);
    const resolved = await promise;
    expect(resolved).toBe(client);
  });
});

// ============================================================================
// setConfig
// ============================================================================

describe('AuroraViewClient.setConfig', () => {
  it('delegates to bridge.setConfig', () => {
    installBridge(makeMockBridge());
    const client = createAuroraView();
    client.setConfig({ callTimeoutMs: 5000 });
    expect(mockBridgeSetConfig).toHaveBeenCalledWith({ callTimeoutMs: 5000 });
  });

  it('does not throw when bridge is absent', () => {
    removeBridge();
    const client = createAuroraView();
    expect(() => client.setConfig({ callTimeoutMs: 1000 })).not.toThrow();
  });

  it('does not throw when bridge has no setConfig', () => {
    const bridge = makeMockBridge();
    delete bridge.setConfig;
    installBridge(bridge);
    const client = createAuroraView();
    expect(() => client.setConfig({ callTimeoutMs: 1000 })).not.toThrow();
  });
});

// ============================================================================
// getRawBridge
// ============================================================================

describe('AuroraViewClient.getRawBridge', () => {
  it('returns the window.auroraview object', () => {
    const bridge = makeMockBridge();
    installBridge(bridge);
    const client = createAuroraView();
    expect(client.getRawBridge()).toBe(bridge);
  });

  it('returns undefined when bridge is absent', () => {
    removeBridge();
    const client = createAuroraView();
    expect(client.getRawBridge()).toBeUndefined();
  });
});

// ============================================================================
// Plugin API accessors
// ============================================================================

describe('AuroraViewClient plugin API accessors', () => {
  it('fs returns bridge.fs', () => {
    const bridge = makeMockBridge();
    installBridge(bridge);
    const client = createAuroraView();
    expect(client.fs).toBe(bridge.fs);
  });

  it('dialog returns bridge.dialog', () => {
    const bridge = makeMockBridge();
    installBridge(bridge);
    const client = createAuroraView();
    expect(client.dialog).toBe(bridge.dialog);
  });

  it('clipboard returns bridge.clipboard', () => {
    const bridge = makeMockBridge();
    installBridge(bridge);
    const client = createAuroraView();
    expect(client.clipboard).toBe(bridge.clipboard);
  });

  it('shell returns bridge.shell', () => {
    const bridge = makeMockBridge();
    installBridge(bridge);
    const client = createAuroraView();
    expect(client.shell).toBe(bridge.shell);
  });

  it('state returns bridge.state', () => {
    const bridge = makeMockBridge();
    installBridge(bridge);
    const client = createAuroraView();
    expect(client.state).toBe(bridge.state);
  });

  it('all accessors return undefined when bridge absent', () => {
    removeBridge();
    const client = createAuroraView();
    expect(client.fs).toBeUndefined();
    expect(client.dialog).toBeUndefined();
    expect(client.clipboard).toBeUndefined();
    expect(client.shell).toBeUndefined();
    expect(client.state).toBeUndefined();
  });
});

// ============================================================================
// Trigger intercept
// ============================================================================

describe('Trigger intercept', () => {
  it('wraps bridge.trigger to forward events to client.on subscribers', () => {
    const bridge = makeMockBridge();
    installBridge(bridge);
    const client = createAuroraView();
    const handler = vi.fn();
    client.on('backend.event', handler);
    // Simulate backend push via wrapped trigger
    bridge.trigger('backend.event', { msg: 'hello' });
    expect(handler).toHaveBeenCalledWith({ msg: 'hello' });
  });

  it('does not double-wrap trigger on second client creation', () => {
    const bridge = makeMockBridge();
    installBridge(bridge);
    createAuroraView();
    const firstWrapped = bridge.trigger;
    _resetClientInstance();
    // Re-install same bridge
    (globalThis as AnyRecord).window.auroraview = bridge;
    createAuroraView();
    // Should still be the same wrapped function (not wrapped again)
    expect(bridge.trigger).toBe(firstWrapped);
  });
});
