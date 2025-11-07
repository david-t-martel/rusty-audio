# Rusty Audio - Test Automation Script for Windows
# Comprehensive testing script for the car stereo-style interface

param(
    [Parameter(Position=0)]
    [ValidateSet("quick", "comprehensive", "ui", "visual", "audio", "all", "manual")]
    [string]$TestMode = "quick",

    [switch]$UpdateBaselines,
    [switch]$Verbose,
    [switch]$GenerateReport,
    [string]$OutputDir = "test_results"
)

# Set up colors for output
$ErrorActionPreference = "Stop"

function Write-Header {
    param([string]$Text)
    Write-Host ""
    Write-Host "🎵 $Text" -ForegroundColor Cyan
    Write-Host ("=" * ($Text.Length + 3)) -ForegroundColor Cyan
}

function Write-Success {
    param([string]$Text)
    Write-Host "✅ $Text" -ForegroundColor Green
}

function Write-Warning {
    param([string]$Text)
    Write-Host "⚠️  $Text" -ForegroundColor Yellow
}

function Write-Error {
    param([string]$Text)
    Write-Host "❌ $Text" -ForegroundColor Red
}

function Write-Info {
    param([string]$Text)
    Write-Host "ℹ️  $Text" -ForegroundColor Blue
}

function Test-Prerequisites {
    Write-Header "Checking Prerequisites"

    # Check Rust installation
    try {
        $rustVersion = rustc --version
        Write-Success "Rust: $rustVersion"
    }
    catch {
        Write-Error "Rust is not installed or not in PATH"
        exit 1
    }

    # Check Cargo
    try {
        $cargoVersion = cargo --version
        Write-Success "Cargo: $cargoVersion"
    }
    catch {
        Write-Error "Cargo is not available"
        exit 1
    }

    # Check if we're in the right directory
    if (Test-Path "Cargo.toml") {
        Write-Success "Found Cargo.toml - in correct project directory"
    }
    else {
        Write-Error "No Cargo.toml found - please run from project root"
        exit 1
    }

    # Check for test data directory
    if (-not (Test-Path $OutputDir)) {
        New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null
        Write-Info "Created output directory: $OutputDir"
    }

    Write-Success "All prerequisites met"
}

function Invoke-CargoTest {
    param(
        [string]$TestName,
        [string[]]$Args,
        [int]$TimeoutSeconds = 300
    )

    Write-Host ""
    Write-Host "🔧 Running $TestName..." -ForegroundColor Yellow

    $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()

    try {
        if ($Verbose) {
            $result = cargo @Args
        }
        else {
            $result = cargo @Args 2>&1
        }

        $exitCode = $LASTEXITCODE
        $duration = $stopwatch.Elapsed.TotalSeconds

        if ($exitCode -eq 0) {
            Write-Success "$TestName completed in $([math]::Round($duration, 2))s"
            return $true
        }
        else {
            Write-Error "$TestName failed after $([math]::Round($duration, 2))s"
            if (-not $Verbose -and $result) {
                Write-Host "Error output:" -ForegroundColor Red
                $result | Select-Object -Last 10 | ForEach-Object { Write-Host "  $_" -ForegroundColor Red }
            }
            return $false
        }
    }
    catch {
        $duration = $stopwatch.Elapsed.TotalSeconds
        Write-Error "$TestName failed with exception after $([math]::Round($duration, 2))s: $($_.Exception.Message)"
        return $false
    }
    finally {
        $stopwatch.Stop()
    }
}

function Run-QuickTests {
    Write-Header "Quick Test Suite"

    $results = @()

    # Core unit tests
    $results += Invoke-CargoTest "Unit Tests" @("test", "--lib", "--quiet")

    # Quick UI tests
    Write-Host ""
    Write-Host "🎨 Running UI Tests..." -ForegroundColor Yellow
    try {
        cargo run --bin test_runner -- quick
        $results += ($LASTEXITCODE -eq 0)
    }
    catch {
        Write-Error "UI tests failed: $($_.Exception.Message)"
        $results += $false
    }

    # Code quality checks
    $results += Invoke-CargoTest "Clippy Lints" @("clippy", "--all-targets", "--all-features", "--", "-D", "warnings")
    $results += Invoke-CargoTest "Format Check" @("fmt", "--all", "--", "--check")

    return ($results | Where-Object { -not $_ }).Count -eq 0
}

function Run-ComprehensiveTests {
    Write-Header "Comprehensive Test Suite"

    $results = @()

    # All unit and integration tests
    $results += Invoke-CargoTest "All Tests" @("test", "--all", "--", "--nocapture")

    # Comprehensive UI tests
    Write-Host ""
    Write-Host "🎨 Running Comprehensive UI Tests..." -ForegroundColor Yellow
    try {
        cargo run --bin test_runner -- comprehensive
        $results += ($LASTEXITCODE -eq 0)
    }
    catch {
        Write-Error "Comprehensive UI tests failed: $($_.Exception.Message)"
        $results += $false
    }

    # Performance benchmarks
    if (Get-Command "cargo" -ErrorAction SilentlyContinue) {
        $results += Invoke-CargoTest "Performance Benchmarks" @("bench", "--", "--output-format", "pretty") -TimeoutSeconds 600
    }

    # Code quality
    $results += Invoke-CargoTest "Clippy Lints" @("clippy", "--all-targets", "--all-features", "--", "-D", "warnings")
    $results += Invoke-CargoTest "Format Check" @("fmt", "--all", "--", "--check")
    $results += Invoke-CargoTest "Documentation Tests" @("test", "--doc")

    return ($results | Where-Object { -not $_ }).Count -eq 0
}

function Run-UITests {
    Write-Header "UI Component Tests"

    Write-Host "🎨 Running UI-specific tests..." -ForegroundColor Yellow
    try {
        cargo run --bin test_runner -- ui
        return ($LASTEXITCODE -eq 0)
    }
    catch {
        Write-Error "UI tests failed: $($_.Exception.Message)"
        return $false
    }
}

function Run-VisualTests {
    Write-Header "Visual Regression Tests"

    if ($UpdateBaselines) {
        Write-Info "Visual baselines will be updated"
    }

    Write-Host "📸 Running visual regression tests..." -ForegroundColor Yellow
    try {
        $args = @("run", "--bin", "test_runner", "--", "visual")
        if ($UpdateBaselines) {
            $args += "--update-baselines"
        }

        cargo @args
        return ($LASTEXITCODE -eq 0)
    }
    catch {
        Write-Error "Visual tests failed: $($_.Exception.Message)"
        return $false
    }
}

function Run-AudioTests {
    Write-Header "Audio Feature Tests"

    Write-Host "🎵 Running audio feature tests..." -ForegroundColor Yellow
    try {
        cargo run --bin test_runner -- audio
        return ($LASTEXITCODE -eq 0)
    }
    catch {
        Write-Error "Audio tests failed: $($_.Exception.Message)"
        return $false
    }
}

function Run-AllTests {
    Write-Header "Complete Test Suite"

    Write-Host "🎯 Running all test suites..." -ForegroundColor Yellow
    try {
        cargo run --bin test_runner -- all
        return ($LASTEXITCODE -eq 0)
    }
    catch {
        Write-Error "Complete test suite failed: $($_.Exception.Message)"
        return $false
    }
}

function Show-ManualTestingGuidance {
    Write-Header "Manual Testing Guidance"

    Write-Info "For comprehensive testing, please also run manual tests:"
    Write-Host ""
    Write-Host "📄 Review: TESTING_PROCEDURES.md" -ForegroundColor Cyan
    Write-Host "📄 Criteria: VALIDATION_CRITERIA.md" -ForegroundColor Cyan
    Write-Host ""

    Write-Host "Key manual test areas:" -ForegroundColor Yellow
    Write-Host "1. 🖥️  HiDPI scaling on different monitors (125%, 150%, 200%)" -ForegroundColor White
    Write-Host "2. 🎵 Audio file loading and playback (MP3, WAV, FLAC)" -ForegroundColor White
    Write-Host "3. 🎚️  Equalizer and effects functionality" -ForegroundColor White
    Write-Host "4. ⌨️  Keyboard navigation and accessibility" -ForegroundColor White
    Write-Host "5. 🖱️  Mouse and touch interactions" -ForegroundColor White
    Write-Host "6. 🎨 Car stereo interface aesthetics" -ForegroundColor White
    Write-Host ""

    Write-Host "Test on these configurations:" -ForegroundColor Yellow
    Write-Host "• Windows 10/11 with HiDPI display (1.25x scaling)" -ForegroundColor White
    Write-Host "• Multiple monitor setups with different DPI" -ForegroundColor White
    Write-Host "• Various screen resolutions (1920x1080, 2560x1440, 3440x1440)" -ForegroundColor White
    Write-Host "• Different audio devices and formats" -ForegroundColor White

    return $true
}

function Generate-TestReport {
    if (-not $GenerateReport) {
        return
    }

    Write-Header "Generating Test Report"

    $reportPath = Join-Path $OutputDir "test_report.html"
    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"

    $htmlContent = @"
<!DOCTYPE html>
<html>
<head>
    <title>Rusty Audio - Test Report</title>
    <style>
        body { font-family: 'Segoe UI', sans-serif; margin: 40px; }
        .header { background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; padding: 20px; border-radius: 8px; }
        .section { margin: 20px 0; padding: 15px; border: 1px solid #ddd; border-radius: 5px; }
        .success { color: #28a745; font-weight: bold; }
        .failure { color: #dc3545; font-weight: bold; }
    </style>
</head>
<body>
    <div class="header">
        <h1>🎵 Rusty Audio - Test Report</h1>
        <p>Car Stereo Style Interface - HiDPI Optimized</p>
        <p>Generated: $timestamp</p>
        <p>Test Mode: $TestMode</p>
    </div>

    <div class="section">
        <h2>📊 Test Summary</h2>
        <p>This report was generated automatically by the test automation script.</p>
        <p>For detailed results, check the console output and individual test logs.</p>
    </div>

    <div class="section">
        <h2>📋 Next Steps</h2>
        <ol>
            <li>Review any failed tests in console output</li>
            <li>Run manual testing procedures from TESTING_PROCEDURES.md</li>
            <li>Verify HiDPI scaling on target hardware</li>
            <li>Check validation criteria in VALIDATION_CRITERIA.md</li>
            <li>Test car stereo interface on actual automotive displays</li>
        </ol>
    </div>
</body>
</html>
"@

    $htmlContent | Out-File -FilePath $reportPath -Encoding UTF8
    Write-Success "Test report generated: $reportPath"
}

# Main execution
Write-Header "RUSTY AUDIO - TEST AUTOMATION SCRIPT"
Write-Host "Test Mode: $TestMode" -ForegroundColor Yellow
Write-Host "Output Directory: $OutputDir" -ForegroundColor Yellow
Write-Host "Update Baselines: $UpdateBaselines" -ForegroundColor Yellow
Write-Host "Verbose Output: $Verbose" -ForegroundColor Yellow

$startTime = Get-Date

# Check prerequisites
Test-Prerequisites

# Run the specified test mode
$success = switch ($TestMode) {
    "quick" { Run-QuickTests }
    "comprehensive" { Run-ComprehensiveTests }
    "ui" { Run-UITests }
    "visual" { Run-VisualTests }
    "audio" { Run-AudioTests }
    "all" { Run-AllTests }
    "manual" { Show-ManualTestingGuidance }
    default {
        Write-Error "Invalid test mode: $TestMode"
        exit 1
    }
}

$endTime = Get-Date
$duration = ($endTime - $startTime).TotalSeconds

# Generate report if requested
Generate-TestReport

# Final results
Write-Header "TEST AUTOMATION COMPLETE"
Write-Host "Total execution time: $([math]::Round($duration, 2)) seconds" -ForegroundColor Yellow

if ($success) {
    Write-Success "All tests completed successfully!"
    Write-Host ""
    Write-Info "Next steps:"
    Write-Host "1. Review the test output above for any warnings" -ForegroundColor White
    Write-Host "2. Run manual tests from TESTING_PROCEDURES.md" -ForegroundColor White
    Write-Host "3. Verify HiDPI behavior on actual hardware" -ForegroundColor White
    Write-Host "4. Check validation criteria compliance" -ForegroundColor White
}
else {
    Write-Error "Some tests failed!"
    Write-Host ""
    Write-Warning "Before proceeding:"
    Write-Host "1. Review the failed test output above" -ForegroundColor White
    Write-Host "2. Fix any identified issues" -ForegroundColor White
    Write-Host "3. Re-run the tests to verify fixes" -ForegroundColor White
    Write-Host "4. Ensure all validation criteria are met" -ForegroundColor White
    exit 1
}

Write-Host ""
Write-Host "🎵 Car stereo interface testing complete!" -ForegroundColor Magenta