#!/usr/bin/env bash
# Desktop-specific benchmark runner with comprehensive profiling
# Supports: criterion benchmarks, flamegraph profiling, dhat heap analysis

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
BENCH_OUTPUT_DIR="${PROJECT_ROOT}/target/bench-results"
FLAMEGRAPH_DIR="${PROJECT_ROOT}/target/flamegraphs"
DHAT_OUTPUT_DIR="${PROJECT_ROOT}/target/dhat-profiles"
CRITERION_DIR="${PROJECT_ROOT}/target/criterion"

# Ensure output directories exist
mkdir -p "${BENCH_OUTPUT_DIR}" "${FLAMEGRAPH_DIR}" "${DHAT_OUTPUT_DIR}"

echo -e "${BLUE}=== Rusty Audio Desktop Benchmark Suite ===${NC}"
echo "Project: ${PROJECT_ROOT}"
echo "Results: ${BENCH_OUTPUT_DIR}"
echo ""

# Function to print section headers
print_header() {
    echo -e "\n${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${GREEN}  $1${NC}"
    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}\n"
}

# Function to run all criterion benchmarks
run_criterion_benchmarks() {
    print_header "Running Criterion Benchmarks"

    local benchmarks=(
        "audio_benchmarks"
        "performance_benchmarks"
        "simd_benchmarks"
        "optimization_benchmarks"
    )

    for bench in "${benchmarks[@]}"; do
        echo -e "${BLUE}Running: ${bench}${NC}"
        cargo bench --bench "${bench}" -- --color always

        # Check if benchmark succeeded
        if [ $? -eq 0 ]; then
            echo -e "${GREEN}✓ ${bench} completed${NC}"
        else
            echo -e "${RED}✗ ${bench} failed${NC}"
        fi
        echo ""
    done

    # Generate summary report
    echo -e "${YELLOW}Generating benchmark summary...${NC}"
    if [ -d "${CRITERION_DIR}" ]; then
        echo "Results saved to: ${CRITERION_DIR}"
        echo "Open ${CRITERION_DIR}/report/index.html for detailed results"
    fi
}

# Function to run flamegraph profiling
run_flamegraph_profiling() {
    print_header "Flamegraph Profiling"

    local target_benches=(
        "simd_benchmarks"           # Hot audio processing paths
        "optimization_benchmarks"    # Zero-copy pipeline hotspots
    )

    for bench in "${target_benches[@]}"; do
        echo -e "${BLUE}Profiling: ${bench}${NC}"

        local output_svg="${FLAMEGRAPH_DIR}/${bench}.svg"

        # Run flamegraph with benchmark
        cargo flamegraph \
            --bench "${bench}" \
            --output "${output_svg}" \
            -- --bench

        if [ -f "${output_svg}" ]; then
            echo -e "${GREEN}✓ Flamegraph saved: ${output_svg}${NC}"

            # Print top hotspots (if flamegraph succeeded)
            echo -e "${YELLOW}Top CPU hotspots:${NC}"
            echo "  Open ${output_svg} in a browser to analyze"
        else
            echo -e "${RED}✗ Flamegraph generation failed${NC}"
        fi
        echo ""
    done
}

# Function to run dhat heap profiling
run_dhat_profiling() {
    print_header "DHAT Heap Profiling"

    echo -e "${YELLOW}Note: dhat profiling requires code instrumentation${NC}"
    echo "Add dhat profiler to your benchmark code:"
    echo ""
    echo "  #[global_allocator]"
    echo "  static ALLOC: dhat::Alloc = dhat::Alloc;"
    echo ""
    echo "  fn main() {"
    echo "      let _profiler = dhat::Profiler::new_heap();"
    echo "      // Run benchmarks..."
    echo "  }"
    echo ""

    # Check if any benchmarks use dhat
    if grep -r "dhat::Profiler" "${PROJECT_ROOT}/benches/" >/dev/null 2>&1; then
        echo -e "${GREEN}Found dhat-instrumented benchmarks${NC}"

        # Run instrumented benchmarks
        DHAT_OUT_FILE="${DHAT_OUTPUT_DIR}/dhat-heap.json" cargo bench

        if [ -f "${DHAT_OUTPUT_DIR}/dhat-heap.json" ]; then
            echo -e "${GREEN}✓ DHAT profile saved: ${DHAT_OUTPUT_DIR}/dhat-heap.json${NC}"
            echo "  View with: firefox ${DHAT_OUTPUT_DIR}/dhat-heap.json"
        fi
    else
        echo -e "${YELLOW}No dhat-instrumented benchmarks found${NC}"
        echo "Skipping heap profiling"
    fi
}

# Function to run specific benchmark with custom options
run_custom_benchmark() {
    local bench_name="$1"
    local filter="${2:-}"

    print_header "Custom Benchmark: ${bench_name}"

    if [ -n "${filter}" ]; then
        echo -e "${BLUE}Filtering: ${filter}${NC}"
        cargo bench --bench "${bench_name}" -- "${filter}"
    else
        cargo bench --bench "${bench_name}"
    fi
}

# Function to compare benchmark results
compare_benchmarks() {
    print_header "Benchmark Comparison"

    if [ ! -d "${CRITERION_DIR}" ]; then
        echo -e "${RED}No criterion results found${NC}"
        return
    fi

    echo -e "${YELLOW}Comparing recent benchmark runs...${NC}"

    # Find recent benchmark result directories
    local recent_dirs=$(find "${CRITERION_DIR}" -mindepth 2 -maxdepth 2 -type d -name "base" | sort -r | head -5)

    if [ -z "${recent_dirs}" ]; then
        echo "No baseline comparisons available yet"
        return
    fi

    echo "Recent benchmarks:"
    echo "${recent_dirs}" | while read -r dir; do
        local bench_name=$(basename $(dirname "${dir}"))
        local timestamp=$(stat -c %y "${dir}" 2>/dev/null || stat -f %Sm "${dir}" 2>/dev/null)
        echo "  - ${bench_name} (${timestamp})"
    done
}

# Function to generate performance report
generate_performance_report() {
    print_header "Performance Report"

    local report_file="${BENCH_OUTPUT_DIR}/performance_report.txt"

    {
        echo "=== Rusty Audio Performance Report ==="
        echo "Generated: $(date)"
        echo ""
        echo "=== System Information ==="
        uname -a
        echo "CPU: $(lscpu | grep 'Model name' | cut -d ':' -f 2 | xargs)"
        echo "Cores: $(nproc)"
        echo "Memory: $(free -h | grep Mem | awk '{print $2}')"
        echo ""

        echo "=== Benchmark Results ==="
        if [ -d "${CRITERION_DIR}" ]; then
            echo "Latest criterion results:"
            find "${CRITERION_DIR}" -name "*.txt" -type f -mtime -1 | while read -r file; do
                echo "  - $(basename $(dirname "${file}"))"
            done
        fi
        echo ""

        echo "=== Profiling Outputs ==="
        echo "Flamegraphs: $(ls -1 "${FLAMEGRAPH_DIR}"/*.svg 2>/dev/null | wc -l) files"
        echo "DHAT profiles: $(ls -1 "${DHAT_OUTPUT_DIR}"/*.json 2>/dev/null | wc -l) files"
        echo ""

        echo "=== Performance Targets ==="
        echo "Audio callback latency: <10ms (target: <5ms)"
        echo "FFT processing: <2ms for 2048-point FFT"
        echo "EQ processing: <1ms for 8-band filter"
        echo "Spectrum smoothing: <0.5ms with SIMD"
        echo ""

    } > "${report_file}"

    cat "${report_file}"
    echo -e "${GREEN}Report saved: ${report_file}${NC}"
}

# Parse command line arguments
MODE="${1:-all}"

case "${MODE}" in
    "criterion"|"bench")
        run_criterion_benchmarks
        ;;
    "flamegraph"|"profile")
        run_flamegraph_profiling
        ;;
    "dhat"|"heap")
        run_dhat_profiling
        ;;
    "compare")
        compare_benchmarks
        ;;
    "report")
        generate_performance_report
        ;;
    "custom")
        if [ $# -lt 2 ]; then
            echo "Usage: $0 custom <benchmark_name> [filter]"
            exit 1
        fi
        run_custom_benchmark "$2" "${3:-}"
        ;;
    "all")
        run_criterion_benchmarks
        run_flamegraph_profiling
        run_dhat_profiling
        generate_performance_report
        ;;
    "help"|"-h"|"--help")
        echo "Usage: $0 [MODE]"
        echo ""
        echo "Modes:"
        echo "  criterion|bench   - Run all criterion benchmarks"
        echo "  flamegraph|profile - Generate flamegraph profiles"
        echo "  dhat|heap         - Run heap profiling with dhat"
        echo "  compare           - Compare recent benchmark results"
        echo "  report            - Generate performance report"
        echo "  custom <name> [filter] - Run specific benchmark with optional filter"
        echo "  all               - Run complete benchmark suite (default)"
        echo "  help              - Show this help message"
        echo ""
        echo "Examples:"
        echo "  $0 criterion              # Run all criterion benchmarks"
        echo "  $0 flamegraph             # Profile hotspots"
        echo "  $0 custom simd_benchmarks bench_biquad_filter"
        echo ""
        ;;
    *)
        echo -e "${RED}Unknown mode: ${MODE}${NC}"
        echo "Run '$0 help' for usage information"
        exit 1
        ;;
esac

echo ""
echo -e "${GREEN}=== Benchmark Suite Complete ===${NC}"
