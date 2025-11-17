#!/bin/bash

# Playwright E2E Test Runner Script
# Automates the complete test workflow: build WASM -> run tests -> generate report

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
WASM_BUILD_CMD="${WASM_BUILD_CMD:-trunk build --release}"
TEST_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$TEST_DIR")"

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  Rusty Audio - Playwright E2E Tests${NC}"
echo -e "${BLUE}========================================${NC}\n"

# Function: Print step header
step() {
    echo -e "\n${BLUE}▶ $1${NC}"
}

# Function: Check command exists
check_command() {
    if ! command -v "$1" &> /dev/null; then
        echo -e "${RED}✗ $1 not found. Please install it first.${NC}"
        exit 1
    fi
}

# Step 1: Verify prerequisites
step "Checking prerequisites..."

check_command "cargo"
check_command "trunk"
check_command "node"
check_command "npm"

echo -e "${GREEN}✓ All prerequisites found${NC}"

# Step 2: Clean previous builds (optional)
if [ "$CLEAN" = "1" ]; then
    step "Cleaning previous builds..."
    cd "$PROJECT_ROOT"
    rm -rf dist/ target/wasm32-unknown-unknown
    echo -e "${GREEN}✓ Cleaned${NC}"
fi

# Step 3: Build WASM
step "Building WASM application..."

cd "$PROJECT_ROOT"

# Set RUSTFLAGS for threading support
export RUSTFLAGS="-C target-feature=+atomics,+bulk-memory,+mutable-globals"

echo "Build command: $WASM_BUILD_CMD"
if $WASM_BUILD_CMD; then
    echo -e "${GREEN}✓ WASM build successful${NC}"
else
    echo -e "${RED}✗ WASM build failed${NC}"
    exit 1
fi

# Verify WASM binary exists
if [ -f "dist/rusty_audio_bg.wasm" ] || [ -f "dist/pkg/rusty_audio_bg.wasm" ]; then
    WASM_SIZE=$(find dist -name "*.wasm" -exec du -h {} + | head -1 | cut -f1)
    echo -e "${GREEN}✓ WASM binary found (size: $WASM_SIZE)${NC}"
else
    echo -e "${RED}✗ WASM binary not found in dist/${NC}"
    exit 1
fi

# Step 4: Install test dependencies
step "Installing test dependencies..."

cd "$TEST_DIR"

if [ ! -d "node_modules" ]; then
    npm install
else
    echo "Dependencies already installed (use CLEAN=1 to reinstall)"
fi

echo -e "${GREEN}✓ Dependencies ready${NC}"

# Step 5: Install Playwright browsers (if needed)
step "Checking Playwright browsers..."

if [ ! -d "$HOME/.cache/ms-playwright" ] && [ ! -d "$HOME/Library/Caches/ms-playwright" ]; then
    echo "Installing Playwright browsers..."
    npm run install-browsers
else
    echo "Browsers already installed"
fi

echo -e "${GREEN}✓ Browsers ready${NC}"

# Step 6: Run tests
step "Running Playwright tests..."

# Determine which tests to run
TEST_ARGS="${@}"

if [ -z "$TEST_ARGS" ]; then
    # No arguments - run all tests
    echo "Running all tests..."
    npm test
elif [ "$TEST_ARGS" = "perf" ] || [ "$TEST_ARGS" = "performance" ]; then
    # Performance tests only
    echo "Running performance benchmarks..."
    npm run test:performance
elif [ "$TEST_ARGS" = "chromium" ] || [ "$TEST_ARGS" = "chrome" ]; then
    # Chromium only
    echo "Running Chromium tests..."
    npm run test:chromium
elif [ "$TEST_ARGS" = "firefox" ]; then
    # Firefox only
    echo "Running Firefox tests..."
    npm run test:firefox
elif [ "$TEST_ARGS" = "webkit" ] || [ "$TEST_ARGS" = "safari" ]; then
    # WebKit only
    echo "Running WebKit tests..."
    npm run test:webkit
else
    # Custom arguments - pass through
    echo "Running custom tests: $TEST_ARGS"
    npx playwright test $TEST_ARGS
fi

TEST_EXIT_CODE=$?

# Step 7: Generate report
step "Generating test report..."

if [ -d "playwright-report" ]; then
    echo -e "${GREEN}✓ Report available: playwright-report/${NC}"
    echo "  View with: npm run report"
fi

# Step 8: Display summary
step "Test Summary"

if [ -f "playwright-report/summary.json" ]; then
    echo -e "\n$(cat playwright-report/summary.json)\n"
fi

# Performance summary
if [ -f "performance-data/performance-summary.json" ]; then
    echo -e "${BLUE}Performance Summary:${NC}"
    echo -e "$(cat performance-data/performance-summary.json | head -20)\n"
fi

# Final status
echo -e "${BLUE}========================================${NC}"

if [ $TEST_EXIT_CODE -eq 0 ]; then
    echo -e "${GREEN}✓ All tests passed!${NC}"
    echo -e "${BLUE}========================================${NC}\n"
    exit 0
else
    echo -e "${RED}✗ Some tests failed${NC}"
    echo -e "${BLUE}========================================${NC}\n"
    echo "View detailed report with: npm run report"
    exit $TEST_EXIT_CODE
fi
