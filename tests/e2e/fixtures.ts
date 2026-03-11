/**
 * Playwright E2E test fixtures for AuroraView Gallery.
 *
 * Connects to the Gallery app via CDP (Chrome DevTools Protocol).
 * The app can be started manually or via the gallery-e2e-setup justfile command.
 */
import { test as base, expect, type Page, type Browser } from '@playwright/test';

type Fixtures = {
  page: Page;
  browser: Browser;
};

// CDP endpoint (default port for AuroraView Gallery)
const CDP_PORT = parseInt(process.env.AURORAVIEW_CDP_PORT || '9222', 10);
const CDP_ENDPOINT = `http://127.0.0.1:${CDP_PORT}`;

/**
 * Wait for CDP endpoint to become available.
 */
async function waitForCDP(port: number, timeout = 30000): Promise<boolean> {
  const deadline = Date.now() + timeout;
  while (Date.now() < deadline) {
    try {
      const response = await fetch(`http://127.0.0.1:${port}/json/version`, {
        method: 'GET',
        signal: AbortSignal.timeout(2000),
      });
      if (response.ok) {
        return true;
      }
    } catch {
      // Ignore connection errors
    }
    await new Promise((resolve) => setTimeout(resolve, 500));
  }
  return false;
}

/**
 * Resolve the main page from browser contexts.
 * Skips DevTools pages and finds the actual Gallery page.
 */
async function isGalleryMainPage(page: Page): Promise<boolean> {
  const url = page.url();
  if (url.startsWith('devtools://') || url.startsWith('about:blank')) {
    return false;
  }

  await page.waitForLoadState('domcontentloaded').catch(() => undefined);

  const urlLooksLikeGallery = /auroraview\.localhost/i.test(url);
  const title = (await page.title().catch(() => '')).toLowerCase();
  const titleLooksLikeGallery = title.includes('auroraview gallery') || (title.includes('auroraview') && title.includes('gallery'));

  const hasGalleryHeader = await page
    .locator('h1')
    .filter({ hasText: /AuroraView Gallery/i })
    .first()
    .isVisible()
    .catch(() => false);

  const hasGalleryRoot = await page
    .locator('[data-testid="gallery-root"], #root')
    .first()
    .isVisible()
    .catch(() => false);

  return (urlLooksLikeGallery || titleLooksLikeGallery || hasGalleryHeader) && hasGalleryRoot;
}

async function resolveMainPage(browser: Browser): Promise<Page> {
  const pickFromPages = async (): Promise<Page | null> => {
    for (const context of browser.contexts()) {
      for (const page of context.pages()) {
        if (await isGalleryMainPage(page)) {
          return page;
        }
      }
    }
    return null;
  };

  const existing = await pickFromPages();
  if (existing) {
    return existing;
  }

  const deadline = Date.now() + 15000;
  while (Date.now() < deadline) {
    const page = await pickFromPages();
    if (page) {
      return page;
    }
    await new Promise((resolve) => setTimeout(resolve, 500));
  }

  throw new Error('Failed to resolve main Gallery page');
}



export const test = base.extend<Fixtures>({
  browser: async ({ playwright }, use) => {
    // Wait for CDP to be available
    console.log(`Waiting for CDP at ${CDP_ENDPOINT}...`);
    const available = await waitForCDP(CDP_PORT);
    if (!available) {
      throw new Error(
        `CDP endpoint not available at ${CDP_ENDPOINT}. ` +
        `Start Gallery first with: just gallery-e2e-packed-playwright or just gallery-e2e-start`
      );

    }

    // Connect to the running Gallery app
    const browser = await playwright.chromium.connectOverCDP(CDP_ENDPOINT);
    console.log(`Connected to Gallery via CDP`);

    await use(browser);

    // Don't close browser - it's the actual app
  },

  page: async ({ browser }, use) => {
    const page = await resolveMainPage(browser);
    await use(page);
  },
});

export { expect };
