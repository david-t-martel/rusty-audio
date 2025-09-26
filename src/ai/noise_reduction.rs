//! AI-Powered Noise Reduction Module
//!
//! Implements intelligent noise reduction using spectral subtraction,
//! Wiener filtering, and ML-based noise profile detection.

use anyhow::{Result, Context};
use rustfft::{FftPlanner, num_complex::Complex};
use std::collections::VecDeque;
use crate::ai::feature_extractor::AudioFeatures;

/// Noise reduction processor using AI techniques
pub struct NoiseReducer {
    fft_planner: FftPlanner<f32>,
    window_size: usize,
    hop_size: usize,
    noise_profile: NoiseProfile,
    reduction_strength: f32,
    smoothing_factor: f32,
    spectral_floor: f32,
    history_buffer: VecDeque<Vec<f32>>,
    adaptive_threshold: AdaptiveThreshold,
}

impl NoiseReducer {
    pub fn new() -> Result<Self> {
        Ok(Self {
            fft_planner: FftPlanner::new(),
            window_size: 2048,
            hop_size: 512,
            noise_profile: NoiseProfile::default(),
            reduction_strength: 0.8,
            smoothing_factor: 0.95,
            spectral_floor: 0.002,
            history_buffer: VecDeque::with_capacity(10),
            adaptive_threshold: AdaptiveThreshold::new(),
        })
    }

    /// Process audio buffer with intelligent noise reduction
    pub fn process(&mut self, buffer: &[f32], features: &AudioFeatures) -> Result<Vec<f32>> {
        // Detect and update noise profile
        self.update_noise_profile(buffer, features)?;

        // Apply multi-stage noise reduction
        let mut processed = buffer.to_vec();

        // Stage 1: Spectral subtraction
        processed = self.spectral_subtraction(&processed)?;

        // Stage 2: Wiener filtering
        processed = self.wiener_filter(&processed)?;

        // Stage 3: Adaptive gating
        processed = self.adaptive_gate(&processed, features)?;

        // Stage 4: Spectral smoothing
        processed = self.spectral_smoothing(&processed)?;

        // Stage 5: Residual noise suppression
        processed = self.suppress_residual_noise(&processed)?;

        Ok(processed)
    }

    /// Update noise profile using ML-based detection
    fn update_noise_profile(&mut self, buffer: &[f32], features: &AudioFeatures) -> Result<()> {
        // Detect if current segment is noise
        let is_noise = self.detect_noise_segment(buffer, features)?;

        if is_noise {
            // Update noise profile with current spectrum
            let spectrum = self.compute_spectrum(buffer)?;
            self.noise_profile.update(&spectrum, 0.1); // Slow adaptation
        }

        Ok(())
    }

    /// Detect if current audio segment is primarily noise
    fn detect_noise_segment(&self, buffer: &[f32], features: &AudioFeatures) -> Result<bool> {
        let mut noise_indicators = 0;
        let mut total_checks = 0;

        // Check energy level
        if let Some(energy) = features.energy {
            total_checks += 1;
            if energy < 0.01 {
                noise_indicators += 1;
            }
        }

        // Check zero crossing rate (high for noise)
        if let Some(zcr) = features.zero_crossing_rate {
            total_checks += 1;
            if zcr > 0.4 {
                noise_indicators += 1;
            }
        }

        // Check spectral flatness (high for white noise)
        if let Some(flatness) = self.calculate_spectral_flatness(buffer) {
            total_checks += 1;
            if flatness > 0.8 {
                noise_indicators += 1;
            }
        }

        // Check for absence of harmonic content
        if let Some(harmonics) = &features.harmonic_peaks {
            total_checks += 1;
            if harmonics.len() < 2 {
                noise_indicators += 1;
            }
        }

        Ok(noise_indicators as f32 / total_checks.max(1) as f32 > 0.6)
    }

    /// Spectral subtraction noise reduction
    fn spectral_subtraction(&mut self, buffer: &[f32]) -> Result<Vec<f32>> {
        let mut output = Vec::with_capacity(buffer.len());
        let mut overlap_buffer = vec![0.0; self.window_size];

        for i in (0..buffer.len()).step_by(self.hop_size) {
            let end = (i + self.window_size).min(buffer.len());
            let mut frame = vec![Complex::new(0.0, 0.0); self.window_size];

            // Copy and window the frame
            for j in 0..end - i {
                let window = 0.5 * (1.0 - (2.0 * std::f32::consts::PI * j as f32 / self.window_size as f32).cos());
                frame[j] = Complex::new(buffer[i + j] * window, 0.0);
            }

            // FFT
            let mut spectrum = frame.clone();
            let fft = self.fft_planner.plan_fft_forward(self.window_size);
            fft.process(&mut spectrum);

            // Spectral subtraction
            for (j, spec) in spectrum.iter_mut().enumerate() {
                let magnitude = spec.norm();
                let phase = spec.arg();

                // Subtract noise spectrum with oversubtraction factor
                let noise_level = self.noise_profile.get_noise_level(j);
                let alpha = self.calculate_oversubtraction_factor(magnitude, noise_level);
                let clean_magnitude = (magnitude.powi(2) - alpha * noise_level.powi(2)).max(0.0).sqrt();

                // Apply spectral floor to prevent musical noise
                let final_magnitude = clean_magnitude.max(self.spectral_floor * magnitude);

                // Reconstruct complex number
                *spec = Complex::from_polar(final_magnitude, phase);
            }

            // IFFT
            let ifft = self.fft_planner.plan_fft_inverse(self.window_size);
            ifft.process(&mut spectrum);

            // Overlap-add
            for j in 0..self.window_size {
                let sample = spectrum[j].re / self.window_size as f32;
                if i + j < buffer.len() {
                    if j < self.hop_size {
                        output.push(sample + overlap_buffer[j]);
                    } else {
                        overlap_buffer[j - self.hop_size] = sample;
                    }
                }
            }
        }

        Ok(output)
    }

    /// Wiener filtering for additional noise reduction
    fn wiener_filter(&mut self, buffer: &[f32]) -> Result<Vec<f32>> {
        let mut output = vec![0.0; buffer.len()];

        for i in (0..buffer.len()).step_by(self.hop_size) {
            let end = (i + self.window_size).min(buffer.len());
            let mut frame = vec![Complex::new(0.0, 0.0); self.window_size];

            // Copy frame
            for j in 0..end - i {
                frame[j] = Complex::new(buffer[i + j], 0.0);
            }

            // FFT
            let fft = self.fft_planner.plan_fft_forward(self.window_size);
            fft.process(&mut frame);

            // Apply Wiener filter
            for (j, spec) in frame.iter_mut().enumerate() {
                let signal_power = spec.norm().powi(2);
                let noise_power = self.noise_profile.get_noise_level(j).powi(2);

                // Wiener filter gain
                let gain = signal_power / (signal_power + noise_power * self.reduction_strength);
                *spec = *spec * gain;
            }

            // IFFT
            let ifft = self.fft_planner.plan_fft_inverse(self.window_size);
            ifft.process(&mut frame);

            // Copy to output
            for j in 0..end - i {
                output[i + j] += frame[j].re / self.window_size as f32;
            }
        }

        Ok(output)
    }

    /// Adaptive gating based on AI analysis
    fn adaptive_gate(&mut self, buffer: &[f32], features: &AudioFeatures) -> Result<Vec<f32>> {
        let mut output = buffer.to_vec();

        // Calculate adaptive threshold
        let threshold = self.adaptive_threshold.calculate(buffer, features)?;

        // Apply smooth gating
        for sample in &mut output {
            let envelope = sample.abs();
            if envelope < threshold {
                // Smooth gate to prevent clicks
                let gate_factor = (envelope / threshold).powi(2);
                *sample *= gate_factor;
            }
        }

        Ok(output)
    }

    /// Spectral smoothing to reduce musical noise artifacts
    fn spectral_smoothing(&mut self, buffer: &[f32]) -> Result<Vec<f32>> {
        let spectrum = self.compute_spectrum(buffer)?;

        // Store in history buffer
        self.history_buffer.push_back(spectrum.clone());
        if self.history_buffer.len() > 5 {
            self.history_buffer.pop_front();
        }

        // Average spectra over time
        let mut smoothed_spectrum = vec![0.0; spectrum.len()];
        for hist_spectrum in &self.history_buffer {
            for (i, &val) in hist_spectrum.iter().enumerate() {
                smoothed_spectrum[i] += val / self.history_buffer.len() as f32;
            }
        }

        // Reconstruct audio from smoothed spectrum
        self.reconstruct_from_spectrum(&smoothed_spectrum, buffer.len())
    }

    /// Suppress residual noise using AI-based detection
    fn suppress_residual_noise(&mut self, buffer: &[f32]) -> Result<Vec<f32>> {
        let mut output = buffer.to_vec();

        // Detect residual noise patterns
        let residual_noise = self.detect_residual_patterns(buffer)?;

        for region in residual_noise {
            // Apply targeted suppression
            for i in region.start..region.end.min(output.len()) {
                output[i] *= 1.0 - region.suppression_amount;
            }
        }

        Ok(output)
    }

    /// Calculate oversubtraction factor for spectral subtraction
    fn calculate_oversubtraction_factor(&self, signal_magnitude: f32, noise_magnitude: f32) -> f32 {
        let snr = if noise_magnitude > 0.0 {
            signal_magnitude / noise_magnitude
        } else {
            100.0
        };

        // Higher oversubtraction for lower SNR
        if snr < 1.0 {
            2.5
        } else if snr < 5.0 {
            2.0
        } else if snr < 10.0 {
            1.5
        } else {
            1.0
        }
    }

    /// Compute spectrum of audio buffer
    fn compute_spectrum(&mut self, buffer: &[f32]) -> Result<Vec<f32>> {
        let mut frame = vec![Complex::new(0.0, 0.0); self.window_size];

        for i in 0..self.window_size.min(buffer.len()) {
            frame[i] = Complex::new(buffer[i], 0.0);
        }

        let fft = self.fft_planner.plan_fft_forward(self.window_size);
        fft.process(&mut frame);

        Ok(frame.iter().map(|c| c.norm()).collect())
    }

    /// Reconstruct audio from spectrum
    fn reconstruct_from_spectrum(&mut self, spectrum: &[f32], target_length: usize) -> Result<Vec<f32>> {
        let mut frame = vec![Complex::new(0.0, 0.0); self.window_size];

        // Assume zero phase for simplicity (can be improved with phase vocoder)
        for (i, &mag) in spectrum.iter().enumerate() {
            frame[i] = Complex::new(mag, 0.0);
        }

        let ifft = self.fft_planner.plan_fft_inverse(self.window_size);
        ifft.process(&mut frame);

        let mut output = vec![0.0; target_length];
        for i in 0..target_length.min(self.window_size) {
            output[i] = frame[i].re / self.window_size as f32;
        }

        Ok(output)
    }

    /// Calculate spectral flatness measure
    fn calculate_spectral_flatness(&self, buffer: &[f32]) -> Option<f32> {
        let mut spectrum = vec![0.0; self.window_size / 2];

        // Simple FFT magnitude calculation
        for (i, chunk) in buffer.chunks(2).enumerate() {
            if i < spectrum.len() && chunk.len() == 2 {
                spectrum[i] = (chunk[0].powi(2) + chunk[1].powi(2)).sqrt();
            }
        }

        // Geometric mean
        let geometric_mean: f32 = spectrum
            .iter()
            .filter(|&&x| x > 0.0)
            .map(|x| x.ln())
            .sum::<f32>()
            .exp()
            .powf(1.0 / spectrum.len() as f32);

        // Arithmetic mean
        let arithmetic_mean = spectrum.iter().sum::<f32>() / spectrum.len() as f32;

        if arithmetic_mean > 0.0 {
            Some(geometric_mean / arithmetic_mean)
        } else {
            None
        }
    }

    /// Detect residual noise patterns
    fn detect_residual_patterns(&self, buffer: &[f32]) -> Result<Vec<NoiseRegion>> {
        let mut regions = Vec::new();
        let window_size = 256;

        for i in (0..buffer.len()).step_by(window_size) {
            let end = (i + window_size).min(buffer.len());
            let window = &buffer[i..end];

            // Check for periodic noise patterns
            if self.is_periodic_noise(window) {
                regions.push(NoiseRegion {
                    start: i,
                    end,
                    suppression_amount: 0.5,
                    noise_type: NoiseType::Periodic,
                });
            }

            // Check for broadband noise
            if self.is_broadband_noise(window) {
                regions.push(NoiseRegion {
                    start: i,
                    end,
                    suppression_amount: 0.3,
                    noise_type: NoiseType::Broadband,
                });
            }
        }

        Ok(regions)
    }

    /// Check if window contains periodic noise
    fn is_periodic_noise(&self, window: &[f32]) -> bool {
        // Simplified autocorrelation check
        let mut autocorr = 0.0;
        let lag = 64;

        if window.len() > lag * 2 {
            for i in 0..window.len() - lag {
                autocorr += window[i] * window[i + lag];
            }
            autocorr /= (window.len() - lag) as f32;
            autocorr.abs() > 0.7
        } else {
            false
        }
    }

    /// Check if window contains broadband noise
    fn is_broadband_noise(&self, window: &[f32]) -> bool {
        let variance = self.calculate_variance(window);
        let mean = window.iter().sum::<f32>() / window.len() as f32;

        // High variance relative to mean indicates broadband noise
        variance > mean.abs() * 2.0
    }

    /// Calculate variance of window
    fn calculate_variance(&self, window: &[f32]) -> f32 {
        let mean = window.iter().sum::<f32>() / window.len() as f32;
        window.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / window.len() as f32
    }

    /// Set reduction strength (0.0 to 1.0)
    pub fn set_strength(&mut self, strength: f32) {
        self.reduction_strength = strength.max(0.0).min(1.0);
    }

    /// Set spectral floor to prevent over-suppression
    pub fn set_spectral_floor(&mut self, floor: f32) {
        self.spectral_floor = floor.max(0.001).min(0.1);
    }
}

/// Noise profile learned from audio
#[derive(Debug, Default)]
struct NoiseProfile {
    spectrum: Vec<f32>,
    confidence: f32,
}

impl NoiseProfile {
    fn update(&mut self, new_spectrum: &[f32], alpha: f32) {
        if self.spectrum.len() != new_spectrum.len() {
            self.spectrum = new_spectrum.to_vec();
            self.confidence = 0.5;
        } else {
            // Exponential moving average
            for (old, &new) in self.spectrum.iter_mut().zip(new_spectrum) {
                *old = *old * (1.0 - alpha) + new * alpha;
            }
            self.confidence = (self.confidence + 0.1).min(1.0);
        }
    }

    fn get_noise_level(&self, bin: usize) -> f32 {
        self.spectrum.get(bin).copied().unwrap_or(0.0) * self.confidence
    }
}

/// Adaptive threshold calculator
struct AdaptiveThreshold {
    history: VecDeque<f32>,
    window_size: usize,
}

impl AdaptiveThreshold {
    fn new() -> Self {
        Self {
            history: VecDeque::with_capacity(100),
            window_size: 100,
        }
    }

    fn calculate(&mut self, buffer: &[f32], features: &AudioFeatures) -> Result<f32> {
        let current_level = buffer.iter().map(|x| x.abs()).sum::<f32>() / buffer.len() as f32;

        self.history.push_back(current_level);
        if self.history.len() > self.window_size {
            self.history.pop_front();
        }

        // Calculate adaptive threshold based on statistics
        let mean = self.history.iter().sum::<f32>() / self.history.len() as f32;
        let variance = self.history
            .iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f32>() / self.history.len() as f32;

        // Threshold is mean minus one standard deviation
        Ok((mean - variance.sqrt()).max(0.001))
    }
}

/// Detected noise region
#[derive(Debug)]
struct NoiseRegion {
    start: usize,
    end: usize,
    suppression_amount: f32,
    noise_type: NoiseType,
}

#[derive(Debug)]
enum NoiseType {
    Periodic,
    Broadband,
    Impulse,
    Tonal,
}