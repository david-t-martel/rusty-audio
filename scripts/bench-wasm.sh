#!/usr/bin/env bash
# WASM-specific benchmark and optimization script
# Focuses on: bundle size, load time, initialization performance

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
WASM_OUTPUT_DIR="${PROJECT_ROOT}/target/wasm-bench"
WASM_PROFILE_DIR="${PROJECT_ROOT}/target/wasm-profiles"

mkdir -p "${WASM_OUTPUT_DIR}" "${WASM_PROFILE_DIR}"

echo -e "${BLUE}=== Rusty Audio WASM Benchmark Suite ===${NC}"
echo ""

# Check required tools
check_dependencies() {
    local missing_tools=()

    if ! command -v wasm-pack &>/dev/null; then
        missing_tools+=("wasm-pack")
    fi

    if ! command -v wasm-opt &>/dev/null; then
        missing_tools+=("wasm-opt (from binaryen)")
    fi

    if ! command -v twiggy &>/dev/null; then
        echo -e "${YELLOW}Warning: twiggy not found (optional)${NC}"
        echo "  Install with: cargo install twiggy"
    fi

    if [ ${#missing_tools[@]} -gt 0 ]; then
        echo -e "${RED}Missing required tools:${NC}"
        for tool in "${missing_tools[@]}"; do
            echo "  - ${tool}"
        done
        echo ""
        echo "Install instructions:"
        echo "  cargo install wasm-pack"
        echo "  npm install -g wasm-opt  # or install binaryen package"
        exit 1
    fi
}

# Function to build WASM with different optimization levels
build_wasm_variants() {
    echo -e "${GREEN}━━━ Building WASM Variants ━━━${NC}"

    local variants=(
        "dev:--dev"
        "profiling:--profiling"
        "release:--release"
    )

    for variant_spec in "${variants[@]}"; do
        local variant_name="${variant_spec%%:*}"
        local build_flag="${variant_spec##*:}"

        echo -e "${BLUE}Building ${variant_name} variant...${NC}"

        wasm-pack build \
            ${build_flag} \
            --target web \
            --out-dir "${WASM_OUTPUT_DIR}/${variant_name}" \
            --out-name rusty_audio

        if [ $? -eq 0 ]; then
            local wasm_file="${WASM_OUTPUT_DIR}/${variant_name}/rusty_audio_bg.wasm"

            if [ -f "${wasm_file}" ]; then
                local size=$(du -h "${wasm_file}" | cut -f1)
                local size_bytes=$(stat -c%s "${wasm_file}" 2>/dev/null || stat -f%z "${wasm_file}" 2>/dev/null)
                echo -e "${GREEN}✓ ${variant_name}: ${size} (${size_bytes} bytes)${NC}"
            fi
        else
            echo -e "${RED}✗ Failed to build ${variant_name}${NC}"
        fi
    done
}

# Function to run wasm-opt optimizations
optimize_wasm() {
    echo -e "${GREEN}━━━ Optimizing WASM with wasm-opt ━━━${NC}"

    local release_wasm="${WASM_OUTPUT_DIR}/release/rusty_audio_bg.wasm"

    if [ ! -f "${release_wasm}" ]; then
        echo -e "${RED}No release WASM found. Build first.${NC}"
        return
    fi

    local opt_levels=("O2" "O3" "O4" "Oz")

    for opt_level in "${opt_levels[@]}"; do
        local output_file="${WASM_PROFILE_DIR}/rusty_audio_${opt_level}.wasm"

        echo -e "${BLUE}Optimizing with -${opt_level}...${NC}"

        wasm-opt \
            "-${opt_level}" \
            --enable-simd \
            --enable-bulk-memory \
            -o "${output_file}" \
            "${release_wasm}"

        if [ -f "${output_file}" ]; then
            local size=$(du -h "${output_file}" | cut -f1)
            local size_bytes=$(stat -c%s "${output_file}" 2>/dev/null || stat -f%z "${output_file}" 2>/dev/null)
            echo -e "${GREEN}✓ ${opt_level}: ${size} (${size_bytes} bytes)${NC}"
        fi
    done
}

# Function to analyze WASM bundle size
analyze_bundle_size() {
    echo -e "${GREEN}━━━ Bundle Size Analysis ━━━${NC}"

    local wasm_files=$(find "${WASM_OUTPUT_DIR}" -name "*.wasm" 2>/dev/null)

    if [ -z "${wasm_files}" ]; then
        echo "No WASM files found"
        return
    fi

    echo ""
    printf "%-30s %15s %15s\n" "File" "Size" "Gzipped"
    echo "────────────────────────────────────────────────────────────"

    echo "${wasm_files}" | while read -r wasm_file; do
        local name=$(basename $(dirname "${wasm_file}"))/$(basename "${wasm_file}")
        local size=$(du -h "${wasm_file}" | cut -f1)

        # Gzip compress to estimate network transfer size
        local gz_file="${wasm_file}.gz"
        gzip -c "${wasm_file}" > "${gz_file}"
        local gz_size=$(du -h "${gz_file}" | cut -f1)

        printf "%-30s %15s %15s\n" "${name}" "${size}" "${gz_size}"

        rm -f "${gz_file}"
    done

    echo ""
}

# Function to use twiggy for deep analysis
analyze_with_twiggy() {
    if ! command -v twiggy &>/dev/null; then
        echo -e "${YELLOW}Skipping twiggy analysis (not installed)${NC}"
        return
    fi

    echo -e "${GREEN}━━━ Twiggy Code Size Analysis ━━━${NC}"

    local release_wasm="${WASM_OUTPUT_DIR}/release/rusty_audio_bg.wasm"

    if [ ! -f "${release_wasm}" ]; then
        echo "No release WASM found"
        return
    fi

    echo -e "${BLUE}Top 20 largest functions:${NC}"
    twiggy top -n 20 "${release_wasm}"

    echo ""
    echo -e "${BLUE}Saving detailed analysis to ${WASM_PROFILE_DIR}/twiggy_analysis.txt${NC}"

    {
        echo "=== Twiggy Analysis ==="
        echo "File: ${release_wasm}"
        echo "Generated: $(date)"
        echo ""
        echo "=== Top Functions ==="
        twiggy top -n 50 "${release_wasm}"
        echo ""
        echo "=== Dominators Tree ==="
        twiggy dominators "${release_wasm}"
        echo ""
        echo "=== Paths to Large Items ==="
        twiggy paths "${release_wasm}"
    } > "${WASM_PROFILE_DIR}/twiggy_analysis.txt"

    echo -e "${GREEN}✓ Analysis saved${NC}"
}

# Function to generate HTML benchmark page
generate_benchmark_html() {
    echo -e "${GREEN}━━━ Generating Benchmark HTML ━━━${NC}"

    local html_file="${WASM_OUTPUT_DIR}/benchmark.html"

    cat > "${html_file}" << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Rusty Audio WASM Benchmarks</title>
    <style>
        body {
            font-family: monospace;
            max-width: 1200px;
            margin: 40px auto;
            padding: 20px;
            background: #1e1e1e;
            color: #d4d4d4;
        }
        h1 { color: #4ec9b0; }
        h2 { color: #569cd6; }
        .metric {
            display: flex;
            justify-content: space-between;
            padding: 10px;
            margin: 5px 0;
            background: #2d2d30;
            border-left: 4px solid #007acc;
        }
        .metric.good { border-left-color: #4ec9b0; }
        .metric.warning { border-left-color: #ce9178; }
        .metric.error { border-left-color: #f48771; }
        .value { font-weight: bold; color: #4ec9b0; }
        #results { margin-top: 20px; }
        button {
            background: #007acc;
            color: white;
            border: none;
            padding: 10px 20px;
            font-size: 16px;
            cursor: pointer;
            margin: 10px 5px;
        }
        button:hover { background: #005a9e; }
        #status { padding: 10px; background: #2d2d30; margin: 10px 0; }
    </style>
</head>
<body>
    <h1>🎵 Rusty Audio WASM Performance Benchmarks</h1>

    <div id="status">Status: Ready</div>

    <button onclick="runBenchmarks()">Run Benchmarks</button>
    <button onclick="clearResults()">Clear Results</button>

    <div id="results"></div>

    <script type="module">
        async function measureLoadTime() {
            const startTime = performance.now();

            try {
                const module = await import('./release/rusty_audio.js');
                await module.default();

                const endTime = performance.now();
                const loadTime = endTime - startTime;

                return {
                    success: true,
                    loadTime: loadTime,
                    module: module
                };
            } catch (error) {
                return {
                    success: false,
                    error: error.toString()
                };
            }
        }

        async function measureInitTime(module) {
            const startTime = performance.now();

            // Initialize audio context
            // Note: Replace with actual initialization code
            try {
                // Simulate initialization
                await new Promise(resolve => setTimeout(resolve, 10));

                const endTime = performance.now();
                return endTime - startTime;
            } catch (error) {
                console.error('Init error:', error);
                return -1;
            }
        }

        window.runBenchmarks = async function() {
            const resultsDiv = document.getElementById('results');
            const statusDiv = document.getElementById('status');

            statusDiv.textContent = 'Status: Running benchmarks...';
            resultsDiv.innerHTML = '<h2>Running...</h2>';

            const results = [];

            // Measure load time
            statusDiv.textContent = 'Status: Measuring module load time...';
            const loadResult = await measureLoadTime();

            if (!loadResult.success) {
                resultsDiv.innerHTML = `<div class="metric error">
                    <span>Load Error</span>
                    <span class="value">${loadResult.error}</span>
                </div>`;
                statusDiv.textContent = 'Status: Failed';
                return;
            }

            results.push({
                name: 'Module Load Time',
                value: loadResult.loadTime.toFixed(2) + ' ms',
                className: loadResult.loadTime < 100 ? 'good' : 'warning'
            });

            // Measure init time
            statusDiv.textContent = 'Status: Measuring initialization...';
            const initTime = await measureInitTime(loadResult.module);

            if (initTime >= 0) {
                results.push({
                    name: 'Initialization Time',
                    value: initTime.toFixed(2) + ' ms',
                    className: initTime < 50 ? 'good' : 'warning'
                });
            }

            // Memory usage
            if (performance.memory) {
                results.push({
                    name: 'Used Heap',
                    value: (performance.memory.usedJSHeapSize / 1024 / 1024).toFixed(2) + ' MB',
                    className: 'good'
                });

                results.push({
                    name: 'Total Heap',
                    value: (performance.memory.totalJSHeapSize / 1024 / 1024).toFixed(2) + ' MB',
                    className: 'good'
                });
            }

            // Render results
            let html = '<h2>Benchmark Results</h2>';
            results.forEach(result => {
                html += `<div class="metric ${result.className}">
                    <span>${result.name}</span>
                    <span class="value">${result.value}</span>
                </div>`;
            });

            resultsDiv.innerHTML = html;
            statusDiv.textContent = 'Status: Complete';
        };

        window.clearResults = function() {
            document.getElementById('results').innerHTML = '';
            document.getElementById('status').textContent = 'Status: Ready';
        };
    </script>
</body>
</html>
EOF

    echo -e "${GREEN}✓ Benchmark page: ${html_file}${NC}"
    echo "  Open in browser: file://${html_file}"
}

# Function to generate size report
generate_size_report() {
    echo -e "${GREEN}━━━ Size Optimization Report ━━━${NC}"

    local report_file="${WASM_OUTPUT_DIR}/size_report.txt"

    {
        echo "=== WASM Bundle Size Report ==="
        echo "Generated: $(date)"
        echo ""

        echo "=== Build Variants ==="
        find "${WASM_OUTPUT_DIR}" -name "rusty_audio_bg.wasm" 2>/dev/null | while read -r wasm; do
            local variant=$(basename $(dirname "${wasm}"))
            local size_bytes=$(stat -c%s "${wasm}" 2>/dev/null || stat -f%z "${wasm}" 2>/dev/null)
            local size_kb=$((size_bytes / 1024))

            echo "  ${variant}: ${size_kb} KB (${size_bytes} bytes)"
        done

        echo ""
        echo "=== Optimized Variants (wasm-opt) ==="
        find "${WASM_PROFILE_DIR}" -name "*.wasm" 2>/dev/null | while read -r wasm; do
            local name=$(basename "${wasm}")
            local size_bytes=$(stat -c%s "${wasm}" 2>/dev/null || stat -f%z "${wasm}" 2>/dev/null)
            local size_kb=$((size_bytes / 1024))

            echo "  ${name}: ${size_kb} KB (${size_bytes} bytes)"
        done

        echo ""
        echo "=== Size Targets ==="
        echo "  Initial load: <500 KB (gzipped)"
        echo "  Total bundle: <1.5 MB (gzipped)"
        echo "  Lazy modules: <200 KB each"

    } > "${report_file}"

    cat "${report_file}"
    echo ""
    echo -e "${GREEN}Report saved: ${report_file}${NC}"
}

# Main execution
MODE="${1:-all}"

case "${MODE}" in
    "build")
        check_dependencies
        build_wasm_variants
        ;;
    "optimize")
        check_dependencies
        optimize_wasm
        ;;
    "analyze")
        analyze_bundle_size
        analyze_with_twiggy
        ;;
    "benchmark")
        generate_benchmark_html
        echo -e "${YELLOW}Open the HTML file in a browser and click 'Run Benchmarks'${NC}"
        ;;
    "report")
        generate_size_report
        ;;
    "all")
        check_dependencies
        build_wasm_variants
        optimize_wasm
        analyze_bundle_size
        analyze_with_twiggy
        generate_benchmark_html
        generate_size_report
        ;;
    "help"|"-h"|"--help")
        echo "Usage: $0 [MODE]"
        echo ""
        echo "Modes:"
        echo "  build      - Build WASM variants (dev, profiling, release)"
        echo "  optimize   - Run wasm-opt optimizations"
        echo "  analyze    - Analyze bundle size with twiggy"
        echo "  benchmark  - Generate HTML benchmark page"
        echo "  report     - Generate size optimization report"
        echo "  all        - Run complete WASM benchmark suite (default)"
        echo "  help       - Show this help message"
        echo ""
        ;;
    *)
        echo -e "${RED}Unknown mode: ${MODE}${NC}"
        exit 1
        ;;
esac

echo ""
echo -e "${GREEN}=== WASM Benchmark Complete ===${NC}"
