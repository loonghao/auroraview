/**
 * Playwright E2E test configuration for AuroraView Gallery.
 *
 * Tests connect to the Gallery app via CDP (Chrome DevTools Protocol).
 * The app must be started separately (manually or via justfile commands).
 */
import { defineConfig, devices } from '@playwright/test';

const isCI = !!process.env.CI;
const cdpBaseUrl = process.env.AURORAVIEW_E2E_CDP_URL ?? 'http://127.0.0.1:9222';

export default defineConfig({
  testDir: './specs',
  testMatch: /.*\.(e2e|test)\.ts$/,
  fullyParallel: false, // Run tests sequentially (shared app instance)
  forbidOnly: isCI,
  retries: isCI ? 2 : 0,
  workers: 1, // Single worker for shared app instance
  timeout: 60_000,
  expect: {
    timeout: 10_000,
  },
  outputDir: './test-results',
  reporter: [
    ['list'],
    ['html', { outputFolder: './report', open: 'never' }],
    ['json', { outputFile: './report/results.json' }],
  ],
  use: {
    baseURL: cdpBaseUrl,
    trace: isCI ? 'on-first-retry' : 'retain-on-failure',
    screenshot: 'only-on-failure',
    video: 'retain-on-failure',
    actionTimeout: 15_000,
    navigationTimeout: 30_000,
  },
  projects: [
    {
      name: 'gallery-cdp',
      use: {
        ...devices['Desktop Chrome'],
      },
    },
  ],
  // No webServer - the app is started separately
});
