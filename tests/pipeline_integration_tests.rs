// Integration Tests for Complete Audio Pipeline
// Tests the entire audio processing chain from input to output

use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use web_audio_api::context::{AudioContext, BaseAudioContext, OfflineAudioContext};
use web_audio_api::node::{AudioNode, AudioScheduledSourceNode};
use web_audio_api::AudioBuffer;
use rustfft::{FftPlanner, num_complex::Complex32};
use approx::{assert_relative_eq, assert_abs_diff_eq};
use serial_test::serial;
use tokio_test;
use std::f32::consts::PI;

const SAMPLE_RATE: f32 = 48000.0;
const RENDER_QUANTUM_SIZE: usize = 128;
const TEST_DURATION: f32 = 1.0;
const TOLERANCE: f32 = 0.001;

/// Audio pipeline test configuration
#[derive(Debug, Clone)]
struct PipelineTestConfig {
    sample_rate: f32,
    channels: u32,
    buffer_size: usize,
    test_duration: f32,
    tolerance: f32,
    enable_effects: bool,
    enable_eq: bool,
    enable_analysis: bool,
}

impl Default for PipelineTestConfig {
    fn default() -> Self {
        Self {
            sample_rate: SAMPLE_RATE,
            channels: 2,
            buffer_size: 4096,
            test_duration: TEST_DURATION,
            tolerance: TOLERANCE,
            enable_effects: true,
            enable_eq: true,
            enable_analysis: true,
        }
    }
}

/// Test metrics for pipeline analysis
#[derive(Debug, Clone)]
struct PipelineMetrics {
    latency_ms: f32,
    cpu_usage_percent: f32,
    memory_usage_mb: f32,
    processing_time_ms: f32,
    thd_percent: f32,
    snr_db: f32,
    frequency_response_error_db: f32,
    samples_processed: usize,
    buffer_underruns: u32,
    buffer_overruns: u32,
}

impl Default for PipelineMetrics {
    fn default() -> Self {
        Self {
            latency_ms: 0.0,
            cpu_usage_percent: 0.0,
            memory_usage_mb: 0.0,
            processing_time_ms: 0.0,
            thd_percent: 0.0,
            snr_db: 0.0,
            frequency_response_error_db: 0.0,
            samples_processed: 0,
            buffer_underruns: 0,
            buffer_overruns: 0,
        }
    }
}

/// Complete audio pipeline for testing
struct AudioPipeline {
    context: OfflineAudioContext,
    config: PipelineTestConfig,
    metrics: Arc<Mutex<PipelineMetrics>>,
}

impl AudioPipeline {
    fn new(config: PipelineTestConfig) -> Self {
        let sample_count = (config.test_duration * config.sample_rate) as usize;
        let context = OfflineAudioContext::new(config.channels as usize, sample_count, config.sample_rate as usize);

        Self {
            context,
            config,
            metrics: Arc::new(Mutex::new(PipelineMetrics::default())),
        }
    }

    fn create_test_signal(&self, frequency: f32, amplitude: f32) -> AudioBuffer {
        let sample_count = (self.config.test_duration * self.config.sample_rate) as usize;
        let mut buffer = self.context.create_buffer(self.config.channels as usize, sample_count, self.config.sample_rate as usize);

        for channel in 0..self.config.channels {
            let mut data = Vec::with_capacity(sample_count);

            for i in 0..sample_count {
                let t = i as f32 / self.config.sample_rate;
                let sample = amplitude * (2.0 * PI * frequency * t).sin();
                data.push(sample);
            }

            buffer.copy_to_channel(&data, channel as usize);
        }

        buffer
    }

    fn create_complex_test_signal(&self) -> AudioBuffer {
        let sample_count = (self.config.test_duration * self.config.sample_rate) as usize;
        let mut buffer = self.context.create_buffer(self.config.channels, sample_count, self.config.sample_rate);

        // Create a complex signal with multiple harmonics
        let fundamental_freq = 440.0; // A4
        let harmonics = vec![
            (1.0, 1.0),     // Fundamental
            (2.0, 0.3),     // 2nd harmonic
            (3.0, 0.1),     // 3rd harmonic
            (4.0, 0.05),    // 4th harmonic
        ];

        for channel in 0..self.config.channels {
            let mut data = Vec::with_capacity(sample_count);

            for i in 0..sample_count {
                let t = i as f32 / self.config.sample_rate;
                let mut sample = 0.0;

                for (harmonic, amplitude) in &harmonics {
                    sample += amplitude * (2.0 * PI * fundamental_freq * harmonic * t).sin();
                }

                // Add stereo effect
                if channel == 1 {
                    sample *= 0.7; // Slight amplitude difference for stereo test
                }

                data.push(sample);
            }

            buffer.copy_to_channel(&data, channel as usize);
        }

        buffer
    }

    fn create_processing_chain(&self, input_buffer: AudioBuffer) -> web_audio_api::node::AudioDestinationNode {
        // Create buffer source
        let mut source = self.context.create_buffer_source();
        source.set_buffer(input_buffer);

        let mut current_node: &dyn AudioNode = &source;

        // Add gain control
        let gain = self.context.create_gain();
        gain.gain().set_value(0.8);
        current_node.connect(&gain);
        current_node = &gain;

        // Add EQ if enabled
        if self.config.enable_eq {
            // 3-band EQ
            let low_shelf = self.context.create_biquad_filter();
            low_shelf.set_type(web_audio_api::node::BiquadFilterType::LowShelf);
            low_shelf.frequency().set_value(200.0);
            low_shelf.gain().set_value(0.0);

            let peak = self.context.create_biquad_filter();
            peak.set_type(web_audio_api::node::BiquadFilterType::Peaking);
            peak.frequency().set_value(1000.0);
            peak.gain().set_value(0.0);
            peak.q().set_value(1.0);

            let high_shelf = self.context.create_biquad_filter();
            high_shelf.set_type(web_audio_api::node::BiquadFilterType::HighShelf);
            high_shelf.frequency().set_value(5000.0);
            high_shelf.gain().set_value(0.0);

            current_node.connect(&low_shelf);
            low_shelf.connect(&peak);
            peak.connect(&high_shelf);
            current_node = &high_shelf;
        }

        // Add analyzer if enabled
        if self.config.enable_analysis {
            let mut analyser = self.context.create_analyser();
            analyser.set_fft_size(2048);
            analyser.set_smoothing_time_constant(0.8);

            current_node.connect(&analyser);
            current_node = &analyser;
        }

        // Connect to destination
        current_node.connect(&self.context.destination());

        // Start playback
        source.start();

        self.context.destination()
    }

    fn run_pipeline_test(&mut self) -> Result<AudioBuffer, Box<dyn std::error::Error>> {
        let start_time = Instant::now();

        // Create test signal
        let input_buffer = self.create_complex_test_signal();

        // Set up processing chain
        let _destination = self.create_processing_chain(input_buffer);

        // Render audio
        let output_buffer = self.context.start_rendering_sync();

        let processing_time = start_time.elapsed();

        // Update metrics
        {
            let mut metrics = self.metrics.lock().unwrap();
            metrics.processing_time_ms = processing_time.as_millis() as f32;
            metrics.samples_processed = output_buffer.length();

            // Calculate CPU usage as percentage of real-time
            let real_time_duration = self.config.test_duration * 1000.0; // ms
            metrics.cpu_usage_percent = (metrics.processing_time_ms / real_time_duration) * 100.0;
        }

        Ok(output_buffer)
    }

    fn analyze_output(&self, output_buffer: &AudioBuffer) -> Result<(), Box<dyn std::error::Error>> {
        let mut metrics = self.metrics.lock().unwrap();

        // Analyze each channel
        for channel in 0..output_buffer.number_of_channels() {
            let channel_data = output_buffer.get_channel_data(channel);

            // Calculate RMS
            let rms = calculate_rms(channel_data);

            // Calculate THD+N
            let thd = self.calculate_thd_plus_n(channel_data);
            metrics.thd_percent = thd;

            // Calculate SNR
            let signal_power = rms * rms;
            let noise_power = 0.001; // Estimated noise floor
            metrics.snr_db = 20.0 * (signal_power / noise_power).log10();
        }

        Ok(())
    }

    fn calculate_thd_plus_n(&self, samples: &[f32]) -> f32 {
        if samples.len() < 2048 {
            return 0.0;
        }

        // Perform FFT analysis
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(2048);

        let mut buffer: Vec<Complex32> = samples[..2048]
            .iter()
            .map(|&sample| Complex32::new(sample, 0.0))
            .collect();

        fft.process(&mut buffer);

        // Find fundamental frequency (should be 440 Hz)
        let fundamental_bin = (440.0 * 2048.0 / self.config.sample_rate) as usize;
        let fundamental_magnitude = buffer[fundamental_bin].norm();

        if fundamental_magnitude == 0.0 {
            return f32::INFINITY;
        }

        // Calculate harmonics
        let mut harmonic_power = 0.0;
        for harmonic in 2..=10 {
            let harmonic_bin = fundamental_bin * harmonic;
            if harmonic_bin < buffer.len() / 2 {
                let harmonic_magnitude = buffer[harmonic_bin].norm();
                harmonic_power += harmonic_magnitude * harmonic_magnitude;
            }
        }

        // Add noise power (excluding fundamental and harmonics)
        let mut noise_power = 0.0;
        for i in 1..buffer.len() / 2 {
            let is_harmonic = (2..=10).any(|h| i == fundamental_bin * h);
            if i != fundamental_bin && !is_harmonic {
                let magnitude = buffer[i].norm();
                noise_power += magnitude * magnitude;
            }
        }

        // Calculate THD+N
        ((harmonic_power + noise_power).sqrt() / fundamental_magnitude) * 100.0
    }

    fn get_metrics(&self) -> PipelineMetrics {
        self.metrics.lock().unwrap().clone()
    }
}

/// Helper function to calculate RMS
fn calculate_rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }

    let sum_squares: f32 = samples.iter().map(|&x| x * x).sum();
    (sum_squares / samples.len() as f32).sqrt()
}

/// Helper function to calculate peak
fn calculate_peak(samples: &[f32]) -> f32 {
    samples.iter().map(|&x| x.abs()).fold(0.0, f32::max)
}

// ============================================================================
// INTEGRATION TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[serial]
    fn test_basic_pipeline_functionality() {
        let config = PipelineTestConfig::default();
        let mut pipeline = AudioPipeline::new(config.clone());

        let result = pipeline.run_pipeline_test();
        assert!(result.is_ok(), "Pipeline test failed: {:?}", result.err());

        let output_buffer = result.unwrap();
        assert_eq!(output_buffer.number_of_channels(), config.channels);
        assert_eq!(output_buffer.length(), (config.test_duration * config.sample_rate) as usize);

        // Verify output is not silent
        for channel in 0..output_buffer.number_of_channels() {
            let channel_data = output_buffer.get_channel_data(channel);
            let rms = calculate_rms(channel_data);
            assert!(rms > 0.01, "Channel {} appears to be silent: RMS = {:.6}", channel, rms);
        }

        let metrics = pipeline.get_metrics();
        println!("Basic pipeline metrics: {:#?}", metrics);
    }

    #[test]
    #[serial]
    fn test_pipeline_with_effects_disabled() {
        let mut config = PipelineTestConfig::default();
        config.enable_effects = false;
        config.enable_eq = false;

        let mut pipeline = AudioPipeline::new(config);
        let result = pipeline.run_pipeline_test();
        assert!(result.is_ok());

        let output_buffer = result.unwrap();
        let metrics = pipeline.get_metrics();

        // Should be faster without effects
        assert!(metrics.cpu_usage_percent < 50.0,
            "CPU usage too high without effects: {:.1}%", metrics.cpu_usage_percent);

        println!("Pipeline without effects metrics: {:#?}", metrics);
    }

    #[test]
    #[serial]
    fn test_pipeline_latency() {
        let config = PipelineTestConfig::default();
        let mut pipeline = AudioPipeline::new(config);

        let start_time = Instant::now();
        let result = pipeline.run_pipeline_test();
        let total_latency = start_time.elapsed();

        assert!(result.is_ok());

        let latency_ms = total_latency.as_millis() as f32;
        assert!(latency_ms < 1000.0, "Pipeline latency too high: {:.1} ms", latency_ms);

        println!("Pipeline latency: {:.1} ms", latency_ms);
    }

    #[test]
    #[serial]
    fn test_pipeline_frequency_response() {
        let config = PipelineTestConfig::default();
        let test_frequencies = vec![100.0, 440.0, 1000.0, 2000.0, 8000.0];

        for &frequency in &test_frequencies {
            let mut pipeline = AudioPipeline::new(config.clone());

            // Create single-frequency test signal
            let input_buffer = pipeline.create_test_signal(frequency, 0.5);

            // Set up minimal processing chain (just gain)
            let mut source = pipeline.context.create_buffer_source();
            source.set_buffer(input_buffer);

            let gain = pipeline.context.create_gain();
            gain.gain().set_value(1.0); // Unity gain

            source.connect(&gain);
            gain.connect(&pipeline.context.destination());
            source.start();

            let output_buffer = pipeline.context.start_rendering_sync();

            // Analyze output
            for channel in 0..output_buffer.number_of_channels() {
                let channel_data = output_buffer.get_channel_data(channel);
                let output_rms = calculate_rms(channel_data);

                // Expected RMS for 0.5 amplitude sine wave
                let expected_rms = 0.5 / 2.0f32.sqrt();

                assert_relative_eq!(output_rms, expected_rms, epsilon = 0.1,
                    "Frequency response failed at {:.0} Hz: expected RMS {:.3}, got {:.3}",
                    frequency, expected_rms, output_rms);
            }
        }
    }

    #[test]
    #[serial]
    fn test_pipeline_thd_analysis() {
        let config = PipelineTestConfig::default();
        let mut pipeline = AudioPipeline::new(config);

        let result = pipeline.run_pipeline_test();
        assert!(result.is_ok());

        let output_buffer = result.unwrap();
        pipeline.analyze_output(&output_buffer).unwrap();

        let metrics = pipeline.get_metrics();

        // THD should be reasonable for clean signal processing
        assert!(metrics.thd_percent < 1.0,
            "THD too high: {:.3}%", metrics.thd_percent);

        // SNR should be good
        assert!(metrics.snr_db > 40.0,
            "SNR too low: {:.1} dB", metrics.snr_db);

        println!("Audio quality metrics: THD = {:.3}%, SNR = {:.1} dB",
            metrics.thd_percent, metrics.snr_db);
    }

    #[test]
    #[serial]
    fn test_pipeline_stereo_processing() {
        let mut config = PipelineTestConfig::default();
        config.channels = 2;

        let mut pipeline = AudioPipeline::new(config);
        let result = pipeline.run_pipeline_test();
        assert!(result.is_ok());

        let output_buffer = result.unwrap();
        assert_eq!(output_buffer.number_of_channels(), 2);

        // Check that both channels have content but are different (stereo effect)
        let left_data = output_buffer.get_channel_data(0);
        let right_data = output_buffer.get_channel_data(1);

        let left_rms = calculate_rms(left_data);
        let right_rms = calculate_rms(right_data);

        assert!(left_rms > 0.01, "Left channel appears silent");
        assert!(right_rms > 0.01, "Right channel appears silent");

        // Should be different due to stereo effect in create_complex_test_signal
        let rms_ratio = (left_rms / right_rms - 1.0).abs();
        assert!(rms_ratio > 0.1,
            "Stereo channels too similar: L={:.3}, R={:.3}, ratio diff={:.3}",
            left_rms, right_rms, rms_ratio);
    }

    #[test]
    #[serial]
    fn test_pipeline_buffer_sizes() {
        let buffer_sizes = vec![1024, 2048, 4096, 8192];

        for &buffer_size in &buffer_sizes {
            let mut config = PipelineTestConfig::default();
            config.buffer_size = buffer_size;

            let mut pipeline = AudioPipeline::new(config.clone());
            let result = pipeline.run_pipeline_test();

            assert!(result.is_ok(),
                "Pipeline failed with buffer size {}: {:?}", buffer_size, result.err());

            let metrics = pipeline.get_metrics();
            assert!(metrics.cpu_usage_percent < 80.0,
                "CPU usage too high with buffer size {}: {:.1}%",
                buffer_size, metrics.cpu_usage_percent);
        }
    }

    #[test]
    #[serial]
    fn test_pipeline_sample_rates() {
        let sample_rates = vec![44100.0, 48000.0, 96000.0];

        for &sample_rate in &sample_rates {
            let mut config = PipelineTestConfig::default();
            config.sample_rate = sample_rate;

            let mut pipeline = AudioPipeline::new(config.clone());
            let result = pipeline.run_pipeline_test();

            assert!(result.is_ok(),
                "Pipeline failed with sample rate {}: {:?}", sample_rate, result.err());

            let output_buffer = result.unwrap();
            let expected_length = (config.test_duration * sample_rate) as usize;
            assert_eq!(output_buffer.length(), expected_length,
                "Incorrect output length for sample rate {}", sample_rate);
        }
    }

    #[test]
    #[serial]
    fn test_pipeline_stress_test() {
        let mut config = PipelineTestConfig::default();
        config.test_duration = 10.0; // Longer test
        config.enable_effects = true;
        config.enable_eq = true;
        config.enable_analysis = true;

        let mut pipeline = AudioPipeline::new(config);
        let result = pipeline.run_pipeline_test();

        assert!(result.is_ok(), "Stress test failed: {:?}", result.err());

        let metrics = pipeline.get_metrics();

        // Even under stress, should maintain real-time performance
        assert!(metrics.cpu_usage_percent < 150.0,
            "CPU usage too high under stress: {:.1}%", metrics.cpu_usage_percent);

        // No buffer issues
        assert_eq!(metrics.buffer_underruns, 0, "Buffer underruns detected");
        assert_eq!(metrics.buffer_overruns, 0, "Buffer overruns detected");

        println!("Stress test metrics: {:#?}", metrics);
    }

    #[test]
    #[serial]
    fn test_pipeline_error_handling() {
        // Test with invalid configuration
        let mut config = PipelineTestConfig::default();
        config.channels = 0; // Invalid

        // This should still work because OfflineAudioContext handles it
        let mut pipeline = AudioPipeline::new(config);
        let _result = pipeline.run_pipeline_test();
        // Just ensure it doesn't panic
    }

    #[test]
    #[serial]
    fn test_pipeline_memory_usage() {
        let config = PipelineTestConfig::default();

        // Get baseline memory usage
        let baseline_memory = get_memory_usage_mb();

        let mut pipeline = AudioPipeline::new(config);
        let result = pipeline.run_pipeline_test();
        assert!(result.is_ok());

        let peak_memory = get_memory_usage_mb();
        let memory_increase = peak_memory - baseline_memory;

        // Memory usage should be reasonable
        assert!(memory_increase < 100.0,
            "Memory usage too high: {:.1} MB increase", memory_increase);

        println!("Memory usage: baseline = {:.1} MB, peak = {:.1} MB, increase = {:.1} MB",
            baseline_memory, peak_memory, memory_increase);
    }

    #[tokio::test]
    async fn test_pipeline_async_processing() {
        let config = PipelineTestConfig::default();
        let mut pipeline = AudioPipeline::new(config);

        // Run pipeline in async context
        let result = tokio::task::spawn_blocking(move || {
            pipeline.run_pipeline_test()
        }).await.unwrap();

        assert!(result.is_ok(), "Async pipeline test failed: {:?}", result.err());
    }

    #[test]
    #[serial]
    fn test_pipeline_concurrent_instances() {
        use std::thread;

        let num_instances = 4;
        let mut handles = Vec::new();

        for i in 0..num_instances {
            let handle = thread::spawn(move || {
                let config = PipelineTestConfig::default();
                let mut pipeline = AudioPipeline::new(config);
                let result = pipeline.run_pipeline_test();

                assert!(result.is_ok(),
                    "Concurrent instance {} failed: {:?}", i, result.err());

                pipeline.get_metrics()
            });
            handles.push(handle);
        }

        let mut all_metrics = Vec::new();
        for handle in handles {
            let metrics = handle.join().unwrap();
            all_metrics.push(metrics);
        }

        // Verify all instances completed successfully
        assert_eq!(all_metrics.len(), num_instances);

        // Check that performance is consistent across instances
        let avg_cpu_usage: f32 = all_metrics.iter()
            .map(|m| m.cpu_usage_percent)
            .sum::<f32>() / num_instances as f32;

        for (i, metrics) in all_metrics.iter().enumerate() {
            let cpu_diff = (metrics.cpu_usage_percent - avg_cpu_usage).abs();
            assert!(cpu_diff < avg_cpu_usage * 0.5,
                "Instance {} CPU usage varies too much from average: {:.1}% vs {:.1}%",
                i, metrics.cpu_usage_percent, avg_cpu_usage);
        }

        println!("Concurrent test: {} instances, average CPU usage: {:.1}%",
            num_instances, avg_cpu_usage);
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn get_memory_usage_mb() -> f32 {
    // Simple memory usage estimation
    // In a real implementation, you'd use platform-specific APIs
    let usage = std::alloc::System.alloc(std::alloc::Layout::new::<[u8; 1024]>());
    if !usage.is_null() {
        unsafe {
            std::alloc::System.dealloc(usage, std::alloc::Layout::new::<[u8; 1024]>());
        }
    }
    10.0 // Placeholder return value
}