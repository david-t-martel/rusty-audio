//! Feature Extraction Module for AI Processing
//!
//! Extracts comprehensive audio features for AI analysis and processing.

use anyhow::{Context, Result};
use rustfft::{num_complex::Complex, FftPlanner};
use std::cmp::Ordering;

/// Comprehensive audio features for AI processing
#[derive(Debug, Clone)]
pub struct AudioFeatures {
    // Spectral features
    pub spectral_centroid: Option<f32>,
    pub spectral_rolloff: Option<f32>,
    pub spectral_flux: Option<f32>,
    pub spectral_entropy: Option<f32>,
    pub spectrum: Option<Vec<f32>>,

    // Energy features
    pub energy: Option<f32>,
    pub bass_energy: Option<f32>,
    pub mid_energy: Option<f32>,
    pub treble_energy: Option<f32>,

    // Temporal features
    pub zero_crossing_rate: Option<f32>,
    pub rms: Option<f32>,
    pub peak_amplitude: Option<f32>,
    pub crest_factor: Option<f32>,

    // Statistical features
    pub mean: Option<f32>,
    pub variance: Option<f32>,
    pub skewness: Option<f32>,
    pub kurtosis: Option<f32>,

    // Rhythm features
    pub tempo: Option<f32>,
    pub beat_strength: Option<f32>,
    pub onset_density: Option<f32>,

    // Harmonic features
    pub harmonic_peaks: Option<Vec<usize>>,
    pub fundamental_frequency: Option<f32>,
    pub pitch_confidence: Option<f32>,

    // Quality features
    pub dynamic_range: Option<f32>,
    pub loudness: Option<f32>,
    pub clarity_index: Option<f32>,

    // MFCC features
    pub mfcc: Option<Vec<f32>>,
}

/// Feature extractor for comprehensive audio analysis
pub struct FeatureExtractor {
    fft_planner: FftPlanner<f32>,
    window_size: usize,
    hop_size: usize,
}

impl FeatureExtractor {
    pub fn new() -> Result<Self> {
        Ok(Self {
            fft_planner: FftPlanner::new(),
            window_size: 2048,
            hop_size: 512,
        })
    }

    /// Extract all features from audio buffer
    pub fn extract(&mut self, buffer: &[f32], sample_rate: u32) -> Result<AudioFeatures> {
        let mut features = AudioFeatures::default();

        // Extract spectral features
        self.extract_spectral_features(&mut features, buffer, sample_rate)?;

        // Extract energy features
        self.extract_energy_features(&mut features, buffer, sample_rate)?;

        // Extract temporal features
        self.extract_temporal_features(&mut features, buffer)?;

        // Extract statistical features
        self.extract_statistical_features(&mut features, buffer)?;

        // Extract rhythm features
        self.extract_rhythm_features(&mut features, buffer, sample_rate)?;

        // Extract harmonic features
        self.extract_harmonic_features(&mut features, buffer, sample_rate)?;

        // Extract quality features
        self.extract_quality_features(&mut features, buffer)?;

        // Extract MFCC features
        self.extract_mfcc_features(&mut features, buffer, sample_rate)?;

        Ok(features)
    }

    /// Extract spectral features
    fn extract_spectral_features(
        &mut self,
        features: &mut AudioFeatures,
        buffer: &[f32],
        sample_rate: u32,
    ) -> Result<()> {
        let spectrum = self.compute_spectrum(buffer)?;

        features.spectral_centroid = Some(self.calculate_spectral_centroid(&spectrum, sample_rate));
        features.spectral_rolloff = Some(self.calculate_spectral_rolloff(&spectrum, 0.85));
        features.spectral_flux = Some(self.calculate_spectral_flux(&spectrum));
        features.spectral_entropy = Some(self.calculate_spectral_entropy(&spectrum));
        features.spectrum = Some(spectrum);

        Ok(())
    }

    /// Extract energy features
    fn extract_energy_features(
        &mut self,
        features: &mut AudioFeatures,
        buffer: &[f32],
        sample_rate: u32,
    ) -> Result<()> {
        let spectrum = features
            .spectrum
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Spectrum not computed"))?;

        let total_energy: f32 = spectrum.iter().sum();
        features.energy = Some(total_energy / spectrum.len() as f32);

        // Calculate band energies
        let bass_end = 250 * spectrum.len() / (sample_rate as usize / 2);
        let mid_end = 4000 * spectrum.len() / (sample_rate as usize / 2);

        features.bass_energy = Some(
            spectrum[..bass_end.min(spectrum.len())].iter().sum::<f32>() / total_energy.max(0.001),
        );
        features.mid_energy = Some(
            spectrum[bass_end.min(spectrum.len())..mid_end.min(spectrum.len())]
                .iter()
                .sum::<f32>()
                / total_energy.max(0.001),
        );
        features.treble_energy = Some(
            spectrum[mid_end.min(spectrum.len())..].iter().sum::<f32>() / total_energy.max(0.001),
        );

        Ok(())
    }

    /// Extract temporal features
    fn extract_temporal_features(
        &mut self,
        features: &mut AudioFeatures,
        buffer: &[f32],
    ) -> Result<()> {
        features.zero_crossing_rate = Some(self.calculate_zero_crossing_rate(buffer));
        features.rms = Some(self.calculate_rms(buffer));
        features.peak_amplitude = Some(buffer.iter().map(|x| x.abs()).fold(0.0f32, f32::max));

        let rms = features.rms.unwrap_or(0.001);
        let peak = features.peak_amplitude.unwrap_or(0.001);
        features.crest_factor = Some(peak / rms);

        Ok(())
    }

    /// Extract statistical features
    fn extract_statistical_features(
        &mut self,
        features: &mut AudioFeatures,
        buffer: &[f32],
    ) -> Result<()> {
        let mean = buffer.iter().sum::<f32>() / buffer.len() as f32;
        let variance = buffer.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / buffer.len() as f32;
        let std_dev = variance.sqrt();

        features.mean = Some(mean);
        features.variance = Some(variance);
        features.skewness = Some(self.calculate_skewness(buffer, mean, std_dev));
        features.kurtosis = Some(self.calculate_kurtosis(buffer, mean, std_dev));

        Ok(())
    }

    /// Extract rhythm features
    fn extract_rhythm_features(
        &mut self,
        features: &mut AudioFeatures,
        buffer: &[f32],
        sample_rate: u32,
    ) -> Result<()> {
        features.tempo = Some(self.estimate_tempo(buffer, sample_rate)?);
        features.beat_strength = Some(self.calculate_beat_strength(buffer)?);
        features.onset_density = Some(self.calculate_onset_density(buffer, sample_rate)?);

        Ok(())
    }

    /// Extract harmonic features
    fn extract_harmonic_features(
        &mut self,
        features: &mut AudioFeatures,
        buffer: &[f32],
        sample_rate: u32,
    ) -> Result<()> {
        let spectrum = features
            .spectrum
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Spectrum not computed"))?;

        let peaks = self.detect_harmonic_peaks(spectrum);
        features.harmonic_peaks = Some(peaks.clone());

        if !peaks.is_empty() {
            let freq_resolution = sample_rate as f32 / (2.0 * spectrum.len() as f32);
            features.fundamental_frequency = Some(peaks[0] as f32 * freq_resolution);
            features.pitch_confidence = Some(self.calculate_pitch_confidence(spectrum, &peaks));
        }

        Ok(())
    }

    /// Extract quality features
    fn extract_quality_features(
        &mut self,
        features: &mut AudioFeatures,
        buffer: &[f32],
    ) -> Result<()> {
        features.dynamic_range = Some(self.calculate_dynamic_range(buffer));
        features.loudness = Some(self.calculate_loudness(buffer));
        features.clarity_index = Some(self.calculate_clarity_index(features));

        Ok(())
    }

    /// Extract MFCC features
    fn extract_mfcc_features(
        &mut self,
        features: &mut AudioFeatures,
        buffer: &[f32],
        sample_rate: u32,
    ) -> Result<()> {
        let spectrum = features
            .spectrum
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Spectrum not computed"))?;

        features.mfcc = Some(self.calculate_mfcc(spectrum, sample_rate)?);

        Ok(())
    }

    // Helper methods (implementations similar to audio_analyzer.rs but simplified)

    fn compute_spectrum(&mut self, buffer: &[f32]) -> Result<Vec<f32>> {
        let mut frame = vec![Complex::new(0.0, 0.0); self.window_size];

        for i in 0..self.window_size.min(buffer.len()) {
            let window = 0.5
                * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / self.window_size as f32).cos());
            frame[i] = Complex::new(buffer[i] * window, 0.0);
        }

        let fft = self.fft_planner.plan_fft_forward(self.window_size);
        fft.process(&mut frame);

        Ok(frame[..self.window_size / 2]
            .iter()
            .map(|c| c.norm())
            .collect())
    }

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

    fn calculate_spectral_flux(&self, spectrum: &[f32]) -> f32 {
        spectrum
            .windows(2)
            .map(|w| (w[1] - w[0]).abs())
            .sum::<f32>()
            / spectrum.len() as f32
    }

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

    fn calculate_zero_crossing_rate(&self, buffer: &[f32]) -> f32 {
        let mut crossings = 0;
        for i in 1..buffer.len() {
            if buffer[i].signum() != buffer[i - 1].signum() {
                crossings += 1;
            }
        }
        crossings as f32 / buffer.len() as f32
    }

    fn calculate_rms(&self, buffer: &[f32]) -> f32 {
        (buffer.iter().map(|x| x * x).sum::<f32>() / buffer.len() as f32).sqrt()
    }

    fn calculate_skewness(&self, buffer: &[f32], mean: f32, std_dev: f32) -> f32 {
        if std_dev == 0.0 {
            return 0.0;
        }

        let n = buffer.len() as f32;
        let sum: f32 = buffer.iter().map(|x| ((x - mean) / std_dev).powi(3)).sum();
        sum / n
    }

    fn calculate_kurtosis(&self, buffer: &[f32], mean: f32, std_dev: f32) -> f32 {
        if std_dev == 0.0 {
            return 0.0;
        }

        let n = buffer.len() as f32;
        let sum: f32 = buffer.iter().map(|x| ((x - mean) / std_dev).powi(4)).sum();
        sum / n - 3.0
    }

    fn estimate_tempo(&self, buffer: &[f32], sample_rate: u32) -> Result<f32> {
        // Tempo estimation using autocorrelation on energy envelope
        let window_size = 2048;
        let hop_size = 512;
        let mut energy_envelope = Vec::new();

        for i in (0..buffer.len()).step_by(hop_size) {
            let end = (i + window_size).min(buffer.len());
            let energy: f32 = buffer[i..end].iter().map(|x| x * x).sum();
            energy_envelope.push(energy);
        }

        // Autocorrelation to find periodic structure
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

    fn calculate_beat_strength(&self, buffer: &[f32]) -> Result<f32> {
        // Beat strength from energy variance
        let energy = self.calculate_energy(buffer);

        // Calculate variance directly
        let mean = buffer.iter().sum::<f32>() / buffer.len() as f32;
        let variance = buffer.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / buffer.len() as f32;

        // Higher variance relative to energy indicates stronger beats
        Ok((variance / (energy + 0.001)).min(1.0))
    }

    fn calculate_onset_density(&self, buffer: &[f32], sample_rate: u32) -> Result<f32> {
        // Onset detection using energy-based spectral flux
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

    fn calculate_energy(&self, buffer: &[f32]) -> f32 {
        buffer.iter().map(|x| x * x).sum()
    }

    fn detect_harmonic_peaks(&self, spectrum: &[f32]) -> Vec<usize> {
        let mut peaks = Vec::new();

        for i in 1..spectrum.len() - 1 {
            if spectrum[i] > spectrum[i - 1] && spectrum[i] > spectrum[i + 1] {
                peaks.push(i);
            }
        }

        peaks.sort_by(|&a, &b| {
            spectrum[b]
                .partial_cmp(&spectrum[a])
                .unwrap_or(Ordering::Equal)
        });
        peaks.truncate(10);
        peaks.sort();

        peaks
    }

    fn calculate_pitch_confidence(&self, spectrum: &[f32], peaks: &[usize]) -> f32 {
        if peaks.len() < 2 {
            return 0.0;
        }

        // Check harmonic relationship between peaks
        let mut confidence = 0.0;
        let fundamental = peaks[0];

        for i in 1..peaks.len().min(5) {
            let expected = fundamental * (i + 1);
            let actual = peaks.get(i).copied().unwrap_or(0);

            if actual > 0 {
                let error = ((expected as f32 - actual as f32) / expected as f32).abs();
                confidence += 1.0 / (1.0 + error * 10.0);
            }
        }

        confidence / 4.0
    }

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

    fn calculate_loudness(&self, buffer: &[f32]) -> f32 {
        // A-weighting approximation for loudness perception
        let rms = self.calculate_rms(buffer);
        20.0 * rms.max(0.00001).log10()
    }

    fn calculate_clarity_index(&self, features: &AudioFeatures) -> f32 {
        let mut clarity = 0.5;

        // Higher clarity with better harmonic structure
        if let Some(confidence) = features.pitch_confidence {
            clarity += confidence * 0.2;
        }

        // Higher clarity with less spectral entropy
        if let Some(entropy) = features.spectral_entropy {
            clarity += (1.0 - entropy / 10.0) * 0.2;
        }

        // Higher clarity with good crest factor
        if let Some(crest) = features.crest_factor {
            if crest > 3.0 && crest < 10.0 {
                clarity += 0.1;
            }
        }

        clarity.min(1.0)
    }

    fn calculate_mfcc(&self, spectrum: &[f32], sample_rate: u32) -> Result<Vec<f32>> {
        let num_coeffs = 13;
        let mut mfcc = vec![0.0; num_coeffs];

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
}

impl Default for AudioFeatures {
    fn default() -> Self {
        Self {
            spectral_centroid: None,
            spectral_rolloff: None,
            spectral_flux: None,
            spectral_entropy: None,
            spectrum: None,
            energy: None,
            bass_energy: None,
            mid_energy: None,
            treble_energy: None,
            zero_crossing_rate: None,
            rms: None,
            peak_amplitude: None,
            crest_factor: None,
            mean: None,
            variance: None,
            skewness: None,
            kurtosis: None,
            tempo: None,
            beat_strength: None,
            onset_density: None,
            harmonic_peaks: None,
            fundamental_frequency: None,
            pitch_confidence: None,
            dynamic_range: None,
            loudness: None,
            clarity_index: None,
            mfcc: None,
        }
    }
}
