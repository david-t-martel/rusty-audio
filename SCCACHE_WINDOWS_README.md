# sccache for Windows - Quick Reference

## Current Status

✅ **Configured**: sccache is enabled in `.cargo/config.toml`
📍 **Platform**: Windows (primary development platform)
⚙️ **Jobs Limit**: 8 CPU cores maximum

## How sccache Works

sccache is a **compilation cache** for Rust:

1. **First compilation**: Rust compiles and sccache saves the results
2. **Second compilation**: sccache returns cached results instead of recompiling
3. **Result**: 5-10x faster rebuild times!

## Verify sccache is Working (Windows)

Open **Windows PowerShell** and run:

```powershell
# Check sccache version
sccache --version

# Check cache statistics
sccache --show-stats
```

You should see:
```
Compile requests                      X
Cache hits                            Y
Cache misses                          Z
Cache hits rate                       X%
Cache location                  Local disk: "C:\Users\david\.cache\sccache"
```

## Test sccache Performance

### Baseline Test (First Build)

```powershell
# Clear cache to start fresh
sccache --zero-stats

# Clean build directory
cargo clean

# Time the first build
Measure-Command { cargo build }
# Expected: ~5-10 minutes

# Check cache stats
sccache --show-stats
# Should show cache misses
```

### Cached Build Test (Second Build)

```powershell
# Clear cache stats but keep cached data
sccache --zero-stats

# Clean and rebuild
cargo clean
Measure-Command { cargo build }
# Expected: ~30-60 seconds (10x faster!)

# Check cache stats
sccache --show-stats
# Should show high cache hit rate (80%+)
```

## Configuration Files

### 1. `.cargo/config.toml` (Project Root)

```toml
[build]
jobs = 8                    # Limit CPU usage
rustc-wrapper = "sccache"   # Enable sccache
```

This tells Cargo to use sccache for all compilations in this project.

### 2. `.cargo/sccache-config.toml`

Optional advanced configuration. Copy to:

**Windows**: `%LOCALAPPDATA%\Mozilla\sccache\config`

Contains settings like:
- Cache size (default 10GB)
- Cache location
- Performance options

## Quick Commands

```powershell
# Show statistics
sccache --show-stats

# Clear statistics (but keep cache)
sccache --zero-stats

# Stop sccache server
sccache --stop-server

# Start sccache server (automatic on first use)
# No explicit start needed - runs automatically

# Clear entire cache
sccache --stop-server
Remove-Item -Recurse -Force "$env:USERPROFILE\.cache\sccache"
```

## Automated Setup Script

Run the PowerShell setup script:

```powershell
# Full setup
.\scripts\setup-sccache-windows.ps1 -Install -Configure

# Just test
.\scripts\setup-sccache-windows.ps1 -Test

# Show stats
.\scripts\setup-sccache-windows.ps1 -Stats

# Clear cache
.\scripts\setup-sccache-windows.ps1 -Clear
```

## Performance Expectations

| Build Type | First Build | With sccache | Improvement |
|-----------|-------------|--------------|-------------|
| Clean debug | ~5 min | ~30 sec | **10x faster** |
| Clean release | ~10 min | ~2 min | **5x faster** |
| Incremental | ~20 sec | ~5 sec | **4x faster** |

## Troubleshooting

### sccache not found

**Problem**: `sccache: command not found`

**Solution**:
```powershell
cargo install sccache --locked
# Restart PowerShell
```

### Cache not working (0% hit rate)

**Problem**: `sccache --show-stats` shows 0 compile requests

**Solution**: Build the project first!
```powershell
cargo build
sccache --show-stats  # Now should show activity
```

### Slow builds despite sccache

**Expected**: First build is always slow - sccache only helps on:
- Rebuilds after small changes
- Clean rebuilds
- Branch switching

**Test properly**:
```powershell
# Build twice to see the benefit
cargo build        # Slow (populates cache)
cargo clean
cargo build        # Fast (uses cache)
```

### sccache server crashes

**Solution**:
```powershell
# Kill all sccache processes
taskkill /F /IM sccache.exe

# Restart by building
cargo build  # Auto-starts sccache
```

### Cache size too large

**Solution**: Reduce cache size:
```powershell
[Environment]::SetEnvironmentVariable("SCCACHE_CACHE_SIZE", "5G", "User")
.\scripts\setup-sccache-windows.ps1 -Clear
```

## Environment Variables (Windows)

Set these for optimal performance:

```powershell
# Set permanently (requires terminal restart)
[Environment]::SetEnvironmentVariable("RUSTC_WRAPPER", "sccache", "User")
[Environment]::SetEnvironmentVariable("SCCACHE_DIR", "$env:USERPROFILE\.cache\sccache", "User")
[Environment]::SetEnvironmentVariable("SCCACHE_CACHE_SIZE", "10G", "User")
```

Or set in current session only:

```powershell
$env:RUSTC_WRAPPER = "sccache"
$env:SCCACHE_DIR = "$env:USERPROFILE\.cache\sccache"
$env:SCCACHE_CACHE_SIZE = "10G"
```

## Monitoring Cache Usage

### View cache directory size

```powershell
# Windows
Get-ChildItem "$env:USERPROFILE\.cache\sccache" -Recurse |
    Measure-Object -Property Length -Sum |
    Select-Object @{Name="Size(GB)";Expression={[math]::Round($_.Sum/1GB, 2)}}
```

### Watch real-time compilation

```powershell
# Open two terminals:

# Terminal 1: Watch cache stats
while ($true) {
    Clear-Host
    sccache --show-stats
    Sleep 2
}

# Terminal 2: Build project
cargo build
```

## WSL vs Windows

**IMPORTANT**: This project uses **Windows sccache**, not WSL sccache!

- ✅ Install sccache on **Windows**: `cargo install sccache`
- ✅ Run builds from **Windows PowerShell**
- ✅ Check stats in **Windows PowerShell**: `sccache --show-stats`
- ❌ Don't use WSL sccache - it won't cache Windows builds!

## Just Commands (Windows PowerShell)

If you have `just` installed:

```powershell
just install-sccache   # Install sccache
just sccache-stats     # Show statistics
just sccache-clear     # Clear cache
just sccache-start     # Start server
```

## Next Steps

1. ✅ sccache installed and configured
2. 🔨 Build the project to populate cache
3. 📊 Check statistics: `sccache --show-stats`
4. 🚀 Enjoy 10x faster rebuilds!

---

**Questions?** See [WINDOWS_SETUP_GUIDE.md](./WINDOWS_SETUP_GUIDE.md) for full setup instructions.
