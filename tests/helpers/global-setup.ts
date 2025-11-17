/**
 * Global Test Setup
 *
 * Performs one-time setup before all tests run:
 * - Validates WASM build exists
 * - Checks test server configuration
 * - Sets up test environment variables
 */

import { chromium, FullConfig } from '@playwright/test';
import * as fs from 'fs';
import * as path from 'path';

async function globalSetup(config: FullConfig) {
  console.log('🚀 Starting Playwright global setup for Rusty Audio WASM tests');

  // 1. Validate WASM build exists
  const distPath = path.join(process.cwd(), 'dist');
  const wasmFiles = [
    'rusty_audio_bg.wasm',
    'pkg/rusty_audio_bg.wasm',
  ];

  let wasmFound = false;
  for (const wasmFile of wasmFiles) {
    const wasmPath = path.join(distPath, wasmFile);
    if (fs.existsSync(wasmPath)) {
      console.log(`✅ Found WASM binary: ${wasmFile}`);
      wasmFound = true;
      break;
    }
  }

  if (!wasmFound) {
    console.error('❌ WASM binary not found in dist/');
    console.error('   Please build the WASM application first:');
    console.error('   trunk build --release');
    throw new Error('WASM binary not found - build required');
  }

  // 2. Validate index.html exists
  const indexPath = path.join(distPath, 'index.html');
  if (!fs.existsSync(indexPath)) {
    console.error('❌ index.html not found in dist/');
    throw new Error('index.html not found - build required');
  }
  console.log('✅ Found index.html');

  // 3. Check for required static assets
  const requiredAssets = [
    'static/rusty-audio-init.js',
    'static/wasm-worker-init.js',
  ];

  for (const asset of requiredAssets) {
    const assetPath = path.join(distPath, asset);
    if (!fs.existsSync(assetPath)) {
      console.warn(`⚠️  Missing static asset: ${asset}`);
    } else {
      console.log(`✅ Found ${asset}`);
    }
  }

  // 4. Validate web server configuration
  const baseURL = config.use?.baseURL || process.env.TEST_URL || 'http://localhost:8080';
  console.log(`🌐 Test server URL: ${baseURL}`);

  // 5. Wait for server to be ready (if webServer is configured)
  if (config.webServer) {
    console.log('⏳ Waiting for web server to start...');
    // Playwright will handle this automatically
  }

  // 6. Perform browser compatibility check
  console.log('🔍 Checking browser compatibility...');

  const browser = await chromium.launch({
    args: [
      '--enable-features=SharedArrayBuffer',
      '--enable-unsafe-webgpu',
    ],
  });

  try {
    const context = await browser.newContext({
      permissions: ['microphone', 'audio-capture'],
    });

    const page = await context.newPage();

    // Simple feature detection
    const features = await page.evaluate(() => {
      return {
        webAssembly: typeof WebAssembly !== 'undefined',
        sharedArrayBuffer: typeof SharedArrayBuffer !== 'undefined',
        atomics: typeof Atomics !== 'undefined',
        webAudioAPI: typeof AudioContext !== 'undefined',
        hardwareConcurrency: navigator.hardwareConcurrency,
      };
    });

    console.log('Browser features detected:');
    console.log(`  - WebAssembly: ${features.webAssembly ? '✅' : '❌'}`);
    console.log(`  - SharedArrayBuffer: ${features.sharedArrayBuffer ? '✅' : '❌'}`);
    console.log(`  - Atomics: ${features.atomics ? '✅' : '❌'}`);
    console.log(`  - Web Audio API: ${features.webAudioAPI ? '✅' : '❌'}`);
    console.log(`  - Hardware Concurrency: ${features.hardwareConcurrency} cores`);

    await context.close();
  } finally {
    await browser.close();
  }

  // 7. Create test output directories
  const outputDirs = [
    'test-results',
    'playwright-report',
    'screenshots',
    'performance-data',
  ];

  for (const dir of outputDirs) {
    const dirPath = path.join(process.cwd(), dir);
    if (!fs.existsSync(dirPath)) {
      fs.mkdirSync(dirPath, { recursive: true });
      console.log(`📁 Created directory: ${dir}`);
    }
  }

  console.log('✅ Global setup complete\n');
}

export default globalSetup;
