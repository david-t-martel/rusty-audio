# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Rusty Audio** is a car-stereo-style desktop audio player built with Rust and the egui GUI framework. It features real-time spectrum visualization, an 8-band equalizer, multiple themes, and comprehensive audio format support.

### Current Architecture
The application uses a **monolithic architecture** with all functionality in a single `src/main.rs` file (~507 lines). While functional, this violates SOLID principles and presents refactoring opportunities.

### Core Technologies
- **Rust 2021 Edition** - Primary language
- **egui/eframe v0.27.2** - Immediate mode GUI framework
- **web-audio-api** - Custom local dependency for audio processing
- **lofty v0.22.4** - Audio metadata extraction
- **catppuccin-egui v5.6.0** - Theme system (currently has compilation errors)

## Build Commands

### Development
```bash
# Build and run (debug)
cargo run

# Build only
cargo build

# Release build (optimized)
cargo build --release
cargo run --release

# Quick compilation check
cargo check
```

### Code Quality
```bash
# Format code (required before commits)
cargo fmt

# Lint and check best practices
cargo clippy

# Strict linting for release
cargo clippy -- -D warnings

# Documentation
cargo doc
```

### Pre-commit Hooks
```bash
# Install hooks (run once)
pre-commit install

# Run all checks manually
pre-commit run --all-files

# Automatic checks on commit
git commit -m "your message"
```

## High-Level Architecture

### Current Structure (Single File)
The entire application resides in `src/main.rs` with these key components:

**Core State**: `AudioPlayerApp` struct manages all application state including audio context, UI state, and file handling.

**UI Tabs**:
- **Playback**: File selection, play controls, metadata display, album art
- **Effects**: Real-time spectrum visualizer with 512-point FFT analysis
- **EQ**: 8-band parametric equalizer (60Hz to 7680Hz)
- **Settings**: Theme selection (6 themes including Catppuccin variants)

**Audio Pipeline**: Uses Web Audio API pattern with `AudioContext` → `AudioBufferSourceNode` → `BiquadFilterNode` → `GainNode` → `AnalyserNode`

### Critical Architecture Issues

**SOLID Principle Violations**:
1. **SRP**: `AudioPlayerApp` handles UI, audio, file management, and state
2. **OCP**: Adding features requires modifying existing code
3. **DIP**: Direct dependencies on concrete implementations

**Current Compilation Issues**:
- Theme system broken due to `catppuccin-egui` API changes
- Needs dependency updates and API compatibility fixes

## Development Status (from GEMINI.md)

### Completed ✅
- Real-time spectrum visualizer
- 8-band equalizer implementation
- UI/UX improvements with theming

### Medium Priority 🔄
- Add support for more audio formats
- Implement playlist system
- Add audio effects (reverb, delay)

### Low Priority 📋
- Enhanced theme support
- Plugin system architecture
- Comprehensive documentation

## Refactoring Priorities

### Immediate Actions Required
1. **Fix Compilation**: Update `catppuccin-egui` integration
2. **Error Handling**: Replace `unwrap()` calls with proper error handling
3. **Add Testing**: Create unit tests for audio processing logic
4. **Add Logging**: Implement debugging and error logging

### Architectural Refactoring (SOLID Compliance)
Break `main.rs` into modules:

```
src/
├── main.rs          # Entry point only
├── app.rs           # Main coordinator
├── audio/           # Audio processing
│   ├── engine.rs    # Core audio engine
│   ├── effects.rs   # EQ and effects
│   └── analyzer.rs  # Spectrum analysis
├── ui/              # User interface
│   ├── player.rs    # Main UI
│   ├── controls.rs  # Playback controls
│   └── themes.rs    # Theme management
├── file/            # File operations
│   └── metadata.rs  # Metadata extraction
└── config.rs        # Settings persistence
```

## Local Dependencies

### web-audio-api-rs Library
Located at `libs/web-audio-api-rs/` - custom fork with local modifications.

**To rebuild the dependency**:
```bash
cd libs/web-audio-api-rs
cargo build --release
```

**When updating**: Check for API compatibility issues as this is a local fork that may diverge from upstream.

## Testing Strategy

### Current State
- **No unit tests** in main project (critical gap)
- Dependency tests exist in `web-audio-api-rs`
- Pre-commit hooks ensure formatting/linting

### Recommended Test Structure
```bash
# Future test commands
cargo test                    # Run all tests
cargo test audio::            # Test audio module
cargo test ui::               # Test UI components
cargo test --release          # Test optimized builds
```

## Audio Format Support

### Currently Supported
- MP3, WAV, FLAC, OGG, M4A, AAC
- Real-time spectrum analysis (512-point FFT)
- 8-band parametric equalizer
- Master volume and panning controls

### Audio Processing Pipeline
1. **File Loading**: Via `rfd` file dialog
2. **Metadata Extraction**: Using `lofty` crate
3. **Audio Decoding**: Through `web-audio-api` integration
4. **DSP Processing**: EQ → Gain → Analysis nodes
5. **Visualization**: Real-time spectrum display

## Performance Considerations

### GUI Framework
- **Immediate Mode**: egui redraws entire UI each frame
- **Efficient for Audio**: Good for real-time spectrum updates
- **Cross-Platform**: Works on Windows, Linux, macOS

### Audio Processing
- **Real-time Analysis**: 512-point FFT for spectrum visualization
- **Low Latency**: Direct audio buffer processing
- **Memory Efficient**: Uses `parking_lot` for synchronization

## Development Best Practices

### Before Making Changes
1. **Search existing code** for similar functionality
2. **Fix compilation errors** before adding features
3. **Run pre-commit checks** before committing
4. **Test audio playback** after audio system changes

### Code Style Requirements
- **Pre-commit hooks mandatory** - run `pre-commit install`
- **Rust formatting** with `cargo fmt`
- **Clippy linting** must pass with `-D warnings`
- **No `unwrap()` in production code** - use proper error handling

### Critical Testing
- **Manual audio testing required** - automated tests can't verify audio output
- **Test all supported formats** when modifying audio pipeline
- **Verify spectrum visualizer** shows correct frequency response
- **Test theme switching** for UI consistency

## Known Issues

1. **Compilation Errors**: `catppuccin-egui` API compatibility
2. **Monolithic Architecture**: Single file limits maintainability
3. **No Error Recovery**: Application crashes on invalid files
4. **Missing Tests**: No automated verification of audio processing
5. **No Settings Persistence**: Theme/EQ settings not saved between runs

## Quick Development Reference

```bash
# Standard development cycle
cargo check && cargo clippy && cargo run

# Pre-commit validation
pre-commit run --all-files

# Release preparation
cargo build --release && cargo clippy -- -D warnings
```