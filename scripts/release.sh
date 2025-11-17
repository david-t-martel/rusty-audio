#!/usr/bin/env bash
# Automated Release Script for Rusty Audio
# Handles versioning, changelog generation, and release creation

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

print_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
print_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
print_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
print_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Configuration
CURRENT_VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
RELEASE_TYPE="${1:-patch}"  # major, minor, patch, or specific version

# Header
echo -e "${CYAN}========================================${NC}"
echo -e "${CYAN}  Rusty Audio - Release Automation${NC}"
echo -e "${CYAN}========================================${NC}"
echo ""
echo -e "${YELLOW}Current version: $CURRENT_VERSION${NC}"
echo -e "${YELLOW}Release type: $RELEASE_TYPE${NC}"
echo ""

# Calculate new version
calculate_version() {
    local current=$1
    local type=$2

    IFS='.' read -r -a parts <<< "$current"
    major="${parts[0]}"
    minor="${parts[1]}"
    patch="${parts[2]}"

    case "$type" in
        major)
            major=$((major + 1))
            minor=0
            patch=0
            ;;
        minor)
            minor=$((minor + 1))
            patch=0
            ;;
        patch)
            patch=$((patch + 1))
            ;;
        *)
            # Assume it's a specific version
            echo "$type"
            return
            ;;
    esac

    echo "${major}.${minor}.${patch}"
}

NEW_VERSION=$(calculate_version "$CURRENT_VERSION" "$RELEASE_TYPE")

echo -e "${GREEN}New version: $NEW_VERSION${NC}"
echo ""

# Confirmation
read -p "Proceed with release v${NEW_VERSION}? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    print_warning "Release cancelled"
    exit 0
fi

# Pre-release checks
print_info "[1/10] Running pre-release checks..."

# Check if on main branch
CURRENT_BRANCH=$(git branch --show-current)
if [ "$CURRENT_BRANCH" != "main" ]; then
    print_error "Must be on main branch to release. Current: $CURRENT_BRANCH"
    exit 1
fi

# Check for uncommitted changes
if ! git diff-index --quiet HEAD --; then
    print_error "Uncommitted changes detected. Please commit or stash them."
    exit 1
fi

# Check if remote is up to date
git fetch origin
LOCAL=$(git rev-parse @)
REMOTE=$(git rev-parse @{u})

if [ "$LOCAL" != "$REMOTE" ]; then
    print_error "Local branch is not up to date with remote. Please pull latest changes."
    exit 1
fi

print_success "Pre-release checks passed"

# Update version in Cargo.toml
print_info "[2/10] Updating version in Cargo.toml..."
sed -i.bak "s/^version = \".*\"/version = \"${NEW_VERSION}\"/" Cargo.toml
rm Cargo.toml.bak
print_success "Version updated to ${NEW_VERSION}"

# Update Cargo.lock
print_info "[3/10] Updating Cargo.lock..."
cargo check --quiet
print_success "Cargo.lock updated"

# Run tests
print_info "[4/10] Running tests..."
if ! cargo test --all-features --all-targets; then
    print_error "Tests failed!"
    print_warning "Reverting version changes..."
    git checkout Cargo.toml Cargo.lock
    exit 1
fi
print_success "All tests passed"

# Run clippy
print_info "[5/10] Running clippy..."
if ! cargo clippy --all-targets --all-features -- -D warnings; then
    print_error "Clippy checks failed!"
    print_warning "Reverting version changes..."
    git checkout Cargo.toml Cargo.lock
    exit 1
fi
print_success "Clippy checks passed"

# Generate changelog
print_info "[6/10] Generating changelog..."

CHANGELOG_FILE="CHANGELOG.md"
CHANGELOG_TEMP="CHANGELOG.tmp"

# Get commits since last tag
LAST_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "")

if [ -n "$LAST_TAG" ]; then
    COMMITS=$(git log --pretty=format:"- %s (%h)" ${LAST_TAG}..HEAD)
else
    COMMITS=$(git log --pretty=format:"- %s (%h)" -20)
fi

# Create changelog entry
cat > "$CHANGELOG_TEMP" <<EOF
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [${NEW_VERSION}] - $(date +%Y-%m-%d)

### Changed
${COMMITS}

EOF

# Append existing changelog if it exists
if [ -f "$CHANGELOG_FILE" ]; then
    # Skip the first line (header) if it exists
    tail -n +2 "$CHANGELOG_FILE" >> "$CHANGELOG_TEMP" 2>/dev/null || true
fi

mv "$CHANGELOG_TEMP" "$CHANGELOG_FILE"
print_success "Changelog updated"

# Build release artifacts
print_info "[7/10] Building release artifacts..."

# Desktop build
print_info "Building desktop binary..."
cargo build --release --features native-binary

# WASM build
print_info "Building WASM..."
if command -v wasm-pack >/dev/null 2>&1; then
    wasm-pack build --target web --out-dir dist/pkg --release -- --no-default-features
else
    print_warning "wasm-pack not found, skipping WASM build"
fi

print_success "Release artifacts built"

# Commit version changes
print_info "[8/10] Committing version changes..."
git add Cargo.toml Cargo.lock CHANGELOG.md
git commit -m "chore: release v${NEW_VERSION}"
print_success "Version changes committed"

# Create git tag
print_info "[9/10] Creating git tag..."
git tag -a "v${NEW_VERSION}" -m "Release version ${NEW_VERSION}"
print_success "Tag v${NEW_VERSION} created"

# Push changes
print_info "[10/10] Pushing changes and tag..."
read -p "Push to remote? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    git push origin main
    git push origin "v${NEW_VERSION}"
    print_success "Changes pushed to remote"
else
    print_warning "Skipping push. Remember to push manually:"
    echo -e "${CYAN}  git push origin main${NC}"
    echo -e "${CYAN}  git push origin v${NEW_VERSION}${NC}"
fi

# Summary
echo ""
echo -e "${CYAN}========================================${NC}"
echo -e "${GREEN}  Release v${NEW_VERSION} Complete!${NC}"
echo -e "${CYAN}========================================${NC}"
echo ""
echo -e "${YELLOW}Release Summary:${NC}"
echo -e "${NC}  - Version: ${NEW_VERSION}${NC}"
echo -e "${NC}  - Tag: v${NEW_VERSION}${NC}"
echo -e "${NC}  - Changelog updated${NC}"
echo -e "${NC}  - All tests passed${NC}"
echo ""
echo -e "${YELLOW}Next steps:${NC}"
echo -e "${NC}  1. GitHub Actions will automatically build release artifacts${NC}"
echo -e "${NC}  2. Monitor workflow: https://github.com/<owner>/rusty-audio/actions${NC}"
echo -e "${NC}  3. GitHub Release will be created automatically${NC}"
echo -e "${NC}  4. Verify release: https://github.com/<owner>/rusty-audio/releases${NC}"
echo ""
echo -e "${YELLOW}Manual steps (if needed):${NC}"
echo -e "${NC}  - Create GitHub Release manually from tag${NC}"
echo -e "${NC}  - Update documentation${NC}"
echo -e "${NC}  - Announce release on social media${NC}"
echo ""
