// Simple UI Test Automation Script for Rusty Audio
//
// This script provides a convenient way to run UI and UX tests for the
// car stereo-style interface with HiDPI optimization.

use rusty_audio::testing::{
    ui_tests::{UiTestRunner, run_quick_ui_tests},
    visual_regression::{run_visual_regression_tests, run_quick_visual_tests},
    audio_feature_tests::{AudioFeatureTestRunner, run_quick_audio_feature_tests},
};
use std::env;
use std::path::PathBuf;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let test_mode = args.get(1).map(|s| s.as_str()).unwrap_or("quick");

    println!("🎨 RUSTY AUDIO - UI/UX TEST AUTOMATION");
    println!("======================================");
    println!("Mode: {}", test_mode);
    println!("");

    let start_time = Instant::now();
    let success = match test_mode {
        "quick" => run_quick_tests(),
        "comprehensive" => run_comprehensive_tests(),
        "ui" => run_ui_only_tests(),
        "visual" => run_visual_only_tests(),
        "audio" => run_audio_only_tests(),
        "all" => run_all_test_suites(),
        _ => {
            println!("Usage: {} [quick|comprehensive|ui|visual|audio|all]", args[0]);
            println!("  quick        - Quick test suite (default)");
            println!("  comprehensive - Full comprehensive testing");
            println!("  ui           - UI component tests only");
            println!("  visual       - Visual regression tests only");
            println!("  audio        - Audio feature tests only");
            println!("  all          - All test suites");
            return Ok(());
        }
    }?;

    let total_time = start_time.elapsed();

    println!("\n🏁 UI/UX TEST AUTOMATION COMPLETE");
    println!("==================================");
    println!("Total execution time: {:.2}s", total_time.as_secs_f32());
    println!("Result: {}", if success { "✅ SUCCESS" } else { "❌ FAILURE" });

    if !success {
        println!("\nSome tests failed. Review the output above for details.");
        std::process::exit(1);
    }

    println!("\n📋 Next Steps:");
    println!("1. Review any failed tests above");
    println!("2. Run manual testing procedures from TESTING_PROCEDURES.md");
    println!("3. Verify HiDPI scaling on actual hardware");
    println!("4. Check validation criteria in VALIDATION_CRITERIA.md");

    Ok(())
}

fn run_quick_tests() -> Result<bool, Box<dyn std::error::Error>> {
    println!("🚀 Running Quick UI/UX Tests");
    println!("============================");

    let mut all_passed = true;

    // Quick UI tests
    println!("\n1️⃣  Quick UI Component Tests");
    let ui_suite = run_quick_ui_tests();
    let ui_success = ui_suite.success_rate() >= 0.85;
    all_passed &= ui_success;
    println!("   UI Tests: {:.1}% pass rate", ui_suite.success_rate() * 100.0);

    // Quick visual tests
    println!("\n2️⃣  Quick Visual Regression Tests");
    let test_dir = PathBuf::from("test_data");
    let visual_suite = run_quick_visual_tests(&test_dir)?;
    let visual_success = visual_suite.success_rate() >= 0.85;
    all_passed &= visual_success;
    println!("   Visual Tests: {:.1}% pass rate", visual_suite.success_rate() * 100.0);

    // Quick audio tests
    println!("\n3️⃣  Quick Audio Feature Tests");
    let audio_suite = run_quick_audio_feature_tests();
    let audio_success = audio_suite.success_rate() >= 0.85;
    all_passed &= audio_success;
    println!("   Audio Tests: {:.1}% pass rate", audio_suite.success_rate() * 100.0);

    Ok(all_passed)
}

fn run_comprehensive_tests() -> Result<bool, Box<dyn std::error::Error>> {
    println!("🎯 Running Comprehensive UI/UX Tests");
    println!("====================================");

    let mut all_passed = true;

    // Comprehensive UI tests
    println!("\n1️⃣  Comprehensive UI Component Tests");
    let mut ui_runner = UiTestRunner::new();
    ui_runner.run_comprehensive_ui_tests();
    let ui_success = calculate_ui_success_rate(&ui_runner) >= 0.95;
    all_passed &= ui_success;

    // Comprehensive visual tests
    println!("\n2️⃣  Comprehensive Visual Regression Tests");
    let test_dir = PathBuf::from("test_data");
    let visual_suite = run_visual_regression_tests(&test_dir)?;
    let visual_success = visual_suite.success_rate() >= 0.98;
    all_passed &= visual_success;

    // Comprehensive audio tests
    println!("\n3️⃣  Comprehensive Audio Feature Tests");
    let mut audio_runner = AudioFeatureTestRunner::new();
    audio_runner.run_comprehensive_audio_tests();
    let audio_success = calculate_audio_success_rate(&audio_runner) >= 0.95;
    all_passed &= audio_success;

    Ok(all_passed)
}

fn run_ui_only_tests() -> Result<bool, Box<dyn std::error::Error>> {
    println!("🎨 Running UI Component Tests Only");
    println!("==================================");

    let mut ui_runner = UiTestRunner::new();
    ui_runner.run_comprehensive_ui_tests();
    let success_rate = calculate_ui_success_rate(&ui_runner);

    println!("\n🎯 UI Test Results: {:.1}% pass rate", success_rate * 100.0);
    Ok(success_rate >= 0.95)
}

fn run_visual_only_tests() -> Result<bool, Box<dyn std::error::Error>> {
    println!("📸 Running Visual Regression Tests Only");
    println!("=======================================");

    let test_dir = PathBuf::from("test_data");
    let visual_suite = run_visual_regression_tests(&test_dir)?;
    let success_rate = visual_suite.success_rate();

    println!("\n🎯 Visual Test Results: {:.1}% pass rate", success_rate * 100.0);
    Ok(success_rate >= 0.98)
}

fn run_audio_only_tests() -> Result<bool, Box<dyn std::error::Error>> {
    println!("🎵 Running Audio Feature Tests Only");
    println!("===================================");

    let mut audio_runner = AudioFeatureTestRunner::new();
    audio_runner.run_comprehensive_audio_tests();
    let success_rate = calculate_audio_success_rate(&audio_runner);

    println!("\n🎯 Audio Test Results: {:.1}% pass rate", success_rate * 100.0);
    Ok(success_rate >= 0.95)
}

fn run_all_test_suites() -> Result<bool, Box<dyn std::error::Error>> {
    println!("🎯 Running All Test Suites");
    println!("==========================");

    let mut all_passed = true;

    // Mathematical and core tests
    println!("\n1️⃣  Mathematical and Core Tests");
    let math_suite = rusty_audio::testing::run_all_tests();
    let math_success = math_suite.success_rate() >= 0.95;
    all_passed &= math_success;
    println!("   Math Tests: {:.1}% pass rate", math_suite.success_rate() * 100.0);

    // UI tests
    println!("\n2️⃣  UI Component Tests");
    let mut ui_runner = UiTestRunner::new();
    ui_runner.run_comprehensive_ui_tests();
    let ui_success = calculate_ui_success_rate(&ui_runner) >= 0.95;
    all_passed &= ui_success;
    println!("   UI Tests: {:.1}% pass rate", calculate_ui_success_rate(&ui_runner) * 100.0);

    // Visual tests
    println!("\n3️⃣  Visual Regression Tests");
    let test_dir = PathBuf::from("test_data");
    let visual_suite = run_visual_regression_tests(&test_dir)?;
    let visual_success = visual_suite.success_rate() >= 0.98;
    all_passed &= visual_success;
    println!("   Visual Tests: {:.1}% pass rate", visual_suite.success_rate() * 100.0);

    // Audio tests
    println!("\n4️⃣  Audio Feature Tests");
    let mut audio_runner = AudioFeatureTestRunner::new();
    audio_runner.run_comprehensive_audio_tests();
    let audio_success = calculate_audio_success_rate(&audio_runner) >= 0.95;
    all_passed &= audio_success;
    println!("   Audio Tests: {:.1}% pass rate", calculate_audio_success_rate(&audio_runner) * 100.0);

    // Performance tests
    println!("\n5️⃣  Performance Tests");
    let perf_suite = rusty_audio::testing::run_realtime_tests();
    let perf_success = perf_suite.success_rate() >= 0.90;
    all_passed &= perf_success;
    println!("   Performance Tests: {:.1}% pass rate", perf_suite.success_rate() * 100.0);

    Ok(all_passed)
}

fn calculate_ui_success_rate(ui_runner: &UiTestRunner) -> f32 {
    let mut total_tests = 0;
    let mut total_passed = 0;

    if let Some(responsive) = &ui_runner.responsive_results {
        total_tests += responsive.results.len();
        total_passed += responsive.passed_count();
    }

    if let Some(interaction) = &ui_runner.interaction_results {
        total_tests += interaction.results.len();
        total_passed += interaction.passed_count();
    }

    if let Some(accessibility) = &ui_runner.accessibility_results {
        total_tests += accessibility.results.len();
        total_passed += accessibility.passed_count();
    }

    if let Some(performance) = &ui_runner.performance_results {
        total_tests += performance.results.len();
        total_passed += performance.passed_count();
    }

    if total_tests > 0 {
        total_passed as f32 / total_tests as f32
    } else {
        0.0
    }
}

fn calculate_audio_success_rate(audio_runner: &AudioFeatureTestRunner) -> f32 {
    let mut total_tests = 0;
    let mut total_passed = 0;

    if let Some(playback) = &audio_runner.playback_results {
        total_tests += playback.results.len();
        total_passed += playback.passed_count();
    }

    if let Some(equalizer) = &audio_runner.equalizer_results {
        total_tests += equalizer.results.len();
        total_passed += equalizer.passed_count();
    }

    if let Some(generator) = &audio_runner.generator_results {
        total_tests += generator.results.len();
        total_passed += generator.passed_count();
    }

    if let Some(spectrum) = &audio_runner.spectrum_results {
        total_tests += spectrum.results.len();
        total_passed += spectrum.passed_count();
    }

    if total_tests > 0 {
        total_passed as f32 / total_tests as f32
    } else {
        0.0
    }
}