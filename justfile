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
    cargo install cargo-tarpaulin cargo-audit cargo-deny
    @echo "✅ Tools installed!"

# Install ast-grep
install-ast-grep:
    @echo "🔧 Installing ast-grep..."
    curl -L https://github.com/ast-grep/ast-grep/releases/latest/download/ast-grep-x86_64-unknown-linux-gnu.tar.gz | tar -xz
    @echo "✅ ast-grep installed!"

# Install sccache
install-sccache:
    @echo "🔧 Installing sccache..."
    cargo install sccache --locked
    @echo "✅ sccache installed!"
    @echo "Enable in .cargo/config.toml: rustc-wrapper = \"sccache\""

# === AST-Grep Code Analysis ===

# Run all ast-grep checks
ast-grep:
    @echo "🔍 Running AST-Grep analysis..."
    ast-grep scan --config .ast-grep/sgconfig.yml src/

# Run ast-grep panic detection
ast-grep-panic:
    @echo "🚨 Checking for panic-inducing code..."
    ast-grep scan --config .ast-grep/panic-detection.yml src/

# Run ast-grep audio safety checks
ast-grep-audio:
    @echo "🎵 Checking audio safety rules..."
    ast-grep scan --config .ast-grep/sgconfig.yml --ruleset audio-safety src/

# Run ast-grep error handling checks
ast-grep-errors:
    @echo "🛡️ Checking error handling..."
    ast-grep scan --config .ast-grep/sgconfig.yml --ruleset error-handling src/

# Run ast-grep performance checks
ast-grep-perf:
    @echo "⚡ Checking performance patterns..."
    ast-grep scan --config .ast-grep/sgconfig.yml --ruleset performance src/

# Full ast-grep analysis with JSON output
ast-grep-report:
    @echo "📊 Generating AST-Grep JSON report..."
    ast-grep scan --config .ast-grep/sgconfig.yml --json src/ > ast-grep-report.json
    @echo "✅ Report saved to ast-grep-report.json"

# === Auto-Claude Integration ===

# Run auto-claude analysis (requires auto-claude CLI)
auto-claude:
    @echo "🤖 Running auto-claude analysis..."
    @echo "Note: Requires auto-claude CLI to be installed"
    @# auto-claude analyze --path src/ --config .auto-claude/config.json
    @echo "⚠️ auto-claude not yet configured - see https://github.com/anthropics/auto-claude"

# Auto-claude code review
auto-claude-review:
    @echo "👁️ Running auto-claude code review..."
    @echo "Analyzing uncommitted changes..."
    @git diff > /tmp/rusty-audio-changes.diff
    @echo "Review saved to /tmp/rusty-audio-changes.diff"
    @echo "⚠️ Run with actual auto-claude when available"

# Auto-claude security audit
auto-claude-security:
    @echo "🔒 Running auto-claude security audit..."
    cargo audit --json | jq '.' > /tmp/audit-results.json
    @echo "Security audit results saved"

# === Comprehensive Quality Gates ===

# Run all quality checks (matches GitHub CI)
quality-full: fmt-check lint test ast-grep-panic ast-grep-audio
    @echo "✅ All quality gates passed!"
    @echo "🎉 Code is ready for commit"

# Run security-focused quality checks
quality-security: ast-grep-panic cargo-audit cargo-deny
    @echo "✅ Security checks passed!"

# Run performance-focused quality checks
quality-performance: ast-grep-perf bench
    @echo "✅ Performance checks passed!"

# === Cargo Security Tools ===

# Run cargo audit
cargo-audit:
    @echo "🔒 Running cargo audit..."
    cargo audit

# Run cargo deny
cargo-deny:
    @echo "🚫 Running cargo deny..."
    cargo deny check

# === Continuous Integration Simulation ===

# Simulate GitHub Actions locally
ci-local: quality-full cargo-audit cargo-deny
    @echo "🎬 Running full CI pipeline locally..."
    @echo "✅ All CI checks passed!"
    @echo "Ready to push!"

# Fast CI check (skip slow tests)
ci-fast: fmt-check lint test-lib ast-grep-panic
    @echo "⚡ Fast CI checks passed!"

# === Git Workflow Helpers ===

# Pre-push checks (run before git push)
pre-push: quality-full
    @echo "✅ Ready to push!"

# Pre-PR checks (comprehensive)
pre-pr: quality-full cargo-audit cargo-deny doc-check
    @echo "✅ Ready to create PR!"

# Quick commit check
quick-commit: fmt lint test-lib
    @echo "✅ Ready for quick commit!"

# === sccache Management ===

# Show sccache statistics
sccache-stats:
    @echo "📊 sccache statistics:"
    @sccache --show-stats

# Clear sccache cache
sccache-clear:
    @echo "🧹 Clearing sccache cache..."
    @sccache --stop-server
    @rm -rf ~/.cache/sccache
    @echo "✅ sccache cache cleared"

# Start sccache server
sccache-start:
    @echo "🚀 Starting sccache server..."
    @sccache --start-server

# === Panic Detection Helpers ===

# Find all unwrap() usage
find-unwrap:
    @echo "🔍 Finding all .unwrap() calls..."
    @grep -rn "\.unwrap()" src/ || echo "✅ No unwrap() found"

# Find all expect() usage
find-expect:
    @echo "🔍 Finding all .expect() calls..."
    @grep -rn "\.expect(" src/ || echo "✅ No expect() found"

# Find all panic!() usage
find-panic:
    @echo "🔍 Finding all panic!() calls..."
    @grep -rn "panic!" src/ || echo "✅ No panic!() found"

# Find all TODO comments
find-todos:
    @echo "🔍 Finding all TODO comments..."
    @grep -rn "TODO" src/ || echo "✅ No TODOs found"

# Comprehensive panic audit
panic-audit: find-unwrap find-expect find-panic ast-grep-panic
    @echo "✅ Panic audit complete!"

# === Development Workflow ===

# Watch and auto-rebuild on changes
watch:
    @echo "👀 Watching for changes..."
    cargo watch -x check -x test

# Watch and auto-run on changes
watch-run:
    @echo "👀 Watching and running..."
    cargo watch -x run

# Watch with clear screen
watch-clear:
    @echo "👀 Watching with clear..."
    cargo watch -c -x check
