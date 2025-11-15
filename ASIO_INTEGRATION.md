# ASIO SDK Integration Guide

## Overview

This document explains how to build Rusty Audio with ASIO support for professional audio interfaces.

## Quick Start

### Option 1: Using Project Configuration (Recommended)

The project is pre-configured to use the ASIO SDK at `T:\projects\asiovst\asiosdk`.

Simply build from PowerShell:

```powershell
cd C:\Users\david\rusty-audio
just build-release
```

The `.cargo/config.toml` file automatically sets `CPAL_ASIO_DIR` to the correct location.

### Option 2: Manual Environment Variable

If you prefer to set the environment variable manually:

```powershell
# Set permanently (recommended)
[System.Environment]::SetEnvironmentVariable(
    "CPAL_ASIO_DIR",
    "T:\projects\asiovst\asiosdk",
    [System.EnvironmentVariableTarget]::User
)

# Restart PowerShell, then build
cargo build --release
```

## ASIO SDK Structure

The SDK at `T:\projects\asiovst\asiosdk` contains:

```
asiosdk/
├── common/                    # Core ASIO implementation
│   ├── asio.h                # Main ASIO header (required)
│   ├── asiosys.h             # System header (required)
│   ├── asiodrvr.h            # Driver interface (required)
│   └── *.cpp                 # Implementation files
│
├── host/                      # Host application support
│   ├── asiodrivers.h         # Driver management (required)
│   └── pc/asiolist.*         # Windows COM driver loading
│
└── driver/                    # Sample driver (not used in build)
```

**Status**: ✅ All required files verified and compatible with asio-sys v0.2.2

## Build Requirements

### Required Tools

1. **Rust Toolchain**
   - Version: 1.70+ (you have 1.90.0 ✅)
   - Target: `x86_64-pc-windows-msvc`

2. **Visual Studio Build Tools**
   - MSVC compiler (cl.exe)
   - Windows SDK
   - Download: https://visualstudio.microsoft.com/downloads/

3. **LLVM/Clang** (for bindgen)
   - Download: https://releases.llvm.org/
   - Or via Chocolatey: `choco install llvm`

### Verify Tools

```powershell
# Check Rust
rustc --version
cargo --version

# Check MSVC
cl.exe /?

# Check Clang (for bindgen)
clang --version

# Check just (task runner)
just --version
```

## Building

### Development Build

```powershell
cd C:\Users\david\rusty-audio

# Fast check (no linking)
just check

# Full debug build
just build
```

### Release Build (Optimized)

```powershell
# Standard release
just build-release

# With ASIO explicitly enabled
just build-windows-asio

# Run release version
just run-release
```

### Build Time

- **First build**: ~8-10 minutes
  - Compiling ASIO SDK C++ sources
  - Generating Rust bindings with bindgen
  - Building all dependencies

- **Incremental rebuilds**: ~1-2 minutes
  - Only changed files recompiled
  - Cached dependencies reused

## Verifying ASIO Integration

### Build Verification

After successful build, check the log for:

```
Compiling asio-sys v0.2.2
CPAL_ASIO_DIR is set at T:\projects\asiovst\asiosdk
...building bindings from ASIO SDK headers
...compiling ASIO SDK sources
   Compiling cpal v0.15
   Compiling rusty-audio v0.1.0
```

### Runtime Verification

```powershell
# Run the application
.\target\release\rusty-audio_native.exe

# In the application:
# 1. Go to Settings tab
# 2. Look for "ASIO" in the audio backend dropdown
# 3. Select ASIO backend
# 4. ASIO devices should appear in device list
```

### Testing with Code

Add this test to verify ASIO:

```rust
#[test]
#[cfg(target_os = "windows")]
fn test_asio_available() {
    use rusty_audio::audio::asio_backend::AsioBackend;

    println!("ASIO available: {}", AsioBackend::asio_available());

    if AsioBackend::asio_available() {
        let backends = AsioBackend::available_backends();
        println!("Available backends: {:?}", backends);
    }
}
```

Run with:
```powershell
cargo test test_asio -- --nocapture
```

## ASIO Driver Requirements

### Important Notes

**Building** with ASIO support only requires the SDK.

**Running** with ASIO requires:

1. **ASIO-compatible audio interface** (physical hardware)
   - Examples: Focusrite Scarlett, RME Fireface, MOTU, Universal Audio
   - Consumer sound cards don't support ASIO

2. **ASIO driver installed** for your device
   - Download from manufacturer's website
   - Install before running the application

3. **Admin privileges** (sometimes required)
   - Right-click executable → "Run as administrator"

### Without ASIO Hardware

If you don't have ASIO hardware:
- ✅ Application will still **build** successfully
- ✅ `AsioBackend::asio_available()` returns `false`
- ✅ Application **falls back to WASAPI** automatically
- ✅ All audio functionality works normally with standard devices

### ASIO4ALL (Generic ASIO Driver)

If you want to test ASIO without professional hardware:

1. Download ASIO4ALL: https://www.asio4all.org/
2. Install (may require disabling driver signature enforcement)
3. Provides ASIO interface for standard sound cards
4. Lower performance than real ASIO hardware but good for testing

## Troubleshooting

### "Failed to download ASIO SDK"

**Symptom**: Build tries to download SDK from Steinberg

**Solution**:
```powershell
# Verify CPAL_ASIO_DIR is set
Get-Content .cargo/config.toml | Select-String CPAL_ASIO_DIR

# Should show: CPAL_ASIO_DIR = { value = "T:\\projects\\asiovst\\asiosdk" ...

# Clean and rebuild
cargo clean
cargo build --release
```

### "Could not find vcvarsall.bat"

**Symptom**: MSVC compiler not found

**Solution**:
1. Install Visual Studio Build Tools
2. Or add MSVC to PATH manually

### "Unable to find libclang"

**Symptom**: bindgen cannot find clang

**Solution**:
```powershell
# Install LLVM
choco install llvm

# Add to PATH
$env:PATH += ";C:\Program Files\LLVM\bin"

# Set LIBCLANG_PATH
$env:LIBCLANG_PATH = "C:\Program Files\LLVM\bin"
```

### Build Hangs or Crashes

**Solution**:
```powershell
# Reduce parallel jobs (if low RAM)
# Edit .cargo/config.toml: jobs = 4

# Or build single-threaded
cargo build --release -j 1
```

## Performance Characteristics

### ASIO Benefits

- **Ultra-low latency**: <5ms round-trip possible
- **Exclusive mode**: Direct hardware access
- **Professional quality**: Bit-perfect audio
- **Multi-channel**: Support for 8+ channels

### ASIO vs WASAPI

| Feature | ASIO | WASAPI Exclusive | WASAPI Shared |
|---------|------|------------------|---------------|
| Latency | 2-10ms | 10-20ms | 20-50ms |
| Exclusivity | Yes | Yes | No |
| Pro Hardware | Required | Optional | No |
| Multichannel | Excellent | Good | Limited |

## Advanced Configuration

### Custom ASIO SDK Location

If your SDK is at a different location:

Edit `.cargo/config.toml`:
```toml
[env]
CPAL_ASIO_DIR = { value = "C:\\path\\to\\your\\asiosdk", force = true }
```

### Disable ASIO (Fallback to WASAPI)

Edit `Cargo.toml` line 207:
```toml
# Remove ASIO feature
cpal = { version = "0.15" }  # No features = WASAPI only
```

### Build-time ASIO Verification

Create `build.rs` in project root:
```rust
fn main() {
    #[cfg(target_os = "windows")]
    {
        if let Ok(path) = std::env::var("CPAL_ASIO_DIR") {
            println!("cargo:warning=Using ASIO SDK at: {}", path);
        } else {
            println!("cargo:warning=CPAL_ASIO_DIR not set - ASIO support may fail");
        }
    }
}
```

## References

- **ASIO SDK**: Steinberg's official Audio Stream Input/Output SDK
- **asio-sys**: Rust bindings crate (via cpal dependency)
- **cpal**: Cross-platform audio I/O library for Rust
- **Rusty Audio ASIO Backend**: `src/audio/asio_backend.rs`

## Quick Reference

```powershell
# Environment check
echo $env:CPAL_ASIO_DIR
# Should output: T:\projects\asiovst\asiosdk

# Quick build
just build-release

# Test ASIO
cargo test test_asio -- --nocapture

# Run application
.\target\release\rusty-audio_native.exe
```

---

**Status**: ✅ SDK verified and configured
**Next Step**: Run `just build-release` from Windows PowerShell
