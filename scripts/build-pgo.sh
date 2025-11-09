#!/bin/bash
# Profile-Guided Optimization (PGO) Build Script for Linux/WSL
# This script performs a multi-stage build with PGO for maximum performance
#
# Usage:
#   ./scripts/build-pgo.sh [--clean] [--skip-workload] [--duration SECONDS]
#
# Process:
#   1. Build instrumented binary (collects profile data)
#   2. Run workload to generate profile data
#   3. Build optimized binary using profile data
#
# Expected performance gain: 10-15% improvement in hot paths

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Defaults
CLEAN=false
SKIP_WORKLOAD=false
WORKLOAD_DURATION=60

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --clean)
            CLEAN=true
            shift
            ;;
        --skip-workload)
            SKIP_WORKLOAD=true
            shift
            ;;
        --duration)
            WORKLOAD_DURATION="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--clean] [--skip-workload] [--duration SECONDS]"
            exit 1
            ;;
    esac
done

# Directories
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
PROFILE_DIR="$PROJECT_ROOT/pgo-data"
INSTRUMENTED_BINARY="$PROJECT_ROOT/target/pgo-instrument/rusty-audio"
OPTIMIZED_BINARY="$PROJECT_ROOT/target/pgo-use/rusty-audio"

echo -e "${CYAN}========================================${NC}"
echo -e "${CYAN}Profile-Guided Optimization Build${NC}"
echo -e "${CYAN}========================================${NC}"
echo ""

# Clean previous PGO data if requested
if [ "$CLEAN" = true ]; then
    echo -e "${YELLOW}[1/4] Cleaning previous PGO data...${NC}"
    rm -rf "$PROFILE_DIR"
    rm -rf "$PROJECT_ROOT/target/pgo-instrument"
    rm -rf "$PROJECT_ROOT/target/pgo-use"
    echo -e "${GREEN}  ✓ Cleaned PGO data${NC}"
fi

# Create profile directory
mkdir -p "$PROFILE_DIR"

# Step 1: Build instrumented binary
echo ""
echo -e "${YELLOW}[1/4] Building instrumented binary...${NC}"
cd "$PROJECT_ROOT"
RUSTFLAGS="-Cprofile-generate=$PROFILE_DIR" \
    cargo build --profile pgo-instrument --target-dir target
echo -e "${GREEN}  ✓ Instrumented binary created${NC}"

# Step 2: Run workload to collect profile data
if [ "$SKIP_WORKLOAD" = false ]; then
    echo ""
    echo -e "${YELLOW}[2/4] Running workload to collect profile data...${NC}"
    echo -e "${CYAN}  Duration: $WORKLOAD_DURATION seconds${NC}"
    echo -e "${CYAN}  Please use the application normally (load files, play audio, use EQ)${NC}"
    echo ""

    # Start the instrumented binary in background
    "$INSTRUMENTED_BINARY" &
    BINARY_PID=$!

    # Wait for specified duration or until user closes the app
    ELAPSED=0
    CHECK_INTERVAL=5
    while [ $ELAPSED -lt $WORKLOAD_DURATION ] && kill -0 $BINARY_PID 2>/dev/null; do
        sleep $CHECK_INTERVAL
        ELAPSED=$((ELAPSED + CHECK_INTERVAL))
        REMAINING=$((WORKLOAD_DURATION - ELAPSED))
        if [ $REMAINING -gt 0 ]; then
            echo -e "  ${NC}Time remaining: $REMAINING seconds...${NC}"
        fi
    done

    # Stop the process if still running
    if kill -0 $BINARY_PID 2>/dev/null; then
        echo -e "${YELLOW}  Stopping instrumented binary...${NC}"
        kill $BINARY_PID
        sleep 2
    fi

    # Check for profile data
    PROFILE_COUNT=$(find "$PROFILE_DIR" -name "*.profraw" | wc -l)
    if [ "$PROFILE_COUNT" -eq 0 ]; then
        echo -e "${RED}Warning: No profile data collected!${NC}"
        echo -e "${YELLOW}  The workload may have been too short.${NC}"
        echo -e "${YELLOW}  Run again with longer workload duration or manually use the app.${NC}"
        exit 1
    fi

    echo -e "${GREEN}  ✓ Collected $PROFILE_COUNT profile data files${NC}"
else
    echo ""
    echo -e "${YELLOW}[2/4] Skipping workload (using existing profile data)...${NC}"
fi

# Step 3: Merge profile data
echo ""
echo -e "${YELLOW}[3/4] Merging profile data...${NC}"

# Find llvm-profdata tool
LLVM_PROFDATA=""
if command -v llvm-profdata &> /dev/null; then
    LLVM_PROFDATA="llvm-profdata"
else
    # Try to find in Rust installation
    RUSTC_DIR="$(dirname "$(rustc --print sysroot)")"
    if [ -f "$RUSTC_DIR/lib/rustlib/x86_64-unknown-linux-gnu/bin/llvm-profdata" ]; then
        LLVM_PROFDATA="$RUSTC_DIR/lib/rustlib/x86_64-unknown-linux-gnu/bin/llvm-profdata"
    else
        echo -e "${RED}Error: llvm-profdata not found!${NC}"
        echo -e "${YELLOW}Install with: rustup component add llvm-tools-preview${NC}"
        exit 1
    fi
fi

MERGED_PROFILE="$PROFILE_DIR/merged.profdata"
"$LLVM_PROFDATA" merge -o "$MERGED_PROFILE" "$PROFILE_DIR"/*.profraw
echo -e "${GREEN}  ✓ Profile data merged to $MERGED_PROFILE${NC}"

# Step 4: Build optimized binary
echo ""
echo -e "${YELLOW}[4/4] Building PGO-optimized binary...${NC}"
cd "$PROJECT_ROOT"
RUSTFLAGS="-Cprofile-use=$MERGED_PROFILE -Cllvm-args=-pgo-warn-missing-function" \
    cargo build --profile pgo-use --target-dir target
echo -e "${GREEN}  ✓ PGO-optimized binary created${NC}"

# Summary
echo ""
echo -e "${CYAN}========================================${NC}"
echo -e "${GREEN}PGO Build Complete!${NC}"
echo -e "${CYAN}========================================${NC}"
echo ""
echo -e "${NC}Binaries:${NC}"
echo -e "  Instrumented: ${NC}$INSTRUMENTED_BINARY${NC}"
echo -e "  Optimized:    ${NC}$OPTIMIZED_BINARY${NC}"
echo ""
echo -e "${GREEN}Expected performance gain: 10-15% in hot paths${NC}"
echo ""
echo -e "${NC}To use the optimized binary:${NC}"
echo -e "  cp $OPTIMIZED_BINARY /usr/local/bin/rusty-audio"
echo ""
