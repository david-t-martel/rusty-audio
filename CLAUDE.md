# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Rusty Audio** is a car-stereo-style desktop audio player built with Rust and the egui GUI framework. It features real-time spectrum visualization, an 8-band equalizer, multiple themes, comprehensive audio format support, AI-enhanced processing, and professional recording capabilities.

### Current Architecture
The application has evolved from a monolithic structure into a **modularized library + binary architecture**:

- **Main application**: `src/main.rs` (~81KB) - Primary UI and coordination
- **Library modules**: Extensive modularization in `src/lib.rs` exposing reusable components
- **Module organization**: Separate directories for `audio/`, `ui/`, `ai/`, `security/`, `testing/`

### Core Technologies
- **Rust 2021 Edition** - Primary language
- **egui/eframe v0.33.0** - Immediate mode GUI framework (upgraded from 0.27)
- **web-audio-api v1.2.0** - Git submodule dependency for Web Audio API pattern
- **Hybrid Audio Backend** - Dual backend system (cpal + web-audio-api)
- **lofty v0.22.4** - Audio metadata extraction
- **Comprehensive test framework** - Property tests, benchmarks, UI testing with egui_kittest

## Build Commands

### Development
```bash
# Build and run (debug)
cargo run

# Build and run specific binary
cargo run --bin test_audio

# Release build (optimized with LTO)
cargo build --release
cargo run --release

# Quick compilation check
cargo check
```

### Testing
```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test module
cargo test audio::
cargo test ui::
cargo test testing::

# Run UI tests with kittest
cargo test ui_tests

# Run property-based tests
cargo test property_tests
```

### Benchmarking
```bash
# Run all benchmarks
cargo bench

# Run specific benchmark suite
cargo bench --bench audio_benchmarks
cargo bench --bench performance_benchmarks
cargo bench --bench realtime_benchmarks
cargo bench --bench memory_benchmarks
cargo bench --bench audio_quality_benchmarks
```

### Code Quality
```bash
# Format code (required before commits)
cargo fmt

# Lint with clippy
cargo clippy

# Strict linting for release
cargo clippy -- -D warnings

# Generate documentation
cargo doc --open
```

### Pre-commit Hooks
```bash
# Install hooks (run once)
pre-commit install

# Run all checks manually
pre-commit run --all-files

# Hooks run automatically on commit
git commit -m "your message"
```

## High-Level Architecture

### Module Organization
The project uses a library + binary architecture with the following structure:

```
src/
├── main.rs                    # Main application binary (81KB)
├── lib.rs                     # Library root with public API
├── audio/                     # Audio backend and processing
│   ├── backend.rs            # Backend trait abstraction
│   ├── device.rs             # CPAL device management
│   ├── manager.rs            # Audio device manager
│   ├── hybrid.rs             # Hybrid dual-backend system
│   ├── web_bridge.rs         # Web Audio API bridge
│   └── recorder.rs           # Recording functionality
├── ui/                        # User interface components
│   ├── components.rs         # Album art, progress, metadata
│   ├── controls.rs           # Sliders, knobs, buttons
│   ├── spectrum.rs           # Spectrum visualizer
│   ├── theme.rs              # Theme management
│   ├── layout.rs             # Layout manager
│   ├── dock_layout.rs        # Docking system
│   ├── accessibility.rs      # Accessibility features
│   ├── signal_generator.rs   # Signal generator panel
│   ├── recording_panel.rs    # Recording UI
│   └── error_handling.rs     # Error manager UI
├── ai/                        # AI-enhanced features
│   ├── audio_analyzer.rs     # Audio analysis
│   ├── eq_optimizer.rs       # EQ optimization
│   ├── noise_reduction.rs    # Noise reduction
│   ├── volume_normalizer.rs  # Volume normalization
│   ├── preset_recommender.rs # Preset recommendations
│   └── ...                   # Additional AI modules
├── security/                  # Security and safety
│   ├── audio_safety.rs       # Audio safety limits
│   ├── file_validator.rs     # File validation
│   ├── input_validator.rs    # Input validation
│   └── thread_safe_state.rs  # Thread-safe state
├── testing/                   # Test framework
│   ├── signal_generators.rs  # Test signal generation
│   ├── spectrum_analysis.rs  # Spectrum analysis tests
│   ├── equalizer_tests.rs    # EQ tests
│   ├── ui_tests.rs           # UI tests with kittest
│   └── property_tests.rs     # Property-based tests
├── bin/                       # Additional binaries
│   └── test_audio.rs         # Audio testing utility
└── benches/                   # Benchmark suites
    ├── audio_benchmarks.rs
    ├── performance_benchmarks.rs
    ├── realtime_benchmarks.rs
    ├── memory_benchmarks.rs
    └── audio_quality_benchmarks.rs
```

### Key Architectural Components

**Hybrid Audio Backend**:
The application uses a dual-backend system (`HybridAudioBackend`) that can switch between:
- **CPAL backend**: Native low-latency audio (primary)
- **Web Audio API**: Fallback with advanced DSP features
- **Automatic fallback**: Graceful degradation on backend failures

**UI Tabs**:
- **Playback**: File selection, play controls, metadata display, album art
- **Effects**: Real-time spectrum visualizer with 512-point FFT analysis
- **EQ**: 8-band parametric equalizer (60Hz to 7680Hz)
- **Generator**: Signal generator for testing (sine, square, sawtooth, noise)
- **Recording**: Audio recording with monitoring and format selection
- **Settings**: Theme selection, accessibility options

**Audio Processing Pipeline**:
`AudioContext` → `AudioBufferSourceNode` → `BiquadFilterNode` (x8 for EQ) → `GainNode` → `AnalyserNode` → Output

**Testing Framework**:
- Property-based testing with `proptest` and `quickcheck`
- UI testing with `egui_kittest`
- Comprehensive benchmark suites for performance tracking
- Signal generation and spectrum analysis utilities

## Web Audio API Submodule

### Important: Git Submodule Dependency
The `web-audio-api-rs` library is included as a **git submodule**, not a local path dependency:

```bash
# Initialize submodule after cloning
git submodule update --init --recursive

# Update submodule to latest commit
cd web-audio-api-rs
git pull origin main
cd ..
git add web-audio-api-rs
git commit -m "Update web-audio-api submodule"
```

**Location**: `web-audio-api-rs/` in project root

**When updating**: The submodule tracks upstream changes from the web-audio-api-rs repository. Any local modifications should be contributed back upstream or maintained in a fork.

### Submodule Workflow
```bash
# Check submodule status
git submodule status

# Update all submodules
git submodule update --remote

# Build the submodule (if needed for testing)
cd web-audio-api-rs
cargo build --release
cargo test
```

## Testing Strategy

### Comprehensive Test Coverage
The project includes multiple testing approaches:

**1. Unit Tests**: Embedded throughout modules
```bash
cargo test                    # Run all tests
cargo test audio::            # Test audio module
cargo test ui::               # Test UI components
cargo test security::         # Test security module
```

**2. Property-Based Tests**: Using `proptest` and `quickcheck`
```bash
cargo test property_tests
```

**3. UI Tests**: Automated UI testing with `egui_kittest`
```bash
cargo test ui_tests
```

**4. Integration Tests**: Full feature integration tests
```bash
cargo test testing::integration_tests
```

**5. Benchmark Suites**: Performance regression detection
```bash
cargo bench                                    # Run all benchmarks
cargo bench --bench audio_benchmarks          # Audio processing
cargo bench --bench performance_benchmarks    # General performance
cargo bench --bench realtime_benchmarks       # Real-time constraints
cargo bench --bench memory_benchmarks         # Memory usage
cargo bench --bench audio_quality_benchmarks  # Quality metrics
```

### Test Signal Generation
The `testing::signal_generators` module provides utilities for generating test signals:
- Sine waves at various frequencies
- Square, sawtooth, triangle waves
- White noise, pink noise
- Frequency sweeps (chirps)
- Impulse responses for convolution testing

## Audio Format Support

### Supported Formats (via Symphonia)
- **Lossless**: WAV, FLAC
- **Lossy**: MP3, OGG Vorbis, M4A, AAC
- **Professional**: Advanced decoding with `symphonia` crate (opt-simd enabled)

### Audio Features
- Real-time spectrum analysis (512-point FFT via `rustfft`)
- 8-band parametric equalizer (60Hz to 7680Hz)
- Master volume and stereo panning controls
- Signal generator (sine, square, sawtooth, noise)
- Audio recording with monitoring (WAV format via `hound`)
- Sample rate conversion (`rubato`)
- MIDI I/O support (`midir`, `wmidi`)

### Audio Processing Pipeline
1. **File Loading**: Via `rfd` cross-platform file dialog
2. **Metadata Extraction**: Using `lofty` crate
3. **Audio Decoding**: Dual path:
   - **Web Audio API**: For web-compatible processing
   - **Symphonia**: For native high-performance decoding
4. **DSP Processing**: EQ → Effects → Gain → Analysis
5. **Visualization**: Real-time spectrum display with customizable modes
6. **Output**: Hybrid backend (CPAL native or Web Audio fallback)

## Performance Considerations

### GUI Framework
- **Immediate Mode**: egui 0.33 redraws UI each frame (efficient for real-time)
- **Docking System**: `egui_dock` for flexible panel layouts
- **Optimized Rendering**: Efficient for audio visualizations
- **Cross-Platform**: Windows, Linux, macOS via eframe

### Audio Processing Optimizations
- **SIMD**: Enabled in Symphonia decoder (`opt-simd` feature)
- **FFT Performance**: `rustfft` for efficient spectrum analysis
- **Parallel Processing**: `rayon` for multi-threaded operations
- **Low-Latency Audio**: CPAL backend for minimal audio latency
- **Lock-Free Sync**: `parking_lot` for efficient synchronization
- **Memory Pooling**: LRU caches for efficient resource management

### Build Optimizations
Release profile configured for maximum performance:
- **LTO**: Link-time optimization enabled
- **Codegen Units**: 1 for better optimization
- **Optimization Level**: 3 (maximum)
- **Strip**: Debug symbols removed
- **Panic**: Abort strategy for smaller binary

## Development Best Practices

### Before Making Changes
1. **Update submodules**: Ensure `web-audio-api-rs` is current with `git submodule update`
2. **Run tests first**: Verify existing functionality with `cargo test`
3. **Check benchmarks**: Run relevant benchmarks to establish baseline
4. **Review module structure**: Understand which module owns the functionality

### Code Style Requirements
- **Pre-commit hooks mandatory**: Automatic `cargo fmt` and `clippy` checks
- **No unwrap() in library code**: Use `Result` and `Option` with proper error handling
- **Security validation**: Use `security::input_validator` for user inputs
- **Thread safety**: Use `security::thread_safe_state` for shared state
- **Property tests**: Add property-based tests for new algorithms

### Critical Testing Requirements
1. **Audio correctness**: Verify signal processing with `testing::signal_generators`
2. **Real-time performance**: Use benchmark suites to detect regressions
3. **UI consistency**: Test with `egui_kittest` for UI components
4. **Memory safety**: Run tests under Miri when modifying unsafe code
5. **Cross-platform**: Test on Windows, Linux, macOS when possible

### Security Considerations
- **Audio safety limits**: Enforce maximum volume via `security::audio_safety`
- **File validation**: Validate audio files before processing with `security::file_validator`
- **Input sanitization**: All user inputs must pass through validators
- **Thread-safe state**: Use provided abstractions for concurrent access

## AI Features Integration

The `ai/` module provides AI-enhanced audio processing:
- **Audio Analysis**: Automatic genre, mood, tempo detection
- **EQ Optimization**: ML-based EQ suggestions
- **Noise Reduction**: Spectral noise reduction algorithms
- **Volume Normalization**: Intelligent loudness normalization
- **Preset Recommendations**: User preference learning

Enable AI features by uncommenting optional dependencies in `Cargo.toml` (candle-core, linfa, smartcore).

## Known Current State

1. **Modular architecture complete**: Successfully refactored from monolithic design
2. **Hybrid audio backend**: Dual-backend system operational
3. **Comprehensive testing**: Property tests, UI tests, benchmarks in place
4. **Theme system**: Using egui 0.33 native theming (catppuccin removed)
5. **Recording functionality**: Audio recording with monitoring implemented

## Quick Development Reference

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

## Additional Resources

### Documentation
- Run `cargo doc --open` to generate and view comprehensive API documentation
- Module-level docs explain architecture decisions and usage patterns

### Examples
The `web-audio-api-rs/examples/` directory contains 30+ examples demonstrating:
- Audio node usage patterns
- Audio effects implementations
- Worklet and processor examples
- Media streams and recording

### Benchmarking Results
After running `cargo bench`, view detailed results in `target/criterion/`:
- HTML reports with performance graphs
- Baseline comparisons for regression detection
- Statistical analysis of benchmark runs