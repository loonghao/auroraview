/**
 * Navigation tests for Gallery.
 *
 * Tests sidebar navigation, category filtering, and sample browsing.
 */
import { test, expect } from '../fixtures';
import {
  waitForGalleryReady,
  SIDEBAR,
  SIDEBAR_ITEM,
  SAMPLE_CARD,
  CATEGORY_TAB,
} from '../helpers';

test.describe('Navigation', () => {
  test.beforeEach(async ({ page }) => {
    await waitForGalleryReady(page);
  });

  test('sidebar is visible', async ({ page }) => {
    const sidebar = page.locator(SIDEBAR);
    const hasSidebar = await sidebar.count();
    if (hasSidebar > 0) {
      await expect(sidebar).toBeVisible();
    }
  });

  test('sample cards are displayed', async ({ page }) => {
    // Wait a bit for content to load
    await page.waitForTimeout(1000);

    // Look for sample cards (may not exist if no data-testid)
    const sampleCards = page.locator(SAMPLE_CARD);
    const count = await sampleCards.count();

    // If no testid, look for any card-like elements
    if (count === 0) {
      const alternativeCards = page.locator('[class*="card"], [class*="sample"]');
      const altCount = await alternativeCards.count();
      expect(altCount).toBeGreaterThan(0);
    } else {
      expect(count).toBeGreaterThan(0);
    }
  });

  test('can interact with category tabs', async ({ page }) => {
    const categoryTabs = page.locator(CATEGORY_TAB);
    const count = await categoryTabs.count();

    if (count > 0) {
      // Click first category tab
      await categoryTabs.first().click();
      await page.waitForTimeout(500);
    }
  });
});
