# AuroraView Gallery E2E Tests

This directory contains end-to-end tests for AuroraView Gallery using Playwright and Chrome DevTools Protocol (CDP).

## Architecture

Tests connect to a running Gallery instance via CDP (Chrome DevTools Protocol). This approach:

- Tests the actual packed application (not a development server)
- Provides real browser automation capabilities
- Supports CI/CD integration
- Allows debugging with headed browser mode

## Quick Start

### Prerequisites

1. Install dependencies:
   ```bash
   cd tests/e2e
   npm install
   npm run install:playwright
   ```

### Running Tests

#### Local Development

1. Build and start Gallery with CDP enabled:
   ```bash
   just gallery-e2e-start
   ```

2. Run tests:
   ```bash
   # Using justfile (recommended)
   just gallery-e2e-test

   # Or directly
   cd tests/e2e
   npm test
   ```

3. Stop Gallery when done:
   ```bash
   just gallery-e2e-stop
   ```

#### Debugging

Run tests in headed mode to see the browser:
```bash
just gallery-e2e-test-headed

# Or
cd tests/e2e
npm run test:headed
```

#### View Test Report

```bash
just gallery-e2e-report

# Or
cd tests/e2e
npm run report
```

## Test Structure

```
tests/e2e/
├── fixtures.ts          # Playwright fixtures (CDP connection)
├── playwright.config.ts # Playwright configuration
├── helpers/             # Test utilities
│   ├── assertions.ts    # Custom assertions
│   ├── navigation.ts    # Navigation helpers
│   ├── screenshots.ts   # Screenshot utilities
│   ├── selectors.ts     # CSS selectors
│   └── index.ts         # Exports
├── specs/               # Test specifications
│   ├── app-launch.e2e.ts    # Basic smoke tests
│   ├── navigation.e2e.ts    # Navigation tests
│   └── features.e2e.ts      # Feature tests
├── screenshots/         # Test screenshots (gitignored)
└── report/              # HTML test report (gitignored)
```

## Writing Tests

### Basic Test

```typescript
import { test, expect } from '../fixtures';
import { waitForGalleryReady } from '../helpers';

test('my test', async ({ page }) => {
  await waitForGalleryReady(page);
  // Your assertions here
  expect(await page.title()).toContain('Gallery');
});
```

### Using Helpers

```typescript
import { test, expect } from '../fixtures';
import {
  waitForGalleryReady,
  createErrorCollector,
  takeScreenshot,
} from '../helpers';

test('check for errors', async ({ page }) => {
  const collector = createErrorCollector(page);
  await waitForGalleryReady(page);
  await takeScreenshot(page, 'my-test');

  expect(collector.critical()).toHaveLength(0);
});
```

## CI Integration

E2E tests are automatically run in CI on Windows (representative platform):

1. Gallery is built and packed
2. Gallery starts with CDP enabled
3. Tests run via Playwright
4. Results are uploaded as artifacts

See `.github/workflows/build-gallery.yml` for details.

## Configuration

### CDP Port

Default CDP port is `9222`. To use a different port:

```bash
export AURORAVIEW_CDP_PORT=9333
just gallery-e2e-test
```

### Environment Variables

- `AURORAVIEW_CDP_PORT`: CDP port (default: 9222)

## Troubleshooting

### CDP Connection Failed

```
Error: CDP endpoint not available at http://127.0.0.1:9222
```

**Solution**: Ensure Gallery is running with CDP enabled:
```bash
just gallery-e2e-start
```

### Tests Timeout

If tests timeout, increase the timeout in `playwright.config.ts`:

```typescript
export default defineConfig({
  timeout: 60000, // 60 seconds per test
  // ...
});
```

### Screenshots Not Captured

Screenshots are only captured on test failure by default. To always capture:

```typescript
await takeScreenshot(page, 'my-screenshot');
```

## Related Commands

```bash
# Full E2E workflow (build, start, test, stop)
just gallery-e2e

# Install Playwright
just gallery-e2e-install

# Start Gallery with CDP
just gallery-e2e-start

# Stop Gallery
just gallery-e2e-stop

# Run tests
just gallery-e2e-test

# Run tests in headed mode
just gallery-e2e-test-headed

# Open test report
just gallery-e2e-report

# Generate docs screenshots with Playwright
just gallery-e2e-screenshots
```

## Resources

- [Playwright Documentation](https://playwright.dev/)
- [Chrome DevTools Protocol](https://chromedevtools.github.io/devtools-protocol/)
- [AuroraView Documentation](../../docs/)
