/**
 * Screenshot helpers for E2E tests.
 */
import type { Page } from '@playwright/test';
import path from 'path';
import fs from 'fs';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const SCREENSHOTS_DIR = path.resolve(__dirname, '..', 'screenshots');

// Ensure screenshots directory exists
if (!fs.existsSync(SCREENSHOTS_DIR)) {
  fs.mkdirSync(SCREENSHOTS_DIR, { recursive: true });
}

/**
 * Take a screenshot and save it under `tests/e2e/screenshots/<name>.png`.
 */
export async function takeScreenshot(
  page: Page,
  name: string,
  opts?: { fullPage?: boolean }
): Promise<string> {
  const screenshotPath = path.join(SCREENSHOTS_DIR, `${name}.png`);
  await page.screenshot({
    path: screenshotPath,
    fullPage: opts?.fullPage ?? false,
  });
  return screenshotPath;
}

/**
 * Take a screenshot of a specific element.
 */
export async function takeElementScreenshot(
  page: Page,
  selector: string,
  name: string
): Promise<string> {
  const screenshotPath = path.join(SCREENSHOTS_DIR, `${name}.png`);
  const element = page.locator(selector);
  await element.screenshot({ path: screenshotPath });
  return screenshotPath;
}
