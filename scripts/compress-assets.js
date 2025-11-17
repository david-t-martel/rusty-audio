#!/usr/bin/env node

/**
 * Asset Compression Script
 *
 * Compresses built assets with Gzip and Brotli for optimal delivery:
 * - WASM binaries
 * - JavaScript files
 * - CSS files (if present)
 * - HTML files
 */

import { readFileSync, writeFileSync, readdirSync, statSync } from 'fs';
import { join, extname, dirname } from 'path';
import { gzipSync, brotliCompressSync, constants } from 'zlib';
import { fileURLToPath } from 'url';
import chalk from 'chalk';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const ROOT_DIR = join(__dirname, '..');
const DIST_DIR = join(ROOT_DIR, 'dist');

// File extensions to compress
const COMPRESSIBLE_EXTENSIONS = ['.wasm', '.js', '.html', '.css', '.json', '.svg'];

// Files to skip
const SKIP_FILES = ['service-worker.js']; // Service worker should not be pre-compressed

let totalOriginalSize = 0;
let totalGzipSize = 0;
let totalBrotliSize = 0;
let filesCompressed = 0;

function formatBytes(bytes) {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
}

function compressFile(filePath) {
  const fileName = filePath.split('/').pop();

  // Skip if in skip list
  if (SKIP_FILES.includes(fileName)) {
    console.log(chalk.gray('⊘'), chalk.gray(`Skipping: ${fileName}`));
    return;
  }

  // Check if file extension is compressible
  const ext = extname(filePath);
  if (!COMPRESSIBLE_EXTENSIONS.includes(ext)) {
    return;
  }

  try {
    // Read original file
    const originalBuffer = readFileSync(filePath);
    const originalSize = originalBuffer.length;

    // Skip very small files (< 1KB)
    if (originalSize < 1024) {
      return;
    }

    // Gzip compression (level 9)
    const gzipBuffer = gzipSync(originalBuffer, {
      level: constants.Z_BEST_COMPRESSION
    });
    writeFileSync(`${filePath}.gz`, gzipBuffer);

    // Brotli compression (quality 11 - maximum)
    const brotliBuffer = brotliCompressSync(originalBuffer, {
      params: {
        [constants.BROTLI_PARAM_QUALITY]: constants.BROTLI_MAX_QUALITY
      }
    });
    writeFileSync(`${filePath}.br`, brotliBuffer);

    // Calculate sizes and savings
    const gzipSize = gzipBuffer.length;
    const brotliSize = brotliBuffer.length;
    const gzipSavings = ((1 - gzipSize / originalSize) * 100).toFixed(1);
    const brotliSavings = ((1 - brotliSize / originalSize) * 100).toFixed(1);

    // Update totals
    totalOriginalSize += originalSize;
    totalGzipSize += gzipSize;
    totalBrotliSize += brotliSize;
    filesCompressed++;

    // Print results
    console.log(chalk.green('✓'), fileName);
    console.log(chalk.gray(`  Original:  ${formatBytes(originalSize)}`));
    console.log(chalk.cyan(`  Gzip:      ${formatBytes(gzipSize)} (${gzipSavings}% savings)`));
    console.log(chalk.blue(`  Brotli:    ${formatBytes(brotliSize)} (${brotliSavings}% savings)`));

  } catch (error) {
    console.log(chalk.red('✗'), `Error compressing ${fileName}:`, error.message);
  }
}

function compressDirectory(dir) {
  const items = readdirSync(dir);

  for (const item of items) {
    const itemPath = join(dir, item);
    const stats = statSync(itemPath);

    if (stats.isDirectory()) {
      // Skip certain directories
      if (item === 'node_modules' || item === '.git') {
        continue;
      }
      compressDirectory(itemPath);
    } else if (stats.isFile()) {
      compressFile(itemPath);
    }
  }
}

console.log(chalk.bold.cyan('\n=== Asset Compression ===\n'));

// Compress all files in dist directory
compressDirectory(DIST_DIR);

// Summary
console.log(chalk.bold('\n=== Compression Summary ===\n'));

if (filesCompressed === 0) {
  console.log(chalk.yellow('No files compressed'));
  process.exit(0);
}

const totalGzipSavings = ((1 - totalGzipSize / totalOriginalSize) * 100).toFixed(1);
const totalBrotliSavings = ((1 - totalBrotliSize / totalOriginalSize) * 100).toFixed(1);

console.log(chalk.gray(`Files compressed: ${filesCompressed}`));
console.log(chalk.gray(`\nTotal original size:  ${formatBytes(totalOriginalSize)}`));
console.log(chalk.cyan(`Total Gzip size:      ${formatBytes(totalGzipSize)} (${totalGzipSavings}% savings)`));
console.log(chalk.blue(`Total Brotli size:    ${formatBytes(totalBrotliSize)} (${totalBrotliSavings}% savings)`));

console.log(chalk.green.bold('\n✓ Compression complete!\n'));
