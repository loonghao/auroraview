/**
 * AuroraView SDK features.ts Tests
 *
 * Tests all public exports from src/features.ts:
 * - Features constant
 * - hasFeature() / hasFeatures() / getAvailableFeatures()
 * - getEnvironment() / isAuroraView() / isDCC() / isStandalone() / isPacked()
 * - waitForFeature()
 *
 * All globals (window, navigator, window.auroraview) are mocked inline.
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import {
  Features,
  hasFeature,
  hasFeatures,
  getAvailableFeatures,
  getEnvironment,
  isAuroraView,
  isDCC,
  isStandalone,
  isPacked,
  waitForFeature,
} from '../src/features';

// ============================================================================
// Helpers
// ============================================================================

type AnyRecord = Record<string, any>;

/** Install a mock window.auroraview bridge */
function installBridge(overrides: AnyRecord = {}): void {
  (globalThis as AnyRecord).window = {
    ...(globalThis as AnyRecord).window,
    auroraview: {
      isReady: () => true,
      startDrag: undefined,
      isMethodBound: undefined,
      clipboard: undefined,
      shell: undefined,
      fs: undefined,
      dialog: undefined,
      state: undefined,
      invoke: undefined,
      api: undefined,
      getBoundMethods: undefined,
      ...overrides,
    },
  };
}

/** Remove window.auroraview */
function removeBridge(): void {
  const w = (globalThis as AnyRecord).window;
  if (w) {
    delete w.auroraview;
  }
}

/** Install a window with auroraview.isReady returning false */
function installBridgeNotReady(): void {
  (globalThis as AnyRecord).window = {
    ...(globalThis as AnyRecord).window,
    auroraview: {
      isReady: () => false,
    },
  };
}

beforeEach(() => {
  // Ensure window exists in node env
  if (typeof (globalThis as AnyRecord).window === 'undefined') {
    (globalThis as AnyRecord).window = {};
  }
  // Ensure navigator exists
  if (typeof (globalThis as AnyRecord).navigator === 'undefined') {
    (globalThis as AnyRecord).navigator = { userAgent: 'Node', platform: '' };
  }
  vi.useFakeTimers();
});

afterEach(() => {
  removeBridge();
  vi.useRealTimers();
  vi.restoreAllMocks();
});

// ============================================================================
// Features constant
// ============================================================================

describe('Features constant', () => {
  it('WINDOW_DRAG equals "windowDrag"', () => {
    expect(Features.WINDOW_DRAG).toBe('windowDrag');
  });
  it('MULTI_WINDOW equals "multiWindow"', () => {
    expect(Features.MULTI_WINDOW).toBe('multiWindow');
  });
  it('CLIPBOARD equals "clipboard"', () => {
    expect(Features.CLIPBOARD).toBe('clipboard');
  });
  it('SHELL equals "shell"', () => {
    expect(Features.SHELL).toBe('shell');
  });
  it('FILE_SYSTEM equals "fileSystem"', () => {
    expect(Features.FILE_SYSTEM).toBe('fileSystem');
  });
  it('DIALOG equals "dialog"', () => {
    expect(Features.DIALOG).toBe('dialog');
  });
  it('STATE equals "state"', () => {
    expect(Features.STATE).toBe('state');
  });
  it('INVOKE equals "invoke"', () => {
    expect(Features.INVOKE).toBe('invoke');
  });
  it('API equals "api"', () => {
    expect(Features.API).toBe('api');
  });
});

// ============================================================================
// hasFeature()
// ============================================================================

describe('hasFeature()', () => {
  describe('when bridge not available', () => {
    it('returns false when window.auroraview is undefined', () => {
      removeBridge();
      expect(hasFeature('windowDrag')).toBe(false);
    });

    it('returns false when bridge isReady returns false', () => {
      installBridgeNotReady();
      expect(hasFeature('clipboard')).toBe(false);
    });
  });

  describe('windowDrag', () => {
    it('returns true when startDrag is a function', () => {
      installBridge({ startDrag: () => {} });
      expect(hasFeature('windowDrag')).toBe(true);
    });

    it('returns false when startDrag is undefined', () => {
      installBridge({ startDrag: undefined });
      expect(hasFeature('windowDrag')).toBe(false);
    });
  });

  describe('multiWindow', () => {
    it('returns true when window.create is bound', () => {
      installBridge({ isMethodBound: (m: string) => m === 'window.create' });
      expect(hasFeature('multiWindow')).toBe(true);
    });

    it('returns true when api.create_window is bound', () => {
      installBridge({ isMethodBound: (m: string) => m === 'api.create_window' });
      expect(hasFeature('multiWindow')).toBe(true);
    });

    it('returns false when neither method is bound', () => {
      installBridge({ isMethodBound: () => false });
      expect(hasFeature('multiWindow')).toBe(false);
    });
  });

  describe('clipboard', () => {
    it('returns true when clipboard object exists', () => {
      installBridge({ clipboard: { readText: vi.fn() } });
      expect(hasFeature('clipboard')).toBe(true);
    });

    it('returns true when clipboard.read is bound', () => {
      installBridge({ isMethodBound: (m: string) => m === 'clipboard.read' });
      expect(hasFeature('clipboard')).toBe(true);
    });

    it('returns true when clipboard.readText is bound', () => {
      installBridge({ isMethodBound: (m: string) => m === 'clipboard.readText' });
      expect(hasFeature('clipboard')).toBe(true);
    });

    it('returns false when no clipboard support', () => {
      installBridge({ clipboard: undefined, isMethodBound: () => false });
      expect(hasFeature('clipboard')).toBe(false);
    });
  });

  describe('shell', () => {
    it('returns true when shell object exists', () => {
      installBridge({ shell: { open: vi.fn() } });
      expect(hasFeature('shell')).toBe(true);
    });

    it('returns true when shell.open is bound', () => {
      installBridge({ isMethodBound: (m: string) => m === 'shell.open' });
      expect(hasFeature('shell')).toBe(true);
    });

    it('returns false when no shell support', () => {
      installBridge({ shell: undefined, isMethodBound: () => false });
      expect(hasFeature('shell')).toBe(false);
    });
  });

  describe('fileSystem', () => {
    it('returns true when fs object exists', () => {
      installBridge({ fs: { readFile: vi.fn() } });
      expect(hasFeature('fileSystem')).toBe(true);
    });

    it('returns true when fs.readFile is bound', () => {
      installBridge({ isMethodBound: (m: string) => m === 'fs.readFile' });
      expect(hasFeature('fileSystem')).toBe(true);
    });

    it('returns false when no fileSystem support', () => {
      installBridge({ fs: undefined, isMethodBound: () => false });
      expect(hasFeature('fileSystem')).toBe(false);
    });
  });

  describe('dialog', () => {
    it('returns true when dialog object exists', () => {
      installBridge({ dialog: { openFile: vi.fn() } });
      expect(hasFeature('dialog')).toBe(true);
    });

    it('returns true when dialog.openFile is bound', () => {
      installBridge({ isMethodBound: (m: string) => m === 'dialog.openFile' });
      expect(hasFeature('dialog')).toBe(true);
    });

    it('returns false when no dialog support', () => {
      installBridge({ dialog: undefined, isMethodBound: () => false });
      expect(hasFeature('dialog')).toBe(false);
    });
  });

  describe('state', () => {
    it('returns true when state object exists', () => {
      installBridge({ state: { get: vi.fn() } });
      expect(hasFeature('state')).toBe(true);
    });

    it('returns false when state is undefined', () => {
      installBridge({ state: undefined });
      expect(hasFeature('state')).toBe(false);
    });
  });

  describe('invoke', () => {
    it('returns true when invoke is a function', () => {
      installBridge({ invoke: vi.fn() });
      expect(hasFeature('invoke')).toBe(true);
    });

    it('returns false when invoke is not a function', () => {
      installBridge({ invoke: undefined });
      expect(hasFeature('invoke')).toBe(false);
    });
  });

  describe('api', () => {
    it('returns true when api object exists', () => {
      installBridge({ api: { myMethod: vi.fn() } });
      expect(hasFeature('api')).toBe(true);
    });

    it('returns false when api is undefined', () => {
      installBridge({ api: undefined });
      expect(hasFeature('api')).toBe(false);
    });
  });
});

// ============================================================================
// hasFeatures()
// ============================================================================

describe('hasFeatures()', () => {
  it('returns a map for given features', () => {
    installBridge({
      clipboard: { readText: vi.fn() },
      state: undefined,
      invoke: undefined,
    });
    const result = hasFeatures(['clipboard', 'state', 'invoke']);
    expect(result.clipboard).toBe(true);
    expect(result.state).toBe(false);
    expect(result.invoke).toBe(false);
  });

  it('returns empty object for empty array', () => {
    installBridge();
    const result = hasFeatures([]);
    expect(Object.keys(result)).toHaveLength(0);
  });

  it('all features false when bridge not ready', () => {
    installBridgeNotReady();
    const result = hasFeatures(['windowDrag', 'api', 'shell']);
    expect(result.windowDrag).toBe(false);
    expect(result.api).toBe(false);
    expect(result.shell).toBe(false);
  });
});

// ============================================================================
// getAvailableFeatures()
// ============================================================================

describe('getAvailableFeatures()', () => {
  it('returns empty array when bridge not ready', () => {
    installBridgeNotReady();
    expect(getAvailableFeatures()).toEqual([]);
  });

  it('returns only available features', () => {
    installBridge({
      startDrag: () => {},  // windowDrag
      clipboard: { readText: vi.fn() },  // clipboard
      // all others missing
    });
    const available = getAvailableFeatures();
    expect(available).toContain('windowDrag');
    expect(available).toContain('clipboard');
    expect(available).not.toContain('shell');
    expect(available).not.toContain('state');
  });

  it('returns all features when all are available', () => {
    installBridge({
      startDrag: () => {},
      isMethodBound: (m: string) => ['window.create', 'clipboard.read', 'shell.open', 'fs.readFile', 'dialog.openFile'].includes(m),
      state: {},
      invoke: () => {},
      api: {},
    });
    const available = getAvailableFeatures();
    expect(available.length).toBeGreaterThanOrEqual(5);
  });
});

// ============================================================================
// isAuroraView()
// ============================================================================

describe('isAuroraView()', () => {
  it('returns true when bridge isReady returns true', () => {
    installBridge();
    expect(isAuroraView()).toBe(true);
  });

  it('returns false when bridge not present', () => {
    removeBridge();
    expect(isAuroraView()).toBe(false);
  });

  it('returns false when isReady returns false', () => {
    installBridgeNotReady();
    expect(isAuroraView()).toBe(false);
  });
});

// ============================================================================
// isStandalone() / isPacked() / isDCC()
// ============================================================================

describe('isStandalone()', () => {
  it('returns true when bridge is present without dcc hint', () => {
    installBridge({ isMethodBound: () => false });
    expect(isStandalone()).toBe(true);
  });

  it('returns false when bridge is absent (browser mode)', () => {
    removeBridge();
    expect(isStandalone()).toBe(false);
  });
});

describe('isPacked()', () => {
  it('returns true when __AURORAVIEW_PACKED__ is set', () => {
    (globalThis as AnyRecord).window.__AURORAVIEW_PACKED__ = true;
    expect(isPacked()).toBe(true);
    delete (globalThis as AnyRecord).window.__AURORAVIEW_PACKED__;
  });

  it('returns false without packed flag', () => {
    installBridge();
    expect(isPacked()).toBe(false);
  });
});

describe('isDCC()', () => {
  it('returns true when isMethodBound dcc.get_host returns true', () => {
    installBridge({ isMethodBound: (m: string) => m === 'dcc.get_host' });
    expect(isDCC()).toBe(true);
  });

  it('returns true when DCC global (maya) is present', () => {
    removeBridge();
    (globalThis as AnyRecord).window.maya = {};
    expect(isDCC()).toBe(true);
    delete (globalThis as AnyRecord).window.maya;
  });

  it('returns false in standalone mode without DCC globals', () => {
    installBridge({ isMethodBound: () => false });
    expect(isDCC()).toBe(false);
  });
});

// ============================================================================
// getEnvironment()
// ============================================================================

describe('getEnvironment()', () => {
  it('returns an object with all required fields', () => {
    installBridge();
    const env = getEnvironment();
    expect(env).toHaveProperty('mode');
    expect(env).toHaveProperty('platform');
    expect(env).toHaveProperty('dccHost');
    expect(env).toHaveProperty('embedded');
    expect(env).toHaveProperty('version');
    expect(env).toHaveProperty('features');
    expect(env).toHaveProperty('userAgent');
    expect(env).toHaveProperty('debug');
  });

  it('features is an array', () => {
    installBridge();
    expect(Array.isArray(getEnvironment().features)).toBe(true);
  });

  it('debug is false by default', () => {
    installBridge();
    expect(getEnvironment().debug).toBe(false);
  });

  it('debug is true when __AURORAVIEW_DEBUG__ is set', () => {
    installBridge();
    (globalThis as AnyRecord).window.__AURORAVIEW_DEBUG__ = true;
    expect(getEnvironment().debug).toBe(true);
    delete (globalThis as AnyRecord).window.__AURORAVIEW_DEBUG__;
  });

  it('version is returned from __AURORAVIEW_VERSION__', () => {
    installBridge();
    (globalThis as AnyRecord).window.__AURORAVIEW_VERSION__ = '1.2.3';
    expect(getEnvironment().version).toBe('1.2.3');
    delete (globalThis as AnyRecord).window.__AURORAVIEW_VERSION__;
  });

  it('mode is "browser" when no bridge', () => {
    removeBridge();
    const env = getEnvironment();
    expect(env.mode).toBe('browser');
  });

  it('mode is "packed" when __AURORAVIEW_PACKED__ is set', () => {
    (globalThis as AnyRecord).window.__AURORAVIEW_PACKED__ = true;
    const env = getEnvironment();
    expect(env.mode).toBe('packed');
    delete (globalThis as AnyRecord).window.__AURORAVIEW_PACKED__;
  });

  it('mode is "dcc" when dcc.get_host is bound', () => {
    installBridge({ isMethodBound: (m: string) => m === 'dcc.get_host' });
    const env = getEnvironment();
    expect(env.mode).toBe('dcc');
  });

  it('dccHost is "maya" when window.maya is present', () => {
    removeBridge();
    (globalThis as AnyRecord).window.maya = {};
    const env = getEnvironment();
    expect(env.dccHost).toBe('maya');
    delete (globalThis as AnyRecord).window.maya;
  });

  it('dccHost is "houdini" when window.hou is present', () => {
    removeBridge();
    (globalThis as AnyRecord).window.hou = {};
    const env = getEnvironment();
    expect(env.dccHost).toBe('houdini');
    delete (globalThis as AnyRecord).window.hou;
  });

  it('dccHost is "blender" when window.bpy is present', () => {
    removeBridge();
    (globalThis as AnyRecord).window.bpy = {};
    const env = getEnvironment();
    expect(env.dccHost).toBe('blender');
    delete (globalThis as AnyRecord).window.bpy;
  });

  it('dccHost is "3dsmax" when window.pymxs is present', () => {
    removeBridge();
    (globalThis as AnyRecord).window.pymxs = {};
    const env = getEnvironment();
    expect(env.dccHost).toBe('3dsmax');
    delete (globalThis as AnyRecord).window.pymxs;
  });

  it('embedded is true when dccHost is non-null', () => {
    removeBridge();
    (globalThis as AnyRecord).window.maya = {};
    const env = getEnvironment();
    expect(env.embedded).toBe(true);
    delete (globalThis as AnyRecord).window.maya;
  });

  it('embedded is false in standalone mode with no DCC host', () => {
    installBridge({ isMethodBound: () => false });
    const env = getEnvironment();
    expect(env.embedded).toBe(false);
  });

  it('userAgent contains navigator.userAgent', () => {
    vi.stubGlobal('navigator', { userAgent: 'TestAgent/1.0', platform: '' });
    installBridge();
    const env = getEnvironment();
    expect(env.userAgent).toBe('TestAgent/1.0');
    vi.unstubAllGlobals();
  });

  it('platform detection: windows from userAgent', () => {
    vi.stubGlobal('navigator', { userAgent: 'Windows NT 10.0', platform: '' });
    installBridge();
    const env = getEnvironment();
    expect(env.platform).toBe('windows');
    vi.unstubAllGlobals();
  });

  it('platform detection: macos from userAgent', () => {
    vi.stubGlobal('navigator', { userAgent: 'Macintosh; Intel Mac OS', platform: '' });
    installBridge();
    const env = getEnvironment();
    expect(env.platform).toBe('macos');
    vi.unstubAllGlobals();
  });

  it('platform detection: linux from userAgent', () => {
    vi.stubGlobal('navigator', { userAgent: 'Linux x86_64', platform: '' });
    installBridge();
    const env = getEnvironment();
    expect(env.platform).toBe('linux');
    vi.unstubAllGlobals();
  });

  it('platform detection: unknown for unrecognized UA', () => {
    vi.stubGlobal('navigator', { userAgent: 'UnknownBrowser/99.0', platform: '' });
    installBridge();
    const env = getEnvironment();
    expect(env.platform).toBe('unknown');
    vi.unstubAllGlobals();
  });
});

// ============================================================================
// waitForFeature()
// ============================================================================

describe('waitForFeature()', () => {
  it('resolves immediately when feature is already available', async () => {
    installBridge({ startDrag: () => {} });
    await expect(waitForFeature('windowDrag', 1000)).resolves.toBeUndefined();
  });

  it('rejects after timeout when feature never becomes available', async () => {
    removeBridge();
    const p = waitForFeature('windowDrag', 200);
    vi.advanceTimersByTime(300);
    await expect(p).rejects.toThrow("Feature 'windowDrag' not available after 200ms");
  });

  it('resolves when feature becomes available mid-polling', async () => {
    removeBridge();

    // Resolve after 150ms by installing bridge
    let featureReady = false;
    vi.spyOn({ hasFeature }, 'hasFeature').mockReturnValue(false);

    // Direct approach: use real timer logic, install bridge after delay
    const p = waitForFeature('windowDrag', 5000);

    // Simulate feature becoming available by installing bridge
    vi.advanceTimersByTime(60);
    installBridge({ startDrag: () => {} });
    vi.advanceTimersByTime(60);

    await expect(p).resolves.toBeUndefined();
    expect(featureReady).toBe(false); // Just checking it resolved
  });

  it('uses default 5000ms timeout', async () => {
    removeBridge();
    const p = waitForFeature('clipboard'); // no timeout arg
    vi.advanceTimersByTime(5100);
    await expect(p).rejects.toThrow('clipboard');
  });
});
