#!/usr/bin/env bash
# Setup script for profiling and benchmarking infrastructure

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}=== Profiling Infrastructure Setup ===${NC}"
echo ""

# Track installation status
MISSING_TOOLS=()
OPTIONAL_TOOLS=()

# Check required Rust tools
check_rust_tool() {
    local tool_name="$1"
    local install_cmd="$2"
    local optional="${3:-false}"

    echo -ne "Checking ${tool_name}... "

    if command -v "${tool_name}" &>/dev/null; then
        echo -e "${GREEN}✓${NC}"
        return 0
    else
        echo -e "${YELLOW}✗ Not installed${NC}"

        if [ "${optional}" = "true" ]; then
            OPTIONAL_TOOLS+=("${tool_name}|${install_cmd}")
        else
            MISSING_TOOLS+=("${tool_name}|${install_cmd}")
        fi
        return 0  # Don't exit on missing tools
    fi
}

# Check system tools
check_system_tool() {
    local tool_name="$1"
    local install_cmd="$2"

    echo -ne "Checking ${tool_name}... "

    if command -v "${tool_name}" &>/dev/null; then
        echo -e "${GREEN}✓${NC}"
        return 0
    else
        echo -e "${YELLOW}✗ Not installed${NC}"
        OPTIONAL_TOOLS+=("${tool_name}|${install_cmd}")
        return 0  # Don't fail on optional tools
    fi
}

# Desktop profiling tools
echo -e "${BLUE}Desktop Profiling Tools:${NC}"
check_rust_tool "cargo-flamegraph" "cargo install flamegraph"
check_rust_tool "cargo-criterion" "cargo install cargo-criterion" true

# WASM tools
echo ""
echo -e "${BLUE}WASM Tools:${NC}"
check_rust_tool "wasm-pack" "cargo install wasm-pack" true
check_rust_tool "twiggy" "cargo install twiggy" true
check_system_tool "wasm-opt" "npm install -g wasm-opt OR install binaryen package"

# System profiling tools (Linux)
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    echo ""
    echo -e "${BLUE}Linux Profiling Tools:${NC}"
    check_system_tool "perf" "sudo apt-get install linux-tools-generic"
fi

# Check perf permissions (Linux)
if [[ "$OSTYPE" == "linux-gnu"* ]] && command -v perf &>/dev/null; then
    echo ""
    echo -e "${BLUE}Checking perf permissions:${NC}"

    PARANOID_LEVEL=$(cat /proc/sys/kernel/perf_event_paranoid 2>/dev/null || echo "unknown")
    echo -ne "perf_event_paranoid level: "

    case "${PARANOID_LEVEL}" in
        -1|0|1)
            echo -e "${GREEN}${PARANOID_LEVEL} (OK)${NC}"
            ;;
        2)
            echo -e "${YELLOW}${PARANOID_LEVEL} (restrictive)${NC}"
            echo "  To enable flamegraph profiling, run:"
            echo "  echo 0 | sudo tee /proc/sys/kernel/perf_event_paranoid"
            ;;
        *)
            echo -e "${RED}${PARANOID_LEVEL} (unknown)${NC}"
            ;;
    esac
fi

# Summary
echo ""
echo -e "${BLUE}=== Installation Summary ===${NC}"

if [ ${#MISSING_TOOLS[@]} -eq 0 ]; then
    echo -e "${GREEN}✓ All required tools are installed${NC}"
else
    echo -e "${YELLOW}Missing required tools:${NC}"
    for tool_spec in "${MISSING_TOOLS[@]}"; do
        IFS='|' read -r tool cmd <<< "${tool_spec}"
        echo "  - ${tool}"
        echo "    Install: ${cmd}"
    done
fi

if [ ${#OPTIONAL_TOOLS[@]} -gt 0 ]; then
    echo ""
    echo -e "${YELLOW}Optional tools (recommended):${NC}"
    for tool_spec in "${OPTIONAL_TOOLS[@]}"; do
        IFS='|' read -r tool cmd <<< "${tool_spec}"
        echo "  - ${tool}"
        echo "    Install: ${cmd}"
    done
fi

# Create directory structure
echo ""
echo -e "${BLUE}=== Creating Directory Structure ===${NC}"

DIRS=(
    "target/bench-results"
    "target/flamegraphs"
    "target/dhat-profiles"
    "target/wasm-bench"
    "target/wasm-profiles"
)

for dir in "${DIRS[@]}"; do
    if [ ! -d "${dir}" ]; then
        mkdir -p "${dir}"
        echo -e "  ${GREEN}✓${NC} Created ${dir}"
    else
        echo -e "  ${BLUE}•${NC} ${dir} (exists)"
    fi
done

# Verify benchmark files
echo ""
echo -e "${BLUE}=== Verifying Benchmark Files ===${NC}"

BENCH_FILES=(
    "benches/audio_benchmarks.rs"
    "benches/performance_benchmarks.rs"
    "benches/simd_benchmarks.rs"
    "benches/optimization_benchmarks.rs"
)

for bench_file in "${BENCH_FILES[@]}"; do
    if [ -f "${bench_file}" ]; then
        echo -e "  ${GREEN}✓${NC} ${bench_file}"
    else
        echo -e "  ${RED}✗${NC} ${bench_file} (missing)"
    fi
done

# Verify scripts
echo ""
echo -e "${BLUE}=== Verifying Scripts ===${NC}"

SCRIPTS=(
    "scripts/bench-desktop.sh"
    "scripts/bench-wasm.sh"
    "scripts/compare-benchmarks.sh"
)

for script in "${SCRIPTS[@]}"; do
    if [ -f "${script}" ]; then
        if [ -x "${script}" ]; then
            echo -e "  ${GREEN}✓${NC} ${script} (executable)"
        else
            echo -e "  ${YELLOW}!${NC} ${script} (not executable)"
            chmod +x "${script}"
            echo -e "    ${GREEN}→${NC} Made executable"
        fi
    else
        echo -e "  ${RED}✗${NC} ${script} (missing)"
    fi
done

# Test criterion setup
echo ""
echo -e "${BLUE}=== Testing Criterion Setup ===${NC}"

if cargo bench --no-run 2>&1 | grep -q "Finished"; then
    echo -e "${GREEN}✓ Criterion benchmarks compile successfully${NC}"
else
    echo -e "${YELLOW}! Benchmark compilation check failed (this may be normal)${NC}"
fi

# Installation script
if [ ${#MISSING_TOOLS[@]} -gt 0 ] || [ ${#OPTIONAL_TOOLS[@]} -gt 0 ]; then
    echo ""
    echo -e "${BLUE}=== Quick Install Commands ===${NC}"
    echo ""

    if [ ${#MISSING_TOOLS[@]} -gt 0 ]; then
        echo -e "${YELLOW}Required tools:${NC}"
        for tool_spec in "${MISSING_TOOLS[@]}"; do
            IFS='|' read -r tool cmd <<< "${tool_spec}"
            echo "  ${cmd}"
        done
        echo ""
    fi

    if [ ${#OPTIONAL_TOOLS[@]} -gt 0 ]; then
        echo -e "${YELLOW}Optional tools:${NC}"
        for tool_spec in "${OPTIONAL_TOOLS[@]}"; do
            IFS='|' read -r tool cmd <<< "${tool_spec}"
            echo "  ${cmd}"
        done
        echo ""
    fi

    echo "Install all at once:"
    echo ""
    echo -e "${GREEN}# Required${NC}"
    for tool_spec in "${MISSING_TOOLS[@]}"; do
        IFS='|' read -r tool cmd <<< "${tool_spec}"
        echo "${cmd}"
    done
    echo ""
    echo -e "${GREEN}# Optional${NC}"
    for tool_spec in "${OPTIONAL_TOOLS[@]}"; do
        IFS='|' read -r tool cmd <<< "${tool_spec}"
        if [[ "${cmd}" == cargo* ]]; then
            echo "${cmd}"
        fi
    done
fi

# Next steps
echo ""
echo -e "${GREEN}=== Setup Complete ===${NC}"
echo ""
echo "Next steps:"
echo "  1. Install any missing tools (see above)"
echo "  2. Run desktop benchmarks: ./scripts/bench-desktop.sh all"
echo "  3. Run WASM benchmarks: ./scripts/bench-wasm.sh all"
echo "  4. Read profiling guide: docs/PROFILING_GUIDE.md"
echo ""
echo "Quick reference:"
echo "  Desktop benchmarks:  ./scripts/bench-desktop.sh criterion"
echo "  Flamegraph profiling: ./scripts/bench-desktop.sh flamegraph"
echo "  WASM build & analyze: ./scripts/bench-wasm.sh all"
echo "  Compare benchmarks:   ./scripts/compare-benchmarks.sh save baseline-name"
echo ""
