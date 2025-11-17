# Rusty-Audio Test Suite Documentation

## Overview

This document describes the comprehensive test suite for rusty-audio, covering backend architecture, frontend UI, and cross-platform compatibility.

## Test Coverage Goals

- **Overall Target**: 85%+ code coverage (per CLAUDE.md requirement)
- **Critical Paths**: 95%+ coverage (audio callbacks, EQ processing, spectrum analysis)
- **UI Components**: 70%+ coverage (egui interaction testing)
- **Platform-Specific**: 80%+ coverage per platform

## Test Organization

```
tests/
├── backend_trait_tests.rs              # AudioBackend trait dyn-safety
├── audio_graph_integration_tests.rs    # Audio routing integration
├── eq_functionality_tests.rs           # EQ filter tests
├── theme_system_tests.rs               # Theme colors and gradients
├── spectrum_analyzer_tests.rs          # FFT and visualization
├── signal_generator_tests.rs           # Waveform generation
├── platform_specific_tests.rs          # CPAL/ASIO/WASM tests
├── ui_interaction_tests.rs             # egui_kittest UI tests
├── property_based_tests.rs             # Property-based testing
└── regression_tests.rs                 # PR #5 bug verification

benches/
├── backend_benchmarks.rs               # Backend switching overhead
├── eq_processing_benchmarks.rs         # EQ filter performance
└── spectrum_fft_benchmarks.rs          # FFT computation speed
```

## Running Tests

### Quick Test Commands

```bash
# Run all tests
cargo test

# Run specific test file
cargo test --test backend_trait_tests

# Run specific test
cargo test test_backend_trait_is_dyn_safe

# Run with output
cargo test -- --nocapture

# Run tests in parallel
cargo test -- --test-threads=8
```

### Test Categories

#### 1. Unit Tests
```bash
# All unit tests
cargo test --lib

# Specific module
cargo test audio::backend::tests
cargo test ui::theme::tests
```

#### 2. Integration Tests
```bash
# Audio graph integration
cargo test --test audio_graph_integration_tests

# Backend trait polymorphism
cargo test --test backend_trait_tests
```

#### 3. Property-Based Tests
```bash
# Run all property tests
cargo test --test property_based_tests

# Increase test cases for thorough testing
PROPTEST_CASES=10000 cargo test --test property_based_tests
```

#### 4. Platform-Specific Tests
```bash
# Desktop tests (CPAL)
cargo test --test platform_specific_tests -- desktop_tests

# Windows ASIO tests
cargo test --test platform_specific_tests --features asio -- windows_tests

# WASM tests
wasm-pack test --headless --chrome
```

#### 5. Regression Tests
```bash
# Verify PR #5 fixes
cargo test --test regression_tests
```

#### 6. UI Tests (with egui_kittest)
```bash
# Interactive UI tests
cargo test --test ui_interaction_tests
```

### Benchmarks

```bash
# Run all benchmarks
cargo bench

# Specific benchmark suite
cargo bench --bench audio_benchmarks
cargo bench --bench performance_benchmarks
cargo bench --bench realtime_benchmarks

# Compare against baseline
cargo bench -- --save-baseline main
cargo bench -- --baseline main
```

## Test Coverage Analysis

### Generate Coverage Report

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate HTML coverage report
cargo tarpaulin --out Html --all-features

# Generate and open report
cargo tarpaulin --out Html --all-features && open tarpaulin-report.html

# Check coverage threshold (85%)
cargo tarpaulin --all-features | grep "coverage"
```

### Coverage Targets by Module

| Module | Target Coverage | Critical |
|--------|----------------|----------|
| `audio::backend` | 95% | Yes |
| `audio::device` | 90% | Yes |
| `audio::hybrid` | 95% | Yes |
| `audio::eq` | 95% | Yes |
| `audio::web_bridge` | 90% | Yes |
| `ui::spectrum` | 85% | No |
| `ui::theme` | 80% | No |
| `ui::controls` | 75% | No |
| `testing/*` | 100% | No |

## Critical Test Cases

### 1. AudioBackend Trait Dyn-Safety (Regression)

**Why Critical**: PR #5 fixed trait object-safety. Regression would break polymorphism.

**Test**: `regression_pr5_backend_trait_dyn_safety`

**Verification**:
```rust
let backend: Box<dyn AudioBackend> = Box::new(HybridAudioBackend::new().unwrap());
assert!(backend.backend_type() != BackendType::Unknown);
```

### 2. EQ Audio Graph Connection (Regression)

**Why Critical**: PR #5 fixed EQ having no effect due to disconnected audio graph.

**Test**: `regression_pr5_eq_audio_graph_connection`

**Verification**:
```rust
backend.set_eq_band(4, 6.0).unwrap();
backend.play().unwrap();
// Verify spectrum shows modified signal
let spectrum = backend.get_spectrum_data();
assert!(max(spectrum) > 0.0);
```

### 3. Windows MMCSS HANDLE Import (Regression)

**Why Critical**: PR #5 fixed Windows compilation with ASIO.

**Test**: `regression_pr5_windows_mmcss_handle_import`

**Verification**: Test compiles on Windows with `--features asio`

### 4. Theme Application (Regression)

**Why Critical**: PR #5 fixed themes not applying to UI.

**Test**: `regression_pr5_theme_application`

**Verification**:
```rust
let visuals = theme.to_egui_visuals();
assert!(visuals.dark_mode == expected_dark_mode);
```

### 5. Spectrum Gradient Smoothness (Regression)

**Why Critical**: PR #5 fixed spectrum showing solid colors instead of gradient.

**Test**: `regression_pr5_spectrum_gradient_smoothness`

**Verification**: Colors interpolate smoothly from blue → cyan → red

## Property-Based Testing Strategies

### EQ Gain Clamping
```rust
proptest! {
    fn prop_eq_gain_clamping(gain in -20.0f32..20.0) {
        let clamped = gain.clamp(-12.0, 12.0);
        prop_assert!(clamped >= -12.0 && clamped <= 12.0);
    }
}
```

**Invariants**:
- Gain must be in range [-12dB, +12dB]
- Values outside range are clamped
- 0dB = unity gain

### Spectrum Normalization
```rust
proptest! {
    fn prop_spectrum_normalization(raw in 0.0f32..10000.0) {
        let normalized = (raw / max_value).min(1.0);
        prop_assert!(normalized >= 0.0 && normalized <= 1.0);
    }
}
```

**Invariants**:
- All spectrum values in [0.0, 1.0]
- 512 frequency bins (FFT size / 2)
- Monotonic increase with frequency (for white noise)

### Volume Application
```rust
proptest! {
    fn prop_volume_application(sample in -1.0..1.0, volume in 0.0..1.0) {
        let output = sample * volume;
        prop_assert!(output.abs() <= sample.abs());
    }
}
```

**Invariants**:
- Volume in [0.0, 1.0]
- Output magnitude ≤ input magnitude
- Volume 0.0 = silence
- Volume 1.0 = unity gain

## Platform-Specific Testing

### Desktop (CPAL)
```bash
# Linux (ALSA/PulseAudio)
cargo test --test platform_specific_tests -- desktop_tests::linux_tests

# Windows (WASAPI)
cargo test --test platform_specific_tests -- desktop_tests::windows_tests

# macOS (CoreAudio)
cargo test --test platform_specific_tests -- desktop_tests::macos_tests
```

**Checks**:
- Device enumeration
- Default device selection
- Low-latency configuration
- Exclusive mode (where supported)

### Windows ASIO
```bash
cargo test --features asio --test platform_specific_tests -- windows_tests::test_asio_backend
```

**Checks**:
- ASIO SDK compilation
- MMCSS thread registration
- Low-latency buffer configuration
- Multi-channel support

### WASM (Web Audio API)
```bash
wasm-pack test --headless --chrome --test platform_specific_tests
```

**Checks**:
- AudioContext creation
- BiquadFilterNode (EQ) creation
- AnalyserNode (spectrum) creation
- Audio graph connection: Source → EQ → Analyser → Destination
- User gesture requirement

## UI Testing with egui_kittest

### Tab Navigation
```rust
harness.get_by_label("Playback").click();
harness.run();
assert!(harness.node_exists("File Selection"));
```

### EQ Slider Interaction
```rust
let slider = harness.get_by_label("60 Hz");
slider.drag(100.0, 0.0); // Drag to +6dB
assert_eq!(backend.get_eq_band(0), 6.0);
```

### Theme Selection
```rust
let dropdown = harness.get_by_label("Theme");
dropdown.select("Mocha");
let visuals = ctx.style().visuals.clone();
assert!(visuals.dark_mode);
```

## Performance Benchmarks

### EQ Processing Benchmark
```bash
cargo bench --bench eq_processing_benchmarks
```

**Metrics**:
- 8-band EQ processing time per buffer
- Target: <1ms for 512 samples @ 48kHz
- Memory allocations: 0 in hot path

### Spectrum FFT Benchmark
```bash
cargo bench --bench spectrum_fft_benchmarks
```

**Metrics**:
- 512-point FFT computation time
- Target: <2ms per frame
- Gradient interpolation: <0.1ms

### Backend Switching Benchmark
```bash
cargo bench --bench backend_benchmarks
```

**Metrics**:
- Backend creation time
- Backend switching latency
- Memory overhead per backend type

## Continuous Integration

### GitHub Actions Workflow

The `.github/workflows/test.yml` pipeline runs:

1. **Unit Tests** (Linux, Windows, macOS) - All unit tests across platforms
2. **Integration Tests** - Audio graph, backend polymorphism
3. **Property Tests** - 1000 cases per property (configurable via PROPTEST_CASES)
4. **Regression Tests** - PR #5 bug verification
5. **Platform Tests** - Windows (ASIO), Linux (ALSA), macOS (CoreAudio)
6. **WASM Tests** - Chrome and Firefox headless browsers
7. **Benchmarks** - Compilation and quick run
8. **Coverage** - Tarpaulin with 85% threshold
9. **Quality** - rustfmt, clippy, doc warnings
10. **Security** - cargo-audit for vulnerabilities

### Coverage Threshold Enforcement

```yaml
- name: Check coverage threshold
  run: |
    COVERAGE=$(cargo tarpaulin | grep -oP '\d+\.\d+(?=% coverage)')
    if (( $(echo "$COVERAGE < 85.0" | bc -l) )); then
      echo "ERROR: Coverage below 85%"
      exit 1
    fi
```

## Test Data and Fixtures

### Test Audio Files
```
test_assets/
├── sine_1khz.wav       # Pure 1kHz sine wave (10s)
├── pink_noise.wav      # Pink noise (5s)
├── sweep_20_20k.wav    # Frequency sweep (30s)
└── silence.wav         # Digital silence (1s)
```

### Generating Test Data
```bash
# Use signal generator to create test files
cargo run --bin test_audio -- generate sine 1000 10.0 test_assets/sine_1khz.wav
cargo run --bin test_audio -- generate pink 0 5.0 test_assets/pink_noise.wav
```

## Debugging Tests

### Run Single Test with Debug Output
```bash
cargo test test_eq_audio_graph_connection -- --nocapture --exact
```

### Run Tests with Backtrace
```bash
RUST_BACKTRACE=1 cargo test
RUST_BACKTRACE=full cargo test  # Full backtrace
```

### Run Tests in Single Thread
```bash
cargo test -- --test-threads=1
```

### Ignore Failing Tests Temporarily
```rust
#[test]
#[ignore]
fn test_flaky_feature() {
    // ...
}

// Run ignored tests
cargo test -- --ignored
```

## Test Maintenance Checklist

- [ ] Run full test suite before PR: `cargo test --all-features`
- [ ] Check coverage: `cargo tarpaulin --all-features`
- [ ] Run benchmarks: `cargo bench`
- [ ] Format code: `cargo fmt`
- [ ] Lint code: `cargo clippy -- -D warnings`
- [ ] Update regression tests for new bugs
- [ ] Document new test cases in this file
- [ ] Verify platform-specific tests on target platforms
- [ ] Test WASM build: `wasm-pack test --headless --chrome`

## Troubleshooting

### Common Issues

**Issue**: Tests fail with "No audio devices found"
**Solution**: Tests may fail in headless CI. Use mock backends or skip device enumeration tests.

**Issue**: WASM tests timeout
**Solution**: Increase timeout in `wasm-pack test --timeout 120`

**Issue**: Coverage below 85%
**Solution**: Identify uncovered code with `cargo tarpaulin --out Html` and add tests

**Issue**: Benchmarks fail in CI
**Solution**: Use `continue-on-error: true` in CI, benchmarks need consistent environment

**Issue**: Property tests find edge case
**Solution**: Shrink failing input with `PROPTEST_VERBOSE=1`, add regression test

## Performance Testing

### Real-Time Audio Constraints

```bash
# Run real-time constraint benchmarks
cargo bench --bench realtime_benchmarks
```

**Requirements**:
- Buffer processing time < buffer duration (no xruns)
- 512 samples @ 48kHz = 10.67ms budget
- Target: <5ms processing time (50% headroom)

**Metrics**:
- P50 latency: <3ms
- P99 latency: <8ms
- P99.9 latency: <10ms

### Memory Usage Benchmarks

```bash
cargo bench --bench memory_benchmarks
```

**Targets**:
- Heap allocations in audio callback: 0
- Total memory usage: <50MB
- Memory growth over time: <1MB/hour

## Continuous Monitoring

### Performance Regression Detection

```bash
# Save baseline before changes
cargo bench -- --save-baseline before

# Make changes...

# Compare after changes
cargo bench -- --baseline before
```

### Coverage Tracking

Track coverage over time in CI:
```yaml
- name: Upload coverage to Codecov
  uses: codecov/codecov-action@v3
```

View trends at: `https://codecov.io/gh/your-org/rusty-audio`

## Contributing Tests

When adding new features:

1. **Write tests first** (TDD approach)
2. **Cover happy path and edge cases**
3. **Add property tests for algorithms**
4. **Update regression tests for bug fixes**
5. **Benchmark performance-critical code**
6. **Document test cases in this file**

### Test Naming Convention

```rust
// Unit tests
#[test]
fn test_eq_band_clamping() { }

// Integration tests
#[test]
fn test_audio_graph_complete_chain() { }

// Property tests
proptest! {
    fn prop_eq_gain_clamping(gain in -20.0..20.0) { }
}

// Regression tests
#[test]
fn regression_pr5_backend_trait_dyn_safety() { }

// Platform-specific tests
#[test]
#[cfg(target_os = "windows")]
fn test_windows_mmcss() { }
```

## Resources

- **proptest book**: https://proptest-rs.github.io/proptest/
- **egui_kittest**: https://docs.rs/egui_kittest/
- **wasm-pack testing**: https://rustwasm.github.io/wasm-pack/book/
- **cargo-tarpaulin**: https://github.com/xd009642/tarpaulin
- **criterion.rs**: https://bheisler.github.io/criterion.rs/book/

---

Last updated: 2025-01-16
