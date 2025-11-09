// Advanced audio performance optimizations

use parking_lot::RwLock;
use std::collections::VecDeque;
use std::sync::Arc;

/// Lock-free audio buffer for real-time processing
pub struct LockFreeAudioBuffer {
    buffer: Arc<Vec<std::sync::atomic::AtomicU32>>,
    write_pos: std::sync::atomic::AtomicUsize,
    read_pos: std::sync::atomic::AtomicUsize,
    size: usize,
}

impl LockFreeAudioBuffer {
    pub fn new(size: usize) -> Self {
        let mut buffer = Vec::with_capacity(size);
        for _ in 0..size {
            buffer.push(std::sync::atomic::AtomicU32::new(0));
        }

        Self {
            buffer: Arc::new(buffer),
            write_pos: std::sync::atomic::AtomicUsize::new(0),
            read_pos: std::sync::atomic::AtomicUsize::new(0),
            size,
        }
    }

    #[inline(always)]
    pub fn write_sample(&self, sample: f32) -> bool {
        let write = self.write_pos.load(std::sync::atomic::Ordering::Acquire);
        let read = self.read_pos.load(std::sync::atomic::Ordering::Acquire);

        let next_write = (write + 1) % self.size;
        if next_write == read {
            return false; // Buffer full
        }

        let bits = sample.to_bits();
        self.buffer[write].store(bits, std::sync::atomic::Ordering::Release);
        self.write_pos
            .store(next_write, std::sync::atomic::Ordering::Release);
        true
    }

    #[inline(always)]
    pub fn read_sample(&self) -> Option<f32> {
        let read = self.read_pos.load(std::sync::atomic::Ordering::Acquire);
        let write = self.write_pos.load(std::sync::atomic::Ordering::Acquire);

        if read == write {
            return None; // Buffer empty
        }

        let bits = self.buffer[read].load(std::sync::atomic::Ordering::Acquire);
        let sample = f32::from_bits(bits);

        let next_read = (read + 1) % self.size;
        self.read_pos
            .store(next_read, std::sync::atomic::Ordering::Release);

        Some(sample)
    }

    pub fn available(&self) -> usize {
        let write = self.write_pos.load(std::sync::atomic::Ordering::Acquire);
        let read = self.read_pos.load(std::sync::atomic::Ordering::Acquire);

        if write >= read {
            write - read
        } else {
            self.size - read + write
        }
    }

    pub fn capacity_remaining(&self) -> usize {
        self.size - self.available() - 1
    }
}

/// SIMD-optimized audio processing functions
#[cfg(target_arch = "x86_64")]
pub mod simd {
    use std::arch::x86_64::*;

    /// Mix two audio buffers using SIMD instructions
    #[target_feature(enable = "sse2")]
    pub unsafe fn mix_buffers_sse(output: &mut [f32], input: &[f32], gain: f32) {
        let len = output.len().min(input.len());
        let simd_len = len - (len % 4);

        let gain_vec = _mm_set1_ps(gain);

        for i in (0..simd_len).step_by(4) {
            let out = _mm_loadu_ps(output.as_ptr().add(i));
            let inp = _mm_loadu_ps(input.as_ptr().add(i));
            let mixed = _mm_add_ps(out, _mm_mul_ps(inp, gain_vec));
            _mm_storeu_ps(output.as_mut_ptr().add(i), mixed);
        }

        // Handle remaining samples
        for i in simd_len..len {
            output[i] += input[i] * gain;
        }
    }

    /// Apply gain to buffer using AVX2
    #[cfg(target_feature = "avx2")]
    #[target_feature(enable = "avx2")]
    pub unsafe fn apply_gain_avx(buffer: &mut [f32], gain: f32) {
        let len = buffer.len();
        let simd_len = len - (len % 8);

        let gain_vec = _mm256_set1_ps(gain);

        for i in (0..simd_len).step_by(8) {
            let data = _mm256_loadu_ps(buffer.as_ptr().add(i));
            let result = _mm256_mul_ps(data, gain_vec);
            _mm256_storeu_ps(buffer.as_mut_ptr().add(i), result);
        }

        // Handle remaining samples
        for i in simd_len..len {
            buffer[i] *= gain;
        }
    }

    /// Compute RMS using SIMD
    #[target_feature(enable = "sse2")]
    pub unsafe fn compute_rms_sse(buffer: &[f32]) -> f32 {
        let len = buffer.len();
        if len == 0 {
            return 0.0;
        }

        let simd_len = len - (len % 4);
        let mut sum_vec = _mm_setzero_ps();

        for i in (0..simd_len).step_by(4) {
            let samples = _mm_loadu_ps(buffer.as_ptr().add(i));
            let squared = _mm_mul_ps(samples, samples);
            sum_vec = _mm_add_ps(sum_vec, squared);
        }

        // Sum the vector elements
        let mut sum = 0.0;
        let mut result = [0.0f32; 4];
        _mm_storeu_ps(result.as_mut_ptr(), sum_vec);
        sum += result[0] + result[1] + result[2] + result[3];

        // Add remaining samples
        for i in simd_len..len {
            sum += buffer[i] * buffer[i];
        }

        (sum / len as f32).sqrt()
    }

    /// Peak detection using SIMD
    #[target_feature(enable = "sse2")]
    pub unsafe fn find_peak_sse(buffer: &[f32]) -> f32 {
        let len = buffer.len();
        if len == 0 {
            return 0.0;
        }

        let simd_len = len - (len % 4);
        let mut max_vec = _mm_setzero_ps();

        for i in (0..simd_len).step_by(4) {
            let samples = _mm_loadu_ps(buffer.as_ptr().add(i));
            let abs_samples = _mm_and_ps(samples, _mm_castsi128_ps(_mm_set1_epi32(0x7FFFFFFF)));
            max_vec = _mm_max_ps(max_vec, abs_samples);
        }

        // Find maximum in vector
        let mut result = [0.0f32; 4];
        _mm_storeu_ps(result.as_mut_ptr(), max_vec);
        let mut peak = result[0].max(result[1]).max(result[2]).max(result[3]);

        // Check remaining samples
        for i in simd_len..len {
            peak = peak.max(buffer[i].abs());
        }

        peak
    }
}

/// Multi-threaded audio file loader with caching
pub struct CachedAudioLoader {
    cache: Arc<RwLock<lru::LruCache<String, Arc<Vec<f32>>>>>,
    max_cache_size: usize,
}

impl CachedAudioLoader {
    pub fn new(max_cache_entries: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(lru::LruCache::new(
                std::num::NonZeroUsize::new(max_cache_entries).unwrap(),
            ))),
            max_cache_size: max_cache_entries,
        }
    }

    pub fn load(&self, path: &std::path::Path) -> Result<Arc<Vec<f32>>, String> {
        let path_str = path.to_string_lossy().to_string();

        // Check cache first
        {
            let mut cache = self.cache.write();
            if let Some(data) = cache.get(&path_str) {
                return Ok(Arc::clone(data));
            }
        }

        // Load from disk if not in cache
        let data = self.load_from_disk(path)?;
        let data_arc = Arc::new(data);

        // Store in cache
        {
            let mut cache = self.cache.write();
            cache.put(path_str, Arc::clone(&data_arc));
        }

        Ok(data_arc)
    }

    fn load_from_disk(&self, path: &std::path::Path) -> Result<Vec<f32>, String> {
        // Placeholder for actual loading implementation
        // Would use symphonia or similar for decoding
        Ok(Vec::new())
    }

    pub fn preload_files(&self, paths: &[std::path::PathBuf]) {
        use rayon::prelude::*;

        paths.par_iter().for_each(|path| {
            let _ = self.load(path);
        });
    }

    pub fn clear_cache(&self) {
        self.cache.write().clear();
    }

    pub fn cache_stats(&self) -> CacheStats {
        let cache = self.cache.read();
        CacheStats {
            entries: cache.len(),
            capacity: self.max_cache_size,
            hit_rate: 0.0, // Would need to track hits/misses
        }
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub entries: usize,
    pub capacity: usize,
    pub hit_rate: f32,
}

/// Optimized FFT processor with caching
pub struct OptimizedFFT {
    planner: Arc<RwLock<rustfft::FftPlanner<f32>>>,
    window: Vec<f32>,
    fft_size: usize,
    scratch: Vec<num_complex::Complex<f32>>,
}

impl OptimizedFFT {
    pub fn new(fft_size: usize) -> Self {
        let mut planner = rustfft::FftPlanner::new();

        // Pre-compute Hann window
        let window = Self::create_hann_window(fft_size);

        Self {
            planner: Arc::new(RwLock::new(planner)),
            window,
            fft_size,
            scratch: vec![num_complex::Complex::new(0.0, 0.0); fft_size],
        }
    }

    fn create_hann_window(size: usize) -> Vec<f32> {
        (0..size)
            .map(|i| {
                0.5 * (1.0 - ((2.0 * std::f32::consts::PI * i as f32) / (size - 1) as f32).cos())
            })
            .collect()
    }

    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        let len = input.len().min(self.fft_size);

        // Apply window and convert to complex
        for i in 0..len {
            self.scratch[i] = num_complex::Complex::new(input[i] * self.window[i], 0.0);
        }

        // Zero-pad if necessary
        for i in len..self.fft_size {
            self.scratch[i] = num_complex::Complex::new(0.0, 0.0);
        }

        // Perform FFT
        let mut planner = self.planner.write();
        let fft = planner.plan_fft_forward(self.fft_size);
        fft.process(&mut self.scratch);

        // Convert to magnitude spectrum
        let norm = 1.0 / (self.fft_size as f32).sqrt();
        for (i, c) in self.scratch.iter().enumerate().take(output.len()) {
            output[i] = c.norm() * norm;
        }
    }

    pub fn process_stereo(
        &mut self,
        left: &[f32],
        right: &[f32],
        output_left: &mut [f32],
        output_right: &mut [f32],
    ) {
        // Process left channel
        self.process(left, output_left);

        // Reuse scratch buffer for right channel
        let len = right.len().min(self.fft_size);
        for i in 0..len {
            self.scratch[i] = num_complex::Complex::new(right[i] * self.window[i], 0.0);
        }
        for i in len..self.fft_size {
            self.scratch[i] = num_complex::Complex::new(0.0, 0.0);
        }

        let mut planner = self.planner.write();
        let fft = planner.plan_fft_forward(self.fft_size);
        fft.process(&mut self.scratch);

        let norm = 1.0 / (self.fft_size as f32).sqrt();
        for (i, c) in self.scratch.iter().enumerate().take(output_right.len()) {
            output_right[i] = c.norm() * norm;
        }
    }
}

/// Adaptive buffer manager for variable latency requirements
pub struct AdaptiveBufferManager {
    min_buffer_size: usize,
    max_buffer_size: usize,
    current_buffer_size: std::sync::atomic::AtomicUsize,
    underrun_count: std::sync::atomic::AtomicU32,
    adjustment_threshold: u32,
}

impl AdaptiveBufferManager {
    pub fn new(min_size: usize, max_size: usize) -> Self {
        Self {
            min_buffer_size: min_size,
            max_buffer_size: max_size,
            current_buffer_size: std::sync::atomic::AtomicUsize::new(min_size),
            underrun_count: std::sync::atomic::AtomicU32::new(0),
            adjustment_threshold: 3,
        }
    }

    pub fn record_underrun(&self) {
        let count = self
            .underrun_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        if count >= self.adjustment_threshold {
            self.increase_buffer_size();
            self.underrun_count
                .store(0, std::sync::atomic::Ordering::Relaxed);
        }
    }

    pub fn record_successful_callback(&self) {
        // Gradually decrease buffer size if no underruns
        let count = self
            .underrun_count
            .load(std::sync::atomic::Ordering::Relaxed);
        if count == 0 {
            self.decrease_buffer_size();
        }
    }

    fn increase_buffer_size(&self) {
        let current = self
            .current_buffer_size
            .load(std::sync::atomic::Ordering::Relaxed);
        let new_size = (current * 2).min(self.max_buffer_size);
        self.current_buffer_size
            .store(new_size, std::sync::atomic::Ordering::Relaxed);
    }

    fn decrease_buffer_size(&self) {
        let current = self
            .current_buffer_size
            .load(std::sync::atomic::Ordering::Relaxed);
        let new_size = ((current * 3) / 4).max(self.min_buffer_size);
        self.current_buffer_size
            .store(new_size, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn get_buffer_size(&self) -> usize {
        self.current_buffer_size
            .load(std::sync::atomic::Ordering::Relaxed)
    }
}

/// Audio thread priority manager
pub struct AudioThreadPriority;

impl AudioThreadPriority {
    /// Set current thread to real-time priority
    pub fn set_realtime() -> Result<(), String> {
        #[cfg(target_os = "windows")]
        {
            use windows::Win32::Foundation::*;
            use windows::Win32::System::Threading::*;

            unsafe {
                let thread = GetCurrentThread();
                if SetThreadPriority(thread, THREAD_PRIORITY_TIME_CRITICAL).is_ok() {
                    Ok(())
                } else {
                    Err("Failed to set thread priority".to_string())
                }
            }
        }

        #[cfg(unix)]
        {
            use libc::{pthread_self, sched_param, sched_setscheduler, SCHED_FIFO};

            unsafe {
                let param = sched_param { sched_priority: 99 };

                if sched_setscheduler(0, SCHED_FIFO, &param) == 0 {
                    Ok(())
                } else {
                    Err("Failed to set thread priority".to_string())
                }
            }
        }

        #[cfg(not(any(target_os = "windows", unix)))]
        {
            Err("Thread priority not supported on this platform".to_string())
        }
    }

    /// Pin thread to specific CPU core
    pub fn pin_to_core(core_id: usize) -> Result<(), String> {
        #[cfg(target_os = "windows")]
        {
            use windows::Win32::Foundation::*;
            use windows::Win32::System::Threading::*;

            unsafe {
                let thread = GetCurrentThread();
                let mask = 1usize << core_id;
                if SetThreadAffinityMask(thread, mask) != 0 {
                    Ok(())
                } else {
                    Err("Failed to set thread affinity".to_string())
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            use libc::{cpu_set_t, pthread_self, pthread_setaffinity_np, CPU_SET, CPU_ZERO};

            unsafe {
                let mut cpuset: cpu_set_t = std::mem::zeroed();
                CPU_ZERO(&mut cpuset);
                CPU_SET(core_id, &mut cpuset);

                if pthread_setaffinity_np(pthread_self(), std::mem::size_of::<cpu_set_t>(), &cpuset)
                    == 0
                {
                    Ok(())
                } else {
                    Err("Failed to set thread affinity".to_string())
                }
            }
        }

        #[cfg(not(any(target_os = "windows", target_os = "linux")))]
        {
            Err("Thread affinity not supported on this platform".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lock_free_buffer() {
        let buffer = LockFreeAudioBuffer::new(16);

        // Write samples
        assert!(buffer.write_sample(1.0));
        assert!(buffer.write_sample(2.0));
        assert!(buffer.write_sample(3.0));

        // Read samples
        assert_eq!(buffer.read_sample(), Some(1.0));
        assert_eq!(buffer.read_sample(), Some(2.0));
        assert_eq!(buffer.read_sample(), Some(3.0));
        assert_eq!(buffer.read_sample(), None); // Buffer empty
    }

    #[test]
    fn test_adaptive_buffer() {
        let manager = AdaptiveBufferManager::new(128, 2048);

        assert_eq!(manager.get_buffer_size(), 128);

        // Simulate underruns
        for _ in 0..4 {
            manager.record_underrun();
        }

        // Buffer should have increased
        assert!(manager.get_buffer_size() > 128);
    }

    #[test]
    fn test_optimized_fft() {
        let mut fft = OptimizedFFT::new(512);
        let input = vec![0.5; 512];
        let mut output = vec![0.0; 256];

        fft.process(&input, &mut output);

        // Check that we got some output
        assert!(output.iter().any(|&x| x > 0.0));
    }
}
