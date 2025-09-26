//! Automatic Audio Format Optimization Module
//!
//! Intelligently optimizes audio format, codec selection, and bitrate
//! based on content analysis and quality requirements.

use anyhow::{Result, Context};
use crate::ai::feature_extractor::AudioFeatures;

/// Format optimizer for intelligent codec and bitrate selection
pub struct FormatOptimizer {
    quality_analyzer: QualityAnalyzer,
    codec_selector: CodecSelector,
    bitrate_optimizer: BitrateOptimizer,
}

impl FormatOptimizer {
    pub fn new() -> Result<Self> {
        Ok(Self {
            quality_analyzer: QualityAnalyzer::new(),
            codec_selector: CodecSelector::new(),
            bitrate_optimizer: BitrateOptimizer::new(),
        })
    }

    /// Optimize audio format based on content analysis
    pub fn optimize(&mut self, buffer: &[f32], features: &AudioFeatures) -> Result<OptimizationResult> {
        // Analyze current quality
        let quality = self.quality_analyzer.analyze(buffer, features)?;

        // Select optimal codec based on content
        let codec = self.codec_selector.select(&quality, features)?;

        // Calculate optimal bitrate
        let bitrate = self.bitrate_optimizer.calculate(&quality, &codec, features)?;

        // Determine sample rate
        let sample_rate = self.determine_optimal_sample_rate(features)?;

        Ok(OptimizationResult {
            codec,
            bitrate,
            sample_rate,
            quality_score: quality.overall_score,
            estimated_file_size: self.estimate_file_size(buffer.len(), bitrate, sample_rate),
        })
    }

    /// Determine optimal sample rate based on content
    fn determine_optimal_sample_rate(&self, features: &AudioFeatures) -> Result<u32> {
        // Check highest frequency content
        if let Some(rolloff) = features.spectral_rolloff {
            if rolloff < 0.3 {
                // Limited high-frequency content
                Ok(44100)
            } else if rolloff < 0.6 {
                // Standard content
                Ok(48000)
            } else {
                // Rich high-frequency content
                Ok(96000)
            }
        } else {
            Ok(48000) // Default
        }
    }

    /// Estimate resulting file size
    fn estimate_file_size(&self, samples: usize, bitrate: u32, sample_rate: u32) -> usize {
        let duration_seconds = samples as f64 / sample_rate as f64;
        ((bitrate as f64 * duration_seconds) / 8.0) as usize
    }
}

/// Quality analyzer for audio content
struct QualityAnalyzer {
    metrics: QualityMetrics,
}

impl QualityAnalyzer {
    fn new() -> Self {
        Self {
            metrics: QualityMetrics::default(),
        }
    }

    fn analyze(&mut self, buffer: &[f32], features: &AudioFeatures) -> Result<AudioQuality> {
        let mut quality = AudioQuality::default();

        // Analyze frequency response
        quality.frequency_response = self.analyze_frequency_response(features)?;

        // Analyze dynamic range
        quality.dynamic_range = features.dynamic_range.unwrap_or(0.0);

        // Analyze noise level
        quality.noise_level = self.estimate_noise_level(buffer)?;

        // Analyze distortion
        quality.distortion = self.estimate_distortion(buffer)?;

        // Calculate overall quality score
        quality.overall_score = self.calculate_overall_score(&quality);

        Ok(quality)
    }

    fn analyze_frequency_response(&self, features: &AudioFeatures) -> Result<f32> {
        // Analyze frequency response quality
        let mut score = 0.5;

        if let Some(rolloff) = features.spectral_rolloff {
            score += rolloff * 0.3;
        }

        if let Some(centroid) = features.spectral_centroid {
            if centroid > 1000.0 && centroid < 4000.0 {
                score += 0.2;
            }
        }

        Ok(score.min(1.0))
    }

    fn estimate_noise_level(&self, buffer: &[f32]) -> Result<f32> {
        // Simple noise floor estimation
        let sorted_magnitudes = {
            let mut mags: Vec<f32> = buffer.iter().map(|x| x.abs()).collect();
            mags.sort_by(|a, b| a.partial_cmp(b).unwrap());
            mags
        };

        // Use 10th percentile as noise floor estimate
        let noise_floor = sorted_magnitudes[sorted_magnitudes.len() / 10];
        Ok(noise_floor)
    }

    fn estimate_distortion(&self, buffer: &[f32]) -> Result<f32> {
        // Check for clipping
        let clipped_samples = buffer.iter().filter(|&&x| x.abs() > 0.99).count();
        let clipping_ratio = clipped_samples as f32 / buffer.len() as f32;

        Ok(clipping_ratio)
    }

    fn calculate_overall_score(&self, quality: &AudioQuality) -> f32 {
        let mut score = 0.0;

        score += quality.frequency_response * 0.3;
        score += (quality.dynamic_range / 40.0).min(1.0) * 0.3;
        score += (1.0 - quality.noise_level * 10.0).max(0.0) * 0.2;
        score += (1.0 - quality.distortion * 100.0).max(0.0) * 0.2;

        score.min(1.0).max(0.0)
    }
}

/// Codec selector based on content type
struct CodecSelector {
    codec_profiles: Vec<CodecProfile>,
}

impl CodecSelector {
    fn new() -> Self {
        Self {
            codec_profiles: Self::initialize_codec_profiles(),
        }
    }

    fn select(&self, quality: &AudioQuality, features: &AudioFeatures) -> Result<AudioCodec> {
        // High quality content -> lossless
        if quality.overall_score > 0.8 {
            return Ok(AudioCodec::FLAC);
        }

        // Speech content -> optimized for speech
        if self.is_speech(features) {
            return Ok(AudioCodec::Opus);
        }

        // Music content
        if quality.dynamic_range > 20.0 {
            Ok(AudioCodec::AAC)
        } else {
            Ok(AudioCodec::MP3)
        }
    }

    fn is_speech(&self, features: &AudioFeatures) -> bool {
        features.spectral_centroid
            .map(|c| c > 500.0 && c < 3000.0)
            .unwrap_or(false)
    }

    fn initialize_codec_profiles() -> Vec<CodecProfile> {
        vec![
            CodecProfile {
                codec: AudioCodec::FLAC,
                min_quality: 0.8,
                efficiency: 0.5, // Lossless but larger files
            },
            CodecProfile {
                codec: AudioCodec::AAC,
                min_quality: 0.5,
                efficiency: 0.8,
            },
            CodecProfile {
                codec: AudioCodec::Opus,
                min_quality: 0.3,
                efficiency: 0.9,
            },
            CodecProfile {
                codec: AudioCodec::MP3,
                min_quality: 0.3,
                efficiency: 0.7,
            },
        ]
    }
}

/// Bitrate optimizer based on perceptual quality
struct BitrateOptimizer;

impl BitrateOptimizer {
    fn new() -> Self {
        Self
    }

    fn calculate(&self, quality: &AudioQuality, codec: &AudioCodec, features: &AudioFeatures) -> Result<u32> {
        let base_bitrate = match codec {
            AudioCodec::FLAC => return Ok(0), // Lossless
            AudioCodec::AAC => 128000,
            AudioCodec::Opus => 96000,
            AudioCodec::MP3 => 192000,
            AudioCodec::Vorbis => 128000,
        };

        // Adjust based on quality requirements
        let quality_multiplier = 0.5 + quality.overall_score * 1.5;

        // Adjust based on frequency content
        let frequency_multiplier = if let Some(rolloff) = features.spectral_rolloff {
            0.8 + rolloff * 0.4
        } else {
            1.0
        };

        let optimal_bitrate = (base_bitrate as f32 * quality_multiplier * frequency_multiplier) as u32;

        // Snap to standard bitrates
        Ok(self.snap_to_standard_bitrate(optimal_bitrate))
    }

    fn snap_to_standard_bitrate(&self, bitrate: u32) -> u32 {
        const STANDARD_BITRATES: &[u32] = &[64000, 96000, 128000, 160000, 192000, 256000, 320000];

        *STANDARD_BITRATES
            .iter()
            .min_by_key(|&&b| (b as i32 - bitrate as i32).abs())
            .unwrap()
    }
}

/// Audio quality metrics
#[derive(Debug, Default)]
struct AudioQuality {
    frequency_response: f32,
    dynamic_range: f32,
    noise_level: f32,
    distortion: f32,
    overall_score: f32,
}

/// Quality metrics for analysis
#[derive(Debug, Default)]
struct QualityMetrics {
    snr: f32,
    thd: f32,
    frequency_flatness: f32,
}

/// Codec profile information
struct CodecProfile {
    codec: AudioCodec,
    min_quality: f32,
    efficiency: f32,
}

/// Optimization result
#[derive(Debug)]
pub struct OptimizationResult {
    pub codec: AudioCodec,
    pub bitrate: u32,
    pub sample_rate: u32,
    pub quality_score: f32,
    pub estimated_file_size: usize,
}

/// Supported audio codecs
#[derive(Debug, Clone, PartialEq)]
pub enum AudioCodec {
    FLAC,
    AAC,
    Opus,
    MP3,
    Vorbis,
}