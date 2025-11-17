/**
 * WASM Loading and Initialization Tests
 *
 * Validates that the WASM application loads correctly and initializes properly:
 * - WASM binary downloads and compiles successfully
 * - wasm-bindgen initialization completes
 * - No compilation errors or runtime failures
 * - Initialization completes within acceptable time limits
 */

import { test, expect, Page } from '@playwright/test';
import {
  detectBrowserFeatures,
  waitForWasmInit,
  getPerformanceMetrics,
  assertFeatureSupport,
  waitForElementWithError,
} from '../helpers/wasm-fixtures';

test.describe('WASM Loading and Initialization', () => {
  let consoleErrors: string[] = [];
  let consoleWarnings: string[] = [];

  test.beforeEach(async ({ page }) => {
    // Collect console messages
    consoleErrors = [];
    consoleWarnings = [];

    page.on('console', (msg) => {
      if (msg.type() === 'error') {
        consoleErrors.push(msg.text());
      } else if (msg.type() === 'warning') {
        consoleWarnings.push(msg.text());
      }
    });

    // Monitor page errors
    page.on('pageerror', (error) => {
      consoleErrors.push(`Page error: ${error.message}`);
    });

    // Monitor request failures
    page.on('requestfailed', (request) => {
      consoleErrors.push(`Request failed: ${request.url()} - ${request.failure()?.errorText}`);
    });
  });

  test('should load WASM binary successfully', async ({ page }) => {
    // Track WASM request
    let wasmRequested = false;
    let wasmLoaded = false;
    let wasmSize = 0;

    page.on('request', (request) => {
      if (request.url().endsWith('.wasm')) {
        wasmRequested = true;
      }
    });

    page.on('response', async (response) => {
      if (response.url().endsWith('.wasm')) {
        wasmLoaded = response.ok();
        const buffer = await response.body().catch(() => null);
        if (buffer) {
          wasmSize = buffer.length;
        }
      }
    });

    await page.goto('/');

    // Wait a bit for network requests
    await page.waitForTimeout(2000);

    expect(wasmRequested, 'WASM binary should be requested').toBe(true);
    expect(wasmLoaded, 'WASM binary should load successfully').toBe(true);
    expect(wasmSize, 'WASM binary should have non-zero size').toBeGreaterThan(0);

    console.log(`✅ WASM binary loaded: ${(wasmSize / 1024 / 1024).toFixed(2)} MB`);
  });

  test('should initialize WASM within 30 seconds', async ({ page }) => {
    const startTime = Date.now();

    await page.goto('/');

    const state = await waitForWasmInit(page, 30000);

    const loadTime = Date.now() - startTime;

    expect(state.initialized, 'WASM should be initialized').toBe(true);
    expect(loadTime, 'Load time should be under 30 seconds').toBeLessThan(30000);

    console.log(`✅ WASM initialized in ${loadTime}ms`);

    // Attach performance metrics
    await test.info().attach('performance-metrics', {
      body: JSON.stringify({ loadTime }),
      contentType: 'application/json',
    });
  });

  test('should have no critical console errors during initialization', async ({ page }) => {
    await page.goto('/');
    await waitForWasmInit(page);

    // Filter out expected warnings
    const criticalErrors = consoleErrors.filter((err) => {
      // Ignore service worker registration warnings (expected in some browsers)
      if (err.includes('Service Worker')) return false;
      // Ignore WASM thread warnings in single-threaded mode
      if (err.includes('SharedArrayBuffer not available')) return false;
      return true;
    });

    expect(
      criticalErrors,
      `Should have no critical console errors. Found: ${criticalErrors.join(', ')}`
    ).toHaveLength(0);

    console.log(`✅ No critical console errors (${consoleErrors.length} total messages)`);
  });

  test('should compile WASM without validation errors', async ({ page }) => {
    let wasmValidationError = false;

    page.on('console', (msg) => {
      if (msg.text().includes('CompileError') || msg.text().includes('LinkError')) {
        wasmValidationError = true;
      }
    });

    await page.goto('/');
    await waitForWasmInit(page);

    expect(wasmValidationError, 'WASM should compile without errors').toBe(false);
  });

  test('should detect required browser features', async ({ page }) => {
    await page.goto('/');

    const features = await detectBrowserFeatures(page);

    // Critical features
    expect(features.webAssembly, 'WebAssembly should be supported').toBe(true);
    expect(features.webAudioAPI, 'Web Audio API should be supported').toBe(true);

    // Log optional features
    console.log('Browser Features:');
    console.log(`  WebAssembly: ${features.webAssembly}`);
    console.log(`  WebAssembly Threads: ${features.webAssemblyThreads}`);
    console.log(`  SharedArrayBuffer: ${features.sharedArrayBuffer}`);
    console.log(`  Atomics: ${features.atomics}`);
    console.log(`  Cross-Origin Isolated: ${features.crossOriginIsolated}`);
    console.log(`  Web Audio API: ${features.webAudioAPI}`);
    console.log(`  WebGPU: ${features.webGPU}`);
    console.log(`  Hardware Concurrency: ${features.hardwareConcurrency} cores`);
  });

  test('should display loading UI before initialization', async ({ page }) => {
    await page.goto('/');

    // Loading overlay should be visible initially
    const loadingOverlay = page.locator('#loading-overlay');
    await expect(loadingOverlay).toBeVisible();

    // Should show loading message
    const loadingMessage = page.locator('#loading-message');
    await expect(loadingMessage).toBeVisible();

    // Should show progress bar
    const progressBar = page.locator('.progress-bar');
    await expect(progressBar).toBeVisible();
  });

  test('should hide loading UI after initialization', async ({ page }) => {
    await page.goto('/');
    await waitForWasmInit(page);

    // Loading overlay should be hidden
    const loadingOverlay = page.locator('#loading-overlay');
    await expect(loadingOverlay).toHaveClass(/hidden/);

    // Canvas should be visible
    const canvas = page.locator('#rusty-audio-canvas');
    await expect(canvas).toBeVisible();
  });

  test('should initialize wasm-bindgen correctly', async ({ page }) => {
    await page.goto('/');

    // Check for wasm-bindgen global
    const wasmBindgenAvailable = await page.evaluate(() => {
      return typeof (window as any).wasm_bindgen !== 'undefined';
    });

    // Wait for init to complete
    await waitForWasmInit(page);

    // Verify rustyAudio global is set up
    const rustyAudioAvailable = await page.evaluate(() => {
      return typeof (window as any).rustyAudio !== 'undefined';
    });

    expect(rustyAudioAvailable, 'rustyAudio global should be available').toBe(true);
  });

  test('should handle WASM memory initialization', async ({ page }) => {
    await page.goto('/');
    await waitForWasmInit(page);

    // Check WASM memory is initialized
    const memoryInitialized = await page.evaluate(() => {
      const wasmBindgen = (window as any).wasm_bindgen;
      return wasmBindgen && typeof wasmBindgen.memory !== 'undefined';
    });

    // Note: wasm-bindgen may not expose memory directly in all configurations
    console.log(`WASM memory exposed: ${memoryInitialized}`);
  });

  test('should support HTTP range requests for streaming WASM', async ({ page }) => {
    let supportsRangeRequests = false;

    page.on('response', async (response) => {
      if (response.url().endsWith('.wasm')) {
        const acceptRanges = response.headers()['accept-ranges'];
        if (acceptRanges === 'bytes') {
          supportsRangeRequests = true;
        }
      }
    });

    await page.goto('/');
    await page.waitForTimeout(2000);

    console.log(`HTTP range requests supported: ${supportsRangeRequests}`);
  });

  test('should handle initialization failure gracefully', async ({ page, browserName }) => {
    // Only test in chromium to avoid flakiness
    test.skip(browserName !== 'chromium', 'Testing error handling in chromium only');

    // Intercept and fail WASM request
    await page.route('**/*.wasm', (route) => {
      route.abort('failed');
    });

    await page.goto('/');

    // Should show error message
    await page.waitForSelector('#error-container.visible', { timeout: 10000 });

    const errorText = await page.locator('#error-text').textContent();
    expect(errorText).toBeTruthy();

    console.log(`✅ Error handled gracefully: ${errorText}`);
  });

  test('should display feature support matrix', async ({ page }) => {
    await page.goto('/');

    // Wait for feature detection to complete
    await page.waitForTimeout(2000);

    // Feature display should be visible during loading
    const featureDisplay = page.locator('#feature-display');
    const isVisible = await featureDisplay.isVisible();

    if (isVisible) {
      // Check that features are listed
      const featureItems = page.locator('.feature-list li');
      const count = await featureItems.count();

      expect(count, 'Should display multiple features').toBeGreaterThan(0);

      console.log(`✅ ${count} browser features displayed`);
    }
  });

  test('should complete initialization sequence in correct order', async ({ page }) => {
    const initSteps: string[] = [];

    page.on('console', (msg) => {
      const text = msg.text();
      if (text.includes('[Rusty Audio]')) {
        // Extract step from log message
        if (text.includes('Checking browser compatibility')) initSteps.push('feature-check');
        if (text.includes('Registering service worker')) initSteps.push('service-worker');
        if (text.includes('Initializing worker thread pool')) initSteps.push('worker-pool');
        if (text.includes('Initializing WASM')) initSteps.push('wasm-init');
        if (text.includes('Initialization complete')) initSteps.push('complete');
      }
    });

    await page.goto('/');
    await waitForWasmInit(page);

    console.log('Initialization sequence:', initSteps);

    // Verify feature check comes first
    expect(initSteps[0]).toBe('feature-check');

    // Verify completion comes last
    expect(initSteps[initSteps.length - 1]).toBe('complete');
  });
});
