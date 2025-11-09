# Profiling and Benchmarking Guide

Comprehensive guide for profiling and optimizing Rusty Audio performance across desktop and WASM targets.

## Quick Start

### Desktop Benchmarks
```bash
# Run all criterion benchmarks
./scripts/bench-desktop.sh criterion

# Generate flamegraph for hotspot analysis
./scripts/bench-desktop.sh flamegraph

# Complete benchmark suite with reports
./scripts/bench-desktop.sh all
```

### WASM Benchmarks
```bash
# Build and analyze WASM bundles
./scripts/bench-wasm.sh all

# Just build optimized variants
./scripts/bench-wasm.sh build

# Analyze bundle size
./scripts/bench-wasm.sh analyze
```

## Desktop Profiling Infrastructure

### Available Benchmarks

We have 7 comprehensive benchmark suites:

1. **audio_benchmarks.rs** - Core audio processing
2. **audio_quality_benchmarks.rs** - Quality metrics
3. **performance_benchmarks.rs** - General performance
4. **memory_benchmarks.rs** - Memory allocation patterns
5. **realtime_benchmarks.rs** - Real-time processing constraints
6. **simd_benchmarks.rs** - SIMD optimization verification
7. **optimization_benchmarks.rs** - Phase-specific optimizations

### Criterion Benchmarks

#### Running Benchmarks

```bash
# All benchmarks
cargo bench

# Specific benchmark suite
cargo bench --bench simd_benchmarks

# Filter by function name
cargo bench --bench simd_benchmarks -- bench_biquad_filter

# Save baseline for comparison
cargo bench --bench simd_benchmarks -- --save-baseline before-opt

# Compare against baseline
cargo bench --bench simd_benchmarks -- --baseline before-opt
```

#### Benchmark Output

Results are saved to:
- HTML reports: `target/criterion/report/index.html`
- Raw data: `target/criterion/<benchmark_name>/`
- Comparison data: `target/criterion/<benchmark_name>/base/`

#### Understanding Results

Criterion provides:
- **Mean time**: Average execution time
- **Std dev**: Variability in measurements
- **Throughput**: Elements/bytes processed per second
- **Comparison**: % change vs baseline

Example output:
```
bench_biquad_filter/8_band_eq/512
                        time:   [45.234 µs 45.678 µs 46.123 µs]
                        thrpt:  [11.098 Melem/s 11.208 Melem/s 11.318 Melem/s]
```

### Flamegraph Profiling

Flamegraphs visualize CPU hotspots in your code.

#### Prerequisites

```bash
# Install flamegraph
cargo install flamegraph

# Linux: Install perf (if not already installed)
sudo apt-get install linux-tools-common linux-tools-generic

# Adjust perf paranoia level (temporary)
echo 0 | sudo tee /proc/sys/kernel/perf_event_paranoid

# Or permanently
echo 'kernel.perf_event_paranoid = 0' | sudo tee -a /etc/sysctl.conf
sudo sysctl -p
```

#### Generating Flamegraphs

```bash
# Profile specific benchmark
cargo flamegraph --bench simd_benchmarks -- --bench

# Profile application
cargo flamegraph --bin rusty-audio_native

# Custom output location
cargo flamegraph --bench simd_benchmarks -o target/flamegraphs/simd.svg

# Use script (recommended)
./scripts/bench-desktop.sh flamegraph
```

#### Interpreting Flamegraphs

- **Width**: CPU time spent in function (wider = more time)
- **Height**: Call stack depth
- **Color**: Random (for differentiation only)
- **Click**: Zoom into specific function
- **Search**: Find specific functions

**What to look for:**
1. Wide plateaus = CPU hotspots
2. Towers = deep call stacks (inlining opportunities)
3. Unexpected functions = optimization opportunities

**Common hotspots in audio code:**
- FFT computation
- Biquad filter processing
- Sample rate conversion
- Memory allocation

### DHAT Heap Profiling

DHAT profiles heap allocations to find memory issues.

#### Setup

Add to your benchmark code:

```rust
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn main() {
    let _profiler = dhat::Profiler::new_heap();

    // Your benchmark code
    criterion::Criterion::default()
        .configure_from_args()
        .final_summary();
}
```

#### Running

```bash
# Run with dhat profiling
DHAT_OUT_FILE=target/dhat-profiles/heap.json cargo bench

# View results in browser
firefox target/dhat-profiles/heap.json
```

#### What to Look For

- **Total allocations**: Should be minimal in hot paths
- **Allocation hotspots**: Functions allocating most memory
- **Allocation lifetimes**: Long-lived allocations
- **Peak memory**: Maximum heap usage

**Audio-specific concerns:**
- Allocations in audio callback (should be ZERO)
- Buffer pool efficiency
- Memory leaks in long-running sessions

### Performance Targets

Based on real-time audio requirements:

#### Latency Targets (48kHz, 512 samples)

| Operation | Target | Critical |
|-----------|--------|----------|
| Audio callback total | <5ms | <10ms |
| 8-band EQ processing | <1ms | <2ms |
| 2048-point FFT | <2ms | <4ms |
| Spectrum smoothing | <0.5ms | <1ms |
| Buffer pool acquire | <10µs | <50µs |

#### Throughput Targets

| Operation | Target |
|-----------|--------|
| Biquad filter | >10 Msamples/s |
| FFT processing | >500 frames/s (2048pt) |
| SIMD operations | >100 Melem/s |

#### Memory Targets

| Metric | Target |
|--------|--------|
| Peak heap usage | <100 MB |
| Audio callback allocations | 0 |
| Buffer pool overhead | <5% |
| Memory fragmentation | <10% |

## WASM Profiling Infrastructure

### Bundle Size Optimization

WASM bundle size directly impacts load time.

#### Size Targets

| Bundle Type | Target (gzipped) |
|-------------|------------------|
| Initial load | <500 KB |
| Total bundle | <1.5 MB |
| Lazy modules | <200 KB each |

#### Build Variants

```bash
# Development build (debug symbols)
wasm-pack build --dev --target web

# Profiling build (some optimizations)
wasm-pack build --profiling --target web

# Release build (full optimizations)
wasm-pack build --release --target web
```

#### wasm-opt Optimizations

```bash
# Size optimization (best for production)
wasm-opt -Oz --enable-simd input.wasm -o output.wasm

# Speed optimization
wasm-opt -O3 --enable-simd input.wasm -o output.wasm

# Aggressive optimization
wasm-opt -O4 --enable-simd input.wasm -o output.wasm
```

**Optimization levels:**
- `-O2`: Moderate optimization (fast compile)
- `-O3`: High optimization (slower compile)
- `-O4`: Aggressive optimization (slowest compile)
- `-Oz`: Size optimization (smallest binary)

### Twiggy Analysis

Twiggy analyzes WASM code size by function.

#### Installation

```bash
cargo install twiggy
```

#### Usage

```bash
# Top 20 largest functions
twiggy top -n 20 target/wasm32-unknown-unknown/release/rusty_audio.wasm

# Dominators (what pulls in large code)
twiggy dominators target/wasm32-unknown-unknown/release/rusty_audio.wasm

# Paths to specific function
twiggy paths target/wasm32-unknown-unknown/release/rusty_audio.wasm process_audio

# Save analysis
twiggy top -n 100 input.wasm > target/wasm-profiles/analysis.txt
```

#### Optimization Strategies

Based on twiggy output:

1. **Remove unused features**: Disable cargo features not needed for WASM
2. **Trim dependencies**: Use `default-features = false`
3. **Replace std functions**: Use custom implementations
4. **Remove panic handlers**: Use `panic = "abort"`
5. **Lazy load**: Split large modules

### WASM Performance Benchmarks

#### Browser-Based Benchmarks

```bash
# Generate benchmark HTML
./scripts/bench-wasm.sh benchmark

# Open in browser
# Click "Run Benchmarks" to measure:
# - Module load time
# - Initialization time
# - Memory usage
```

#### Metrics to Track

1. **Load Time**: Time to fetch and compile WASM
   - Target: <100ms on broadband
   - Critical: <500ms

2. **Initialization**: Time to set up audio context
   - Target: <50ms
   - Critical: <200ms

3. **Memory Usage**: Heap allocated
   - Target: <50 MB
   - Critical: <100 MB

4. **Audio Latency**: Time from input to output
   - Target: <10ms
   - Critical: <20ms

### Chrome DevTools Profiling

#### Performance Profiling

1. Open Chrome DevTools (F12)
2. Go to Performance tab
3. Click Record
4. Interact with audio player
5. Stop recording
6. Analyze:
   - Main thread activity
   - Audio worklet performance
   - Memory allocations

#### Memory Profiling

1. Go to Memory tab
2. Take heap snapshot
3. Interact with player
4. Take another snapshot
5. Compare snapshots:
   - Look for leaks (growing allocations)
   - Identify large objects

#### Network Profiling

1. Go to Network tab
2. Reload page
3. Analyze:
   - WASM download time
   - Compression effectiveness
   - Caching behavior

## Benchmark Comparison Workflow

### Before Optimization

```bash
# Save baseline
cargo bench -- --save-baseline before-opt

# Save flamegraph
cargo flamegraph --bench simd_benchmarks -o before-opt.svg
```

### After Optimization

```bash
# Compare against baseline
cargo bench -- --baseline before-opt

# Generate new flamegraph
cargo flamegraph --bench simd_benchmarks -o after-opt.svg

# View comparison
./scripts/bench-desktop.sh compare
```

### Interpreting Comparisons

Criterion shows performance change:

```
bench_biquad_filter/8_band_eq/512
                        time:   [35.123 µs 35.456 µs 35.789 µs]
                        change: [-23.45% -22.12% -20.78%] (p = 0.00 < 0.05)
                        Performance has improved.
```

**Legend:**
- Negative % = Faster (improvement)
- Positive % = Slower (regression)
- p-value < 0.05 = Statistically significant

## Continuous Performance Monitoring

### Automated Benchmarking

Add to CI/CD pipeline:

```yaml
# .github/workflows/benchmark.yml
name: Benchmark

on:
  pull_request:
  push:
    branches: [main]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Run benchmarks
        run: |
          cargo bench --bench simd_benchmarks -- --save-baseline pr-${{ github.event.number }}

      - name: Compare with main
        run: |
          git checkout main
          cargo bench --bench simd_benchmarks -- --baseline pr-${{ github.event.number }}
```

### Performance Regression Detection

Set thresholds for acceptable regression:

```bash
# Fail if >5% slower
cargo bench -- --baseline main --significance-level 0.05 --noise-threshold 0.05
```

## Best Practices

### Benchmark Design

1. **Realistic workloads**: Use actual audio data
2. **Stable environment**: Disable CPU frequency scaling
3. **Multiple runs**: Let criterion run enough iterations
4. **Isolated tests**: Avoid external dependencies

### Profiling Tips

1. **Profile release builds**: Debug builds have misleading profiles
2. **Disable inlining selectively**: Use `#[inline(never)]` for clarity
3. **Use blackbox**: Prevent compiler from optimizing away code
4. **Profile actual usage**: Not just benchmarks

### Optimization Workflow

1. **Measure first**: Profile before optimizing
2. **Focus on hotspots**: Optimize where it matters (80/20 rule)
3. **Verify improvements**: Benchmark before and after
4. **Don't over-optimize**: Diminishing returns

### Common Pitfalls

1. **Premature optimization**: Profile first!
2. **Micro-optimizations**: Focus on algorithmic improvements
3. **Ignoring cache**: Memory access patterns matter
4. **Unsafe code**: Only when necessary and proven faster

## Performance Checklist

Before shipping an optimization:

- [ ] Benchmark shows statistically significant improvement
- [ ] Flamegraph confirms hotspot is addressed
- [ ] No memory leaks (DHAT clean)
- [ ] WASM bundle size within targets
- [ ] Audio callback latency <5ms
- [ ] No allocations in audio thread
- [ ] Tests still pass
- [ ] Code quality maintained

## Troubleshooting

### Benchmarks are Noisy

```bash
# Increase sample size
cargo bench -- --sample-size 1000

# Increase measurement time
cargo bench -- --measurement-time 30

# Disable CPU frequency scaling
echo performance | sudo tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor
```

### Flamegraph Not Working

```bash
# Check perf permissions
cat /proc/sys/kernel/perf_event_paranoid
# Should be -1, 0, or 1

# Try with sudo
sudo cargo flamegraph --bench simd_benchmarks

# Check kernel symbols
ls /boot/System.map-$(uname -r)
```

### DHAT Not Generating Output

```bash
# Ensure DHAT_OUT_FILE is set
export DHAT_OUT_FILE=target/dhat-heap.json
cargo bench

# Check file was created
ls -lh target/dhat-heap.json
```

### WASM Build Fails

```bash
# Update wasm-pack
cargo install wasm-pack --force

# Clear cache
rm -rf target/wasm32-unknown-unknown

# Check rustup targets
rustup target add wasm32-unknown-unknown
```

## Resources

### Tools
- [Criterion.rs](https://github.com/bheisler/criterion.rs) - Statistical benchmarking
- [cargo-flamegraph](https://github.com/flamegraph-rs/flamegraph) - CPU profiling
- [DHAT](https://docs.rs/dhat) - Heap profiling
- [Twiggy](https://rustwasm.github.io/twiggy/) - WASM code size profiler
- [wasm-opt](https://github.com/WebAssembly/binaryen) - WASM optimizer

### Documentation
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [WASM Optimization Guide](https://rustwasm.github.io/docs/book/reference/code-size.html)
- [Audio Performance Guide](https://rust-audio.discourse.group/)

### Community
- [Rust Audio Discord](https://discord.gg/rust-audio)
- [WebAssembly Discord](https://discord.gg/webassembly)
