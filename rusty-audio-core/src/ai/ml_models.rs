//! ML Model Management Module
//!
//! Manages loading, caching, and inference for machine learning models
//! used throughout the AI-enhanced audio processing pipeline.

use anyhow::{anyhow, Context, Result};
use ndarray::{Array1, Array2, ArrayD};
use parking_lot::RwLock;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Central ML model manager
pub struct ModelManager {
    models: HashMap<ModelType, Arc<RwLock<Box<dyn Model>>>>,
    model_cache: ModelCache,
    config: ModelConfig,
}

impl ModelManager {
    pub fn new() -> Result<Self> {
        Ok(Self {
            models: HashMap::new(),
            model_cache: ModelCache::new(100 * 1024 * 1024), // 100MB cache
            config: ModelConfig::default(),
        })
    }

    /// Load a model by type
    pub fn load_model(&mut self, model_type: ModelType) -> Result<()> {
        let model_path = self.config.get_model_path(&model_type)?;

        // Check if already loaded
        if self.models.contains_key(&model_type) {
            return Ok(());
        }

        // Load model based on type
        let model: Box<dyn Model> = match model_type {
            ModelType::GenreClassifier => Box::new(GenreClassifierModel::load(&model_path)?),
            ModelType::NoiseDetector => Box::new(NoiseDetectorModel::load(&model_path)?),
            ModelType::QualityPredictor => Box::new(QualityPredictorModel::load(&model_path)?),
            ModelType::SpeechDetector => Box::new(SpeechDetectorModel::load(&model_path)?),
            ModelType::MoodClassifier => Box::new(MoodClassifierModel::load(&model_path)?),
        };

        self.models.insert(model_type, Arc::new(RwLock::new(model)));
        Ok(())
    }

    /// Run inference on a model
    pub fn predict(&self, model_type: ModelType, input: &ModelInput) -> Result<ModelOutput> {
        let model = self
            .models
            .get(&model_type)
            .ok_or_else(|| anyhow::anyhow!("Model {:?} not loaded", model_type))?;

        // Check cache
        let cache_key = self.compute_cache_key(&model_type, input);
        if let Some(cached) = self.model_cache.get(&cache_key) {
            return Ok(cached);
        }

        // Run inference
        let output = model.read().predict(input)?;

        // Cache result
        self.model_cache.put(cache_key, output.clone());

        Ok(output)
    }

    /// Compute cache key for model inputs
    fn compute_cache_key(&self, model_type: &ModelType, input: &ModelInput) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        format!("{:?}", model_type).hash(&mut hasher);

        // Hash input features
        match input {
            ModelInput::Features(features) => {
                for f in features.iter() {
                    f.to_bits().hash(&mut hasher);
                }
            }
            ModelInput::Spectrum(spectrum) => {
                for s in spectrum.iter() {
                    s.to_bits().hash(&mut hasher);
                }
            }
            ModelInput::Raw(raw) => {
                for r in raw.iter() {
                    r.to_bits().hash(&mut hasher);
                }
            }
        }

        hasher.finish()
    }

    /// Update model weights (for online learning)
    pub fn update_model(&mut self, model_type: ModelType, update: &ModelUpdate) -> Result<()> {
        let model = self
            .models
            .get(&model_type)
            .ok_or_else(|| anyhow::anyhow!("Model {:?} not loaded", model_type))?;

        model.write().update(update)?;

        // Clear cache after update
        self.model_cache.clear();

        Ok(())
    }
}

/// Model trait for all ML models
pub trait Model: Send + Sync {
    /// Run prediction
    fn predict(&self, input: &ModelInput) -> Result<ModelOutput>;

    /// Update model (for online learning)
    fn update(&mut self, update: &ModelUpdate) -> Result<()>;

    /// Get model info
    fn info(&self) -> ModelInfo;
}

/// Genre classifier model
struct GenreClassifierModel {
    weights: Array2<f32>,
    biases: Array1<f32>,
    labels: Vec<String>,
}

impl GenreClassifierModel {
    fn load(path: &Path) -> Result<Self> {
        // In production, load from file
        // For now, create a simple model
        Ok(Self {
            weights: Array2::zeros((13, 5)), // 13 MFCC features -> 5 genres
            biases: Array1::zeros(5),
            labels: vec![
                "Rock".to_string(),
                "Electronic".to_string(),
                "Classical".to_string(),
                "Jazz".to_string(),
                "Pop".to_string(),
            ],
        })
    }

    fn softmax(&self, logits: &Array1<f32>) -> Array1<f32> {
        let max = logits.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
        let exp_sum: f32 = logits.iter().map(|&x| (x - max).exp()).sum();
        logits.mapv(|x| (x - max).exp() / exp_sum)
    }
}

impl Model for GenreClassifierModel {
    fn predict(&self, input: &ModelInput) -> Result<ModelOutput> {
        match input {
            ModelInput::Features(features) => {
                // Simple linear model for demonstration
                let input_array = Array1::from_vec(features.clone());
                let logits = self.weights.t().dot(&input_array) + &self.biases;
                let probs = self.softmax(&logits);

                // Find best genre
                let (best_idx, &best_prob) = probs
                    .iter()
                    .enumerate()
                    .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(Ordering::Equal))
                    .ok_or_else(|| anyhow!("No probabilities produced by genre classifier"))?;

                Ok(ModelOutput::Classification {
                    label: self.labels[best_idx].clone(),
                    confidence: best_prob,
                    probabilities: probs.to_vec(),
                })
            }
            _ => Err(anyhow::anyhow!("Invalid input type for genre classifier")),
        }
    }

    fn update(&mut self, update: &ModelUpdate) -> Result<()> {
        // Implement online learning (e.g., gradient descent)
        Ok(())
    }

    fn info(&self) -> ModelInfo {
        ModelInfo {
            name: "GenreClassifier".to_string(),
            version: "1.0.0".to_string(),
            input_shape: vec![13],
            output_shape: vec![5],
            parameters: (self.weights.len() + self.biases.len()) as u64,
        }
    }
}

/// Noise detector model
struct NoiseDetectorModel {
    threshold: f32,
}

impl NoiseDetectorModel {
    fn load(_path: &Path) -> Result<Self> {
        Ok(Self { threshold: 0.5 })
    }
}

impl Model for NoiseDetectorModel {
    fn predict(&self, input: &ModelInput) -> Result<ModelOutput> {
        match input {
            ModelInput::Spectrum(spectrum) => {
                // Simple spectral flatness-based noise detection
                let geometric_mean = spectrum
                    .iter()
                    .filter(|&&x| x > 0.0)
                    .map(|x| x.ln())
                    .sum::<f32>()
                    .exp()
                    .powf(1.0 / spectrum.len() as f32);

                let arithmetic_mean = spectrum.iter().sum::<f32>() / spectrum.len() as f32;

                let flatness = if arithmetic_mean > 0.0 {
                    geometric_mean / arithmetic_mean
                } else {
                    0.0
                };

                Ok(ModelOutput::Detection {
                    detected: flatness > self.threshold,
                    confidence: flatness,
                })
            }
            _ => Err(anyhow::anyhow!("Invalid input type for noise detector")),
        }
    }

    fn update(&mut self, _update: &ModelUpdate) -> Result<()> {
        Ok(())
    }

    fn info(&self) -> ModelInfo {
        ModelInfo {
            name: "NoiseDetector".to_string(),
            version: "1.0.0".to_string(),
            input_shape: vec![1024],
            output_shape: vec![1],
            parameters: 1,
        }
    }
}

/// Quality predictor model
struct QualityPredictorModel {
    feature_weights: Array1<f32>,
}

impl QualityPredictorModel {
    fn load(_path: &Path) -> Result<Self> {
        Ok(Self {
            feature_weights: Array1::from_vec(vec![0.3, 0.3, 0.2, 0.2]), // Simple weights
        })
    }
}

impl Model for QualityPredictorModel {
    fn predict(&self, input: &ModelInput) -> Result<ModelOutput> {
        match input {
            ModelInput::Features(features) => {
                // Simple weighted sum
                let score = features
                    .iter()
                    .zip(self.feature_weights.iter())
                    .map(|(f, w)| f * w)
                    .sum::<f32>()
                    .min(1.0)
                    .max(0.0);

                Ok(ModelOutput::Regression {
                    value: score,
                    confidence: 0.8,
                })
            }
            _ => Err(anyhow::anyhow!("Invalid input type for quality predictor")),
        }
    }

    fn update(&mut self, _update: &ModelUpdate) -> Result<()> {
        Ok(())
    }

    fn info(&self) -> ModelInfo {
        ModelInfo {
            name: "QualityPredictor".to_string(),
            version: "1.0.0".to_string(),
            input_shape: vec![4],
            output_shape: vec![1],
            parameters: 4,
        }
    }
}

/// Speech detector model
struct SpeechDetectorModel {
    vad_threshold: f32,
}

impl SpeechDetectorModel {
    fn load(_path: &Path) -> Result<Self> {
        Ok(Self { vad_threshold: 0.3 })
    }
}

impl Model for SpeechDetectorModel {
    fn predict(&self, input: &ModelInput) -> Result<ModelOutput> {
        match input {
            ModelInput::Features(features) => {
                // Simple VAD based on energy and ZCR
                let energy = features.get(0).copied().unwrap_or(0.0);
                let zcr = features.get(1).copied().unwrap_or(0.0);

                let speech_score = energy * 0.7 + (1.0 - zcr) * 0.3;

                Ok(ModelOutput::Detection {
                    detected: speech_score > self.vad_threshold,
                    confidence: speech_score,
                })
            }
            _ => Err(anyhow::anyhow!("Invalid input type for speech detector")),
        }
    }

    fn update(&mut self, _update: &ModelUpdate) -> Result<()> {
        Ok(())
    }

    fn info(&self) -> ModelInfo {
        ModelInfo {
            name: "SpeechDetector".to_string(),
            version: "1.0.0".to_string(),
            input_shape: vec![2],
            output_shape: vec![1],
            parameters: 1,
        }
    }
}

/// Mood classifier model
struct MoodClassifierModel {
    moods: Vec<String>,
}

impl MoodClassifierModel {
    fn load(_path: &Path) -> Result<Self> {
        Ok(Self {
            moods: vec![
                "Happy".to_string(),
                "Sad".to_string(),
                "Energetic".to_string(),
                "Calm".to_string(),
                "Aggressive".to_string(),
            ],
        })
    }
}

impl Model for MoodClassifierModel {
    fn predict(&self, input: &ModelInput) -> Result<ModelOutput> {
        match input {
            ModelInput::Features(features) => {
                // Simple mood detection based on energy and tempo
                let energy = features.get(0).copied().unwrap_or(0.5);
                let tempo = features.get(1).copied().unwrap_or(100.0) / 200.0;

                let mood = if energy > 0.7 && tempo > 0.6 {
                    "Energetic"
                } else if energy < 0.3 && tempo < 0.4 {
                    "Calm"
                } else if energy > 0.6 {
                    "Happy"
                } else {
                    "Sad"
                };

                Ok(ModelOutput::Classification {
                    label: mood.to_string(),
                    confidence: 0.7,
                    probabilities: vec![0.2, 0.2, 0.2, 0.2, 0.2],
                })
            }
            _ => Err(anyhow::anyhow!("Invalid input type for mood classifier")),
        }
    }

    fn update(&mut self, _update: &ModelUpdate) -> Result<()> {
        Ok(())
    }

    fn info(&self) -> ModelInfo {
        ModelInfo {
            name: "MoodClassifier".to_string(),
            version: "1.0.0".to_string(),
            input_shape: vec![2],
            output_shape: vec![5],
            parameters: 10,
        }
    }
}

/// Model cache for inference results
struct ModelCache {
    cache: RwLock<lru::LruCache<u64, ModelOutput>>,
    max_size: usize,
}

impl ModelCache {
    fn new(max_size: usize) -> Self {
        Self {
            cache: RwLock::new(lru::LruCache::new(
                std::num::NonZeroUsize::new(max_size.max(1)).unwrap_or(std::num::NonZeroUsize::MIN),
            )),
            max_size,
        }
    }

    fn get(&self, key: &u64) -> Option<ModelOutput> {
        self.cache.write().get(key).cloned()
    }

    fn put(&self, key: u64, value: ModelOutput) {
        self.cache.write().put(key, value);
    }

    fn clear(&self) {
        self.cache.write().clear();
    }
}

/// Model configuration
#[derive(Debug)]
struct ModelConfig {
    model_dir: PathBuf,
    auto_download: bool,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            model_dir: PathBuf::from("assets/models"),
            auto_download: false,
        }
    }
}

impl ModelConfig {
    fn get_model_path(&self, model_type: &ModelType) -> Result<PathBuf> {
        let filename = match model_type {
            ModelType::GenreClassifier => "genre_classifier.onnx",
            ModelType::NoiseDetector => "noise_detector.onnx",
            ModelType::QualityPredictor => "quality_predictor.onnx",
            ModelType::SpeechDetector => "speech_detector.onnx",
            ModelType::MoodClassifier => "mood_classifier.onnx",
        };

        Ok(self.model_dir.join(filename))
    }
}

/// Model types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModelType {
    GenreClassifier,
    NoiseDetector,
    QualityPredictor,
    SpeechDetector,
    MoodClassifier,
}

/// Model input types
#[derive(Debug, Clone)]
pub enum ModelInput {
    Features(Vec<f32>),
    Spectrum(Vec<f32>),
    Raw(Vec<f32>),
}

/// Model output types
#[derive(Debug, Clone)]
pub enum ModelOutput {
    Classification {
        label: String,
        confidence: f32,
        probabilities: Vec<f32>,
    },
    Detection {
        detected: bool,
        confidence: f32,
    },
    Regression {
        value: f32,
        confidence: f32,
    },
}

/// Model update for online learning
#[derive(Debug)]
pub struct ModelUpdate {
    pub input: ModelInput,
    pub target: ModelOutput,
    pub learning_rate: f32,
}

/// Model information
#[derive(Debug)]
pub struct ModelInfo {
    pub name: String,
    pub version: String,
    pub input_shape: Vec<usize>,
    pub output_shape: Vec<usize>,
    pub parameters: u64,
}
