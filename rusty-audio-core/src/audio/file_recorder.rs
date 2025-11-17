//! File-based audio recording destination
//!
//! This module provides an AudioDestination that writes audio to WAV files.

use super::backend::Result;
use super::router::AudioDestination;
use std::path::{Path, PathBuf};

#[cfg(not(target_arch = "wasm32"))]
use hound::{WavSpec, WavWriter};
#[cfg(not(target_arch = "wasm32"))]
use std::fs::File;
#[cfg(not(target_arch = "wasm32"))]
use std::io::BufWriter;

/// File recorder destination
///
/// Writes audio samples to a WAV file.
#[cfg(not(target_arch = "wasm32"))]
pub struct FileRecorderDestination {
    writer: WavWriter<BufWriter<File>>,
    path: PathBuf,
    sample_rate: u32,
    channels: u16,
    samples_written: u64,
}

#[cfg(not(target_arch = "wasm32"))]
impl FileRecorderDestination {
    /// Create a new file recorder destination
    ///
    /// # Arguments
    /// * `path` - Path to the output WAV file
    /// * `sample_rate` - Sample rate
    /// * `channels` - Number of channels
    /// * `bits_per_sample` - Bits per sample (16, 24, or 32)
    ///
    /// # Returns
    /// A new file recorder that writes to the specified path
    pub fn new<P: AsRef<Path>>(
        path: P,
        sample_rate: u32,
        channels: u16,
        bits_per_sample: u16,
    ) -> Result<Self> {
        let spec = WavSpec {
            channels,
            sample_rate,
            bits_per_sample,
            sample_format: if bits_per_sample == 32 {
                hound::SampleFormat::Float
            } else {
                hound::SampleFormat::Int
            },
        };

        let writer = WavWriter::create(path.as_ref(), spec).map_err(|e| {
            super::backend::AudioBackendError::Other(anyhow::anyhow!(
                "Failed to create WAV file: {}",
                e
            ))
        })?;

        Ok(Self {
            writer,
            path: path.as_ref().to_path_buf(),
            sample_rate,
            channels,
            samples_written: 0,
        })
    }

    /// Create a new file recorder with default settings (32-bit float)
    pub fn new_f32<P: AsRef<Path>>(path: P, sample_rate: u32, channels: u16) -> Result<Self> {
        Self::new(path, sample_rate, channels, 32)
    }

    /// Create a new file recorder with 16-bit integer samples
    pub fn new_i16<P: AsRef<Path>>(path: P, sample_rate: u32, channels: u16) -> Result<Self> {
        Self::new(path, sample_rate, channels, 16)
    }

    /// Get the output file path
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Get the number of samples written
    pub fn samples_written(&self) -> u64 {
        self.samples_written
    }

    /// Get the duration of recorded audio in seconds
    pub fn duration_seconds(&self) -> f64 {
        self.samples_written as f64 / (self.sample_rate as f64 * self.channels as f64)
    }

    /// Finalize the file (automatically called on drop, but can be called manually)
    pub fn finalize(self) -> Result<()> {
        self.writer.finalize().map_err(|e| {
            super::backend::AudioBackendError::Other(anyhow::anyhow!(
                "Failed to finalize WAV file: {}",
                e
            ))
        })
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl AudioDestination for FileRecorderDestination {
    fn write_samples(&mut self, buffer: &[f32]) -> Result<()> {
        for &sample in buffer {
            self.writer.write_sample(sample).map_err(|e| {
                super::backend::AudioBackendError::Other(anyhow::anyhow!(
                    "Failed to write sample: {}",
                    e
                ))
            })?;
        }
        self.samples_written += buffer.len() as u64;
        Ok(())
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn channels(&self) -> u16 {
        self.channels
    }

    fn flush(&mut self) -> Result<()> {
        self.writer.flush().map_err(|e| {
            super::backend::AudioBackendError::Other(anyhow::anyhow!(
                "Failed to flush WAV file: {}",
                e
            ))
        })
    }
}

// Note: Drop implementation removed - hound::WavWriter automatically handles
// finalization in its own Drop implementation

// WASM stub (file I/O not supported)
#[cfg(target_arch = "wasm32")]
pub struct FileRecorderDestination;

#[cfg(target_arch = "wasm32")]
impl FileRecorderDestination {
    pub fn new<P: AsRef<Path>>(
        _path: P,
        _sample_rate: u32,
        _channels: u16,
        _bits_per_sample: u16,
    ) -> Result<Self> {
        Err(super::backend::AudioBackendError::BackendNotAvailable(
            "File recording not supported on WASM".to_string(),
        ))
    }

    pub fn new_f32<P: AsRef<Path>>(_path: P, _sample_rate: u32, _channels: u16) -> Result<Self> {
        Self::new(_path, _sample_rate, _channels, 32)
    }
}

#[cfg(target_arch = "wasm32")]
impl AudioDestination for FileRecorderDestination {
    fn write_samples(&mut self, _buffer: &[f32]) -> Result<()> {
        Ok(())
    }

    fn sample_rate(&self) -> u32 {
        44100
    }

    fn channels(&self) -> u16 {
        2
    }
}

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_file_recorder_creation() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        let recorder = FileRecorderDestination::new_f32(path, 44100, 2);
        assert!(recorder.is_ok());

        let recorder = recorder.unwrap();
        assert_eq!(recorder.sample_rate(), 44100);
        assert_eq!(recorder.channels(), 2);
        assert_eq!(recorder.samples_written(), 0);
    }

    #[test]
    fn test_file_recorder_write() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        let mut recorder = FileRecorderDestination::new_f32(path, 44100, 1).unwrap();

        // Write some samples
        let samples = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        recorder.write_samples(&samples).unwrap();

        assert_eq!(recorder.samples_written(), 5);
        assert!((recorder.duration_seconds() - (5.0 / 44100.0)).abs() < 0.0001);
    }

    #[test]
    fn test_file_recorder_finalize() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();

        {
            let mut recorder = FileRecorderDestination::new_i16(&path, 48000, 2).unwrap();
            recorder.write_samples(&[0.1, 0.2, 0.3, 0.4]).unwrap();
            recorder.finalize().unwrap();
        }

        // File should exist and be readable
        assert!(path.exists());
    }
}
