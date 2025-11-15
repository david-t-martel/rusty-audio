# Windows Build Instructions for Rusty Audio

## Quick Start (Recommended)

Open **PowerShell** on Windows (not WSL) and run:

```powershell
cd C:\Users\david\rusty-audio

# Option 1: Using justfile (recommended)
just build-release

# Option 2: Direct cargo command
cargo build --release
```

The compiled application will be at:
```
C:\Users\david\rusty-audio\target\release\rusty-audio_native.exe
```

## If ASIO SDK Error Occurs

If you see an error about ASIO SDK or PowerShell, you have two options:

### Option A: Install ASIO SDK (for professional audio)

1. Download ASIO SDK from: https://www.steinberg.net/asiosdk
2. Extract to: `C:\ASIOSDK`
3. Set environment variable in PowerShell:
   ```powershell
   $env:CPAL_ASIO_DIR = "C:\ASIOSDK"
   ```
4. Build again: `just build-release`

### Option B: Disable ASIO temporarily (simpler, for testing)

Edit `Cargo.toml` line 207-208:

**Before:**
```toml
[target.'cfg(target_os = "windows")'.dependencies]
cpal = { version = "0.15", features = ["asio"] }
```

**After:**
```toml
[target.'cfg(target_os = "windows")'.dependencies]
cpal = { version = "0.15" }
```

Then build normally: `just build-release`

## Running the Application

After successful build:

```powershell
# From project directory
.\target\release\rusty-audio_native.exe

# Or double-click the .exe in Windows Explorer
```

## Testing PR #4 Fixes

The latest build includes critical audio system improvements:
- ✅ Fixed OutputDeviceDestination (now actually outputs to speakers)
- ✅ Fixed InputDeviceSource (microphone input now works)
- ✅ Fixed audio router N×M fan-out
- ✅ Volume slider now controls actual audio
- ✅ Fixed device enumeration

Test by:
1. Opening the application
2. Going to "Signal Generator" tab
3. Adjusting volume slider - should control audio output
4. Testing audio device enumeration

## Available Just Commands

View all available commands:
```powershell
just --list
```

Key commands:
- `just build` - Debug build (faster)
- `just build-release` - Release build (optimized)
- `just run-release` - Build and run release version
- `just test` - Run test suite
- `just quality` - Run all quality checks

## Troubleshooting

**Error: "just: command not found"**
Install just:
```powershell
cargo install just
```

**Error: "Cannot find rustc"**
Rust toolchain not in PATH. Restart PowerShell or run:
```powershell
refreshenv  # If using chocolatey
```

**Build takes too long**
Use debug build for faster iteration:
```powershell
just build
just run
```

## Current Status

- **Branch**: main
- **Last Commit**: 269a5f5 (PR #4 audio fixes)
- **Build Target**: Windows x86_64 (MSVC)
- **Expected Binary Size**: 20-25 MB
- **Build Time**: ~5-10 minutes (first build), ~1-2 minutes (incremental)
