# ASIO Integration Quick Start

## ✅ Pre-Build Checklist

Before building on Windows PowerShell, verify:

1. **ASIO SDK Location**:
   ```powershell
   Test-Path "T:\projects\asiovst\asiosdk\common\asio.h"
   # Should return: True
   ```

2. **Rust Toolchain**:
   ```powershell
   rustc --version
   cargo --version
   # Verify: 1.70+ (you have 1.90.0 ✅)
   ```

3. **Build Tools Installed**:
   ```powershell
   cl.exe /?           # MSVC compiler
   clang --version     # LLVM/Clang for bindgen
   just --version      # Just task runner
   ```

## 🔨 Building

From Windows PowerShell (NOT WSL):

```powershell
cd C:\Users\david\rusty-audio

# Option 1: Using just (recommended)
just build-release
just build-windows-asio  # Equivalent, shows ASIO SDK path

# Option 2: Direct cargo
cargo build --release

# Option 3: With test run
cargo test test_asio_integration -- --nocapture
cargo build --release
```

## ✅ Verification After Build

1. **Check build logs for ASIO**:
   Look for:
   ```
   Compiling asio-sys v0.2.2
   CPAL_ASIO_DIR is set at T:\projects\asiovst\asiosdk
   ...building bindings from ASIO SDK headers
   ...compiling ASIO SDK sources
      Compiling cpal v0.15
      Compiling rusty-audio v0.1.0
   ```

2. **Run the application**:
   ```powershell
   .\target\release\rusty-audio_native.exe
   ```

3. **Check ASIO backend in Settings**:
   - Open application
   - Go to Settings tab
   - Look for "ASIO" in audio backend dropdown
   - Select ASIO backend
   - ASIO devices should appear in device list

4. **Run ASIO integration test**:
   ```powershell
   cargo test test_asio_integration -- --nocapture
   ```

   Expected output if ASIO hardware is connected:
   ```
   === ASIO SDK Integration Test ===
   ASIO available: true
   Available backends: [Asio, Wasapi, DirectSound]
   ✅ ASIO backend is available

   === Enumerating ASIO Devices ===
   Found X ASIO output devices
   Device 1: [Your ASIO Interface Name]
   ...
   ✅ ASIO integration successful!
   ```

## ⚠️ Troubleshooting

### "Failed to download ASIO SDK"
**Solution**: Check .cargo/config.toml has correct path:
```powershell
Get-Content .cargo\config.toml | Select-String CPAL_ASIO_DIR
# Should show: CPAL_ASIO_DIR = { value = "T:\\projects\\asiovst\\asiosdk"
```

### "Could not find vcvarsall.bat"
**Solution**: Install Visual Studio Build Tools
- Download: https://visualstudio.microsoft.com/downloads/
- Or add MSVC to PATH manually

### "Unable to find libclang"
**Solution**: Install LLVM/Clang
```powershell
choco install llvm
$env:LIBCLANG_PATH = "C:\Program Files\LLVM\bin"
```

### No ASIO devices found
**Solutions**:
1. Install ASIO drivers for your audio interface
2. Or install ASIO4ALL for testing: https://www.asio4all.org/
3. Application will fall back to WASAPI automatically

## 📊 Expected Results

- **With ASIO hardware**: Application uses ASIO backend, <10ms latency
- **Without ASIO hardware**: Application falls back to WASAPI, everything still works
- **Build time**: ~8-10 minutes first build, ~1-2 minutes incremental

## 🎯 Next Steps After Successful Build

1. Test audio playback with ASIO backend
2. Verify low-latency performance (<10ms)
3. Test with professional audio interface if available
4. Configure ASIO driver settings (buffer size, sample rate)

## 📝 Files Changed for ASIO Integration

- `.cargo/config.toml` - ASIO SDK path configuration
- `src/audio/asio_backend.rs` - Added integration test
- `justfile` - Fixed Windows build command
- `Cargo.toml` - ASIO feature enabled (line 207)

---

**See Also**:
- `ASIO_INTEGRATION.md` - Comprehensive integration guide
- `BUILD_WINDOWS.md` - General Windows build instructions
