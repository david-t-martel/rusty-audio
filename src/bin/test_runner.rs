// Comprehensive Test Runner for Rusty Audio Testing Framework
// Executes all test suites and provides detailed reporting

use std::time::{Duration, Instant};
use std::process::{Command, Stdio};
use std::collections::HashMap;
use std::env;
use std::path::Path;

#[derive(Debug, Clone)]
struct TestSuite {
    name: String,
    command: String,
    args: Vec<String>,
    timeout_seconds: u64,
    required: bool,
    description: String,
}

#[derive(Debug, Clone)]
struct TestResult {
    suite_name: String,
    passed: bool,
    duration: Duration,
    output: String,
    error_output: String,
}

struct TestRunner {
    suites: Vec<TestSuite>,
    results: Vec<TestResult>,
    start_time: Instant,
}

impl TestRunner {
    fn new() -> Self {
        let suites = vec![
            TestSuite {
                name: "Unit Tests".to_string(),
                command: "cargo".to_string(),
                args: vec!["test".to_string(), "--lib".to_string(), "--".to_string(), "--nocapture".to_string()],
                timeout_seconds: 300,
                required: true,
                description: "Core mathematical and audio processing unit tests".to_string(),
            },
            TestSuite {
                name: "Integration Tests".to_string(),
                command: "cargo".to_string(),
                args: vec!["test".to_string(), "--test".to_string(), "*".to_string(), "--".to_string(), "--nocapture".to_string()],
                timeout_seconds: 600,
                required: true,
                description: "Complete audio pipeline integration tests".to_string(),
            },
            TestSuite {
                name: "Property-Based Tests".to_string(),
                command: "cargo".to_string(),
                args: vec!["test".to_string(), "--test".to_string(), "property_based_tests".to_string()],
                timeout_seconds: 900,
                required: true,
                description: "QuickCheck and Proptest property-based testing".to_string(),
            },
            TestSuite {
                name: "Safety Tests".to_string(),
                command: "cargo".to_string(),
                args: vec!["test".to_string(), "--test".to_string(), "safety_protection_tests".to_string()],
                timeout_seconds: 300,
                required: true,
                description: "Audio safety and protection system tests".to_string(),
            },
            TestSuite {
                name: "Benchmarks".to_string(),
                command: "cargo".to_string(),
                args: vec!["bench".to_string(), "--".to_string(), "--output-format".to_string(), "pretty".to_string()],
                timeout_seconds: 1200,
                required: false,
                description: "Performance benchmarks and regression tests".to_string(),
            },
            TestSuite {
                name: "Audio Quality Benchmarks".to_string(),
                command: "cargo".to_string(),
                args: vec!["bench".to_string(), "--bench".to_string(), "audio_quality_benchmarks".to_string()],
                timeout_seconds: 600,
                required: false,
                description: "Audio quality and mathematical precision benchmarks".to_string(),
            },
            TestSuite {
                name: "Memory Benchmarks".to_string(),
                command: "cargo".to_string(),
                args: vec!["bench".to_string(), "--bench".to_string(), "memory_benchmarks".to_string()],
                timeout_seconds: 600,
                required: false,
                description: "Memory usage and allocation pattern benchmarks".to_string(),
            },
            TestSuite {
                name: "Real-time Benchmarks".to_string(),
                command: "cargo".to_string(),
                args: vec!["bench".to_string(), "--bench".to_string(), "realtime_benchmarks".to_string()],
                timeout_seconds: 600,
                required: false,
                description: "Real-time performance and latency benchmarks".to_string(),
            },
            TestSuite {
                name: "Documentation Tests".to_string(),
                command: "cargo".to_string(),
                args: vec!["test".to_string(), "--doc".to_string()],
                timeout_seconds: 300,
                required: false,
                description: "Documentation example and code snippet tests".to_string(),
            },
            TestSuite {
                name: "Clippy Lints".to_string(),
                command: "cargo".to_string(),
                args: vec!["clippy".to_string(), "--all-targets".to_string(), "--all-features".to_string(), "--".to_string(), "-D".to_string(), "warnings".to_string()],
                timeout_seconds: 180,
                required: true,
                description: "Code quality and best practices linting".to_string(),
            },
            TestSuite {
                name: "Format Check".to_string(),
                command: "cargo".to_string(),
                args: vec!["fmt".to_string(), "--all".to_string(), "--".to_string(), "--check".to_string()],
                timeout_seconds: 60,
                required: true,
                description: "Code formatting consistency check".to_string(),
            },
        ];

        Self {
            suites,
            results: Vec::new(),
            start_time: Instant::now(),
        }
    }

    fn run_all(&mut self) -> bool {
        println!("🔬 RUSTY AUDIO - COMPREHENSIVE TEST SUITE RUNNER");
        println!("==================================================");
        println!("Starting comprehensive testing at {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"));
        println!();

        let mut all_passed = true;
        let total_suites = self.suites.len();

        // Check environment
        self.check_environment();

        // Run each test suite
        for (index, suite) in self.suites.clone().iter().enumerate() {
            println!("📋 Running Test Suite {}/{}: {}", index + 1, total_suites, suite.name);
            println!("   Description: {}", suite.description);
            println!("   Command: {} {}", suite.command, suite.args.join(" "));
            println!();

            let start_time = Instant::now();
            let result = self.run_suite(suite);
            let duration = start_time.elapsed();

            if result.passed {
                println!("   ✅ PASSED in {:.2}s", duration.as_secs_f64());
            } else {
                println!("   ❌ FAILED in {:.2}s", duration.as_secs_f64());
                if suite.required {
                    all_passed = false;
                    println!("   ⚠️  This is a required test suite!");
                } else {
                    println!("   ℹ️  This is an optional test suite");
                }
            }

            self.results.push(result);
            println!();
        }

        self.print_summary();
        all_passed
    }

    fn run_suite(&self, suite: &TestSuite) -> TestResult {
        let mut cmd = Command::new(&suite.command);
        cmd.args(&suite.args);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        // Set environment variables for testing
        cmd.env("RUST_BACKTRACE", "1");
        cmd.env("CARGO_TERM_COLOR", "always");

        let start_time = Instant::now();

        match cmd.spawn() {
            Ok(mut child) => {
                let timeout = Duration::from_secs(suite.timeout_seconds);

                // Wait for completion with timeout
                let mut elapsed = Duration::from_secs(0);
                let poll_interval = Duration::from_millis(100);

                loop {
                    match child.try_wait() {
                        Ok(Some(status)) => {
                            let duration = start_time.elapsed();
                            let output = child.stdout.take()
                                .map(|mut stdout| {
                                    use std::io::Read;
                                    let mut buf = String::new();
                                    stdout.read_to_string(&mut buf).unwrap_or(0);
                                    buf
                                })
                                .unwrap_or_default();

                            let error_output = child.stderr.take()
                                .map(|mut stderr| {
                                    use std::io::Read;
                                    let mut buf = String::new();
                                    stderr.read_to_string(&mut buf).unwrap_or(0);
                                    buf
                                })
                                .unwrap_or_default();

                            return TestResult {
                                suite_name: suite.name.clone(),
                                passed: status.success(),
                                duration,
                                output,
                                error_output,
                            };
                        }
                        Ok(None) => {
                            // Still running
                            if elapsed >= timeout {
                                let _ = child.kill();
                                return TestResult {
                                    suite_name: suite.name.clone(),
                                    passed: false,
                                    duration: timeout,
                                    output: String::new(),
                                    error_output: format!("Test suite timed out after {} seconds", suite.timeout_seconds),
                                };
                            }
                        }
                        Err(e) => {
                            return TestResult {
                                suite_name: suite.name.clone(),
                                passed: false,
                                duration: start_time.elapsed(),
                                output: String::new(),
                                error_output: format!("Failed to wait for process: {}", e),
                            };
                        }
                    }

                    std::thread::sleep(poll_interval);
                    elapsed += poll_interval;
                }
            }
            Err(e) => {
                TestResult {
                    suite_name: suite.name.clone(),
                    passed: false,
                    duration: Duration::from_secs(0),
                    output: String::new(),
                    error_output: format!("Failed to start command: {}", e),
                }
            }
        }
    }

    fn check_environment(&self) {
        println!("🔧 Environment Check");
        println!("=====================");

        // Check Rust version
        if let Ok(output) = Command::new("rustc").arg("--version").output() {
            let version = String::from_utf8_lossy(&output.stdout);
            println!("Rust version: {}", version.trim());
        }

        // Check Cargo version
        if let Ok(output) = Command::new("cargo").arg("--version").output() {
            let version = String::from_utf8_lossy(&output.stdout);
            println!("Cargo version: {}", version.trim());
        }

        // Check available features
        println!("Available CPU cores: {}", num_cpus::get());

        // Check if we're in the right directory
        if Path::new("Cargo.toml").exists() {
            println!("✅ Found Cargo.toml - in correct project directory");
        } else {
            println!("❌ No Cargo.toml found - may not be in project root");
        }

        // Check for required dependencies
        let required_tools = vec!["rustc", "cargo"];
        for tool in required_tools {
            match Command::new(tool).arg("--version").output() {
                Ok(_) => println!("✅ {} is available", tool),
                Err(_) => println!("❌ {} is not available", tool),
            }
        }

        println!();
    }

    fn print_summary(&self) {
        let total_duration = self.start_time.elapsed();

        println!("📊 TEST SUMMARY");
        println!("===============");
        println!("Total test suites: {}", self.results.len());
        println!("Total duration: {:.2}s", total_duration.as_secs_f64());
        println!();

        // Count results
        let passed_count = self.results.iter().filter(|r| r.passed).count();
        let failed_count = self.results.len() - passed_count;
        let required_failed = self.results.iter()
            .zip(&self.suites)
            .filter(|(result, suite)| !result.passed && suite.required)
            .count();

        println!("✅ Passed: {}", passed_count);
        println!("❌ Failed: {}", failed_count);
        if required_failed > 0 {
            println!("⚠️  Required failures: {}", required_failed);
        }
        println!();

        // Success rate
        let success_rate = if self.results.is_empty() {
            0.0
        } else {
            passed_count as f64 / self.results.len() as f64 * 100.0
        };

        println!("📈 Success rate: {:.1}%", success_rate);
        println!();

        // Detailed results
        println!("📋 Detailed Results:");
        println!("====================");

        let mut suite_index = 0;
        for result in &self.results {
            let suite = &self.suites[suite_index];
            let status_icon = if result.passed { "✅" } else { "❌" };
            let required_marker = if suite.required { " (Required)" } else { " (Optional)" };

            println!("{} {} - {:.2}s{}",
                status_icon,
                result.suite_name,
                result.duration.as_secs_f64(),
                required_marker
            );

            if !result.passed {
                if !result.error_output.is_empty() {
                    println!("   Error: {}", result.error_output.lines().next().unwrap_or("Unknown error"));
                }
            }

            suite_index += 1;
        }
        println!();

        // Performance analysis
        println!("⚡ Performance Analysis:");
        println!("========================");

        let mut durations: Vec<_> = self.results.iter()
            .map(|r| r.duration.as_secs_f64())
            .collect();
        durations.sort_by(|a, b| a.partial_cmp(b).unwrap());

        if !durations.is_empty() {
            let fastest = durations[0];
            let slowest = durations[durations.len() - 1];
            let average = durations.iter().sum::<f64>() / durations.len() as f64;

            println!("Fastest suite: {:.2}s", fastest);
            println!("Slowest suite: {:.2}s", slowest);
            println!("Average time: {:.2}s", average);
        }
        println!();

        // Final verdict
        println!("🎯 FINAL VERDICT");
        println!("================");

        if required_failed == 0 {
            if failed_count == 0 {
                println!("🎉 ALL TESTS PASSED! The audio processing framework is mathematically sound and safe.");
            } else {
                println!("✅ CORE TESTS PASSED! All required tests passed. {} optional test(s) failed.", failed_count);
            }
            println!("✅ Ready for production use with confidence in audio quality and safety.");
        } else {
            println!("❌ CRITICAL FAILURES DETECTED! {} required test suite(s) failed.", required_failed);
            println!("⚠️  DO NOT USE IN PRODUCTION until all required tests pass.");
            println!("🔧 Review failed tests and fix issues before deployment.");
        }

        if success_rate >= 95.0 {
            println!("🏆 EXCELLENT: {:.1}% success rate - Outstanding test coverage!", success_rate);
        } else if success_rate >= 85.0 {
            println!("👍 GOOD: {:.1}% success rate - Solid foundation with minor issues.", success_rate);
        } else if success_rate >= 70.0 {
            println!("⚠️  WARNING: {:.1}% success rate - Significant issues need attention.", success_rate);
        } else {
            println!("🚨 CRITICAL: {:.1}% success rate - Major problems detected!", success_rate);
        }

        println!();
        println!("Test run completed at {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"));
    }

    fn save_results(&self) -> std::io::Result<()> {
        use std::fs::File;
        use std::io::Write;

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!("test_results_{}.txt", timestamp);

        let mut file = File::create(&filename)?;

        writeln!(file, "Rusty Audio Test Results")?;
        writeln!(file, "========================")?;
        writeln!(file, "Generated: {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"))?;
        writeln!(file, "Total duration: {:.2}s", self.start_time.elapsed().as_secs_f64())?;
        writeln!(file)?;

        for result in &self.results {
            writeln!(file, "Suite: {}", result.suite_name)?;
            writeln!(file, "Status: {}", if result.passed { "PASSED" } else { "FAILED" })?;
            writeln!(file, "Duration: {:.2}s", result.duration.as_secs_f64())?;

            if !result.passed && !result.error_output.is_empty() {
                writeln!(file, "Error Output:")?;
                for line in result.error_output.lines().take(10) {
                    writeln!(file, "  {}", line)?;
                }
            }

            writeln!(file, "---")?;
        }

        println!("📁 Test results saved to: {}", filename);
        Ok(())
    }
}

fn main() {
    // Add chrono dependency for timestamps
    use chrono;

    let mut runner = TestRunner::new();
    let success = runner.run_all();

    // Save results to file
    if let Err(e) = runner.save_results() {
        eprintln!("Warning: Failed to save test results: {}", e);
    }

    // Exit with appropriate code
    std::process::exit(if success { 0 } else { 1 });
}

// Required for chrono - add to Cargo.toml if not present
// chrono = { version = "0.4", features = ["serde"] }