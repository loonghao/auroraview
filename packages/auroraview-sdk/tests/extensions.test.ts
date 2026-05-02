/**
 * Extension View Manager Tests
 *
 * Tests the ExtensionViewManager class with a mocked invoker,
 * covering createView, getView, devtools, CDP info, and error paths.
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { ExtensionViewManager, getExtensionViewManager } from '../src/extensions/view-manager';
import type { ExtensionViewInfo, CdpConnectionInfo } from '../src/extensions/types';

// ============================================================================
// Helpers
// ============================================================================

function mockViewInfo(overrides: Partial<ExtensionViewInfo> = {}): ExtensionViewInfo {
  return {
    viewId: 'view-001',
    extensionId: 'ext-test',
    viewType: 'side_panel',
    state: 'visible',
    title: 'Test Panel',
    devtoolsOpen: false,
    devtoolsUrl: 'http://localhost:9222',
    debugPort: 9222,
    ...overrides,
  };
}

// ============================================================================
// ExtensionViewManager.createView
// ============================================================================

describe('ExtensionViewManager.createView', () => {
  it('should call invoker with correct command', async () => {
    const mockInvoker = vi.fn().mockResolvedValue(mockViewInfo());
    const manager = new ExtensionViewManager(mockInvoker);

    await manager.createView({
      extensionId: 'ext-1',
      viewType: 'popup',
      htmlPath: 'popup.html',
    });

    expect(mockInvoker).toHaveBeenCalledWith(
      'plugin:extensions|create_view',
      expect.objectContaining({
        extensionId: 'ext-1',
        viewType: 'popup',
        htmlPath: 'popup.html',
      })
    );
  });

  it('should return the view info from invoker', async () => {
    const expected = mockViewInfo({ viewId: 'v123', title: 'My Panel' });
    const mockInvoker = vi.fn().mockResolvedValue(expected);
    const manager = new ExtensionViewManager(mockInvoker);

    const result = await manager.createView({
      extensionId: 'ext-x',
      viewType: 'side_panel',
      htmlPath: 'panel.html',
    });

    expect(result).toEqual(expected);
  });

  it('should pass optional config fields', async () => {
    const mockInvoker = vi.fn().mockResolvedValue(mockViewInfo());
    const manager = new ExtensionViewManager(mockInvoker);

    await manager.createView({
      extensionId: 'ext-2',
      viewType: 'popup',
      htmlPath: 'popup.html',
      title: 'My Popup',
      width: 400,
      height: 300,
      devTools: true,
      debugPort: 9999,
      visible: false,
      parentHwnd: 12345,
    });

    expect(mockInvoker).toHaveBeenCalledWith(
      'plugin:extensions|create_view',
      expect.objectContaining({
        title: 'My Popup',
        width: 400,
        height: 300,
        devTools: true,
        debugPort: 9999,
        visible: false,
        parentHwnd: 12345,
      })
    );
  });

  it('should propagate errors from invoker', async () => {
    const mockInvoker = vi.fn().mockRejectedValue(new Error('Create failed'));
    const manager = new ExtensionViewManager(mockInvoker);

    await expect(
      manager.createView({ extensionId: 'e', viewType: 'popup', htmlPath: 'p.html' })
    ).rejects.toThrow('Create failed');
  });
});

// ============================================================================
// ExtensionViewManager.getView
// ============================================================================

describe('ExtensionViewManager.getView', () => {
  it('should return view info on success', async () => {
    const info = mockViewInfo({ viewId: 'target-view' });
    const mockInvoker = vi.fn().mockResolvedValue(info);
    const manager = new ExtensionViewManager(mockInvoker);

    const result = await manager.getView('target-view');
    expect(result).toEqual(info);
    expect(mockInvoker).toHaveBeenCalledWith('plugin:extensions|get_view', {
      viewId: 'target-view',
    });
  });

  it('should return null when invoker throws', async () => {
    const mockInvoker = vi.fn().mockRejectedValue(new Error('not found'));
    const manager = new ExtensionViewManager(mockInvoker);

    const result = await manager.getView('missing-view');
    expect(result).toBeNull();
  });
});

// ============================================================================
// ExtensionViewManager.getExtensionViews / getAllViews
// ============================================================================

describe('ExtensionViewManager.getExtensionViews', () => {
  it('should return array of views for extension', async () => {
    const views = [mockViewInfo({ viewId: 'v1' }), mockViewInfo({ viewId: 'v2' })];
    const mockInvoker = vi.fn().mockResolvedValue(views);
    const manager = new ExtensionViewManager(mockInvoker);

    const result = await manager.getExtensionViews('my-ext');
    expect(result).toHaveLength(2);
    expect(mockInvoker).toHaveBeenCalledWith(
      'plugin:extensions|get_extension_views',
      { extensionId: 'my-ext' }
    );
  });

  it('should return empty array when no views', async () => {
    const mockInvoker = vi.fn().mockResolvedValue([]);
    const manager = new ExtensionViewManager(mockInvoker);

    const result = await manager.getExtensionViews('empty-ext');
    expect(result).toEqual([]);
  });
});

describe('ExtensionViewManager.getAllViews', () => {
  it('should call correct command with no args', async () => {
    const views = [mockViewInfo()];
    const mockInvoker = vi.fn().mockResolvedValue(views);
    const manager = new ExtensionViewManager(mockInvoker);

    const result = await manager.getAllViews();
    expect(result).toEqual(views);
    expect(mockInvoker).toHaveBeenCalledWith('plugin:extensions|get_all_views');
  });
});

// ============================================================================
// DevTools methods
// ============================================================================

describe('ExtensionViewManager devtools', () => {
  it('openDevtools should call correct command', async () => {
    const mockInvoker = vi.fn().mockResolvedValue(undefined);
    const manager = new ExtensionViewManager(mockInvoker);

    await manager.openDevtools('view-abc');
    expect(mockInvoker).toHaveBeenCalledWith('plugin:extensions|open_devtools', {
      viewId: 'view-abc',
    });
  });

  it('closeDevtools should call correct command', async () => {
    const mockInvoker = vi.fn().mockResolvedValue(undefined);
    const manager = new ExtensionViewManager(mockInvoker);

    await manager.closeDevtools('view-abc');
    expect(mockInvoker).toHaveBeenCalledWith('plugin:extensions|close_devtools', {
      viewId: 'view-abc',
    });
  });
});

// ============================================================================
// show / hide / destroy
// ============================================================================

describe('ExtensionViewManager visibility', () => {
  it('showView should call correct command', async () => {
    const mockInvoker = vi.fn().mockResolvedValue(undefined);
    const manager = new ExtensionViewManager(mockInvoker);

    await manager.showView('my-view');
    expect(mockInvoker).toHaveBeenCalledWith('plugin:extensions|show_view', {
      viewId: 'my-view',
    });
  });

  it('hideView should call correct command', async () => {
    const mockInvoker = vi.fn().mockResolvedValue(undefined);
    const manager = new ExtensionViewManager(mockInvoker);

    await manager.hideView('my-view');
    expect(mockInvoker).toHaveBeenCalledWith('plugin:extensions|hide_view', {
      viewId: 'my-view',
    });
  });

  it('destroyView should call correct command', async () => {
    const mockInvoker = vi.fn().mockResolvedValue(undefined);
    const manager = new ExtensionViewManager(mockInvoker);

    await manager.destroyView('view-to-delete');
    expect(mockInvoker).toHaveBeenCalledWith('plugin:extensions|destroy_view', {
      viewId: 'view-to-delete',
    });
  });
});

// ============================================================================
// CDP Info
// ============================================================================

describe('ExtensionViewManager.getCdpInfo', () => {
  it('should return CDP connection info on success', async () => {
    const cdpInfo: CdpConnectionInfo = {
      viewId: 'view-123',
      host: 'localhost',
      port: 9222,
      wsUrl: 'ws://localhost:9222/devtools/page/abc',
      devtoolsFrontendUrl: 'http://localhost:9222',
    };
    const mockInvoker = vi.fn().mockResolvedValue(cdpInfo);
    const manager = new ExtensionViewManager(mockInvoker);

    const result = await manager.getCdpInfo('view-123');
    expect(result).toEqual(cdpInfo);
    expect(mockInvoker).toHaveBeenCalledWith('plugin:extensions|get_cdp_info', {
      viewId: 'view-123',
    });
  });

  it('should return null when invoker throws', async () => {
    const mockInvoker = vi.fn().mockRejectedValue(new Error('no CDP'));
    const manager = new ExtensionViewManager(mockInvoker);

    const result = await manager.getCdpInfo('dead-view');
    expect(result).toBeNull();
  });
});

describe('ExtensionViewManager.getAllCdpConnections', () => {
  it('should return all CDP connections', async () => {
    const connections: CdpConnectionInfo[] = [
      { viewId: 'v1', host: 'localhost', port: 9222, wsUrl: 'ws://...', devtoolsFrontendUrl: 'http://...' },
      { viewId: 'v2', host: 'localhost', port: 9223, wsUrl: 'ws://...', devtoolsFrontendUrl: 'http://...' },
    ];
    const mockInvoker = vi.fn().mockResolvedValue(connections);
    const manager = new ExtensionViewManager(mockInvoker);

    const result = await manager.getAllCdpConnections();
    expect(result).toHaveLength(2);
    expect(mockInvoker).toHaveBeenCalledWith('plugin:extensions|get_all_cdp_connections');
  });
});

// ============================================================================
// Fallback invoker (window.auroraview)
// ============================================================================

describe('ExtensionViewManager without explicit invoker', () => {
  let originalWindow: typeof window;

  beforeEach(() => {
    originalWindow = (global as unknown as { window: typeof window }).window;
  });

  afterEach(() => {
    (global as unknown as { window: typeof window }).window = originalWindow;
  });

  it('should use window.auroraview.invoke when no invoker provided', async () => {
    const mockInvoke = vi.fn().mockResolvedValue(mockViewInfo());
    (global as unknown as { window: { auroraview: { invoke: typeof mockInvoke } } }).window = {
      auroraview: { invoke: mockInvoke },
    };

    const manager = new ExtensionViewManager();
    await manager.createView({
      extensionId: 'e1',
      viewType: 'side_panel',
      htmlPath: 'panel.html',
    });

    expect(mockInvoke).toHaveBeenCalled();
  });

  it('should reject when window.auroraview is not available', async () => {
    (global as unknown as { window: { auroraview?: unknown } }).window = {};

    const manager = new ExtensionViewManager();
    await expect(
      manager.createView({ extensionId: 'e', viewType: 'popup', htmlPath: 'p.html' })
    ).rejects.toThrow('AuroraView bridge not available');
  });
});

// ============================================================================
// getExtensionViewManager singleton
// ============================================================================

describe('getExtensionViewManager', () => {
  it('should return an ExtensionViewManager instance', () => {
    const manager = getExtensionViewManager();
    expect(manager).toBeInstanceOf(ExtensionViewManager);
  });

  it('should return the same instance on repeated calls', () => {
    const m1 = getExtensionViewManager();
    const m2 = getExtensionViewManager();
    expect(m1).toBe(m2);
  });
});
