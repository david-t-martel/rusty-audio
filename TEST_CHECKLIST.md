# Rusty-Audio Test Suite Checklist

## Pre-Testing: Fix Codebase Issues

### Critical Compilation Errors

- [ ] **Fix MMCSS HANDLE Import** (`src/audio/mmcss.rs`)
  ```rust
  // Line 85 and 160: Change from
  windows::Win32::System::Threading::HANDLE
  // To
  windows::Win32::Foundation::HANDLE
  ```
  **Command**: Edit `src/audio/mmcss.rs` lines 85 and 160

- [ ] **Fix Router Borrowing Issue** (`src/audio/router.rs:462`)
  ```rust
  // Clone routes before iteration to avoid borrow conflict
  let routes: Vec<_> = state.routes.values().cloned().collect();
  for route in routes {
      if let Some(source) = state.sources.get_mut(&route.source) {
          // ...
      }
  }
  ```
  **Command**: Edit `src/audio/router.rs` line 462

- [ ] **Verify Library Compiles**
  ```bash
  cargo check --lib --all-features
  ```
  **Expected**: No errors, warnings OK

---

## Phase 1: Unit Tests (Week 1)

### Backend Trait Tests

- [ ] **Compile backend_trait_tests.rs**
  ```bash
  cargo test --test backend_trait_tests --no-run
  ```

- [ ] **Run dyn-safety tests**
  ```bash
  cargo test --test backend_trait_tests test_backend_trait_is_dyn_safe
  ```

- [ ] **Run backend lifecycle tests**
  ```bash
  cargo test --test backend_trait_tests test_backend_lifecycle
  ```

- [ ] **Run volume clamping tests**
  ```bash
  cargo test --test backend_trait_tests test_backend_volume_clamping
  ```

- [ ] **Run EQ band tests**
  ```bash
  cargo test --test backend_trait_tests test_backend_eq_band_range
  ```

- [ ] **Run spectrum data tests**
  ```bash
  cargo test --test backend_trait_tests test_backend_spectrum_data_size
  ```

### EQ Functionality Tests

- [ ] **Compile eq_functionality_tests.rs**
  ```bash
  cargo test --test eq_functionality_tests --no-run
  ```

- [ ] **Run frequency band tests**
  ```bash
  cargo test --test eq_functionality_tests test_eq_frequency_bands
  ```

- [ ] **Run gain clamping tests**
  ```bash
  cargo test --test eq_functionality_tests test_eq_gain_clamping
  ```

- [ ] **Run dB conversion tests**
  ```bash
  cargo test --test eq_functionality_tests test_eq_db_to_linear_conversion
  ```

- [ ] **Run filter response tests**
  ```bash
  cargo test --test eq_functionality_tests test_eq_filter_at_center_frequency
  ```

- [ ] **Run signal processing tests**
  ```bash
  cargo test --test eq_functionality_tests test_eq_unity_gain_bypass
  cargo test --test eq_functionality_tests test_eq_boost_increases_amplitude
  cargo test --test eq_functionality_tests test_eq_cut_decreases_amplitude
  ```

### Theme System Tests

- [ ] **Compile theme_system_tests.rs**
  ```bash
  cargo test --test theme_system_tests --no-run
  ```

- [ ] **Run theme enumeration tests**
  ```bash
  cargo test --test theme_system_tests test_all_themes_available
  ```

- [ ] **Run dark theme tests**
  ```bash
  cargo test --test theme_system_tests test_dark_theme_colors
  ```

- [ ] **Run light theme tests**
  ```bash
  cargo test --test theme_system_tests test_light_theme_colors
  ```

- [ ] **Run high contrast tests**
  ```bash
  cargo test --test theme_system_tests test_high_contrast_theme
  ```

- [ ] **Run spectrum gradient tests**
  ```bash
  cargo test --test theme_system_tests test_spectrum_gradient_blue_to_red
  cargo test --test theme_system_tests test_spectrum_gradient_interpolation
  ```

- [ ] **Run WCAG compliance tests**
  ```bash
  cargo test --test theme_system_tests test_wcag_aa_compliance
  cargo test --test theme_system_tests test_high_contrast_wcag_aaa
  ```

---

## Phase 2: Integration Tests (Week 2)

### Audio Graph Integration

- [ ] **Compile audio_graph_integration_tests.rs**
  ```bash
  cargo test --test audio_graph_integration_tests --no-run
  ```

- [ ] **Run complete chain test**
  ```bash
  cargo test --test audio_graph_integration_tests test_audio_graph_complete_chain
  ```

- [ ] **Run source to EQ tests**
  ```bash
  cargo test --test audio_graph_integration_tests test_source_to_eq_connection
  ```

- [ ] **Run EQ to analyser tests**
  ```bash
  cargo test --test audio_graph_integration_tests test_eq_to_analyser_connection
  ```

- [ ] **Run gain node tests**
  ```bash
  cargo test --test audio_graph_integration_tests test_gain_node_volume_control
  cargo test --test audio_graph_integration_tests test_gain_node_mute
  ```

- [ ] **Run dynamic routing tests**
  ```bash
  cargo test --test audio_graph_integration_tests test_eq_changes_during_playback
  cargo test --test audio_graph_integration_tests test_eq_reset_during_playback
  ```

- [ ] **Run concurrent access tests**
  ```bash
  cargo test --test audio_graph_integration_tests test_concurrent_eq_access
  cargo test --test audio_graph_integration_tests test_concurrent_spectrum_reads
  ```

---

## Phase 3: Property-Based Tests (Week 2-3)

### EQ Properties

- [ ] **Run EQ gain clamping property**
  ```bash
  PROPTEST_CASES=1000 cargo test --test property_based_tests prop_eq_gain_clamping
  ```

- [ ] **Run dB conversion property**
  ```bash
  PROPTEST_CASES=1000 cargo test --test property_based_tests prop_eq_db_to_linear_conversion
  ```

- [ ] **Run band independence property**
  ```bash
  PROPTEST_CASES=1000 cargo test --test property_based_tests prop_eq_band_independence
  ```

### Spectrum Properties

- [ ] **Run spectrum normalization property**
  ```bash
  PROPTEST_CASES=1000 cargo test --test property_based_tests prop_spectrum_normalization
  ```

- [ ] **Run spectrum bin count property**
  ```bash
  cargo test --test property_based_tests prop_spectrum_bin_count
  ```

- [ ] **Run gradient interpolation property**
  ```bash
  PROPTEST_CASES=1000 cargo test --test property_based_tests prop_spectrum_gradient_interpolation
  ```

### Audio Buffer Properties

- [ ] **Run buffer underrun property**
  ```bash
  PROPTEST_CASES=1000 cargo test --test property_based_tests prop_audio_buffer_underrun_handling
  ```

- [ ] **Run sample rate conversion property**
  ```bash
  PROPTEST_CASES=1000 cargo test --test property_based_tests prop_sample_rate_conversion
  ```

### Volume Properties

- [ ] **Run volume clamping property**
  ```bash
  PROPTEST_CASES=1000 cargo test --test property_based_tests prop_volume_clamping
  ```

- [ ] **Run volume application property**
  ```bash
  PROPTEST_CASES=1000 cargo test --test property_based_tests prop_volume_application
  ```

---

## Phase 4: Platform-Specific Tests (Week 3)

### Desktop Tests (All Platforms)

- [ ] **Run CPAL backend tests**
  ```bash
  cargo test --test platform_specific_tests test_cpal_backend_available
  cargo test --test platform_specific_tests test_cpal_device_enumeration
  ```

- [ ] **Run hybrid backend tests**
  ```bash
  cargo test --test platform_specific_tests test_hybrid_backend_fallback
  cargo test --test platform_specific_tests test_hybrid_backend_preference_order
  ```

### Windows-Specific Tests

- [ ] **Run ASIO backend test**
  ```bash
  cargo test --test platform_specific_tests --features asio test_asio_backend_compilation
  ```

- [ ] **Run MMCSS tests** (CRITICAL REGRESSION)
  ```bash
  cargo test --test platform_specific_tests --features asio test_mmcss_handle_import
  cargo test --test platform_specific_tests --features asio test_mmcss_thread_registration
  ```

- [ ] **Run Windows audio stack tests**
  ```bash
  cargo test --test platform_specific_tests test_windows_audio_stack_config
  ```

### Linux-Specific Tests

- [ ] **Run ALSA/PulseAudio tests**
  ```bash
  cargo test --test platform_specific_tests test_alsa_or_pulseaudio_backend
  ```

- [ ] **Run realtime scheduling tests**
  ```bash
  cargo test --test platform_specific_tests test_linux_realtime_scheduling
  ```

### macOS-Specific Tests

- [ ] **Run CoreAudio tests**
  ```bash
  cargo test --test platform_specific_tests test_coreaudio_backend
  ```

- [ ] **Run exclusive mode tests**
  ```bash
  cargo test --test platform_specific_tests test_macos_exclusive_mode
  ```

### WASM Tests

- [ ] **Build WASM**
  ```bash
  wasm-pack build --target web --dev
  ```

- [ ] **Run WASM tests**
  ```bash
  wasm-pack test --headless --chrome --test platform_specific_tests
  ```

- [ ] **Test Web Audio API**
  ```bash
  wasm-pack test --headless --chrome -- test_web_audio_backend_available
  wasm-pack test --headless --chrome -- test_web_audio_eq_node_creation
  wasm-pack test --headless --chrome -- test_web_audio_analyser_node
  ```

---

## Phase 5: Regression Tests (Week 3)

### PR #5 Regressions (CRITICAL)

- [ ] **Test backend trait dyn-safety** (PR #5 Issue 1)
  ```bash
  cargo test --test regression_tests regression_pr5_backend_trait_dyn_safety
  ```

- [ ] **Test MMCSS HANDLE import** (PR #5 Issue 2)
  ```bash
  cargo test --test regression_tests --features asio regression_pr5_windows_mmcss_handle_import
  ```

- [ ] **Test EQ audio graph connection** (PR #5 Issue 3)
  ```bash
  cargo test --test regression_tests regression_pr5_eq_audio_graph_connection
  ```

- [ ] **Test theme application** (PR #5 Issue 4)
  ```bash
  cargo test --test regression_tests regression_pr5_theme_application
  ```

- [ ] **Test spectrum gradient** (PR #5 Issue 5)
  ```bash
  cargo test --test regression_tests regression_pr5_spectrum_gradient_smoothness
  ```

### Compilation Regressions

- [ ] **Test all targets compile**
  ```bash
  cargo test --test regression_tests regression_compilation_all_targets
  ```

- [ ] **Test ASIO feature compiles** (Windows)
  ```bash
  cargo test --test regression_tests --features asio regression_compilation_asio_feature
  ```

- [ ] **Test WASM compiles**
  ```bash
  wasm-pack test --headless --chrome -- regression_compilation_wasm_web_sys
  ```

### Panic Regressions

- [ ] **Test no panics in audio callback**
  ```bash
  cargo test --test regression_tests regression_no_panic_in_audio_callback
  ```

- [ ] **Test no panics with invalid input**
  ```bash
  cargo test --test regression_tests regression_no_panic_invalid_input
  ```

---

## Phase 6: Coverage and Quality (Week 4)

### Coverage Analysis

- [ ] **Generate coverage report**
  ```bash
  cargo tarpaulin --out Html --all-features
  ```

- [ ] **Check coverage threshold (85%)**
  ```bash
  cargo tarpaulin --all-features | grep "coverage"
  ```

- [ ] **Identify uncovered code**
  ```bash
  open tarpaulin-report.html
  # Review red/yellow lines
  ```

- [ ] **Add tests for uncovered code**

- [ ] **Verify 85%+ coverage achieved**

### Code Quality

- [ ] **Run rustfmt**
  ```bash
  cargo fmt
  ```

- [ ] **Run clippy (no warnings)**
  ```bash
  cargo clippy --all-targets --all-features -- -D warnings
  ```

- [ ] **Check documentation**
  ```bash
  cargo doc --no-deps --all-features
  ```

- [ ] **Run security audit**
  ```bash
  cargo audit
  ```

### Benchmarks

- [ ] **Compile benchmarks**
  ```bash
  cargo bench --no-run --all-features
  ```

- [ ] **Run audio benchmarks**
  ```bash
  cargo bench --bench audio_benchmarks
  ```

- [ ] **Run performance benchmarks**
  ```bash
  cargo bench --bench performance_benchmarks
  ```

- [ ] **Review benchmark results**
  ```bash
  open target/criterion/report/index.html
  ```

---

## Phase 7: CI/CD Setup (Week 4)

### GitHub Actions Workflow

- [ ] **Validate workflow YAML**
  ```bash
  # Use GitHub CLI or online validator
  gh workflow view test.yml
  ```

- [ ] **Push to trigger CI**
  ```bash
  git add .github/workflows/test.yml
  git commit -m "Add comprehensive test CI/CD pipeline"
  git push
  ```

- [ ] **Monitor CI run**
  ```bash
  gh run watch
  ```

- [ ] **Verify all jobs pass**
  - test-unit (Linux, Windows, macOS × stable, nightly)
  - test-integration
  - test-property
  - test-regression
  - test-platform-windows
  - test-platform-linux
  - test-platform-macos
  - test-wasm
  - coverage
  - quality
  - security

- [ ] **Fix any CI failures**

- [ ] **Set up branch protection**
  - Require tests to pass before merge
  - Require 85% coverage
  - Require code review

---

## Final Verification

### All Tests Pass

- [ ] **Run full test suite**
  ```bash
  cargo test --all-features
  ```

- [ ] **Run all property tests (10,000 cases)**
  ```bash
  PROPTEST_CASES=10000 cargo test --test property_based_tests
  ```

- [ ] **Run all platform tests**
  ```bash
  cargo test --test platform_specific_tests
  ```

- [ ] **Verify WASM build**
  ```bash
  wasm-pack build --target web --release
  ```

### Coverage Meets Target

- [ ] **Overall coverage ≥ 85%**
- [ ] **Backend coverage ≥ 95%**
- [ ] **EQ coverage ≥ 95%**
- [ ] **Hybrid backend coverage ≥ 95%**
- [ ] **UI coverage ≥ 70%**

### Documentation Complete

- [ ] **TESTING.md reviewed and accurate**
- [ ] **TEST_SUMMARY.md up to date**
- [ ] **TEST_IMPLEMENTATION_REPORT.md complete**
- [ ] **All test files have header comments**

### CI/CD Operational

- [ ] **All CI jobs passing**
- [ ] **Coverage uploaded to Codecov**
- [ ] **Security audit clean**
- [ ] **No clippy warnings**
- [ ] **No format violations**

---

## Success Criteria

✅ **All tests pass** (unit, integration, property, platform, regression)
✅ **Coverage ≥ 85%** (verified by Tarpaulin)
✅ **No compilation errors** (cargo check succeeds)
✅ **No clippy warnings** (cargo clippy clean)
✅ **Code formatted** (cargo fmt applied)
✅ **CI/CD operational** (all jobs passing)
✅ **Documentation complete** (TESTING.md, TEST_SUMMARY.md)
✅ **Benchmarks running** (performance tracked)

---

## Notes

- Check off items as completed
- Add notes for failures or issues
- Update estimated completion dates
- Document any deviations from plan

**Last Updated**: 2025-01-16
**Status**: ⚠️ Blocked by codebase compilation errors
**Next Action**: Fix MMCSS HANDLE import and router borrowing issue
