# Playwright E2E Test Runner Script (PowerShell)
# Automates the complete test workflow: build WASM -> run tests -> generate report

param(
    [string]$TestType = "all",
    [switch]$Clean,
    [switch]$Help
)

$ErrorActionPreference = "Stop"

# Configuration
$WasmBuildCmd = "trunk build --release"
$TestDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent $TestDir

# Colors
function Write-Step {
    param([string]$Message)
    Write-Host "`n▶ $Message" -ForegroundColor Blue
}

function Write-Success {
    param([string]$Message)
    Write-Host "✓ $Message" -ForegroundColor Green
}

function Write-Error-Custom {
    param([string]$Message)
    Write-Host "✗ $Message" -ForegroundColor Red
}

function Write-Warning-Custom {
    param([string]$Message)
    Write-Host "⚠ $Message" -ForegroundColor Yellow
}

# Help
if ($Help) {
    Write-Host @"

Rusty Audio - Playwright E2E Test Runner

Usage:
    .\run-tests.ps1 [TestType] [-Clean] [-Help]

Test Types:
    all         - Run all tests (default)
    chromium    - Run Chromium tests only
    firefox     - Run Firefox tests only
    webkit      - Run WebKit tests only
    perf        - Run performance benchmarks only
    mobile      - Run mobile Chrome tests

Options:
    -Clean      - Clean previous builds before building
    -Help       - Show this help message

Examples:
    .\run-tests.ps1
    .\run-tests.ps1 chromium
    .\run-tests.ps1 perf
    .\run-tests.ps1 -Clean

"@
    exit 0
}

Write-Host @"

========================================
  Rusty Audio - Playwright E2E Tests
========================================

"@ -ForegroundColor Blue

# Step 1: Check prerequisites
Write-Step "Checking prerequisites..."

$prerequisites = @("cargo", "trunk", "node", "npm")

foreach ($cmd in $prerequisites) {
    if (-not (Get-Command $cmd -ErrorAction SilentlyContinue)) {
        Write-Error-Custom "$cmd not found. Please install it first."
        exit 1
    }
}

Write-Success "All prerequisites found"

# Step 2: Clean previous builds
if ($Clean) {
    Write-Step "Cleaning previous builds..."
    Set-Location $ProjectRoot

    if (Test-Path "dist") {
        Remove-Item -Recurse -Force "dist"
    }
    if (Test-Path "target\wasm32-unknown-unknown") {
        Remove-Item -Recurse -Force "target\wasm32-unknown-unknown"
    }

    Write-Success "Cleaned"
}

# Step 3: Build WASM
Write-Step "Building WASM application..."

Set-Location $ProjectRoot

# Set RUSTFLAGS for threading support
$env:RUSTFLAGS = "-C target-feature=+atomics,+bulk-memory,+mutable-globals"

Write-Host "Build command: $WasmBuildCmd"

try {
    Invoke-Expression $WasmBuildCmd
    Write-Success "WASM build successful"
}
catch {
    Write-Error-Custom "WASM build failed"
    exit 1
}

# Verify WASM binary exists
$wasmFiles = Get-ChildItem -Path "dist" -Filter "*.wasm" -Recurse -ErrorAction SilentlyContinue

if ($wasmFiles) {
    $wasmSize = ($wasmFiles[0].Length / 1MB).ToString("0.00")
    Write-Success "WASM binary found (size: ${wasmSize}MB)"
}
else {
    Write-Error-Custom "WASM binary not found in dist/"
    exit 1
}

# Step 4: Install test dependencies
Write-Step "Installing test dependencies..."

Set-Location $TestDir

if (-not (Test-Path "node_modules")) {
    npm install
}
else {
    Write-Host "Dependencies already installed (use -Clean to reinstall)"
}

Write-Success "Dependencies ready"

# Step 5: Install Playwright browsers
Write-Step "Checking Playwright browsers..."

$playwrightCache = "$env:USERPROFILE\AppData\Local\ms-playwright"

if (-not (Test-Path $playwrightCache)) {
    Write-Host "Installing Playwright browsers..."
    npm run install-browsers
}
else {
    Write-Host "Browsers already installed"
}

Write-Success "Browsers ready"

# Step 6: Run tests
Write-Step "Running Playwright tests..."

$testExitCode = 0

try {
    switch ($TestType.ToLower()) {
        "all" {
            Write-Host "Running all tests..."
            npm test
        }
        "perf" {
            Write-Host "Running performance benchmarks..."
            npm run test:performance
        }
        "performance" {
            Write-Host "Running performance benchmarks..."
            npm run test:performance
        }
        "chromium" {
            Write-Host "Running Chromium tests..."
            npm run test:chromium
        }
        "chrome" {
            Write-Host "Running Chromium tests..."
            npm run test:chromium
        }
        "firefox" {
            Write-Host "Running Firefox tests..."
            npm run test:firefox
        }
        "webkit" {
            Write-Host "Running WebKit tests..."
            npm run test:webkit
        }
        "safari" {
            Write-Host "Running WebKit tests..."
            npm run test:webkit
        }
        "mobile" {
            Write-Host "Running mobile Chrome tests..."
            npm run test:mobile
        }
        default {
            Write-Host "Running custom tests: $TestType"
            npx playwright test $TestType
        }
    }
}
catch {
    $testExitCode = 1
}

# Step 7: Generate report
Write-Step "Generating test report..."

if (Test-Path "playwright-report") {
    Write-Success "Report available: playwright-report\"
    Write-Host "  View with: npm run report"
}

# Step 8: Display summary
Write-Step "Test Summary"

if (Test-Path "playwright-report\summary.json") {
    Get-Content "playwright-report\summary.json" | Write-Host
}

# Performance summary
if (Test-Path "performance-data\performance-summary.json") {
    Write-Host "`nPerformance Summary:" -ForegroundColor Blue
    Get-Content "performance-data\performance-summary.json" | Select-Object -First 20 | Write-Host
}

# Final status
Write-Host "`n========================================" -ForegroundColor Blue

if ($testExitCode -eq 0) {
    Write-Success "All tests passed!"
    Write-Host "========================================`n" -ForegroundColor Blue
    exit 0
}
else {
    Write-Error-Custom "Some tests failed"
    Write-Host "========================================`n" -ForegroundColor Blue
    Write-Host "View detailed report with: npm run report"
    exit $testExitCode
}
