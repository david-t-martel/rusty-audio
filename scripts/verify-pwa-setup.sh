#!/usr/bin/env bash
# Verification script for Rusty Audio PWA deployment setup
# Checks all prerequisites and configuration

set -euo pipefail

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

PASS="${GREEN}✓${NC}"
FAIL="${RED}✗${NC}"
WARN="${YELLOW}⚠${NC}"
INFO="${BLUE}ℹ${NC}"

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "======================================"
echo "  Rusty Audio - PWA Setup Verification"
echo "======================================"
echo ""

# Track results
WARNINGS=0
ERRORS=0

# Check function
check() {
    local name="$1"
    local command="$2"
    local required="${3:-true}"

    printf "%-40s " "$name"

    if eval "$command" >/dev/null 2>&1; then
        echo -e "$PASS"
        return 0
    else
        if [[ "$required" == "true" ]]; then
            echo -e "$FAIL (required)"
            ((ERRORS++))
            return 1
        else
            echo -e "$WARN (optional)"
            ((WARNINGS++))
            return 1
        fi
    fi
}

# Section 1: Required Tools
echo "Required Tools:"
echo "----------------"
check "Rust compiler" "rustc --version"
check "Cargo package manager" "cargo --version"
check "WASM target installed" "rustup target list --installed | grep -q wasm32-unknown-unknown"
check "wasm-bindgen CLI" "wasm-bindgen --version"

echo ""

# Section 2: Optional Tools
echo "Optional Tools (for optimization):"
echo "-----------------------------------"
check "wasm-opt (binaryen)" "wasm-opt --version" false
check "Python (for local server)" "python3 --version" false
check "ImageMagick (for icons)" "convert -version" false
check "optipng (PNG optimization)" "optipng --version" false

echo ""

# Section 3: Required Files
echo "Required Files:"
echo "---------------"
check "index.html" "test -f ${PROJECT_ROOT}/www/index.html"
check "manifest.json" "test -f ${PROJECT_ROOT}/www/manifest.json"
check "Service worker (sw.js)" "test -f ${PROJECT_ROOT}/www/sw.js"
check "Build script" "test -x ${PROJECT_ROOT}/scripts/build-wasm.sh"
check "Deploy script" "test -x ${PROJECT_ROOT}/scripts/deploy-wasm.sh"

echo ""

# Section 4: PWA Icons
echo "PWA Icons:"
echo "----------"
REQUIRED_ICONS=(192 512)
OPTIONAL_ICONS=(72 96 128 144 152 384)

for size in "${REQUIRED_ICONS[@]}"; do
    check "icon-${size}.png (required)" "test -f ${PROJECT_ROOT}/www/icon-${size}.png"
done

missing_optional=0
for size in "${OPTIONAL_ICONS[@]}"; do
    if [[ ! -f "${PROJECT_ROOT}/www/icon-${size}.png" ]]; then
        ((missing_optional++))
    fi
done

if [[ $missing_optional -gt 0 ]]; then
    printf "%-40s " "Optional icons (${missing_optional} missing)"
    echo -e "$WARN (run generate-icons.sh)"
    ((WARNINGS++))
else
    printf "%-40s " "All optional icons"
    echo -e "$PASS"
fi

echo ""

# Section 5: Cargo Configuration
echo "Cargo Configuration:"
echo "--------------------"

# Check for lib crate type
if grep -q 'crate-type.*=.*\["cdylib"' "${PROJECT_ROOT}/Cargo.toml" 2>/dev/null; then
    printf "%-40s " "Library crate type (cdylib)"
    echo -e "$PASS"
else
    printf "%-40s " "Library crate type (cdylib)"
    echo -e "$FAIL"
    echo "  Add to Cargo.toml:"
    echo "  [lib]"
    echo "  crate-type = [\"cdylib\", \"rlib\"]"
    ((ERRORS++))
fi

# Check for WASM dependencies
if grep -q 'wasm-bindgen' "${PROJECT_ROOT}/Cargo.toml" 2>/dev/null; then
    printf "%-40s " "wasm-bindgen dependency"
    echo -e "$PASS"
else
    printf "%-40s " "wasm-bindgen dependency"
    echo -e "$FAIL"
    ((ERRORS++))
fi

echo ""

# Section 6: GitHub Actions (if exists)
if [[ -f "${PROJECT_ROOT}/.github/workflows/deploy-pwa.yml" ]]; then
    echo "CI/CD Configuration:"
    echo "--------------------"
    printf "%-40s " "GitHub Actions workflow"
    echo -e "$PASS"
    printf "%-40s " "Lighthouse configuration"
    if [[ -f "${PROJECT_ROOT}/.github/lighthouse/lighthouserc.json" ]]; then
        echo -e "$PASS"
    else
        echo -e "$WARN (optional)"
    fi
    echo ""
fi

# Section 7: Manifest Validation
echo "Manifest Validation:"
echo "--------------------"

if [[ -f "${PROJECT_ROOT}/www/manifest.json" ]]; then
    # Check if valid JSON
    if python3 -c "import json; json.load(open('${PROJECT_ROOT}/www/manifest.json'))" 2>/dev/null; then
        printf "%-40s " "Valid JSON"
        echo -e "$PASS"

        # Check required fields
        manifest="${PROJECT_ROOT}/www/manifest.json"

        for field in name short_name start_url display icons; do
            if grep -q "\"$field\"" "$manifest"; then
                printf "%-40s " "Has '$field' field"
                echo -e "$PASS"
            else
                printf "%-40s " "Has '$field' field"
                echo -e "$FAIL"
                ((ERRORS++))
            fi
        done
    else
        printf "%-40s " "Valid JSON"
        echo -e "$FAIL"
        ((ERRORS++))
    fi
else
    printf "%-40s " "manifest.json exists"
    echo -e "$FAIL"
    ((ERRORS++))
fi

echo ""

# Section 8: Build Test (optional, only if user wants)
if [[ "${RUN_BUILD_TEST:-false}" == "true" ]]; then
    echo "Build Test:"
    echo "-----------"
    printf "%-40s " "Running test build..."

    if ./scripts/build-wasm.sh >/dev/null 2>&1; then
        echo -e "$PASS"

        if [[ -f "${PROJECT_ROOT}/dist/rusty_audio_bg.wasm" ]]; then
            wasm_size=$(stat -c%s "${PROJECT_ROOT}/dist/rusty_audio_bg.wasm" 2>/dev/null || stat -f%z "${PROJECT_ROOT}/dist/rusty_audio_bg.wasm")
            wasm_mb=$((wasm_size / 1024 / 1024))

            printf "%-40s " "WASM bundle size: ${wasm_mb}MB"

            if [[ $wasm_size -lt 2097152 ]]; then
                echo -e "$PASS"
            elif [[ $wasm_size -lt 5242880 ]]; then
                echo -e "$WARN (consider optimization)"
                ((WARNINGS++))
            else
                echo -e "$FAIL (too large: >5MB)"
                ((ERRORS++))
            fi
        fi
    else
        echo -e "$FAIL"
        echo "  Run: ./scripts/build-wasm.sh for details"
        ((ERRORS++))
    fi
    echo ""
fi

# Summary
echo "======================================"
echo "  Summary"
echo "======================================"
echo ""

if [[ $ERRORS -eq 0 ]] && [[ $WARNINGS -eq 0 ]]; then
    echo -e "${GREEN}✓ All checks passed!${NC}"
    echo ""
    echo "Ready to build and deploy:"
    echo "  1. Build: ./scripts/build-wasm.sh"
    echo "  2. Test: ./scripts/deploy-wasm.sh local"
    echo "  3. Deploy: ./scripts/deploy-wasm.sh github"
    exit 0
elif [[ $ERRORS -eq 0 ]]; then
    echo -e "${YELLOW}⚠ Passed with $WARNINGS warning(s)${NC}"
    echo ""
    echo "You can proceed, but consider fixing warnings:"
    echo "  - Install optional tools for better optimization"
    echo "  - Generate all icon sizes for better PWA support"
    echo ""
    echo "To build anyway:"
    echo "  ./scripts/build-wasm.sh"
    exit 0
else
    echo -e "${RED}✗ Failed with $ERRORS error(s) and $WARNINGS warning(s)${NC}"
    echo ""
    echo "Fix the errors above before building."
    echo ""
    echo "Common fixes:"
    echo "  - Install Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    echo "  - Add WASM target: rustup target add wasm32-unknown-unknown"
    echo "  - Install wasm-bindgen: cargo install wasm-bindgen-cli"
    echo "  - Generate icons: ./scripts/generate-icons.sh logo.png"
    exit 1
fi
