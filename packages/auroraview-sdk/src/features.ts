/**
 * AuroraView SDK Feature Detection API
 *
 * Provides runtime detection of available features and environment information.
 *
 * @module features
 */

// ============================================
// Feature Types
// ============================================

/**
 * Available feature names that can be detected at runtime.
 */
export type FeatureName =
  | 'windowDrag'
  | 'multiWindow'
  | 'clipboard'
  | 'shell'
  | 'fileSystem'
  | 'dialog'
  | 'state'
  | 'invoke'
  | 'api';

/**
 * Platform types
 */
export type Platform = 'windows' | 'macos' | 'linux' | 'unknown';

/**
 * Runtime mode types
 */
export type RuntimeMode = 'standalone' | 'dcc' | 'browser' | 'packed';

/**
 * Known DCC host applications
 */
export type DCCHost =
  | 'maya'
  | 'houdini'
  | 'blender'
  | 'nuke'
  | '3dsmax'
  | 'photoshop'
  | 'unreal'
  | null;

/**
 * Environment information returned by getEnvironment()
 */
export interface Environment {
  /** Runtime mode of the application */
  mode: RuntimeMode;
  /** Operating system platform */
  platform: Platform;
  /** DCC host application (if running inside one) */
  dccHost: DCCHost;
  /** Whether running in embedded mode (inside another application) */
  embedded: boolean;
  /** AuroraView version string */
  version: string;
  /** List of available features */
  features: FeatureName[];
  /** User agent string */
  userAgent: string;
  /** Debug mode enabled */
  debug: boolean;
}

/**
 * Feature constants for convenient access
 */
export const Features = {
  WINDOW_DRAG: 'windowDrag' as const,
  MULTI_WINDOW: 'multiWindow' as const,
  CLIPBOARD: 'clipboard' as const,
  SHELL: 'shell' as const,
  FILE_SYSTEM: 'fileSystem' as const,
  DIALOG: 'dialog' as const,
  STATE: 'state' as const,
  INVOKE: 'invoke' as const,
  API: 'api' as const,
} as const;

// ============================================
// Implementation
// ============================================

/**
 * Check if a specific feature is available.
 *
 * @param feature - The feature name to check
 * @returns true if the feature is available, false otherwise
 *
 * @example
 * ```typescript
 * import { hasFeature, Features } from '@auroraview/sdk';
 *
 * if (hasFeature(Features.WINDOW_DRAG)) {
 *   window.auroraview.startDrag();
 * }
 *
 * if (hasFeature('clipboard')) {
 *   const text = await window.auroraview.clipboard.readText();
 * }
 * ```
 */
export function hasFeature(feature: FeatureName): boolean {
  if (typeof window === 'undefined') return false;

  const bridge = window.auroraview;
  if (!bridge?.isReady?.()) return false;

  switch (feature) {
    case 'windowDrag':
      return typeof bridge.startDrag === 'function';

    case 'multiWindow':
      return (
        bridge.isMethodBound?.('window.create') === true ||
        bridge.isMethodBound?.('api.create_window') === true
      );

    case 'clipboard':
      return (
        bridge.clipboard !== undefined ||
        bridge.isMethodBound?.('clipboard.read') === true ||
        bridge.isMethodBound?.('clipboard.readText') === true
      );

    case 'shell':
      return (
        bridge.shell !== undefined ||
        bridge.isMethodBound?.('shell.open') === true
      );

    case 'fileSystem':
      return (
        bridge.fs !== undefined ||
        bridge.isMethodBound?.('fs.readFile') === true
      );

    case 'dialog':
      return (
        bridge.dialog !== undefined ||
        bridge.isMethodBound?.('dialog.openFile') === true
      );

    case 'state':
      return bridge.state !== undefined;

    case 'invoke':
      return typeof bridge.invoke === 'function';

    case 'api':
      return bridge.api !== undefined;

    default:
      return false;
  }
}

/**
 * Check multiple features at once.
 *
 * @param features - Array of feature names to check
 * @returns Object mapping feature names to their availability
 *
 * @example
 * ```typescript
 * const available = hasFeatures(['clipboard', 'shell', 'dialog']);
 * // { clipboard: true, shell: true, dialog: false }
 * ```
 */
export function hasFeatures(
  features: FeatureName[]
): Record<FeatureName, boolean> {
  const result: Partial<Record<FeatureName, boolean>> = {};
  for (const feature of features) {
    result[feature] = hasFeature(feature);
  }
  return result as Record<FeatureName, boolean>;
}

/**
 * Get all available features.
 *
 * @returns Array of available feature names
 *
 * @example
 * ```typescript
 * const features = getAvailableFeatures();
 * // ['windowDrag', 'clipboard', 'shell', ...]
 * ```
 */
export function getAvailableFeatures(): FeatureName[] {
  const allFeatures: FeatureName[] = [
    'windowDrag',
    'multiWindow',
    'clipboard',
    'shell',
    'fileSystem',
    'dialog',
    'state',
    'invoke',
    'api',
  ];

  return allFeatures.filter((f) => hasFeature(f));
}

/**
 * Detect the current platform.
 */
function detectPlatform(): Platform {
  if (typeof navigator === 'undefined') return 'unknown';

  const ua = navigator.userAgent.toLowerCase();
  const platform = navigator.platform?.toLowerCase() || '';

  if (ua.includes('win') || platform.includes('win')) {
    return 'windows';
  }
  if (ua.includes('mac') || platform.includes('mac')) {
    return 'macos';
  }
  if (ua.includes('linux') || platform.includes('linux')) {
    return 'linux';
  }

  return 'unknown';
}

/**
 * Detect the runtime mode.
 */
function detectMode(): RuntimeMode {
  if (typeof window === 'undefined') return 'browser';

  // Check for packed mode environment variable
  const w = window as { __AURORAVIEW_PACKED__?: boolean };
  if (w.__AURORAVIEW_PACKED__) {
    return 'packed';
  }

  // Check if running inside AuroraView
  if (!window.auroraview) {
    return 'browser';
  }

  // Check for DCC host indicators
  const bridge = window.auroraview;
  if (bridge.isMethodBound?.('dcc.get_host')) {
    return 'dcc';
  }

  return 'standalone';
}

/**
 * Detect DCC host application.
 */
function detectDCCHost(): DCCHost {
  if (typeof window === 'undefined') return null;

  // Check window global for DCC indicators
  const w = window as Record<string, unknown>;

  // Maya
  if (w.maya || w.cmds) return 'maya';

  // Houdini
  if (w.hou) return 'houdini';

  // Blender
  if (w.bpy) return 'blender';

  // Nuke
  if (w.nuke) return 'nuke';

  // 3ds Max
  if (w.MaxPlus || w.pymxs) return '3dsmax';

  // Check user agent for hints
  if (typeof navigator !== 'undefined') {
    const ua = navigator.userAgent.toLowerCase();
    if (ua.includes('maya')) return 'maya';
    if (ua.includes('houdini')) return 'houdini';
    if (ua.includes('blender')) return 'blender';
    if (ua.includes('nuke')) return 'nuke';
    if (ua.includes('3dsmax') || ua.includes('3ds max')) return '3dsmax';
    if (ua.includes('photoshop')) return 'photoshop';
    if (ua.includes('unreal')) return 'unreal';
  }

  return null;
}

/**
 * Get the AuroraView version.
 */
function getVersion(): string {
  if (typeof window === 'undefined') return 'unknown';

  const w = window as { __AURORAVIEW_VERSION__?: string };
  if (w.__AURORAVIEW_VERSION__) {
    return w.__AURORAVIEW_VERSION__;
  }

  // Try to get from bridge
  const bridge = window.auroraview;
  if (bridge) {
    const methods = bridge.getBoundMethods?.() || [];
    // If bridge exists, assume version 0.4.x+
    if (methods.length > 0) {
      return '0.4.0+';
    }
  }

  return 'unknown';
}

/**
 * Check if debug mode is enabled.
 */
function isDebugMode(): boolean {
  if (typeof window === 'undefined') return false;

  const w = window as { __AURORAVIEW_DEBUG__?: boolean };
  return w.__AURORAVIEW_DEBUG__ === true;
}

/**
 * Get comprehensive environment information.
 *
 * @returns Environment object with all detected information
 *
 * @example
 * ```typescript
 * import { getEnvironment } from '@auroraview/sdk';
 *
 * const env = getEnvironment();
 * console.log(env);
 * // {
 * //   mode: 'standalone',
 * //   platform: 'windows',
 * //   dccHost: null,
 * //   embedded: false,
 * //   version: '0.4.5',
 * //   features: ['windowDrag', 'clipboard', 'shell', ...],
 * //   userAgent: 'Mozilla/5.0...',
 * //   debug: false
 * // }
 *
 * // Conditional behavior based on environment
 * if (env.mode === 'dcc' && env.dccHost === 'maya') {
 *   // Maya-specific code
 * }
 * ```
 */
export function getEnvironment(): Environment {
  const mode = detectMode();
  const dccHost = detectDCCHost();

  return {
    mode,
    platform: detectPlatform(),
    dccHost,
    embedded: mode === 'dcc' || dccHost !== null,
    version: getVersion(),
    features: getAvailableFeatures(),
    userAgent: typeof navigator !== 'undefined' ? navigator.userAgent : '',
    debug: isDebugMode(),
  };
}

/**
 * Check if running in AuroraView environment (not a regular browser).
 *
 * @returns true if running inside AuroraView
 *
 * @example
 * ```typescript
 * if (isAuroraView()) {
 *   // Use native features
 *   await window.auroraview.clipboard.writeText('Hello');
 * } else {
 *   // Fallback to browser APIs
 *   await navigator.clipboard.writeText('Hello');
 * }
 * ```
 */
export function isAuroraView(): boolean {
  if (typeof window === 'undefined') return false;
  return window.auroraview?.isReady?.() === true;
}

/**
 * Check if running in DCC (Digital Content Creation) environment.
 *
 * @returns true if running inside a DCC application
 */
export function isDCC(): boolean {
  return detectMode() === 'dcc' || detectDCCHost() !== null;
}

/**
 * Check if running in standalone mode.
 *
 * @returns true if running as a standalone application
 */
export function isStandalone(): boolean {
  return detectMode() === 'standalone';
}

/**
 * Check if running in packed mode.
 *
 * @returns true if running as a packed application
 */
export function isPacked(): boolean {
  return detectMode() === 'packed';
}

/**
 * Wait for a specific feature to become available.
 *
 * @param feature - The feature to wait for
 * @param timeoutMs - Maximum time to wait (default: 5000ms)
 * @returns Promise that resolves when feature is available, or rejects on timeout
 *
 * @example
 * ```typescript
 * try {
 *   await waitForFeature('clipboard', 3000);
 *   const text = await window.auroraview.clipboard.readText();
 * } catch (e) {
 *   console.log('Clipboard not available');
 * }
 * ```
 */
export function waitForFeature(
  feature: FeatureName,
  timeoutMs: number = 5000
): Promise<void> {
  return new Promise((resolve, reject) => {
    // Check immediately
    if (hasFeature(feature)) {
      resolve();
      return;
    }

    const startTime = Date.now();
    const checkInterval = setInterval(() => {
      if (hasFeature(feature)) {
        clearInterval(checkInterval);
        resolve();
        return;
      }

      if (Date.now() - startTime > timeoutMs) {
        clearInterval(checkInterval);
        reject(new Error(`Feature '${feature}' not available after ${timeoutMs}ms`));
      }
    }, 50);
  });
}
