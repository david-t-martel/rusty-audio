# Windows Development Setup Guide

This guide will help you set up rusty-audio for optimal Windows development with sccache enabled for fast compilation.

## Prerequisites

### Required
- **Windows 10/11** (64-bit)
- **Rust** (latest stable) - Install from [rustup.rs](https://rustup.rs/)
- **Visual Studio Build Tools** or **Visual Studio 2022** with C++ workload
- **Git** for Windows

### Recommended
- **Windows Terminal** for better command-line experience
- **VSCode** with rust-analyzer extension
- **PowerShell 7+** for modern scripting

## Quick Start (5 minutes)

> **IMPORTANT**: All commands must be run in **Windows PowerShell** or **Windows Terminal**, NOT in WSL!

### 1. Install sccache on Windows

Open **Windows PowerShell** (not WSL) and run:

```powershell
# Install sccache for Windows
cargo install sccache --locked
```

Wait for installation to complete (~2-3 minutes).

Verify sccache is installed:

```powershell
sccache --version
# Should show: sccache 0.8.x or higher
```

### 2. Configure sccache

Run the automated setup script:

```powershell
.\scripts\setup-sccache-windows.ps1 -Install -Configure
```

### 3. Restart Terminal

Close and reopen your terminal for environment variables to take effect.

### 4. Verify Setup

```powershell
# Check sccache is working
sccache --version
sccache --show-stats

# Build the project (first build will be slow)
cargo build

# Check cache statistics
sccache --show-stats
```

You should see compile requests and cache statistics!

## Detailed Setup

### sccache Configuration

sccache is **pre-configured** in `.cargo/config.toml`:

```toml
[build]
rustc-wrapper = "sccache"
```

This means sccache will automatically cache all Rust compilations.

### Manual Configuration (if needed)

If the automated script doesn't work, configure manually:

#### 1. Set Environment Variables

```powershell
# Set permanently (requires restart)
[Environment]::SetEnvironmentVariable("RUSTC_WRAPPER", "sccache", "User")
[Environment]::SetEnvironmentVariable("SCCACHE_DIR", "$env:USERPROFILE\.cache\sccache", "User")
[Environment]::SetEnvironmentVariable("SCCACHE_CACHE_SIZE", "10G", "User")
```

#### 2. Create Cache Directory

```powershell
New-Item -ItemType Directory -Force -Path "$env:USERPROFILE\.cache\sccache"
```

#### 3. Restart Terminal

Close and reopen PowerShell/Windows Terminal.

## Building the Project

### Debug Build (Fast)

```powershell
cargo build
```

First build: ~5 minutes
Subsequent builds with sccache: ~30 seconds

### Release Build (Optimized)

```powershell
cargo build --release
```

First build: ~10 minutes
Subsequent builds with sccache: ~2 minutes

### Run the Application

```powershell
cargo run --release
```

## Testing

### Run All Tests

```powershell
cargo test
```

### Run Specific Test

```powershell
cargo test test_name -- --nocapture
```

### Run Benchmarks

```powershell
cargo bench
```

## sccache Management

### Check Cache Statistics

```powershell
sccache --show-stats
```

Output shows:
- Compile requests
- Cache hits/misses
- Cache hit rate (higher is better!)
- Cache location

### Clear Cache

```powershell
.\scripts\setup-sccache-windows.ps1 -Clear
```

Or manually:

```powershell
sccache --stop-server
Remove-Item -Recurse -Force "$env:USERPROFILE\.cache\sccache"
New-Item -ItemType Directory -Force -Path "$env:USERPROFILE\.cache\sccache"
```

### Monitor Cache Performance

```powershell
# Before build
sccache --show-stats

# Build project
cargo build

# After build - check improvement
sccache --show-stats
```

## Performance Optimization

### Parallel Compilation

Already configured in `.cargo/config.toml`:

```toml
[build]
jobs = 16  # Use all CPU cores
```

### Incremental Compilation

Enabled by default for debug builds:

```toml
[profile.dev]
incremental = true
```

### Expected Build Times

| Build Type | First Build | With sccache | Improvement |
|-----------|-------------|--------------|-------------|
| Debug | ~5 min | ~30 sec | **10x faster** |
| Release | ~10 min | ~2 min | **5x faster** |
| Clean rebuild | ~10 min | ~1 min | **10x faster** |

## Common Issues & Solutions

### Issue: "sccache: command not found"

**Solution**: Install sccache and add to PATH:

```powershell
cargo install sccache --locked
# Restart terminal
```

### Issue: Cache not working (0 hits)

**Solution**: Check environment variables:

```powershell
$env:RUSTC_WRAPPER
# Should output: sccache

# If empty, set it:
$env:RUSTC_WRAPPER = "sccache"
cargo build  # Should now use sccache
```

### Issue: Build errors with sccache

**Solution**: Temporarily disable sccache:

```powershell
$env:RUSTC_WRAPPER = ""
cargo build
```

Then re-enable:

```powershell
$env:RUSTC_WRAPPER = "sccache"
```

### Issue: Cache fills up disk

**Solution**: Reduce cache size:

```powershell
[Environment]::SetEnvironmentVariable("SCCACHE_CACHE_SIZE", "5G", "User")
.\scripts\setup-sccache-windows.ps1 -Clear
```

### Issue: Slow first build even with sccache

**Expected behavior!** sccache only helps on:
- Rebuilds after code changes
- Clean rebuilds
- Switching between branches

First build must compile everything.

## Just Commands (Task Runner)

Install just for convenient task running:

```powershell
cargo install just
```

Then use these commands:

### Development

```powershell
just build          # Debug build
just build-release  # Release build
just run            # Run debug
just run-release    # Run release
just test           # Run tests
```

### Code Quality

```powershell
just lint           # Run clippy
just fmt            # Format code
just quality        # All checks
```

### sccache Management

```powershell
just sccache-stats  # Show statistics
just sccache-clear  # Clear cache
just install-sccache # Install sccache
```

### CI Simulation

```powershell
just ci-local       # Full CI pipeline
just pre-commit     # Pre-commit checks
just pre-push       # Pre-push checks
```

See all commands:

```powershell
just --list
```

## VSCode Setup

### Recommended Extensions

Install these extensions:
- **rust-analyzer** - Rust language server
- **CodeLLDB** - Debugging support
- **Even Better TOML** - TOML syntax highlighting
- **Error Lens** - Inline error messages

### Settings

Add to `.vscode/settings.json`:

```json
{
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.cargo.allFeatures": true,
  "rust-analyzer.procMacro.enable": true,
  "rust-analyzer.cargo.buildScripts.enable": true
}
```

## Windows-Specific Features

### Stack Size Increase

Configured in `.cargo/config.toml` for audio processing:

```toml
[target.x86_64-pc-windows-msvc]
rustflags = [
    "-C", "link-arg=/STACK:8388608"  # 8MB stack
]
```

### ASIO Support (Future)

Low-latency audio support will use ASIO4ALL on Windows.

### Windows Audio APIs

The project uses CPAL which wraps:
- **WASAPI** (Windows Audio Session API) - Default
- **DirectSound** - Fallback
- **ASIO** (planned) - Professional audio

## Git Workflow

### Clone Repository

```powershell
git clone https://github.com/your-repo/rusty-audio.git
cd rusty-audio
git submodule update --init --recursive
```

### Development Workflow

```powershell
# Create feature branch
git checkout -b feat/my-feature

# Make changes
# ...

# Run quality checks
just pre-commit

# Commit
git add .
git commit -m "feat: add my feature"

# Push
git push origin feat/my-feature
```

## Troubleshooting

### PowerShell Execution Policy

If scripts don't run:

```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

### Audio Device Issues

If no audio devices found:

1. Check Windows sound settings
2. Ensure audio devices are enabled
3. Run as administrator (some audio APIs need it)

### Compilation Errors

1. Update Rust: `rustup update`
2. Clean build: `cargo clean && cargo build`
3. Check Visual Studio Build Tools are installed

## Performance Monitoring

### Build Time Tracking

```powershell
# Time a build
Measure-Command { cargo build }

# With sccache stats
sccache --zero-stats
cargo build
sccache --show-stats
```

### CPU Usage

Open Task Manager during build:
- Rust should use ~80-100% CPU
- Multiple `rustc` processes
- sccache server process

## Next Steps

1. ✅ sccache configured
2. ✅ Project builds successfully
3. 📖 Read [CLAUDE.md](./CLAUDE.md) for architecture
4. 📖 Read [WORKFLOW_OPTIMIZATION_GUIDE.md](./WORKFLOW_OPTIMIZATION_GUIDE.md)
5. 🎵 Run the audio player: `cargo run --release`
6. 🧪 Explore code in `src/`

## Getting Help

- **Documentation**: See CLAUDE.md
- **Issues**: Check GitHub Issues
- **Rust Help**: https://users.rust-lang.org/
- **Audio**: See web-audio-api-rs examples

## Useful Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Cargo Book](https://doc.rust-lang.org/cargo/)
- [sccache GitHub](https://github.com/mozilla/sccache)
- [CPAL Documentation](https://docs.rs/cpal/)
- [egui Documentation](https://docs.rs/egui/)

---

**Happy Windows Development! 🎵🪟**
