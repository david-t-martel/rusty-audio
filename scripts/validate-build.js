#!/usr/bin/env node

/**
 * Build Validation Script
 *
 * Validates the built WASM application:
 * - Checks for required files
 * - Verifies WASM binary integrity
 * - Validates JavaScript bindings
 * - Checks for proper headers
 * - Verifies multithreading support markers
 */

import { existsSync, readFileSync, statSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';
import chalk from 'chalk';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const ROOT_DIR = join(__dirname, '..');
const DIST_DIR = join(ROOT_DIR, 'dist');

let errors = 0;
let warnings = 0;

function checkFile(path, description, required = true) {
  if (existsSync(path)) {
    const stats = statSync(path);
    const size = (stats.size / 1024).toFixed(2);
    console.log(chalk.green('✓'), description, chalk.gray(`(${size} KB)`));
    return true;
  } else {
    if (required) {
      console.log(chalk.red('✗'), description, chalk.red('MISSING'));
      errors++;
    } else {
      console.log(chalk.yellow('⚠'), description, chalk.yellow('MISSING (optional)'));
      warnings++;
    }
    return false;
  }
}

function validateWasm(path) {
  if (!existsSync(path)) return false;

  const buffer = readFileSync(path);

  // Check WASM magic number (0x00 0x61 0x73 0x6D)
  const magic = buffer.slice(0, 4);
  const expectedMagic = Buffer.from([0x00, 0x61, 0x73, 0x6D]);

  if (!magic.equals(expectedMagic)) {
    console.log(chalk.red('✗'), 'WASM magic number invalid');
    errors++;
    return false;
  }

  // Check WASM version (version 1 = 0x01 0x00 0x00 0x00)
  const version = buffer.slice(4, 8);
  const expectedVersion = Buffer.from([0x01, 0x00, 0x00, 0x00]);

  if (!version.equals(expectedVersion)) {
    console.log(chalk.yellow('⚠'), 'WASM version unexpected:', version);
    warnings++;
  }

  console.log(chalk.green('✓'), 'WASM binary structure valid');

  // Check for threading indicators (shared memory section)
  const wasmStr = buffer.toString('binary');
  const hasSharedMemory = wasmStr.includes('memory') || wasmStr.includes('shared');

  if (hasSharedMemory) {
    console.log(chalk.green('✓'), 'Threading markers detected');
  } else {
    console.log(chalk.yellow('⚠'), 'No threading markers found (may not be multithreaded)');
    warnings++;
  }

  return true;
}

function validateJavaScript(path) {
  if (!existsSync(path)) return false;

  const content = readFileSync(path, 'utf-8');

  // Check for critical functions
  const requiredFunctions = [
    'init',
    'WebAssembly',
    'importScripts'
  ];

  const missingFunctions = requiredFunctions.filter(fn => !content.includes(fn));

  if (missingFunctions.length > 0) {
    console.log(chalk.yellow('⚠'), 'Missing some expected functions:', missingFunctions.join(', '));
    warnings++;
  } else {
    console.log(chalk.green('✓'), 'JavaScript bindings contain expected functions');
  }

  // Check for threading support
  if (content.includes('Worker') || content.includes('SharedArrayBuffer')) {
    console.log(chalk.green('✓'), 'Threading support detected in JS bindings');
  } else {
    console.log(chalk.yellow('⚠'), 'No threading support detected in JS bindings');
    warnings++;
  }

  return true;
}

function validateHeaders(path) {
  if (!existsSync(path)) return false;

  const content = readFileSync(path, 'utf-8');

  const requiredHeaders = [
    'Cross-Origin-Opener-Policy',
    'Cross-Origin-Embedder-Policy',
    'Cross-Origin-Resource-Policy'
  ];

  const missingHeaders = requiredHeaders.filter(header => !content.includes(header));

  if (missingHeaders.length === 0) {
    console.log(chalk.green('✓'), 'Required COOP/COEP/CORP headers found');
    return true;
  } else {
    console.log(chalk.red('✗'), 'Missing headers:', missingHeaders.join(', '));
    errors++;
    return false;
  }
}

console.log(chalk.bold.cyan('\n=== Rusty Audio Build Validation ===\n'));

// Check required files
console.log(chalk.bold('Required Files:'));
checkFile(join(DIST_DIR, 'index.html'), 'index.html');
checkFile(join(DIST_DIR, 'rusty_audio_bg.wasm'), 'rusty_audio_bg.wasm');
checkFile(join(DIST_DIR, 'rusty_audio.js'), 'rusty_audio.js');

console.log(chalk.bold('\nOptional Files:'));
checkFile(join(DIST_DIR, 'service-worker.js'), 'service-worker.js', false);
checkFile(join(DIST_DIR, 'manifest.webmanifest'), 'manifest.webmanifest', false);
checkFile(join(DIST_DIR, '_headers'), '_headers', false);

// Validate WASM binary
console.log(chalk.bold('\nWASM Binary Validation:'));
const wasmPath = join(DIST_DIR, 'rusty_audio_bg.wasm');
validateWasm(wasmPath);

// Validate JavaScript bindings
console.log(chalk.bold('\nJavaScript Bindings:'));
const jsPath = join(DIST_DIR, 'rusty_audio.js');
validateJavaScript(jsPath);

// Validate headers
console.log(chalk.bold('\nHeader Validation:'));
const htmlPath = join(DIST_DIR, 'index.html');
validateHeaders(htmlPath);

// Check static assets
console.log(chalk.bold('\nStatic Assets:'));
const staticDirs = ['icons', 'static'];
staticDirs.forEach(dir => {
  const dirPath = join(DIST_DIR, dir);
  if (existsSync(dirPath)) {
    console.log(chalk.green('✓'), `${dir}/ directory present`);
  } else {
    console.log(chalk.yellow('⚠'), `${dir}/ directory missing`);
    warnings++;
  }
});

// Summary
console.log(chalk.bold('\n=== Validation Summary ===\n'));

if (errors === 0 && warnings === 0) {
  console.log(chalk.green.bold('✓ All checks passed!'));
  process.exit(0);
} else {
  if (errors > 0) {
    console.log(chalk.red(`✗ ${errors} error(s) found`));
  }
  if (warnings > 0) {
    console.log(chalk.yellow(`⚠ ${warnings} warning(s) found`));
  }

  if (errors > 0) {
    console.log(chalk.red('\nBuild validation FAILED\n'));
    process.exit(1);
  } else {
    console.log(chalk.yellow('\nBuild validation passed with warnings\n'));
    process.exit(0);
  }
}
