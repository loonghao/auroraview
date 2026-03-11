/**
 * App Launch - basic smoke tests.
 *
 * Verifies the Gallery app opens, the UI loads, and no
 * critical console errors are thrown on startup.
 */
import { test, expect } from '../fixtures';
import { createErrorCollector, waitForGalleryReady } from '../helpers';

test.describe('App Launch', () => {
  test('window opens and has a title', async ({ page }) => {
    const title = await page.title();
    expect(title).toBeTruthy();
    // Title should contain 'Gallery' or 'AuroraView'
    expect(
      title.toLowerCase().includes('gallery') ||
      title.toLowerCase().includes('auroraview')
    ).toBeTruthy();
  });

  test('renderer loads successfully', async ({ page }) => {
    await page.waitForSelector('body', { state: 'visible' });
    const body = await page.locator('body').textContent();
    expect(body).toBeTruthy();
    expect(body!.length).toBeGreaterThan(0);
  });

  test('no uncaught console errors on load', async ({ page }) => {
    const collector = createErrorCollector(page);
    await page.waitForTimeout(2000);
    const criticalErrors = collector.critical();
    expect(criticalErrors).toHaveLength(0);
  });

  test('main UI elements are visible', async ({ page }) => {
    // Wait for Gallery to be ready
    await waitForGalleryReady(page);

    // Check for main container
    const root = page.locator('#root');
    await expect(root).toBeVisible();
  });
});
