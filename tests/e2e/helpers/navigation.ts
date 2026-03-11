/**
 * Navigation helpers for E2E tests.
 */
import type { Page } from '@playwright/test';
import { expect } from '../fixtures';

/**
 * Wait for the Gallery app to be fully loaded.
 */
export async function waitForGalleryReady(page: Page, timeout = 30000): Promise<void> {
  // Wait for the main container to be visible
  await page.waitForSelector('[data-testid="gallery-root"], #root', {
    state: 'visible',
    timeout,
  });

  // Wait for any loading spinners to disappear
  const loadingSpinner = page.locator('[data-testid="loading-spinner"]');
  const hasSpinner = await loadingSpinner.count();
  if (hasSpinner > 0) {
    await expect(loadingSpinner).not.toBeVisible({ timeout });
  }

  // Additional wait for React to hydrate
  await page.waitForTimeout(1000);
}

/**
 * Navigate to a specific section in the Gallery.
 */
export async function navigateToSection(
  page: Page,
  sectionId: string
): Promise<void> {
  const sectionLink = page.locator(`[data-section="${sectionId}"], a[href="#${sectionId}"]`);
  await sectionLink.click();
  await page.waitForTimeout(500);
}

/**
 * Open a sample in the Gallery.
 */
export async function openSample(page: Page, sampleName: string): Promise<void> {
  const sampleCard = page.locator(`[data-sample="${sampleName}"]`).first();
  await sampleCard.click();
  await page.waitForTimeout(500);
}

/**
 * Close any open modal or dialog.
 */
export async function closeModal(page: Page): Promise<void> {
  const closeButton = page.locator('[data-testid="modal-close"], button[aria-label="Close"]').first();
  const hasClose = await closeButton.count();
  if (hasClose > 0) {
    await closeButton.click();
    await page.waitForTimeout(300);
  }
}
