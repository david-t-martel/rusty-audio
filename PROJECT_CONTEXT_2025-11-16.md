# Rusty Audio Project Context
**Saved: 2025-11-16**
**Context Manager: Claude Code**

## 1. Project Overview

**Project Goals:**
- Create a professional audio player with car-stereo-style UI
- Support both native desktop and WASM web deployment
- Implement 8-band parametric equalizer with real-time spectrum visualization
- Add OAuth 2.0 authentication for web version
- Enable WASM multithreading using SharedArrayBuffer and Web Workers

**Key Architectural Decisions:**
- **Workspace Architecture**: Migrated from monolithic to 3-crate workspace:
  - `rusty-audio-core`: Shared library code
  - `rusty-audio-desktop`: Native desktop application
  - `rusty-audio-web`: WASM web application
- **Hybrid Audio Backend**: Dual backend system (CPAL native + Web Audio API fallback)
- **Cross-Origin Isolation**: Required COOP/COEP/CORP headers for SharedArrayBuffer
- **Git Submodule**: web-audio-api-rs as external dependency

**Technology Stack:**
- Rust 2021 Edition
- egui/eframe v0.33.0 (GUI framework)
- web-audio-api v1.2.0 (git submodule)
- CPAL for native audio
- Cloudflare Workers for OAuth backend
- wasm-bindgen-rayon for WASM multithreading

## 2. Current State (as of 2025-11-16)

### Critical Issues - MAIN BRANCH BROKEN ❌

**Compilation Errors:**
1. **egui-winit v0.33.0**: Missing `accesskit_update` field in pattern matching (line 882)
   - This is a DEPENDENCY issue, not our code
   - Pattern needs to be updated to include `accesskit_update` field or use `..` to ignore
   - **Location**: Dependencies, not our code
   - **Impact**: Blocking cargo build for desktop application

2. **AudioBackend Trait Object Safety**:
   - Cannot use `Box<dyn AudioBackend>` due to generic parameters
   - **Files**: `integrated_audio_manager.rs`, `backend_selector.rs`
   - **Fix Required**: Remove generics, use type aliases for callbacks

3. **MMCSS Type Error**: Incorrect HANDLE import
   - **File**: `src/audio/mmcss.rs:85, 160`
   - **Fix**: Change from `System::Threading::HANDLE` to `Foundation::HANDLE`

4. **PR #5 Critical Bug**: EQ/Analyser nodes not connected to audio graph
   - **Impact**: EQ adjustments have NO EFFECT on audio output
   - **User-Visible**: Non-functional equalizer
   - **Fix**: Connect nodes: Source → EQ → Analyser → Gain → Output

**Recently Completed:**
- ✅ Workspace separation (commit 31603b1, pushed to GitHub)
- ✅ OAuth 2.0 Authorization Code Flow with PKCE
- ✅ Cloudflare Workers backend (6 API endpoints)
- ✅ WASM multithreading with worker pool
- ✅ Fixed 7 P0 critical issues (in previous session)
- ✅ 78 WASM unit tests + 59 Playwright E2E tests
- ✅ 50+ documentation files
- ✅ GitHub Actions CI/CD pipeline

**Work in Progress:**
- Frontend verification checklist at 50% complete (testing section pending)
- Development server running at http://localhost:8080
- Formatting inconsistencies (~40 files need reformatting)
- web-audio-api-rs submodule has modified/untracked content

**Technical Debt:**
- Unused imports warnings in testing modules (59 warnings)
- `unsafe` code warnings (enabled -W unsafe-code lint)
- Unexpected cfg condition for `system-metrics` feature (not in Cargo.toml)

**Performance Baselines:**
- Desktop: 60 FPS sustained with real-time spectrum
- WASM: Worker pool with 4 workers
- FFT: 512-point analysis
- Audio latency: <25ms target

## 3. Architecture Details

### Component Hierarchy
```
┌─────────────────────────────────────────────────────┐
│          IntegratedAudioManager                     │
│  (High-level API - UI interacts with this)         │
│  - Manages playback state                          │
│  - Routes audio sources to destinations            │
│  - Handles backend selection                       │
└────────────────┬────────────────────────────────────┘
                 │
         ┌───────┴────────┐
         │                │
┌────────▼─────────┐  ┌───▼────────────────────────┐
│   AudioRouter    │  │   AudioBackend (trait)     │
│  (Routing Graph) │  │   - Device abstraction     │
│  - Sources       │  │   - Stream management      │
│  - Destinations  │  │   - Platform-specific      │
│  - Routes        │  └────────────┬───────────────┘
└──────────────────┘               │
                          ┌────────┴─────────┐
                          │                  │
                  ┌───────▼──────┐    ┌──────▼───────┐
                  │ CpalBackend  │    │ AsioBackend  │
                  │ (Cross-plat) │    │ (Windows)    │
                  └──────────────┘    └──────────────┘
                          │
                  ┌───────▼───────────┐
                  │ WebAudioBackend   │
                  │ (WASM/Browser)    │
                  └───────────────────┘
```

### Audio Processing Pipeline
```
AudioContext → AudioBufferSourceNode → BiquadFilterNode (x8 for EQ)
  → GainNode → AnalyserNode → Output
```

### Backend Selection Strategy
**Windows:**
1. ASIO (professional audio, lowest latency)
2. WASAPI (standard Windows audio)
3. DirectSound (legacy fallback)

**macOS:** CoreAudio (via CPAL)
**Linux:** ALSA/PulseAudio (via CPAL)
**WASM:** Web Audio API

### Thread Safety Model
| Platform | Model | Synchronization |
|----------|-------|-----------------|
| Native (CPAL/ASIO) | Multi-threaded (separate audio thread) | `Arc<Mutex<>>` |
| WASM (Web Audio) | Single-threaded (main thread) | `Rc<RefCell<>>` |

## 4. Design Decisions

**Workspace Separation Rationale:**
- **Before**: Monolithic 81KB main.rs with conditional compilation chaos
- **After**: Clean separation with zero code duplication
- **Benefit**: Separate feature gates, cleaner dependencies, easier testing

**OAuth 2.0 Architecture:**
- **Flow**: Authorization Code with PKCE (most secure for public clients)
- **Providers**: Google, GitHub, Microsoft
- **Backend**: Cloudflare Workers (serverless, global edge deployment)
- **Security**: AES-256-GCM token encryption, JWT signing (HS256), session KV storage
- **Storage**: Browser localStorage for tokens, KV for server sessions

**WASM Multithreading Strategy:**
- **Requirements**: Cross-origin isolation (COOP/COEP/CORP headers mandatory)
- **Worker Pool**: Managed pool with health monitoring
- **Communication**: SharedArrayBuffer for zero-copy data transfer
- **Fallback**: Graceful degradation to single-threaded if headers not present
- **Panic Boundaries**: Comprehensive error handling with recovery

**Hybrid Backend Selection Logic:**
1. Try ASIO (Windows professional audio)
2. Fall back to CPAL (native cross-platform)
3. Fall back to Web Audio API (WASM or when native fails)

## 5. Code Patterns

**Error Handling:**
```rust
// Good: Specific error types with context
fn operation() -> Result<T, AudioBackendError> {
    device.start()
        .map_err(|e| AudioBackendError::DeviceUnavailable(
            format!("Cannot start: {}", e)
        ))
}

// Bad: Generic errors or unwrap
fn bad_operation() -> Result<T> {
    device.start().unwrap()  // ❌ Never in library code
}
```

**Type Aliases for Trait Objects:**
```rust
pub type OutputCallback = Box<dyn FnMut(&mut [f32]) + Send + 'static>;
pub type InputCallback = Box<dyn FnMut(&[f32]) + Send + 'static>;
```

**Coding Conventions:**
- **Error Handling**: Never use `unwrap()` in library code; use `Result<T, E>`
- **Thread Safety**: Use `parking_lot::Mutex` for shared state
- **Security Validation**: All user inputs through `security::input_validator`
- **Memory Safety**: SIMD optimizations with careful unsafe blocks
- **Testing**: Property-based tests for algorithms, UI tests with egui_kittest

## 6. Implementation Fixes Required

### Fix Priority Order:

1. **MMCSS HANDLE Import (5 minutes)**
   ```diff
   - use windows::Win32::System::Threading::HANDLE;
   + use windows::Win32::Foundation::HANDLE;
   ```

2. **AudioBackend Trait Dyn-Safety (30 minutes)**
   - Add type aliases
   - Remove generic parameters from trait methods
   - Add `as_any()` and `as_any_mut()` methods

3. **Backend Implementations (1-2 hours)**
   - Implement callback methods with `OutputCallback`/`InputCallback` types
   - Add `as_any()` downcasting support
   - Use `Arc<Mutex<>>` for thread-safe callback sharing

4. **PR #5 EQ Connection Bug (1-2 hours)**
   - Connect audio nodes correctly: Source → EQ → Analyser → Gain → Output
   - Add test to verify EQ affects audio output

## 7. Critical File Locations

**Configuration:**
- `.cargo/config.toml` - Shared target directory (`C:\Users\david\.cargo\shared-target`)
- `Cargo.toml` - Workspace root
- `cloudflare-pages.toml` - Deployment config
- `Trunk.toml` - WASM build config

**Core Modules:**
- `rusty-audio-core/src/audio/` - Audio backend abstraction
- `rusty-audio-core/src/ui/` - UI components
- `rusty-audio-core/src/ai/` - AI features (optional)
- `rusty-audio-web/src/auth/` - OAuth implementation
- `workers/auth-service/` - Cloudflare Workers backend

**Documentation:**
- `ARCHITECTURE_SUMMARY.md` - High-level architecture (current state: BROKEN)
- `AUDIO_BACKEND_ARCHITECTURE.md` - Backend design (70 pages)
- `IMPLEMENTATION_CHECKLIST.md` - Step-by-step fixes
- `FRONTEND_VERIFICATION.txt` - Frontend checklist
- `CLAUDE.md` - Project-specific AI agent instructions

**Git:**
- Submodule: `web-audio-api-rs/` (git submodule, not path dependency)
- Main branch: `main`
- Latest commit: `31603b1` (workspace separation, OAuth, WASM multithreading)
- Remote: https://github.com/david-t-martel/rusty-audio.git
- Last working commit: `456b5f4` (Merge PR #2)

## 8. Testing Strategy

**Compilation Tests:**
```bash
cargo check                                    # Standard build
cargo check --all-features                     # All features
cargo check --target wasm32-unknown-unknown    # WASM target
```

**Test Suites:**
- **Unit Tests**: Embedded in modules
- **Property Tests**: Using proptest and quickcheck
- **UI Tests**: egui_kittest for automated UI testing
- **Integration Tests**: Full feature tests
- **Benchmarks**: 5 comprehensive benchmark suites
- **Coverage Target**: 85% minimum

**Benchmark Suites:**
```bash
cargo bench                                    # Run all benchmarks
cargo bench --bench audio_benchmarks          # Audio processing
cargo bench --bench performance_benchmarks    # General performance
cargo bench --bench realtime_benchmarks       # Real-time constraints
cargo bench --bench memory_benchmarks         # Memory usage
cargo bench --bench audio_quality_benchmarks  # Quality metrics
```

## 9. Performance Characteristics

### Latency Comparison
| Backend | Typical Latency | Buffer Size | Use Case |
|---------|----------------|-------------|----------|
| ASIO | 1.3-5ms | 64-256 samples | Professional audio, live performance |
| WASAPI (Exclusive) | 5-10ms | 128-512 samples | Music production |
| WASAPI (Shared) | 10-20ms | 512-1024 samples | General playback |
| CoreAudio | 5-10ms | 128-512 samples | macOS audio |
| ALSA | 10-20ms | 512-1024 samples | Linux audio |
| Web Audio API | 20-50ms | Browser-dependent | Browser playback |

### Memory Usage
- **Idle:** ~5MB (backend initialized)
- **Playing:** ~10-15MB (active streams + buffers)
- **Ring Buffer:** ~200KB (8x buffer size for hybrid mode)

### CPU Usage
- **CPAL/ASIO:** ~1-3% (audio thread, 48kHz stereo)
- **Web Audio API:** ~2-5% (browser overhead)
- **EQ Processing:** ~0.5% (8 bands, software DSP)
- **Spectrum Analysis:** ~1% (512 FFT, 60Hz update)

## 10. Agent Coordination History

**Previous Session Agents Used:**
- **architect-reviewer**: Analyzed workspace architecture before split
- **rust-pro**: Assisted with workspace migration and WASM compilation
- **security-auditor**: Reviewed OAuth implementation for vulnerabilities
- **deployment-engineer**: Set up Cloudflare Workers deployment

**Successful Agent Combinations:**
- **python-pro + rust-pro**: Excellent for cross-language analysis
- **search-specialist + architect-reviewer**: Great for finding duplication patterns
- **security-auditor + code-reviewer**: Comprehensive security + quality review

**Agent-Specific Context:**
- **rust-pro**: Knows about shared target directory at `C:\Users\david\.cargo\shared-target`
- **deployment-engineer**: Familiar with Cloudflare Pages config (cloudflare-pages.toml)
- **security-auditor**: Aware of OAuth 2.0 PKCE implementation in `workers/auth-service/`

## 11. Immediate Next Actions

1. **Fix MMCSS import** - Quick 5-minute fix in `src/audio/mmcss.rs`
2. **Fix AudioBackend trait** - Make dyn-compatible (30 minutes)
3. **Update backend implementations** - Use new callback types (1-2 hours)
4. **Fix EQ connection bug** - Connect nodes properly (1-2 hours)
5. **Run cargo fmt** - Fix formatting on all workspace crates
6. **Investigate egui-winit** - Upgrade or patch for `accesskit_update` field
7. **Check submodule** - `git submodule status` for web-audio-api-rs
8. **Clean warnings** - Remove unused imports from testing modules
9. **Complete frontend verification** - Finish testing section of checklist

## 12. Future Roadmap

**Planned Features:**
- [ ] Deploy to Cloudflare Pages with OAuth
- [ ] Implement MIDI I/O functionality
- [ ] Add advanced format support (beyond WAV/FLAC/MP3)
- [ ] AI-enhanced audio processing features (optional dependencies)
- [ ] AudioWorklet for modern Web Audio API
- [ ] Multi-device routing (aggregate devices)

**Identified Improvements:**
- Add `system-metrics` feature flag to Cargo.toml
- Optimize pre-commit hooks (currently too slow)
- Consider JACK/PipeWire backends for Linux
- Automatic latency tuning with adaptive buffer sizing
- Sample rate conversion with automatic resampling

**Technical Debt to Address:**
- 59 unused import warnings in testing modules
- Unsafe code warnings (review all unsafe blocks)
- Dependency version conflicts (reqwest default-features warning)
- Pre-commit hooks performance (formatting 325 files takes minutes)

## 13. Development Commands

### Quick Development Cycle
```bash
# Standard development cycle
cargo check && cargo clippy && cargo test && cargo run

# Pre-commit validation (automatic on commit)
pre-commit run --all-files

# Full quality check before release
cargo fmt && \
  cargo clippy -- -D warnings && \
  cargo test && \
  cargo bench --no-run && \
  cargo build --release

# Update and test submodule
git submodule update --remote && \
  cd web-audio-api-rs && cargo test && cd ..
```

### Testing EQ Connection (PR #5 Bug)
```rust
// Generate 1kHz sine wave
let test_signal = generate_sine_wave(1000.0, 1.0, 48000);

// Set 1kHz EQ band to -20dB
eq.set_band_gain(4, -20.0);

// Process audio
let output = process_audio(&test_signal, &eq);

// Measure attenuation
let input_rms = calculate_rms(&test_signal);
let output_rms = calculate_rms(&output);
let attenuation_db = 20.0 * (output_rms / input_rms).log10();

// If EQ is connected: attenuation ≈ -20dB
// If EQ is bypassed: attenuation ≈ 0dB
log::info!("Attenuation: {:.1}dB (expected: -20dB)", attenuation_db);
```

## Context Metadata

**Saved By:** Claude Code Context Manager
**Date:** 2025-11-16
**Project State:** Main branch broken, fixes identified
**Last Working Commit:** 456b5f4
**Current Commit:** 31603b1
**Repository:** https://github.com/david-t-martel/rusty-audio.git

**Next Context Update Needed When:**
- Main branch compilation issues are fixed
- EQ connection bug is resolved
- Frontend verification is complete
- Deployment to Cloudflare Pages is ready

---
*This context provides a complete snapshot of the Rusty Audio project state for efficient agent handoff and session restoration.*