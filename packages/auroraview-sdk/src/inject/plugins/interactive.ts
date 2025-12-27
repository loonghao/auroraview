/**
 * Interactive regions plugin for click-through windows
 *
 * This plugin monitors DOM elements with the `data-interactive` attribute
 * and reports their positions to the native layer for click-through support.
 *
 * @example
 * ```html
 * <!-- Mark an element as interactive -->
 * <button data-interactive>Click Me</button>
 *
 * <!-- The plugin will automatically track this element's position -->
 * ```
 *
 * @example
 * ```typescript
 * import { interactive } from 'auroraview-sdk';
 *
 * // Manually trigger a region update
 * interactive.update();
 *
 * // Get current regions
 * const regions = interactive.getRegions();
 *
 * // Enable/disable tracking
 * interactive.setEnabled(false);
 * ```
 */

import { invokePlugin, initPlugin } from './utils';

/**
 * Interactive region data
 */
export interface InteractiveRegion {
  /** X coordinate (left edge) in pixels */
  x: number;
  /** Y coordinate (top edge) in pixels */
  y: number;
  /** Width in pixels */
  width: number;
  /** Height in pixels */
  height: number;
  /** Element ID (if available) */
  id?: string;
}

/**
 * Configuration for the interactive plugin
 */
export interface InteractiveConfig {
  /** Attribute name to look for (default: 'data-interactive') */
  attribute: string;
  /** Debounce delay in milliseconds (default: 100) */
  debounceMs: number;
  /** Whether to observe DOM changes (default: true) */
  observeChanges: boolean;
  /** Whether to observe resize events (default: true) */
  observeResize: boolean;
  /** Whether to observe scroll events (default: true) */
  observeScroll: boolean;
}

const DEFAULT_CONFIG: InteractiveConfig = {
  attribute: 'data-interactive',
  debounceMs: 100,
  observeChanges: true,
  observeResize: true,
  observeScroll: true,
};

let config: InteractiveConfig = { ...DEFAULT_CONFIG };
let enabled = true;
let observer: MutationObserver | null = null;
let resizeObserver: ResizeObserver | null = null;
let debounceTimer: ReturnType<typeof setTimeout> | null = null;
let lastRegions: InteractiveRegion[] = [];

/**
 * Collect all interactive regions from the DOM
 */
function collectRegions(): InteractiveRegion[] {
  const elements = document.querySelectorAll(`[${config.attribute}]`);
  const regions: InteractiveRegion[] = [];

  elements.forEach((element) => {
    const rect = element.getBoundingClientRect();

    // Skip invisible elements
    if (rect.width === 0 || rect.height === 0) {
      return;
    }

    // Skip elements outside viewport
    if (
      rect.bottom < 0 ||
      rect.top > window.innerHeight ||
      rect.right < 0 ||
      rect.left > window.innerWidth
    ) {
      return;
    }

    regions.push({
      x: Math.round(rect.left),
      y: Math.round(rect.top),
      width: Math.round(rect.width),
      height: Math.round(rect.height),
      id: element.id || undefined,
    });
  });

  return regions;
}

/**
 * Check if regions have changed
 */
function regionsChanged(newRegions: InteractiveRegion[]): boolean {
  if (newRegions.length !== lastRegions.length) {
    return true;
  }

  for (let i = 0; i < newRegions.length; i++) {
    const a = newRegions[i];
    const b = lastRegions[i];
    if (a.x !== b.x || a.y !== b.y || a.width !== b.width || a.height !== b.height) {
      return true;
    }
  }

  return false;
}

/**
 * Send regions to native layer
 */
async function sendRegions(regions: InteractiveRegion[]): Promise<void> {
  try {
    await invokePlugin('window', 'update_interactive_regions', { regions });
    console.debug('[AuroraView] Updated interactive regions:', regions.length);
  } catch (error) {
    console.error('[AuroraView] Failed to update interactive regions:', error);
  }
}

/**
 * Update interactive regions (debounced)
 */
function scheduleUpdate(): void {
  if (!enabled) return;

  if (debounceTimer) {
    clearTimeout(debounceTimer);
  }

  debounceTimer = setTimeout(() => {
    const regions = collectRegions();
    if (regionsChanged(regions)) {
      lastRegions = regions;
      sendRegions(regions);
    }
  }, config.debounceMs);
}

/**
 * Start observing DOM changes
 */
function startObserving(): void {
  if (!config.observeChanges) return;

  // MutationObserver for DOM changes
  observer = new MutationObserver((mutations) => {
    // Check if any mutation affects interactive elements
    let needsUpdate = false;

    for (const mutation of mutations) {
      if (mutation.type === 'childList') {
        // Check added/removed nodes
        for (const node of mutation.addedNodes) {
          if (node instanceof Element && node.hasAttribute(config.attribute)) {
            needsUpdate = true;
            break;
          }
        }
        for (const node of mutation.removedNodes) {
          if (node instanceof Element && node.hasAttribute(config.attribute)) {
            needsUpdate = true;
            break;
          }
        }
      } else if (mutation.type === 'attributes') {
        if (mutation.attributeName === config.attribute) {
          needsUpdate = true;
        }
      }

      if (needsUpdate) break;
    }

    if (needsUpdate) {
      scheduleUpdate();
    }
  });

  observer.observe(document.body, {
    childList: true,
    subtree: true,
    attributes: true,
    attributeFilter: [config.attribute],
  });

  // ResizeObserver for element size changes
  if (config.observeResize) {
    resizeObserver = new ResizeObserver(() => {
      scheduleUpdate();
    });

    // Observe all interactive elements
    document.querySelectorAll(`[${config.attribute}]`).forEach((element) => {
      resizeObserver?.observe(element);
    });
  }

  // Window resize and scroll events
  if (config.observeResize) {
    window.addEventListener('resize', scheduleUpdate, { passive: true });
  }
  if (config.observeScroll) {
    window.addEventListener('scroll', scheduleUpdate, { passive: true });
  }
}

/**
 * Stop observing DOM changes
 */
function stopObserving(): void {
  if (observer) {
    observer.disconnect();
    observer = null;
  }

  if (resizeObserver) {
    resizeObserver.disconnect();
    resizeObserver = null;
  }

  window.removeEventListener('resize', scheduleUpdate);
  window.removeEventListener('scroll', scheduleUpdate);

  if (debounceTimer) {
    clearTimeout(debounceTimer);
    debounceTimer = null;
  }
}

/**
 * Interactive regions API
 */
export const interactive = {
  /**
   * Manually trigger a region update
   */
  update(): void {
    scheduleUpdate();
  },

  /**
   * Force immediate update (bypasses debounce)
   */
  forceUpdate(): void {
    if (debounceTimer) {
      clearTimeout(debounceTimer);
      debounceTimer = null;
    }
    const regions = collectRegions();
    lastRegions = regions;
    sendRegions(regions);
  },

  /**
   * Get current interactive regions
   */
  getRegions(): InteractiveRegion[] {
    return collectRegions();
  },

  /**
   * Enable or disable tracking
   */
  setEnabled(value: boolean): void {
    enabled = value;
    if (enabled) {
      startObserving();
      scheduleUpdate();
    } else {
      stopObserving();
      // Clear regions when disabled
      lastRegions = [];
      sendRegions([]);
    }
  },

  /**
   * Check if tracking is enabled
   */
  isEnabled(): boolean {
    return enabled;
  },

  /**
   * Update configuration
   */
  configure(newConfig: Partial<InteractiveConfig>): void {
    const wasObserving = observer !== null;

    if (wasObserving) {
      stopObserving();
    }

    config = { ...config, ...newConfig };

    if (wasObserving && enabled) {
      startObserving();
      scheduleUpdate();
    }
  },

  /**
   * Get current configuration
   */
  getConfig(): InteractiveConfig {
    return { ...config };
  },

  /**
   * Initialize the plugin (called automatically)
   */
  init(): void {
    if (document.readyState === 'loading') {
      document.addEventListener('DOMContentLoaded', () => {
        startObserving();
        scheduleUpdate();
      });
    } else {
      startObserving();
      scheduleUpdate();
    }
  },

  /**
   * Cleanup the plugin
   */
  destroy(): void {
    stopObserving();
    lastRegions = [];
  },
};

// Auto-initialize when loaded
initPlugin('interactive', interactive);

// Start observing when DOM is ready
if (typeof document !== 'undefined') {
  interactive.init();
}

export default interactive;
