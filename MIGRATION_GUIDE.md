# Migration Guide: Monolithic to Workspace Architecture

## Overview

This guide provides step-by-step instructions for migrating the Rusty Audio codebase from its current monolithic structure to the new workspace-based architecture with separated desktop and web applications.

**Estimated Time:** 8-12 hours

**Risk Level:** Medium (comprehensive testing required)

---

## Table of Contents

1. [Pre-Migration Checklist](#1-pre-migration-checklist)
2. [Phase 1: Workspace Setup](#2-phase-1-workspace-setup)
3. [Phase 2: Extract Core Library](#3-phase-2-extract-core-library)
4. [Phase 3: Create Desktop Crate](#4-phase-3-create-desktop-crate)
5. [Phase 4: Create Web Crate](#5-phase-4-create-web-crate)
6. [Phase 5: Implement OAuth](#6-phase-5-implement-oauth)
7. [Phase 6: Update CI/CD](#7-phase-6-update-cicd)
8. [Post-Migration Validation](#8-post-migration-validation)
9. [Rollback Plan](#9-rollback-plan)
10. [Troubleshooting](#10-troubleshooting)

---

## 1. Pre-Migration Checklist

### 1.1 Backup Current State

```bash
# Create backup branch
git checkout -b pre-workspace-migration
git push origin pre-workspace-migration

# Tag current state
git tag v0.1.0-monolithic
git push origin v0.1.0-monolithic

# Create archive backup
tar -czf rusty-audio-backup-$(date +%Y%m%d).tar.gz .
```

### 1.2 Verify Current State

```bash
# Ensure all tests pass
cargo test --all
cargo clippy --all -- -D warnings
cargo fmt --all -- --check

# Verify desktop build
cargo build --release
cargo run --release

# Verify WASM build
wasm-pack build --target web
```

### 1.3 Document Current Dependencies

```bash
# List current dependencies
cargo tree --depth 1 > dependencies-before.txt
cargo tree --duplicates > duplicates-before.txt
```

### 1.4 Required Tools

```bash
# Install required tools
cargo install cargo-workspaces
cargo install wasm-pack
cargo install cargo-watch
```

---

## 2. Phase 1: Workspace Setup

### 2.1 Create Workspace Root

**Duration:** 30 minutes

```bash
# Backup current Cargo.toml
cp Cargo.toml Cargo.toml.backup

# Create workspace root Cargo.toml
cat > Cargo.toml << 'EOF'
[workspace]
resolver = "2"
members = [
    "rusty-audio-core",
    "rusty-audio-desktop",
    "rusty-audio-web",
]

[workspace.package]
version = "0.2.0"
edition = "2021"
license = "MIT OR Apache-2.0"
authors = ["Your Name <your.email@example.com>"]

[workspace.dependencies]
# Shared dependencies (from WORKSPACE_ARCHITECTURE.md)
egui = "0.33.0"
eframe = { version = "0.33.0", default-features = false }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
thiserror = "1.0"
log = "0.4"
tracing = "0.1"
parking_lot = "0.12.3"
cfg-if = "1.0"
rustfft = "6.0"
num-complex = "0.4.4"
realfft = "3.3"
rayon = "1.10"
lru = "0.12"
ndarray = "0.15"
chrono = "0.4"
uuid = { version = "1.0", features = ["v4", "serde"] }

[workspace.lints.rust]
unsafe_code = "warn"

[workspace.lints.clippy]
unwrap_used = "warn"
expect_used = "warn"
panic = "warn"
indexing_slicing = "warn"
EOF
```

### 2.2 Create Crate Directories

```bash
# Create directory structure
mkdir -p rusty-audio-core/src
mkdir -p rusty-audio-desktop/src
mkdir -p rusty-audio-web/src
mkdir -p rusty-audio-web/static

# Create placeholder Cargo.toml files
touch rusty-audio-core/Cargo.toml
touch rusty-audio-desktop/Cargo.toml
touch rusty-audio-web/Cargo.toml
```

### 2.3 Verify Workspace Structure

```bash
# Check workspace members
cargo metadata --format-version 1 | jq '.workspace_members'

# This should list 3 crates
```

---

## 3. Phase 2: Extract Core Library

### 3.1 Create Core Cargo.toml

**Duration:** 1 hour

```bash
cat > rusty-audio-core/Cargo.toml << 'EOF'
[package]
name = "rusty-audio-core"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
egui.workspace = true
serde.workspace = true
serde_json.workspace = true
anyhow.workspace = true
thiserror.workspace = true
log.workspace = true
tracing.workspace = true
parking_lot.workspace = true
rustfft.workspace = true
num-complex.workspace = true
realfft.workspace = true
rayon.workspace = true
lru.workspace = true

# Core-specific dependencies
approx = "0.5.1"
statrs = "0.16"
rand = "0.8"

[features]
default = []
ai = ["ndarray"]
advanced-dsp = ["realfft"]

[dependencies.ndarray]
version = "0.15"
optional = true
EOF
```

### 3.2 Move Shared Modules to Core

```bash
# Audio modules (shared DSP, no backend implementations)
mkdir -p rusty-audio-core/src/audio/dsp
cp src/audio/backend.rs rusty-audio-core/src/audio/backend.rs
cp src/signal_generators.rs rusty-audio-core/src/audio/dsp/signal_generators.rs

# UI modules (all shared components)
mkdir -p rusty-audio-core/src/ui
cp src/ui/components.rs rusty-audio-core/src/ui/
cp src/ui/controls.rs rusty-audio-core/src/ui/
cp src/ui/spectrum.rs rusty-audio-core/src/ui/
cp src/ui/theme.rs rusty-audio-core/src/ui/
cp src/ui/layout.rs rusty-audio-core/src/ui/
cp src/ui/accessibility.rs rusty-audio-core/src/ui/
cp src/ui/enhanced_button.rs rusty-audio-core/src/ui/
cp src/ui/enhanced_controls.rs rusty-audio-core/src/ui/

# Security modules
mkdir -p rusty-audio-core/src/security
cp -r src/security/* rusty-audio-core/src/security/

# AI modules (optional feature)
mkdir -p rusty-audio-core/src/ai
cp -r src/ai/* rusty-audio-core/src/ai/

# Metadata and error handling
cp src/metadata.rs rusty-audio-core/src/
cp src/error.rs rusty-audio-core/src/
```

### 3.3 Create Core Library Entry Point

```bash
cat > rusty-audio-core/src/lib.rs << 'EOF'
//! Rusty Audio Core Library
//!
//! Shared audio engine, DSP algorithms, and UI components for
//! both desktop and web applications.

pub mod audio;
pub mod ui;
pub mod security;
pub mod metadata;
pub mod error;

#[cfg(feature = "ai")]
pub mod ai;

/// Convenience re-exports
pub mod prelude {
    pub use crate::audio::backend::{AudioBackend, AudioConfig, PlaybackState};
    pub use crate::error::{AudioError, Result};
    pub use crate::ui::theme::{Theme, ThemeManager};
}
EOF
```

### 3.4 Update Module Declarations

Edit each module to expose public API:

```bash
cat > rusty-audio-core/src/audio/mod.rs << 'EOF'
pub mod backend;
pub mod dsp;

pub use backend::{AudioBackend, AudioConfig, BackendHealth, EqBand, PlaybackState};

#[cfg(not(target_arch = "wasm32"))]
pub use backend::AudioDevice;
EOF
```

```bash
cat > rusty-audio-core/src/ui/mod.rs << 'EOF'
pub mod components;
pub mod controls;
pub mod spectrum;
pub mod theme;
pub mod layout;
pub mod accessibility;
pub mod enhanced_button;
pub mod enhanced_controls;

pub use theme::{Theme, ThemeManager};
pub use components::{AlbumArtDisplay, MetadataDisplay, ProgressBar};
pub use controls::{CircularKnob, EnhancedButton};
EOF
```

### 3.5 Fix Import Paths in Core

**Search and replace pattern:**
```bash
# Replace absolute crate paths with relative paths
find rusty-audio-core/src -name "*.rs" -exec sed -i 's/use rusty_audio::/use crate::/g' {} \;

# Replace `use crate::` with `use super::` where appropriate
# (manual review recommended)
```

### 3.6 Test Core Library

```bash
cd rusty-audio-core
cargo build
cargo test
cargo clippy
cd ..
```

**Expected Warnings:**
- Some unused imports (will be used by desktop/web)
- Missing implementations (backends will implement traits)

---

## 4. Phase 3: Create Desktop Crate

### 4.1 Create Desktop Cargo.toml

**Duration:** 1 hour

```bash
cat > rusty-audio-desktop/Cargo.toml << 'EOF'
[package]
name = "rusty-audio-desktop"
version.workspace = true
edition.workspace = true
license.workspace = true

[[bin]]
name = "rusty-audio"
path = "src/main.rs"

[dependencies]
rusty-audio-core = { path = "../rusty-audio-core", features = ["ai", "advanced-dsp"] }

# Workspace dependencies
eframe = { workspace = true, features = ["wgpu"] }
egui.workspace = true
serde.workspace = true
serde_json.workspace = true
anyhow.workspace = true
thiserror.workspace = true
log.workspace = true
parking_lot.workspace = true

# Desktop-specific dependencies
tokio = { version = "1.0", features = ["full"] }
rfd = "0.14.1"
lofty = "0.22.4"
image = "0.25.8"

# Audio dependencies (native only)
cpal = "0.15"
rodio = { version = "0.17", default-features = false, features = ["wav", "vorbis", "flac", "mp3"] }
symphonia = { version = "0.5", features = ["all", "opt-simd"] }
rubato = "0.15"
midir = "0.9"
wmidi = "4.0"
hound = "3.5"

[target.'cfg(target_os = "windows")'.dependencies]
windows = { version = "0.58", features = [
    "Win32_System_Threading",
    "Win32_Media_Audio",
] }
cpal = { version = "0.15", features = ["asio"] }

[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
strip = true
panic = "abort"
EOF
```

### 4.2 Move Desktop-Specific Code

```bash
# Main application
cp src/main.rs rusty-audio-desktop/src/

# Desktop audio backends
mkdir -p rusty-audio-desktop/src/audio
cp src/audio/device.rs rusty-audio-desktop/src/audio/
cp src/audio/manager.rs rusty-audio-desktop/src/audio/
cp src/audio/recorder.rs rusty-audio-desktop/src/audio/
cp src/audio/asio_backend.rs rusty-audio-desktop/src/audio/

# Platform-specific code
mkdir -p rusty-audio-desktop/src/platform
cp src/audio/mmcss.rs rusty-audio-desktop/src/platform/windows.rs

# Desktop UI components
mkdir -p rusty-audio-desktop/src/ui
# (Desktop-specific panels if any)
```

### 4.3 Update Desktop Imports

Edit `rusty-audio-desktop/src/main.rs`:

```rust
// Replace:
// use rusty_audio::ui::*;
// use rusty_audio::audio::*;

// With:
use rusty_audio_core::prelude::*;
use rusty_audio_core::ui::*;

// Desktop-specific modules
mod audio;
mod platform;

use audio::{CpalBackend, AsioBackend};
```

### 4.4 Create Desktop Module Structure

```bash
cat > rusty-audio-desktop/src/audio/mod.rs << 'EOF'
pub mod device;
pub mod manager;
pub mod recorder;

#[cfg(target_os = "windows")]
pub mod asio_backend;

pub use device::CpalBackend;
pub use manager::AudioDeviceManager;
pub use recorder::AudioRecorder;

#[cfg(target_os = "windows")]
pub use asio_backend::AsioBackend;
EOF
```

### 4.5 Test Desktop Build

```bash
cd rusty-audio-desktop
cargo build
cargo test
cargo run

# Verify UI loads and audio playback works
cd ..
```

---

## 5. Phase 4: Create Web Crate

### 5.1 Create Web Cargo.toml

**Duration:** 1 hour

```bash
cat > rusty-audio-web/Cargo.toml << 'EOF'
[package]
name = "rusty-audio-web"
version.workspace = true
edition.workspace = true
license.workspace = true

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
rusty-audio-core = { path = "../rusty-audio-core" }

# Workspace dependencies
egui.workspace = true
serde.workspace = true
serde_json.workspace = true
anyhow.workspace = true
thiserror.workspace = true
log.workspace = true
parking_lot.workspace = true

# WASM dependencies
wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"
js-sys = "0.3"
web-sys = { version = "0.3", features = [
    "AudioContext",
    "AudioDestinationNode",
    "Window",
    "Document",
    "Storage",
] }
getrandom = { version = "0.2", features = ["js"] }
console_error_panic_hook = "0.1"
console_log = "1"

# HTTP client for WASM
gloo-net = "0.5"
gloo-storage = "0.3"
gloo-utils = "0.2"

# Cryptography
sha2 = "0.10"
base64 = "0.21"
rand = "0.8"

# Database (IndexedDB wrapper)
rexie = "0.5"
serde-wasm-bindgen = "0.6"

[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
panic = "abort"
strip = true
EOF
```

### 5.2 Move Web-Specific Code

```bash
# WASM entry point
cp src/web.rs rusty-audio-web/src/lib.rs

# Web audio backend
mkdir -p rusty-audio-web/src/audio
cp src/audio/web_audio_backend.rs rusty-audio-web/src/audio/
cp src/audio/wasm_processing.rs rusty-audio-web/src/audio/

# Web panic handler
cp src/wasm_panic_handler.rs rusty-audio-web/src/

# Create static assets
mkdir -p rusty-audio-web/static
cat > rusty-audio-web/static/index.html << 'EOF'
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Rusty Audio</title>
    <link rel="stylesheet" href="styles.css">
</head>
<body>
    <script type="module">
        import init from './pkg/rusty_audio_web.js';
        init();
    </script>
</body>
</html>
EOF
```

### 5.3 Update Web Imports

Edit `rusty-audio-web/src/lib.rs`:

```rust
use rusty_audio_core::prelude::*;
use rusty_audio_core::ui::*;

mod audio;
mod auth;  // OAuth module (Phase 5)
mod storage;

pub use audio::WebAudioBackend;
```

### 5.4 Test WASM Build

```bash
cd rusty-audio-web
wasm-pack build --target web --dev

# Serve locally
python3 -m http.server 8080 --directory static
# Open http://localhost:8080

cd ..
```

---

## 6. Phase 5: Implement OAuth

**Duration:** 3-4 hours

See `OAUTH_ARCHITECTURE.md` for detailed implementation.

### 6.1 Create Auth Module Structure

```bash
mkdir -p rusty-audio-web/src/auth
touch rusty-audio-web/src/auth/mod.rs
touch rusty-audio-web/src/auth/oauth.rs
touch rusty-audio-web/src/auth/pkce.rs
touch rusty-audio-web/src/auth/storage.rs
touch rusty-audio-web/src/auth/providers.rs
```

### 6.2 Create Storage Module

```bash
mkdir -p rusty-audio-web/src/storage
touch rusty-audio-web/src/storage/mod.rs
touch rusty-audio-web/src/storage/indexed_db.rs
touch rusty-audio-web/src/storage/encryption.rs
```

### 6.3 Implement OAuth Flow

Copy implementations from `OAUTH_ARCHITECTURE.md`:
- Section 3.1: PKCE implementation
- Section 3.2: OAuth client
- Section 4.1: IndexedDB storage
- Section 4.2: Encryption

### 6.4 Add Auth UI

```bash
mkdir -p rusty-audio-web/src/ui
touch rusty-audio-web/src/ui/login.rs
touch rusty-audio-web/src/ui/main_app.rs
```

### 6.5 Test OAuth Flow

**Prerequisites:**
- Register OAuth applications with Google/GitHub
- Set environment variables for client IDs

```bash
# Build with OAuth
cd rusty-audio-web
GOOGLE_CLIENT_ID="your-client-id" wasm-pack build --target web

# Test in browser
# Navigate to login page, test authentication
```

---

## 7. Phase 6: Update CI/CD

**Duration:** 1-2 hours

### 7.1 Update GitHub Actions

Create separate workflows for desktop and web:

```bash
mkdir -p .github/workflows
touch .github/workflows/desktop-ci.yml
touch .github/workflows/web-ci.yml
```

Copy workflow configurations from `DEPLOYMENT_ARCHITECTURE.md` Section 4.

### 7.2 Update Pre-commit Hooks

```bash
# .pre-commit-config.yaml
repos:
  - repo: local
    hooks:
      - id: cargo-fmt
        name: cargo fmt (workspace)
        entry: cargo fmt
        args: [--all, --check]
        language: system
        pass_filenames: false

      - id: cargo-clippy
        name: cargo clippy (workspace)
        entry: cargo clippy
        args: [--workspace, --, -D, warnings]
        language: system
        pass_filenames: false
```

### 7.3 Test CI Locally

```bash
# Install act (GitHub Actions local runner)
# https://github.com/nektos/act

act -j test-desktop
act -j test-web
```

---

## 8. Post-Migration Validation

### 8.1 Functional Testing

**Desktop:**
```bash
cd rusty-audio-desktop
cargo build --release
./target/release/rusty-audio

# Test checklist:
# - [ ] Application launches
# - [ ] File dialog opens
# - [ ] Audio playback works
# - [ ] Spectrum visualizer displays
# - [ ] EQ adjustments work
# - [ ] Recording functionality
# - [ ] Settings persist
```

**Web:**
```bash
cd rusty-audio-web
wasm-pack build --release --target web

# Serve and test:
# - [ ] Application loads in browser
# - [ ] OAuth login works (all providers)
# - [ ] Audio playback works
# - [ ] Presets save/load
# - [ ] Session persists across reloads
# - [ ] Logout clears session
```

### 8.2 Performance Testing

**Desktop:**
```bash
# Benchmark suite
cd rusty-audio-desktop
cargo bench

# Compare with baseline
cargo-flamegraph --bench audio_benchmarks
```

**Web:**
```bash
# WASM size check
ls -lh rusty-audio-web/pkg/*.wasm

# Expected: < 3 MB after wasm-opt
wasm-opt -Oz rusty_audio_web_bg.wasm -o optimized.wasm
ls -lh optimized.wasm
```

### 8.3 Dependency Audit

```bash
# Check for duplicate dependencies
cargo tree --workspace --duplicates

# Security audit
cargo audit

# License compliance
cargo-license --workspace
```

### 8.4 Documentation Review

```bash
# Generate docs for all crates
cargo doc --workspace --no-deps --open

# Verify all public APIs are documented
cargo doc --workspace --no-deps 2>&1 | grep warning
```

---

## 9. Rollback Plan

### 9.1 If Migration Fails Midway

```bash
# Restore from backup branch
git checkout pre-workspace-migration
git reset --hard origin/pre-workspace-migration

# Restore archived backup
tar -xzf rusty-audio-backup-YYYYMMDD.tar.gz

# Resume development
cargo build
cargo test
```

### 9.2 If Issues Found After Migration

```bash
# Create hotfix branch
git checkout -b hotfix-workspace-issues

# Option 1: Fix issues incrementally
# Make targeted fixes, test, commit

# Option 2: Temporary revert
git revert <migration-commit-hash>
git push origin main

# Fix issues on feature branch
git checkout -b fix-workspace-migration
# Make fixes
git push origin fix-workspace-migration
# Create PR for review
```

---

## 10. Troubleshooting

### 10.1 Common Issues

#### Issue: Circular Dependencies

**Symptom:**
```
error[E0369]: circular dependency detected
```

**Solution:**
```bash
# Review dependency graph
cargo tree --workspace --edges features

# Break circular dependency by:
# 1. Moving shared code to core
# 2. Using trait objects instead of concrete types
# 3. Feature-gating problematic dependencies
```

#### Issue: Missing Symbols in WASM

**Symptom:**
```
WebAssembly.LinkError: Import #0 module="env" function="foo" error: unknown import
```

**Solution:**
```toml
# Add missing web-sys features
[dependencies.web-sys]
features = [
    "AudioContext",
    # Add missing feature here
]
```

#### Issue: Tests Fail After Migration

**Symptom:**
```
error: could not compile `rusty-audio-core` due to 5 previous errors
```

**Solution:**
```bash
# Review test imports
find . -name "*.rs" -path "*/tests/*" -exec grep "use rusty_audio::" {} \;

# Update imports
sed -i 's/use rusty_audio::/use rusty_audio_core::/g' tests/*.rs

# Re-run tests
cargo test --workspace
```

### 10.2 Performance Regressions

If performance degrades after migration:

```bash
# Compare benchmarks before/after
cargo bench --workspace > bench-after.txt
diff bench-before.txt bench-after.txt

# Profile release build
cargo build --release --workspace
perf record -g ./target/release/rusty-audio
perf report
```

### 10.3 WASM Build Failures

```bash
# Clear wasm-pack cache
rm -rf rusty-audio-web/pkg
rm -rf rusty-audio-web/target

# Rebuild with verbose output
wasm-pack build --target web --dev --verbose

# Check wasm-bindgen version compatibility
wasm-bindgen --version
cargo tree -p wasm-bindgen
```

---

## 11. Migration Timeline

### Week 1: Preparation and Core Extraction
- **Day 1-2:** Backup, workspace setup, core library scaffolding
- **Day 3-4:** Move shared modules to core, fix imports
- **Day 5:** Test core library independently

### Week 2: Desktop and Web Separation
- **Day 1-2:** Create desktop crate, test thoroughly
- **Day 3-4:** Create web crate, WASM build working
- **Day 5:** Integration testing

### Week 3: OAuth and Deployment
- **Day 1-3:** Implement OAuth 2.0 flow
- **Day 4:** Update CI/CD pipelines
- **Day 5:** Staging deployment, final testing

### Week 4: Production Launch
- **Day 1-2:** Production deployment
- **Day 3-5:** Monitoring, hotfixes, documentation

---

## 12. Success Criteria

Migration is complete when:

- [ ] Workspace structure matches `WORKSPACE_ARCHITECTURE.md`
- [ ] All tests pass in all three crates
- [ ] Desktop build produces working native binary
- [ ] Web build produces working WASM bundle < 3 MB
- [ ] OAuth flow works with all three providers
- [ ] CI/CD pipelines pass
- [ ] Documentation is up-to-date
- [ ] Performance benchmarks are within 5% of baseline
- [ ] No duplicate dependencies (cargo tree --duplicates is empty)
- [ ] Security audit passes (cargo audit)

---

## 13. Post-Migration Cleanup

After successful migration:

```bash
# Remove old monolithic files
git rm src/web.rs
git rm src/web_refactored.rs
# etc.

# Update README
# Update CHANGELOG
# Tag new version
git tag v0.2.0-workspace
git push origin v0.2.0-workspace

# Archive backup branch
git branch -D pre-workspace-migration
git push origin --delete pre-workspace-migration
```

---

## Conclusion

This migration transforms Rusty Audio from a monolithic codebase into a clean, modular workspace architecture with:
- **Zero code duplication** via shared core library
- **Clean separation** of desktop and web applications
- **Scalable architecture** ready for future features
- **Production-ready OAuth 2.0** for web application

Follow this guide carefully, test thoroughly at each phase, and you'll have a robust foundation for future development.
