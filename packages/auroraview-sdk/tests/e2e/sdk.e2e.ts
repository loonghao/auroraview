/**
 * SDK E2E Tests
 *
 * Tests the SDK in a real browser environment using Playwright.
 * These tests verify that the SDK works correctly when loaded in a browser.
 */

import { test, expect } from '@playwright/test';

test.describe('SDK Browser Integration', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to test page
    await page.goto('/');
  });

  test('should load SDK in browser', async ({ page }) => {
    // Check that SDK is loaded
    const sdkLoaded = await page.evaluate(() => {
      return typeof (window as any).AuroraView !== 'undefined';
    });
    expect(sdkLoaded).toBe(true);
  });

  test('should create client instance', async ({ page }) => {
    const clientCreated = await page.evaluate(() => {
      const { createAuroraView } = (window as any).AuroraView;
      const client = createAuroraView();
      return typeof client.call === 'function' && typeof client.on === 'function';
    });
    expect(clientCreated).toBe(true);
  });

  test('should handle events correctly', async ({ page }) => {
    const eventHandled = await page.evaluate(() => {
      return new Promise((resolve) => {
        const { EventEmitter } = (window as any).AuroraView;
        const emitter = new EventEmitter();
        
        let received = false;
        emitter.on('test', () => {
          received = true;
        });
        
        emitter.emit('test', { data: 'hello' });
        resolve(received);
      });
    });
    expect(eventHandled).toBe(true);
  });

  test('should support unsubscribe pattern', async ({ page }) => {
    const unsubscribeWorks = await page.evaluate(() => {
      const { EventEmitter } = (window as any).AuroraView;
      const emitter = new EventEmitter();
      
      let count = 0;
      const unsubscribe = emitter.on('test', () => {
        count++;
      });
      
      emitter.emit('test', 1);
      unsubscribe();
      emitter.emit('test', 2);
      
      return count === 1;
    });
    expect(unsubscribeWorks).toBe(true);
  });

  test('should handle once() correctly', async ({ page }) => {
    const onceWorks = await page.evaluate(() => {
      const { EventEmitter } = (window as any).AuroraView;
      const emitter = new EventEmitter();
      
      let count = 0;
      emitter.once('test', () => {
        count++;
      });
      
      emitter.emit('test', 1);
      emitter.emit('test', 2);
      emitter.emit('test', 3);
      
      return count === 1;
    });
    expect(onceWorks).toBe(true);
  });
});

test.describe('SDK Type Safety', () => {
  test('should export all required types', async ({ page }) => {
    const typesExist = await page.evaluate(() => {
      const sdk = (window as any).AuroraView;
      return (
        typeof sdk.createAuroraView === 'function' &&
        typeof sdk.EventEmitter === 'function'
      );
    });
    expect(typesExist).toBe(true);
  });
});
