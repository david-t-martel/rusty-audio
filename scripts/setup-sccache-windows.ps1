# Setup sccache for Windows development
# Run this script to configure sccache for optimal Windows performance

param(
    [switch]$Install,
    [switch]$Configure,
    [switch]$Test,
    [switch]$Stats,
    [switch]$Clear
)

$ErrorActionPreference = "Stop"

function Write-Header {
    param([string]$Message)
    Write-Host "`n===========================================================" -ForegroundColor Cyan
    Write-Host "  $Message" -ForegroundColor Cyan
    Write-Host "===========================================================`n" -ForegroundColor Cyan
}

function Install-Sccache {
    Write-Header "Installing sccache"

    if (Get-Command sccache -ErrorAction SilentlyContinue) {
        Write-Host "✅ sccache is already installed" -ForegroundColor Green
        sccache --version
    } else {
        Write-Host "📦 Installing sccache via cargo..." -ForegroundColor Yellow
        cargo install sccache --locked
        Write-Host "✅ sccache installed successfully" -ForegroundColor Green
    }
}

function Configure-Sccache {
    Write-Header "Configuring sccache for Windows"

    # Create config directory
    $configDir = "$env:LOCALAPPDATA\Mozilla\sccache"
    if (!(Test-Path $configDir)) {
        New-Item -ItemType Directory -Force -Path $configDir | Out-Null
        Write-Host "✅ Created config directory: $configDir" -ForegroundColor Green
    }

    # Copy config file
    $configSource = ".cargo\sccache-config.toml"
    $configDest = "$configDir\config"

    if (Test-Path $configSource) {
        Copy-Item $configSource $configDest -Force
        Write-Host "✅ Copied sccache config to $configDest" -ForegroundColor Green
    }

    # Create cache directory
    $cacheDir = "$env:USERPROFILE\.cache\sccache"
    if (!(Test-Path $cacheDir)) {
        New-Item -ItemType Directory -Force -Path $cacheDir | Out-Null
        Write-Host "✅ Created cache directory: $cacheDir" -ForegroundColor Green
    }

    # Set environment variables
    [Environment]::SetEnvironmentVariable("RUSTC_WRAPPER", "sccache", "User")
    [Environment]::SetEnvironmentVariable("SCCACHE_DIR", $cacheDir, "User")
    [Environment]::SetEnvironmentVariable("SCCACHE_CACHE_SIZE", "10G", "User")

    Write-Host "✅ Environment variables set:" -ForegroundColor Green
    Write-Host "   RUSTC_WRAPPER=sccache"
    Write-Host "   SCCACHE_DIR=$cacheDir"
    Write-Host "   SCCACHE_CACHE_SIZE=10G"

    Write-Host "`n⚠️  Please restart your terminal for environment variables to take effect" -ForegroundColor Yellow
}

function Test-Sccache {
    Write-Header "Testing sccache"

    Write-Host "Checking sccache version..." -ForegroundColor Cyan
    sccache --version

    Write-Host "`nChecking sccache status..." -ForegroundColor Cyan
    sccache --show-stats

    Write-Host "`nRunning test build..." -ForegroundColor Cyan
    $env:RUSTC_WRAPPER = "sccache"
    cargo check

    Write-Host "`nCache statistics after test build:" -ForegroundColor Cyan
    sccache --show-stats

    Write-Host "`n✅ sccache test complete" -ForegroundColor Green
}

function Show-Stats {
    Write-Header "sccache Statistics"
    sccache --show-stats
}

function Clear-Cache {
    Write-Header "Clearing sccache cache"

    Write-Host "⚠️  This will delete all cached compilation artifacts" -ForegroundColor Yellow
    $confirm = Read-Host "Are you sure? (y/N)"

    if ($confirm -eq "y" -or $confirm -eq "Y") {
        sccache --stop-server
        $cacheDir = "$env:USERPROFILE\.cache\sccache"
        if (Test-Path $cacheDir) {
            Remove-Item -Recurse -Force $cacheDir
            Write-Host "✅ Cache cleared: $cacheDir" -ForegroundColor Green
        }
        New-Item -ItemType Directory -Force -Path $cacheDir | Out-Null
        Write-Host "✅ Cache directory recreated" -ForegroundColor Green
    } else {
        Write-Host "❌ Cancelled" -ForegroundColor Red
    }
}

# Main execution
if ($Install) {
    Install-Sccache
}

if ($Configure) {
    Configure-Sccache
}

if ($Test) {
    Test-Sccache
}

if ($Stats) {
    Show-Stats
}

if ($Clear) {
    Clear-Cache
}

# If no parameters, show help
if (!$Install -and !$Configure -and !$Test -and !$Stats -and !$Clear) {
    Write-Header "sccache Setup for Windows"

    Write-Host "Usage: .\scripts\setup-sccache-windows.ps1 [OPTIONS]`n" -ForegroundColor White
    Write-Host "OPTIONS:" -ForegroundColor Yellow
    Write-Host "  -Install      Install sccache via cargo"
    Write-Host "  -Configure    Configure sccache for Windows"
    Write-Host "  -Test         Test sccache with a build"
    Write-Host "  -Stats        Show cache statistics"
    Write-Host "  -Clear        Clear sccache cache"
    Write-Host ""
    Write-Host "QUICK START:" -ForegroundColor Cyan
    Write-Host "  1. .\scripts\setup-sccache-windows.ps1 -Install"
    Write-Host "  2. .\scripts\setup-sccache-windows.ps1 -Configure"
    Write-Host "  3. Restart your terminal"
    Write-Host "  4. .\scripts\setup-sccache-windows.ps1 -Test"
    Write-Host ""
    Write-Host "EXAMPLES:" -ForegroundColor Cyan
    Write-Host "  # Full setup"
    Write-Host "  .\scripts\setup-sccache-windows.ps1 -Install -Configure"
    Write-Host ""
    Write-Host "  # Check cache performance"
    Write-Host "  .\scripts\setup-sccache-windows.ps1 -Stats"
    Write-Host ""
}
