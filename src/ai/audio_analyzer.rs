//! Intelligent Audio Analysis Module
//!
//! Performs comprehensive audio analysis using FFT, statistical analysis,
//! and pattern recognition for intelligent feature extraction.

use anyhow::{Context, Result};
use rustfft::{num_complex::Complex, FftPlanner};
use std::cmp::Ordering;
use std::f32::consts::PI;

/// Audio analyzer for intelligent analysis
pub struct AudioAnalyzer {
    fft_planner: FftPlanner<f32>,
    window_size: usize,
    hop_size: usize,
}

impl AudioAnalyzer {
    pub fn new() -> Result<Self> {
        Ok(Self {
            fft_planner: FftPlanner::new(),
            window_size: 2048,
            hop_size: 512,
        })
    }

    /// Analyze audio buffer and extract comprehensive features
    pub fn analyze(&mut self, buffer: &[f32], sample_rate: u32) -> Result<AudioAnalysis> {
        let spectral_features = self.extract_spectral_features(buffer, sample_rate)?;
        let temporal_features = self.extract_temporal_features(buffer)?;
        let statistical_features = self.extract_statistical_features(buffer)?;
        let rhythm_features = self.extract_rhythm_features(buffer, sample_rate)?;

        let overall_quality_score =
            self.calculate_quality_score(&spectral_features, &temporal_features);

        Ok(AudioAnalysis {
            spectral_features,
            temporal_features,
            statistical_features,
            rhythm_features,
            overall_quality_score,
        })
    }

    /// Extract spectral features using FFT
    fn extract_spectral_features(
        &mut self,
        buffer: &[f32],
        sample_rate: u32,
    ) -> Result<SpectralFeatures> {
        let mut windowed_buffer = vec![Complex::new(0.0, 0.0); self.window_size];

        // Apply Hann window
        for i in 0..self.window_size.min(buffer.len()) {
            let window = 0.5 * (1.0 - (2.0 * PI * i as f32 / self.window_size as f32).cos());
            windowed_buffer[i] = Complex::new(buffer[i] * window, 0.0);
        }

        // Perform FFT
        let fft = self.fft_planner.plan_fft_forward(self.window_size);
        fft.process(&mut windowed_buffer);

        // Calculate magnitude spectrum
        let magnitude_spectrum: Vec<f32> = windowed_buffer[..self.window_size / 2]
            .iter()
            .map(|c| c.norm())
            .collect();

        // Calculate spectral features
        let spectral_centroid = self.calculate_spectral_centroid(&magnitude_spectrum, sample_rate);
        let spectral_rolloff = self.calculate_spectral_rolloff(&magnitude_spectrum, 0.85);
        let spectral_flux = self.calculate_spectral_flux(&magnitude_spectrum);
        let spectral_entropy = self.calculate_spectral_entropy(&magnitude_spectrum);
        let mfcc = self.calculate_mfcc(&magnitude_spectrum, sample_rate)?;

        Ok(SpectralFeatures {
            spectral_centroid,
            spectral_rolloff,
            spectral_flux,
            spectral_entropy,
            mfcc,
            harmonic_peaks: self.detect_harmonic_peaks(&magnitude_spectrum),
            fundamental_frequency: self
                .estimate_fundamental_frequency(&magnitude_spectrum, sample_rate),
        })
    }

    /// Extract temporal features from audio
    fn extract_temporal_features(&self, buffer: &[f32]) -> Result<TemporalFeatures> {
        let zero_crossing_rate = self.calculate_zero_crossing_rate(buffer);
        let energy = self.calculate_energy(buffer);
        let rms = self.calculate_rms(buffer);
        let peak_amplitude = buffer.iter().map(|x| x.abs()).fold(0.0f32, f32::max);
        let crest_factor = if rms > 0.0 { peak_amplitude / rms } else { 0.0 };

        Ok(TemporalFeatures {
            zero_crossing_rate,
            energy,
            rms,
            peak_amplitude,
            crest_factor,
            dynamic_range: self.calculate_dynamic_range(buffer),
        })
    }

    /// Extract statistical features
    fn extract_statistical_features(&self, buffer: &[f32]) -> Result<StatisticalFeatures> {
        let mean = buffer.iter().sum::<f32>() / buffer.len() as f32;
        let variance = buffer.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / buffer.len() as f32;
        let std_dev = variance.sqrt();
        let skewness = self.calculate_skewness(buffer, mean, std_dev);
        let kurtosis = self.calculate_kurtosis(buffer, mean, std_dev);

        Ok(StatisticalFeatures {
            mean,
            variance,
            std_dev,
            skewness,
            kurtosis,
            median: self.calculate_median(buffer),
        })
    }

    /// Extract rhythm and tempo features
    fn extract_rhythm_features(&self, buffer: &[f32], sample_rate: u32) -> Result<RhythmFeatures> {
        let tempo = self.estimate_tempo(buffer, sample_rate)?;
        let beat_strength = self.calculate_beat_strength(buffer, sample_rate)?;
        let onset_density = self.calculate_onset_density(buffer, sample_rate)?;

        Ok(RhythmFeatures {
            tempo,
            beat_strength,
            onset_density,
            rhythm_regularity: self.calculate_rhythm_regularity(buffer, sample_rate)?,
        })
    }

    /// Calculate spectral centroid
    fn calculate_spectral_centroid(&self, spectrum: &[f32], sample_rate: u32) -> f32 {
        let mut weighted_sum = 0.0;
        let mut magnitude_sum = 0.0;

        for (i, &magnitude) in spectrum.iter().enumerate() {
            let frequency = i as f32 * sample_rate as f32 / (2.0 * spectrum.len() as f32);
            weighted_sum += frequency * magnitude;
            magnitude_sum += magnitude;
        }

        if magnitude_sum > 0.0 {
            weighted_sum / magnitude_sum
        } else {
            0.0
        }
    }

    /// Calculate spectral rolloff
    fn calculate_spectral_rolloff(&self, spectrum: &[f32], threshold: f32) -> f32 {
        let total_energy: f32 = spectrum.iter().sum();
        let threshold_energy = total_energy * threshold;

        let mut cumulative_energy = 0.0;
        for (i, &magnitude) in spectrum.iter().enumerate() {
            cumulative_energy += magnitude;
            if cumulative_energy >= threshold_energy {
                return i as f32 / spectrum.len() as f32;
            }
        }

        1.0
    }

    /// Calculate spectral flux
    fn calculate_spectral_flux(&self, spectrum: &[f32]) -> f32 {
        // Simplified version - in production, compare with previous frame
        spectrum
            .windows(2)
            .map(|w| (w[1] - w[0]).abs())
            .sum::<f32>()
            / spectrum.len() as f32
    }

    /// Calculate spectral entropy
    fn calculate_spectral_entropy(&self, spectrum: &[f32]) -> f32 {
        let total_energy: f32 = spectrum.iter().sum();
        if total_energy == 0.0 {
            return 0.0;
        }

        let mut entropy = 0.0;
        for &magnitude in spectrum {
            if magnitude > 0.0 {
                let probability = magnitude / total_energy;
                entropy -= probability * probability.log2();
            }
        }

        entropy
    }

    /// Calculate MFCCs (simplified)
    fn calculate_mfcc(&self, spectrum: &[f32], sample_rate: u32) -> Result<Vec<f32>> {
        // Simplified MFCC calculation - in production, use proper mel filterbank
        let num_coeffs = 13;
        let mut mfcc = vec![0.0; num_coeffs];

        // Create simple mel-scale bins
        for i in 0..num_coeffs {
            let start_idx = i * spectrum.len() / num_coeffs;
            let end_idx = ((i + 1) * spectrum.len() / num_coeffs).min(spectrum.len());

            let bin_energy: f32 = spectrum[start_idx..end_idx].iter().sum();
            mfcc[i] = if bin_energy > 0.0 {
                bin_energy.ln()
            } else {
                0.0
            };
        }

        Ok(mfcc)
    }

    /// Detect harmonic peaks in spectrum
    fn detect_harmonic_peaks(&self, spectrum: &[f32]) -> Vec<usize> {
        let mut peaks = Vec::new();

        for i in 1..spectrum.len() - 1 {
            if spectrum[i] > spectrum[i - 1] && spectrum[i] > spectrum[i + 1] {
                peaks.push(i);
            }
        }

        // Sort by magnitude and keep top peaks
        peaks.sort_by(|&a, &b| {
            spectrum[b]
                .partial_cmp(&spectrum[a])
                .unwrap_or(Ordering::Equal)
        });
        peaks.truncate(10);
        peaks.sort();

        peaks
    }

    /// Estimate fundamental frequency
    fn estimate_fundamental_frequency(&self, spectrum: &[f32], sample_rate: u32) -> Option<f32> {
        // Simplified pitch detection using peak finding
        let peaks = self.detect_harmonic_peaks(spectrum);

        if peaks.len() > 1 {
            let freq_resolution = sample_rate as f32 / (2.0 * spectrum.len() as f32);
            Some(peaks[0] as f32 * freq_resolution)
        } else {
            None
        }
    }

    /// Calculate zero crossing rate
    fn calculate_zero_crossing_rate(&self, buffer: &[f32]) -> f32 {
        let mut crossings = 0;
        for i in 1..buffer.len() {
            if buffer[i].signum() != buffer[i - 1].signum() {
                crossings += 1;
            }
        }
        crossings as f32 / buffer.len() as f32
    }

    /// Calculate signal energy
    fn calculate_energy(&self, buffer: &[f32]) -> f32 {
        buffer.iter().map(|x| x * x).sum::<f32>() / buffer.len() as f32
    }

    /// Calculate RMS
    fn calculate_rms(&self, buffer: &[f32]) -> f32 {
        (buffer.iter().map(|x| x * x).sum::<f32>() / buffer.len() as f32).sqrt()
    }

    /// Calculate dynamic range
    fn calculate_dynamic_range(&self, buffer: &[f32]) -> f32 {
        let max = buffer.iter().map(|x| x.abs()).fold(0.0f32, f32::max);
        let min = buffer
            .iter()
            .map(|x| x.abs())
            .filter(|&x| x > 0.0)
            .fold(1.0f32, f32::min);

        if min > 0.0 && max > min {
            20.0 * (max / min).log10()
        } else {
            0.0
        }
    }

    /// Calculate skewness
    fn calculate_skewness(&self, buffer: &[f32], mean: f32, std_dev: f32) -> f32 {
        if std_dev == 0.0 {
            return 0.0;
        }

        let n = buffer.len() as f32;
        let sum: f32 = buffer.iter().map(|x| ((x - mean) / std_dev).powi(3)).sum();
        sum / n
    }

    /// Calculate kurtosis
    fn calculate_kurtosis(&self, buffer: &[f32], mean: f32, std_dev: f32) -> f32 {
        if std_dev == 0.0 {
            return 0.0;
        }

        let n = buffer.len() as f32;
        let sum: f32 = buffer.iter().map(|x| ((x - mean) / std_dev).powi(4)).sum();
        sum / n - 3.0
    }

    /// Calculate median
    fn calculate_median(&self, buffer: &[f32]) -> f32 {
        let mut sorted = buffer.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));

        let mid = sorted.len() / 2;
        if sorted.len() % 2 == 0 {
            (sorted[mid - 1] + sorted[mid]) / 2.0
        } else {
            sorted[mid]
        }
    }

    /// Estimate tempo (simplified BPM detection)
    fn estimate_tempo(&self, buffer: &[f32], sample_rate: u32) -> Result<f32> {
        // Simplified tempo estimation using autocorrelation
        // In production, use more sophisticated onset detection and tempo tracking

        // Energy envelope
        let window_size = 2048;
        let hop_size = 512;
        let mut energy_envelope = Vec::new();

        for i in (0..buffer.len()).step_by(hop_size) {
            let end = (i + window_size).min(buffer.len());
            let energy: f32 = buffer[i..end].iter().map(|x| x * x).sum();
            energy_envelope.push(energy);
        }

        // Autocorrelation
        let min_lag = (60.0 * sample_rate as f32 / (200.0 * hop_size as f32)) as usize; // 200 BPM max
        let max_lag = (60.0 * sample_rate as f32 / (40.0 * hop_size as f32)) as usize; // 40 BPM min

        let mut best_correlation = 0.0;
        let mut best_lag = min_lag;

        for lag in min_lag..max_lag.min(energy_envelope.len() / 2) {
            let mut correlation = 0.0;
            for i in 0..energy_envelope.len() - lag {
                correlation += energy_envelope[i] * energy_envelope[i + lag];
            }

            if correlation > best_correlation {
                best_correlation = correlation;
                best_lag = lag;
            }
        }

        let tempo = 60.0 * sample_rate as f32 / (best_lag as f32 * hop_size as f32);
        Ok(tempo)
    }

    /// Calculate beat strength
    fn calculate_beat_strength(&self, buffer: &[f32], sample_rate: u32) -> Result<f32> {
        // Simplified beat strength calculation
        let energy = self.calculate_energy(buffer);
        let variance = self.extract_statistical_features(buffer)?.variance;

        // Higher variance relative to energy indicates stronger beats
        Ok((variance / (energy + 0.001)).min(1.0))
    }

    /// Calculate onset density
    fn calculate_onset_density(&self, buffer: &[f32], sample_rate: u32) -> Result<f32> {
        // Simplified onset detection using spectral flux
        let window_size = 2048;
        let hop_size = 512;
        let mut onsets = 0;

        for i in (hop_size..buffer.len()).step_by(hop_size) {
            let prev_energy = self.calculate_energy(&buffer[i - hop_size..i]);
            let curr_energy =
                self.calculate_energy(&buffer[i..i.min(i + hop_size).min(buffer.len())]);

            if curr_energy > prev_energy * 1.5 {
                onsets += 1;
            }
        }

        let duration_seconds = buffer.len() as f32 / sample_rate as f32;
        Ok(onsets as f32 / duration_seconds)
    }

    /// Calculate rhythm regularity
    fn calculate_rhythm_regularity(&self, buffer: &[f32], sample_rate: u32) -> Result<f32> {
        // Measures consistency of onset intervals
        // Returns value between 0 (irregular) and 1 (perfectly regular)

        let hop_size = 512;
        let mut onset_times = Vec::new();

        // Detect onsets and record their times
        for i in (hop_size..buffer.len()).step_by(hop_size) {
            let prev_energy = self.calculate_energy(&buffer[i - hop_size..i]);
            let curr_energy =
                self.calculate_energy(&buffer[i..i.min(i + hop_size).min(buffer.len())]);

            if curr_energy > prev_energy * 1.5 {
                onset_times.push(i as f32 / sample_rate as f32);
            }
        }

        // Need at least 3 onsets to measure regularity
        if onset_times.len() < 3 {
            return Ok(0.0);
        }

        // Calculate inter-onset intervals (IOIs)
        let mut intervals = Vec::with_capacity(onset_times.len() - 1);
        for i in 1..onset_times.len() {
            intervals.push(onset_times[i] - onset_times[i - 1]);
        }

        // Calculate coefficient of variation (CV) of intervals
        // Lower CV = more regular rhythm
        let mean_interval = intervals.iter().sum::<f32>() / intervals.len() as f32;

        if mean_interval == 0.0 {
            return Ok(0.0);
        }

        let variance = intervals
            .iter()
            .map(|&interval| {
                let diff = interval - mean_interval;
                diff * diff
            })
            .sum::<f32>()
            / intervals.len() as f32;

        let std_dev = variance.sqrt();
        let cv = std_dev / mean_interval;

        // Convert CV to regularity score (0-1 scale)
        // CV of 0 = perfectly regular (score 1.0)
        // CV of 1+ = very irregular (score approaches 0)
        let regularity = (1.0 / (1.0 + cv * 2.0)).max(0.0).min(1.0);

        Ok(regularity)
    }

    /// Calculate overall quality score
    fn calculate_quality_score(
        &self,
        spectral: &SpectralFeatures,
        temporal: &TemporalFeatures,
    ) -> f32 {
        let mut score = 0.0;

        // Penalize clipping
        if temporal.crest_factor < 3.0 {
            score -= 0.2;
        }

        // Good dynamic range
        if temporal.dynamic_range > 20.0 && temporal.dynamic_range < 60.0 {
            score += 0.3;
        }

        // Good spectral balance
        if spectral.spectral_centroid > 200.0 && spectral.spectral_centroid < 4000.0 {
            score += 0.3;
        }

        // Clear harmonic content
        if spectral.harmonic_peaks.len() > 3 {
            score += 0.2;
        }

        (score + 0.5_f32).max(0.0).min(1.0)
    }
}

/// Complete audio analysis results
#[derive(Debug, Clone)]
pub struct AudioAnalysis {
    pub spectral_features: SpectralFeatures,
    pub temporal_features: TemporalFeatures,
    pub statistical_features: StatisticalFeatures,
    pub rhythm_features: RhythmFeatures,
    pub overall_quality_score: f32,
}

/// Spectral domain features
#[derive(Debug, Clone)]
pub struct SpectralFeatures {
    pub spectral_centroid: f32,
    pub spectral_rolloff: f32,
    pub spectral_flux: f32,
    pub spectral_entropy: f32,
    pub mfcc: Vec<f32>,
    pub harmonic_peaks: Vec<usize>,
    pub fundamental_frequency: Option<f32>,
}

/// Temporal domain features
#[derive(Debug, Clone)]
pub struct TemporalFeatures {
    pub zero_crossing_rate: f32,
    pub energy: f32,
    pub rms: f32,
    pub peak_amplitude: f32,
    pub crest_factor: f32,
    pub dynamic_range: f32,
}

/// Statistical features
#[derive(Debug, Clone)]
pub struct StatisticalFeatures {
    pub mean: f32,
    pub variance: f32,
    pub std_dev: f32,
    pub skewness: f32,
    pub kurtosis: f32,
    pub median: f32,
}

/// Rhythm and tempo features
#[derive(Debug, Clone)]
pub struct RhythmFeatures {
    pub tempo: f32,
    pub beat_strength: f32,
    pub onset_density: f32,
    pub rhythm_regularity: f32,
}
