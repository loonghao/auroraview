/**
 * Assertion helpers for E2E tests.
 */
import type { Page } from '@playwright/test';
import { expect } from '../fixtures';

/**
 * Assert that the page body contains at least one of the given strings.
 * Useful for i18n-agnostic checks.
 */
export async function expectBodyContainsAny(
  page: Page,
  candidates: string[]
): Promise<void> {
  const content = await page.locator('body').textContent();
  const found = candidates.some((c) => content?.includes(c));
  expect(found, `Expected body to contain one of: ${candidates.join(', ')}`).toBeTruthy();
}

/**
 * Assert the current URL contains the given substring.
 */
export async function expectUrlContains(page: Page, substring: string): Promise<void> {
  const url = page.url();
  expect(url).toContain(substring);
}

/**
 * Collect console errors from the page, ignoring known benign ones.
 * Returns only "critical" errors.
 */
export function createErrorCollector(page: Page): {
  errors: string[];
  critical: () => string[];
} {
  const errors: string[] = [];
  page.on('pageerror', (err) => errors.push(err.message));

  // Patterns for errors that are known to be benign
  const IGNORED_PATTERNS = [
    'ResizeObserver',
    'net::ERR_',
    'WebSocket',
    'Sentry', // Sentry SDK internal errors
  ];

  return {
    errors,
    critical: () =>
      errors.filter((e) => !IGNORED_PATTERNS.some((p) => e.includes(p))),
  };
}

/**
 * Wait for a specific element to be visible and contain text.
 */
export async function expectElementVisibleWithText(
  page: Page,
  selector: string,
  text: string,
  timeout = 10000
): Promise<void> {
  const element = page.locator(selector);
  await expect(element).toBeVisible({ timeout });
  await expect(element).toContainText(text, { timeout });
}
