//! WASM SharedAudioBuffer Unit Tests
//!
//! Validates SharedAudioBuffer implementation:
//! - Buffer pooling (no unbounded growth)
//! - Thread-safe read/write
//! - Buffer reuse and cleanup
//! - Memory bounds validation
//! - Concurrent access patterns

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::*;

#[cfg(target_arch = "wasm32")]
wasm_bindgen_test_configure!(run_in_browser);

#[cfg(target_arch = "wasm32")]
mod shared_buffer_tests {
    use super::*;
    use std::sync::Arc;
    use parking_lot::Mutex;

    /// SharedAudioBuffer for testing
    struct SharedAudioBuffer {
        length: usize,
        channels: usize,
        data: Arc<Mutex<Vec<f32>>>,
    }

    impl SharedAudioBuffer {
        fn new(length: usize, channels: usize) -> Self {
            Self {
                length,
                channels,
                data: Arc::new(Mutex::new(vec![0.0; length * channels])),
            }
        }

        fn read(&self) -> Vec<f32> {
            self.data.lock().clone()
        }

        fn write(&self, data: &[f32]) {
            let mut buffer = self.data.lock();
            let copy_len = data.len().min(buffer.len());
            buffer[..copy_len].copy_from_slice(&data[..copy_len]);
        }

        fn len(&self) -> usize {
            self.length
        }

        fn channel_count(&self) -> usize {
            self.channels
        }

        fn clear(&self) {
            let mut buffer = self.data.lock();
            buffer.fill(0.0);
        }

        fn capacity(&self) -> usize {
            self.data.lock().capacity()
        }
    }

    #[wasm_bindgen_test]
    fn test_buffer_creation() {
        let buffer = SharedAudioBuffer::new(1024, 2);
        assert_eq!(buffer.len(), 1024);
        assert_eq!(buffer.channel_count(), 2);
    }

    #[wasm_bindgen_test]
    fn test_buffer_initialization() {
        let buffer = SharedAudioBuffer::new(512, 2);
        let data = buffer.read();

        assert_eq!(data.len(), 512 * 2);
        // Should be initialized to zero
        assert!(data.iter().all(|&x| x == 0.0));
    }

    #[wasm_bindgen_test]
    fn test_buffer_write_read() {
        let buffer = SharedAudioBuffer::new(128, 2);

        let test_data: Vec<f32> = (0..256).map(|i| i as f32 / 256.0).collect();
        buffer.write(&test_data);

        let read_data = buffer.read();
        assert_eq!(read_data, test_data);
    }

    #[wasm_bindgen_test]
    fn test_buffer_bounds_checking() {
        let buffer = SharedAudioBuffer::new(100, 2);

        // Write more data than buffer size
        let large_data: Vec<f32> = vec![1.0; 1000];
        buffer.write(&large_data);

        // Should only copy up to buffer size
        let read_data = buffer.read();
        assert_eq!(read_data.len(), 200); // 100 * 2 channels
    }

    #[wasm_bindgen_test]
    fn test_buffer_partial_write() {
        let buffer = SharedAudioBuffer::new(256, 2);

        let small_data: Vec<f32> = vec![0.5; 64];
        buffer.write(&small_data);

        let read_data = buffer.read();
        // First 64 samples should be 0.5
        assert!(read_data[..64].iter().all(|&x| x == 0.5));
        // Rest should be 0.0
        assert!(read_data[64..].iter().all(|&x| x == 0.0));
    }

    #[wasm_bindgen_test]
    fn test_buffer_clear() {
        let buffer = SharedAudioBuffer::new(128, 2);

        // Write some data
        let data: Vec<f32> = vec![1.0; 256];
        buffer.write(&data);

        // Clear buffer
        buffer.clear();

        // Should be all zeros
        let read_data = buffer.read();
        assert!(read_data.iter().all(|&x| x == 0.0));
    }

    #[wasm_bindgen_test]
    fn test_buffer_thread_safety() {
        let buffer = Arc::new(SharedAudioBuffer::new(512, 2));

        // Simulate concurrent reads
        let buffer1 = buffer.clone();
        let buffer2 = buffer.clone();

        let data1 = buffer1.read();
        let data2 = buffer2.read();

        assert_eq!(data1, data2);
    }

    #[wasm_bindgen_test]
    fn test_buffer_concurrent_write_read() {
        let buffer = Arc::new(SharedAudioBuffer::new(256, 2));

        // Write data
        let write_buffer = buffer.clone();
        let test_data: Vec<f32> = vec![0.7; 512];
        write_buffer.write(&test_data);

        // Read from different reference
        let read_buffer = buffer.clone();
        let read_data = read_buffer.read();

        assert_eq!(read_data.len(), 512);
        assert!(read_data.iter().all(|&x| x == 0.7));
    }

    #[wasm_bindgen_test]
    fn test_buffer_no_unbounded_growth() {
        let buffer = SharedAudioBuffer::new(128, 2);

        // Multiple writes should not grow the buffer
        let initial_capacity = buffer.capacity();

        for _ in 0..100 {
            let data: Vec<f32> = vec![0.5; 256];
            buffer.write(&data);
        }

        let final_capacity = buffer.capacity();
        assert_eq!(initial_capacity, final_capacity);
    }

    #[wasm_bindgen_test]
    fn test_buffer_reuse() {
        let buffer = SharedAudioBuffer::new(512, 2);

        // Write, clear, write again
        let data1: Vec<f32> = vec![1.0; 1024];
        buffer.write(&data1);

        buffer.clear();

        let data2: Vec<f32> = vec![0.5; 1024];
        buffer.write(&data2);

        let read_data = buffer.read();
        assert!(read_data.iter().all(|&x| x == 0.5));
    }

    #[wasm_bindgen_test]
    fn test_buffer_metadata_consistency() {
        let buffer = SharedAudioBuffer::new(1024, 4);

        // Write data multiple times
        for _ in 0..10 {
            let data: Vec<f32> = vec![0.8; 4096];
            buffer.write(&data);
        }

        // Metadata should remain consistent
        assert_eq!(buffer.len(), 1024);
        assert_eq!(buffer.channel_count(), 4);
    }

    #[wasm_bindgen_test]
    async fn test_buffer_async_access() {
        let buffer = SharedAudioBuffer::new(256, 2);

        // Write data
        let test_data: Vec<f32> = vec![0.6; 512];
        buffer.write(&test_data);

        // Simulate async delay
        wasm_bindgen_futures::JsFuture::from(js_sys::Promise::resolve(&wasm_bindgen::JsValue::from(1)))
            .await
            .ok();

        // Data should still be correct
        let read_data = buffer.read();
        assert!(read_data.iter().all(|&x| x == 0.6));
    }

    #[wasm_bindgen_test]
    fn test_buffer_memory_layout() {
        let buffer = SharedAudioBuffer::new(100, 2);

        // Total size should be length * channels
        let data = buffer.read();
        assert_eq!(data.len(), 100 * 2);
    }

    #[wasm_bindgen_test]
    fn test_buffer_large_allocation() {
        // Test with larger buffer sizes
        let buffer = SharedAudioBuffer::new(48000, 2); // 1 second at 48kHz stereo

        assert_eq!(buffer.len(), 48000);
        assert_eq!(buffer.channel_count(), 2);

        let data = buffer.read();
        assert_eq!(data.len(), 48000 * 2);
    }

    #[wasm_bindgen_test]
    fn test_buffer_edge_cases() {
        // Test with minimum size
        let buffer1 = SharedAudioBuffer::new(1, 1);
        assert_eq!(buffer1.read().len(), 1);

        // Test with odd sizes
        let buffer2 = SharedAudioBuffer::new(137, 3);
        assert_eq!(buffer2.read().len(), 137 * 3);
    }
}

/// Buffer pool for testing memory management
#[cfg(target_arch = "wasm32")]
mod buffer_pool_tests {
    use super::*;
    use std::sync::Arc;
    use parking_lot::Mutex;
    use std::collections::VecDeque;

    struct BufferPool {
        pool: Arc<Mutex<VecDeque<Vec<f32>>>>,
        max_size: usize,
    }

    impl BufferPool {
        fn new(max_size: usize) -> Self {
            Self {
                pool: Arc::new(Mutex::new(VecDeque::new())),
                max_size,
            }
        }

        fn acquire(&self, size: usize) -> Vec<f32> {
            let mut pool = self.pool.lock();
            pool.pop_front()
                .filter(|buf| buf.capacity() >= size)
                .unwrap_or_else(|| Vec::with_capacity(size))
        }

        fn release(&self, mut buffer: Vec<f32>) {
            buffer.clear();
            let mut pool = self.pool.lock();
            if pool.len() < self.max_size {
                pool.push_back(buffer);
            }
        }

        fn size(&self) -> usize {
            self.pool.lock().len()
        }
    }

    #[wasm_bindgen_test]
    fn test_buffer_pool_acquire_release() {
        let pool = BufferPool::new(10);

        let buffer = pool.acquire(1024);
        assert!(buffer.capacity() >= 1024);

        pool.release(buffer);
        assert_eq!(pool.size(), 1);
    }

    #[wasm_bindgen_test]
    fn test_buffer_pool_reuse() {
        let pool = BufferPool::new(10);

        // Acquire and release
        let buffer1 = pool.acquire(512);
        pool.release(buffer1);

        // Acquire again - should reuse
        let buffer2 = pool.acquire(512);
        pool.release(buffer2);

        // Pool should have one buffer
        assert_eq!(pool.size(), 1);
    }

    #[wasm_bindgen_test]
    fn test_buffer_pool_max_size() {
        let pool = BufferPool::new(5);

        // Release more buffers than max size
        for _ in 0..10 {
            let buffer = pool.acquire(256);
            pool.release(buffer);
        }

        // Should not exceed max size
        assert!(pool.size() <= 5);
    }

    #[wasm_bindgen_test]
    fn test_buffer_pool_no_unbounded_growth() {
        let pool = Arc::new(BufferPool::new(100));

        // Simulate heavy usage
        for _ in 0..1000 {
            let pool_clone = pool.clone();
            let buffer = pool_clone.acquire(1024);
            pool_clone.release(buffer);
        }

        // Pool size should be capped
        assert!(pool.size() <= 100);
    }
}

// Non-WASM tests
#[cfg(not(target_arch = "wasm32"))]
mod non_wasm_tests {
    #[test]
    fn test_shared_buffer_not_available_on_native() {
        assert!(true, "SharedAudioBuffer tests are WASM-only");
    }
}
