#!/usr/bin/env node

/**
 * Local Development Server for Rusty Audio WASM Application
 *
 * Features:
 * - Proper COOP/COEP/CORP headers for SharedArrayBuffer (multithreading)
 * - Hot reload support
 * - WASM module serving with correct MIME types
 * - Service worker support
 * - Performance monitoring
 * - Detailed logging
 *
 * Usage:
 *   node scripts/dev-server.js [--port 8080] [--verbose]
 */

import express from 'express';
import compression from 'compression';
import cors from 'cors';
import helmet from 'helmet';
import morgan from 'morgan';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';
import { existsSync, statSync } from 'fs';
import chalk from 'chalk';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const ROOT_DIR = join(__dirname, '..');
const DIST_DIR = join(ROOT_DIR, 'dist');

// Parse command line arguments
const args = process.argv.slice(2);
const PORT = args.includes('--port') ? parseInt(args[args.indexOf('--port') + 1]) : 8080;
const VERBOSE = args.includes('--verbose') || args.includes('-v');
const PROD_MODE = process.env.NODE_ENV === 'production';

// Create Express app
const app = express();

// Logging middleware
if (VERBOSE) {
  app.use(morgan('dev'));
} else {
  app.use(morgan('tiny'));
}

// Security middleware (with relaxed CSP for local development)
app.use(helmet({
  contentSecurityPolicy: false, // Disable CSP for local dev (enable in production)
  crossOriginEmbedderPolicy: true,
  crossOriginOpenerPolicy: true,
  crossOriginResourcePolicy: { policy: "cross-origin" }
}));

// CORS middleware (allow all origins in development)
app.use(cors({
  origin: '*',
  credentials: true
}));

// Compression middleware
app.use(compression({
  threshold: 1024, // Compress responses larger than 1KB
  level: 9 // Maximum compression
}));

/**
 * Critical Headers Middleware for Multithreading Support
 * These headers enable SharedArrayBuffer and cross-origin isolation
 */
app.use((req, res, next) => {
  // Cross-Origin Isolation headers (required for SharedArrayBuffer)
  res.setHeader('Cross-Origin-Opener-Policy', 'same-origin');
  res.setHeader('Cross-Origin-Embedder-Policy', 'require-corp');
  res.setHeader('Cross-Origin-Resource-Policy', 'cross-origin');

  // Security headers
  res.setHeader('X-Content-Type-Options', 'nosniff');
  res.setHeader('X-Frame-Options', 'DENY');
  res.setHeader('Referrer-Policy', 'no-referrer');
  res.setHeader('Permissions-Policy', 'autoplay=(self), microphone=(self), camera=(), geolocation=()');

  // WASM-specific headers
  if (req.path.endsWith('.wasm')) {
    res.setHeader('Content-Type', 'application/wasm');
    res.setHeader('Cache-Control', PROD_MODE ? 'public, max-age=31536000, immutable' : 'no-cache');
  }

  // JavaScript module headers
  if (req.path.endsWith('.js') || req.path.endsWith('.mjs')) {
    res.setHeader('Content-Type', 'application/javascript');
    res.setHeader('X-Content-Type-Options', 'nosniff');
  }

  // Service Worker special handling
  if (req.path === '/service-worker.js') {
    res.setHeader('Service-Worker-Allowed', '/');
    res.setHeader('Cache-Control', 'no-store, no-cache, must-revalidate');
  }

  // Log headers in verbose mode
  if (VERBOSE) {
    console.log(chalk.gray(`${req.method} ${req.path}`));
    console.log(chalk.cyan('Response Headers:'), {
      'COOP': res.getHeader('Cross-Origin-Opener-Policy'),
      'COEP': res.getHeader('Cross-Origin-Embedder-Policy'),
      'CORP': res.getHeader('Cross-Origin-Resource-Policy')
    });
  }

  next();
});

/**
 * Health Check Endpoint
 */
app.get('/health', (req, res) => {
  const health = {
    status: 'ok',
    timestamp: new Date().toISOString(),
    uptime: process.uptime(),
    environment: PROD_MODE ? 'production' : 'development',
    features: {
      multithreading: true,
      serviceWorker: true,
      wasm: true,
      compression: true
    },
    headers: {
      coop: 'same-origin',
      coep: 'require-corp',
      corp: 'cross-origin'
    }
  };

  res.json(health);
});

/**
 * Feature Detection Endpoint
 */
app.get('/api/features', (req, res) => {
  res.json({
    threading: {
      enabled: true,
      maxWorkers: parseInt(process.env.MAX_WORKER_THREADS || '8')
    },
    wasm: {
      streaming: true,
      bulk_memory: true,
      simd: true,
      threads: true
    },
    browser: {
      crossOriginIsolated: true,
      sharedArrayBuffer: true
    }
  });
});

/**
 * Performance Metrics Endpoint
 */
app.get('/api/metrics', (req, res) => {
  const metrics = {
    memory: process.memoryUsage(),
    uptime: process.uptime(),
    timestamp: Date.now()
  };

  res.json(metrics);
});

/**
 * WASM Info Endpoint
 */
app.get('/api/wasm/info', (req, res) => {
  const wasmPath = join(DIST_DIR, 'rusty_audio_bg.wasm');

  if (existsSync(wasmPath)) {
    const stats = statSync(wasmPath);
    res.json({
      exists: true,
      size: stats.size,
      sizeHuman: `${(stats.size / (1024 * 1024)).toFixed(2)} MB`,
      modified: stats.mtime,
      path: wasmPath
    });
  } else {
    res.status(404).json({
      exists: false,
      error: 'WASM binary not found. Run `npm run build:wasm` first.'
    });
  }
});

/**
 * Static file serving from dist directory
 */
app.use(express.static(DIST_DIR, {
  setHeaders: (res, path) => {
    // Additional caching headers for static assets
    if (path.endsWith('.wasm')) {
      res.setHeader('Cache-Control', PROD_MODE ? 'public, max-age=31536000, immutable' : 'no-cache');
    } else if (path.endsWith('.js') && !path.includes('service-worker')) {
      res.setHeader('Cache-Control', PROD_MODE ? 'public, max-age=31536000, immutable' : 'no-cache');
    } else if (path.match(/\.(png|jpg|jpeg|gif|svg|ico|webp)$/)) {
      res.setHeader('Cache-Control', 'public, max-age=86400'); // 1 day for images
    }
  }
}));

/**
 * SPA fallback - serve index.html for all non-API routes
 */
app.get('*', (req, res) => {
  // Skip API routes
  if (req.path.startsWith('/api/') || req.path.startsWith('/health')) {
    return res.status(404).json({ error: 'Not found' });
  }

  const indexPath = join(DIST_DIR, 'index.html');

  if (existsSync(indexPath)) {
    res.sendFile(indexPath);
  } else {
    res.status(500).send(`
      <html>
        <head><title>Build Required</title></head>
        <body style="font-family: system-ui; padding: 2rem; max-width: 800px; margin: 0 auto;">
          <h1>🛠️ Build Required</h1>
          <p>The WASM application hasn't been built yet.</p>
          <h2>Run:</h2>
          <pre style="background: #f5f5f5; padding: 1rem; border-radius: 4px;">npm run build:wasm</pre>
          <p>Then refresh this page.</p>
        </body>
      </html>
    `);
  }
});

/**
 * Error handling middleware
 */
app.use((err, req, res, next) => {
  console.error(chalk.red('Server Error:'), err);
  res.status(500).json({
    error: 'Internal Server Error',
    message: VERBOSE ? err.message : undefined
  });
});

/**
 * Start the server
 */
app.listen(PORT, '0.0.0.0', () => {
  console.log(chalk.green('\n✓ Rusty Audio Development Server Started\n'));
  console.log(chalk.cyan('  Server URL:'), chalk.bold(`http://localhost:${PORT}`));
  console.log(chalk.cyan('  Health Check:'), `http://localhost:${PORT}/health`);
  console.log(chalk.cyan('  Features API:'), `http://localhost:${PORT}/api/features`);
  console.log(chalk.cyan('  WASM Info:'), `http://localhost:${PORT}/api/wasm/info`);
  console.log(chalk.cyan('  Mode:'), PROD_MODE ? chalk.yellow('Production') : chalk.green('Development'));
  console.log(chalk.cyan('  Verbose:'), VERBOSE ? chalk.yellow('Yes') : chalk.gray('No'));

  console.log(chalk.green('\n✓ Multithreading Headers Enabled:'));
  console.log(chalk.gray('  • Cross-Origin-Opener-Policy: same-origin'));
  console.log(chalk.gray('  • Cross-Origin-Embedder-Policy: require-corp'));
  console.log(chalk.gray('  • Cross-Origin-Resource-Policy: cross-origin'));

  // Check if WASM binary exists
  const wasmPath = join(DIST_DIR, 'rusty_audio_bg.wasm');
  if (!existsSync(wasmPath)) {
    console.log(chalk.yellow('\n⚠️  WASM binary not found!'));
    console.log(chalk.gray('   Run: npm run build:wasm\n'));
  } else {
    const stats = statSync(wasmPath);
    console.log(chalk.green(`\n✓ WASM Binary: ${(stats.size / (1024 * 1024)).toFixed(2)} MB\n`));
  }

  console.log(chalk.gray('Press Ctrl+C to stop\n'));
});

// Graceful shutdown
process.on('SIGTERM', () => {
  console.log(chalk.yellow('\n⚠️  Shutting down gracefully...'));
  process.exit(0);
});

process.on('SIGINT', () => {
  console.log(chalk.yellow('\n⚠️  Shutting down gracefully...'));
  process.exit(0);
});
