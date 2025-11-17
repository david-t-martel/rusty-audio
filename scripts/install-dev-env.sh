#!/usr/bin/env bash

##############################################################################
# Development Environment Installation Script
#
# Automated setup for Rusty Audio local development environment
#
# Usage:
#   ./scripts/install-dev-env.sh [--skip-deps] [--skip-verify]
##############################################################################

set -euo pipefail

# Color output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

print_header() {
    echo -e "\n${BOLD}${CYAN}═══════════════════════════════════════════${NC}"
    echo -e "${BOLD}${CYAN}  $1${NC}"
    echo -e "${BOLD}${CYAN}═══════════════════════════════════════════${NC}\n"
}

print_success() { echo -e "${GREEN}✓${NC} $1"; }
print_error() { echo -e "${RED}✗${NC} $1" >&2; }
print_warning() { echo -e "${YELLOW}⚠${NC} $1"; }
print_info() { echo -e "${CYAN}→${NC} $1"; }

# Parse arguments
SKIP_DEPS=false
SKIP_VERIFY=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --skip-deps) SKIP_DEPS=true; shift ;;
        --skip-verify) SKIP_VERIFY=true; shift ;;
        *) print_error "Unknown option: $1"; exit 1 ;;
    esac
done

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

print_header "Rusty Audio Development Environment Setup"

# Step 1: Verify prerequisites
if [ "$SKIP_VERIFY" = false ]; then
    print_info "Verifying prerequisites..."

    if ! command -v cargo &> /dev/null; then
        print_error "Rust/Cargo not found"
        echo "Install from: https://rustup.rs/"
        exit 1
    fi

    if ! command -v node &> /dev/null; then
        print_error "Node.js not found"
        echo "Install from: https://nodejs.org/"
        exit 1
    fi

    # Check Node.js version
    NODE_VERSION=$(node --version | sed 's/v//' | cut -d. -f1)
    if [ "$NODE_VERSION" -lt 18 ]; then
        print_error "Node.js version 18+ required (found: $NODE_VERSION)"
        exit 1
    fi

    print_success "Prerequisites verified"
else
    print_warning "Skipping verification"
fi

# Step 2: Install Trunk if not present
print_info "Checking for Trunk..."
if ! command -v trunk &> /dev/null; then
    print_info "Installing Trunk..."
    cargo install trunk
    print_success "Trunk installed"
else
    print_success "Trunk already installed"
fi

# Step 3: Add WASM target if not present
print_info "Checking WASM target..."
if ! rustup target list --installed | grep -q "wasm32-unknown-unknown"; then
    print_info "Adding WASM target..."
    rustup target add wasm32-unknown-unknown
    print_success "WASM target added"
else
    print_success "WASM target already installed"
fi

# Step 4: Install Node.js dependencies
if [ "$SKIP_DEPS" = false ]; then
    print_info "Installing Node.js dependencies..."
    npm install
    print_success "Dependencies installed"
else
    print_warning "Skipping dependency installation"
fi

# Step 5: Verify installation
print_info "Verifying installation..."
node "$SCRIPT_DIR/setup-verify.js"

# Step 6: Build WASM
print_header "Building WASM Application"
print_info "This may take a few minutes..."

export RUSTFLAGS="-C target-feature=+atomics,+bulk-memory,+mutable-globals"
trunk build

if [ $? -eq 0 ]; then
    print_success "WASM build completed"
else
    print_error "WASM build failed"
    exit 1
fi

# Step 7: Validate build
print_info "Validating build..."
node "$SCRIPT_DIR/validate-build.js"

# Success!
print_header "Setup Complete!"

echo -e "${GREEN}✓${NC} Development environment ready!"
echo ""
echo -e "${CYAN}Quick start:${NC}"
echo -e "  ${BOLD}npm run dev${NC}          - Start development server"
echo -e "  ${BOLD}npm run health${NC}       - Check server health"
echo -e "  ${BOLD}npm run test:threading${NC} - Test multithreading"
echo ""
echo -e "${CYAN}Then open:${NC} ${BOLD}http://localhost:8080${NC}"
echo ""
echo -e "${CYAN}Documentation:${NC}"
echo -e "  Quick start:      QUICK_START_LOCAL_DEV.md"
echo -e "  Full guide:       LOCAL_DEV_SETUP.md"
echo -e "  Implementation:   LOCAL_DEV_IMPLEMENTATION_SUMMARY.md"
echo ""
