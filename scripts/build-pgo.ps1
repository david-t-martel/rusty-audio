# Profile-Guided Optimization (PGO) Build Script
# This script performs a multi-stage build with PGO for maximum performance
#
# Usage:
#   .\scripts\build-pgo.ps1
#
# Process:
#   1. Build instrumented binary (collects profile data)
#   2. Run workload to generate profile data
#   3. Build optimized binary using profile data
#
# Expected performance gain: 10-15% improvement in hot paths

param(
    [switch]$Clean,
    [switch]$SkipWorkload,
    [string]$WorkloadDuration = "60"  # seconds
)

$ErrorActionPreference = "Stop"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Profile-Guided Optimization Build" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Directories
$ProjectRoot = Split-Path -Parent $PSScriptRoot
$ProfileDir = Join-Path $ProjectRoot "pgo-data"
$InstrumentedBinary = Join-Path $ProjectRoot "target\pgo-instrument\rusty-audio.exe"
$OptimizedBinary = Join-Path $ProjectRoot "target\pgo-use\rusty-audio.exe"

# Clean previous PGO data if requested
if ($Clean) {
    Write-Host "[1/4] Cleaning previous PGO data..." -ForegroundColor Yellow
    if (Test-Path $ProfileDir) {
        Remove-Item -Recurse -Force $ProfileDir
        Write-Host "  ✓ Removed $ProfileDir" -ForegroundColor Green
    }
    if (Test-Path "$ProjectRoot\target\pgo-instrument") {
        Remove-Item -Recurse -Force "$ProjectRoot\target\pgo-instrument"
        Write-Host "  ✓ Removed instrumented build" -ForegroundColor Green
    }
    if (Test-Path "$ProjectRoot\target\pgo-use") {
        Remove-Item -Recurse -Force "$ProjectRoot\target\pgo-use"
        Write-Host "  ✓ Removed optimized build" -ForegroundColor Green
    }
}

# Create profile directory
New-Item -ItemType Directory -Force -Path $ProfileDir | Out-Null

# Step 1: Build instrumented binary
Write-Host ""
Write-Host "[1/4] Building instrumented binary..." -ForegroundColor Yellow
$env:RUSTFLAGS = "-Cprofile-generate=$ProfileDir"
Push-Location $ProjectRoot
try {
    cargo build --profile pgo-instrument --target-dir target
    if ($LASTEXITCODE -ne 0) {
        throw "Instrumented build failed"
    }
    Write-Host "  ✓ Instrumented binary created" -ForegroundColor Green
} finally {
    Pop-Location
    $env:RUSTFLAGS = ""
}

# Step 2: Run workload to collect profile data
if (-not $SkipWorkload) {
    Write-Host ""
    Write-Host "[2/4] Running workload to collect profile data..." -ForegroundColor Yellow
    Write-Host "  Duration: $WorkloadDuration seconds" -ForegroundColor Cyan
    Write-Host "  Please use the application normally (load files, play audio, use EQ)" -ForegroundColor Cyan
    Write-Host ""

    # Start the instrumented binary
    $Process = Start-Process -FilePath $InstrumentedBinary -PassThru -WindowStyle Normal

    # Wait for specified duration or until user closes the app
    $Elapsed = 0
    $CheckInterval = 5
    while ($Elapsed -lt $WorkloadDuration -and -not $Process.HasExited) {
        Start-Sleep -Seconds $CheckInterval
        $Elapsed += $CheckInterval
        $Remaining = $WorkloadDuration - $Elapsed
        if ($Remaining -gt 0) {
            Write-Host "  Time remaining: $Remaining seconds..." -ForegroundColor Gray
        }
    }

    # Stop the process if still running
    if (-not $Process.HasExited) {
        Write-Host "  Stopping instrumented binary..." -ForegroundColor Yellow
        $Process.CloseMainWindow() | Out-Null
        Start-Sleep -Seconds 2
        if (-not $Process.HasExited) {
            $Process.Kill()
        }
    }

    # Check for profile data
    $ProfileFiles = Get-ChildItem -Path $ProfileDir -Filter "*.profraw" -Recurse
    if ($ProfileFiles.Count -eq 0) {
        Write-Warning "No profile data collected! The workload may have been too short."
        Write-Host "  Run again with longer workload duration or manually use the app." -ForegroundColor Yellow
        exit 1
    }

    Write-Host "  ✓ Collected $($ProfileFiles.Count) profile data files" -ForegroundColor Green
} else {
    Write-Host ""
    Write-Host "[2/4] Skipping workload (using existing profile data)..." -ForegroundColor Yellow
}

# Step 3: Merge profile data
Write-Host ""
Write-Host "[3/4] Merging profile data..." -ForegroundColor Yellow

# Find llvm-profdata tool
$LlvmProfdata = Get-Command llvm-profdata -ErrorAction SilentlyContinue
if (-not $LlvmProfdata) {
    # Try to find in Rust installation
    $RustcPath = (Get-Command rustc).Source
    $RustcDir = Split-Path -Parent $RustcPath
    $LlvmProfdataPath = Join-Path $RustcDir "llvm-profdata.exe"
    if (Test-Path $LlvmProfdataPath) {
        $LlvmProfdata = Get-Item $LlvmProfdataPath
    } else {
        Write-Error "llvm-profdata not found! Install LLVM tools or use rustup component add llvm-tools-preview"
        exit 1
    }
}

$MergedProfile = Join-Path $ProfileDir "merged.profdata"
& $LlvmProfdata.Source merge -o $MergedProfile (Join-Path $ProfileDir "*.profraw")
if ($LASTEXITCODE -ne 0) {
    throw "Profile merge failed"
}

Write-Host "  ✓ Profile data merged to $MergedProfile" -ForegroundColor Green

# Step 4: Build optimized binary
Write-Host ""
Write-Host "[4/4] Building PGO-optimized binary..." -ForegroundColor Yellow
$env:RUSTFLAGS = "-Cprofile-use=$MergedProfile -Cllvm-args=-pgo-warn-missing-function"
Push-Location $ProjectRoot
try {
    cargo build --profile pgo-use --target-dir target
    if ($LASTEXITCODE -ne 0) {
        throw "PGO-optimized build failed"
    }
    Write-Host "  ✓ PGO-optimized binary created" -ForegroundColor Green
} finally {
    Pop-Location
    $env:RUSTFLAGS = ""
}

# Summary
Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "PGO Build Complete!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Binaries:" -ForegroundColor White
Write-Host "  Instrumented: $InstrumentedBinary" -ForegroundColor Gray
Write-Host "  Optimized:    $OptimizedBinary" -ForegroundColor Gray
Write-Host ""
Write-Host "Expected performance gain: 10-15% in hot paths" -ForegroundColor Green
Write-Host ""
Write-Host "To use the optimized binary:" -ForegroundColor White
Write-Host "  Copy $OptimizedBinary to your desired location" -ForegroundColor Gray
Write-Host ""
