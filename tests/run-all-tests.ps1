# Comprehensive test runner for WASM audio application (PowerShell)
# Runs all unit tests, E2E tests, and benchmarks

param(
    [switch]$RunBenchmarks = $false,
    [switch]$SkipE2E = $false,
    [string]$Browser = "chrome"
)

$ErrorActionPreference = "Continue"

# Test results tracking
$script:TestsPassed = 0
$script:TestsFailed = 0
$script:TestLog = "test-results-$(Get-Date -Format 'yyyyMMdd-HHmmss').log"

# Function to print colored output
function Write-Status {
    param(
        [string]$Status,
        [string]$Message
    )

    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"

    switch ($Status) {
        "INFO" {
            Write-Host "[INFO] " -ForegroundColor Blue -NoNewline
            Write-Host $Message
        }
        "SUCCESS" {
            Write-Host "[✓] " -ForegroundColor Green -NoNewline
            Write-Host $Message
            $script:TestsPassed++
        }
        "FAIL" {
            Write-Host "[✗] " -ForegroundColor Red -NoNewline
            Write-Host $Message
            $script:TestsFailed++
        }
        "WARN" {
            Write-Host "[!] " -ForegroundColor Yellow -NoNewline
            Write-Host $Message
        }
    }

    "[$timestamp] [$Status] $Message" | Out-File -Append -FilePath $script:TestLog
}

# Function to run a test command
function Invoke-Test {
    param(
        [string]$TestName,
        [scriptblock]$TestCommand
    )

    Write-Status "INFO" "Running: $TestName"

    try {
        $output = & $TestCommand 2>&1 | Out-String
        $output | Out-File -Append -FilePath $script:TestLog

        if ($LASTEXITCODE -eq 0) {
            Write-Status "SUCCESS" $TestName
            return $true
        }
        else {
            Write-Status "FAIL" "$TestName (exit code: $LASTEXITCODE)"
            return $false
        }
    }
    catch {
        Write-Status "FAIL" "$TestName (error: $_)"
        $_.Exception.Message | Out-File -Append -FilePath $script:TestLog
        return $false
    }
}

# Print header
Write-Host ""
Write-Host "=======================================================================" -ForegroundColor Cyan
Write-Host "  WASM Audio Application - Comprehensive Test Suite" -ForegroundColor Cyan
Write-Host "=======================================================================" -ForegroundColor Cyan
Write-Host ""
Write-Status "INFO" "Log file: $script:TestLog"
Write-Host ""

# ============================================================================
# 1. Environment Setup
# ============================================================================
Write-Status "INFO" "Checking environment..."

# Check Rust
if (-not (Get-Command rustc -ErrorAction SilentlyContinue)) {
    Write-Status "FAIL" "Rust not installed"
    exit 1
}
$rustVersion = rustc --version
Write-Status "SUCCESS" "Rust installed: $rustVersion"

# Check wasm-pack
if (-not (Get-Command wasm-pack -ErrorAction SilentlyContinue)) {
    Write-Status "WARN" "wasm-pack not installed, please install manually"
    Write-Status "INFO" "Install with: cargo install wasm-pack"
    exit 1
}
$wasmPackVersion = wasm-pack --version
Write-Status "SUCCESS" "wasm-pack installed: $wasmPackVersion"

# Check Node.js
if (-not (Get-Command node -ErrorAction SilentlyContinue)) {
    Write-Status "FAIL" "Node.js not installed"
    exit 1
}
$nodeVersion = node --version
Write-Status "SUCCESS" "Node.js installed: $nodeVersion"

Write-Host ""

# ============================================================================
# 2. WASM Unit Tests
# ============================================================================
Write-Host "=======================================================================" -ForegroundColor Cyan
Write-Host "  WASM Unit Tests" -ForegroundColor Cyan
Write-Host "=======================================================================" -ForegroundColor Cyan
Write-Host ""

# Check for browser
$browserAvailable = $false
if (Get-Command chrome -ErrorAction SilentlyContinue) {
    $browserAvailable = $true
    Write-Status "INFO" "Using Chrome for tests"
}
elseif (Get-Command firefox -ErrorAction SilentlyContinue) {
    $Browser = "firefox"
    $browserAvailable = $true
    Write-Status "INFO" "Using Firefox for tests"
}

if ($browserAvailable) {
    Invoke-Test "WASM WorkerPool Tests" {
        wasm-pack test --headless --$Browser tests/wasm_worker_pool_tests.rs
    }

    Invoke-Test "WASM SharedAudioBuffer Tests" {
        wasm-pack test --headless --$Browser tests/wasm_shared_audio_buffer_tests.rs
    }

    Invoke-Test "WASM AudioContext Tests" {
        wasm-pack test --headless --$Browser tests/wasm_audio_context_tests.rs
    }

    Invoke-Test "WASM Panic Boundary Tests" {
        wasm-pack test --headless --$Browser tests/wasm_panic_boundary_tests.rs
    }

    Invoke-Test "WASM Memory Management Tests" {
        wasm-pack test --headless --$Browser tests/wasm_memory_management_tests.rs
    }
}
else {
    Write-Status "WARN" "No suitable browser found, skipping WASM unit tests"
}

Write-Host ""

# ============================================================================
# 3. Native Unit Tests
# ============================================================================
Write-Host "=======================================================================" -ForegroundColor Cyan
Write-Host "  Native Unit Tests" -ForegroundColor Cyan
Write-Host "=======================================================================" -ForegroundColor Cyan
Write-Host ""

Invoke-Test "Native Unit Tests" {
    cargo test --lib --bins
}

Write-Host ""

# ============================================================================
# 4. Integration Tests
# ============================================================================
Write-Host "=======================================================================" -ForegroundColor Cyan
Write-Host "  Integration Tests" -ForegroundColor Cyan
Write-Host "=======================================================================" -ForegroundColor Cyan
Write-Host ""

Invoke-Test "Integration Tests" {
    cargo test --test '*'
}

Write-Host ""

# ============================================================================
# 5. Build WASM for E2E Tests
# ============================================================================
Write-Host "=======================================================================" -ForegroundColor Cyan
Write-Host "  Build WASM" -ForegroundColor Cyan
Write-Host "=======================================================================" -ForegroundColor Cyan
Write-Host ""

Invoke-Test "WASM Build (debug)" {
    wasm-pack build --target web --dev --out-dir static/pkg
}

Invoke-Test "WASM Build (release)" {
    wasm-pack build --target web --release --out-dir pkg
}

# Check WASM size
if (Test-Path "pkg/rusty_audio_bg.wasm") {
    $wasmSize = (Get-Item "pkg/rusty_audio_bg.wasm").Length
    $wasmSizeMB = [math]::Round($wasmSize / 1MB, 2)
    Write-Status "INFO" "WASM size: ${wasmSizeMB} MB"

    if ($wasmSizeMB -gt 10) {
        Write-Status "WARN" "WASM binary is large (${wasmSizeMB} MB)"
    }
}

Write-Host ""

# ============================================================================
# 6. E2E Tests
# ============================================================================
if (-not $SkipE2E) {
    Write-Host "=======================================================================" -ForegroundColor Cyan
    Write-Host "  E2E Tests (Playwright)" -ForegroundColor Cyan
    Write-Host "=======================================================================" -ForegroundColor Cyan
    Write-Host ""

    Push-Location tests

    # Install dependencies if needed
    if (-not (Test-Path "node_modules")) {
        Write-Status "INFO" "Installing npm dependencies..."
        npm ci
    }

    # Install Playwright browsers if needed
    if (-not (Test-Path "node_modules/@playwright")) {
        Write-Status "INFO" "Installing Playwright browsers..."
        npx playwright install --with-deps
    }

    # Start test server in background
    Write-Status "INFO" "Starting test server..."
    $serverJob = Start-Job -ScriptBlock {
        param($dir)
        Set-Location $dir
        python -m http.server 8080
    } -ArgumentList (Resolve-Path "../static")

    Start-Sleep -Seconds 3

    # Verify server is running
    try {
        $response = Invoke-WebRequest -Uri "http://localhost:8080" -UseBasicParsing -TimeoutSec 5
        Write-Status "SUCCESS" "Test server running (Job: $($serverJob.Id))"
    }
    catch {
        Write-Status "FAIL" "Test server failed to start"
        Stop-Job $serverJob
        Remove-Job $serverJob
        Pop-Location
        exit 1
    }

    # Run E2E tests
    Invoke-Test "E2E: WASM Loading Tests" {
        npm test -- wasm-loading.spec.ts
    }

    Invoke-Test "E2E: Multithreading Tests" {
        npm test -- multithreading.spec.ts
    }

    Invoke-Test "E2E: Audio Functionality Tests" {
        npm test -- audio-functionality.spec.ts
    }

    Invoke-Test "E2E: UI Rendering Tests" {
        npm test -- ui-rendering.spec.ts
    }

    Invoke-Test "E2E: Performance Tests" {
        npm test -- performance.spec.ts
    }

    # Stop test server
    Write-Status "INFO" "Stopping test server..."
    Stop-Job $serverJob
    Remove-Job $serverJob

    Pop-Location

    Write-Host ""
}

# ============================================================================
# 7. Linting and Formatting
# ============================================================================
Write-Host "=======================================================================" -ForegroundColor Cyan
Write-Host "  Code Quality Checks" -ForegroundColor Cyan
Write-Host "=======================================================================" -ForegroundColor Cyan
Write-Host ""

Invoke-Test "Cargo Format Check" {
    cargo fmt --all -- --check
}

Invoke-Test "Clippy Lint" {
    cargo clippy --all-targets --all-features -- -D warnings
}

Write-Host ""

# ============================================================================
# 8. Security Audit
# ============================================================================
Write-Host "=======================================================================" -ForegroundColor Cyan
Write-Host "  Security Audit" -ForegroundColor Cyan
Write-Host "=======================================================================" -ForegroundColor Cyan
Write-Host ""

if (Get-Command cargo-audit -ErrorAction SilentlyContinue) {
    Invoke-Test "Cargo Security Audit" {
        cargo audit
    }
}
else {
    Write-Status "WARN" "cargo-audit not installed, skipping security audit"
}

Write-Host ""

# ============================================================================
# 9. Performance Benchmarks (optional)
# ============================================================================
if ($RunBenchmarks) {
    Write-Host "=======================================================================" -ForegroundColor Cyan
    Write-Host "  Performance Benchmarks" -ForegroundColor Cyan
    Write-Host "=======================================================================" -ForegroundColor Cyan
    Write-Host ""

    Invoke-Test "Build Benchmarks" {
        cargo bench --no-run
    }

    Write-Status "INFO" "Benchmarks built (run 'cargo bench' to execute)"
    Write-Host ""
}

# ============================================================================
# Test Summary
# ============================================================================
Write-Host ""
Write-Host "=======================================================================" -ForegroundColor Cyan
Write-Host "  Test Summary" -ForegroundColor Cyan
Write-Host "=======================================================================" -ForegroundColor Cyan
Write-Host ""

$totalTests = $script:TestsPassed + $script:TestsFailed

Write-Host "Total tests: $totalTests"
Write-Host "Passed: " -NoNewline
Write-Host $script:TestsPassed -ForegroundColor Green
Write-Host "Failed: " -NoNewline
Write-Host $script:TestsFailed -ForegroundColor Red
Write-Host ""
Write-Host "Full log: $script:TestLog"
Write-Host ""

# Exit with error if any tests failed
if ($script:TestsFailed -gt 0) {
    Write-Status "FAIL" "Some tests failed"
    Write-Host ""
    Write-Host "To view failures, check: $script:TestLog"
    exit 1
}
else {
    Write-Status "SUCCESS" "All tests passed!"
    exit 0
}
