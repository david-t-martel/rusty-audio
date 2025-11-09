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
                // Get the oldest sample
                let delayed_sample = self.lookahead_buffer.pop_front().unwrap();

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

/// LUFS/LKFS loudness meter
struct LoudnessMeter {
    sample_rate: u32,
    block_size: usize,
    blocks: VecDeque<f32>,
}

impl LoudnessMeter {
    fn new() -> Result<Self> {
        Ok(Self {
            sample_rate: 48000,
            block_size: 400, // 400ms blocks for integrated loudness
            blocks: VecDeque::with_capacity(75), // 30 seconds of blocks
        })
    }

    fn measure_integrated(&mut self, buffer: &[f32]) -> Result<f32> {
        // Simplified LUFS calculation
        // In production, use proper ITU-R BS.1770 algorithm

        let mut sum = 0.0;
        for &sample in buffer {
            sum += sample * sample;
        }

        let mean_square = sum / buffer.len() as f32;
        let lufs = if mean_square > 0.0 {
            -0.691 + 10.0 * mean_square.log10()
        } else {
            -70.0
        };

        self.blocks.push_back(lufs);
        if self.blocks.len() > 75 {
            self.blocks.pop_front();
        }

        // Return integrated loudness
        let integrated = self.blocks.iter().sum::<f32>() / self.blocks.len() as f32;
        Ok(integrated)
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
