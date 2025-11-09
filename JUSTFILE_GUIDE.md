# Rusty Audio - Justfile Build Guide

**Last Updated**: 2025-11-08
**Status**: ✅ Complete with Windows and WASM support

---

## 🚀 Quick Start

```bash
# View all available commands
just --list

# Show detailed help
just help

# Build for Windows (AVX2 optimized)
just build-windows

# Build for WASM/PWA
just build-wasm-release

# Build both targets
just build-matrix
```

---

## 🪟 Windows Build Commands

### Primary Windows Target (MSVC - AVX2 Optimized)

The primary Windows target is `x86_64-pc-windows-msvc` with:
- **CPU Target**: x86-64-v3 (AVX2, BMI2, FMA)
- **Stack Size**: 8MB (for audio processing)
- **SIMD**: AVX2/SSE optimizations enabled
- **Optimization**: Link-time optimization (LTO)

```bash
# Release build (recommended)
just build-windows

# Debug build
just build-windows-debug

# Test Windows binary
just test-windows

# Run benchmarks (SIMD optimized)
just bench-windows

# Complete Windows release (build + test)
just release-windows
```

### Alternative Windows Toolchain (GNU)

```bash
# Build with GNU toolchain
just build-windows-gnu
```

### Windows-Specific Features

```bash
# Build with ASIO support (future)
just build-windows-asio
```

---

## 🌐 WASM/PWA Build Commands

### WASM Compilation

```bash
# Development build
just build-wasm

# Release build (size optimized: opt-level=z, fat LTO)
just build-wasm-release

# Using cargo alias (equivalent to build-wasm-release)
just build-wasm-alias
```

### PWA Deployment Pipeline

```bash
# Build complete PWA bundle
just pwa-build

# Deploy locally (for testing)
just pwa-deploy-local

# Deploy to GitHub Pages
just pwa-deploy-github

# Deploy to Cloudflare
just pwa-deploy-cloudflare

# Deploy to Netlify
just pwa-deploy-netlify

# Verify PWA setup
just pwa-verify

# Complete PWA release (build + verify)
just release-pwa
```

---

## 🏗️ Complete Build Matrix

### Build All Targets

```bash
# Build all platforms:
# 1. Windows MSVC (release)
# 2. Windows GNU (release)
# 3. WASM (development)
# 4. WASM (release)
just build-matrix

# Build and test all targets
just build-test-all
```

### Dual Release (Windows + PWA)

```bash
# Complete dual-platform release
just release-dual

# Output:
# - Windows executable: target/x86_64-pc-windows-msvc/release/rusty-audio.exe
# - PWA bundle: www/ directory
```

---

## 📊 Benchmarking and Profiling

### Benchmark Commands

```bash
# Run benchmarks (native target)
just bench

# Run specific benchmark
just bench-one <benchmark-name>

# Run Windows-specific benchmarks (SIMD optimized)
just bench-windows

# Run all optimization benchmarks
just bench-all
```

### Profiling Commands

```bash
# Desktop profiling suite (criterion, flamegraph, dhat)
just profile-desktop

# WASM profiling and size analysis
just profile-wasm

# Compare benchmark results
just bench-compare

# Generate flamegraph
just profile-flame
```

---

## 🧪 Testing Commands

### Basic Testing

```bash
# Run all tests
just test

# Run tests with verbose output
just test-verbose

# Run specific test
just test-one <test-name>
```

### Platform-Specific Testing

```bash
# Test Windows binary (requires native Windows)
just test-windows

# Test UI components (requires Windows with display)
just test-ui

# Test UI on Windows target
just test-ui-windows

# Test with property-based testing
just test-property
```

### Audio-Specific Testing

```bash
# Test audio backend
just test-audio

# Test hybrid audio system
just test-hybrid

# Test device enumeration
just test-devices
```

---

## ✅ Quality and Validation

### Code Quality

```bash
# Format code
just fmt

# Check formatting
just fmt-check

# Run clippy linter
just lint

# Run clippy with auto-fixes
just lint-fix

# Run all quality checks (fmt-check + lint + test)
just quality

# Full quality check (includes ast-grep)
just quality-full
```

### Pre-Commit Checks

```bash
# Quick commit check
just quick-commit

# Full pre-commit check
just pre-commit

# Pre-push checks
just pre-push

# Pre-PR checks (comprehensive)
just pre-pr
```

### Validation

```bash
# Complete project validation (all targets + tests + PWA)
just validate-all

# Quick validation (fmt + lint + build Windows + WASM)
just validate-quick
```

---

## 🛡️ Security and Safety

### Panic Detection

```bash
# Find all unwrap() calls
just find-unwrap

# Find all expect() calls
just find-expect

# Find all panic!() calls
just find-panic

# Comprehensive panic audit
just panic-audit
```

### AST-Grep Analysis

```bash
# Run all ast-grep checks
just ast-grep

# Panic detection
just ast-grep-panic

# Audio safety checks
just ast-grep-audio

# Error handling checks
just ast-grep-errors

# Performance checks
just ast-grep-perf

# Generate JSON report
just ast-grep-report
```

### Security Tools

```bash
# Run cargo audit
just cargo-audit

# Run cargo deny
just cargo-deny

# Security-focused quality checks
just quality-security
```

---

## 🔧 Development Workflow

### Watch Mode

```bash
# Watch and auto-rebuild
just watch

# Watch and auto-run
just watch-run

# Watch with clear screen
just watch-clear
```

### Documentation

```bash
# Generate and open documentation
just doc

# Generate documentation without opening
just doc-build

# Check documentation links
just doc-check
```

### Dependency Management

```bash
# Update dependencies
just update

# Check for outdated dependencies
just outdated

# Show dependency tree
just tree
```

---

## ⚙️ Tool Management

### sccache

```bash
# Show sccache statistics
just sccache-stats

# Clear sccache cache
just sccache-clear

# Start sccache server
just sccache-start
```

### Install Tools

```bash
# Install required tools
just install-tools

# Install ast-grep
just install-ast-grep

# Install sccache
just install-sccache
```

---

## 📈 CI/CD Simulation

```bash
# Run CI checks locally (what GitHub Actions would run)
just ci

# Fast CI check (skip slow tests)
just ci-fast

# Run full CI pipeline
just ci-local
```

---

## 🎯 Common Workflows

### 1. Quick Development Cycle

```bash
# Check → Build → Test → Run
just check
just build-windows-debug
just test
just run-release
```

### 2. Pre-Commit Workflow

```bash
# Format → Lint → Test → Build
just fmt
just lint
just test
just build-windows
```

### 3. Release Preparation

```bash
# Complete validation → Build all targets → Test
just validate-all
just build-matrix
just release-dual
```

### 4. PWA Deployment

```bash
# Build → Verify → Deploy
just build-wasm-release
just pwa-verify
just pwa-deploy-github
```

### 5. Performance Optimization

```bash
# Build → Benchmark → Profile
just build-windows
just bench-all
just profile-desktop
```

---

## 🔍 Build Targets

### Configured Targets in .cargo/config.toml

| Target | CPU | Optimization | Use Case |
|--------|-----|--------------|----------|
| `x86_64-pc-windows-msvc` | x86-64-v3 (AVX2) | Performance | Primary Windows |
| `x86_64-pc-windows-gnu` | native | Compatibility | Alternative Windows |
| `x86_64-unknown-linux-gnu` | native | Development | WSL/Linux dev |
| `wasm32-unknown-unknown` | - | Size (opt-level=z) | Web/PWA |

### Profile Configurations

| Profile | Opt Level | LTO | Use Case |
|---------|-----------|-----|----------|
| `dev` | 1 | No | Fast iteration |
| `release` | 3 | Thin | Production desktop |
| `wasm-release` | z | Fat | PWA bundle |
| `release-with-debug` | 3 | No | Profiling |

---

## ❌ Known Limitations

### WSL Cross-Compilation

**Issue**: winit requires display server
**Impact**: Can't run UI tests or benchmarks in headless WSL
**Workaround**: Build and test on native Windows

### sccache Connectivity

**Issue**: sccache may fail in WSL
**Workaround**: Disable temporarily with `unset RUSTC_WRAPPER`

### Property Testing

**Issue**: Optional dependencies
**Workaround**: Enable with `just test-property` or `cargo test --features property-testing`

---

## 📚 Related Documentation

- **Build Configuration**: `.cargo/config.toml`
- **Dependency Configuration**: `Cargo.toml`
- **Integration Summary**: `OPTIMIZATION_INTEGRATION_SUMMARY.md`
- **Profiling Guide**: `PROFILING_GUIDE.md`
- **SIMD Integration**: `SIMD_INTEGRATION_REPORT.md`
- **PWA Deployment**: `PWA_QUICKSTART.md`, `DEPLOYMENT.md`
- **UI Testing**: `UI_TESTING_VALIDATION_REPORT.md`

---

## 🎉 Success Criteria

✅ **Windows Builds**: MSVC and GNU toolchains supported
✅ **WASM Builds**: Development and size-optimized release profiles
✅ **PWA Deployment**: 6 deployment targets ready
✅ **Profiling**: Desktop and WASM profiling scripts integrated
✅ **Testing**: Windows-specific and cross-platform test suites
✅ **Quality**: Comprehensive linting, formatting, and security checks

---

## 🆘 Getting Help

```bash
# Show detailed help
just help

# List all commands
just --list

# Show justfile syntax
just --summary
```

For more information, see the comprehensive documentation files in the project root.
