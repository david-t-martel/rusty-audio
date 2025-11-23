# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Rusty Audio** is a cross-platform audio player with both desktop (native) and web (WASM/PWA) implementations. It features real-time spectrum visualization, an 8-band equalizer, multiple themes, comprehensive audio format support, and professional recording capabilities with ASIO support on Windows.

### Workspace Architecture

The project uses a **Cargo workspace** with three main components:

```
rusty-audio/
├── Cargo.toml                 # Workspace root with shared dependencies
├── rusty-audio-core/          # Shared library (platform-agnostic)
│   └── src/
│       ├── audio/             # Audio backends, DSP, effects
│       ├── ui/                # UI components, themes, layouts
│       ├── ai/                # AI-enhanced features (optional)
│       └── security/          # Input validation, safety limits
├── rusty-audio-desktop/       # Native desktop application
│   ├── src/main.rs            # Desktop entry point
│   └── benches/               # Performance benchmarks
└── rusty-audio-web/           # WASM/PWA web application
    └── src/
        ├── lib.rs             # WASM entry point
        └── auth/              # OAuth 2.0 with PKCE
```

**Core Technologies:**
- **Rust 2021** with workspace dependency management
- **egui/eframe 0.33** - Immediate mode GUI
- **CPAL 0.15** with ASIO support (Windows desktop)
- **web-audio-api 1.2** - Advanced DSP (native backend)
- **WASM** - Web deployment with multithreading support
- **OAuth 2.0** - Google, GitHub, Microsoft authentication (web only)

### Key Architectural Patterns

1. **Feature-Gated Compilation**: Core library supports both `native` and `wasm` features
2. **Hybrid Audio Backend**: Automatic fallback between CPAL and Web Audio API
3. **Shared UI Components**: Same egui code runs on desktop and web
4. **Platform-Specific Optimizations**: ASIO on Windows, lock-free ring buffers for real-time audio

## Build Commands

### Workspace Commands

```bash
# Build all workspace members
cargo build --workspace

# Test all workspace members
cargo test --workspace

# Check compilation without building
cargo check --workspace

# Build specific package
cargo build -p rusty-audio-desktop --release
cargo build -p rusty-audio-core --features native
```

### Desktop Application

```bash
# Run desktop app (debug)
cd rusty-audio-desktop
cargo run

# Run desktop app (release, better audio performance)
cargo run --release

# Windows with ASIO support (automatically enabled on Windows)
cargo build --release  # ASIO features enabled via Cargo.toml

# Run benchmarks
cargo bench --bench audio_benchmarks
cargo bench --bench performance_benchmarks
```

### WASM/Web Application

```bash
# Build WASM (development)
cd rusty-audio-web
wasm-pack build --target web --dev

# Build WASM (release, optimized)
wasm-pack build --target web --release

# Build with Trunk (includes all assets)
trunk build --release

# Serve with live reload
trunk serve
# Open http://localhost:8080

# Run WASM tests in headless browsers
wasm-pack test --headless --firefox
```

### Using Justfile (Recommended)

The project includes a comprehensive `justfile` with 100+ commands:

```bash
# Quick development cycle
just check          # Fast compile check
just build          # Build debug version
just run            # Run desktop app
just test           # Run all tests

# WASM/PWA workflow
just build-wasm     # Build WASM (dev)
just serve-wasm     # Start Trunk dev server
just pwa-build      # Build complete PWA bundle
just test-wasm-full # Full WASM test suite

# Code quality
just fmt            # Format code
just lint           # Run clippy
just quality        # Format + lint + test
just pre-commit     # Full pre-commit check

# Platform-specific
just build-windows-asio  # Windows with ASIO

# View all commands
just --list
just help
```

## Testing

### Test Organization

```bash
# All workspace tests
cargo test --workspace

# Core library tests only
cargo test -p rusty-audio-core

# Desktop app tests
cargo test -p rusty-audio-desktop

# Specific test module
cargo test -p rusty-audio-core audio::
cargo test -p rusty-audio-desktop -- --nocapture

# Property-based tests
cargo test -p rusty-audio-core --features property-testing

# UI tests with egui_kittest
cargo test ui_tests
```

### WASM Testing

```bash
# Using wasm-pack
cd rusty-audio-web
wasm-pack test --headless --firefox --chrome

# Using justfile
just test-wasm-headless   # Headless browsers
just test-wasm-browser    # Interactive browser tests
just test-wasm-full       # Build + serve + test + cleanup
```

### Benchmarks (Desktop Only)

```bash
cd rusty-audio-desktop
cargo bench

# Specific benchmark suites
cargo bench --bench audio_benchmarks          # Audio processing
cargo bench --bench performance_benchmarks    # General performance
cargo bench --bench simd_benchmarks           # SIMD optimizations
cargo bench --bench optimization_benchmarks   # Real-time constraints

# View results
open target/criterion/report/index.html
```

## High-Level Architecture

### Core Library (`rusty-audio-core`)

**Purpose**: Platform-agnostic audio processing and UI components shared between desktop and web.

**Key Modules**:
- `audio::backend` - Audio backend trait abstraction
- `audio::hybrid` - Dual-backend system (CPAL + Web Audio API)
- `audio::manager` - Device management and audio routing
- `ui::components` - Reusable UI widgets (progress bars, album art, metadata)
- `ui::spectrum` - Real-time spectrum analyzer (512-point FFT)
- `ui::theme` - Theme management system
- `security::audio_safety` - Volume limiting and safety checks
- `security::input_validator` - Input sanitization

**Features**:
- `native` - Enable native audio (CPAL, file dialogs, recording)
- `wasm` - Enable WASM bindings (web-sys, wasm-bindgen)
- `audio-optimizations` - Real-time thread priority, lock-free structures
- `ai-features` - AI-enhanced processing (optional)
- `property-testing` - Property-based tests (dev only)

### Desktop Application (`rusty-audio-desktop`)

**Purpose**: Native desktop app with professional audio interface support.

**Key Features**:
- ASIO support on Windows (low-latency, <10ms)
- MIDI I/O with `midir` and `wmidi`
- Audio recording to WAV files
- File dialogs with `rfd`
- Native performance benchmarks

**Entry Point**: `src/main.rs`
- Initializes `eframe::NativeOptions` with WGPU backend
- Sets up audio context with device detection
- Configures desktop-specific features (MMCSS thread priority on Windows)

### Web Application (`rusty-audio-web`)

**Purpose**: Progressive Web App with OAuth authentication.

**Key Features**:
- OAuth 2.0 with PKCE (Google, GitHub, Microsoft)
- Secure token storage in `localStorage`
- WASM multithreading with `wasm-bindgen-rayon`
- Service Worker for offline support
- Web Audio API integration

**Entry Point**: `src/lib.rs` with `#[wasm_bindgen]`
- Exports WASM entry points
- Configures panic hook for browser console
- Sets up Web Audio context

**OAuth Flow** (`src/auth/`):
1. `oauth_client.rs` - PKCE challenge generation, token exchange
2. `providers.rs` - Provider-specific endpoints (Google, GitHub, Microsoft)
3. `session.rs` - Session management with automatic refresh
4. `token_storage.rs` - Secure storage in `localStorage`

### Audio Processing Pipeline

```
Input (File/Device)
  ↓
Decoder (Symphonia/Web Audio)
  ↓
AudioBufferSourceNode
  ↓
BiquadFilterNode (x8 for EQ: 60Hz, 120Hz, 240Hz, 480Hz, 960Hz, 1920Hz, 3840Hz, 7680Hz)
  ↓
GainNode (Master Volume)
  ↓
StereoPannerNode
  ↓
AnalyserNode (512-point FFT for visualization)
  ↓
Output (CPAL/Web Audio Destination)
```

### Hybrid Backend System

The `audio::hybrid` module provides automatic fallback:

```rust
HybridAudioBackend {
    primary: CpalBackend,     // Low-latency native audio
    fallback: WebAudioBackend // Advanced DSP features
}
```

**Fallback Triggers**:
- Device initialization failure
- Sample rate mismatch
- Buffer underruns
- Explicit user selection

## Development Workflow

### Initial Setup

```bash
# Clone with submodules
git clone --recursive <repo-url>
cd rusty-audio

# Or initialize submodules if already cloned
git submodule update --init --recursive

# Install Rust tools
rustup target add wasm32-unknown-unknown
cargo install wasm-pack trunk

# Install optional tools
just install-tools       # cargo-watch, cargo-outdated, etc.
just install-wasm-tools  # wasm-pack, trunk, wasm-bindgen-cli
```

### Making Changes

1. **Modify core library** (`rusty-audio-core/src/`)
   ```bash
   cd rusty-audio-core
   cargo check --features native,wasm  # Check both platforms
   cargo test --all-features
   ```

2. **Test on desktop** (`rusty-audio-desktop/`)
   ```bash
   cd rusty-audio-desktop
   cargo run                    # Quick test
   cargo run --release          # Performance test
   cargo bench                  # Regression check
   ```

3. **Test on web** (`rusty-audio-web/`)
   ```bash
   cd rusty-audio-web
   wasm-pack build --dev
   trunk serve                  # Live reload
   wasm-pack test --headless --firefox
   ```

### Pre-Commit Checklist

```bash
# Using justfile (recommended)
just pre-commit    # Runs: fmt + lint + test + check-bin

# Or manually
cargo fmt --all
cargo clippy --workspace -- -D warnings
cargo test --workspace
cargo check -p rusty-audio-desktop --bin rusty-audio
```

### Common Issues

**WASM build fails with linking errors:**
```bash
# Ensure wasm32 target is installed
rustup target add wasm32-unknown-unknown

# Check wasm-pack version
wasm-pack --version  # Should be 0.12+
```

**Desktop audio initialization fails:**
```bash
# Check audio device permissions
# Windows: Ensure audio services are running
# Linux: Check PulseAudio/JACK/ALSA configuration
# macOS: Grant microphone/audio permissions in System Preferences

# Test device enumeration
cargo run --bin rusty-audio -- --list-devices
```

**ASIO not working on Windows:**
```bash
# Verify ASIO SDK path in .cargo/config.toml
# Should point to: C:\path\to\ASIOSDK2.3.3

# Rebuild with ASIO feature (automatically enabled on Windows)
cargo clean
cargo build --release -p rusty-audio-desktop
```

## Web Audio API Submodule

**Location**: `web-audio-api-rs/` (git submodule)

**Important**: This is a git submodule, not a local path dependency.

```bash
# Update submodule to latest
cd web-audio-api-rs
git pull origin main
cd ..
git add web-audio-api-rs
git commit -m "Update web-audio-api submodule"

# Check submodule status
git submodule status

# Build submodule (optional, for testing)
cd web-audio-api-rs
cargo build --release
cargo test
```

**Integration**: Referenced in workspace `Cargo.toml` as:
```toml
web-audio-api = { version = "1.2.0", features = ["default"] }
```

## Platform-Specific Features

### Windows

- **ASIO Support**: Low-latency audio via `cpal` with `asio` feature
- **MMCSS Thread Priority**: Real-time audio thread scheduling via Windows API
- **Media Foundation**: Hardware-accelerated audio decoding

### Linux

- **ALSA/PulseAudio/JACK**: Automatic backend selection via `cpal`
- **Session Management**: Integration with desktop environment

### macOS

- **CoreAudio**: Low-latency audio via `cpal`
- **App Sandbox**: Security restrictions for file access

### WASM/Web

- **SharedArrayBuffer**: Required for multithreading (COOP/COEP headers)
- **Web Audio API**: Browser-native audio processing
- **Web Storage API**: Session and settings persistence
- **OAuth 2.0**: Authentication without backend server

## Performance Optimization

### Build Profiles

```toml
[profile.release]
opt-level = 3
lto = "fat"              # Link-time optimization
codegen-units = 1        # Better optimization
strip = true             # Remove debug symbols
panic = "abort"          # Smaller binary

[profile.wasm-release]
inherits = "release"
lto = false              # WASM doesn't benefit from LTO
```

### Real-Time Audio Optimizations

Enabled via `audio-optimizations` feature:
- Lock-free ring buffers (`rtrb`)
- Thread priority elevation (Windows MMCSS)
- Pre-allocated buffer pools
- SIMD-accelerated FFT (`rustfft` with `opt-simd`)

### WASM Bundle Optimization

```bash
# Build with optimizations
wasm-pack build --release

# Further optimize with wasm-opt
just wasm-optimize

# Check bundle size
just wasm-size

# Analyze bundle features
just wasm-analyze
```

## Security Considerations

### Audio Safety (`security::audio_safety`)

- **Maximum Volume Limit**: Prevents hearing damage
- **Sample Rate Validation**: Prevents buffer overruns
- **Buffer Size Constraints**: Prevents memory exhaustion

### Input Validation (`security::input_validator`)

- **File Path Sanitization**: Prevents directory traversal
- **Parameter Range Checks**: Prevents out-of-bounds access
- **User Input Escaping**: Prevents injection attacks

### WASM Security

- **Content Security Policy**: Restricts script execution
- **COOP/COEP Headers**: Enables SharedArrayBuffer safely
- **OAuth PKCE**: Prevents authorization code interception

## Documentation

### Generate API Docs

```bash
# All workspace members
cargo doc --workspace --no-deps --open

# Specific package
cargo doc -p rusty-audio-core --open

# With private items
cargo doc --workspace --document-private-items
```

### Additional Documentation

- `WORKSPACE_README.md` - Workspace architecture and migration guide
- `ASIO_INTEGRATION.md` - Windows ASIO setup and troubleshooting
- `DEPLOYMENT.md` - Deployment instructions for desktop and web
- `TESTING.md` - Comprehensive testing guide
- `PERFORMANCE_GUIDE.md` - Performance optimization techniques

## OAuth Authentication (Web Only)

### Supported Providers

1. **Google** - OAuth 2.0 with OpenID Connect
2. **GitHub** - OAuth 2.0
3. **Microsoft** - OAuth 2.0 with OpenID Connect

### Usage Example

```rust
use rusty_audio_web::auth::{OAuthClient, OAuthProvider};

// Initialize client
let mut client = OAuthClient::new(
    OAuthProvider::Google,
    "your-client-id".to_string(),
    "http://localhost:8080/callback".to_string(),
);

// Start login flow (generates PKCE challenge)
let auth_url = client.initiate_auth().await?;
// Redirect user to auth_url

// Handle callback with authorization code
let session = client.handle_callback(&code).await?;

// Use access token
if let Some(token) = session.access_token {
    // Make authenticated API requests
}
```

### Security Features

- **PKCE**: Proof Key for Code Exchange (RFC 7636)
- **State Parameter**: CSRF protection with random verification
- **Secure Storage**: Tokens encrypted in production builds
- **Automatic Refresh**: Token refresh before expiration

## CI/CD Integration

### GitHub Actions Workflow

```bash
# Simulate CI locally
just ci-local       # Full CI pipeline
just ci-fast        # Skip slow tests

# Pre-push checks
just pre-push       # Quality checks
just pre-pr         # Comprehensive checks including WASM
```

### Quality Gates

1. **Formatting**: `cargo fmt --check`
2. **Linting**: `cargo clippy -- -D warnings`
3. **Tests**: `cargo test --workspace`
4. **Documentation**: `cargo doc --no-deps`
5. **WASM Build**: `wasm-pack build --target web`
6. **Security Audit**: `cargo audit`

## Key Differences from Monolithic Architecture

**Before (monolithic)**:
```rust
use rusty_audio::audio::AudioBackend;
use rusty_audio::ui::Theme;
```

**After (workspace)**:
```rust
// In desktop app
use rusty_audio_core::audio::AudioBackend;
use rusty_audio_core::ui::Theme;

// In WASM app
use rusty_audio_core::prelude::*;
```

**Import Changes**:
- All shared code is now in `rusty_audio_core::*`
- Platform-specific code is in respective binaries
- Feature flags control platform-specific dependencies

## Documentation Guidelines

**CRITICAL: Prevent Documentation Clutter**

This project previously accumulated 128+ markdown files before cleanup. To prevent this from happening again:

### What NOT to Create

❌ **Session Summaries** - Use git commit messages instead
- No `SESSION_COMPLETION_SUMMARY.md`, `PHASE_N_COMPLETE.md`, etc.
- Progress belongs in git history, not documentation files

❌ **Completion Reports** - Use PR descriptions and git tags
- No `*_IMPLEMENTATION_COMPLETE.md`, `*_VERIFIED.md`, etc.
- Completion status belongs in GitHub Issues/Projects

❌ **Status Updates** - Use git commits and PR updates
- No `*_PROGRESS.md`, `*_STATUS.md`, `*_SUMMARY.md`
- Status tracking belongs in project management tools

❌ **Review/Audit Reports** - Use PR review comments
- No `CODE_REVIEW_REPORT.md`, `*_ANALYSIS.md`, `*_AUDIT_REPORT.md`
- Reviews belong in PR conversations

❌ **Duplicate Guides** - Update existing docs instead
- No variants: `*_GUIDE.md` + `*_QUICKSTART.md` + `*_REFERENCE.md` for same topic
- Consolidate into ONE canonical document

❌ **Temporary Artifacts** - Delete after use
- No `COMMIT_MESSAGE.md`, `pr_description.md`, `runtime_test.md`
- Use these for drafting, then delete them

### What TO Create (Sparingly)

✅ **Architecture Documentation** - Only if adding novel information
- Must explain HIGH-LEVEL design patterns across multiple modules
- Must not duplicate existing WORKSPACE_README.md or module docs
- Examples: `AUDIO_BACKEND_ARCHITECTURE.md`, `OAUTH_ARCHITECTURE.md`

✅ **Feature Documentation** - For complex implemented features
- Must document features that are IMPLEMENTED or actively being built
- Must provide unique value beyond inline code documentation
- Examples: `ASIO_INTEGRATION.md`, `WASM_MULTITHREADING_GUIDE.md`

✅ **Testing/Deployment Guides** - For complex procedures
- Must document non-obvious multi-step processes
- Must be actively maintained when processes change
- Examples: `TESTING.md`, `DEPLOYMENT.md`, `BROWSER_TESTING_GUIDE.md`

✅ **Design Artifacts** - For UI/UX reference
- Must contain visual mockups, diagrams, or design rationale
- Examples: `UI_MOCKUPS_AND_FLOWS.md`, `WIREFRAMES_AND_INTERACTION_PATTERNS.md`

### Documentation Maintenance Rules

1. **Update, Don't Duplicate**: If a doc exists, update it. Never create `*_v2.md` or `*_enhanced.md`
2. **Delete Outdated Docs**: When info is superseded, delete the old file completely
3. **One Source of Truth**: Each topic should have ONE canonical document
4. **Link, Don't Copy**: Reference other docs instead of duplicating information
5. **Index in WORKSPACE_README**: All kept docs should be referenced in the main README

### Before Creating a New .md File, Ask:

1. **Does this information belong in existing docs?** (Check WORKSPACE_README.md, DEVELOPER_GUIDE.md, CLAUDE.md first)
2. **Will this be maintained?** (If not, use git commit messages or PR descriptions)
3. **Is this a temporary artifact?** (Draft it, use it, then delete it)
4. **Does this add unique value?** (Novel architecture, complex features, design artifacts)

### Current Documentation Structure (42 files)

**Core Docs** (4): WORKSPACE_README.md, CLAUDE.md, DEVELOPER_GUIDE.md, USER_MANUAL.md
**Architecture** (5): WORKSPACE_ARCHITECTURE.md, AUDIO_BACKEND_ARCHITECTURE.md, etc.
**Features** (8): ASIO_INTEGRATION.md, OAUTH_ARCHITECTURE.md, WASM_MULTITHREADING_GUIDE.md, etc.
**Build/Deploy** (6): BUILD_WINDOWS.md, JUSTFILE_GUIDE.md, DEPLOYMENT.md, etc.
**Testing** (4): TESTING.md, TESTING_FRAMEWORK_README.md, BROWSER_TESTING_GUIDE.md, etc.
**Security** (3): SECURITY_IMPLEMENTATION_GUIDE.md, SECURITY_THREAT_MODEL.md, etc.
**Reference** (12): SIGNAL_GENERATOR_GUIDE.md, TROUBLESHOOTING_GUIDE.md, API_SPECIFICATION.md, etc.

**Goal**: Keep this count under 50 files. Anything above 50 indicates documentation debt.

## Quick Reference

```bash
# Development cycle
just check && just test && just run

# Full quality check
just quality

# WASM development
just serve-wasm

# Benchmarks
just bench

# Documentation
cargo doc --workspace --open

# Clean everything
just clean-all
```
