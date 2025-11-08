# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## Project Overview

**Rusty Audio** is a professional car-stereo-style desktop audio player with industry-leading accessibility (WCAG 2.1 AA compliance), comprehensive safety features, and hybrid audio backend support. Built with Rust and egui, it combines real-time audio processing, signal generation, and mathematical verification capabilities.

## Essential Build Commands

### Development Workflow

```powershell
# Quick compilation check (fast)
cargo make check
# or: cargo check --all-targets

# Build and run debug binary
cargo make run
# or: cargo run --bin rusty-audio

# Build release (optimized)
cargo make release
# or: cargo build --release --bin rusty-audio

# Code quality checks
cargo make fmt       # Format code
cargo make clippy    # Lint with strict warnings
cargo make fix       # Auto-fix compilation issues
```

### Testing

```powershell
# Run all tests
cargo make test
# or: cargo test --lib

# Run specific test
cargo test test_name -- --exact --nocapture

# Run UI tests specifically
cargo make test-ui
# or: cargo test --lib -- ui

# Run test automation script (Windows)
.\scripts\run_tests.ps1 -TestMode quick          # Quick test suite
.\scripts\run_tests.ps1 -TestMode comprehensive  # Full test suite
.\scripts\run_tests.ps1 -TestMode manual         # Show manual testing guidance
```

### Benchmarking

```powershell
# Run all benchmarks
cargo make bench
# or: cargo bench --no-fail-fast

# Performance analysis script
.\run_performance_analysis.ps1
```

### Documentation

```powershell
# Generate and open documentation
cargo make doc
# or: cargo doc --no-deps --open
```

### Cleaning

```powershell
# Standard clean
cargo make clean

# Deep clean (removes target directory)
cargo make clean-all
```

## High-Level Architecture

### Module Structure

```
src/
├── main.rs                     # Application entry, integrates all systems
├── lib.rs                      # Public library API, exposes testing framework
├── audio/                      # Audio backend abstraction layer
│   ├── backend.rs             # AudioBackend trait, device management
│   ├── hybrid.rs              # HybridAudioBackend (Web Audio + CPAL)
│   ├── web_bridge.rs          # Bridge for Web Audio API integration
│   ├── device.rs              # CPAL native audio device handling
│   └── manager.rs             # Audio device enumeration and selection
├── ui/                        # User interface components
│   ├── accessibility.rs       # AccessibilityManager, WCAG 2.1 AA compliance
│   ├── enhanced_controls.rs   # AccessibleSlider, AccessibleKnob
│   ├── enhanced_button.rs     # Buttons with safety indicators
│   ├── error_handling.rs      # ErrorManager with recovery flows
│   ├── signal_generator.rs    # 8-type signal generation UI
│   ├── spectrum.rs            # Real-time spectrum visualizer (512-pt FFT)
│   ├── theme.rs               # Theme system (6 themes, car-stereo aesthetic)
│   ├── dock_layout.rs         # Dockable panel system
│   └── components.rs          # Reusable UI widgets
├── audio_engine.rs            # Core audio processing pipeline
├── audio_performance.rs       # Optimized spectrum processing
├── testing/                   # Mathematical verification framework
│   ├── signal_generators.rs   # 8 signal types for testing
│   ├── property_tests.rs      # Property-based testing
│   └── mathematical_framework # Signal verification suite
├── security/                  # Safety-critical audio handling
│   ├── audio_safety.rs        # Volume limiting, hearing protection
│   └── secure_config.rs       # Settings persistence
└── ai/                        # AI-enhanced features (optional)
    ├── eq_optimizer.rs        # ML-based EQ optimization
    ├── noise_reduction.rs     # Adaptive noise reduction
    └── volume_normalizer.rs   # Intelligent volume normalization
```

### Key Architectural Patterns

#### Hybrid Audio Backend

The application supports **two audio backends** that can be switched at runtime or compile-time:

1. **Web Audio API Backend**: Uses `web-audio-api` crate for cross-platform audio via browser-like API
   - Located in `src/audio/web_bridge.rs`
   - Accessed via `AudioContext`, `AudioBufferSourceNode`, `BiquadFilterNode`, `AnalyserNode`
   - DSP graph: Source → EQ → Gain → Analyser → Output

2. **Native CPAL Backend**: Uses `cpal` for low-latency native audio
   - Located in `src/audio/device.rs`
   - Direct hardware access with callback-based audio rendering
   - Sample format conversion and device enumeration

**Abstraction Layer**: `src/audio/backend.rs` defines the `AudioBackend` trait that both implementations satisfy. `HybridAudioBackend` in `src/audio/hybrid.rs` orchestrates switching between backends based on:
- Performance requirements (CPAL for <5ms latency)
- Platform capabilities (WASM uses Web Audio only)
- User preferences (configurable in Settings)

**Communication Pattern**: UI thread → lock-free ring buffer → audio callback thread. Uses `HybridRingBuffer` to bridge the UI and audio processing with minimal latency.

#### DSP Pipeline Architecture

All audio processing flows through a shared pipeline regardless of backend:

```
File/Signal Generator → Decoder → Source Node → EQ (8-band parametric) → 
Effects Chain → Gain Node → Analyser (FFT) → Output Device
```

- **8-band parametric equalizer**: Frequencies at 60, 120, 240, 480, 960, 1920, 3840, 7680 Hz
- **Real-time spectrum analysis**: 512-point FFT with optimized processing (`audio_performance::OptimizedSpectrumProcessor`)
- **Effects chain**: Reverb, delay, compression (expandable via `Effect` trait)
- **Signal generator**: 8 mathematical waveforms (sine, square, triangle, sawtooth, white noise, pink noise, chirp, multi-tone)

#### Accessibility & Safety Systems

**Accessibility Framework** (`src/ui/accessibility.rs`):
- Centralized `AccessibilityManager` for all a11y controls
- Every interactive element has ARIA labels
- Full keyboard navigation (Tab, Arrow keys, Enter, Space)
- Screen reader announcements via platform APIs
- High contrast mode support
- Accessible controls: `AccessibleSlider`, `AccessibleKnob`, `AccessibleButton`

**Safety Features** (`src/security/audio_safety.rs`):
- **Emergency volume reduction**: Press `Escape` for instant volume to 20%
- **Volume limiting**: Maximum 85dB SPL equivalent
- **Gradual volume changes**: 100ms ramp to prevent hearing damage
- **Visual safety indicators**: Color-coded volume warnings (green < 70%, yellow < 85%, red ≥ 85%)
- **Audio warnings**: Screen reader announcements for dangerous levels
- **Safety validation**: All volume operations pass through `volume_safety::validate_volume()`

#### Error Recovery System

`ErrorManager` in `src/ui/error_handling.rs` provides structured error handling:

```rust
enum AudioError {
    DeviceNotFound { recovery: RecoveryAction },
    FormatUnsupported { fallback: AudioFormat },
    PlaybackFailed { context: ErrorContext },
}
```

Every error includes:
- User-friendly message
- Specific recovery action suggestions
- Accessibility announcements
- Context for debugging

## Critical Development Guidelines

### PRIME DIRECTIVE: Anti-Duplication

**Zero tolerance** for file name variants. NEVER create:
- `enhanced_*` / `simple_*` variants
- `*_v2` / `*_new` versions
- `improved_*` / `refactored_*` copies

Always edit canonical files directly. Violating this principle breaks the entire codebase organization.

### Audio Safety Requirements

**All audio operations MUST**:
1. Pass through safety validation (`volume_safety::validate_volume()`)
2. Use gradual transitions (no instant jumps > 10dB)
3. Respect maximum gain ceiling (85dB SPL)
4. Update visual safety indicators
5. Trigger accessibility announcements for warnings

**Tests validate**:
- Volume cannot exceed safety limits (`tests/safety_protection_tests.rs`)
- Emergency reduction responds < 100ms
- Ramp times meet hearing safety standards

### Compilation Requirements

**Before committing**:
```powershell
# Format code (mandatory)
cargo make fmt

# Lint with deny-warnings
cargo make clippy
# Must pass with zero warnings

# Compile check
cargo make check

# Run tests
cargo make test
```

**Release builds** use aggressive optimizations:
- LTO enabled (`lto = true`)
- Single codegen unit (`codegen-units = 1`)
- Strip symbols (`strip = true`)
- Opt-level 3

### Windows-Specific Considerations

This project is developed on **Windows with PowerShell (pwsh)**:
- Use PowerShell scripts in `.\scripts\` for automation
- Binary outputs should be versioned: `rusty-audio_v{VERSION}.exe`
- Test on HiDPI displays (125%, 150%, 200% scaling)
- Verify with Windows Narrator for accessibility

Python tooling (if used) leverages `uv python` from `~\bin` or `~\.local\bin` per user rules.

## Testing Strategy

### Test Organization

```
tests/
├── comprehensive_unit_tests.rs       # Core functionality tests
├── integration.rs                    # Full system integration tests
├── mathematical_framework_tests.rs   # Signal verification tests
├── property_based_tests.rs           # Property testing with proptest
├── safety_protection_tests.rs        # Safety feature validation
└── pipeline_integration_tests.rs     # DSP pipeline tests

benches/
├── audio_benchmarks.rs               # Core audio processing benchmarks
├── performance_benchmarks.rs         # Overall performance metrics
├── realtime_benchmarks.rs            # Latency and real-time guarantees
├── audio_quality_benchmarks.rs       # Signal quality measurements
└── memory_benchmarks.rs              # Memory usage and allocation
```

### Testing Commands

```powershell
# Quick validation
cargo test --lib --quiet

# Comprehensive testing
cargo test --all -- --nocapture

# Test specific module
cargo test ui::accessibility --exact --nocapture

# Property-based tests (slower)
cargo test property_tests

# Mathematical signal verification
cargo test mathematical_framework

# Run test automation with reporting
.\scripts\run_tests.ps1 -TestMode comprehensive -GenerateReport
```

### Manual Testing Requirements

**Cannot be automated** - must test manually:
1. Audio playback with different formats (MP3, WAV, FLAC, OGG, M4A)
2. Screen reader functionality (NVDA, JAWS, Windows Narrator)
3. Keyboard navigation across all tabs
4. HiDPI scaling at 125%, 150%, 200%
5. Multi-monitor setups with different DPI
6. Emergency volume reduction (press Escape during playback)
7. Visual safety indicators respond to volume changes
8. Theme switching maintains car-stereo aesthetic

See `TESTING_PROCEDURES.md` and `VALIDATION_CRITERIA.md` for detailed checklists.

### Benchmarking

```powershell
# Run all benchmarks (uses Criterion)
cargo bench --no-fail-fast

# Specific benchmark
cargo bench audio_benchmarks

# Performance analysis script
.\run_performance_analysis.ps1
```

**Performance Targets**:
- GUI rendering: 60+ FPS
- Audio latency: <10ms processing
- Memory usage: ~50MB baseline
- CPU usage: <5% idle, <15% active playback
- Startup time: <500ms to interactive

## Signal Generator

8 mathematical signal types for testing (`src/ui/signal_generator.rs`):

1. **Sine wave**: Pure tones, frequency response testing
2. **Square wave**: Harmonic distortion analysis
3. **Triangle wave**: Linearity testing
4. **Sawtooth wave**: Aliasing analysis
5. **White noise**: Full spectrum testing
6. **Pink noise**: Perceptual testing (1/f spectrum)
7. **Chirp**: Frequency sweep analysis (20Hz - 20kHz)
8. **Multi-tone**: Intermodulation distortion testing

All generators support:
- Frequency range: 20Hz - 20kHz
- Amplitude control with safety limits
- Real-time waveform visualization
- Mathematical verification via `testing::mathematical_framework`

## Key Dependencies

**Core**:
- `egui/eframe 0.27.2` - Immediate mode GUI
- `web-audio-api 1.2.0` - Audio processing (custom local fork)
- `cpal 0.15` - Native audio backend
- `rodio 0.17` - Audio decoding
- `symphonia 0.5` - Professional audio decoding

**DSP**:
- `rustfft 6.0` - FFT for spectrum analysis
- `rubato 0.15` - Sample rate conversion

**Safety & Accessibility**:
- `parking_lot 0.12` - Lock-free synchronization
- Custom accessibility framework (no external dependency)

**Testing**:
- `criterion 0.5` - Benchmarking
- `proptest 1.0` - Property-based testing
- `rstest 0.18` - Parameterized testing

## Local Dependencies

**web-audio-api-rs** library at `libs/web-audio-api-rs/`:
- Custom fork with local modifications
- To rebuild: `cd libs/web-audio-api-rs && cargo build --release`
- Check API compatibility when updating

## Troubleshooting

### Compilation Errors

```powershell
# Check errors with logging
.\compile_check.ps1
# Outputs to: compile_errors.log

# Auto-fix common issues
cargo make fix
```

### Audio Issues

If audio doesn't work:
1. Check audio device selection in Settings tab
2. Verify backend mode (Web Audio vs CPAL) in Settings
3. Try switching backends via `HybridMode` toggle
4. Check system audio device permissions
5. Verify sample rate compatibility (44.1kHz or 48kHz)

### Performance Issues

```powershell
# Enable performance monitoring
cargo run --release --features performance-monitor

# Run performance benchmarks
cargo bench

# Check optimized spectrum processor
# (should use SIMD when available)
```

### Accessibility Issues

Test with screen readers:
- **Windows**: Windows Narrator (Win + Ctrl + Enter)
- **NVDA**: Free screen reader for Windows
- **JAWS**: Commercial screen reader

Verify all controls announce their state and have ARIA labels.

## Documentation Resources

**In Repository**:
- `DEVELOPER_GUIDE.md` - Detailed API documentation and architecture
- `PROJECT_CONTEXT.md` - Development history and decisions
- `USER_MANUAL.md` - End-user documentation
- `TESTING_PROCEDURES.md` - Manual testing checklists
- `VALIDATION_CRITERIA.md` - Acceptance criteria
- `CLAUDE.md` - AI agent collaboration guidelines
- `SIGNAL_GENERATOR_GUIDE.md` - Signal generation documentation
- `SAFETY_ACCESSIBILITY.md` - Safety and accessibility guidelines

**Generated Docs**:
```powershell
cargo doc --no-deps --open
```

## Example Commands

```powershell
# Standard development cycle
cargo check && cargo clippy && cargo run

# Pre-commit validation
cargo make fmt
cargo make clippy
cargo test --lib

# Release build with validation
cargo build --release
cargo clippy -- -D warnings
.\scripts\run_tests.ps1 -TestMode comprehensive

# Run mathematical testing demo
cargo run --example mathematical_testing_demo

# Quick iteration (watch mode, requires cargo-watch)
cargo make watch
```

## Important Notes

1. **Always test audio safety features** after modifying audio pipeline
2. **Verify accessibility** with screen reader after UI changes
3. **Run benchmarks** after performance-sensitive changes
4. **Test on HiDPI displays** for visual correctness
5. **Check safety indicators** update correctly during volume changes
6. **Binary naming**: Use versioned names for releases (e.g., `rusty-audio_v0.1.0.exe`)
7. **Python tooling**: Use `uv python` from `~\.local\bin` if needed per user rules
