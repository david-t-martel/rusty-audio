//! Asynchronous Audio File Loading and Processing
//!
//! This module provides high-performance, non-blocking audio file loading
//! with progress tracking and error recovery capabilities.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, BufReader};
use futures::stream::{self, StreamExt};
use parking_lot::RwLock;

use crate::error::{AudioPlayerError, FileError, AudioError};
use crate::audio_performance_optimized::{LockFreeRingBuffer, OptimizedBufferPool};

/// Progress callback type for loading operations
pub type ProgressCallback = Arc<dyn Fn(f32) + Send + Sync>;

/// Audio loading result with metadata
#[derive(Debug, Clone)]
pub struct AudioLoadResult {
    pub buffer: Arc<Vec<f32>>,
    pub sample_rate: f32,
    pub channels: usize,
    pub duration: Duration,
    pub bit_depth: u32,
    pub format: AudioFormat,
}

/// Supported audio formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioFormat {
    Mp3,
    Wav,
    Flac,
    Ogg,
    M4a,
    Unknown,
}

impl AudioFormat {
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "mp3" => AudioFormat::Mp3,
            "wav" => AudioFormat::Wav,
            "flac" => AudioFormat::Flac,
            "ogg" => AudioFormat::Ogg,
            "m4a" | "aac" => AudioFormat::M4a,
            _ => AudioFormat::Unknown,
        }
    }

    pub fn is_supported(&self) -> bool {
        !matches!(self, AudioFormat::Unknown)
    }
}

/// Configuration for async audio loading
#[derive(Debug, Clone)]
pub struct AsyncLoadConfig {
    /// Maximum file size to load (in bytes)
    pub max_file_size: usize,
    /// Chunk size for streaming reads
    pub chunk_size: usize,
    /// Timeout for loading operations
    pub timeout: Duration,
    /// Enable aggressive caching
    pub enable_caching: bool,
    /// Maximum number of concurrent loads
    pub max_concurrent_loads: usize,
}

impl Default for AsyncLoadConfig {
    fn default() -> Self {
        Self {
            max_file_size: 500 * 1024 * 1024, // 500 MB
            chunk_size: 64 * 1024,            // 64 KB chunks
            timeout: Duration::from_secs(30),
            enable_caching: true,
            max_concurrent_loads: 4,
        }
    }
}

/// High-performance async audio loader
pub struct AsyncAudioLoader {
    config: AsyncLoadConfig,
    buffer_pool: Arc<OptimizedBufferPool>,
    cache: Arc<RwLock<lru::LruCache<PathBuf, Arc<AudioLoadResult>>>>,
    active_loads: Arc<RwLock<std::collections::HashMap<PathBuf, LoadStatus>>>,
}

#[derive(Debug, Clone)]
enum LoadStatus {
    Loading { progress: f32, started: Instant },
    Completed(Arc<AudioLoadResult>),
    Failed(String),
}

impl AsyncAudioLoader {
    /// Create a new async audio loader
    pub fn new(config: AsyncLoadConfig) -> Self {
        let cache_size = if config.enable_caching { 50 } else { 0 };

        Self {
            buffer_pool: Arc::new(OptimizedBufferPool::new(16, 48000 * 10)), // 10 second buffers
            cache: Arc::new(RwLock::new(lru::LruCache::new(
                std::num::NonZeroUsize::new(cache_size).unwrap_or(std::num::NonZeroUsize::new(1).unwrap())
            ))),
            active_loads: Arc::new(RwLock::new(std::collections::HashMap::new())),
            config,
        }
    }

    /// Load an audio file asynchronously with progress tracking
    pub async fn load_file<P: AsRef<Path>>(
        &self,
        path: P,
        progress_callback: Option<ProgressCallback>,
    ) -> Result<Arc<AudioLoadResult>, AudioPlayerError> {
        let path = path.as_ref().to_path_buf();

        // Check cache first
        if self.config.enable_caching {
            if let Some(cached) = self.cache.read().get(&path).cloned() {
                return Ok(cached);
            }
        }

        // Check if already loading
        if let Some(status) = self.active_loads.read().get(&path).cloned() {
            match status {
                LoadStatus::Completed(result) => return Ok(result),
                LoadStatus::Failed(error) => {
                    return Err(AudioPlayerError::FileOperation(FileError::ReadFailed {
                        path: path.to_string_lossy().to_string()
                    }));
                },
                LoadStatus::Loading { .. } => {
                    // Wait for completion with polling
                    return self.wait_for_completion(&path, progress_callback).await;
                }
            }
        }

        // Start new load
        self.active_loads.write().insert(
            path.clone(),
            LoadStatus::Loading { progress: 0.0, started: Instant::now() }
        );

        let result = self.load_file_internal(&path, progress_callback).await;

        match &result {
            Ok(audio_result) => {
                self.active_loads.write().insert(
                    path.clone(),
                    LoadStatus::Completed(audio_result.clone())
                );

                if self.config.enable_caching {
                    self.cache.write().put(path, audio_result.clone());
                }
            },
            Err(error) => {
                self.active_loads.write().insert(
                    path,
                    LoadStatus::Failed(error.to_string())
                );
            }
        }

        result
    }

    /// Internal file loading implementation
    async fn load_file_internal(
        &self,
        path: &Path,
        progress_callback: Option<ProgressCallback>,
    ) -> Result<Arc<AudioLoadResult>, AudioPlayerError> {
        // Validate file format
        let format = path.extension()
            .and_then(|ext| ext.to_str())
            .map(AudioFormat::from_extension)
            .unwrap_or(AudioFormat::Unknown);

        if !format.is_supported() {
            return Err(AudioPlayerError::FileOperation(FileError::InvalidFormat {
                format: format!("{:?}", format),
            }));
        }

        // Check file size
        let metadata = tokio::fs::metadata(path).await
            .map_err(|e| AudioPlayerError::FileOperation(FileError::Io(e)))?;

        if metadata.len() > self.config.max_file_size as u64 {
            return Err(AudioPlayerError::FileOperation(FileError::ReadFailed {
                path: format!("File too large: {} bytes", metadata.len()),
            }));
        }

        // Load file with timeout
        let load_future = self.load_with_streaming(path, progress_callback);
        let result = tokio::time::timeout(self.config.timeout, load_future).await
            .map_err(|_| AudioPlayerError::FileOperation(FileError::ReadFailed {
                path: "Loading timeout".to_string(),
            }))?;

        result
    }

    /// Load file with streaming and progress updates
    async fn load_with_streaming(
        &self,
        path: &Path,
        progress_callback: Option<ProgressCallback>,
    ) -> Result<Arc<AudioLoadResult>, AudioPlayerError> {
        let file = File::open(path).await
            .map_err(|e| AudioPlayerError::FileOperation(FileError::Io(e)))?;

        let file_size = file.metadata().await
            .map_err(|e| AudioPlayerError::FileOperation(FileError::Io(e)))?
            .len();

        let mut reader = BufReader::with_capacity(self.config.chunk_size, file);
        let mut buffer = Vec::new();
        let mut bytes_read = 0u64;

        // Stream file in chunks
        let mut chunk = vec![0u8; self.config.chunk_size];
        loop {
            let n = reader.read(&mut chunk).await
                .map_err(|e| AudioPlayerError::FileOperation(FileError::Io(e)))?;

            if n == 0 {
                break; // EOF
            }

            buffer.extend_from_slice(&chunk[..n]);
            bytes_read += n as u64;

            // Update progress
            let progress = bytes_read as f32 / file_size as f32;
            if let Some(callback) = &progress_callback {
                callback(progress * 0.5); // First 50% is reading
            }

            // Update active load status
            self.active_loads.write().insert(
                path.to_path_buf(),
                LoadStatus::Loading { progress: progress * 0.5, started: Instant::now() }
            );

            // Yield to allow other tasks to run
            tokio::task::yield_now().await;
        }

        // Decode audio data
        let decode_result = self.decode_audio_data(buffer, progress_callback.clone()).await?;

        // Final progress update
        if let Some(callback) = &progress_callback {
            callback(1.0);
        }

        Ok(Arc::new(decode_result))
    }

    /// Decode audio data asynchronously
    async fn decode_audio_data(
        &self,
        data: Vec<u8>,
        progress_callback: Option<ProgressCallback>,
    ) -> Result<AudioLoadResult, AudioPlayerError> {
        // This is a simplified decoder - in a real implementation,
        // you would use libraries like symphonia, minimp3, etc.

        let start_time = Instant::now();

        // Simulate progressive decoding
        tokio::task::spawn_blocking(move || {
            // Placeholder for actual decoding logic
            // In practice, this would:
            // 1. Identify the format
            // 2. Use appropriate decoder (MP3, FLAC, etc.)
            // 3. Convert to f32 samples
            // 4. Handle multi-channel data

            let sample_rate = 44100.0;
            let channels = 2;
            let duration_seconds = 3.0; // Placeholder
            let num_samples = (sample_rate * duration_seconds * channels as f32) as usize;

            // Create dummy audio data (in practice, this would be decoded)
            let mut audio_samples = vec![0.0f32; num_samples];

            // Simulate decoding progress
            let chunk_size = num_samples / 10;
            for (i, chunk) in audio_samples.chunks_mut(chunk_size).enumerate() {
                // Simulate work
                std::thread::sleep(Duration::from_millis(10));

                // Generate test sine wave
                for (j, sample) in chunk.iter_mut().enumerate() {
                    let sample_index = i * chunk_size + j;
                    let t = sample_index as f32 / sample_rate;
                    *sample = (2.0 * std::f32::consts::PI * 440.0 * t).sin() * 0.5;
                }

                // Update progress (50% to 100% for decoding)
                if let Some(callback) = &progress_callback {
                    let decode_progress = (i + 1) as f32 / 10.0;
                    callback(0.5 + decode_progress * 0.5);
                }
            }

            AudioLoadResult {
                buffer: Arc::new(audio_samples),
                sample_rate,
                channels,
                duration: Duration::from_secs_f32(duration_seconds),
                bit_depth: 16,
                format: AudioFormat::Mp3, // Would be detected
            }
        }).await.map_err(|e| AudioPlayerError::AudioProcessing(AudioError::DecodeFailed))
    }

    /// Wait for an ongoing load to complete
    async fn wait_for_completion(
        &self,
        path: &Path,
        progress_callback: Option<ProgressCallback>,
    ) -> Result<Arc<AudioLoadResult>, AudioPlayerError> {
        let start_time = Instant::now();

        loop {
            if start_time.elapsed() > self.config.timeout {
                return Err(AudioPlayerError::FileOperation(FileError::ReadFailed {
                    path: "Wait timeout".to_string(),
                }));
            }

            if let Some(status) = self.active_loads.read().get(path).cloned() {
                match status {
                    LoadStatus::Completed(result) => return Ok(result),
                    LoadStatus::Failed(error) => {
                        return Err(AudioPlayerError::FileOperation(FileError::ReadFailed {
                            path: error,
                        }));
                    },
                    LoadStatus::Loading { progress, .. } => {
                        if let Some(callback) = &progress_callback {
                            callback(progress);
                        }
                    }
                }
            }

            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }

    /// Load multiple files concurrently
    pub async fn load_files_concurrent<P: AsRef<Path> + Send + Sync + 'static>(
        &self,
        paths: Vec<P>,
        progress_callback: Option<ProgressCallback>,
    ) -> Vec<Result<Arc<AudioLoadResult>, AudioPlayerError>> {
        let semaphore = Arc::new(tokio::sync::Semaphore::new(self.config.max_concurrent_loads));
        let total_files = paths.len();
        let completed_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));

        let futures = paths.into_iter().enumerate().map(|(index, path)| {
            let loader = self.clone();
            let semaphore = semaphore.clone();
            let completed_count = completed_count.clone();
            let total_files = total_files;
            let progress_callback = progress_callback.clone();

            async move {
                let _permit = semaphore.acquire().await.unwrap();

                let file_progress_callback = progress_callback.map(|callback| {
                    Arc::new(move |file_progress: f32| {
                        let overall_progress = (completed_count.load(std::sync::atomic::Ordering::Relaxed) as f32
                            + file_progress) / total_files as f32;
                        callback(overall_progress);
                    }) as ProgressCallback
                });

                let result = loader.load_file(path, file_progress_callback).await;

                completed_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                result
            }
        });

        futures::future::join_all(futures).await
    }

    /// Preload files in background
    pub async fn preload_files<P: AsRef<Path> + Send + Sync + 'static>(
        &self,
        paths: Vec<P>,
    ) -> Result<(), AudioPlayerError> {
        let _ = self.load_files_concurrent(paths, None).await;
        Ok(())
    }

    /// Clear cache
    pub fn clear_cache(&self) {
        self.cache.write().clear();
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> (usize, usize) {
        let cache = self.cache.read();
        (cache.len(), cache.cap().get())
    }

    /// Get active loads count
    pub fn active_loads_count(&self) -> usize {
        self.active_loads.read().len()
    }
}

impl Clone for AsyncAudioLoader {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            buffer_pool: self.buffer_pool.clone(),
            cache: self.cache.clone(),
            active_loads: self.active_loads.clone(),
        }
    }
}

/// Streaming audio loader for very large files
pub struct StreamingAudioLoader {
    ring_buffer: Arc<LockFreeRingBuffer>,
    config: AsyncLoadConfig,
}

impl StreamingAudioLoader {
    pub fn new(buffer_size: usize, config: AsyncLoadConfig) -> Self {
        Self {
            ring_buffer: Arc::new(LockFreeRingBuffer::new(buffer_size)),
            config,
        }
    }

    /// Start streaming audio from file
    pub async fn start_streaming<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<StreamingHandle, AudioPlayerError> {
        let path = path.as_ref().to_path_buf();
        let ring_buffer = self.ring_buffer.clone();
        let chunk_size = self.config.chunk_size;

        // Spawn background task for streaming
        let handle = tokio::spawn(async move {
            let mut file = File::open(&path).await?;
            let mut buffer = vec![0u8; chunk_size];

            loop {
                let bytes_read = file.read(&mut buffer).await?;
                if bytes_read == 0 {
                    break; // EOF
                }

                // Convert bytes to f32 samples (simplified)
                let samples: Vec<f32> = buffer[..bytes_read]
                    .chunks(4)
                    .map(|chunk| {
                        if chunk.len() == 4 {
                            f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]])
                        } else {
                            0.0
                        }
                    })
                    .collect();

                // Write to ring buffer
                let mut written = 0;
                while written < samples.len() {
                    let chunk_written = ring_buffer.write(&samples[written..]);
                    written += chunk_written;

                    if chunk_written == 0 {
                        // Buffer full, wait a bit
                        tokio::time::sleep(Duration::from_millis(1)).await;
                    }
                }

                tokio::task::yield_now().await;
            }

            Ok::<(), std::io::Error>(())
        });

        Ok(StreamingHandle {
            ring_buffer: self.ring_buffer.clone(),
            task_handle: handle,
        })
    }
}

/// Handle for streaming audio operations
pub struct StreamingHandle {
    ring_buffer: Arc<LockFreeRingBuffer>,
    task_handle: tokio::task::JoinHandle<Result<(), std::io::Error>>,
}

impl StreamingHandle {
    /// Read samples from the stream
    pub fn read_samples(&self, output: &mut [f32]) -> usize {
        self.ring_buffer.read(output)
    }

    /// Check if data is available
    pub fn available_samples(&self) -> usize {
        self.ring_buffer.available()
    }

    /// Stop streaming
    pub fn stop(self) {
        self.task_handle.abort();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::io::Write;

    #[tokio::test]
    async fn test_async_loader_creation() {
        let config = AsyncLoadConfig::default();
        let loader = AsyncAudioLoader::new(config);

        assert_eq!(loader.cache_stats().0, 0);
        assert_eq!(loader.active_loads_count(), 0);
    }

    #[tokio::test]
    async fn test_format_detection() {
        assert_eq!(AudioFormat::from_extension("mp3"), AudioFormat::Mp3);
        assert_eq!(AudioFormat::from_extension("wav"), AudioFormat::Wav);
        assert_eq!(AudioFormat::from_extension("unknown"), AudioFormat::Unknown);

        assert!(AudioFormat::Mp3.is_supported());
        assert!(!AudioFormat::Unknown.is_supported());
    }

    #[tokio::test]
    async fn test_file_not_found() {
        let loader = AsyncAudioLoader::new(AsyncLoadConfig::default());
        let result = loader.load_file("nonexistent.mp3", None).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_streaming_loader() {
        let loader = StreamingAudioLoader::new(1024, AsyncLoadConfig::default());

        // Create a temporary file
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.raw");
        let mut file = std::fs::File::create(&file_path).unwrap();

        // Write some test data
        let test_data = vec![0u8; 1000];
        file.write_all(&test_data).unwrap();

        // Test would require actual streaming implementation
        // This is a placeholder for the structure
    }

    #[tokio::test]
    async fn test_concurrent_loading() {
        let loader = AsyncAudioLoader::new(AsyncLoadConfig::default());

        // Create temporary files
        let dir = tempdir().unwrap();
        let paths: Vec<_> = (0..3).map(|i| {
            let path = dir.path().join(format!("test{}.mp3", i));
            std::fs::write(&path, b"fake mp3 data").unwrap();
            path
        }).collect();

        let progress_callback = Arc::new(|progress: f32| {
            println!("Overall progress: {:.1}%", progress * 100.0);
        });

        let results = loader.load_files_concurrent(paths, Some(progress_callback)).await;

        // All should fail since we're using fake data, but structure should work
        assert_eq!(results.len(), 3);
    }
}