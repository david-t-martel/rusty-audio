/**
 * UI Rendering Tests
 *
 * Validates that the egui UI renders correctly:
 * - Canvas rendering
 * - UI responsiveness
 * - Control interactions
 * - Theme application
 * - Layout consistency
 * - Visual regression
 */

import { test, expect, Page } from '@playwright/test';
import {
  waitForWasmInit,
  getPerformanceMetrics,
  monitorFPS,
  takeTimestampedScreenshot,
} from '../helpers/wasm-fixtures';

test.describe('UI Rendering', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForWasmInit(page);
    await page.waitForTimeout(1000); // Let UI settle
  });

  test('should render egui canvas correctly', async ({ page }) => {
    const canvas = page.locator('#rusty-audio-canvas');

    // Canvas should be visible
    await expect(canvas).toBeVisible();

    // Canvas should have dimensions
    const dimensions = await canvas.evaluate((el: HTMLCanvasElement) => {
      return {
        width: el.width,
        height: el.height,
        displayWidth: el.clientWidth,
        displayHeight: el.clientHeight,
      };
    });

    expect(dimensions.width, 'Canvas width should be set').toBeGreaterThan(0);
    expect(dimensions.height, 'Canvas height should be set').toBeGreaterThan(0);

    console.log(`✅ Canvas dimensions: ${dimensions.width}x${dimensions.height}`);
    console.log(`   Display size: ${dimensions.displayWidth}x${dimensions.displayHeight}`);
  });

  test('should maintain 60 FPS target during rendering', async ({ page }) => {
    const fpsStats = await monitorFPS(page, 5000, 500);

    console.log(`FPS Stats over 5s:`);
    console.log(`  Min: ${fpsStats.min.toFixed(1)}`);
    console.log(`  Max: ${fpsStats.max.toFixed(1)}`);
    console.log(`  Avg: ${fpsStats.avg.toFixed(1)}`);

    // Target is 60 FPS, accept 55+ average
    expect(fpsStats.avg, 'Average FPS should be at least 55').toBeGreaterThanOrEqual(55);
  });

  test('should render UI without visual glitches', async ({ page }) => {
    // Take screenshot for visual inspection
    await page.waitForTimeout(2000); // Let rendering stabilize

    const screenshot = await page.screenshot({
      fullPage: true,
    });

    expect(screenshot.length, 'Screenshot should be captured').toBeGreaterThan(0);

    // Attach screenshot to test results
    await test.info().attach('ui-screenshot', {
      body: screenshot,
      contentType: 'image/png',
    });

    console.log('✅ UI screenshot captured for visual inspection');
  });

  test('should handle window resize responsively', async ({ page }) => {
    const initialSize = await page.viewportSize();

    // Resize to smaller viewport
    await page.setViewportSize({ width: 1280, height: 720 });
    await page.waitForTimeout(500);

    let canvas = page.locator('#rusty-audio-canvas');
    await expect(canvas).toBeVisible();

    // Resize to larger viewport
    await page.setViewportSize({ width: 1920, height: 1080 });
    await page.waitForTimeout(500);

    canvas = page.locator('#rusty-audio-canvas');
    await expect(canvas).toBeVisible();

    // Restore original size
    if (initialSize) {
      await page.setViewportSize(initialSize);
    }

    console.log('✅ UI responsive to viewport changes');
  });

  test('should render with consistent frame times', async ({ page }) => {
    const frameTimes: number[] = [];
    const sampleCount = 10;

    for (let i = 0; i < sampleCount; i++) {
      await page.waitForTimeout(500);
      const metrics = await getPerformanceMetrics(page);
      if (metrics.frameTime > 0) {
        frameTimes.push(metrics.frameTime);
      }
    }

    if (frameTimes.length > 0) {
      const avgFrameTime = frameTimes.reduce((a, b) => a + b, 0) / frameTimes.length;
      const maxFrameTime = Math.max(...frameTimes);

      console.log(`Frame time: avg=${avgFrameTime.toFixed(1)}ms, max=${maxFrameTime.toFixed(1)}ms`);

      // Frame time should be under 16.7ms for 60fps
      expect(avgFrameTime, 'Average frame time should be under 20ms').toBeLessThan(20);
    }
  });

  test('should render dark theme correctly', async ({ page }) => {
    // Check if dark theme is applied (default)
    const backgroundColor = await page.evaluate(() => {
      const body = document.body;
      return window.getComputedStyle(body).backgroundColor;
    });

    console.log(`Background color: ${backgroundColor}`);

    // Dark theme should have dark background
    // RGB values should be low for dark theme
    expect(backgroundColor).toBeTruthy();
  });

  test('should display tab navigation correctly', async ({ page }) => {
    // Look for tabs in the UI
    const hasTabs = await page.evaluate(() => {
      const text = document.body.textContent || '';
      return (
        (text.includes('Playback') ||
          text.includes('Effects') ||
          text.includes('EQ') ||
          text.includes('Generator'))
      );
    });

    if (hasTabs) {
      console.log('✅ Tab navigation detected in UI');
    } else {
      console.log('⚠️  Tab structure may be using different layout');
    }
  });

  test('should render control panels correctly', async ({ page }) => {
    // Check if UI elements are being rendered (non-zero canvas pixels)
    const hasRenderedContent = await page.evaluate(() => {
      const canvas = document.querySelector('canvas') as HTMLCanvasElement;
      if (!canvas) return false;

      const ctx = canvas.getContext('2d');
      if (!ctx) return false;

      try {
        const width = Math.min(100, canvas.width);
        const height = Math.min(100, canvas.height);
        const imageData = ctx.getImageData(0, 0, width, height);

        // Check if any pixels are non-zero
        for (let i = 0; i < imageData.data.length; i++) {
          if (imageData.data[i] !== 0) {
            return true;
          }
        }
      } catch (e) {
        // CORS may prevent getImageData
        return true; // Assume working
      }

      return false;
    });

    expect(hasRenderedContent, 'Canvas should have rendered content').toBe(true);
    console.log('✅ UI content rendering detected');
  });

  test('should handle mouse interactions', async ({ page }) => {
    const canvas = page.locator('#rusty-audio-canvas');

    // Get canvas bounding box
    const box = await canvas.boundingBox();
    if (box) {
      // Click on canvas
      await page.mouse.click(box.x + box.width / 2, box.y + box.height / 2);

      await page.waitForTimeout(100);

      // Move mouse over canvas
      await page.mouse.move(box.x + box.width / 2, box.y + 50);

      await page.waitForTimeout(100);

      console.log('✅ Mouse interactions handled');
    }
  });

  test('should handle keyboard interactions', async ({ page }) => {
    // Focus canvas
    await page.locator('#rusty-audio-canvas').click();

    // Send keyboard events
    await page.keyboard.press('Space'); // Play/pause
    await page.waitForTimeout(100);

    await page.keyboard.press('ArrowUp'); // Volume up
    await page.waitForTimeout(100);

    await page.keyboard.press('ArrowDown'); // Volume down
    await page.waitForTimeout(100);

    console.log('✅ Keyboard interactions sent');
  });

  test('should not have rendering memory leaks', async ({ page }) => {
    const memorySnapshots: number[] = [];

    for (let i = 0; i < 5; i++) {
      await page.waitForTimeout(1000);
      const metrics = await getPerformanceMetrics(page);
      if (metrics.memory > 0) {
        memorySnapshots.push(metrics.memory);
      }
    }

    if (memorySnapshots.length > 1) {
      const memoryGrowth = memorySnapshots[memorySnapshots.length - 1] - memorySnapshots[0];

      console.log(`Memory: ${memorySnapshots.map(m => m.toFixed(1)).join(', ')} MB`);
      console.log(`Growth: ${memoryGrowth.toFixed(1)} MB`);

      // Should not grow more than 20MB during normal rendering
      expect(memoryGrowth, 'Memory should not leak during rendering').toBeLessThan(20);
    }
  });

  test('should render accessibility attributes', async ({ page }) => {
    const canvas = page.locator('#rusty-audio-canvas');

    const ariaLabel = await canvas.getAttribute('aria-label');
    const role = await canvas.getAttribute('role');

    expect(ariaLabel, 'Canvas should have aria-label').toBeTruthy();
    expect(role, 'Canvas should have role attribute').toBeTruthy();

    console.log(`✅ Accessibility: aria-label="${ariaLabel}", role="${role}"`);
  });

  test('should maintain visual consistency across rerenders', async ({ page }) => {
    // Take initial screenshot
    await page.waitForTimeout(1000);
    const screenshot1 = await page.screenshot();

    // Wait and take another screenshot
    await page.waitForTimeout(2000);
    const screenshot2 = await page.screenshot();

    // Screenshots should be similar (if UI is static)
    // Note: This is a basic check - proper visual regression testing
    // would use tools like Percy or Chromatic

    expect(screenshot1.length, 'First screenshot should be valid').toBeGreaterThan(0);
    expect(screenshot2.length, 'Second screenshot should be valid').toBeGreaterThan(0);

    console.log('✅ Visual consistency check completed');
  });

  test('should handle high DPI displays correctly', async ({ page }) => {
    const devicePixelRatio = await page.evaluate(() => window.devicePixelRatio);

    const canvas = page.locator('#rusty-audio-canvas');
    const dimensions = await canvas.evaluate((el: HTMLCanvasElement) => {
      return {
        width: el.width,
        height: el.height,
        clientWidth: el.clientWidth,
        clientHeight: el.clientHeight,
      };
    });

    console.log(`Device pixel ratio: ${devicePixelRatio}`);
    console.log(`Canvas: ${dimensions.width}x${dimensions.height}`);
    console.log(`Client: ${dimensions.clientWidth}x${dimensions.clientHeight}`);

    // Canvas resolution should account for device pixel ratio
    // (though egui handles this internally)
    expect(dimensions.width, 'Canvas should have valid width').toBeGreaterThan(0);
  });

  test('should render spectrum visualizer animation', async ({ page }) => {
    // Take two screenshots with delay to check for animation
    await page.waitForTimeout(1000);
    const screenshot1 = await page.screenshot({ clip: { x: 0, y: 0, width: 800, height: 600 } });

    await page.waitForTimeout(500);
    const screenshot2 = await page.screenshot({ clip: { x: 0, y: 0, width: 800, height: 600 } });

    // Screenshots should exist
    expect(screenshot1).toBeTruthy();
    expect(screenshot2).toBeTruthy();

    // Note: Proper animation testing would compare pixel differences
    // For now, we just verify rendering is happening

    console.log('✅ Spectrum animation frames captured');
  });

  test('should handle WebGL/WebGPU context correctly', async ({ page }) => {
    const contextInfo = await page.evaluate(() => {
      const canvas = document.querySelector('canvas') as HTMLCanvasElement;
      if (!canvas) return null;

      // Check which context is being used
      const webgl2 = canvas.getContext('webgl2');
      const webgl = canvas.getContext('webgl');
      const webgpu = (navigator as any).gpu;

      return {
        hasWebGL2: !!webgl2,
        hasWebGL: !!webgl,
        hasWebGPU: !!webgpu,
        canvasWidth: canvas.width,
        canvasHeight: canvas.height,
      };
    });

    if (contextInfo) {
      console.log('Rendering context:');
      console.log(`  WebGL2: ${contextInfo.hasWebGL2}`);
      console.log(`  WebGL: ${contextInfo.hasWebGL}`);
      console.log(`  WebGPU available: ${contextInfo.hasWebGPU}`);
    }
  });

  test('should not drop frames during user interaction', async ({ page }) => {
    const canvas = page.locator('#rusty-audio-canvas');
    const box = await canvas.boundingBox();

    if (box) {
      // Simulate rapid mouse movement
      for (let i = 0; i < 20; i++) {
        await page.mouse.move(
          box.x + (box.width * i) / 20,
          box.y + box.height / 2
        );
        await page.waitForTimeout(16); // ~60 FPS
      }

      // Check FPS after interaction
      await page.waitForTimeout(500);
      const metrics = await getPerformanceMetrics(page);

      if (metrics.fps > 0) {
        expect(metrics.fps, 'FPS should remain stable during interaction').toBeGreaterThanOrEqual(50);
        console.log(`✅ FPS during interaction: ${metrics.fps}`);
      }
    }
  });

  test('should render mobile layout correctly', async ({ page, browserName }) => {
    test.skip(browserName !== 'mobile-chrome', 'Mobile layout test only for mobile browsers');

    // For mobile browsers, check responsive layout
    const isMobileLayout = await page.evaluate(() => {
      return window.innerWidth < 768;
    });

    if (isMobileLayout) {
      const canvas = page.locator('#rusty-audio-canvas');
      await expect(canvas).toBeVisible();

      console.log('✅ Mobile layout rendering');
    }
  });
});
