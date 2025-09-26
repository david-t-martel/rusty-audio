#!/usr/bin/env powershell
# Performance Analysis Script for Rusty Audio

param(
    [switch]$RunBenchmarks,
    [switch]$Profile,
    [switch]$GenerateReport,
    [switch]$All
)

Write-Host "===========================================" -ForegroundColor Cyan
Write-Host "   Rusty Audio Performance Analysis Tool   " -ForegroundColor Cyan
Write-Host "===========================================" -ForegroundColor Cyan
Write-Host ""

$projectRoot = Get-Location

# Function to check if command exists
function Test-Command($command) {
    $null = Get-Command $command -ErrorAction SilentlyContinue
    return $?
}

# Check prerequisites
Write-Host "Checking prerequisites..." -ForegroundColor Yellow
if (-not (Test-Command "cargo")) {
    Write-Error "Cargo not found. Please install Rust."
    exit 1
}

# Run benchmarks
if ($RunBenchmarks -or $All) {
    Write-Host ""
    Write-Host "Running performance benchmarks..." -ForegroundColor Green
    Write-Host "=================================" -ForegroundColor Green

    # Run audio benchmarks
    Write-Host "Running audio benchmarks..." -ForegroundColor Yellow
    cargo bench --bench audio_benchmarks

    # Run performance benchmarks
    Write-Host "Running performance benchmarks..." -ForegroundColor Yellow
    cargo bench --bench performance_benchmarks

    Write-Host "Benchmarks completed!" -ForegroundColor Green
}

# Run profiling
if ($Profile -or $All) {
    Write-Host ""
    Write-Host "Running performance profiling..." -ForegroundColor Green
    Write-Host "================================" -ForegroundColor Green

    # Check if perf/flamegraph is available
    if (Test-Command "cargo-flamegraph") {
        Write-Host "Generating flamegraph..." -ForegroundColor Yellow
        $env:CARGO_PROFILE_RELEASE_DEBUG = "true"
        cargo flamegraph --bench audio_benchmarks --root
        Write-Host "Flamegraph saved to flamegraph.svg" -ForegroundColor Green
    } else {
        Write-Host "cargo-flamegraph not installed. Install with: cargo install flamegraph" -ForegroundColor Yellow
    }

    # Windows Performance Toolkit profiling
    if ($IsWindows) {
        Write-Host "Note: For detailed profiling on Windows, consider using:" -ForegroundColor Yellow
        Write-Host "  - Windows Performance Toolkit (WPA/WPR)" -ForegroundColor Gray
        Write-Host "  - Intel VTune Profiler" -ForegroundColor Gray
        Write-Host "  - AMD uProf" -ForegroundColor Gray
    }
}

# Generate performance report
if ($GenerateReport -or $All) {
    Write-Host ""
    Write-Host "Generating performance report..." -ForegroundColor Green
    Write-Host "================================" -ForegroundColor Green

    $reportPath = Join-Path $projectRoot "performance_report.md"
    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"

    # Create report header
    @"
# Rusty Audio Performance Report
Generated: $timestamp

## System Information
- OS: $($env:OS)
- Processor: $($env:PROCESSOR_IDENTIFIER)
- Cores: $($env:NUMBER_OF_PROCESSORS)

## Benchmark Results

### Audio Processing Benchmarks
"@ | Out-File -FilePath $reportPath -Encoding UTF8

    # Parse benchmark results if available
    $benchDir = Join-Path $projectRoot "target\criterion"
    if (Test-Path $benchDir) {
        Get-ChildItem -Path $benchDir -Directory | ForEach-Object {
            $benchName = $_.Name
            $reportFile = Join-Path $_.FullName "base\estimates.json"

            if (Test-Path $reportFile) {
                Add-Content -Path $reportPath -Value "`n#### $benchName"

                # Try to extract key metrics
                $json = Get-Content $reportFile -Raw | ConvertFrom-Json -ErrorAction SilentlyContinue
                if ($json) {
                    Add-Content -Path $reportPath -Value "- Mean: $($json.mean.point_estimate) ns"
                    Add-Content -Path $reportPath -Value "- Std Dev: $($json.std_dev.point_estimate) ns"
                }
            }
        }
    }

    # Add optimization recommendations
    @"

## Performance Optimization Recommendations

### 1. Audio Callback Optimization
- Target latency: < 1ms
- Current status: Check callback_latency_us in metrics
- Recommendations:
  - Use lock-free buffers for real-time threads
  - Minimize allocations in audio callback
  - Pre-allocate all buffers

### 2. Memory Optimization
- Use memory pools for frequent allocations
- Implement buffer recycling
- Monitor peak memory usage

### 3. CPU Optimization
- Enable SIMD operations where possible
- Use parallel processing for non-real-time tasks
- Profile hot paths and optimize critical loops

### 4. UI Rendering
- Target frame time: < 16.67ms (60 FPS)
- Use dirty rect optimization
- Cache complex calculations

### 5. File Loading
- Implement streaming for large files
- Use memory-mapped I/O where appropriate
- Cache decoded audio data

## Next Steps
1. Review benchmark results in target/criterion/*/report/index.html
2. Analyze flamegraph.svg for hot spots
3. Implement targeted optimizations
4. Re-run benchmarks to verify improvements
"@ | Out-File -FilePath $reportPath -Append -Encoding UTF8

    Write-Host "Performance report saved to: $reportPath" -ForegroundColor Green

    # Open report in default editor
    if (Test-Command "code") {
        code $reportPath
    } else {
        notepad $reportPath
    }
}

# Build optimized release version
Write-Host ""
Write-Host "Building optimized release version..." -ForegroundColor Green
cargo build --release

# Display summary
Write-Host ""
Write-Host "===========================================" -ForegroundColor Cyan
Write-Host "         Performance Analysis Complete      " -ForegroundColor Cyan
Write-Host "===========================================" -ForegroundColor Cyan

if ($All -or $RunBenchmarks) {
    Write-Host "✓ Benchmarks completed" -ForegroundColor Green
    Write-Host "  View detailed results: target\criterion\report\index.html" -ForegroundColor Gray
}

if ($All -or $Profile) {
    if (Test-Path "flamegraph.svg") {
        Write-Host "✓ Flamegraph generated: flamegraph.svg" -ForegroundColor Green
    }
}

if ($All -or $GenerateReport) {
    Write-Host "✓ Performance report generated: performance_report.md" -ForegroundColor Green
}

Write-Host ""
Write-Host "Optimization Tips:" -ForegroundColor Yellow
Write-Host "- Use 'cargo bench' to run benchmarks" -ForegroundColor Gray
Write-Host "- Use 'cargo build --release' for optimized builds" -ForegroundColor Gray
Write-Host "- Profile with 'cargo flamegraph' (install: cargo install flamegraph)" -ForegroundColor Gray
Write-Host "- Monitor real-time metrics in the application" -ForegroundColor Gray

Write-Host ""
Write-Host "Run with -All flag to execute all analysis steps" -ForegroundColor Cyan