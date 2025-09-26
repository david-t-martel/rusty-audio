//! Intelligent EQ Optimization Module
//!
//! Uses AI to analyze audio characteristics and suggest optimal EQ settings
//! based on genre detection, frequency balance, and masking analysis.

use anyhow::{Result, Context};
use crate::ai::feature_extractor::AudioFeatures;
use super::{EQSettings, EQBand};

/// EQ optimizer using AI-based analysis
pub struct EQOptimizer {
    genre_profiles: Vec<GenreProfile>,
    target_curve: TargetCurve,
    learning_rate: f32,
}

impl EQOptimizer {
    pub fn new() -> Result<Self> {
        Ok(Self {
            genre_profiles: Self::initialize_genre_profiles(),
            target_curve: TargetCurve::default(),
            learning_rate: 0.1,
        })
    }

    /// Optimize EQ settings based on audio features
    pub fn optimize(&mut self, features: &AudioFeatures) -> Result<EQSettings> {
        // Detect genre/style
        let genre = self.detect_genre(features)?;

        // Analyze frequency balance
        let frequency_balance = self.analyze_frequency_balance(features)?;

        // Detect and correct frequency masking
        let masking_correction = self.calculate_masking_correction(features)?;

        // Generate optimized EQ settings
        let bands = self.generate_eq_bands(
            &genre,
            &frequency_balance,
            &masking_correction,
            features,
        )?;

        Ok(EQSettings {
            bands,
            confidence: self.calculate_confidence(features),
            genre_hint: Some(genre.name.clone()),
        })
    }

    /// Detect music genre based on audio features
    fn detect_genre(&self, features: &AudioFeatures) -> Result<GenreProfile> {
        let mut best_match = &self.genre_profiles[0];
        let mut best_score = 0.0;

        for profile in &self.genre_profiles {
            let score = self.calculate_genre_similarity(features, profile)?;
            if score > best_score {
                best_score = score;
                best_match = profile;
            }
        }

        Ok(best_match.clone())
    }

    /// Calculate similarity between audio features and genre profile
    fn calculate_genre_similarity(&self, features: &AudioFeatures, profile: &GenreProfile) -> Result<f32> {
        let mut score = 0.0;

        // Compare spectral centroid
        if let Some(centroid) = features.spectral_centroid {
            let centroid_diff = (centroid - profile.typical_spectral_centroid).abs();
            score += 1.0 / (1.0 + centroid_diff / 1000.0);
        }

        // Compare tempo
        if let Some(tempo) = features.tempo {
            let tempo_diff = (tempo - profile.typical_tempo).abs();
            score += 1.0 / (1.0 + tempo_diff / 50.0);
        }

        // Compare energy distribution
        if let Some(bass_energy) = features.bass_energy {
            let bass_diff = (bass_energy - profile.typical_bass_energy).abs();
            score += 1.0 / (1.0 + bass_diff);
        }

        // Compare dynamic range
        if let Some(dynamic_range) = features.dynamic_range {
            let dr_diff = (dynamic_range - profile.typical_dynamic_range).abs();
            score += 1.0 / (1.0 + dr_diff / 10.0);
        }

        Ok(score / 4.0) // Normalize
    }

    /// Analyze frequency balance and identify issues
    fn analyze_frequency_balance(&self, features: &AudioFeatures) -> Result<FrequencyBalance> {
        let mut balance = FrequencyBalance::default();

        // Analyze bass region (20-250 Hz)
        if let Some(bass_energy) = features.bass_energy {
            balance.bass_level = bass_energy;
            balance.bass_correction = if bass_energy < 0.2 {
                3.0 // Boost
            } else if bass_energy > 0.4 {
                -2.0 // Cut
            } else {
                0.0
            };
        }

        // Analyze midrange (250-4000 Hz)
        if let Some(mid_energy) = features.mid_energy {
            balance.mid_level = mid_energy;
            balance.mid_correction = if mid_energy < 0.3 {
                2.0
            } else if mid_energy > 0.5 {
                -1.5
            } else {
                0.0
            };
        }

        // Analyze treble (4000-20000 Hz)
        if let Some(treble_energy) = features.treble_energy {
            balance.treble_level = treble_energy;
            balance.treble_correction = if treble_energy < 0.15 {
                2.5
            } else if treble_energy > 0.35 {
                -2.0
            } else {
                0.0
            };
        }

        // Check for frequency holes or peaks
        balance.identify_problems(features)?;

        Ok(balance)
    }

    /// Calculate corrections for frequency masking
    fn calculate_masking_correction(&self, features: &AudioFeatures) -> Result<MaskingCorrection> {
        let mut correction = MaskingCorrection::default();

        // Detect masked frequencies using critical bands
        if let Some(spectrum) = &features.spectrum {
            for (i, &magnitude) in spectrum.iter().enumerate() {
                let freq = i as f32 * 22050.0 / spectrum.len() as f32;

                // Check for masking in adjacent critical bands
                let masking_threshold = self.calculate_masking_threshold(spectrum, i);

                if magnitude < masking_threshold * 0.5 {
                    correction.masked_frequencies.push(freq);
                    correction.boost_amounts.push((masking_threshold - magnitude).min(6.0));
                }
            }
        }

        Ok(correction)
    }

    /// Calculate masking threshold for a frequency bin
    fn calculate_masking_threshold(&self, spectrum: &[f32], bin: usize) -> f32 {
        let mut threshold = 0.0;

        // Simple masking model - check neighboring bins
        for offset in 1..=5 {
            if bin >= offset {
                threshold += spectrum[bin - offset] * (0.5 / offset as f32);
            }
            if bin + offset < spectrum.len() {
                threshold += spectrum[bin + offset] * (0.5 / offset as f32);
            }
        }

        threshold
    }

    /// Generate optimized EQ bands
    fn generate_eq_bands(
        &self,
        genre: &GenreProfile,
        balance: &FrequencyBalance,
        masking: &MaskingCorrection,
        features: &AudioFeatures,
    ) -> Result<Vec<EQBand>> {
        let mut bands = Vec::new();

        // Standard 8-band EQ frequencies
        let frequencies = vec![60.0, 150.0, 350.0, 1000.0, 3500.0, 8000.0, 12000.0, 16000.0];

        for (i, &freq) in frequencies.iter().enumerate() {
            let mut gain = 0.0;

            // Apply genre-specific adjustments
            gain += genre.eq_adjustments[i.min(genre.eq_adjustments.len() - 1)];

            // Apply frequency balance corrections
            if freq < 250.0 {
                gain += balance.bass_correction * 0.5;
            } else if freq < 4000.0 {
                gain += balance.mid_correction * 0.5;
            } else {
                gain += balance.treble_correction * 0.5;
            }

            // Apply masking corrections
            for (j, &masked_freq) in masking.masked_frequencies.iter().enumerate() {
                if (freq - masked_freq).abs() < freq * 0.1 {
                    gain += masking.boost_amounts[j] * 0.3;
                }
            }

            // Apply problem-specific corrections
            for problem in &balance.problems {
                if problem.frequency_range.0 <= freq && freq <= problem.frequency_range.1 {
                    gain += problem.suggested_correction;
                }
            }

            // Limit gain to reasonable range
            gain = gain.max(-12.0).min(12.0);

            // Calculate Q factor based on frequency and correction amount
            let q = self.calculate_q_factor(freq, gain.abs());

            bands.push(EQBand {
                frequency: freq,
                gain,
                q,
            });
        }

        // Add notch filters for specific problems
        for problem in &balance.problems {
            if problem.problem_type == ProblemType::Resonance {
                let center_freq = (problem.frequency_range.0 + problem.frequency_range.1) / 2.0;
                bands.push(EQBand {
                    frequency: center_freq,
                    gain: problem.suggested_correction,
                    q: 8.0, // Narrow notch
                });
            }
        }

        Ok(bands)
    }

    /// Calculate appropriate Q factor
    fn calculate_q_factor(&self, frequency: f32, gain_magnitude: f32) -> f32 {
        // Higher Q for more precise corrections
        let base_q = if frequency < 200.0 {
            0.7 // Wider for bass
        } else if frequency < 2000.0 {
            1.0 // Medium for mids
        } else {
            0.8 // Slightly wider for treble
        };

        // Adjust Q based on gain magnitude
        base_q * (1.0 + gain_magnitude / 12.0)
    }

    /// Calculate confidence in the optimization
    fn calculate_confidence(&self, features: &AudioFeatures) -> f32 {
        let mut confidence = 0.5_f32;

        // Higher confidence with more complete features
        if features.spectral_centroid.is_some() {
            confidence += 0.1;
        }
        if features.tempo.is_some() {
            confidence += 0.1;
        }
        if features.spectrum.is_some() {
            confidence += 0.2;
        }
        if features.dynamic_range.is_some() {
            confidence += 0.1;
        }

        confidence.min(1.0_f32)
    }

    /// Initialize genre profiles with typical characteristics
    fn initialize_genre_profiles() -> Vec<GenreProfile> {
        vec![
            GenreProfile {
                name: "Rock".to_string(),
                typical_spectral_centroid: 2500.0,
                typical_tempo: 120.0,
                typical_bass_energy: 0.35,
                typical_dynamic_range: 25.0,
                eq_adjustments: vec![2.0, 1.0, 0.0, 0.5, 1.5, 2.0, 1.0, 0.5],
            },
            GenreProfile {
                name: "Electronic".to_string(),
                typical_spectral_centroid: 3000.0,
                typical_tempo: 128.0,
                typical_bass_energy: 0.45,
                typical_dynamic_range: 20.0,
                eq_adjustments: vec![4.0, 2.0, -1.0, 0.0, 1.0, 2.0, 3.0, 2.0],
            },
            GenreProfile {
                name: "Classical".to_string(),
                typical_spectral_centroid: 2000.0,
                typical_tempo: 90.0,
                typical_bass_energy: 0.25,
                typical_dynamic_range: 35.0,
                eq_adjustments: vec![0.0, 0.5, 0.0, 0.0, 0.5, 1.0, 1.5, 1.0],
            },
            GenreProfile {
                name: "Jazz".to_string(),
                typical_spectral_centroid: 2200.0,
                typical_tempo: 100.0,
                typical_bass_energy: 0.30,
                typical_dynamic_range: 30.0,
                eq_adjustments: vec![1.0, 0.5, 0.0, 0.5, 1.0, 1.5, 2.0, 1.5],
            },
            GenreProfile {
                name: "Pop".to_string(),
                typical_spectral_centroid: 2800.0,
                typical_tempo: 110.0,
                typical_bass_energy: 0.35,
                typical_dynamic_range: 18.0,
                eq_adjustments: vec![2.0, 1.5, 0.5, 1.0, 2.0, 2.5, 2.0, 1.5],
            },
        ]
    }

    /// Learn from user adjustments (for future adaptive learning)
    pub fn learn_from_adjustment(&mut self, original: &EQSettings, adjusted: &EQSettings, features: &AudioFeatures) {
        // Calculate adjustment deltas
        for (orig, adj) in original.bands.iter().zip(adjusted.bands.iter()) {
            let delta = adj.gain - orig.gain;

            // Update learning model (simplified)
            // In production, use proper ML model update
            self.learning_rate *= 0.99; // Decay learning rate
        }

        // Store user preference for this type of audio
        // This would be saved to a database in production
    }
}

/// Genre profile with typical audio characteristics
#[derive(Debug, Clone)]
struct GenreProfile {
    name: String,
    typical_spectral_centroid: f32,
    typical_tempo: f32,
    typical_bass_energy: f32,
    typical_dynamic_range: f32,
    eq_adjustments: Vec<f32>,
}

/// Frequency balance analysis
#[derive(Debug, Default)]
struct FrequencyBalance {
    bass_level: f32,
    mid_level: f32,
    treble_level: f32,
    bass_correction: f32,
    mid_correction: f32,
    treble_correction: f32,
    problems: Vec<FrequencyProblem>,
}

impl FrequencyBalance {
    fn identify_problems(&mut self, features: &AudioFeatures) -> Result<()> {
        // Check for muddy bass
        if self.bass_level > 0.5 {
            self.problems.push(FrequencyProblem {
                problem_type: ProblemType::MuddyBass,
                frequency_range: (80.0, 250.0),
                severity: (self.bass_level - 0.5) * 2.0,
                suggested_correction: -2.0,
            });
        }

        // Check for harsh mids
        if self.mid_level > 0.6 {
            self.problems.push(FrequencyProblem {
                problem_type: ProblemType::HarshMids,
                frequency_range: (2000.0, 4000.0),
                severity: (self.mid_level - 0.6) * 2.5,
                suggested_correction: -2.5,
            });
        }

        // Check for lack of presence
        if self.treble_level < 0.1 {
            self.problems.push(FrequencyProblem {
                problem_type: ProblemType::LackOfPresence,
                frequency_range: (5000.0, 10000.0),
                severity: (0.1 - self.treble_level) * 3.0,
                suggested_correction: 3.0,
            });
        }

        Ok(())
    }
}

/// Frequency problem detection
#[derive(Debug)]
struct FrequencyProblem {
    problem_type: ProblemType,
    frequency_range: (f32, f32),
    severity: f32,
    suggested_correction: f32,
}

#[derive(Debug, PartialEq)]
enum ProblemType {
    MuddyBass,
    HarshMids,
    LackOfPresence,
    Resonance,
    Sibilance,
}

/// Masking correction data
#[derive(Debug, Default)]
struct MaskingCorrection {
    masked_frequencies: Vec<f32>,
    boost_amounts: Vec<f32>,
}

/// Target curve for EQ optimization
#[derive(Debug)]
struct TargetCurve {
    curve_type: CurveType,
    points: Vec<(f32, f32)>, // (frequency, target_level)
}

impl Default for TargetCurve {
    fn default() -> Self {
        Self {
            curve_type: CurveType::Flat,
            points: vec![
                (20.0, 0.0),
                (100.0, 0.0),
                (1000.0, 0.0),
                (10000.0, 0.0),
                (20000.0, 0.0),
            ],
        }
    }
}

#[derive(Debug)]
enum CurveType {
    Flat,
    HarmanTarget,
    VShape,
    Custom,
}