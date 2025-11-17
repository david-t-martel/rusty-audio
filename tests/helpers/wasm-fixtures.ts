/**
 * WASM Testing Fixtures and Utilities
 *
 * Reusable fixtures for WASM application testing, including:
 * - Feature detection helpers
 * - WASM loading utilities
 * - Performance measurement utilities
 * - Audio context helpers
 */

import { Page, expect } from '@playwright/test';

/**
 * Browser feature support detection
 */
export interface BrowserFeatures {
  webAssembly: boolean;
  webAssemblyThreads: boolean;
  sharedArrayBuffer: boolean;
  atomics: boolean;
  crossOriginIsolated: boolean;
  serviceWorker: boolean;
  webAudioAPI: boolean;
  webGPU: boolean;
  offscreenCanvas: boolean;
  hardwareConcurrency: number;
}

/**
 * WASM initialization state
 */
export interface WasmState {
  initialized: boolean;
  loadTime: number;
  features: BrowserFeatures;
  workerPoolActive: boolean;
  workerCount: number;
  errors: string[];
}

/**
 * Performance metrics
 */
export interface PerformanceMetrics {
  fps: number;
  frameTime: number;
  memory: number;
  audioLatency: number;
  wasmLoadTime: number;
  initializationTime: number;
}

/**
 * Worker pool status
 */
export interface WorkerPoolStatus {
  totalWorkers: number;
  availableWorkers: number;
  busyWorkers: number;
  pendingTasks: number;
  totalTasks: number;
  initialized: boolean;
}

/**
 * Detect browser features
 */
export async function detectBrowserFeatures(page: Page): Promise<BrowserFeatures> {
  return await page.evaluate(() => {
    return {
      webAssembly: typeof WebAssembly !== 'undefined',
      webAssemblyThreads: (function() {
        try {
          const test = new Uint8Array([0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00]);
          return WebAssembly.validate(test);
        } catch (e) {
          return false;
        }
      })(),
      sharedArrayBuffer: typeof SharedArrayBuffer !== 'undefined',
      atomics: typeof Atomics !== 'undefined',
      crossOriginIsolated: window.crossOriginIsolated === true,
      serviceWorker: 'serviceWorker' in navigator,
      webAudioAPI: typeof AudioContext !== 'undefined' || typeof (window as any).webkitAudioContext !== 'undefined',
      webGPU: 'gpu' in navigator,
      offscreenCanvas: typeof OffscreenCanvas !== 'undefined',
      hardwareConcurrency: navigator.hardwareConcurrency || 0,
    };
  });
}

/**
 * Wait for WASM to initialize with timeout
 */
export async function waitForWasmInit(
  page: Page,
  timeout: number = 30000
): Promise<WasmState> {
  const startTime = Date.now();

  try {
    // Wait for loading overlay to hide
    await page.waitForSelector('#loading-overlay.hidden', { timeout });

    // Wait for canvas to be visible
    await page.waitForSelector('#rusty-audio-canvas', { state: 'visible', timeout: 5000 });

    const loadTime = Date.now() - startTime;

    // Get initialization state
    const state = await page.evaluate(() => {
      const rustyAudio = (window as any).rustyAudio;
      const workerPool = rustyAudio?.workerPool;

      return {
        initialized: rustyAudio?.state?.wasmInitialized || false,
        features: rustyAudio?.state?.features || {},
        workerPoolActive: workerPool?.initialized || false,
        workerCount: workerPool?.workers?.length || 0,
        errors: [],
      };
    });

    return {
      ...state,
      loadTime,
    };
  } catch (error) {
    // Check for error messages
    const errorVisible = await page.locator('#error-container.visible').isVisible();
    const errorText = errorVisible
      ? await page.locator('#error-text').textContent()
      : null;

    throw new Error(
      `WASM initialization failed after ${Date.now() - startTime}ms: ${errorText || error}`
    );
  }
}

/**
 * Get current performance metrics
 */
export async function getPerformanceMetrics(page: Page): Promise<PerformanceMetrics> {
  return await page.evaluate(() => {
    const rustyAudio = (window as any).rustyAudio;
    const perfStats = rustyAudio?.state?.perfStats || {};

    return {
      fps: perfStats.fps || 0,
      frameTime: perfStats.frameTime || 0,
      memory: perfStats.memory || 0,
      audioLatency: perfStats.audioLatency || 0,
      wasmLoadTime: Date.now() - rustyAudio?.state?.startTime || 0,
      initializationTime: Date.now() - rustyAudio?.state?.startTime || 0,
    };
  });
}

/**
 * Get worker pool status
 */
export async function getWorkerPoolStatus(page: Page): Promise<WorkerPoolStatus | null> {
  return await page.evaluate(() => {
    const workerPool = (window as any).rustyAudio?.workerPool;
    if (!workerPool || !workerPool.initialized) {
      return null;
    }

    return workerPool.getStats();
  });
}

/**
 * Wait for performance to stabilize (FPS >= target for duration)
 */
export async function waitForPerformanceStable(
  page: Page,
  targetFps: number = 55,
  duration: number = 2000
): Promise<void> {
  const startTime = Date.now();
  const checkInterval = 500;

  while (Date.now() - startTime < duration) {
    const metrics = await getPerformanceMetrics(page);

    if (metrics.fps < targetFps) {
      // Reset timer if FPS drops
      await page.waitForTimeout(checkInterval);
      continue;
    }

    await page.waitForTimeout(checkInterval);
  }
}

/**
 * Check for console errors
 */
export async function getConsoleErrors(page: Page): Promise<string[]> {
  // This should be collected via page.on('console') listeners
  // Implementation depends on test setup
  return [];
}

/**
 * Inject performance monitoring script
 */
export async function injectPerformanceMonitor(page: Page): Promise<void> {
  await page.evaluate(() => {
    // Enable performance monitor
    const perfMonitor = document.getElementById('perf-monitor');
    if (perfMonitor) {
      perfMonitor.classList.add('visible');
    }
  });
}

/**
 * Take performance snapshot with all metrics
 */
export async function takePerformanceSnapshot(page: Page): Promise<{
  metrics: PerformanceMetrics;
  workerPool: WorkerPoolStatus | null;
  memory: any;
  timing: any;
}> {
  const metrics = await getPerformanceMetrics(page);
  const workerPool = await getWorkerPoolStatus(page);

  const { memory, timing } = await page.evaluate(() => {
    return {
      memory: (performance as any).memory
        ? {
            usedJSHeapSize: (performance as any).memory.usedJSHeapSize,
            totalJSHeapSize: (performance as any).memory.totalJSHeapSize,
            jsHeapSizeLimit: (performance as any).memory.jsHeapSizeLimit,
          }
        : null,
      timing: performance.timing
        ? {
            loadEventEnd: performance.timing.loadEventEnd,
            loadEventStart: performance.timing.loadEventStart,
            domContentLoadedEventEnd: performance.timing.domContentLoadedEventEnd,
            responseEnd: performance.timing.responseEnd,
          }
        : null,
    };
  });

  return { metrics, workerPool, memory, timing };
}

/**
 * Simulate audio workload for performance testing
 */
export async function simulateAudioWorkload(
  page: Page,
  duration: number = 5000
): Promise<void> {
  await page.evaluate(
    ({ duration }) => {
      // This would trigger audio processing in the actual app
      // For now, just wait
      return new Promise((resolve) => setTimeout(resolve, duration));
    },
    { duration }
  );
}

/**
 * Check if Web Audio API is functional
 */
export async function checkWebAudioAPI(page: Page): Promise<boolean> {
  return await page.evaluate(() => {
    try {
      const AudioContextClass = window.AudioContext || (window as any).webkitAudioContext;
      if (!AudioContextClass) return false;

      const ctx = new AudioContextClass();
      const canRun = ctx.state !== 'suspended';
      ctx.close();
      return canRun;
    } catch (e) {
      return false;
    }
  });
}

/**
 * Wait for element with custom timeout and error message
 */
export async function waitForElementWithError(
  page: Page,
  selector: string,
  timeout: number = 10000,
  errorMessage?: string
): Promise<void> {
  try {
    await page.waitForSelector(selector, { timeout });
  } catch (error) {
    const actualError =
      errorMessage || `Element "${selector}" not found within ${timeout}ms`;
    throw new Error(actualError);
  }
}

/**
 * Assert feature support with detailed error
 */
export function assertFeatureSupport(
  features: BrowserFeatures,
  requiredFeatures: (keyof BrowserFeatures)[]
): void {
  const missingFeatures = requiredFeatures.filter((feature) => {
    const value = features[feature];
    return typeof value === 'boolean' ? !value : false;
  });

  if (missingFeatures.length > 0) {
    throw new Error(
      `Missing required browser features: ${missingFeatures.join(', ')}\n` +
        `Current support: ${JSON.stringify(features, null, 2)}`
    );
  }
}

/**
 * Create screenshot with timestamp
 */
export async function takeTimestampedScreenshot(
  page: Page,
  name: string
): Promise<string> {
  const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
  const filename = `screenshots/${name}-${timestamp}.png`;
  await page.screenshot({ path: filename, fullPage: true });
  return filename;
}

/**
 * Monitor FPS for duration and return stats
 */
export async function monitorFPS(
  page: Page,
  duration: number = 10000,
  sampleInterval: number = 500
): Promise<{ min: number; max: number; avg: number; samples: number[] }> {
  const samples: number[] = [];
  const startTime = Date.now();

  while (Date.now() - startTime < duration) {
    const metrics = await getPerformanceMetrics(page);
    samples.push(metrics.fps);
    await page.waitForTimeout(sampleInterval);
  }

  return {
    min: Math.min(...samples),
    max: Math.max(...samples),
    avg: samples.reduce((a, b) => a + b, 0) / samples.length,
    samples,
  };
}

/**
 * Check for memory leaks by monitoring memory growth
 */
export async function checkMemoryLeak(
  page: Page,
  testDuration: number = 30000,
  sampleInterval: number = 1000,
  maxGrowthMB: number = 100
): Promise<{ leaked: boolean; growthMB: number; samples: number[] }> {
  const samples: number[] = [];
  const startTime = Date.now();

  while (Date.now() - startTime < testDuration) {
    const metrics = await getPerformanceMetrics(page);
    samples.push(metrics.memory);
    await page.waitForTimeout(sampleInterval);
  }

  const growthMB = Math.max(...samples) - Math.min(...samples);
  const leaked = growthMB > maxGrowthMB;

  return { leaked, growthMB, samples };
}
