import { defineConfig, devices } from '@playwright/test';

const projects = [
  {
    name: 'chromium',
    use: { ...devices['Desktop Chrome'] },
  },
];

// Optional: run the same suite against system Microsoft Edge on Windows.
// Enable via: AURORAVIEW_PLAYWRIGHT_CHANNEL=msedge
if (process.platform === 'win32' && process.env.AURORAVIEW_PLAYWRIGHT_CHANNEL === 'msedge') {
  projects.push({
    name: 'msedge',
    // Reuse Chrome device profile, but run using Edge channel.
    use: { ...devices['Desktop Chrome'], channel: 'msedge' },
  });
}

export default defineConfig({
  testDir: './tests/e2e',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: [
    ['html', { outputFolder: 'test-results/html' }],
    ['json', { outputFile: 'test-results/e2e-results.json' }],
  ],
  use: {
    baseURL: 'http://localhost:5173',
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
  },
  projects,
  webServer: {
    command: 'npm run dev:test',
    url: 'http://localhost:5173',
    reuseExistingServer: !process.env.CI,
    timeout: 120 * 1000,
  },
});

