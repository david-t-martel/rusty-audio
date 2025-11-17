#!/usr/bin/env node

/**
 * Threading Test Script
 *
 * Tests multithreading capabilities of the WASM application:
 * - SharedArrayBuffer availability
 * - Cross-origin isolation
 * - Worker thread creation
 * - WASM threading features
 */

import { chromium } from '@playwright/test';
import chalk from 'chalk';

const DEFAULT_URL = 'http://localhost:8080';
const url = process.argv[2] || DEFAULT_URL;

async function testThreading() {
  console.log(chalk.bold.cyan('\n=== Multithreading Test ==='));
  console.log(chalk.gray(`Target: ${url}\n`));

  const browser = await chromium.launch({
    headless: true,
    args: [
      '--enable-features=SharedArrayBuffer',
      '--enable-webassembly-threads'
    ]
  });

  const context = await browser.newContext({
    ignoreHTTPSErrors: true
  });

  const page = await context.newPage();

  // Collect console messages
  const consoleMessages = [];
  page.on('console', msg => {
    consoleMessages.push({
      type: msg.type(),
      text: msg.text()
    });
  });

  // Collect errors
  const errors = [];
  page.on('pageerror', error => {
    errors.push(error.message);
  });

  try {
    console.log(chalk.bold('Loading application...'));

    // Navigate to the page
    await page.goto(url, { waitUntil: 'domcontentloaded', timeout: 30000 });

    // Wait a bit for initialization
    await page.waitForTimeout(2000);

    // Test 1: Check cross-origin isolation
    console.log(chalk.bold('\nTest 1: Cross-Origin Isolation'));
    const isIsolated = await page.evaluate(() => {
      return typeof crossOriginIsolated !== 'undefined' && crossOriginIsolated;
    });

    if (isIsolated) {
      console.log(chalk.green('✓'), 'Cross-origin isolated');
    } else {
      console.log(chalk.red('✗'), 'NOT cross-origin isolated');
    }

    // Test 2: SharedArrayBuffer availability
    console.log(chalk.bold('\nTest 2: SharedArrayBuffer'));
    const hasSAB = await page.evaluate(() => {
      return typeof SharedArrayBuffer !== 'undefined';
    });

    if (hasSAB) {
      console.log(chalk.green('✓'), 'SharedArrayBuffer available');

      // Try to create a SharedArrayBuffer
      const sabWorks = await page.evaluate(() => {
        try {
          const sab = new SharedArrayBuffer(1024);
          return sab.byteLength === 1024;
        } catch (e) {
          return false;
        }
      });

      if (sabWorks) {
        console.log(chalk.green('✓'), 'SharedArrayBuffer functional');
      } else {
        console.log(chalk.red('✗'), 'SharedArrayBuffer creation failed');
      }
    } else {
      console.log(chalk.red('✗'), 'SharedArrayBuffer NOT available');
    }

    // Test 3: Worker support
    console.log(chalk.bold('\nTest 3: Web Workers'));
    const hasWorker = await page.evaluate(() => {
      return typeof Worker !== 'undefined';
    });

    if (hasWorker) {
      console.log(chalk.green('✓'), 'Worker API available');

      // Test worker creation
      const workerCreated = await page.evaluate(() => {
        try {
          const blob = new Blob(['self.postMessage("ready")'], { type: 'application/javascript' });
          const worker = new Worker(URL.createObjectURL(blob));
          worker.terminate();
          return true;
        } catch (e) {
          return false;
        }
      });

      if (workerCreated) {
        console.log(chalk.green('✓'), 'Worker creation successful');
      } else {
        console.log(chalk.red('✗'), 'Worker creation failed');
      }
    } else {
      console.log(chalk.red('✗'), 'Worker API NOT available');
    }

    // Test 4: WASM features
    console.log(chalk.bold('\nTest 4: WebAssembly Features'));

    const wasmFeatures = await page.evaluate(async () => {
      const features = {
        basic: typeof WebAssembly !== 'undefined',
        streaming: typeof WebAssembly.instantiateStreaming === 'function',
        threads: false,
        simd: false,
        bulkMemory: false
      };

      // Test threads
      try {
        const threadsWasm = new Uint8Array([
          0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00,
          0x05, 0x04, 0x01, 0x03, 0x01, 0x01 // Shared memory section
        ]);
        await WebAssembly.instantiate(threadsWasm);
        features.threads = true;
      } catch (e) {
        // Threads not supported
      }

      return features;
    });

    if (wasmFeatures.basic) {
      console.log(chalk.green('✓'), 'WebAssembly available');
    }

    if (wasmFeatures.streaming) {
      console.log(chalk.green('✓'), 'Streaming compilation supported');
    }

    if (wasmFeatures.threads) {
      console.log(chalk.green('✓'), 'WASM threads supported');
    } else {
      console.log(chalk.yellow('⚠'), 'WASM threads not detected');
    }

    // Test 5: Check response headers
    console.log(chalk.bold('\nTest 5: Response Headers'));

    const response = await page.goto(url);
    const headers = response.headers();

    const requiredHeaders = {
      'cross-origin-opener-policy': 'same-origin',
      'cross-origin-embedder-policy': 'require-corp'
    };

    let headersValid = true;
    for (const [header, expected] of Object.entries(requiredHeaders)) {
      const actual = headers[header];
      if (actual === expected) {
        console.log(chalk.green('✓'), `${header}: ${actual}`);
      } else {
        console.log(chalk.red('✗'), `${header}: expected '${expected}', got '${actual}'`);
        headersValid = false;
      }
    }

    // Test 6: Check for WASM loading
    console.log(chalk.bold('\nTest 6: WASM Loading'));

    const wasmLoaded = await page.evaluate(() => {
      return typeof window.wasm !== 'undefined' ||
             performance.getEntriesByType('resource')
               .some(r => r.name.includes('.wasm'));
    });

    if (wasmLoaded) {
      console.log(chalk.green('✓'), 'WASM module loaded');
    } else {
      console.log(chalk.yellow('⚠'), 'WASM module loading unclear');
    }

    // Check for errors
    console.log(chalk.bold('\nErrors:'));
    if (errors.length === 0) {
      console.log(chalk.green('✓'), 'No JavaScript errors detected');
    } else {
      console.log(chalk.red('✗'), `${errors.length} error(s) detected:`);
      errors.forEach(err => {
        console.log(chalk.red('  -'), err);
      });
    }

    // Summary
    console.log(chalk.bold('\n=== Test Summary ===\n'));

    const allPassed = isIsolated && hasSAB && hasWorker && wasmFeatures.basic && headersValid;

    if (allPassed) {
      console.log(chalk.green.bold('✓ All multithreading tests passed!\n'));
      await browser.close();
      process.exit(0);
    } else {
      console.log(chalk.yellow.bold('⚠ Some tests failed or showed warnings\n'));
      await browser.close();
      process.exit(1);
    }

  } catch (error) {
    console.error(chalk.red('\nTest error:'), error.message);
    await browser.close();
    process.exit(1);
  }
}

// Run tests
testThreading().catch(error => {
  console.error(chalk.red('Fatal error:'), error);
  process.exit(1);
});
