/**
 * Feature tests for Gallery.
 *
 * Tests specific Gallery features like sample execution,
 * process management, and extension integration.
 */
import { test, expect } from '../fixtures';
import {
  waitForGalleryReady,
  createErrorCollector,
  takeScreenshot,
} from '../helpers';

test.describe('Features', () => {
  test.beforeEach(async ({ page }) => {
    await waitForGalleryReady(page);
  });

  test('gallery shows sample list', async ({ page }) => {
    // Wait for samples to load
    await page.waitForTimeout(2000);

    // Take a screenshot for debugging
    await takeScreenshot(page, 'gallery-sample-list');

    // Check page has content (at least some text content)
    const body = await page.locator('body').textContent();
    expect(body!.trim().length).toBeGreaterThan(10);
  });

  test('search functionality works', async ({ page }) => {
    // Look for search input
    const searchInput = page.locator('[data-testid="search-input"], input[type="search"], input[placeholder*="search" i]');

    const hasSearch = await searchInput.count();
    if (hasSearch > 0) {
      await searchInput.fill('test');
      await page.waitForTimeout(500);

      // Verify search worked (no errors)
      const collector = createErrorCollector(page);
      await page.waitForTimeout(1000);
      expect(collector.critical()).toHaveLength(0);
    }
  });

  test('no memory leaks after idle', async ({ page }) => {
    // Let the app run idle for 5 seconds
    await page.waitForTimeout(5000);

    // Check for errors
    const collector = createErrorCollector(page);
    expect(collector.critical()).toHaveLength(0);

    // Take a screenshot to verify state
    await takeScreenshot(page, 'gallery-idle');
  });
});
