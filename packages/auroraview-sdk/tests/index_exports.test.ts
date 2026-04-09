/**
 * AuroraView SDK index.ts export integrity tests
 *
 * Verifies that all public symbols are exported correctly from the
 * package entry point and that types/values are the expected shapes.
 * Does NOT test implementation logic (that's covered by dedicated files).
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { _resetClientInstance } from '../src/core/bridge';

// ============================================================================
// Mock helpers
// ============================================================================

type AnyRecord = Record<string, any>;

function installBridge(overrides: AnyRecord = {}): void {
  (globalThis as AnyRecord).window = {
    ...((globalThis as AnyRecord).window ?? {}),
    addEventListener: vi.fn(),
    auroraview: {
      _ready: true,
      call: vi.fn().mockResolvedValue({}),
      invoke: vi.fn().mockResolvedValue({}),
      send_event: vi.fn(),
      on: vi.fn().mockReturnValue(() => {}),
      trigger: vi.fn(),
      whenReady: vi.fn().mockResolvedValue(undefined),
      isReady: () => true,
      ...overrides,
    },
  };
}

function removeBridge(): void {
  (globalThis as AnyRecord).window = {
    auroraview: undefined,
    addEventListener: vi.fn(),
  };
}

beforeEach(() => {
  _resetClientInstance();
  installBridge();
});

afterEach(() => {
  _resetClientInstance();
  removeBridge();
  vi.clearAllMocks();
});

// ============================================================================
// Core bridge exports
// ============================================================================

describe('SDK index – core bridge exports', () => {
  it('createAuroraView is exported as a function', async () => {
    const { createAuroraView } = await import('../src/index');
    expect(typeof createAuroraView).toBe('function');
  });

  it('getAuroraView is exported as a function', async () => {
    const { getAuroraView } = await import('../src/index');
    expect(typeof getAuroraView).toBe('function');
  });

  it('createAuroraView returns client with call/on/off/emit', async () => {
    const { createAuroraView } = await import('../src/index');
    const client = createAuroraView();
    expect(typeof client.call).toBe('function');
    expect(typeof client.on).toBe('function');
    expect(typeof client.off).toBe('function');
    expect(typeof client.emit).toBe('function');
  });
});

// ============================================================================
// EventEmitter exports
// ============================================================================

describe('SDK index – EventEmitter exports', () => {
  it('EventEmitter is exported as a class', async () => {
    const { EventEmitter } = await import('../src/index');
    expect(typeof EventEmitter).toBe('function');
  });

  it('getGlobalEmitter is exported as a function', async () => {
    const { getGlobalEmitter } = await import('../src/index');
    expect(typeof getGlobalEmitter).toBe('function');
  });

  it('getGlobalEmitter returns EventEmitter instance', async () => {
    const { EventEmitter, getGlobalEmitter } = await import('../src/index');
    const em = getGlobalEmitter();
    expect(em).toBeInstanceOf(EventEmitter);
  });
});

// ============================================================================
// Feature detection exports
// ============================================================================

describe('SDK index – feature detection exports', () => {
  it('hasFeature is exported as a function', async () => {
    const { hasFeature } = await import('../src/index');
    expect(typeof hasFeature).toBe('function');
  });

  it('hasFeatures is exported as a function', async () => {
    const { hasFeatures } = await import('../src/index');
    expect(typeof hasFeatures).toBe('function');
  });

  it('getAvailableFeatures is exported as a function', async () => {
    const { getAvailableFeatures } = await import('../src/index');
    expect(typeof getAvailableFeatures).toBe('function');
  });

  it('getEnvironment is exported as a function', async () => {
    const { getEnvironment } = await import('../src/index');
    expect(typeof getEnvironment).toBe('function');
  });

  it('isAuroraView is exported as a function', async () => {
    const { isAuroraView } = await import('../src/index');
    expect(typeof isAuroraView).toBe('function');
  });

  it('isDCC is exported as a function', async () => {
    const { isDCC } = await import('../src/index');
    expect(typeof isDCC).toBe('function');
  });

  it('isStandalone is exported as a function', async () => {
    const { isStandalone } = await import('../src/index');
    expect(typeof isStandalone).toBe('function');
  });

  it('isPacked is exported as a function', async () => {
    const { isPacked } = await import('../src/index');
    expect(typeof isPacked).toBe('function');
  });

  it('waitForFeature is exported as a function', async () => {
    const { waitForFeature } = await import('../src/index');
    expect(typeof waitForFeature).toBe('function');
  });

  it('Features constant is exported as an object', async () => {
    const { Features } = await import('../src/index');
    expect(typeof Features).toBe('object');
    expect(Features).not.toBeNull();
  });
});

// ============================================================================
// Window management exports
// ============================================================================

describe('SDK index – window management exports', () => {
  it('Window is exported as a class', async () => {
    const { Window } = await import('../src/index');
    expect(typeof Window).toBe('function');
  });

  it('getCurrentWindow is exported as a function', async () => {
    const { getCurrentWindow } = await import('../src/index');
    expect(typeof getCurrentWindow).toBe('function');
  });

  it('createWindow is exported as a function', async () => {
    const { createWindow } = await import('../src/index');
    expect(typeof createWindow).toBe('function');
  });

  it('closeCurrentWindow is exported as a function', async () => {
    const { closeCurrentWindow } = await import('../src/index');
    expect(typeof closeCurrentWindow).toBe('function');
  });

  it('startDrag is exported as a function', async () => {
    const { startDrag } = await import('../src/index');
    expect(typeof startDrag).toBe('function');
  });

  it('getCurrentWindow returns a Window instance', async () => {
    const { Window, getCurrentWindow } = await import('../src/index');
    const win = getCurrentWindow();
    expect(win).toBeInstanceOf(Window);
  });

  it('Window.getCurrent returns window with label "main" by default', async () => {
    const { Window } = await import('../src/index');
    expect(Window.getCurrent().label).toBe('main');
  });
});

// ============================================================================
// Plugin exports
// ============================================================================

describe('SDK index – plugin exports', () => {
  it('interactive is exported as an object', async () => {
    const { interactive } = await import('../src/index');
    expect(interactive).toBeDefined();
    expect(typeof interactive).toBe('object');
  });
});

// ============================================================================
// Integration: client event flow through index
// ============================================================================

describe('SDK index – integration event flow', () => {
  it('on/off via getAuroraView works correctly', async () => {
    const { getAuroraView } = await import('../src/index');
    const client = getAuroraView();
    const handler = vi.fn();
    const unsub = client.on('test.event', handler);
    expect(typeof unsub).toBe('function');
    unsub();
  });

  it('emit does not throw when bridge available', async () => {
    const { getAuroraView } = await import('../src/index');
    const client = getAuroraView();
    expect(() => client.emit('my.event', { x: 1 })).not.toThrow();
  });

  it('isReady returns boolean', async () => {
    const { getAuroraView } = await import('../src/index');
    const ready = getAuroraView().isReady();
    expect(typeof ready).toBe('boolean');
  });

  it('getRawBridge returns bridge object when available', async () => {
    const { getAuroraView } = await import('../src/index');
    const bridge = getAuroraView().getRawBridge();
    expect(bridge).toBeDefined();
  });
});
