/**
 * Sentry trigger tests for packed Gallery.
 *
 * Verifies Telemetry panel test actions:
 * - Test Sentry
 * - Test Promise Rejection
 */
import type { Page } from '@playwright/test';
import { test, expect } from '../fixtures';
import { waitForGalleryReady, createErrorCollector } from '../helpers';

function panelHintLocator(page: Page) {
  return page.locator('div').filter({
    hasText: /Sentry event:|Sentry test sent|Unhandled rejection test scheduled|Sentry 未初始化|Sentry not ready/,
  }).first();
}

async function ensureTelemetryPanelOpen(page: Page): Promise<void> {
  const sentryButton = page.getByRole('button', { name: 'Test Sentry' });
  const telemetryToggle = page
    .locator('button[title="Telemetry"], button[aria-label="Telemetry"], [data-testid="sidebar-telemetry"]')
    .first();

  if (await sentryButton.isVisible().catch(() => false)) {
    return;
  }

  await expect(telemetryToggle).toBeVisible();
  await telemetryToggle.click();

  if (!(await sentryButton.isVisible().catch(() => false))) {
    await telemetryToggle.click();
  }

  await expect(sentryButton).toBeVisible();
}

test.describe('Sentry Triggers', () => {

  test.beforeEach(async ({ page }) => {
    if (!/auroraview\.localhost/i.test(page.url())) {
      await page.goto('https://auroraview.localhost/index.html', { waitUntil: 'domcontentloaded' });
    }
    await waitForGalleryReady(page);
  });


  test('telemetry panel can trigger sentry and promise rejection actions', async ({ page }) => {
    const collector = createErrorCollector(page);

    await ensureTelemetryPanelOpen(page);

    const sentryButton = page.getByRole('button', { name: 'Test Sentry' });
    const promiseButton = page.getByRole('button', { name: 'Test Promise Rejection' });

    await expect(promiseButton).toBeVisible();


    await sentryButton.click();
    const sentryHint = panelHintLocator(page);
    await expect(sentryHint).toBeVisible();

    await promiseButton.click();
    const promiseHint = panelHintLocator(page);
    await expect(promiseHint).toBeVisible();

    await page.waitForTimeout(1200);

    // Ensure app still responsive after trigger.
    await expect(page.locator('#root')).toBeVisible();

    // If Sentry is initialized, unhandled rejection may surface as pageerror;
    // if not initialized, we expect no critical runtime break.
    const criticalErrors = collector
      .critical()
      .filter((err) => !err.includes('[gallery.sentry_test] manual unhandled rejection trigger'));
    expect(criticalErrors).toHaveLength(0);
  });
});
