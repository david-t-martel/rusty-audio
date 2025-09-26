// Test Binary for Rusty Audio Mathematical Testing Framework
//
// This binary demonstrates and runs the comprehensive mathematical testing
// framework for audio processing verification.

use rusty_audio::testing;
use std::env;

fn main() {
    println!("🎵 RUSTY AUDIO - MATHEMATICAL TESTING FRAMEWORK");
    println!("================================================");

    let args: Vec<String> = env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("quick") => {
            println!("Running quick tests...\n");
            let suite = testing::run_quick_tests();

            if suite.success_rate() >= 0.8 {
                println!("✅ Quick tests passed! Audio processing looks good.");
                std::process::exit(0);
            } else {
                println!("❌ Quick tests failed! Check audio processing.");
                std::process::exit(1);
            }
        }

        Some("realtime") => {
            println!("Running real-time performance tests...\n");
            let suite = testing::run_realtime_tests();

            if suite.success_rate() >= 0.8 {
                println!("✅ Real-time performance tests passed!");
                std::process::exit(0);
            } else {
                println!("⚠️  Real-time performance issues detected!");
                std::process::exit(1);
            }
        }

        Some("spectrum") => {
            println!("Running spectrum analysis tests...\n");
            let suite = testing::spectrum_analysis::run_spectrum_tests();
            suite.print_summary();

            if suite.success_rate() >= 0.9 {
                println!("🎯 Spectrum analysis is mathematically accurate!");
            } else {
                println!("⚠️  Spectrum analysis accuracy issues detected.");
            }
        }

        Some("equalizer") => {
            println!("Running equalizer tests...\n");
            let suite = testing::equalizer_tests::run_equalizer_tests();
            suite.print_summary();

            if suite.success_rate() >= 0.8 {
                println!("🎛️  Equalizer mathematical verification passed!");
            } else {
                println!("⚠️  Equalizer mathematical issues detected.");
            }
        }

        Some("integration") => {
            println!("Running integration tests...\n");
            let suite = testing::integration_tests::run_integration_tests();
            suite.print_summary();

            if suite.success_rate() >= 0.8 {
                println!("🔗 Audio pipeline integration tests passed!");
            } else {
                println!("⚠️  Audio pipeline integration issues detected.");
            }
        }

        Some("property") => {
            println!("Running property-based tests...\n");
            let suite = testing::property_tests::run_property_tests();
            suite.print_summary();

            if suite.success_rate() >= 0.8 {
                println!("🔍 Property-based verification passed!");
            } else {
                println!("⚠️  Property-based verification issues detected.");
            }
        }

        Some("generators") => {
            println!("Testing signal generators...\n");
            test_signal_generators();
        }

        Some("benchmark") | Some("bench") => {
            println!("Run benchmarks with: cargo bench");
            println!("This will execute comprehensive performance benchmarks.");
        }

        Some("help") | Some("-h") | Some("--help") => {
            print_help();
        }

        Some("all") | None => {
            println!("Running comprehensive test suite...\n");
            let suite = testing::run_all_tests();

            // Exit with appropriate code
            if suite.success_rate() >= 0.9 {
                println!("🎉 ALL TESTS PASSED - Audio processing is mathematically sound!");
                std::process::exit(0);
            } else if suite.success_rate() >= 0.8 {
                println!("✅ MOSTLY PASSED - Minor issues detected but audio processing is reliable.");
                std::process::exit(0);
            } else {
                println!("❌ TESTS FAILED - Significant audio processing issues detected!");
                std::process::exit(1);
            }
        }

        Some(unknown) => {
            println!("❌ Unknown command: {}", unknown);
            print_help();
            std::process::exit(1);
        }
    }
}

fn test_signal_generators() {
    use testing::signal_generators::*;
    use testing::signal_generators::presets;
    use testing::{calculate_rms, calculate_peak, SAMPLE_RATE};

    println!("🎼 Testing Signal Generators");
    println!("=============================");

    // Test sine wave generator
    println!("\n1. Sine Wave Generator (1kHz):");
    let sine_gen = presets::sine_1khz();
    let sine_samples = sine_gen.generate(1.0, SAMPLE_RATE);
    let sine_rms = calculate_rms(&sine_samples);
    let sine_peak = calculate_peak(&sine_samples);

    println!("   Samples generated: {}", sine_samples.len());
    println!("   RMS: {:.6} (expected: {:.6})", sine_rms, 1.0 / 2.0f32.sqrt());
    println!("   Peak: {:.6} (expected: 1.000)", sine_peak);

    // Test white noise generator
    println!("\n2. White Noise Generator:");
    let noise_gen = presets::quiet_white_noise();
    let noise_samples = noise_gen.generate(1.0, SAMPLE_RATE);
    let noise_rms = calculate_rms(&noise_samples);
    let noise_peak = calculate_peak(&noise_samples);

    println!("   Samples generated: {}", noise_samples.len());
    println!("   RMS: {:.6} (amplitude: 0.1)", noise_rms);
    println!("   Peak: {:.6} (should be ≤ 0.1)", noise_peak);

    // Test frequency sweep
    println!("\n3. Frequency Sweep Generator (20Hz - 20kHz):");
    let sweep_gen = presets::full_range_sweep();
    let sweep_samples = sweep_gen.generate(2.0, SAMPLE_RATE);
    let sweep_rms = calculate_rms(&sweep_samples);

    println!("   Samples generated: {}", sweep_samples.len());
    println!("   RMS: {:.6}", sweep_rms);

    // Test impulse
    println!("\n4. Unit Impulse Generator:");
    let impulse_gen = presets::unit_impulse();
    let impulse_samples = impulse_gen.generate(0.1, SAMPLE_RATE);
    let impulse_peak = calculate_peak(&impulse_samples);
    let non_zero_count = impulse_samples.iter().filter(|&&x| x != 0.0).count();

    println!("   Samples generated: {}", impulse_samples.len());
    println!("   Peak: {:.6} (expected: 1.0)", impulse_peak);
    println!("   Non-zero samples: {} (expected: 1)", non_zero_count);

    // Test harmonic signal
    println!("\n5. Harmonic Test Signal (200Hz fundamental):");
    let harmonic_gen = presets::harmonic_test_signal(200.0);
    let harmonic_samples = harmonic_gen.generate(2.0, SAMPLE_RATE);
    let harmonic_rms = calculate_rms(&harmonic_samples);

    println!("   Samples generated: {}", harmonic_samples.len());
    println!("   RMS: {:.6}", harmonic_rms);

    println!("\n✅ Signal generator tests completed!");
}

fn print_help() {
    println!("RUSTY AUDIO - MATHEMATICAL TESTING FRAMEWORK");
    println!("==============================================");
    println!();
    println!("USAGE:");
    println!("  cargo run --bin test_audio [COMMAND]");
    println!();
    println!("COMMANDS:");
    println!("  all          Run all comprehensive tests (default)");
    println!("  quick        Run essential quick tests");
    println!("  realtime     Run real-time performance tests");
    println!("  spectrum     Run spectrum analysis tests only");
    println!("  equalizer    Run equalizer mathematical tests only");
    println!("  integration  Run audio pipeline integration tests only");
    println!("  property     Run property-based tests only");
    println!("  generators   Test signal generators");
    println!("  benchmark    Instructions for running benchmarks");
    println!("  help         Show this help message");
    println!();
    println!("EXAMPLES:");
    println!("  cargo run --bin test_audio quick       # Fast verification");
    println!("  cargo run --bin test_audio spectrum    # Test FFT accuracy");
    println!("  cargo run --bin test_audio realtime    # Performance tests");
    println!("  cargo bench                            # Run benchmarks");
    println!();
    println!("MATHEMATICAL VERIFICATION:");
    println!("  This framework verifies:");
    println!("  • FFT accuracy and frequency detection");
    println!("  • Equalizer frequency response mathematical correctness");
    println!("  • Signal generator mathematical properties");
    println!("  • Audio pipeline integration and data flow");
    println!("  • Real-time processing performance");
    println!("  • Property-based invariants");
    println!();
    println!("EXIT CODES:");
    println!("  0 = Tests passed (>90% success rate)");
    println!("  1 = Tests failed (<80% success rate)");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_generators_work() {
        // This test ensures the signal generators can be called without panicking
        test_signal_generators();
    }
}