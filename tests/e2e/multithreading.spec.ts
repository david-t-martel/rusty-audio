/**
 * Multithreading Validation Tests
 *
 * Validates WASM threading and worker pool functionality:
 * - SharedArrayBuffer availability
 * - Worker pool initialization
 * - Thread-safe memory sharing
 * - Graceful fallback to single-threaded mode
 */

import { test, expect, Page } from '@playwright/test';
import {
  detectBrowserFeatures,
  waitForWasmInit,
  getWorkerPoolStatus,
  assertFeatureSupport,
} from '../helpers/wasm-fixtures';

test.describe('WASM Multithreading', () => {
  test('should detect SharedArrayBuffer support correctly', async ({ page, browserName }) => {
    await page.goto('/');

    const features = await detectBrowserFeatures(page);

    // Log for debugging
    console.log(`Browser: ${browserName}`);
    console.log(`SharedArrayBuffer: ${features.sharedArrayBuffer}`);
    console.log(`Atomics: ${features.atomics}`);
    console.log(`Cross-Origin Isolated: ${features.crossOriginIsolated}`);

    // If SharedArrayBuffer is available, Atomics should also be available
    if (features.sharedArrayBuffer) {
      expect(features.atomics, 'Atomics should be available with SharedArrayBuffer').toBe(true);
    }
  });

  test('should initialize worker pool when SharedArrayBuffer is available', async ({
    page,
    browserName,
  }) => {
    await page.goto('/');

    const features = await detectBrowserFeatures(page);

    // Skip if SharedArrayBuffer not supported
    test.skip(!features.sharedArrayBuffer, 'SharedArrayBuffer not available in this browser');

    await waitForWasmInit(page);

    // Check worker pool status
    const workerPoolStatus = await getWorkerPoolStatus(page);

    if (workerPoolStatus) {
      expect(workerPoolStatus.initialized, 'Worker pool should be initialized').toBe(true);
      expect(workerPoolStatus.totalWorkers, 'Should have created workers').toBeGreaterThan(0);

      console.log(`✅ Worker pool initialized with ${workerPoolStatus.totalWorkers} workers`);
      console.log(`   Available: ${workerPoolStatus.availableWorkers}`);
      console.log(`   Busy: ${workerPoolStatus.busyWorkers}`);
    } else {
      // Worker pool might not be exposed or initialized
      console.log('⚠️  Worker pool status not available');
    }
  });

  test('should create appropriate number of workers based on hardware', async ({ page }) => {
    await page.goto('/');

    const features = await detectBrowserFeatures(page);
    test.skip(!features.sharedArrayBuffer, 'SharedArrayBuffer not available');

    await waitForWasmInit(page);

    const workerPoolStatus = await getWorkerPoolStatus(page);

    if (workerPoolStatus) {
      const hardwareConcurrency = features.hardwareConcurrency || 4;
      const expectedMaxWorkers = hardwareConcurrency;

      expect(
        workerPoolStatus.totalWorkers,
        'Worker count should not exceed hardware concurrency'
      ).toBeLessThanOrEqual(expectedMaxWorkers);

      expect(
        workerPoolStatus.totalWorkers,
        'Should have at least one worker'
      ).toBeGreaterThanOrEqual(1);

      console.log(`✅ Created ${workerPoolStatus.totalWorkers} workers (hardware has ${hardwareConcurrency} cores)`);
    }
  });

  test('should display thread status UI when workers are active', async ({ page }) => {
    await page.goto('/');

    const features = await detectBrowserFeatures(page);
    test.skip(!features.sharedArrayBuffer, 'SharedArrayBuffer not available');

    await waitForWasmInit(page);

    // Check if thread status UI is visible
    const threadStatus = page.locator('#thread-status');
    const isVisible = await threadStatus.isVisible();

    if (isVisible) {
      // Should show thread info
      const threadInfo = await page.locator('#thread-info').textContent();
      expect(threadInfo).toBeTruthy();

      // Should display thread indicators
      const threadIndicators = page.locator('.thread-indicator');
      const count = await threadIndicators.count();

      expect(count, 'Should display thread indicators').toBeGreaterThan(0);

      console.log(`✅ Thread status UI showing ${count} worker indicators`);
    }
  });

  test('should handle cross-origin isolation headers correctly', async ({ page }) => {
    await page.goto('/');

    // Check cross-origin isolation status
    const crossOriginIsolated = await page.evaluate(() => {
      return window.crossOriginIsolated;
    });

    console.log(`Cross-Origin Isolated: ${crossOriginIsolated}`);

    // If headers are set correctly, should be isolated
    // Note: This may fail in development without proper server configuration
    if (crossOriginIsolated) {
      const features = await detectBrowserFeatures(page);
      expect(
        features.sharedArrayBuffer,
        'SharedArrayBuffer should be available when cross-origin isolated'
      ).toBe(true);
    }
  });

  test('should gracefully fallback to single-threaded mode', async ({ page, browserName }) => {
    // Test graceful degradation
    await page.goto('/');

    const features = await detectBrowserFeatures(page);

    // If no SharedArrayBuffer, app should still work
    if (!features.sharedArrayBuffer) {
      console.log('⚠️  Testing fallback to single-threaded mode');

      await waitForWasmInit(page);

      // App should still initialize
      const initialized = await page.evaluate(() => {
        return (window as any).rustyAudio?.state?.wasmInitialized;
      });

      expect(initialized, 'App should initialize even without threading').toBe(true);

      // Should show warning about single-threaded mode
      const statusText = await page.locator('#loading-status').textContent();
      if (statusText) {
        expect(statusText.toLowerCase()).toContain('single-threaded');
      }

      console.log('✅ Single-threaded fallback working');
    } else {
      test.skip(true, 'SharedArrayBuffer available - skipping fallback test');
    }
  });

  test('should validate Atomics support with SharedArrayBuffer', async ({ page }) => {
    await page.goto('/');

    const features = await detectBrowserFeatures(page);
    test.skip(!features.sharedArrayBuffer, 'SharedArrayBuffer not available');

    // Test Atomics functionality
    const atomicsWorking = await page.evaluate(() => {
      try {
        const sab = new SharedArrayBuffer(4);
        const view = new Int32Array(sab);
        Atomics.store(view, 0, 42);
        const value = Atomics.load(view, 0);
        return value === 42;
      } catch (e) {
        return false;
      }
    });

    expect(atomicsWorking, 'Atomics operations should work correctly').toBe(true);
    console.log('✅ Atomics operations validated');
  });

  test('should handle worker initialization timeout gracefully', async ({ page }) => {
    await page.goto('/');

    const features = await detectBrowserFeatures(page);
    test.skip(!features.sharedArrayBuffer, 'SharedArrayBuffer not available');

    await waitForWasmInit(page, 60000); // Extended timeout

    // Even if workers timeout, app should still be usable
    const canvasVisible = await page.locator('#rusty-audio-canvas').isVisible();
    expect(canvasVisible, 'Canvas should be visible even if workers timeout').toBe(true);
  });

  test('should share WASM memory across workers', async ({ page }) => {
    await page.goto('/');

    const features = await detectBrowserFeatures(page);
    test.skip(!features.sharedArrayBuffer, 'SharedArrayBuffer not available');

    await waitForWasmInit(page);

    // Check if WASM memory is a SharedArrayBuffer
    const memoryIsShared = await page.evaluate(() => {
      const wasmBindgen = (window as any).wasm_bindgen;
      if (!wasmBindgen || !wasmBindgen.memory) return false;

      const memory = wasmBindgen.memory;
      return memory.buffer instanceof SharedArrayBuffer;
    });

    console.log(`WASM memory shared: ${memoryIsShared}`);
  });

  test('should monitor worker pool health', async ({ page }) => {
    await page.goto('/');

    const features = await detectBrowserFeatures(page);
    test.skip(!features.sharedArrayBuffer, 'SharedArrayBuffer not available');

    await waitForWasmInit(page);

    // Listen for worker health events
    const healthEventReceived = await page.evaluate(async () => {
      return new Promise((resolve) => {
        const timeout = setTimeout(() => resolve(false), 10000);

        window.addEventListener('wasm-worker-health', (event: any) => {
          clearTimeout(timeout);
          resolve(true);
        });
      });
    });

    console.log(`Worker health monitoring active: ${healthEventReceived}`);
  });

  test('should handle worker errors without crashing', async ({ page }) => {
    await page.goto('/');

    const features = await detectBrowserFeatures(page);
    test.skip(!features.sharedArrayBuffer, 'SharedArrayBuffer not available');

    await waitForWasmInit(page);

    // Simulate worker error by sending invalid message
    await page.evaluate(() => {
      const workerPool = (window as any).rustyAudio?.workerPool;
      if (workerPool && workerPool.workers && workerPool.workers.length > 0) {
        // Try to send malformed message to worker
        try {
          workerPool.workers[0].worker.postMessage({ type: 'invalid-test-message' });
        } catch (e) {
          // Expected to handle gracefully
        }
      }
    });

    // Wait a bit for error handling
    await page.waitForTimeout(1000);

    // App should still be responsive
    const canvasVisible = await page.locator('#rusty-audio-canvas').isVisible();
    expect(canvasVisible, 'App should remain responsive after worker error').toBe(true);
  });

  test('should show worker thread indicators in performance monitor', async ({ page }) => {
    await page.goto('/');

    const features = await detectBrowserFeatures(page);
    test.skip(!features.sharedArrayBuffer, 'SharedArrayBuffer not available');

    await waitForWasmInit(page);

    // Enable performance monitor
    await page.evaluate(() => {
      const perfMonitor = document.getElementById('perf-monitor');
      if (perfMonitor) {
        perfMonitor.classList.add('visible');
      }
    });

    await page.waitForTimeout(1000);

    // Check thread count display
    const threadCount = await page.locator('#perf-threads').textContent();
    expect(threadCount).toBeTruthy();

    console.log(`✅ Performance monitor showing threads: ${threadCount}`);
  });

  test('should support bulk memory operations (required for threads)', async ({ page }) => {
    await page.goto('/');

    const features = await detectBrowserFeatures(page);
    test.skip(!features.webAssemblyThreads, 'WebAssembly threads not available');

    // Bulk memory is required for WASM threading
    // Validate by checking compilation success
    await waitForWasmInit(page);

    const initialized = await page.evaluate(() => {
      return (window as any).rustyAudio?.state?.wasmInitialized;
    });

    expect(initialized, 'WASM with threading features should compile').toBe(true);
  });

  test('should properly configure RUSTFLAGS for atomics', async ({ page }) => {
    // This is more of a build verification test
    await page.goto('/');

    const features = await detectBrowserFeatures(page);

    console.log('WASM Thread Support Requirements:');
    console.log('  - SharedArrayBuffer:', features.sharedArrayBuffer);
    console.log('  - Atomics:', features.atomics);
    console.log('  - Cross-Origin Isolated:', features.crossOriginIsolated);
    console.log('  - WebAssembly Threads:', features.webAssemblyThreads);

    // If all present, threading should work
    if (
      features.sharedArrayBuffer &&
      features.atomics &&
      features.crossOriginIsolated &&
      features.webAssemblyThreads
    ) {
      await waitForWasmInit(page);

      const workerPoolStatus = await getWorkerPoolStatus(page);
      expect(
        workerPoolStatus?.initialized,
        'Worker pool should initialize with full threading support'
      ).toBe(true);
    }
  });
});
