# Performance Baseline Metrics

Expected performance characteristics for Rusty Audio based on benchmark infrastructure.

## Desktop Target Performance

### Audio Callback Latency

Target real-time performance for 48kHz sample rate, 512 samples per buffer:

| Component | Target | Critical | Current |
|-----------|--------|----------|---------|
| **Total Callback** | <5ms | <10ms | TBD |
| 8-Band EQ | <1ms | <2ms | TBD |
| 2048-Point FFT | <2ms | <4ms | TBD |
| Spectrum Smoothing | <0.5ms | <1ms | TBD |
| Buffer Pool Acquire | <10µs | <50µs | TBD |

**Buffer Size Calculations:**
- At 48kHz, 512 samples = 10.67ms of audio
- Processing must complete in <5ms for real-time with headroom
- Remaining 5.67ms for system overhead and other processing

### SIMD Operations Throughput

Expected throughput with AVX2 optimizations:

| Operation | Target Throughput | Expected Speedup |
|-----------|------------------|------------------|
| Biquad Filter | >10 Msamples/s | 8x vs scalar |
| Vector Add (SIMD) | >100 Melem/s | 8x vs scalar |
| Scalar Multiply | >100 Melem/s | 8x vs scalar |
| Peak Calculation | >50 Msamples/s | 8x vs scalar |
| RMS Calculation | >50 Msamples/s | 8x vs scalar |

### Memory Performance

| Metric | Target | Notes |
|--------|--------|-------|
| Peak Heap Usage | <100 MB | During normal playback |
| Audio Callback Allocations | **0** | Critical: No allocations in hot path |
| Buffer Pool Overhead | <5% | Memory efficiency |
| Cache Line Alignment | 64-byte aligned | AVX2 performance |

### FFT Processing

| FFT Size | Target Time | Throughput |
|----------|-------------|------------|
| 512-point | <0.5ms | >2000 frames/s |
| 1024-point | <1ms | >1000 frames/s |
| 2048-point | <2ms | >500 frames/s |
| 4096-point | <4ms | >250 frames/s |

## WASM Target Performance

### Bundle Size Targets

| Bundle Component | Target (gzipped) | Uncompressed | Notes |
|-----------------|------------------|--------------|-------|
| Initial Load | <500 KB | <1.2 MB | Core player functionality |
| Total Bundle | <1.5 MB | <4 MB | Full feature set |
| Lazy Modules | <200 KB | <600 KB | Optional features |

### Load Time Targets

| Metric | Target | Critical | Connection |
|--------|--------|----------|------------|
| Module Download | <100ms | <500ms | Broadband (5+ Mbps) |
| WASM Compilation | <50ms | <200ms | Modern browsers |
| Initialization | <50ms | <200ms | Audio context setup |
| **Total Time to Interactive** | <200ms | <1000ms | User experience |

### Runtime Performance

| Operation | Target | Notes |
|-----------|--------|-------|
| Audio Worklet Latency | <10ms | Same as desktop target |
| Memory Usage | <50 MB | Browser heap limit consideration |
| UI Frame Rate | >60 FPS | During spectrum visualization |
| Audio Dropout Rate | <0.1% | Network/buffer resilience |

## Optimization Phases Performance Gains

Based on optimization benchmark infrastructure:

### Phase 1.3: Pre-allocated Buffer Pool

| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| Buffer Acquire Time | TBD | TBD | Target: 90% faster |
| Allocation Count | TBD | 0 | Zero allocations |
| Memory Fragmentation | TBD | TBD | Target: <5% |

### Phase 3: Parallel EQ Processing

| Block Size | Sequential | Parallel | Speedup |
|------------|-----------|----------|---------|
| 128 samples | TBD | TBD | Target: 2x |
| 512 samples | TBD | TBD | Target: 3x |
| 2048 samples | TBD | TBD | Target: 4x |

Expected gains from rayon parallel processing across 8 EQ bands.

### Phase 4.1: Cache-Line Alignment

| Access Pattern | Unaligned | Aligned | Improvement |
|----------------|-----------|---------|-------------|
| Sequential Read | TBD | TBD | Target: 10-15% |
| Parallel Access | TBD | TBD | Target: 30-40% |
| SIMD Operations | TBD | TBD | Target: 5-10% |

### Phase 4.2: Zero-Copy Audio Pipeline

| Pipeline Stage | Traditional | Zero-Copy | Memory Saved |
|----------------|-------------|-----------|--------------|
| EQ Processing | TBD | TBD | 1 buffer copy |
| Spectrum Analysis | TBD | TBD | 1 buffer copy |
| **Total Latency** | TBD | TBD | Target: 20% faster |

## CPU Architecture Performance

### AVX2 vs SSE vs Scalar

Expected performance on x86_64 with different SIMD levels:

| Operation | Scalar | SSE (4-wide) | AVX2 (8-wide) |
|-----------|--------|--------------|---------------|
| f32 Vector Add | 1x | 3.8x | 7.5x |
| f32 Multiply | 1x | 3.8x | 7.5x |
| Biquad Filter | 1x | 3.5x | 7.0x |
| Peak Detection | 1x | 3.7x | 7.2x |

### ARM NEON Performance

Expected performance on ARM processors with NEON:

| Operation | Scalar | NEON (4-wide) |
|-----------|--------|---------------|
| f32 Vector Add | 1x | 3.5x |
| f32 Multiply | 1x | 3.5x |
| Biquad Filter | 1x | 3.2x |

## Profiling Overhead

Acceptable overhead when profiling is enabled:

| Profiling Tool | Overhead | Use Case |
|---------------|----------|----------|
| Criterion Benchmark | 0% | Isolated benchmark runs |
| Flamegraph (perf) | <5% | Production profiling acceptable |
| DHAT Heap Profiler | 20-50% | Development only |
| Chrome DevTools | <10% | WASM performance analysis |

## Hardware Configurations

### Minimum Specifications

For acceptable performance (<10ms audio latency):

- **CPU**: 2+ cores, 2.0+ GHz
- **RAM**: 4 GB
- **OS**: Windows 10/11, macOS 10.15+, Linux (kernel 5.0+)
- **Browser** (WASM): Chrome 90+, Firefox 88+, Safari 14+

### Recommended Specifications

For optimal performance (<5ms audio latency):

- **CPU**: 4+ cores, 3.0+ GHz, AVX2 support
- **RAM**: 8 GB
- **OS**: Latest stable versions
- **Browser** (WASM): Latest Chrome/Firefox with WASM SIMD

## Measurement Methodology

### Running Baselines

To establish baseline performance:

```bash
# 1. Clean environment
cargo clean
rm -rf target/criterion

# 2. Run full benchmark suite
./scripts/bench-desktop.sh all

# 3. Save baseline
./scripts/compare-benchmarks.sh save baseline-v1.0

# 4. Generate report
./scripts/bench-desktop.sh report
```

### Validating Optimizations

To measure optimization impact:

```bash
# 1. Save pre-optimization baseline
./scripts/compare-benchmarks.sh save before-opt

# 2. Implement optimization

# 3. Compare performance
./scripts/compare-benchmarks.sh compare before-opt

# 4. Verify improvements are significant (p < 0.05)
# 5. Check flamegraph for hotspot reduction
./scripts/bench-desktop.sh flamegraph
```

### WASM Performance Testing

To measure WASM bundle size and load time:

```bash
# 1. Build all variants
./scripts/bench-wasm.sh build

# 2. Optimize with wasm-opt
./scripts/bench-wasm.sh optimize

# 3. Analyze bundle size
./scripts/bench-wasm.sh analyze

# 4. Run browser benchmarks
./scripts/bench-wasm.sh benchmark
# Open generated HTML file in browser
```

## Continuous Performance Monitoring

### Git Pre-commit Hook

Prevent performance regressions:

```bash
# Add to .git/hooks/pre-commit
#!/bin/bash
set -e

# Run quick benchmark smoke test
cargo bench --bench simd_benchmarks -- bench_biquad_filter --quick

# Check for significant regressions
if cargo bench -- --baseline main 2>&1 | grep -q "regression detected"; then
    echo "Performance regression detected!"
    echo "Run 'cargo bench' to see details"
    exit 1
fi
```

### CI/CD Integration

Add to GitHub Actions workflow:

```yaml
- name: Performance Benchmark
  run: |
    cargo bench --bench simd_benchmarks -- --save-baseline ci-${{ github.sha }}

- name: Compare with main
  if: github.event_name == 'pull_request'
  run: |
    git fetch origin main
    git checkout origin/main
    cargo bench --bench simd_benchmarks -- --save-baseline main
    git checkout -
    cargo bench --bench simd_benchmarks -- --baseline main
```

## Performance Regression Thresholds

Acceptable performance change thresholds:

| Change Magnitude | Action Required |
|-----------------|-----------------|
| <5% improvement/regression | No action (noise) |
| 5-10% regression | Review and justify |
| >10% regression | **Block merge** until fixed |
| >10% improvement | Document optimization |

## Current Status

**Last Updated:** 2025-11-08

**Baseline Availability:** Not yet established

**Next Steps:**
1. Run complete benchmark suite to establish baselines
2. Verify SIMD optimizations are active (check CPU features)
3. Profile audio callback with flamegraph
4. Measure WASM bundle sizes after build
5. Test on multiple hardware configurations

## Filling In Baseline Data

To populate the "TBD" values in this document:

```bash
# 1. Setup profiling infrastructure
./scripts/setup-profiling.sh

# 2. Run complete benchmark suite
./scripts/bench-desktop.sh all

# 3. Extract key metrics from criterion output
# Located at: target/criterion/<benchmark>/base/estimates.json

# 4. Update this document with actual numbers

# 5. Commit baseline for future comparisons
./scripts/compare-benchmarks.sh save baseline-v1.0
git add docs/PERFORMANCE_BASELINE.md
git commit -m "docs: establish performance baseline metrics"
```
