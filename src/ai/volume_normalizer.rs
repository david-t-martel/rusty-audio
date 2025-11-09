//! Smart Volume Normalization Module
//!
//! Implements intelligent volume normalization using LUFS/LKFS standards,
//! adaptive dynamics processing, and content-aware level adjustment.

use crate::ai::feature_extractor::AudioFeatures;
use anyhow::{Context, Result};
use std::collections::VecDeque;

/// Smart volume normalizer with AI-based content awareness
pub struct VolumeNormalizer {
    target_lufs: f32,
    gate_threshold: f32,
    lookahead_buffer: VecDeque<f32>,
    lookahead_size: usize,
    history_buffer: VecDeque<f32>,
    history_size: usize,
    content_analyzer: ContentAnalyzer,
    dynamics_processor: DynamicsProcessor,
    loudness_meter: LoudnessMeter,
}

impl VolumeNormalizer {
    pub fn new() -> Result<Self> {
        Ok(Self {
            target_lufs: -16.0, // Standard for streaming platforms
            gate_threshold: -40.0,
            lookahead_buffer: VecDeque::with_capacity(2048),
            lookahead_size: 2048,
            history_buffer: VecDeque::with_capacity(48000), // 1 second at 48kHz
            history_size: 48000,
            content_analyzer: ContentAnalyzer::new(),
            dynamics_processor: DynamicsProcessor::new()?,
            loudness_meter: LoudnessMeter::new()?,
        })
    }

    /// Normalize audio with intelligent content-aware processing
    pub fn normalize(&mut self, buffer: &[f32], features: &AudioFeatures) -> Result<Vec<f32>> {
        // Analyze content type
        let content_type = self.content_analyzer.analyze(features)?;

        // Adjust target based on content type
        let adjusted_target = self.adjust_target_for_content(&content_type);

        // Measure integrated loudness
        let current_lufs = self.loudness_meter.measure_integrated(buffer)?;

        // Calculate gain adjustment
        let gain = self.calculate_intelligent_gain(
            current_lufs,
            adjusted_target,
            &content_type,
            features,
        )?;

        // Apply multi-stage processing
        let mut processed = buffer.to_vec();

        // Stage 1: Apply lookahead limiting
        processed = self.apply_lookahead_limiting(&processed, gain)?;

        // Stage 2: Apply dynamics processing based on content
        processed = self.dynamics_processor.process(&processed, &content_type)?;

        // Stage 3: Apply final gain with smooth transitions
        processed = self.apply_smooth_gain(&processed, gain)?;

        // Stage 4: True peak limiting
        processed = self.apply_true_peak_limiting(&processed)?;

        // Update history for adaptive processing
        self.update_history(&processed);

        Ok(processed)
    }

    /// Adjust target LUFS based on content type
    fn adjust_target_for_content(&self, content_type: &ContentType) -> f32 {
        match content_type {
            ContentType::Speech => -18.0, // Clearer for speech
            ContentType::Music(genre) => match genre {
                MusicGenre::Classical => -20.0,  // More dynamic range
                MusicGenre::Electronic => -14.0, // Louder for club music
                MusicGenre::Rock => -15.0,       // Standard rock loudness
                MusicGenre::Jazz => -18.0,       // Preserve dynamics
                _ => -16.0,                      // Default
            },
            ContentType::Podcast => -16.0, // Standard podcast level
            ContentType::Mixed => -17.0,   // Balanced for mixed content
        }
    }

    /// Calculate intelligent gain with multiple factors
    fn calculate_intelligent_gain(
        &self,
        current_lufs: f32,
        target_lufs: f32,
        content_type: &ContentType,
        features: &AudioFeatures,
    ) -> Result<f32> {
        // Basic gain calculation
        let mut gain = target_lufs - current_lufs;

        // Limit gain to prevent over-amplification
        gain = gain.max(-24.0).min(24.0);

        // Adjust gain based on dynamic range
        if let Some(dynamic_range) = features.dynamic_range {
            if dynamic_range < 10.0 {
                // Already compressed, be conservative
                gain *= 0.7;
            } else if dynamic_range > 30.0 {
                // Very dynamic, allow more gain
                gain *= 1.2;
            }
        }

        // Adjust for crest factor
        if let Some(crest_factor) = features.crest_factor {
            if crest_factor > 10.0 {
                // High peaks, reduce gain to prevent clipping
                gain -= 3.0;
            }
        }

        // Content-specific adjustments
        match content_type {
            ContentType::Speech => {
                // Be more aggressive with speech normalization
                gain *= 1.1;
            }
            ContentType::Music(MusicGenre::Classical) => {
                // Preserve dynamics in classical music
                gain *= 0.8;
            }
            _ => {}
        }

        Ok(self.db_to_linear(gain))
    }

    /// Apply lookahead limiting
    fn apply_lookahead_limiting(&mut self, buffer: &[f32], gain: f32) -> Result<Vec<f32>> {
        let mut output = Vec::with_capacity(buffer.len());

        for &sample in buffer {
            // Add to lookahead buffer
            self.lookahead_buffer.push_back(sample);

            if self.lookahead_buffer.len() >= self.lookahead_size {
                if let Some(delayed_sample) = self.lookahead_buffer.pop_front() {
                    // Find the maximum in the lookahead window
                    let peak = self
                        .lookahead_buffer
                        .iter()
                        .map(|x| x.abs())
                        .fold(0.0f32, f32::max);

                    // Calculate limiting factor
                    let limit_factor = if peak * gain > 0.95 {
                        0.95 / (peak * gain)
                    } else {
                        1.0
                    };

                    // Apply gain with limiting
                    output.push(delayed_sample * gain * limit_factor);
                }
            }
        }

        // Flush remaining samples
        while let Some(sample) = self.lookahead_buffer.pop_front() {
            output.push(sample * gain * 0.95); // Conservative gain for tail
        }

        Ok(output)
    }

    /// Apply smooth gain transitions
    fn apply_smooth_gain(&self, buffer: &[f32], target_gain: f32) -> Result<Vec<f32>> {
        let mut output = vec![0.0; buffer.len()];
        let mut current_gain = 1.0;
        let smoothing_factor = 0.001; // Smooth over ~1000 samples

        for (i, &sample) in buffer.iter().enumerate() {
            // Exponential smoothing towards target gain
            current_gain += (target_gain - current_gain) * smoothing_factor;
            output[i] = sample * current_gain;
        }

        Ok(output)
    }

    /// Apply true peak limiting to prevent inter-sample peaks
    fn apply_true_peak_limiting(&self, buffer: &[f32]) -> Result<Vec<f32>> {
        let mut output = buffer.to_vec();
        let true_peak_limit = 0.99; // -0.1 dBTP

        // Simple oversampling for true peak detection
        let oversample_factor = 4;
        let mut oversampled = vec![0.0; buffer.len() * oversample_factor];

        // Upsample (simple linear interpolation)
        for i in 0..buffer.len() - 1 {
            for j in 0..oversample_factor {
                let t = j as f32 / oversample_factor as f32;
                oversampled[i * oversample_factor + j] = buffer[i] * (1.0 - t) + buffer[i + 1] * t;
            }
        }

        // Find true peaks
        let true_peak = oversampled.iter().map(|x| x.abs()).fold(0.0f32, f32::max);

        // Apply limiting if needed
        if true_peak > true_peak_limit {
            let reduction = true_peak_limit / true_peak;
            for sample in &mut output {
                *sample *= reduction;
            }
        }

        Ok(output)
    }

    /// Update history buffer for adaptive processing
    fn update_history(&mut self, buffer: &[f32]) {
        for &sample in buffer {
            self.history_buffer.push_back(sample);
            if self.history_buffer.len() > self.history_size {
                self.history_buffer.pop_front();
            }
        }
    }

    /// Convert dB to linear gain
    fn db_to_linear(&self, db: f32) -> f32 {
        10.0_f32.powf(db / 20.0)
    }

    /// Set target loudness in LUFS
    pub fn set_target_lufs(&mut self, target: f32) {
        self.target_lufs = target.max(-30.0).min(0.0);
    }

    /// Enable adaptive mode that learns from content
    pub fn enable_adaptive_mode(&mut self, enabled: bool) {
        self.content_analyzer.adaptive_mode = enabled;
    }
}

/// Content analyzer for intelligent processing decisions
struct ContentAnalyzer {
    adaptive_mode: bool,
    content_history: VecDeque<ContentType>,
}

impl ContentAnalyzer {
    fn new() -> Self {
        Self {
            adaptive_mode: true,
            content_history: VecDeque::with_capacity(10),
        }
    }

    fn analyze(&mut self, features: &AudioFeatures) -> Result<ContentType> {
        let content_type = self.detect_content_type(features)?;

        if self.adaptive_mode {
            self.content_history.push_back(content_type.clone());
            if self.content_history.len() > 10 {
                self.content_history.pop_front();
            }
        }

        Ok(content_type)
    }

    fn detect_content_type(&self, features: &AudioFeatures) -> Result<ContentType> {
        // Speech detection
        if self.is_speech(features) {
            return Ok(ContentType::Speech);
        }

        // Music genre detection
        if let Some(genre) = self.detect_music_genre(features) {
            return Ok(ContentType::Music(genre));
        }

        // Podcast detection (speech with music)
        if self.is_podcast(features) {
            return Ok(ContentType::Podcast);
        }

        Ok(ContentType::Mixed)
    }

    fn is_speech(&self, features: &AudioFeatures) -> bool {
        let mut speech_indicators = 0;
        let mut total_checks = 0;

        // Check for speech-like spectral centroid
        if let Some(centroid) = features.spectral_centroid {
            total_checks += 1;
            if centroid > 500.0 && centroid < 3000.0 {
                speech_indicators += 1;
            }
        }

        // Check for high zero crossing rate variation
        if let Some(zcr) = features.zero_crossing_rate {
            total_checks += 1;
            if zcr > 0.1 && zcr < 0.4 {
                speech_indicators += 1;
            }
        }

        // Check for speech-like pitch range
        if let Some(f0) = features.fundamental_frequency {
            total_checks += 1;
            if f0 > 80.0 && f0 < 400.0 {
                speech_indicators += 1;
            }
        }

        speech_indicators as f32 / total_checks.max(1) as f32 > 0.6
    }

    fn detect_music_genre(&self, features: &AudioFeatures) -> Option<MusicGenre> {
        // Simple genre detection based on features
        if let Some(tempo) = features.tempo {
            if tempo > 120.0 && tempo < 140.0 {
                if features.bass_energy.unwrap_or(0.0) > 0.4 {
                    return Some(MusicGenre::Electronic);
                }
            } else if tempo < 80.0 {
                if features.dynamic_range.unwrap_or(0.0) > 25.0 {
                    return Some(MusicGenre::Classical);
                }
            } else if tempo > 100.0 && tempo < 120.0 {
                if features.spectral_centroid.unwrap_or(0.0) > 2000.0 {
                    return Some(MusicGenre::Rock);
                }
            }
        }

        Some(MusicGenre::Pop) // Default
    }

    fn is_podcast(&self, features: &AudioFeatures) -> bool {
        // Podcast typically has speech characteristics with occasional music
        self.is_speech(features) && features.dynamic_range.unwrap_or(0.0) < 20.0
    }
}

/// Dynamics processor for content-aware compression
struct DynamicsProcessor {
    compressor: Compressor,
    expander: Expander,
    gate: Gate,
}

impl DynamicsProcessor {
    fn new() -> Result<Self> {
        Ok(Self {
            compressor: Compressor::new(),
            expander: Expander::new(),
            gate: Gate::new(),
        })
    }

    fn process(&mut self, buffer: &[f32], content_type: &ContentType) -> Result<Vec<f32>> {
        let mut output = buffer.to_vec();

        // Apply content-specific dynamics processing
        match content_type {
            ContentType::Speech => {
                // Gentle compression for speech
                self.compressor.set_ratio(3.0);
                self.compressor.set_threshold(-20.0);
                output = self.compressor.process(&output)?;

                // Gate to remove background noise
                self.gate.set_threshold(-40.0);
                output = self.gate.process(&output)?;
            }
            ContentType::Music(genre) => match genre {
                MusicGenre::Classical => {
                    // Minimal processing for classical
                    self.compressor.set_ratio(1.5);
                    self.compressor.set_threshold(-10.0);
                    output = self.compressor.process(&output)?;
                }
                MusicGenre::Electronic | MusicGenre::Pop => {
                    // More aggressive for electronic/pop
                    self.compressor.set_ratio(4.0);
                    self.compressor.set_threshold(-15.0);
                    output = self.compressor.process(&output)?;
                }
                _ => {
                    // Standard processing
                    self.compressor.set_ratio(2.5);
                    self.compressor.set_threshold(-18.0);
                    output = self.compressor.process(&output)?;
                }
            },
            _ => {
                // Balanced processing for mixed content
                self.compressor.set_ratio(2.0);
                self.compressor.set_threshold(-20.0);
                output = self.compressor.process(&output)?;
            }
        }

        Ok(output)
    }
}

/// Simple compressor implementation
struct Compressor {
    threshold: f32,
    ratio: f32,
    attack: f32,
    release: f32,
    envelope: f32,
}

impl Compressor {
    fn new() -> Self {
        Self {
            threshold: -20.0,
            ratio: 2.0,
            attack: 0.001,
            release: 0.1,
            envelope: 0.0,
        }
    }

    fn process(&mut self, buffer: &[f32]) -> Result<Vec<f32>> {
        let mut output = Vec::with_capacity(buffer.len());
        let threshold_linear = 10.0_f32.powf(self.threshold / 20.0);

        for &sample in buffer {
            let input_level = sample.abs();

            // Update envelope
            let target = if input_level > self.envelope {
                self.attack
            } else {
                self.release
            };
            self.envelope += (input_level - self.envelope) * target;

            // Calculate gain reduction
            let gain = if self.envelope > threshold_linear {
                let excess = self.envelope / threshold_linear;
                let compressed = excess.powf(1.0 / self.ratio);
                threshold_linear * compressed / self.envelope
            } else {
                1.0
            };

            output.push(sample * gain);
        }

        Ok(output)
    }

    fn set_threshold(&mut self, threshold: f32) {
        self.threshold = threshold;
    }

    fn set_ratio(&mut self, ratio: f32) {
        self.ratio = ratio.max(1.0);
    }
}

/// Simple expander implementation
struct Expander {
    threshold: f32,
    ratio: f32,
}

impl Expander {
    fn new() -> Self {
        Self {
            threshold: -40.0,
            ratio: 2.0,
        }
    }
}

/// Simple gate implementation
struct Gate {
    threshold: f32,
    envelope: f32,
}

impl Gate {
    fn new() -> Self {
        Self {
            threshold: -40.0,
            envelope: 0.0,
        }
    }

    fn process(&mut self, buffer: &[f32]) -> Result<Vec<f32>> {
        let mut output = Vec::with_capacity(buffer.len());
        let threshold_linear = 10.0_f32.powf(self.threshold / 20.0);

        for &sample in buffer {
            self.envelope = self.envelope * 0.999 + sample.abs() * 0.001;

            let gain = if self.envelope < threshold_linear {
                (self.envelope / threshold_linear).powi(2)
            } else {
                1.0
            };

            output.push(sample * gain);
        }

        Ok(output)
    }

    fn set_threshold(&mut self, threshold: f32) {
        self.threshold = threshold;
    }
}

/// K-weighting filter implementing ITU-R BS.1770-4 standard
/// Two-stage filtering: pre-filter (high-shelf) + RLB filter (high-pass)
struct KWeightingFilter {
    // Pre-filter (high-shelf) coefficients
    pre_b0: f32,
    pre_b1: f32,
    pre_b2: f32,
    pre_a1: f32,
    pre_a2: f32,

    // RLB filter (high-pass) coefficients
    rlb_b0: f32,
    rlb_b1: f32,
    rlb_b2: f32,
    rlb_a1: f32,
    rlb_a2: f32,

    // Pre-filter state variables (for biquad IIR)
    pre_x1: f32,
    pre_x2: f32,
    pre_y1: f32,
    pre_y2: f32,

    // RLB filter state variables
    rlb_x1: f32,
    rlb_x2: f32,
    rlb_y1: f32,
    rlb_y2: f32,
}

impl KWeightingFilter {
    /// Create new K-weighting filter for given sample rate
    fn new(sample_rate: f32) -> Self {
        use std::f32::consts::PI;

        // Stage 1: Pre-filter (high-shelf)
        // fc = 1681.97 Hz, Q = 0.7071, Gain = 3.999843853973347 dB
        let fc_pre = 1681.97;
        let gain_db = 3.999843853973347;
        let k_pre = (PI * fc_pre / sample_rate).tan();
        let vh = 10.0_f32.powf(gain_db / 20.0);
        let sqrt_vh = vh.sqrt();

        let denominator_pre = 1.0 + 2.0_f32.sqrt() * k_pre + k_pre * k_pre;
        let pre_b0 = (vh + sqrt_vh * k_pre + k_pre * k_pre) / denominator_pre;
        let pre_b1 = 2.0 * (k_pre * k_pre - vh) / denominator_pre;
        let pre_b2 = (vh - sqrt_vh * k_pre + k_pre * k_pre) / denominator_pre;
        let pre_a1 = 2.0 * (k_pre * k_pre - 1.0) / denominator_pre;
        let pre_a2 = (1.0 - 2.0_f32.sqrt() * k_pre + k_pre * k_pre) / denominator_pre;

        // Stage 2: RLB filter (high-pass)
        // fc = 38.13547 Hz, Q = 0.5003270373253953
        let fc_rlb = 38.13547;
        let q_rlb = 0.5003270373253953;
        let k_rlb = (PI * fc_rlb / sample_rate).tan();

        let denominator_rlb = 1.0 + k_rlb / q_rlb + k_rlb * k_rlb;
        let rlb_b0 = 1.0 / denominator_rlb;
        let rlb_b1 = -2.0 * rlb_b0;
        let rlb_b2 = rlb_b0;
        let rlb_a1 = 2.0 * (k_rlb * k_rlb - 1.0) / denominator_rlb;
        let rlb_a2 = (1.0 - k_rlb / q_rlb + k_rlb * k_rlb) / denominator_rlb;

        Self {
            pre_b0,
            pre_b1,
            pre_b2,
            pre_a1,
            pre_a2,
            rlb_b0,
            rlb_b1,
            rlb_b2,
            rlb_a1,
            rlb_a2,
            pre_x1: 0.0,
            pre_x2: 0.0,
            pre_y1: 0.0,
            pre_y2: 0.0,
            rlb_x1: 0.0,
            rlb_x2: 0.0,
            rlb_y1: 0.0,
            rlb_y2: 0.0,
        }
    }

    /// Process a single sample through K-weighting filters
    fn process(&mut self, sample: f32) -> f32 {
        // Stage 1: Pre-filter (high-shelf)
        let pre_output =
            self.pre_b0 * sample + self.pre_b1 * self.pre_x1 + self.pre_b2 * self.pre_x2
                - self.pre_a1 * self.pre_y1
                - self.pre_a2 * self.pre_y2;

        // Update pre-filter state
        self.pre_x2 = self.pre_x1;
        self.pre_x1 = sample;
        self.pre_y2 = self.pre_y1;
        self.pre_y1 = pre_output;

        // Stage 2: RLB filter (high-pass)
        let rlb_output =
            self.rlb_b0 * pre_output + self.rlb_b1 * self.rlb_x1 + self.rlb_b2 * self.rlb_x2
                - self.rlb_a1 * self.rlb_y1
                - self.rlb_a2 * self.rlb_y2;

        // Update RLB filter state
        self.rlb_x2 = self.rlb_x1;
        self.rlb_x1 = pre_output;
        self.rlb_y2 = self.rlb_y1;
        self.rlb_y1 = rlb_output;

        rlb_output
    }

    /// Reset filter state (e.g., between tracks)
    fn reset(&mut self) {
        self.pre_x1 = 0.0;
        self.pre_x2 = 0.0;
        self.pre_y1 = 0.0;
        self.pre_y2 = 0.0;
        self.rlb_x1 = 0.0;
        self.rlb_x2 = 0.0;
        self.rlb_y1 = 0.0;
        self.rlb_y2 = 0.0;
    }
}

/// LUFS/LKFS loudness meter implementing ITU-R BS.1770-4 standard
struct LoudnessMeter {
    sample_rate: u32,
    block_size: usize,
    blocks: VecDeque<f32>,
    k_weighting_filter: KWeightingFilter,
    gate_threshold_absolute: f32, // -70 LKFS
    gate_threshold_relative: f32, // -10 LU relative to ungated loudness
}

impl LoudnessMeter {
    fn new() -> Result<Self> {
        let sample_rate = 48000;
        Ok(Self {
            sample_rate,
            block_size: 400, // 400ms blocks for integrated loudness
            blocks: VecDeque::with_capacity(75), // 30 seconds of blocks
            k_weighting_filter: KWeightingFilter::new(sample_rate as f32),
            gate_threshold_absolute: -70.0,
            gate_threshold_relative: -10.0,
        })
    }

    /// Measure integrated loudness using ITU-R BS.1770-4 algorithm
    fn measure_integrated(&mut self, buffer: &[f32]) -> Result<f32> {
        // Reset filter state for new measurement
        self.k_weighting_filter.reset();

        // Apply K-weighting filter to all samples and calculate mean square
        let mut sum_squares = 0.0;
        for &sample in buffer {
            let weighted_sample = self.k_weighting_filter.process(sample);
            sum_squares += weighted_sample * weighted_sample;
        }

        let mean_square = sum_squares / buffer.len() as f32;

        // Convert to LUFS using ITU-R BS.1770-4 formula
        // LUFS = -0.691 + 10 * log10(mean_square)
        let block_loudness = if mean_square > 1e-10 {
            -0.691 + 10.0 * mean_square.log10()
        } else {
            -70.0 // Silence floor
        };

        // Store block loudness for gated integration
        self.blocks.push_back(block_loudness);
        if self.blocks.len() > 75 {
            self.blocks.pop_front();
        }

        // Apply gating as per ITU-R BS.1770-4
        // Stage 1: Absolute gate at -70 LKFS
        let gated_blocks: Vec<f32> = self
            .blocks
            .iter()
            .copied()
            .filter(|&loudness| loudness > self.gate_threshold_absolute)
            .collect();

        if gated_blocks.is_empty() {
            return Ok(-70.0); // All blocks below absolute gate
        }

        // Stage 2: Relative gate at -10 LU below ungated mean
        let ungated_mean = gated_blocks.iter().sum::<f32>() / gated_blocks.len() as f32;
        let relative_gate = ungated_mean + self.gate_threshold_relative;

        let relative_gated_blocks: Vec<f32> = gated_blocks
            .into_iter()
            .filter(|&loudness| loudness > relative_gate)
            .collect();

        if relative_gated_blocks.is_empty() {
            return Ok(ungated_mean); // Fall back to ungated mean
        }

        // Final integrated loudness
        let integrated =
            relative_gated_blocks.iter().sum::<f32>() / relative_gated_blocks.len() as f32;
        Ok(integrated)
    }

    /// Set sample rate (requires filter recalculation)
    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        self.sample_rate = sample_rate;
        self.k_weighting_filter = KWeightingFilter::new(sample_rate as f32);
    }
}

/// Content type classification
#[derive(Debug, Clone)]
enum ContentType {
    Speech,
    Music(MusicGenre),
    Podcast,
    Mixed,
}

#[derive(Debug, Clone)]
enum MusicGenre {
    Classical,
    Electronic,
    Rock,
    Jazz,
    Pop,
}
