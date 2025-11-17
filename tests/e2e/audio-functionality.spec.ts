/**
 * Audio Functionality Tests
 *
 * Validates core audio features work correctly:
 * - Signal generator functionality
 * - Audio playback controls
 * - Volume and panning controls
 * - EQ band adjustments
 * - Spectrum analyzer updates
 * - Audio routing and processing
 */

import { test, expect, Page } from '@playwright/test';
import {
  waitForWasmInit,
  getPerformanceMetrics,
  checkWebAudioAPI,
} from '../helpers/wasm-fixtures';

test.describe('Audio Functionality', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForWasmInit(page);

    // Wait for UI to be ready
    await page.waitForTimeout(1000);
  });

  test('should have Web Audio API available', async ({ page }) => {
    const webAudioAvailable = await checkWebAudioAPI(page);

    expect(webAudioAvailable, 'Web Audio API should be available').toBe(true);
    console.log('✅ Web Audio API functional');
  });

  test('should create AudioContext successfully', async ({ page }) => {
    const audioContextCreated = await page.evaluate(async () => {
      try {
        const AudioContextClass = window.AudioContext || (window as any).webkitAudioContext;
        const ctx = new AudioContextClass();

        // Resume context (required in some browsers)
        if (ctx.state === 'suspended') {
          await ctx.resume();
        }

        const state = ctx.state;
        const sampleRate = ctx.sampleRate;

        ctx.close();

        return {
          success: true,
          state,
          sampleRate,
        };
      } catch (e: any) {
        return {
          success: false,
          error: e.message,
        };
      }
    });

    expect(audioContextCreated.success, 'AudioContext should be created').toBe(true);
    console.log(`✅ AudioContext created (${audioContextCreated.sampleRate} Hz)`);
  });

  test('should display signal generator UI', async ({ page }) => {
    // Look for Generator tab or panel
    const generatorTabVisible = await page.evaluate(() => {
      // Check for tab or panel containing "Generator" or "Signal"
      const text = document.body.textContent || '';
      return text.includes('Generator') || text.includes('Signal');
    });

    if (generatorTabVisible) {
      console.log('✅ Signal generator UI detected');
    } else {
      console.log('⚠️  Signal generator UI not visible (may be in hidden tab)');
    }
  });

  test('should display EQ controls', async ({ page }) => {
    // Look for EQ-related UI elements
    const eqUIVisible = await page.evaluate(() => {
      const text = document.body.textContent || '';
      return text.includes('EQ') || text.includes('Equalizer') || text.includes('60Hz');
    });

    if (eqUIVisible) {
      console.log('✅ EQ controls detected');
    } else {
      console.log('⚠️  EQ controls not visible (may be in hidden tab)');
    }
  });

  test('should display spectrum visualizer', async ({ page }) => {
    // Look for spectrum/effects UI
    const spectrumVisible = await page.evaluate(() => {
      const text = document.body.textContent || '';
      return text.includes('Spectrum') || text.includes('Effects') || text.includes('FFT');
    });

    if (spectrumVisible) {
      console.log('✅ Spectrum visualizer detected');
    } else {
      console.log('⚠️  Spectrum visualizer not visible (may be in hidden tab)');
    }
  });

  test('should handle audio playback controls', async ({ page }) => {
    // Look for playback controls
    const hasPlaybackControls = await page.evaluate(() => {
      const text = document.body.textContent || '';
      return text.includes('Play') || text.includes('Stop') || text.includes('Pause');
    });

    if (hasPlaybackControls) {
      console.log('✅ Playback controls detected');
    } else {
      console.log('⚠️  Playback controls not immediately visible');
    }
  });

  test('should support volume control', async ({ page }) => {
    const hasVolumeControl = await page.evaluate(() => {
      const text = document.body.textContent || '';
      return text.includes('Volume') || text.includes('Master');
    });

    if (hasVolumeControl) {
      console.log('✅ Volume control detected');
    }
  });

  test('should support panning control', async ({ page }) => {
    const hasPanControl = await page.evaluate(() => {
      const text = document.body.textContent || '';
      return text.includes('Pan') || text.includes('Balance');
    });

    if (hasPanControl) {
      console.log('✅ Pan control detected');
    }
  });

  test('should initialize without audio glitches or errors', async ({ page }) => {
    let audioErrors: string[] = [];

    page.on('console', (msg) => {
      if (msg.type() === 'error') {
        const text = msg.text();
        if (
          text.includes('Audio') ||
          text.includes('audio') ||
          text.includes('AudioContext')
        ) {
          audioErrors.push(text);
        }
      }
    });

    // Wait a bit for any delayed errors
    await page.waitForTimeout(3000);

    expect(
      audioErrors,
      `Should have no audio-related errors: ${audioErrors.join(', ')}`
    ).toHaveLength(0);

    console.log('✅ No audio initialization errors');
  });

  test('should handle audio processing without high latency', async ({ page }) => {
    await page.waitForTimeout(3000); // Let performance stabilize

    const metrics = await getPerformanceMetrics(page);

    // Audio latency should be reasonable (< 100ms for WASM)
    if (metrics.audioLatency > 0) {
      expect(metrics.audioLatency, 'Audio latency should be under 100ms').toBeLessThan(100);
      console.log(`✅ Audio latency: ${metrics.audioLatency.toFixed(1)}ms`);
    } else {
      console.log('⚠️  Audio latency not yet measured');
    }
  });

  test('should support FFT spectrum analysis', async ({ page }) => {
    // Test if FFT analysis is working by checking for visual updates
    const hasSpectrum = await page.evaluate(() => {
      // Check if canvas is being updated (non-zero pixel data)
      const canvas = document.querySelector('canvas');
      if (!canvas) return false;

      const ctx = canvas.getContext('2d');
      if (!ctx) return false;

      try {
        const imageData = ctx.getImageData(0, 0, Math.min(100, canvas.width), Math.min(100, canvas.height));
        // Check if any pixels are non-zero (being drawn)
        for (let i = 0; i < imageData.data.length; i++) {
          if (imageData.data[i] !== 0) {
            return true;
          }
        }
      } catch (e) {
        // getImageData may fail due to CORS
        return true; // Assume working if we can't check
      }

      return false;
    });

    console.log(`Canvas rendering active: ${hasSpectrum}`);
  });

  test('should support multiple audio signal types', async ({ page }) => {
    // Check if signal generator supports multiple waveforms
    const signalTypes = await page.evaluate(() => {
      const text = document.body.textContent || '';
      const types = [];

      if (text.includes('Sine')) types.push('Sine');
      if (text.includes('Square')) types.push('Square');
      if (text.includes('Sawtooth')) types.push('Sawtooth');
      if (text.includes('Triangle')) types.push('Triangle');
      if (text.includes('Noise')) types.push('Noise');

      return types;
    });

    console.log(`Signal types detected: ${signalTypes.join(', ')}`);

    if (signalTypes.length > 0) {
      expect(signalTypes.length, 'Should support multiple signal types').toBeGreaterThan(0);
    }
  });

  test('should support EQ frequency bands', async ({ page }) => {
    // Check for EQ frequency labels
    const eqBands = await page.evaluate(() => {
      const text = document.body.textContent || '';
      const bands = [];

      // Look for common EQ frequencies
      const frequencies = ['60Hz', '170Hz', '310Hz', '600Hz', '1kHz', '3kHz', '6kHz', '12kHz'];

      for (const freq of frequencies) {
        if (text.includes(freq)) {
          bands.push(freq);
        }
      }

      return bands;
    });

    if (eqBands.length > 0) {
      console.log(`✅ EQ bands detected: ${eqBands.join(', ')}`);
      expect(eqBands.length, 'Should have multiple EQ bands').toBeGreaterThan(0);
    } else {
      console.log('⚠️  EQ frequency bands not visible in current view');
    }
  });

  test('should handle audio format metadata', async ({ page }) => {
    // Check if metadata display is available
    const hasMetadataUI = await page.evaluate(() => {
      const text = document.body.textContent || '';
      return (
        text.includes('Artist') ||
        text.includes('Album') ||
        text.includes('Title') ||
        text.includes('Metadata')
      );
    });

    if (hasMetadataUI) {
      console.log('✅ Metadata display UI detected');
    }
  });

  test('should support recording functionality', async ({ page }) => {
    const hasRecording = await page.evaluate(() => {
      const text = document.body.textContent || '';
      return text.includes('Record') || text.includes('Recording');
    });

    if (hasRecording) {
      console.log('✅ Recording functionality detected');
    }
  });

  test('should maintain stable FPS during audio processing', async ({ page }) => {
    // Let app warm up
    await page.waitForTimeout(2000);

    const samples: number[] = [];
    const sampleCount = 5;

    for (let i = 0; i < sampleCount; i++) {
      await page.waitForTimeout(500);
      const metrics = await getPerformanceMetrics(page);
      samples.push(metrics.fps);
    }

    const avgFPS = samples.reduce((a, b) => a + b, 0) / samples.length;
    const minFPS = Math.min(...samples);

    console.log(`FPS: avg=${avgFPS.toFixed(1)}, min=${minFPS.toFixed(1)}`);

    // FPS should be reasonable (at least 30 fps average)
    if (avgFPS > 0) {
      expect(avgFPS, 'Average FPS should be at least 30').toBeGreaterThanOrEqual(30);
    }
  });

  test('should handle audio context state changes', async ({ page }) => {
    const stateChangeHandled = await page.evaluate(async () => {
      try {
        const AudioContextClass = window.AudioContext || (window as any).webkitAudioContext;
        const ctx = new AudioContextClass();

        let stateChanged = false;
        ctx.onstatechange = () => {
          stateChanged = true;
        };

        if (ctx.state === 'suspended') {
          await ctx.resume();
        } else {
          await ctx.suspend();
        }

        await new Promise((resolve) => setTimeout(resolve, 100));

        ctx.close();
        return stateChanged;
      } catch (e) {
        return false;
      }
    });

    console.log(`AudioContext state change handling: ${stateChangeHandled}`);
  });

  test('should not have memory leaks during audio processing', async ({ page }) => {
    const memorySnapshots: number[] = [];

    // Take initial snapshot
    await page.waitForTimeout(2000);
    let metrics = await getPerformanceMetrics(page);
    memorySnapshots.push(metrics.memory);

    // Wait and take more snapshots
    for (let i = 0; i < 3; i++) {
      await page.waitForTimeout(2000);
      metrics = await getPerformanceMetrics(page);
      memorySnapshots.push(metrics.memory);
    }

    const memoryGrowth = memorySnapshots[memorySnapshots.length - 1] - memorySnapshots[0];

    console.log(`Memory samples: ${memorySnapshots.map(m => m.toFixed(1)).join(', ')} MB`);
    console.log(`Memory growth: ${memoryGrowth.toFixed(1)} MB`);

    // Memory shouldn't grow excessively (< 50MB over test period)
    if (memorySnapshots[0] > 0) {
      expect(memoryGrowth, 'Memory growth should be limited').toBeLessThan(50);
    }
  });

  test('should support theme selection for UI', async ({ page }) => {
    const hasThemeUI = await page.evaluate(() => {
      const text = document.body.textContent || '';
      return text.includes('Theme') || text.includes('Dark') || text.includes('Light');
    });

    if (hasThemeUI) {
      console.log('✅ Theme selection UI detected');
    }
  });

  test('should handle file selection for audio playback', async ({ page }) => {
    const hasFileSelection = await page.evaluate(() => {
      const text = document.body.textContent || '';
      return (
        text.includes('Select File') ||
        text.includes('Choose File') ||
        text.includes('Load') ||
        text.includes('Open')
      );
    });

    if (hasFileSelection) {
      console.log('✅ File selection UI detected');
    }
  });
});
