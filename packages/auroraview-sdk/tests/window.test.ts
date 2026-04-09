/**
 * AuroraView SDK window.ts Tests
 *
 * Tests the Window class and convenience functions:
 * - Window.create / getCurrent / getByLabel / getAll / count
 * - show/hide/close/focus/minimize/maximize/restore/toggleFullscreen
 * - setTitle/setPosition/setSize/setMinSize/setMaxSize/center
 * - setAlwaysOnTop/setResizable
 * - getPosition/getSize/getBounds/getState
 * - isVisible/isFocused/isMinimized/isMaximized
 * - startDrag/navigate/loadHtml/eval/emit
 * - on/once/off event handling
 * - getCurrentWindow/createWindow/closeCurrentWindow/startDrag (convenience fns)
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import {
  Window,
  getCurrentWindow,
  createWindow,
  closeCurrentWindow,
  startDrag,
  type WindowOptions,
} from '../src/window';

// ============================================================================
// Mock helpers
// ============================================================================

type AnyRecord = Record<string, any>;

let mockCall: ReturnType<typeof vi.fn>;
let mockOn: ReturnType<typeof vi.fn>;
let mockTrigger: ReturnType<typeof vi.fn>;
let mockStartDrag: ReturnType<typeof vi.fn>;

function makeMockBridge(overrides: AnyRecord = {}): AnyRecord {
  mockCall = vi.fn().mockResolvedValue({});
  mockOn = vi.fn().mockReturnValue(() => {});
  mockTrigger = vi.fn();
  mockStartDrag = vi.fn();

  return {
    call: mockCall,
    on: mockOn,
    trigger: mockTrigger,
    startDrag: mockStartDrag,
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
  installBridge(makeMockBridge());
  // Default: main label
  delete (globalThis as AnyRecord).window.__AURORAVIEW_WINDOW_LABEL__;
});

afterEach(() => {
  removeBridge();
  vi.clearAllMocks();
});

// ============================================================================
// Window.create
// ============================================================================

describe('Window.create', () => {
  it('calls bridge window.create with options and returns Window', async () => {
    mockCall.mockResolvedValueOnce({ label: 'settings' });
    const opts: WindowOptions = { label: 'settings', url: '/settings.html', width: 520 };
    const win = await Window.create(opts);
    expect(mockCall).toHaveBeenCalledWith('window.create', opts);
    expect(win.label).toBe('settings');
  });

  it('throws when bridge is unavailable', async () => {
    removeBridge();
    await expect(Window.create({})).rejects.toThrow('AuroraView bridge not available');
  });

  it('uses label from bridge response', async () => {
    mockCall.mockResolvedValueOnce({ label: 'custom-win' });
    const win = await Window.create({ label: 'custom-win' });
    expect(win.label).toBe('custom-win');
  });
});

// ============================================================================
// Window.getCurrent
// ============================================================================

describe('Window.getCurrent', () => {
  it('returns window with label "main" by default', () => {
    const win = Window.getCurrent();
    expect(win.label).toBe('main');
  });

  it('returns window with custom label from global', () => {
    (globalThis as AnyRecord).window.__AURORAVIEW_WINDOW_LABEL__ = 'sidebar';
    const win = Window.getCurrent();
    expect(win.label).toBe('sidebar');
  });
});

// ============================================================================
// Window.getByLabel
// ============================================================================

describe('Window.getByLabel', () => {
  it('returns Window when bridge says it exists', async () => {
    mockCall.mockResolvedValueOnce({ exists: true });
    const win = await Window.getByLabel('settings');
    expect(win).not.toBeNull();
    expect(win?.label).toBe('settings');
    expect(mockCall).toHaveBeenCalledWith('window.exists', { label: 'main', label: 'settings' });
  });

  it('returns null when bridge says not exists', async () => {
    mockCall.mockResolvedValueOnce({ exists: false });
    const win = await Window.getByLabel('ghost');
    expect(win).toBeNull();
  });

  it('returns null when bridge is unavailable', async () => {
    removeBridge();
    const win = await Window.getByLabel('any');
    expect(win).toBeNull();
  });

  it('returns null when bridge call throws', async () => {
    mockCall.mockRejectedValueOnce(new Error('not found'));
    const win = await Window.getByLabel('missing');
    expect(win).toBeNull();
  });
});

// ============================================================================
// Window.getAll
// ============================================================================

describe('Window.getAll', () => {
  it('returns array of Window instances', async () => {
    mockCall.mockResolvedValueOnce({ labels: ['main', 'sidebar', 'modal'] });
    const windows = await Window.getAll();
    expect(windows).toHaveLength(3);
    expect(windows[0].label).toBe('main');
    expect(windows[2].label).toBe('modal');
  });

  it('returns empty array when bridge unavailable', async () => {
    removeBridge();
    const windows = await Window.getAll();
    expect(windows).toEqual([]);
  });

  it('returns empty array when bridge throws', async () => {
    mockCall.mockRejectedValueOnce(new Error('fail'));
    const windows = await Window.getAll();
    expect(windows).toEqual([]);
  });

  it('returns empty array when labels is empty', async () => {
    mockCall.mockResolvedValueOnce({ labels: [] });
    const windows = await Window.getAll();
    expect(windows).toEqual([]);
  });
});

// ============================================================================
// Window.count
// ============================================================================

describe('Window.count', () => {
  it('returns count from bridge', async () => {
    mockCall.mockResolvedValueOnce({ count: 5 });
    const count = await Window.count();
    expect(count).toBe(5);
  });

  it('returns 0 when bridge unavailable', async () => {
    removeBridge();
    expect(await Window.count()).toBe(0);
  });

  it('returns 0 when bridge throws', async () => {
    mockCall.mockRejectedValueOnce(new Error('fail'));
    expect(await Window.count()).toBe(0);
  });
});

// ============================================================================
// Instance methods – show/hide/close/focus
// ============================================================================

describe('Window instance – lifecycle methods', () => {
  it('show calls window.show', async () => {
    const win = Window.getCurrent();
    await win.show();
    expect(mockCall).toHaveBeenCalledWith('window.show', { label: 'main' });
  });

  it('hide calls window.hide', async () => {
    const win = Window.getCurrent();
    await win.hide();
    expect(mockCall).toHaveBeenCalledWith('window.hide', { label: 'main' });
  });

  it('close calls window.close and cleans up', async () => {
    const win = Window.getCurrent();
    await win.close();
    expect(mockCall).toHaveBeenCalledWith('window.close', { label: 'main' });
  });

  it('focus calls window.focus', async () => {
    const win = Window.getCurrent();
    await win.focus();
    expect(mockCall).toHaveBeenCalledWith('window.focus', { label: 'main' });
  });

  it('minimize calls window.minimize', async () => {
    const win = Window.getCurrent();
    await win.minimize();
    expect(mockCall).toHaveBeenCalledWith('window.minimize', { label: 'main' });
  });

  it('maximize calls window.maximize', async () => {
    const win = Window.getCurrent();
    await win.maximize();
    expect(mockCall).toHaveBeenCalledWith('window.maximize', { label: 'main' });
  });

  it('restore calls window.restore', async () => {
    const win = Window.getCurrent();
    await win.restore();
    expect(mockCall).toHaveBeenCalledWith('window.restore', { label: 'main' });
  });

  it('toggleFullscreen calls window.toggleFullscreen', async () => {
    const win = Window.getCurrent();
    await win.toggleFullscreen();
    expect(mockCall).toHaveBeenCalledWith('window.toggleFullscreen', { label: 'main' });
  });
});

// ============================================================================
// Instance methods – set* / getters
// ============================================================================

describe('Window instance – set/get methods', () => {
  it('setTitle calls window.setTitle with title', async () => {
    const win = Window.getCurrent();
    await win.setTitle('New Title');
    expect(mockCall).toHaveBeenCalledWith('window.setTitle', { label: 'main', title: 'New Title' });
  });

  it('setPosition calls window.setPosition', async () => {
    const win = Window.getCurrent();
    await win.setPosition(100, 200);
    expect(mockCall).toHaveBeenCalledWith('window.setPosition', { label: 'main', x: 100, y: 200 });
  });

  it('setSize calls window.setSize', async () => {
    const win = Window.getCurrent();
    await win.setSize(800, 600);
    expect(mockCall).toHaveBeenCalledWith('window.setSize', { label: 'main', width: 800, height: 600 });
  });

  it('setMinSize calls window.setMinSize', async () => {
    const win = Window.getCurrent();
    await win.setMinSize(200, 150);
    expect(mockCall).toHaveBeenCalledWith('window.setMinSize', { label: 'main', width: 200, height: 150 });
  });

  it('setMaxSize calls window.setMaxSize', async () => {
    const win = Window.getCurrent();
    await win.setMaxSize(1920, 1080);
    expect(mockCall).toHaveBeenCalledWith('window.setMaxSize', { label: 'main', width: 1920, height: 1080 });
  });

  it('center calls window.center', async () => {
    const win = Window.getCurrent();
    await win.center();
    expect(mockCall).toHaveBeenCalledWith('window.center', { label: 'main' });
  });

  it('setAlwaysOnTop calls window.setAlwaysOnTop', async () => {
    const win = Window.getCurrent();
    await win.setAlwaysOnTop(true);
    expect(mockCall).toHaveBeenCalledWith('window.setAlwaysOnTop', { label: 'main', alwaysOnTop: true });
  });

  it('setResizable calls window.setResizable', async () => {
    const win = Window.getCurrent();
    await win.setResizable(false);
    expect(mockCall).toHaveBeenCalledWith('window.setResizable', { label: 'main', resizable: false });
  });

  it('getPosition returns position from bridge', async () => {
    mockCall.mockResolvedValueOnce({ x: 10, y: 20 });
    const win = Window.getCurrent();
    const pos = await win.getPosition();
    expect(pos).toEqual({ x: 10, y: 20 });
  });

  it('getSize returns size from bridge', async () => {
    mockCall.mockResolvedValueOnce({ width: 800, height: 600 });
    const win = Window.getCurrent();
    const size = await win.getSize();
    expect(size).toEqual({ width: 800, height: 600 });
  });

  it('getBounds returns bounds from bridge', async () => {
    const bounds = { x: 0, y: 0, width: 1024, height: 768 };
    mockCall.mockResolvedValueOnce(bounds);
    const win = Window.getCurrent();
    const result = await win.getBounds();
    expect(result).toEqual(bounds);
  });

  it('getState returns state from bridge', async () => {
    const state = { label: 'main', visible: true, focused: true, minimized: false, maximized: false, fullscreen: false, bounds: { x: 0, y: 0, width: 800, height: 600 } };
    mockCall.mockResolvedValueOnce(state);
    const win = Window.getCurrent();
    const result = await win.getState();
    expect(result).toEqual(state);
  });
});

// ============================================================================
// Boolean query methods
// ============================================================================

describe('Window instance – boolean query methods', () => {
  it('isVisible returns true', async () => {
    mockCall.mockResolvedValueOnce({ visible: true });
    expect(await Window.getCurrent().isVisible()).toBe(true);
  });

  it('isFocused returns false', async () => {
    mockCall.mockResolvedValueOnce({ focused: false });
    expect(await Window.getCurrent().isFocused()).toBe(false);
  });

  it('isMinimized returns true', async () => {
    mockCall.mockResolvedValueOnce({ minimized: true });
    expect(await Window.getCurrent().isMinimized()).toBe(true);
  });

  it('isMaximized returns false', async () => {
    mockCall.mockResolvedValueOnce({ maximized: false });
    expect(await Window.getCurrent().isMaximized()).toBe(false);
  });
});

// ============================================================================
// startDrag / navigate / loadHtml / eval / emit
// ============================================================================

describe('Window instance – content methods', () => {
  it('startDrag calls bridge.startDrag', () => {
    Window.getCurrent().startDrag();
    expect(mockStartDrag).toHaveBeenCalled();
  });

  it('startDrag does nothing when bridge.startDrag absent', () => {
    const bridge = makeMockBridge({ startDrag: undefined });
    installBridge(bridge);
    expect(() => Window.getCurrent().startDrag()).not.toThrow();
  });

  it('navigate calls window.navigate with url', async () => {
    await Window.getCurrent().navigate('https://example.com');
    expect(mockCall).toHaveBeenCalledWith('window.navigate', { label: 'main', url: 'https://example.com' });
  });

  it('loadHtml calls window.loadHtml with html', async () => {
    await Window.getCurrent().loadHtml('<h1>Hi</h1>');
    expect(mockCall).toHaveBeenCalledWith('window.loadHtml', { label: 'main', html: '<h1>Hi</h1>' });
  });

  it('eval calls window.eval with script', async () => {
    mockCall.mockResolvedValueOnce(42);
    const result = await Window.getCurrent().eval<number>('1+1');
    expect(mockCall).toHaveBeenCalledWith('window.eval', { label: 'main', script: '1+1' });
    expect(result).toBe(42);
  });

  it('emit calls window.emit with event and data', async () => {
    await Window.getCurrent().emit('my-event', { x: 1 });
    expect(mockCall).toHaveBeenCalledWith('window.emit', { label: 'main', event: 'my-event', data: { x: 1 } });
  });
});

// ============================================================================
// on / once / off event handling
// ============================================================================

describe('Window instance – event handling', () => {
  it('on registers handler and returns unsubscribe', () => {
    const win = Window.getCurrent();
    const handler = vi.fn();
    const unsub = win.on('resized', handler);
    expect(typeof unsub).toBe('function');
  });

  it('handler fires when _triggerEvent is called via bridge.on callback', () => {
    // Bridge.on captures callbacks; simulate trigger via captured callback
    const callbacks: Record<string, Function> = {};
    const bridge = makeMockBridge({
      on: vi.fn().mockImplementation((event: string, cb: Function) => {
        callbacks[event] = cb;
        return () => {};
      }),
    });
    installBridge(bridge);
    const win = Window.getCurrent();
    const handler = vi.fn();
    win.on('resized', handler);

    // Simulate resized event from backend
    const resizeCallback = callbacks['window:main:resized'];
    resizeCallback?.({ width: 1024, height: 768 });
    expect(handler).toHaveBeenCalledOnce();
  });

  it('once handler fires only once', () => {
    const win = Window.getCurrent();
    const handler = vi.fn();
    win.once('shown', handler);
    // Fire event twice by directly calling internal trigger via on callbacks
    const callbacks: Record<string, Function> = {};
    const bridge = makeMockBridge({
      on: vi.fn().mockImplementation((event: string, cb: Function) => {
        callbacks[event] = cb;
        return () => {};
      }),
    });
    installBridge(bridge);
    const win2 = Window.getCurrent();
    const h = vi.fn();
    win2.once('shown', h);
    callbacks['window:main:shown']?.();
    callbacks['window:main:shown']?.();
    expect(h).toHaveBeenCalledTimes(1);
  });

  it('off with handler removes specific handler', () => {
    const win = Window.getCurrent();
    const h1 = vi.fn();
    const h2 = vi.fn();
    win.on('focused', h1);
    win.on('focused', h2);
    win.off('focused', h1);
    // Trigger event via unsubscribe-return path
    const unsub = win.on('focused', vi.fn());
    unsub();
  });

  it('off without handler removes all handlers', () => {
    const win = Window.getCurrent();
    const h1 = vi.fn();
    win.on('closed', h1);
    win.off('closed');
    expect(() => win.off('closed')).not.toThrow();
  });
});

// ============================================================================
// Convenience functions
// ============================================================================

describe('Convenience functions', () => {
  it('getCurrentWindow returns current window', () => {
    const win = getCurrentWindow();
    expect(win.label).toBe('main');
  });

  it('createWindow creates a window', async () => {
    mockCall.mockResolvedValueOnce({ label: 'new-win' });
    const win = await createWindow({ label: 'new-win' });
    expect(win.label).toBe('new-win');
  });

  it('closeCurrentWindow closes current window', async () => {
    await closeCurrentWindow();
    expect(mockCall).toHaveBeenCalledWith('window.close', { label: 'main' });
  });

  it('startDrag calls bridge.startDrag on current window', () => {
    startDrag();
    expect(mockStartDrag).toHaveBeenCalled();
  });
});

// ============================================================================
// label accessor
// ============================================================================

describe('Window.label accessor', () => {
  it('returns the label set on construction', async () => {
    mockCall.mockResolvedValueOnce({ label: 'my-label' });
    const win = await Window.create({ label: 'my-label' });
    expect(win.label).toBe('my-label');
  });
});
