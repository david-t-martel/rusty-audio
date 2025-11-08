# Rusty Audio - Build Automation with Just
# https://github.com/casey/just

# Default recipe (shows available commands)
default:
    @just --list

# Build configurations
debug := "debug"
release := "release"

# === Development Commands ===

# Quick compile check (fastest feedback)
check:
    @echo "🔍 Running cargo check..."
    cargo check --all-targets

# Check library only (faster for backend work)
check-lib:
    @echo "📚 Checking library..."
    cargo check --lib

# Check main binary only
check-bin:
    @echo "🎵 Checking binary..."
    cargo check --bin rusty-audio

# === Building ===

# Build debug version (fast compilation)
build:
    @echo "🔨 Building debug version..."
    cargo build

# Build release version (optimized)
build-release:
    @echo "⚡ Building release version..."
    cargo build --release

# Build all targets
build-all:
    @echo "🏗️ Building all targets..."
    cargo build --all-targets

# Clean build artifacts
clean:
    @echo "🧹 Cleaning build artifacts..."
    cargo clean

# === Running ===

# Run debug version
run:
    @echo "▶️ Running rusty-audio (debug)..."
    cargo run

# Run release version (better performance)
run-release:
    @echo "⚡ Running rusty-audio (release)..."
    cargo run --release

# Run with environment variable for logging
run-debug:
    @echo "🐛 Running with debug logging..."
    $env:RUST_LOG="debug" cargo run

# === Testing ===

# Run all tests
test:
    @echo "🧪 Running tests..."
    cargo test --all-targets

# Run tests with output
test-verbose:
    @echo "🧪 Running tests (verbose)..."
    cargo test --all-targets -- --nocapture

# Run library tests only
test-lib:
    @echo "📚 Testing library..."
    cargo test --lib

# Run integration tests only
test-integration:
    @echo "🔗 Running integration tests..."
    cargo test --test '*'

# Run specific test
test-one TEST:
    @echo "🎯 Running test: {{TEST}}..."
    cargo test {{TEST}} -- --nocapture

# === Code Quality ===

# Run clippy (linter)
lint:
    @echo "📎 Running clippy..."
    cargo clippy --all-targets -- -D warnings

# Run clippy with fixes
lint-fix:
    @echo "🔧 Running clippy with automatic fixes..."
    cargo clippy --fix --all-targets --allow-dirty --allow-staged

# Format code
fmt:
    @echo "🎨 Formatting code..."
    cargo fmt --all

# Check formatting without modifying
fmt-check:
    @echo "👀 Checking code formatting..."
    cargo fmt --all -- --check

# Run all quality checks
quality: fmt-check lint test
    @echo "✅ All quality checks passed!"

# === Benchmarks ===

# Run benchmarks
bench:
    @echo "📊 Running benchmarks..."
    cargo bench

# Run specific benchmark
bench-one BENCH:
    @echo "📊 Running benchmark: {{BENCH}}..."
    cargo bench {{BENCH}}

# === Documentation ===

# Generate and open documentation
doc:
    @echo "📖 Generating documentation..."
    cargo doc --open --no-deps

# Generate documentation without opening
doc-build:
    @echo "📖 Building documentation..."
    cargo doc --no-deps

# Check documentation links
doc-check:
    @echo "🔗 Checking documentation..."
    cargo doc --no-deps

# === Platform-Specific ===

# Build for Windows with ASIO support (future)
build-windows-asio:
    @echo "🪟 Building for Windows with ASIO..."
    cargo build --release --features asio

# Build for WASM target
build-wasm:
    @echo "🌐 Building for WASM..."
    cargo build --target wasm32-unknown-unknown --lib

# === Advanced ===

# Update dependencies
update:
    @echo "⬆️ Updating dependencies..."
    cargo update

# Check for outdated dependencies
outdated:
    @echo "🔍 Checking for outdated dependencies..."
    cargo outdated

# Run cargo tree (dependency graph)
tree:
    @echo "🌳 Dependency tree..."
    cargo tree

# Check compilation time
time-build:
    @echo "⏱️ Timing build..."
    cargo build --timings

# Expand macros for debugging
expand FILE:
    @echo "🔍 Expanding macros in {{FILE}}..."
    cargo expand --lib {{FILE}}

# === Audio-Specific ===

# Test audio backend
test-audio:
    @echo "🎵 Testing audio backend..."
    cargo test --lib audio::

# Test hybrid audio system
test-hybrid:
    @echo "🔀 Testing hybrid audio..."
    cargo test --lib hybrid

# Test device enumeration
test-devices:
    @echo "🎧 Testing device enumeration..."
    cargo test --lib device

# === CI/CD Simulation ===

# Run CI checks locally (what GitHub Actions would run)
ci: fmt-check lint test
    @echo "✅ CI checks passed!"

# Full pre-commit check
pre-commit: fmt lint test check-bin
    @echo "✅ Ready to commit!"

# Full pre-release check
pre-release: quality build-release test doc-check
    @echo "✅ Ready for release!"

# === Profiling & Performance ===

# Profile with perf (Linux)
profile-perf:
    @echo "📊 Profiling with perf..."
    cargo build --release
    perf record -F 99 -g target/release/rusty-audio
    perf script | stackcollapse-perf.pl | flamegraph.pl > flame.svg

# Profile with cargo-flamegraph
profile-flame:
    @echo "🔥 Generating flamegraph..."
    cargo flamegraph

# === Workspace Management ===

# Show workspace status
status:
    @echo "📊 Workspace Status:"
    @echo ""
    @echo "Build artifacts:"
    @du -sh target/ 2>/dev/null || echo "No build artifacts"
    @echo ""
    @echo "Git status:"
    @git status -s
    @echo ""
    @echo "Recent commits:"
    @git log --oneline -5

# Clean everything (including Cargo cache)
clean-all: clean
    @echo "🧹 Cleaning Cargo cache..."
    @rm -rf ~/.cargo/registry/cache
    @rm -rf ~/.cargo/git/db

# === Help ===

# Show detailed help for key commands
help:
    @echo "🎵 Rusty Audio - Build Commands"
    @echo ""
    @echo "Quick Start:"
    @echo "  just check          - Fast compile check"
    @echo "  just build          - Build debug version"
    @echo "  just run            - Run debug version"
    @echo "  just test           - Run all tests"
    @echo ""
    @echo "Development:"
    @echo "  just fmt            - Format code"
    @echo "  just lint           - Run clippy"
    @echo "  just quality        - Run all checks"
    @echo "  just pre-commit     - Full pre-commit check"
    @echo ""
    @echo "Audio Testing:"
    @echo "  just test-audio     - Test audio backend"
    @echo "  just test-hybrid    - Test hybrid system"
    @echo "  just test-devices   - Test device enumeration"
    @echo ""
    @echo "Run 'just --list' for all commands"

# === Install Tools ===

# Install required tools
install-tools:
    @echo "🔧 Installing required tools..."
    cargo install cargo-watch cargo-outdated cargo-tree cargo-expand
    @echo "✅ Tools installed!"
