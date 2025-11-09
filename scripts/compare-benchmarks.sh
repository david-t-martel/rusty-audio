#!/usr/bin/env bash
# Benchmark comparison tool for before/after optimization analysis

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

CRITERION_DIR="${PROJECT_ROOT}/target/criterion"

usage() {
    cat << EOF
Benchmark Comparison Tool

Usage:
  $0 save <baseline-name>        - Save current benchmarks as baseline
  $0 compare <baseline-name>     - Compare current vs baseline
  $0 list                        - List available baselines
  $0 report <baseline-name>      - Generate detailed comparison report
  $0 clean                       - Remove old baselines

Examples:
  $0 save before-simd-opt        # Save baseline before SIMD optimization
  $0 compare before-simd-opt     # Compare current vs baseline
  $0 report before-simd-opt      # Generate detailed report

EOF
}

# Save current benchmarks as baseline
save_baseline() {
    local baseline_name="$1"

    echo -e "${BLUE}Saving baseline: ${baseline_name}${NC}"

    # Run all benchmarks and save
    cargo bench -- --save-baseline "${baseline_name}"

    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✓ Baseline saved: ${baseline_name}${NC}"

        # Create metadata file
        local metadata_file="${CRITERION_DIR}/.baselines/${baseline_name}.meta"
        mkdir -p "$(dirname "${metadata_file}")"

        cat > "${metadata_file}" << EOF
Baseline: ${baseline_name}
Created: $(date)
Branch: $(git branch --show-current 2>/dev/null || echo "unknown")
Commit: $(git rev-parse --short HEAD 2>/dev/null || echo "unknown")
Description: Manual baseline for comparison
EOF

        echo "Metadata: ${metadata_file}"
    else
        echo -e "${RED}✗ Failed to save baseline${NC}"
        exit 1
    fi
}

# Compare current benchmarks against baseline
compare_baseline() {
    local baseline_name="$1"

    echo -e "${BLUE}Comparing against baseline: ${baseline_name}${NC}"

    # Check if baseline exists
    if [ ! -d "${CRITERION_DIR}/.baselines/${baseline_name}" ]; then
        echo -e "${RED}Error: Baseline '${baseline_name}' not found${NC}"
        echo "Available baselines:"
        list_baselines
        exit 1
    fi

    # Run benchmarks with comparison
    cargo bench -- --baseline "${baseline_name}"

    echo ""
    echo -e "${GREEN}Comparison complete${NC}"
    echo "View HTML report: ${CRITERION_DIR}/report/index.html"
}

# List available baselines
list_baselines() {
    local baselines_dir="${CRITERION_DIR}/.baselines"

    if [ ! -d "${baselines_dir}" ]; then
        echo "No baselines found"
        return
    fi

    echo -e "${BLUE}Available baselines:${NC}"
    echo ""

    find "${baselines_dir}" -maxdepth 1 -type d ! -path "${baselines_dir}" | while read -r baseline_dir; do
        local baseline_name=$(basename "${baseline_dir}")
        local meta_file="${baseline_dir}.meta"

        echo -e "${GREEN}${baseline_name}${NC}"

        if [ -f "${meta_file}" ]; then
            grep -E "Created|Branch|Commit" "${meta_file}" | sed 's/^/  /'
        fi

        echo ""
    done
}

# Generate detailed comparison report
generate_report() {
    local baseline_name="$1"
    local report_file="${PROJECT_ROOT}/target/bench-results/comparison_${baseline_name}.md"

    echo -e "${BLUE}Generating comparison report: ${baseline_name}${NC}"

    mkdir -p "$(dirname "${report_file}")"

    {
        echo "# Benchmark Comparison Report"
        echo ""
        echo "**Baseline:** ${baseline_name}"
        echo "**Generated:** $(date)"
        echo "**Branch:** $(git branch --show-current 2>/dev/null || echo 'unknown')"
        echo "**Commit:** $(git rev-parse --short HEAD 2>/dev/null || echo 'unknown')"
        echo ""

        # Check for baseline metadata
        local meta_file="${CRITERION_DIR}/.baselines/${baseline_name}.meta"
        if [ -f "${meta_file}" ]; then
            echo "## Baseline Information"
            echo ""
            echo '```'
            cat "${meta_file}"
            echo '```'
            echo ""
        fi

        echo "## Summary"
        echo ""

        # Parse criterion results for summary
        if [ -d "${CRITERION_DIR}" ]; then
            echo "### Performance Changes"
            echo ""
            echo "| Benchmark | Change | Significance |"
            echo "|-----------|--------|--------------|"

            # Find recent comparison results
            # Note: This is a simplified parser - real implementation would parse JSON
            find "${CRITERION_DIR}" -name "change" -type d -mtime -1 | head -10 | while read -r change_dir; do
                local bench_name=$(basename $(dirname $(dirname "${change_dir}")))
                echo "| ${bench_name} | See HTML report | - |"
            done

            echo ""
        fi

        echo "## Detailed Results"
        echo ""
        echo "View complete results in HTML report:"
        echo "${CRITERION_DIR}/report/index.html"
        echo ""

        echo "## Recommendations"
        echo ""
        echo "- Review flamegraph for hotspots"
        echo "- Check DHAT for memory allocations"
        echo "- Verify improvements are statistically significant (p < 0.05)"
        echo "- Test on different hardware configurations"
        echo ""

    } > "${report_file}"

    cat "${report_file}"
    echo ""
    echo -e "${GREEN}✓ Report saved: ${report_file}${NC}"
}

# Clean old baselines
clean_baselines() {
    echo -e "${YELLOW}Cleaning old baselines...${NC}"

    local baselines_dir="${CRITERION_DIR}/.baselines"

    if [ ! -d "${baselines_dir}" ]; then
        echo "No baselines to clean"
        return
    fi

    # List baselines older than 30 days
    echo "Baselines older than 30 days:"
    find "${baselines_dir}" -maxdepth 1 -type d -mtime +30 ! -path "${baselines_dir}" | while read -r old_baseline; do
        local name=$(basename "${old_baseline}")
        echo "  - ${name}"
    done

    read -p "Delete these baselines? (y/N) " -n 1 -r
    echo

    if [[ $REPLY =~ ^[Yy]$ ]]; then
        find "${baselines_dir}" -maxdepth 1 -type d -mtime +30 ! -path "${baselines_dir}" -exec rm -rf {} \;
        echo -e "${GREEN}✓ Old baselines removed${NC}"
    else
        echo "Cancelled"
    fi
}

# Main command dispatcher
case "${1:-help}" in
    "save")
        if [ $# -lt 2 ]; then
            echo "Error: Missing baseline name"
            usage
            exit 1
        fi
        save_baseline "$2"
        ;;
    "compare")
        if [ $# -lt 2 ]; then
            echo "Error: Missing baseline name"
            usage
            exit 1
        fi
        compare_baseline "$2"
        ;;
    "list")
        list_baselines
        ;;
    "report")
        if [ $# -lt 2 ]; then
            echo "Error: Missing baseline name"
            usage
            exit 1
        fi
        generate_report "$2"
        ;;
    "clean")
        clean_baselines
        ;;
    "help"|"-h"|"--help")
        usage
        ;;
    *)
        echo -e "${RED}Unknown command: $1${NC}"
        usage
        exit 1
        ;;
esac
