# Playwright E2E Tests for Rusty Audio WASM

Comprehensive end-to-end testing suite for the Rusty Audio WASM application using Playwright.

## Overview

This test suite validates the entire WASM audio application across multiple browsers, testing:

- **WASM Loading & Initialization**: Binary download, compilation, and initialization
- **Multithreading**: SharedArrayBuffer, worker pool, thread-safe operations
- **Audio Functionality**: Signal generation, EQ, spectrum analysis, recording
- **UI Rendering**: egui canvas rendering, FPS, interactions, responsiveness
- **Performance**: Load times, memory usage, FPS stability, audio latency

## Test Structure

```
tests/
├── e2e/                              # Test specifications
│   ├── wasm-loading.spec.ts         # WASM initialization tests
│   ├── multithreading.spec.ts       # Threading validation tests
│   ├── audio-functionality.spec.ts  # Audio feature tests
│   ├── ui-rendering.spec.ts         # UI rendering tests
│   └── performance.spec.ts          # Performance benchmarks
├── helpers/                          # Test utilities
│   ├── wasm-fixtures.ts             # Reusable test fixtures
│   ├── global-setup.ts              # Global test setup
│   ├── global-teardown.ts           # Global test teardown
│   └── performance-reporter.ts      # Custom performance reporter
└── package.json                      # Test dependencies

playwright.config.ts                  # Playwright configuration
```

## Prerequisites

### 1. Build WASM Application

Before running tests, build the WASM application:

```bash
# Using Trunk (recommended)
trunk build --release

# Or using wasm-pack
wasm-pack build --target web --out-dir dist/pkg
```

### 2. Install Test Dependencies

```bash
cd tests/
npm install

# Install Playwright browsers
npm run install-browsers
```

## Running Tests

### Quick Start

```bash
# Run all tests
npm test

# Run specific browser
npm run test:chromium
npm run test:firefox
npm run test:webkit

# Run performance benchmarks only
npm run test:performance

# Run tests in headed mode (see browser)
npm run test:headed

# Debug tests with Playwright Inspector
npm run test:debug

# Interactive UI mode
npm run test:ui
```

### Advanced Usage

```bash
# Run specific test file
npx playwright test wasm-loading.spec.ts

# Run specific test
npx playwright test -g "should load WASM binary successfully"

# Run with specific number of workers
npx playwright test --workers=1

# Generate HTML report
npm run report

# Record new tests with Codegen
npm run codegen
```

## Test Categories

### 1. WASM Loading Tests (`wasm-loading.spec.ts`)

**What it tests:**
- WASM binary downloads correctly
- Initialization completes within 30 seconds
- No console errors during startup
- WASM compiles without validation errors
- Browser features are correctly detected
- Loading UI displays and hides properly
- wasm-bindgen initializes correctly
- Error handling works gracefully

**Key assertions:**
```typescript
expect(wasmLoaded).toBe(true);
expect(loadTime).toBeLessThan(30000);
expect(criticalErrors).toHaveLength(0);
```

### 2. Multithreading Tests (`multithreading.spec.ts`)

**What it tests:**
- SharedArrayBuffer availability detection
- Worker pool initialization
- Correct number of workers based on hardware
- Thread status UI display
- Cross-origin isolation headers
- Graceful fallback to single-threaded mode
- Atomics support validation
- Worker error handling

**Key assertions:**
```typescript
expect(features.sharedArrayBuffer).toBe(true);
expect(workerPoolStatus.initialized).toBe(true);
expect(workerPoolStatus.totalWorkers).toBeGreaterThan(0);
```

### 3. Audio Functionality Tests (`audio-functionality.spec.ts`)

**What it tests:**
- Web Audio API availability
- AudioContext creation
- Signal generator UI
- EQ controls (8 bands: 60Hz - 12kHz)
- Spectrum visualizer
- Playback controls (play, pause, stop)
- Volume and panning controls
- Audio latency (< 100ms)
- FFT spectrum analysis
- Multiple signal types (sine, square, sawtooth, noise)
- Recording functionality
- Memory leak prevention

**Key assertions:**
```typescript
expect(webAudioAvailable).toBe(true);
expect(metrics.audioLatency).toBeLessThan(100);
expect(audioErrors).toHaveLength(0);
```

### 4. UI Rendering Tests (`ui-rendering.spec.ts`)

**What it tests:**
- egui canvas renders correctly
- FPS maintains 60 target (55+ accepted)
- Frame time consistency (< 16.7ms avg)
- Window resize responsiveness
- Dark theme application
- Tab navigation
- Mouse and keyboard interactions
- No rendering memory leaks (< 20MB growth)
- Accessibility attributes (aria-label, role)
- High DPI display support
- WebGL/WebGPU context handling
- Mobile layout (responsive)

**Key assertions:**
```typescript
expect(dimensions.width).toBeGreaterThan(0);
expect(fpsStats.avg).toBeGreaterThanOrEqual(55);
expect(avgFrameTime).toBeLessThan(20);
```

### 5. Performance Benchmarks (`performance.spec.ts`)

**What it tests:**
- WASM initialization time (< 3s target)
- Steady-state FPS (60 FPS target)
- Memory usage (< 200MB)
- Memory leak detection (< 50MB growth over 30s)
- Audio latency (< 50ms)
- Frame time consistency (std dev < 5ms)
- Worker pool overhead
- Time to interactive (< 5s)
- Canvas rendering performance
- WASM binary size (< 10MB)
- First contentful paint (< 1.5s)

**Performance Grading:**
- A+: < 1000ms init time
- A: < 1500ms
- B: < 2000ms
- C: < 2500ms
- D: < 3000ms
- F: ≥ 3000ms

**Key assertions:**
```typescript
expect(loadTime).toBeLessThan(3000);
expect(fpsStats.avg).toBeGreaterThanOrEqual(55);
expect(memoryMB).toBeLessThan(200);
expect(leakCheck.leaked).toBe(false);
```

## Browser Support Matrix

| Browser | Threading | Web Audio | WebGPU | Status |
|---------|-----------|-----------|---------|--------|
| Chromium | ✅ | ✅ | ✅ | Full support |
| Firefox | ✅ | ✅ | ⚠️ | Full support (WebGPU experimental) |
| WebKit | ⚠️ | ✅ | ❌ | Limited threading |
| Mobile Chrome | ✅ | ✅ | ⚠️ | Full support |

## Test Configuration

### Playwright Config (`playwright.config.ts`)

**Key settings:**
- **Timeout**: 60s per test (WASM compilation can be slow)
- **Retries**: 2 in CI, 1 locally
- **Workers**: 1-2 (avoid audio device conflicts)
- **Screenshots**: On failure only
- **Trace**: On first retry (CI), retain on failure (local)

**Browser arguments (Chromium):**
```javascript
'--enable-features=SharedArrayBuffer',
'--enable-unsafe-webgpu',
'--use-fake-ui-for-media-stream',
'--use-fake-device-for-media-stream',
'--autoplay-policy=no-user-gesture-required'
```

## CI/CD Integration

### GitHub Actions Workflow

The test suite runs automatically on:
- Push to `main` or `develop` branches
- Pull requests
- Manual workflow dispatch

**Workflow stages:**
1. **Build WASM** - Compile Rust to WASM with Trunk
2. **Test Browsers** - Run tests on Chromium, Firefox, WebKit in parallel
3. **Performance** - Run performance benchmarks
4. **Publish Results** - Aggregate and publish HTML reports

**Artifacts preserved:**
- Test results (7 days)
- Performance data (30 days)
- HTML reports (14 days)
- Screenshots (7 days)

## Test Utilities

### `wasm-fixtures.ts`

**Key functions:**
- `detectBrowserFeatures()` - Detect WebAssembly, SharedArrayBuffer, etc.
- `waitForWasmInit()` - Wait for WASM initialization with timeout
- `getPerformanceMetrics()` - Get FPS, memory, latency metrics
- `getWorkerPoolStatus()` - Get worker pool statistics
- `monitorFPS()` - Monitor FPS over duration
- `checkMemoryLeak()` - Detect memory leaks
- `takePerformanceSnapshot()` - Comprehensive performance snapshot

### Custom Reporter

Performance metrics are automatically collected and saved to:
- `performance-data/performance-metrics.json` - Raw metrics
- `performance-data/performance-summary.json` - Aggregated summary

## Debugging Tests

### View Tests in UI Mode
```bash
npm run test:ui
```

### Debug Specific Test
```bash
npx playwright test --debug -g "should load WASM"
```

### Record New Tests
```bash
npm run codegen
```

### Check Test Traces
After test failure, view traces:
```bash
npx playwright show-trace test-results/*/trace.zip
```

## Performance Baselines

**Expected performance (Chromium on modern hardware):**
- Init time: 1-2 seconds
- FPS: 60 (±2)
- Frame time: 12-16ms
- Memory: 80-120MB
- Audio latency: 10-30ms

**Warning thresholds:**
- Init time > 2.5s
- FPS < 55
- Frame time > 20ms
- Memory > 150MB
- Audio latency > 50ms

## Troubleshooting

### WASM Not Loading

**Issue**: `WASM binary not found`

**Solution**:
```bash
trunk build --release
ls -la dist/*.wasm
```

### SharedArrayBuffer Not Available

**Issue**: Tests fail with "SharedArrayBuffer not available"

**Solution**: Ensure server sends correct headers:
```
Cross-Origin-Opener-Policy: same-origin
Cross-Origin-Embedder-Policy: require-corp
```

For local testing with `http-server`:
```bash
npx http-server dist -p 8080 --cors \
  -H "Cross-Origin-Opener-Policy: same-origin" \
  -H "Cross-Origin-Embedder-Policy: require-corp"
```

### Tests Timing Out

**Issue**: Tests exceed 60s timeout

**Possible causes**:
1. WASM build not optimized (use `--release`)
2. Network issues downloading WASM
3. Slow CI runner

**Solution**: Increase timeout in `playwright.config.ts`:
```typescript
timeout: 90000, // 90 seconds
```

### Audio Tests Failing

**Issue**: Audio-related tests fail

**Possible causes**:
1. Web Audio API not available
2. AutoPlay policy blocking audio
3. No fake audio devices in headless mode

**Solution**: Ensure browser args include:
```javascript
'--use-fake-ui-for-media-stream',
'--use-fake-device-for-media-stream',
'--autoplay-policy=no-user-gesture-required'
```

## Best Practices

1. **Always build WASM before testing**
   ```bash
   trunk build --release && npm test
   ```

2. **Run tests sequentially for audio tests**
   ```bash
   npx playwright test --workers=1
   ```

3. **Use headed mode for debugging**
   ```bash
   npm run test:headed
   ```

4. **Check performance regularly**
   ```bash
   npm run test:performance
   ```

5. **Review HTML reports after failures**
   ```bash
   npm run report
   ```

## Contributing

When adding new tests:

1. Place in appropriate spec file (`wasm-loading`, `audio-functionality`, etc.)
2. Use fixtures from `wasm-fixtures.ts` for consistency
3. Add performance metrics to `performance.spec.ts` if relevant
4. Update this README with new test descriptions
5. Ensure tests pass in all browsers (or mark browser-specific with `test.skip()`)

## License

Same as parent Rusty Audio project.
