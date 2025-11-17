#!/usr/bin/env pwsh

<#
.SYNOPSIS
    Build and Serve Script for Rusty Audio WASM Application (PowerShell)

.DESCRIPTION
    Provides a unified workflow for:
    - Building the WASM binary with proper multithreading flags
    - Optimizing the WASM output
    - Compressing assets (Brotli + Gzip)
    - Validating the build
    - Starting the development server

.PARAMETER SkipBuild
    Skip the WASM build step

.PARAMETER SkipOptimize
    Skip WASM optimization

.PARAMETER SkipCompress
    Skip asset compression

.PARAMETER Port
    Server port (default: 8080)

.PARAMETER Verbose
    Enable verbose logging

.PARAMETER Prod
    Production mode

.EXAMPLE
    .\scripts\build-and-serve.ps1
    .\scripts\build-and-serve.ps1 -Prod
    .\scripts\build-and-serve.ps1 -SkipOptimize -Port 3000
#>

param(
    [switch]$SkipBuild,
    [switch]$SkipOptimize,
    [switch]$SkipCompress,
    [int]$Port = 8080,
    [switch]$Verbose,
    [switch]$Prod
)

$ErrorActionPreference = "Stop"

# Color functions
function Write-Header($Message) {
    Write-Host "`n============================================================" -ForegroundColor Blue
    Write-Host "  $Message" -ForegroundColor Cyan
    Write-Host "============================================================`n" -ForegroundColor Blue
}

function Write-Success($Message) {
    Write-Host "✓ $Message" -ForegroundColor Green
}

function Write-Error($Message) {
    Write-Host "✗ $Message" -ForegroundColor Red
}

function Write-Warning($Message) {
    Write-Host "⚠ $Message" -ForegroundColor Yellow
}

function Write-Info($Message) {
    Write-Host "→ $Message" -ForegroundColor Cyan
}

# Get script directory
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent $ScriptDir
$DistDir = Join-Path $ProjectRoot "dist"

Set-Location $ProjectRoot

Write-Header "Rusty Audio Build and Serve"

# Check prerequisites
Write-Info "Checking prerequisites..."

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Error "Rust/Cargo not found. Install from https://rustup.rs/"
    exit 1
}

if (-not (Get-Command trunk -ErrorAction SilentlyContinue)) {
    Write-Error "Trunk not found. Install with: cargo install trunk"
    exit 1
}

if (-not (Get-Command node -ErrorAction SilentlyContinue)) {
    Write-Error "Node.js not found. Install from https://nodejs.org/"
    exit 1
}

if (-not (Get-Command wasm-opt -ErrorAction SilentlyContinue)) {
    Write-Warning "wasm-opt not found. Install binaryen for optimization."
    $SkipOptimize = $true
}

Write-Success "All prerequisites found"

# Check if node_modules exists
if (-not (Test-Path "node_modules")) {
    Write-Info "Installing Node.js dependencies..."
    npm install
    Write-Success "Dependencies installed"
}

# Build step
if (-not $SkipBuild) {
    Write-Header "Building WASM Binary"

    # Set Rust flags for multithreading
    $env:RUSTFLAGS = "-C target-feature=+atomics,+bulk-memory,+mutable-globals"

    if ($Prod) {
        Write-Info "Building in RELEASE mode..."
        trunk build --release
    } else {
        Write-Info "Building in DEBUG mode (faster build)..."
        trunk build
    }

    if ($LASTEXITCODE -eq 0) {
        Write-Success "WASM build completed"

        # Display WASM file size
        $WasmFile = Join-Path $DistDir "rusty_audio_bg.wasm"
        if (Test-Path $WasmFile) {
            $WasmSize = (Get-Item $WasmFile).Length
            $WasmSizeMB = [math]::Round($WasmSize / 1MB, 2)
            Write-Info "WASM size: $WasmSizeMB MB"
        }
    } else {
        Write-Error "WASM build failed"
        exit 1
    }
} else {
    Write-Warning "Skipping build step"
}

# Optimization step
if (-not $SkipOptimize -and $Prod) {
    Write-Header "Optimizing WASM Binary"

    $WasmFile = Join-Path $DistDir "rusty_audio_bg.wasm"

    if (Test-Path $WasmFile) {
        $OriginalSize = (Get-Item $WasmFile).Length

        Write-Info "Running wasm-opt with threading optimizations..."

        wasm-opt $WasmFile `
            -Oz `
            --enable-threads `
            --enable-bulk-memory `
            --enable-simd `
            -o "$WasmFile.opt"

        if ($LASTEXITCODE -eq 0) {
            Move-Item "$WasmFile.opt" $WasmFile -Force

            $OptimizedSize = (Get-Item $WasmFile).Length
            $Reduction = [math]::Round((1 - $OptimizedSize / $OriginalSize) * 100, 2)

            Write-Success "Optimization complete"
            Write-Info "Original: $([math]::Round($OriginalSize / 1MB, 2)) MB"
            Write-Info "Optimized: $([math]::Round($OptimizedSize / 1MB, 2)) MB"
            Write-Info "Reduction: $Reduction%"
        } else {
            Write-Error "Optimization failed, using unoptimized binary"
        }
    } else {
        Write-Error "WASM file not found: $WasmFile"
    }
} elseif ($SkipOptimize) {
    Write-Warning "Skipping optimization step"
} else {
    Write-Info "Optimization only runs in production mode"
}

# Compression step
if (-not $SkipCompress -and $Prod) {
    Write-Header "Compressing Assets"

    $WasmFile = Join-Path $DistDir "rusty_audio_bg.wasm"

    if (Test-Path $WasmFile) {
        Write-Info "Compressing WASM binary..."

        # Node.js compression script
        node (Join-Path $ScriptDir "compress-assets.js")

        Write-Success "Asset compression complete"
    }
} elseif ($SkipCompress) {
    Write-Warning "Skipping compression step"
} else {
    Write-Info "Compression only runs in production mode"
}

# Validation step
Write-Header "Validating Build"

$ValidationFailed = $false

# Check for required files
$RequiredFiles = @(
    "index.html",
    "rusty_audio_bg.wasm",
    "rusty_audio.js"
)

foreach ($File in $RequiredFiles) {
    $FilePath = Join-Path $DistDir $File
    if (Test-Path $FilePath) {
        Write-Success "Found: $File"
    } else {
        Write-Error "Missing: $File"
        $ValidationFailed = $true
    }
}

# Check for service worker
$ServiceWorker = Join-Path $DistDir "service-worker.js"
if (Test-Path $ServiceWorker) {
    Write-Success "Service worker present"
} else {
    Write-Warning "Service worker not found (optional)"
}

# Check for static assets
$StaticDir = Join-Path $DistDir "static"
$IconsDir = Join-Path $DistDir "icons"
if ((Test-Path $StaticDir) -or (Test-Path $IconsDir)) {
    Write-Success "Static assets found"
} else {
    Write-Warning "Static assets directory not found"
}

if ($ValidationFailed) {
    Write-Error "Build validation failed"
    exit 1
}

Write-Success "Build validation passed"

# Start development server
Write-Header "Starting Development Server"

Write-Info "Server will start on port: $Port"
Write-Info "Press Ctrl+C to stop the server"

$ServerArgs = @("$ScriptDir\dev-server.js", "--port", $Port)
if ($Verbose) {
    $ServerArgs += "--verbose"
}

node @ServerArgs
