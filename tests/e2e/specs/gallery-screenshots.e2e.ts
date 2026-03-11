/**
 * Gallery 文档截图（Playwright + CDP）
 *
 * 仅在 AURORAVIEW_SCREENSHOTS=1 时执行，避免影响常规 E2E 流程。
 */
import path from 'node:path';
import { mkdir } from 'node:fs/promises';
import type { Page } from '@playwright/test';
import { test } from '../fixtures';
import { waitForGalleryReady } from '../helpers';


const OUTPUT_DIR = path.resolve(process.cwd(), '..', '..', 'docs', 'public', 'gallery');
const SCREENSHOT_WAIT = 1200;

async function reloadAndReady(page: Page) {

  await page.reload({ waitUntil: 'domcontentloaded' });
  await waitForGalleryReady(page);
  await page.waitForTimeout(600);
}

async function clickAndCapture(
  page: Page,

  selector: string,
  outputName: string
) {
  await reloadAndReady(page);
  const button = page.locator(selector).first();
  await button.waitFor({ state: 'visible', timeout: 10000 });
  await button.click({ timeout: 10000 });
  await page.waitForTimeout(SCREENSHOT_WAIT);
  await page.screenshot({
    path: path.join(OUTPUT_DIR, `${outputName}.png`),
    fullPage: false,
  });
}

test.describe('Gallery Docs Screenshots', () => {
  test.skip(
    process.env.AURORAVIEW_SCREENSHOTS !== '1',
    'Set AURORAVIEW_SCREENSHOTS=1 to enable docs screenshot capture'
  );

  test('capture docs/public/gallery images', async ({ page }) => {
    await mkdir(OUTPUT_DIR, { recursive: true });

    await page.setViewportSize({ width: 1440, height: 900 });
    await reloadAndReady(page);

    // main.png
    await page.screenshot({
      path: path.join(OUTPUT_DIR, 'main.png'),
      fullPage: false,
    });

    // getting_started.png
    await clickAndCapture(page, 'button[title*="Getting Started"]', 'getting_started');

    // api_patterns.png
    await clickAndCapture(page, 'button[title*="API Patterns"]', 'api_patterns');

    // window_features.png
    await clickAndCapture(page, 'button[title*="Window Features"]', 'window_features');

    // settings.png
    await clickAndCapture(
      page,
      'button[title="Settings"], button[aria-label="Settings"]',
      'settings'
    );
  });

});
