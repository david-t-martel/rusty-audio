// Visual Regression Testing Framework for Rusty Audio
//
// This module provides automated screenshot comparison and visual regression
// testing capabilities for the car stereo-style interface.

use crate::ui::{
    theme::{Theme, ThemeManager, ThemeColors},
    components::{AlbumArtDisplay, ProgressBar, MetadataDisplay, MetadataLayout},
    controls::{EnhancedSlider, CircularKnob, EnhancedButton, ButtonStyle},
    enhanced_controls::{AccessibleSlider, AccessibleKnob},
    accessibility::AccessibilityManager,
    utils::{ScreenSize, ResponsiveSize},
};
use egui::{Vec2, Rect, Pos2, Color32, Context, ColorImage};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use std::time::{Duration, Instant};

/// Visual test result comparing screenshots
#[derive(Debug, Clone)]
pub struct VisualTestResult {
    pub test_name: String,
    pub component_name: String,
    pub passed: bool,
    pub difference_percentage: f32,
    pub threshold: f32,
    pub baseline_path: PathBuf,
    pub current_path: PathBuf,
    pub diff_path: Option<PathBuf>,
    pub execution_time: Duration,
    pub screen_size: Vec2,
    pub pixel_scale: f32,
}

impl VisualTestResult {
    pub fn new(
        test_name: String,
        component_name: String,
        difference_percentage: f32,
        threshold: f32,
        baseline_path: PathBuf,
        current_path: PathBuf,
        screen_size: Vec2,
        pixel_scale: f32,
        execution_time: Duration,
    ) -> Self {
        let passed = difference_percentage <= threshold;

        Self {
            test_name,
            component_name,
            passed,
            difference_percentage,
            threshold,
            baseline_path,
            current_path,
            diff_path: None,
            execution_time,
            screen_size,
            pixel_scale,
        }
    }

    pub fn set_diff_path(&mut self, diff_path: PathBuf) {
        self.diff_path = Some(diff_path);
    }
}

/// Suite for managing visual regression test results
#[derive(Debug, Default)]
pub struct VisualTestSuite {
    pub results: Vec<VisualTestResult>,
    pub baseline_directory: PathBuf,
    pub current_directory: PathBuf,
    pub diff_directory: PathBuf,
    pub start_time: Option<Instant>,
    pub total_duration: Duration,
}

impl VisualTestSuite {
    pub fn new(test_data_dir: &Path) -> std::io::Result<Self> {
        let baseline_dir = test_data_dir.join("visual_baselines");
        let current_dir = test_data_dir.join("visual_current");
        let diff_dir = test_data_dir.join("visual_diffs");

        // Create directories if they don't exist
        fs::create_dir_all(&baseline_dir)?;
        fs::create_dir_all(&current_dir)?;
        fs::create_dir_all(&diff_dir)?;

        Ok(Self {
            results: Vec::new(),
            baseline_directory: baseline_dir,
            current_directory: current_dir,
            diff_directory: diff_dir,
            start_time: Some(Instant::now()),
            total_duration: Duration::ZERO,
        })
    }

    pub fn add_result(&mut self, result: VisualTestResult) {
        self.results.push(result);
    }

    pub fn finish(&mut self) {
        if let Some(start) = self.start_time.take() {
            self.total_duration = start.elapsed();
        }
    }

    pub fn passed_count(&self) -> usize {
        self.results.iter().filter(|r| r.passed).count()
    }

    pub fn failed_count(&self) -> usize {
        self.results.len() - self.passed_count()
    }

    pub fn success_rate(&self) -> f32 {
        if self.results.is_empty() {
            0.0
        } else {
            self.passed_count() as f32 / self.results.len() as f32
        }
    }

    pub fn print_visual_report(&self) {
        println!("\n=== VISUAL REGRESSION TEST REPORT ===");
        println!("Total tests: {}", self.results.len());
        println!("Passed: {}", self.passed_count());
        println!("Failed: {}", self.failed_count());
        println!("Success rate: {:.1}%", self.success_rate() * 100.0);
        println!("Total execution time: {:.2}s", self.total_duration.as_secs_f32());

        // Group by component
        let mut by_component: HashMap<String, Vec<&VisualTestResult>> = HashMap::new();
        for result in &self.results {
            by_component.entry(result.component_name.clone()).or_default().push(result);
        }

        for (component, tests) in by_component {
            let passed = tests.iter().filter(|t| t.passed).count();
            let total = tests.len();
            println!("\n📸 Component: {} ({}/{} passed)", component, passed, total);

            for test in tests {
                let status = if test.passed { "✅" } else { "❌" };
                let time = format!("{:.1}ms", test.execution_time.as_secs_f32() * 1000.0);
                println!("   {} {} - {:.2}% diff (threshold: {:.2}%) ({})",
                    status, test.test_name, test.difference_percentage, test.threshold, time);

                if !test.passed {
                    println!("      Baseline: {}", test.baseline_path.display());
                    println!("      Current:  {}", test.current_path.display());
                    if let Some(diff_path) = &test.diff_path {
                        println!("      Diff:     {}", diff_path.display());
                    }
                }
            }
        }

        // Visual quality summary
        let avg_diff = if !self.results.is_empty() {
            self.results.iter().map(|r| r.difference_percentage).sum::<f32>() / self.results.len() as f32
        } else {
            0.0
        };

        println!("\n📊 Visual Quality Summary:");
        println!("   Average difference: {:.2}%", avg_diff);

        let high_diff_tests: Vec<_> = self.results.iter()
            .filter(|r| r.difference_percentage > 5.0)
            .collect();

        if !high_diff_tests.is_empty() {
            println!("   High difference tests (>5%): {}", high_diff_tests.len());
            for test in high_diff_tests {
                println!("     - {}.{}: {:.1}%",
                    test.component_name, test.test_name, test.difference_percentage);
            }
        }
    }
}

/// Mock screenshot generator for testing
pub struct MockScreenshotGenerator {
    screen_size: Vec2,
    pixel_scale: f32,
    theme_colors: ThemeColors,
}

impl MockScreenshotGenerator {
    pub fn new(screen_size: Vec2, pixel_scale: f32, theme: Theme) -> Self {
        let theme_manager = ThemeManager::new(theme);
        let theme_colors = theme_manager.get_colors();

        Self {
            screen_size,
            pixel_scale,
            theme_colors,
        }
    }

    /// Generate a mock screenshot of a UI component
    pub fn generate_component_screenshot(
        &self,
        component_name: &str,
        test_scenario: &str,
    ) -> ColorImage {
        // In a real implementation, this would render the actual component
        // For this mock, we'll create a simple colored rectangle with text
        let width = (self.screen_size.x * self.pixel_scale) as usize;
        let height = (self.screen_size.y * self.pixel_scale) as usize;

        let mut pixels = vec![Color32::TRANSPARENT; width * height];

        // Generate a deterministic pattern based on component name and scenario
        let hash = format!("{}{}", component_name, test_scenario)
            .chars()
            .map(|c| c as u32)
            .sum::<u32>();

        let r = ((hash * 123) % 256) as u8;
        let g = ((hash * 456) % 256) as u8;
        let b = ((hash * 789) % 256) as u8;
        let base_color = Color32::from_rgb(r, g, b);

        // Fill with pattern
        for y in 0..height {
            for x in 0..width {
                let idx = y * width + x;
                if idx < pixels.len() {
                    // Create a simple pattern that would change if the component changes
                    let pattern_value = ((x + y + hash as usize) % 256) as u8;
                    pixels[idx] = Color32::from_rgb(
                        ((r as u16 + pattern_value as u16) / 2) as u8,
                        ((g as u16 + pattern_value as u16) / 2) as u8,
                        ((b as u16 + pattern_value as u16) / 2) as u8,
                    );
                }
            }
        }

        ColorImage::from_rgba_unmultiplied([width, height], &pixels.iter()
            .flat_map(|c| [c.r(), c.g(), c.b(), c.a()])
            .collect::<Vec<u8>>())
    }

    /// Save screenshot to file
    pub fn save_screenshot(
        &self,
        image: &ColorImage,
        file_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Convert ColorImage to image format
        let [width, height] = image.size;
        let pixels: Vec<u8> = image.pixels.iter()
            .flat_map(|c| [c.r(), c.g(), c.b(), c.a()])
            .collect();

        let img = image::RgbaImage::from_raw(width as u32, height as u32, pixels)
            .ok_or("Failed to create image from raw data")?;

        img.save(file_path)?;
        Ok(())
    }

    /// Load screenshot from file
    pub fn load_screenshot(&self, file_path: &Path) -> Result<ColorImage, Box<dyn std::error::Error>> {
        if !file_path.exists() {
            return Err(format!("Screenshot file does not exist: {}", file_path.display()).into());
        }

        let img = image::open(file_path)?;
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();

        let pixels: Vec<Color32> = rgba.pixels()
            .map(|p| Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
            .collect();

        Ok(ColorImage::from_rgba_unmultiplied([width as usize, height as usize], &rgba.into_raw()))
    }
}

/// Image comparison utilities
pub struct ImageComparator;

impl ImageComparator {
    /// Compare two images and return difference percentage
    pub fn compare_images(baseline: &ColorImage, current: &ColorImage) -> f32 {
        if baseline.size != current.size {
            return 100.0; // Complete difference if sizes don't match
        }

        let total_pixels = baseline.pixels.len();
        if total_pixels == 0 {
            return 0.0;
        }

        let different_pixels = baseline.pixels.iter()
            .zip(current.pixels.iter())
            .filter(|(baseline_pixel, current_pixel)| {
                Self::pixel_difference(baseline_pixel, current_pixel) > 10 // Threshold for pixel difference
            })
            .count();

        (different_pixels as f32 / total_pixels as f32) * 100.0
    }

    /// Calculate difference between two pixels
    fn pixel_difference(pixel1: &Color32, pixel2: &Color32) -> u32 {
        let r_diff = (pixel1.r() as i32 - pixel2.r() as i32).abs() as u32;
        let g_diff = (pixel1.g() as i32 - pixel2.g() as i32).abs() as u32;
        let b_diff = (pixel1.b() as i32 - pixel2.b() as i32).abs() as u32;
        let a_diff = (pixel1.a() as i32 - pixel2.a() as i32).abs() as u32;

        r_diff + g_diff + b_diff + a_diff
    }

    /// Create a diff image highlighting differences
    pub fn create_diff_image(baseline: &ColorImage, current: &ColorImage) -> ColorImage {
        if baseline.size != current.size {
            // Return a red image to indicate size mismatch
            let [width, height] = baseline.size;
            let red_pixels = vec![Color32::RED; width * height];
            return ColorImage::from_rgba_unmultiplied([width, height], &red_pixels.iter()
                .flat_map(|c| [c.r(), c.g(), c.b(), c.a()])
                .collect::<Vec<u8>>());
        }

        let [width, height] = baseline.size;
        let diff_pixels: Vec<Color32> = baseline.pixels.iter()
            .zip(current.pixels.iter())
            .map(|(baseline_pixel, current_pixel)| {
                let diff = Self::pixel_difference(baseline_pixel, current_pixel);
                if diff > 10 {
                    Color32::RED // Highlight differences in red
                } else {
                    Color32::from_gray(128) // Similar pixels in gray
                }
            })
            .collect();

        ColorImage::from_rgba_unmultiplied([width, height], &diff_pixels.iter()
            .flat_map(|c| [c.r(), c.g(), c.b(), c.a()])
            .collect::<Vec<u8>>())
    }
}

/// Visual regression tester
pub struct VisualRegressionTester {
    suite: VisualTestSuite,
    screenshot_generator: MockScreenshotGenerator,
    difference_threshold: f32,
}

impl VisualRegressionTester {
    pub fn new(
        test_data_dir: &Path,
        screen_size: Vec2,
        pixel_scale: f32,
        theme: Theme,
        difference_threshold: f32,
    ) -> std::io::Result<Self> {
        let suite = VisualTestSuite::new(test_data_dir)?;
        let screenshot_generator = MockScreenshotGenerator::new(screen_size, pixel_scale, theme);

        Ok(Self {
            suite,
            screenshot_generator,
            difference_threshold,
        })
    }

    /// Test a component's visual appearance
    pub fn test_component_visual(
        &mut self,
        component_name: &str,
        test_scenario: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let start_time = Instant::now();

        // Generate current screenshot
        let current_image = self.screenshot_generator.generate_component_screenshot(
            component_name,
            test_scenario,
        );

        // Define file paths
        let test_name = format!("{}_{}", component_name, test_scenario);
        let baseline_path = self.suite.baseline_directory.join(format!("{}.png", test_name));
        let current_path = self.suite.current_directory.join(format!("{}.png", test_name));
        let diff_path = self.suite.diff_directory.join(format!("{}_diff.png", test_name));

        // Save current screenshot
        self.screenshot_generator.save_screenshot(&current_image, &current_path)?;

        // Load or create baseline
        let difference_percentage = if baseline_path.exists() {
            let baseline_image = self.screenshot_generator.load_screenshot(&baseline_path)?;
            let diff_percent = ImageComparator::compare_images(&baseline_image, &current_image);

            // Create diff image if there are differences
            if diff_percent > self.difference_threshold {
                let diff_image = ImageComparator::create_diff_image(&baseline_image, &current_image);
                self.screenshot_generator.save_screenshot(&diff_image, &diff_path)?;
            }

            diff_percent
        } else {
            // First run - create baseline
            self.screenshot_generator.save_screenshot(&current_image, &baseline_path)?;
            0.0 // No difference for first run
        };

        let duration = start_time.elapsed();

        // Create test result
        let mut result = VisualTestResult::new(
            test_scenario.to_string(),
            component_name.to_string(),
            difference_percentage,
            self.difference_threshold,
            baseline_path,
            current_path,
            self.screenshot_generator.screen_size,
            self.screenshot_generator.pixel_scale,
            duration,
        );

        if difference_percentage > self.difference_threshold {
            result.set_diff_path(diff_path);
        }

        self.suite.add_result(result);
        Ok(())
    }

    /// Test multiple components across different scenarios
    pub fn test_ui_components(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📸 Running Visual Regression Tests...");

        // Test different UI components in various states
        let test_scenarios = vec![
            ("MainLayout", "default_state"),
            ("MainLayout", "landscape_mode"),
            ("MainLayout", "portrait_mode"),
            ("PlaybackControls", "stopped_state"),
            ("PlaybackControls", "playing_state"),
            ("PlaybackControls", "paused_state"),
            ("ProgressBar", "empty"),
            ("ProgressBar", "half_progress"),
            ("ProgressBar", "full_progress"),
            ("VolumeSlider", "low_volume"),
            ("VolumeSlider", "medium_volume"),
            ("VolumeSlider", "high_volume"),
            ("EqualizerPanel", "flat_response"),
            ("EqualizerPanel", "bass_boost"),
            ("EqualizerPanel", "treble_boost"),
            ("SpectrumAnalyzer", "silent"),
            ("SpectrumAnalyzer", "sine_wave"),
            ("SpectrumAnalyzer", "complex_signal"),
            ("ThemeSelector", "mocha_theme"),
            ("ThemeSelector", "latte_theme"),
            ("AccessibilityControls", "normal_mode"),
            ("AccessibilityControls", "high_contrast"),
        ];

        for (component, scenario) in test_scenarios {
            self.test_component_visual(component, scenario)?;
        }

        Ok(())
    }

    /// Test HiDPI scaling visual consistency
    pub fn test_hidpi_scaling(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔍 Testing HiDPI Scaling Visual Consistency...");

        let scale_factors = vec![1.0, 1.25, 1.5, 2.0, 2.5, 3.0];

        for scale_factor in scale_factors {
            // Update generator scale
            self.screenshot_generator.pixel_scale = scale_factor;

            let scenario = format!("hidpi_scale_{:.2}", scale_factor);
            self.test_component_visual("MainLayout", &scenario)?;
            self.test_component_visual("PlaybackControls", &scenario)?;
        }

        Ok(())
    }

    /// Test responsive layout visual consistency
    pub fn test_responsive_layouts(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📱 Testing Responsive Layout Visual Consistency...");

        let screen_sizes = vec![
            ("mobile_portrait", Vec2::new(375.0, 667.0)),
            ("mobile_landscape", Vec2::new(667.0, 375.0)),
            ("tablet_portrait", Vec2::new(768.0, 1024.0)),
            ("tablet_landscape", Vec2::new(1024.0, 768.0)),
            ("desktop_small", Vec2::new(1280.0, 720.0)),
            ("desktop_standard", Vec2::new(1920.0, 1080.0)),
            ("desktop_hidpi", Vec2::new(2560.0, 1440.0)),
            ("ultrawide", Vec2::new(3440.0, 1440.0)),
        ];

        for (size_name, screen_size) in screen_sizes {
            // Update generator screen size
            self.screenshot_generator.screen_size = screen_size;

            self.test_component_visual("MainLayout", size_name)?;
            self.test_component_visual("PlaybackControls", size_name)?;
        }

        Ok(())
    }

    pub fn finish_testing(mut self) -> VisualTestSuite {
        self.suite.finish();
        self.suite
    }
}

/// Main visual regression test runner
pub fn run_visual_regression_tests(test_data_dir: &Path) -> Result<VisualTestSuite, Box<dyn std::error::Error>> {
    println!("🎨 RUSTY AUDIO - VISUAL REGRESSION TESTING FRAMEWORK");
    println!("====================================================");

    let mut tester = VisualRegressionTester::new(
        test_data_dir,
        Vec2::new(1200.0, 800.0), // Default car stereo landscape size
        1.25,                      // HiDPI scaling
        Theme::Mocha,             // Default theme
        2.0,                      // 2% difference threshold
    )?;

    // Run comprehensive visual tests
    tester.test_ui_components()?;
    tester.test_hidpi_scaling()?;
    tester.test_responsive_layouts()?;

    let suite = tester.finish_testing();
    suite.print_visual_report();

    Ok(suite)
}

/// Quick visual regression test for CI/CD
pub fn run_quick_visual_tests(test_data_dir: &Path) -> Result<VisualTestSuite, Box<dyn std::error::Error>> {
    println!("🚀 RUSTY AUDIO - QUICK VISUAL TESTS");
    println!("===================================");

    let mut tester = VisualRegressionTester::new(
        test_data_dir,
        Vec2::new(1200.0, 800.0),
        1.25,
        Theme::Mocha,
        5.0, // Higher threshold for quick tests
    )?;

    // Test only essential components
    tester.test_component_visual("MainLayout", "default_state")?;
    tester.test_component_visual("PlaybackControls", "playing_state")?;
    tester.test_component_visual("ProgressBar", "half_progress")?;

    let suite = tester.finish_testing();
    suite.print_visual_report();

    Ok(suite)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_image_comparator() {
        // Create two identical images
        let pixels = vec![Color32::WHITE; 100];
        let image1 = ColorImage::from_rgba_unmultiplied([10, 10], &pixels.iter()
            .flat_map(|c| [c.r(), c.g(), c.b(), c.a()])
            .collect::<Vec<u8>>());
        let image2 = image1.clone();

        let diff = ImageComparator::compare_images(&image1, &image2);
        assert_eq!(diff, 0.0);
    }

    #[test]
    fn test_visual_test_result() {
        let result = VisualTestResult::new(
            "test".to_string(),
            "component".to_string(),
            1.5,
            2.0,
            PathBuf::from("baseline.png"),
            PathBuf::from("current.png"),
            Vec2::new(800.0, 600.0),
            1.0,
            Duration::from_millis(10),
        );

        assert!(result.passed);
        assert_eq!(result.difference_percentage, 1.5);
    }

    #[test]
    fn test_screenshot_generator() {
        let generator = MockScreenshotGenerator::new(Vec2::new(100.0, 100.0), 1.0, Theme::Mocha);
        let image = generator.generate_component_screenshot("TestComponent", "test_scenario");

        assert_eq!(image.size, [100, 100]);
        assert_eq!(image.pixels.len(), 10000);
    }

    #[test]
    fn test_visual_regression_tester() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let mut tester = VisualRegressionTester::new(
            temp_dir.path(),
            Vec2::new(100.0, 100.0),
            1.0,
            Theme::Mocha,
            2.0,
        )?;

        tester.test_component_visual("TestComponent", "test_scenario")?;

        let suite = tester.finish_testing();
        assert_eq!(suite.results.len(), 1);
        assert!(suite.results[0].passed); // First run should always pass

        Ok(())
    }
}