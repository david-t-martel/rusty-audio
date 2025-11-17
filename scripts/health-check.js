#!/usr/bin/env node

/**
 * Health Check Script
 *
 * Performs comprehensive health checks on the running development server:
 * - Server availability
 * - Header validation
 * - WASM binary accessibility
 * - Feature detection
 * - Performance metrics
 */

import fetch from 'node-fetch';
import chalk from 'chalk';

const DEFAULT_URL = 'http://localhost:8080';
const url = process.argv[2] || DEFAULT_URL;

let passed = 0;
let failed = 0;

async function checkEndpoint(endpoint, description) {
  try {
    const response = await fetch(`${url}${endpoint}`);

    if (response.ok) {
      console.log(chalk.green('✓'), description, chalk.gray(`(${response.status})`));
      passed++;
      return response;
    } else {
      console.log(chalk.red('✗'), description, chalk.red(`(${response.status})`));
      failed++;
      return null;
    }
  } catch (error) {
    console.log(chalk.red('✗'), description, chalk.red(`(${error.message})`));
    failed++;
    return null;
  }
}

async function checkHeaders(endpoint, requiredHeaders) {
  try {
    const response = await fetch(`${url}${endpoint}`);

    console.log(chalk.bold('\n  Header Validation:'));

    for (const [header, expectedValue] of Object.entries(requiredHeaders)) {
      const actualValue = response.headers.get(header);

      if (actualValue === expectedValue) {
        console.log(chalk.green('  ✓'), header, chalk.gray(`= ${actualValue}`));
        passed++;
      } else {
        console.log(
          chalk.red('  ✗'),
          header,
          chalk.red(`Expected: ${expectedValue}, Got: ${actualValue || 'missing'}`)
        );
        failed++;
      }
    }
  } catch (error) {
    console.log(chalk.red('  ✗'), 'Header check failed:', error.message);
    failed++;
  }
}

async function runHealthChecks() {
  console.log(chalk.bold.cyan('\n=== Rusty Audio Health Check ==='));
  console.log(chalk.gray(`Target: ${url}\n`));

  // Basic connectivity
  console.log(chalk.bold('Connectivity:'));
  const healthResponse = await checkEndpoint('/health', 'Health endpoint');

  if (healthResponse) {
    const health = await healthResponse.json();
    console.log(chalk.gray(`  Status: ${health.status}`));
    console.log(chalk.gray(`  Uptime: ${health.uptime?.toFixed(2) || 'N/A'}s`));
    console.log(chalk.gray(`  Environment: ${health.environment || 'unknown'}`));
  }

  // Check main endpoints
  console.log(chalk.bold('\nEndpoints:'));
  await checkEndpoint('/', 'Index page');
  await checkEndpoint('/api/features', 'Features API');
  await checkEndpoint('/api/wasm/info', 'WASM info');

  // Check WASM binary
  console.log(chalk.bold('\nWASM Binary:'));
  const wasmResponse = await checkEndpoint('/rusty_audio_bg.wasm', 'WASM binary');

  if (wasmResponse) {
    const contentType = wasmResponse.headers.get('content-type');
    if (contentType === 'application/wasm') {
      console.log(chalk.green('  ✓'), 'Content-Type correct');
      passed++;
    } else {
      console.log(chalk.red('  ✗'), 'Content-Type incorrect:', contentType);
      failed++;
    }

    const contentLength = wasmResponse.headers.get('content-length');
    if (contentLength) {
      const sizeMB = (parseInt(contentLength) / (1024 * 1024)).toFixed(2);
      console.log(chalk.gray(`  Size: ${sizeMB} MB`));
    }
  }

  // Check critical headers for multithreading
  console.log(chalk.bold('\nMultithreading Headers:'));
  await checkHeaders('/', {
    'cross-origin-opener-policy': 'same-origin',
    'cross-origin-embedder-policy': 'require-corp',
    'cross-origin-resource-policy': 'cross-origin'
  });

  // Check features API
  console.log(chalk.bold('\nFeatures:'));
  try {
    const featuresResponse = await fetch(`${url}/api/features`);
    if (featuresResponse.ok) {
      const features = await featuresResponse.json();

      if (features.threading?.enabled) {
        console.log(chalk.green('  ✓'), `Threading enabled (max workers: ${features.threading.maxWorkers})`);
        passed++;
      } else {
        console.log(chalk.yellow('  ⚠'), 'Threading disabled');
      }

      if (features.wasm?.threads) {
        console.log(chalk.green('  ✓'), 'WASM threads support');
        passed++;
      }

      if (features.browser?.crossOriginIsolated) {
        console.log(chalk.green('  ✓'), 'Cross-origin isolation enabled');
        passed++;
      }
    }
  } catch (error) {
    console.log(chalk.red('  ✗'), 'Features check failed:', error.message);
    failed++;
  }

  // Summary
  console.log(chalk.bold('\n=== Summary ===\n'));

  const total = passed + failed;
  const percentage = total > 0 ? ((passed / total) * 100).toFixed(1) : 0;

  console.log(chalk.green(`✓ Passed: ${passed}`));
  if (failed > 0) {
    console.log(chalk.red(`✗ Failed: ${failed}`));
  }
  console.log(chalk.cyan(`Success Rate: ${percentage}%\n`));

  if (failed === 0) {
    console.log(chalk.green.bold('✓ All health checks passed!\n'));
    process.exit(0);
  } else {
    console.log(chalk.red.bold('✗ Some health checks failed\n'));
    process.exit(1);
  }
}

// Run health checks
runHealthChecks().catch(error => {
  console.error(chalk.red('\nHealth check error:'), error);
  process.exit(1);
});
