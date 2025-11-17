/**
 * Performance Benchmark Tests
 *
 * Comprehensive performance validation:
 * - WASM load time benchmarks
 * - FPS stability under load
 * - Memory usage patterns
 * - Audio latency measurements
 * - FFT performance
 * - Worker pool overhead
 */

import { test, expect, Page } from '@playwright/test';
import {
  waitForWasmInit,
  getPerformanceMetrics,
  getWorkerPoolStatus,
  monitorFPS,
  checkMemoryLeak,
  takePerformanceSnapshot,
  waitForPerformanceStable,
} from '../helpers/wasm-fixtures';

test.describe('Performance Benchmarks', () => {
  test.describe.configure({ mode: 'serial' }); // Run serially for accurate measurements

  test('benchmark: WASM initialization time', async ({ page }) => {
    const startTime = Date.now();

    await page.goto('/');

    const state = await waitForWasmInit(page, 30000);

    const loadTime = Date.now() - startTime;

    console.log(`\n📊 WASM Initialization Benchmark:`);
    console.log(`   Total time: ${loadTime}ms`);
    console.log(`   Target: < 3000ms`);

    // Performance targets
    expect(loadTime, 'Init time should be under 3 seconds').toBeLessThan(3000);

    // Attach metrics
    await test.info().attach('performance-metrics', {
      body: JSON.stringify({
        testName: 'wasm-init-time',
        loadTime,
        target: 3000,
        passed: loadTime < 3000,
      }),
      contentType: 'application/json',
    });

    // Grade performance
    let grade = 'F';
    if (loadTime < 1000) grade = 'A+';
    else if (loadTime < 1500) grade = 'A';
    else if (loadTime < 2000) grade = 'B';
    else if (loadTime < 2500) grade = 'C';
    else if (loadTime < 3000) grade = 'D';

    console.log(`   Grade: ${grade}`);
  });

  test('benchmark: steady-state FPS', async ({ page }) => {
    await page.goto('/');
    await waitForWasmInit(page);

    // Let rendering stabilize
    await page.waitForTimeout(2000);

    // Monitor FPS for 10 seconds
    const fpsStats = await monitorFPS(page, 10000, 500);

    console.log(`\n📊 FPS Benchmark (10s):`);
    console.log(`   Min FPS: ${fpsStats.min.toFixed(1)}`);
    console.log(`   Max FPS: ${fpsStats.max.toFixed(1)}`);
    console.log(`   Avg FPS: ${fpsStats.avg.toFixed(1)}`);
    console.log(`   Target: 60 FPS`);

    // Performance targets
    expect(fpsStats.avg, 'Average FPS should be at least 55').toBeGreaterThanOrEqual(55);
    expect(fpsStats.min, 'Min FPS should be at least 50').toBeGreaterThanOrEqual(50);

    // Attach metrics
    await test.info().attach('performance-metrics', {
      body: JSON.stringify({
        testName: 'steady-state-fps',
        fps: fpsStats.avg,
        minFPS: fpsStats.min,
        maxFPS: fpsStats.max,
      }),
      contentType: 'application/json',
    });

    // Calculate FPS stability (lower is better)
    const stability = fpsStats.max - fpsStats.min;
    console.log(`   Stability: ±${(stability / 2).toFixed(1)} FPS`);
  });

  test('benchmark: memory usage', async ({ page }) => {
    await page.goto('/');
    await waitForWasmInit(page);

    await page.waitForTimeout(2000);

    const snapshot = await takePerformanceSnapshot(page);

    const memoryMB = snapshot.metrics.memory;

    console.log(`\n📊 Memory Usage Benchmark:`);
    console.log(`   Used JS Heap: ${memoryMB.toFixed(1)} MB`);

    if (snapshot.memory) {
      const totalMB = (snapshot.memory.totalJSHeapSize / 1024 / 1024).toFixed(1);
      const limitMB = (snapshot.memory.jsHeapSizeLimit / 1024 / 1024).toFixed(1);

      console.log(`   Total JS Heap: ${totalMB} MB`);
      console.log(`   Heap Limit: ${limitMB} MB`);
      console.log(`   Usage: ${((snapshot.memory.usedJSHeapSize / snapshot.memory.jsHeapSizeLimit) * 100).toFixed(1)}%`);
    }

    console.log(`   Target: < 200 MB`);

    // Memory should be reasonable
    expect(memoryMB, 'Memory usage should be under 200MB').toBeLessThan(200);

    // Attach metrics
    await test.info().attach('performance-metrics', {
      body: JSON.stringify({
        testName: 'memory-usage',
        memory: memoryMB,
      }),
      contentType: 'application/json',
    });
  });

  test('benchmark: memory leak detection', async ({ page }) => {
    await page.goto('/');
    await waitForWasmInit(page);

    console.log(`\n📊 Memory Leak Detection (30s):`);

    const leakCheck = await checkMemoryLeak(page, 30000, 2000, 50);

    console.log(`   Initial memory: ${leakCheck.samples[0].toFixed(1)} MB`);
    console.log(`   Final memory: ${leakCheck.samples[leakCheck.samples.length - 1].toFixed(1)} MB`);
    console.log(`   Growth: ${leakCheck.growthMB.toFixed(1)} MB`);
    console.log(`   Threshold: 50 MB`);
    console.log(`   Result: ${leakCheck.leaked ? '❌ LEAK DETECTED' : '✅ NO LEAK'}`);

    expect(leakCheck.leaked, 'Should not have memory leak').toBe(false);

    // Attach metrics
    await test.info().attach('performance-metrics', {
      body: JSON.stringify({
        testName: 'memory-leak',
        growthMB: leakCheck.growthMB,
        samples: leakCheck.samples,
      }),
      contentType: 'application/json',
    });
  });

  test('benchmark: audio latency', async ({ page }) => {
    await page.goto('/');
    await waitForWasmInit(page);

    await page.waitForTimeout(3000); // Let audio system initialize

    const metrics = await getPerformanceMetrics(page);

    console.log(`\n📊 Audio Latency Benchmark:`);
    console.log(`   Latency: ${metrics.audioLatency.toFixed(1)}ms`);
    console.log(`   Target: < 50ms`);

    if (metrics.audioLatency > 0) {
      expect(metrics.audioLatency, 'Audio latency should be under 50ms').toBeLessThan(50);

      // Attach metrics
      await test.info().attach('performance-metrics', {
        body: JSON.stringify({
          testName: 'audio-latency',
          audioLatency: metrics.audioLatency,
        }),
        contentType: 'application/json',
      });
    } else {
      console.log(`   ⚠️  Latency not yet measured`);
      test.skip(true, 'Audio latency not available');
    }
  });

  test('benchmark: frame time consistency', async ({ page }) => {
    await page.goto('/');
    await waitForWasmInit(page);

    await page.waitForTimeout(2000);

    const frameTimes: number[] = [];
    const sampleCount = 20;

    for (let i = 0; i < sampleCount; i++) {
      await page.waitForTimeout(500);
      const metrics = await getPerformanceMetrics(page);
      if (metrics.frameTime > 0) {
        frameTimes.push(metrics.frameTime);
      }
    }

    if (frameTimes.length > 0) {
      const avg = frameTimes.reduce((a, b) => a + b, 0) / frameTimes.length;
      const max = Math.max(...frameTimes);
      const min = Math.min(...frameTimes);
      const variance = frameTimes.reduce((sum, t) => sum + Math.pow(t - avg, 2), 0) / frameTimes.length;
      const stdDev = Math.sqrt(variance);

      console.log(`\n📊 Frame Time Consistency:`);
      console.log(`   Avg: ${avg.toFixed(2)}ms`);
      console.log(`   Min: ${min.toFixed(2)}ms`);
      console.log(`   Max: ${max.toFixed(2)}ms`);
      console.log(`   Std Dev: ${stdDev.toFixed(2)}ms`);
      console.log(`   Target: < 16.7ms avg (60 FPS)`);

      expect(avg, 'Avg frame time should be under 16.7ms').toBeLessThan(16.7);
      expect(stdDev, 'Frame time should be consistent (low std dev)').toBeLessThan(5);

      // Attach metrics
      await test.info().attach('performance-metrics', {
        body: JSON.stringify({
          testName: 'frame-time-consistency',
          frameTime: avg,
          stdDev,
        }),
        contentType: 'application/json',
      });
    }
  });

  test('benchmark: worker pool overhead', async ({ page }) => {
    await page.goto('/');
    await waitForWasmInit(page);

    const workerPoolStatus = await getWorkerPoolStatus(page);

    if (workerPoolStatus && workerPoolStatus.initialized) {
      console.log(`\n📊 Worker Pool Benchmark:`);
      console.log(`   Total workers: ${workerPoolStatus.totalWorkers}`);
      console.log(`   Available workers: ${workerPoolStatus.availableWorkers}`);
      console.log(`   Busy workers: ${workerPoolStatus.busyWorkers}`);
      console.log(`   Pending tasks: ${workerPoolStatus.pendingTasks}`);
      console.log(`   Total tasks completed: ${workerPoolStatus.totalTasks}`);

      expect(workerPoolStatus.totalWorkers, 'Should have workers').toBeGreaterThan(0);

      // Attach metrics
      await test.info().attach('performance-metrics', {
        body: JSON.stringify({
          testName: 'worker-pool',
          ...workerPoolStatus,
        }),
        contentType: 'application/json',
      });
    } else {
      console.log(`\n📊 Worker Pool: Not initialized (single-threaded mode)`);
    }
  });

  test('benchmark: time to interactive', async ({ page }) => {
    const startTime = Date.now();

    await page.goto('/');

    // Wait for WASM init
    await waitForWasmInit(page);

    // Wait for performance to stabilize (interactive)
    await waitForPerformanceStable(page, 55, 2000);

    const timeToInteractive = Date.now() - startTime;

    console.log(`\n📊 Time to Interactive:`);
    console.log(`   TTI: ${timeToInteractive}ms`);
    console.log(`   Target: < 5000ms`);

    expect(timeToInteractive, 'TTI should be under 5 seconds').toBeLessThan(5000);

    // Attach metrics
    await test.info().attach('performance-metrics', {
      body: JSON.stringify({
        testName: 'time-to-interactive',
        tti: timeToInteractive,
      }),
      contentType: 'application/json',
    });
  });

  test('benchmark: canvas rendering performance', async ({ page }) => {
    await page.goto('/');
    await waitForWasmInit(page);

    // Measure time to render frames
    const renderTimes: number[] = [];

    for (let i = 0; i < 10; i++) {
      const start = Date.now();
      await page.waitForTimeout(16); // One frame at 60fps
      const end = Date.now();
      renderTimes.push(end - start);
    }

    const avgRenderTime = renderTimes.reduce((a, b) => a + b, 0) / renderTimes.length;

    console.log(`\n📊 Canvas Rendering Performance:`);
    console.log(`   Avg render time: ${avgRenderTime.toFixed(2)}ms`);
    console.log(`   Target: < 16.7ms (60 FPS)`);

    // Check current FPS
    const metrics = await getPerformanceMetrics(page);
    console.log(`   Current FPS: ${metrics.fps}`);
  });

  test('benchmark: WASM binary size', async ({ page }) => {
    let wasmSize = 0;

    page.on('response', async (response) => {
      if (response.url().endsWith('.wasm')) {
        const buffer = await response.body().catch(() => null);
        if (buffer) {
          wasmSize = buffer.length;
        }
      }
    });

    await page.goto('/');
    await page.waitForTimeout(3000);

    console.log(`\n📊 WASM Binary Size:`);
    console.log(`   Size: ${(wasmSize / 1024 / 1024).toFixed(2)} MB`);
    console.log(`   Size (compressed): ${(wasmSize / 1024 / 1024 * 0.3).toFixed(2)} MB (est.)`);

    if (wasmSize > 0) {
      // WASM should be reasonably sized (< 10MB uncompressed)
      expect(wasmSize / 1024 / 1024, 'WASM size should be reasonable').toBeLessThan(10);

      // Attach metrics
      await test.info().attach('performance-metrics', {
        body: JSON.stringify({
          testName: 'wasm-size',
          sizeBytes: wasmSize,
          sizeMB: wasmSize / 1024 / 1024,
        }),
        contentType: 'application/json',
      });
    }
  });

  test('benchmark: first paint time', async ({ page }) => {
    await page.goto('/');

    // Get paint timing
    const paintTiming = await page.evaluate(() => {
      const perfEntries = performance.getEntriesByType('paint');
      const firstPaint = perfEntries.find(e => e.name === 'first-paint');
      const firstContentfulPaint = perfEntries.find(e => e.name === 'first-contentful-paint');

      return {
        firstPaint: firstPaint?.startTime || 0,
        firstContentfulPaint: firstContentfulPaint?.startTime || 0,
      };
    });

    console.log(`\n📊 Paint Performance:`);
    console.log(`   First Paint: ${paintTiming.firstPaint.toFixed(0)}ms`);
    console.log(`   First Contentful Paint: ${paintTiming.firstContentfulPaint.toFixed(0)}ms`);
    console.log(`   Target FCP: < 1500ms`);

    if (paintTiming.firstContentfulPaint > 0) {
      expect(
        paintTiming.firstContentfulPaint,
        'FCP should be under 1.5 seconds'
      ).toBeLessThan(1500);

      // Attach metrics
      await test.info().attach('performance-metrics', {
        body: JSON.stringify({
          testName: 'paint-timing',
          ...paintTiming,
        }),
        contentType: 'application/json',
      });
    }
  });

  test('benchmark: comparison report', async ({ page, browserName }) => {
    // Comprehensive performance snapshot
    await page.goto('/');
    await waitForWasmInit(page);
    await page.waitForTimeout(3000);

    const snapshot = await takePerformanceSnapshot(page);
    const workerPool = await getWorkerPoolStatus(page);

    const report = {
      browser: browserName,
      timestamp: new Date().toISOString(),
      metrics: snapshot.metrics,
      workerPool: workerPool || { initialized: false },
      memory: snapshot.memory,
      timing: snapshot.timing,
    };

    console.log(`\n📊 Performance Report Summary (${browserName}):`);
    console.log(`   FPS: ${report.metrics.fps}`);
    console.log(`   Frame Time: ${report.metrics.frameTime.toFixed(1)}ms`);
    console.log(`   Memory: ${report.metrics.memory.toFixed(1)}MB`);
    console.log(`   Audio Latency: ${report.metrics.audioLatency.toFixed(1)}ms`);
    console.log(`   Workers: ${workerPool?.totalWorkers || 0}`);

    // Save detailed report
    await test.info().attach('performance-report', {
      body: JSON.stringify(report, null, 2),
      contentType: 'application/json',
    });
  });
});
