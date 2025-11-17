#!/usr/bin/env bash
# Comprehensive test runner for WASM audio application
# Runs all unit tests, E2E tests, and benchmarks

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test results tracking
TESTS_PASSED=0
TESTS_FAILED=0
TEST_LOG="test-results-$(date +%Y%m%d-%H%M%S).log"

# Function to print colored output
print_status() {
    local status=$1
    local message=$2

    case $status in
        "INFO")
            echo -e "${BLUE}[INFO]${NC} $message"
            ;;
        "SUCCESS")
            echo -e "${GREEN}[✓]${NC} $message"
            TESTS_PASSED=$((TESTS_PASSED + 1))
            ;;
        "FAIL")
            echo -e "${RED}[✗]${NC} $message"
            TESTS_FAILED=$((TESTS_FAILED + 1))
            ;;
        "WARN")
            echo -e "${YELLOW}[!]${NC} $message"
            ;;
    esac

    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [$status] $message" >> "$TEST_LOG"
}

# Function to run a test command
run_test() {
    local test_name=$1
    local test_command=$2

    print_status "INFO" "Running: $test_name"

    if eval "$test_command" >> "$TEST_LOG" 2>&1; then
        print_status "SUCCESS" "$test_name"
        return 0
    else
        print_status "FAIL" "$test_name"
        return 1
    fi
}

# Print header
echo ""
echo "======================================================================="
echo "  WASM Audio Application - Comprehensive Test Suite"
echo "======================================================================="
echo ""
print_status "INFO" "Log file: $TEST_LOG"
echo ""

# ============================================================================
# 1. Environment Setup
# ============================================================================
print_status "INFO" "Checking environment..."

# Check Rust
if ! command -v rustc &> /dev/null; then
    print_status "FAIL" "Rust not installed"
    exit 1
fi
print_status "SUCCESS" "Rust installed: $(rustc --version)"

# Check wasm-pack
if ! command -v wasm-pack &> /dev/null; then
    print_status "WARN" "wasm-pack not installed, installing..."
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi
print_status "SUCCESS" "wasm-pack installed: $(wasm-pack --version)"

# Check Node.js
if ! command -v node &> /dev/null; then
    print_status "FAIL" "Node.js not installed"
    exit 1
fi
print_status "SUCCESS" "Node.js installed: $(node --version)"

echo ""

# ============================================================================
# 2. WASM Unit Tests
# ============================================================================
echo "======================================================================="
echo "  WASM Unit Tests"
echo "======================================================================="
echo ""

# Check for Chrome/Chromium
if command -v google-chrome &> /dev/null || command -v chromium &> /dev/null; then
    BROWSER="chrome"
    print_status "INFO" "Using Chrome for tests"
elif command -v firefox &> /dev/null; then
    BROWSER="firefox"
    print_status "INFO" "Using Firefox for tests"
else
    print_status "WARN" "No suitable browser found, skipping WASM unit tests"
    BROWSER="none"
fi

if [ "$BROWSER" != "none" ]; then
    # WorkerPool tests
    run_test "WASM WorkerPool Tests" \
        "wasm-pack test --headless --$BROWSER tests/wasm_worker_pool_tests.rs"

    # SharedAudioBuffer tests
    run_test "WASM SharedAudioBuffer Tests" \
        "wasm-pack test --headless --$BROWSER tests/wasm_shared_audio_buffer_tests.rs"

    # AudioContext tests
    run_test "WASM AudioContext Tests" \
        "wasm-pack test --headless --$BROWSER tests/wasm_audio_context_tests.rs"

    # Panic Boundary tests
    run_test "WASM Panic Boundary Tests" \
        "wasm-pack test --headless --$BROWSER tests/wasm_panic_boundary_tests.rs"

    # Memory Management tests
    run_test "WASM Memory Management Tests" \
        "wasm-pack test --headless --$BROWSER tests/wasm_memory_management_tests.rs"
fi

echo ""

# ============================================================================
# 3. Native Unit Tests
# ============================================================================
echo "======================================================================="
echo "  Native Unit Tests"
echo "======================================================================="
echo ""

run_test "Native Unit Tests" \
    "cargo test --lib --bins"

echo ""

# ============================================================================
# 4. Integration Tests
# ============================================================================
echo "======================================================================="
echo "  Integration Tests"
echo "======================================================================="
echo ""

run_test "Integration Tests" \
    "cargo test --test '*'"

echo ""

# ============================================================================
# 5. Build WASM for E2E Tests
# ============================================================================
echo "======================================================================="
echo "  Build WASM"
echo "======================================================================="
echo ""

run_test "WASM Build (debug)" \
    "wasm-pack build --target web --dev --out-dir static/pkg"

run_test "WASM Build (release)" \
    "wasm-pack build --target web --release --out-dir pkg"

# Check WASM size
if [ -f "pkg/rusty_audio_bg.wasm" ]; then
    WASM_SIZE=$(stat -c%s pkg/rusty_audio_bg.wasm 2>/dev/null || stat -f%z pkg/rusty_audio_bg.wasm 2>/dev/null)
    WASM_SIZE_MB=$((WASM_SIZE / 1024 / 1024))
    print_status "INFO" "WASM size: ${WASM_SIZE_MB} MB"

    if [ $WASM_SIZE_MB -gt 10 ]; then
        print_status "WARN" "WASM binary is large (${WASM_SIZE_MB} MB)"
    fi
fi

echo ""

# ============================================================================
# 6. E2E Tests
# ============================================================================
echo "======================================================================="
echo "  E2E Tests (Playwright)"
echo "======================================================================="
echo ""

cd tests

# Install dependencies if needed
if [ ! -d "node_modules" ]; then
    print_status "INFO" "Installing npm dependencies..."
    npm ci
fi

# Install Playwright browsers if needed
if [ ! -d "node_modules/@playwright" ]; then
    print_status "INFO" "Installing Playwright browsers..."
    npx playwright install --with-deps
fi

# Start test server in background
print_status "INFO" "Starting test server..."
python3 -m http.server 8080 --directory ../static > /dev/null 2>&1 &
SERVER_PID=$!
sleep 3

# Verify server is running
if ! curl -s http://localhost:8080 > /dev/null; then
    print_status "FAIL" "Test server failed to start"
    kill $SERVER_PID 2>/dev/null
    cd ..
    exit 1
fi

print_status "SUCCESS" "Test server running (PID: $SERVER_PID)"

# Run E2E tests
run_test "E2E: WASM Loading Tests" \
    "npm test -- wasm-loading.spec.ts"

run_test "E2E: Multithreading Tests" \
    "npm test -- multithreading.spec.ts"

run_test "E2E: Audio Functionality Tests" \
    "npm test -- audio-functionality.spec.ts"

run_test "E2E: UI Rendering Tests" \
    "npm test -- ui-rendering.spec.ts"

run_test "E2E: Performance Tests" \
    "npm test -- performance.spec.ts"

# Stop test server
print_status "INFO" "Stopping test server..."
kill $SERVER_PID 2>/dev/null

cd ..

echo ""

# ============================================================================
# 7. Linting and Formatting
# ============================================================================
echo "======================================================================="
echo "  Code Quality Checks"
echo "======================================================================="
echo ""

run_test "Cargo Format Check" \
    "cargo fmt --all -- --check"

run_test "Clippy Lint" \
    "cargo clippy --all-targets --all-features -- -D warnings"

echo ""

# ============================================================================
# 8. Security Audit
# ============================================================================
echo "======================================================================="
echo "  Security Audit"
echo "======================================================================="
echo ""

if command -v cargo-audit &> /dev/null; then
    run_test "Cargo Security Audit" \
        "cargo audit"
else
    print_status "WARN" "cargo-audit not installed, skipping security audit"
fi

echo ""

# ============================================================================
# 9. Performance Benchmarks (optional)
# ============================================================================
if [ "${RUN_BENCHMARKS:-false}" = "true" ]; then
    echo "======================================================================="
    echo "  Performance Benchmarks"
    echo "======================================================================="
    echo ""

    run_test "Build Benchmarks" \
        "cargo bench --no-run"

    print_status "INFO" "Benchmarks built (run 'cargo bench' to execute)"
    echo ""
fi

# ============================================================================
# Test Summary
# ============================================================================
echo ""
echo "======================================================================="
echo "  Test Summary"
echo "======================================================================="
echo ""

TOTAL_TESTS=$((TESTS_PASSED + TESTS_FAILED))

echo "Total tests: $TOTAL_TESTS"
echo -e "${GREEN}Passed: $TESTS_PASSED${NC}"
echo -e "${RED}Failed: $TESTS_FAILED${NC}"
echo ""
echo "Full log: $TEST_LOG"
echo ""

# Exit with error if any tests failed
if [ $TESTS_FAILED -gt 0 ]; then
    print_status "FAIL" "Some tests failed"
    echo ""
    echo "To view failures, check: $TEST_LOG"
    exit 1
else
    print_status "SUCCESS" "All tests passed!"
    exit 0
fi
