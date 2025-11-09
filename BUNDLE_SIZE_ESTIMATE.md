# Rusty Audio - WASM Bundle Size Estimate

## Current Project Analysis

Based on the Cargo.toml dependencies and project structure, here's the expected WASM bundle size.

## Dependency Analysis

### Core Dependencies (WASM Build)
- **eframe 0.33.0**: ~400-600 KB (egui + wgpu renderer)
- **egui 0.33.0**: Included in eframe
- **egui_dock 0.18**: ~50-80 KB
- **serde 1.0**: ~30-50 KB
- **parking_lot 0.12**: ~20-30 KB
- **rustfft 6.0**: ~100-150 KB
- **realfft 3.3**: ~50-80 KB
- **num-complex 0.4**: ~10-20 KB
- **ndarray 0.15**: ~80-120 KB
- **rayon 1.10**: Replaced with single-threaded WASM stubs (~10 KB)

### WASM-Specific Dependencies
- **wasm-bindgen**: ~40-60 KB (JS glue)
- **web-sys**: ~150-250 KB (browser APIs)
- **js-sys**: ~30-50 KB
- **console_error_panic_hook**: ~5-10 KB
- **console_log**: ~5-10 KB

### Project Code Estimate
- **UI modules** (src/ui/): ~80-120 KB
- **Audio modules** (src/audio/): ~60-100 KB
- **AI modules** (src/ai/): ~100-150 KB
- **Security modules** (src/security/): ~40-60 KB
- **Testing modules**: Not included in WASM build
- **Main application**: ~50-80 KB

## Size Estimates

### Unoptimized Debug Build
- Total dependencies: ~1,500-2,000 KB
- Project code: ~330-510 KB
- **Total**: ~1,800-2,500 KB (1.8-2.5 MB)

### Optimized Release Build (Before wasm-opt)
With Cargo profile optimizations:
- `opt-level = 3`: 30-40% reduction
- `lto = "fat"`: Additional 20-30% reduction
- `codegen-units = 1`: Additional 10-15% reduction
- `strip = true`: Remove debug symbols

**Estimated**: ~900-1,200 KB (0.9-1.2 MB)

### After wasm-opt Optimization
wasm-opt with `-Oz` flag typically achieves 40-60% additional reduction:

**Estimated**: ~450-720 KB (450-720 KB)

### Compressed (Production Delivery)

#### Gzip (Level 9)
WASM typically compresses well with gzip (60-70% reduction):
- **Estimated**: ~180-290 KB

#### Brotli (Level 11)
Brotli achieves even better compression (70-80% reduction):
- **Estimated**: ~135-216 KB

## Production Bundle Breakdown

### First Load (Compressed with Brotli)

| Asset | Size (Uncompressed) | Size (Brotli) |
|-------|---------------------|---------------|
| rusty_audio_bg.wasm | 450-720 KB | 135-216 KB |
| rusty_audio.js | 45-60 KB | 12-18 KB |
| index.html | 13 KB | 4-5 KB |
| manifest.json | 3 KB | 1-2 KB |
| sw.js | 11 KB | 3-4 KB |
| Icons (8 files) | 120-150 KB | N/A (served as-is) |
| **Total** | **642-957 KB** | **275-395 KB** |

### Subsequent Loads (Cached)
After first load, only HTML needs revalidation:
- **~5 KB** (index.html, compressed)

## Optimization Opportunities

### 1. Feature Flags (Highest Impact)
Remove unused features to significantly reduce size:

```toml
[features]
default = []  # Minimal build
full = ["audio-optimizations", "ai-features", "security-hardening"]
audio-optimizations = []
ai-features = ["ndarray", "statrs"]
security-hardening = []
```

**Potential savings**: 200-400 KB by disabling AI/ML features for web build

### 2. Dependency Optimization
Replace heavy dependencies with lighter alternatives:

- **rayon**: Already replaced with wasm-thread-pool (automatic)
- **ndarray**: Only include if AI features needed (~80-120 KB savings)
- **statrs**: Optional, remove for basic builds (~40-60 KB savings)

### 3. Code Splitting (Future Enhancement)
Split audio processing into separate WASM module:
- Main app: 300-400 KB
- Audio processor: 150-200 KB
- Load audio processor on-demand

**Total first load reduction**: 150-200 KB

### 4. Lazy Loading
Defer non-critical modules:
- AI features loaded on first use
- Advanced audio effects loaded on demand
- Recording module loaded when needed

## Target Bundle Sizes

### Minimum (Basic Audio Player)
With minimal features:
- WASM (compressed): ~150-200 KB
- JavaScript: ~10-15 KB
- **Total**: ~160-215 KB

### Standard (Full Features)
Current configuration:
- WASM (compressed): ~180-250 KB
- JavaScript: ~12-18 KB
- **Total**: ~200-270 KB

### Maximum (All Features + AI)
With all AI/ML features enabled:
- WASM (compressed): ~250-350 KB
- JavaScript: ~15-20 KB
- **Total**: ~270-370 KB

## Comparison with Competitors

### Web Audio Players

| Player | Technology | Bundle Size | Notes |
|--------|-----------|-------------|-------|
| **Rusty Audio** | Rust + WASM | ~200-270 KB | Full-featured, offline |
| Amplitude.js | JavaScript | ~80 KB | Basic features only |
| Howler.js | JavaScript | ~30 KB | Audio library, no UI |
| Wavesurfer.js | JavaScript | ~150 KB | Waveform visualization |
| React Audio Player | React + JS | ~300-500 KB | Full React bundle |
| Vue Music | Vue + JS | ~250-400 KB | Full Vue bundle |

**Rusty Audio is competitive**, especially considering:
- Native performance (WASM)
- Full offline support (PWA)
- Advanced audio processing (FFT, EQ)
- No framework overhead after first load

## Build Time vs Size Tradeoff

| Profile | Build Time | WASM Size | Compressed |
|---------|-----------|-----------|------------|
| dev | 30-60s | ~2.5 MB | ~800 KB |
| release | 2-4 min | ~700 KB | ~250 KB |
| release (opt-level=z) | 3-5 min | ~500 KB | ~180 KB |
| release + wasm-opt | 4-6 min | ~450 KB | ~150 KB |

## Recommendations

### For Development
```bash
# Fast builds, larger size
PROFILE=dev ./scripts/build-wasm.sh
```

### For Production
```bash
# Slower builds, optimal size
OPTIMIZE_LEVEL=z ./scripts/build-wasm.sh
```

### For CI/CD
Use caching to speed up release builds:
```yaml
- uses: actions/cache@v4
  with:
    path: target/
    key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
```

## Monitoring Bundle Size

### Set Up Size Budgets
Add to CI/CD pipeline:

```bash
# Fail if WASM exceeds 2MB uncompressed
WASM_SIZE=$(stat -c%s dist/rusty_audio_bg.wasm)
if [ $WASM_SIZE -gt 2097152 ]; then
  echo "WASM too large: $WASM_SIZE bytes"
  exit 1
fi
```

### Track Size Over Time
```bash
# Log size to file
echo "$(date +%Y-%m-%d),$WASM_SIZE" >> bundle-size-history.csv

# Visualize with gnuplot or similar
```

## Conclusion

**Expected Production Bundle Size: 200-270 KB (compressed)**

This includes:
- Full audio player functionality
- Real-time spectrum visualization
- 8-band equalizer
- Offline support via PWA
- Modern, responsive UI

**Size is competitive** with JavaScript alternatives while providing:
- Better performance (native WASM)
- Type safety (Rust)
- Memory safety (no runtime crashes)
- Smaller incremental updates (better caching)

---

*Last updated: 2025-01-08*
