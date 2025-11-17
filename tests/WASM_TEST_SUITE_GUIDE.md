# WASM Audio Application Test Suite Guide

## Overview

This document provides comprehensive guidance on the WASM test suite for the rusty-audio application. The test suite validates all critical fixes from the WASM refactoring and ensures production readiness.

## Test Suite Structure

### 1. Unit Tests (Rust - `tests/wasm_*.rs`)

#### WorkerPool Tests (`wasm_worker_pool_tests.rs`)
**Purpose**: Validate WorkerPool implementation for WASM multithreading

**Test Coverage**:
- ✅ Initialization without deadlock
- ✅ Proper cleanup on drop
- ✅ Concurrent initialization attempts
- ✅ Worker count configuration
- ✅ Failure recovery
- ✅ Thread safety with Arc/Mutex
- ✅ State consistency

**Running**:
```bash
# Run all worker pool tests
wasm-pack test --headless --chrome tests/wasm_worker_pool_tests.rs

# Run specific test
wasm-pack test --headless --chrome tests/wasm_worker_pool_tests.rs -- test_worker_pool_initialization
```

**Expected Results**:
- All tests pass
- No deadlocks
- Worker count ≥ 1 and ≤ 16
- Initialization is idempotent

#### SharedAudioBuffer Tests (`wasm_shared_audio_buffer_tests.rs`)
**Purpose**: Validate buffer pooling and thread-safe memory management

**Test Coverage**:
- ✅ Buffer pooling (no unbounded growth)
- ✅ Thread-safe read/write operations
- ✅ Buffer reuse and cleanup
- ✅ Memory bounds validation
- ✅ Concurrent access patterns
- ✅ Buffer pool size limits

**Running**:
```bash
wasm-pack test --headless --chrome tests/wasm_shared_audio_buffer_tests.rs
```

**Expected Results**:
- Buffer pool size ≤ configured maximum
- No memory leaks after 100 iterations
- Read/write operations are thread-safe
- Bounds checking prevents overruns

#### AudioContext Tests (`wasm_audio_context_tests.rs`)
**Purpose**: Validate thread-safe AudioContext management

**Test Coverage**:
- ✅ Main thread assertion
- ✅ Initialization guard
- ✅ Concurrent access protection
- ✅ Proper cleanup on close
- ✅ Clone safety
- ✅ State consistency

**Running**:
```bash
wasm-pack test --headless --chrome tests/wasm_audio_context_tests.rs
```

**Expected Results**:
- AudioContext created successfully on main thread
- Multiple initialization attempts are safe
- Clone operations maintain shared state
- Cleanup releases resources

#### Panic Boundary Tests (`wasm_panic_boundary_tests.rs`)
**Purpose**: Validate panic handling at WASM boundaries

**Test Coverage**:
- ✅ Panic is caught at WASM boundary
- ✅ Error propagation to JS
- ✅ Recovery after panic
- ✅ No memory corruption
- ✅ Result-based error handling

**Running**:
```bash
wasm-pack test --headless --chrome tests/wasm_panic_boundary_tests.rs
```

**Expected Results**:
- Panics are caught and converted to JsValue errors
- Application recovers after panic
- Memory remains valid after panic
- Subsequent operations work correctly

#### Memory Management Tests (`wasm_memory_management_tests.rs`)
**Purpose**: Validate memory management and leak prevention

**Test Coverage**:
- ✅ No leaks over sustained usage
- ✅ Buffer pool stabilizes
- ✅ Memory bounds checking
- ✅ Allocation/deallocation tracking
- ✅ Memory pressure recovery
- ✅ GC cooperation

**Running**:
```bash
wasm-pack test --headless --chrome tests/wasm_memory_management_tests.rs
```

**Expected Results**:
- Memory usage stabilizes within limits
- No unbounded growth detected
- Pool size ≤ configured maximum
- Proper cleanup on buffer release

### 2. Integration Tests (TypeScript - `tests/e2e/*.spec.ts`)

#### Multithreading Tests (`e2e/multithreading.spec.ts`)
**Purpose**: Validate WASM threading and worker pool in browser

**Test Coverage**:
- SharedArrayBuffer detection
- Worker pool initialization
- Hardware-appropriate worker count
- Cross-origin isolation
- Graceful fallback to single-threaded mode

**Running**:
```bash
npm test -- multithreading.spec.ts
```

**Expected Results**:
- Worker pool initializes with SharedArrayBuffer
- Worker count ≤ navigator.hardwareConcurrency
- Cross-origin isolated = true
- Fallback works without SharedArrayBuffer

#### Performance Tests (`e2e/performance.spec.ts`)
**Purpose**: Validate performance benchmarks

**Test Coverage**:
- WASM initialization time < 3s
- Steady-state FPS ≥ 55
- Memory usage < 200MB
- Audio latency < 50ms
- Frame time consistency
- Time to interactive < 5s

**Running**:
```bash
npm test -- performance.spec.ts
```

**Expected Results**:
- All performance targets met
- No memory leaks over 30s
- FPS remains stable under load

#### Audio Functionality Tests (`e2e/audio-functionality.spec.ts`)
**Purpose**: Validate audio pipeline

**Test Coverage**:
- Signal generation
- EQ processing correctness
- Spectrum analyzer accuracy
- Recording functionality
- Audio routing

**Expected Results**:
- Audio plays without errors
- EQ affects frequency response
- Spectrum data is valid
- Recording produces valid WAV file

### 3. Performance Benchmarks (Criterion - `benches/`)

#### WASM-Specific Benchmarks
**Purpose**: Track performance regressions

**Coverage**:
- FFT processing throughput
- Worker task distribution
- Buffer allocation latency
- Memory access patterns

**Running**:
```bash
cargo bench --target wasm32-unknown-unknown
```

**Expected Results**:
- FFT processing < 10ms for 2048 samples
- Worker spawn time < 100ms
- Buffer allocation < 1ms

## Test Execution

### Local Development

#### Run All Unit Tests
```bash
# WASM tests (requires Chrome/Firefox)
wasm-pack test --headless --chrome
wasm-pack test --headless --firefox

# Native tests
cargo test
```

#### Run All E2E Tests
```bash
npm test
```

#### Run Specific Test Suites
```bash
# Only multithreading tests
npm test -- multithreading.spec.ts

# Only performance tests
npm test -- performance.spec.ts

# With specific browser
npm test -- --project=chromium
npm test -- --project=firefox
```

### CI/CD Integration (GitHub Actions)

#### Workflow Configuration (`.github/workflows/wasm-tests.yml`)
```yaml
name: WASM Test Suite

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  unit-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          target: wasm32-unknown-unknown

      - name: Install wasm-pack
        run: curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

      - name: Run WASM unit tests
        run: |
          wasm-pack test --headless --chrome tests/wasm_worker_pool_tests.rs
          wasm-pack test --headless --chrome tests/wasm_shared_audio_buffer_tests.rs
          wasm-pack test --headless --chrome tests/wasm_audio_context_tests.rs
          wasm-pack test --headless --chrome tests/wasm_panic_boundary_tests.rs
          wasm-pack test --headless --chrome tests/wasm_memory_management_tests.rs

  e2e-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '18'

      - name: Install dependencies
        run: npm ci
        working-directory: tests

      - name: Install Playwright browsers
        run: npx playwright install --with-deps
        working-directory: tests

      - name: Run E2E tests
        run: npm test
        working-directory: tests

      - name: Upload test results
        if: always()
        uses: actions/upload-artifact@v3
        with:
          name: playwright-report
          path: tests/playwright-report/

  performance-benchmarks:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Run benchmarks
        run: cargo bench --target wasm32-unknown-unknown

      - name: Store benchmark results
        uses: benchmark-action/github-action-benchmark@v1
        with:
          tool: 'cargo'
          output-file-path: target/criterion/output.txt
```

## Coverage Requirements

### Overall Coverage Targets
- **Unit test coverage**: ≥ 85%
- **Integration test coverage**: ≥ 70%
- **Critical path coverage**: 100%
- **Error path coverage**: 100%

### Measuring Coverage
```bash
# Install tarpaulin for coverage
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html --output-dir target/coverage

# View report
open target/coverage/index.html
```

## Test Utilities

### WASM Test Helpers (`tests/helpers/wasm-fixtures.ts`)

#### Available Helpers
- `detectBrowserFeatures()` - Detect WASM features
- `waitForWasmInit()` - Wait for WASM initialization
- `getWorkerPoolStatus()` - Get worker pool state
- `getPerformanceMetrics()` - Get performance data
- `monitorFPS()` - Monitor frame rate
- `checkMemoryLeak()` - Detect memory leaks
- `takePerformanceSnapshot()` - Capture performance snapshot

#### Usage Example
```typescript
import { waitForWasmInit, getPerformanceMetrics } from '../helpers/wasm-fixtures';

test('performance test', async ({ page }) => {
  await page.goto('/');
  await waitForWasmInit(page);

  const metrics = await getPerformanceMetrics(page);
  expect(metrics.fps).toBeGreaterThanOrEqual(55);
});
```

## Debugging Failed Tests

### Common Issues

#### 1. WASM Initialization Timeout
**Symptom**: Tests fail with "WASM init timeout"
**Solution**:
```typescript
// Increase timeout
await waitForWasmInit(page, 60000); // 60 seconds
```

#### 2. SharedArrayBuffer Not Available
**Symptom**: Worker pool tests fail
**Solution**:
- Ensure COOP/COEP headers are set
- Check browser supports SharedArrayBuffer
- Test fallback to single-threaded mode

#### 3. Memory Leak Detection False Positive
**Symptom**: Memory leak test fails intermittently
**Solution**:
- Trigger manual GC before measurement
- Increase measurement duration
- Adjust growth threshold

#### 4. Performance Test Flakiness
**Symptom**: FPS/latency tests fail randomly
**Solution**:
- Let performance stabilize longer
- Use median instead of single sample
- Run on isolated CI runner

### Debug Logging

#### Enable WASM Debug Logs
```rust
// In your WASM code
console_log::init_with_level(log::Level::Debug).ok();
log::debug!("Worker pool initialized");
```

#### View Logs in Tests
```bash
# Run tests with debug output
RUST_LOG=debug wasm-pack test --headless --chrome
```

## Best Practices

### Writing New Tests

1. **Use `wasm_bindgen_test`** for WASM-specific tests
2. **Install panic hook** in tests that may panic
3. **Clean up resources** after each test
4. **Use async/await** for asynchronous operations
5. **Test both success and error paths**

### Example Test Template
```rust
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::*;

#[cfg(target_arch = "wasm32")]
wasm_bindgen_test_configure!(run_in_browser);

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test]
async fn test_my_feature() {
    // Setup
    console_error_panic_hook::set_once();

    // Test
    let result = my_function().await;

    // Assert
    assert!(result.is_ok());

    // Cleanup (if needed)
}
```

## Continuous Improvement

### Performance Tracking
- Benchmark results stored in git
- Regression detection on PR
- Performance trends dashboard

### Test Maintenance
- Review flaky tests monthly
- Update browser versions quarterly
- Add tests for new features
- Remove obsolete tests

### Coverage Monitoring
- Track coverage in CI
- Require coverage increase on PR
- Identify uncovered code paths

## Troubleshooting

### Test Environment Issues

#### Docker Setup (for consistent CI environment)
```dockerfile
FROM rust:latest

RUN rustup target add wasm32-unknown-unknown
RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
RUN curl -fsSL https://deb.nodesource.com/setup_18.x | bash -
RUN apt-get install -y nodejs chromium

WORKDIR /app
COPY . .

CMD ["./tests/run-tests.sh"]
```

#### Run Tests in Docker
```bash
docker build -t rusty-audio-tests .
docker run --rm rusty-audio-tests
```

## Test Results Interpretation

### Success Criteria
- ✅ All unit tests pass
- ✅ All E2E tests pass
- ✅ Performance benchmarks within targets
- ✅ No memory leaks detected
- ✅ Coverage requirements met

### Failure Analysis
- 🔴 **Red**: Critical failure, blocks merge
- 🟡 **Yellow**: Warning, investigate
- 🟢 **Green**: All tests pass

## Resources

### Documentation
- [wasm-bindgen-test Guide](https://rustwasm.github.io/wasm-bindgen/wasm-bindgen-test/index.html)
- [Playwright API](https://playwright.dev/docs/api/class-test)
- [Criterion Benchmarks](https://bheisler.github.io/criterion.rs/book/)

### Tools
- [wasm-pack](https://rustwasm.github.io/wasm-pack/)
- [Playwright](https://playwright.dev/)
- [Chrome DevTools](https://developer.chrome.com/docs/devtools/)

### Support
- Report test failures in GitHub Issues
- Discuss test strategy in team meetings
- Update this guide with new findings

---

**Last Updated**: 2025-11-16
**Maintainer**: Test Automation Team
**Version**: 1.0.0
