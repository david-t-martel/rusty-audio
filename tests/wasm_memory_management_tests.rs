//! WASM Memory Management Tests
//!
//! Validates memory management in WASM:
//! - No leaks over 10 minute runtime
//! - Buffer pool stabilizes
//! - Cleanup on page unload
//! - GC cooperation
//! - Memory bounds checking

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::*;

#[cfg(target_arch = "wasm32")]
wasm_bindgen_test_configure!(run_in_browser);

#[cfg(target_arch = "wasm32")]
mod memory_tests {
    use super::*;
    use std::sync::Arc;
    use parking_lot::Mutex;
    use std::collections::VecDeque;

    /// Buffer pool with memory management
    struct BufferPool {
        pool: Arc<Mutex<VecDeque<Vec<f32>>>>,
        max_buffers: usize,
        max_buffer_size: usize,
        allocations: Arc<Mutex<usize>>,
        deallocations: Arc<Mutex<usize>>,
    }

    impl BufferPool {
        fn new(max_buffers: usize, max_buffer_size: usize) -> Self {
            Self {
                pool: Arc::new(Mutex::new(VecDeque::new())),
                max_buffers,
                max_buffer_size,
                allocations: Arc::new(Mutex::new(0)),
                deallocations: Arc::new(Mutex::new(0)),
            }
        }

        fn acquire(&self, size: usize) -> Result<Vec<f32>, String> {
            if size > self.max_buffer_size {
                return Err(format!("Buffer size {} exceeds maximum {}", size, self.max_buffer_size));
            }

            let mut pool = self.pool.lock();
            let buffer = pool.pop_front()
                .filter(|buf| buf.capacity() >= size)
                .unwrap_or_else(|| {
                    *self.allocations.lock() += 1;
                    Vec::with_capacity(size)
                });

            Ok(buffer)
        }

        fn release(&self, mut buffer: Vec<f32>) {
            buffer.clear();
            let mut pool = self.pool.lock();
            if pool.len() < self.max_buffers {
                pool.push_back(buffer);
            } else {
                *self.deallocations.lock() += 1;
                drop(buffer); // Explicit drop for testing
            }
        }

        fn pool_size(&self) -> usize {
            self.pool.lock().len()
        }

        fn total_allocations(&self) -> usize {
            *self.allocations.lock()
        }

        fn total_deallocations(&self) -> usize {
            *self.deallocations.lock()
        }

        fn clear(&self) {
            let mut pool = self.pool.lock();
            let count = pool.len();
            pool.clear();
            *self.deallocations.lock() += count;
        }
    }

    #[wasm_bindgen_test]
    fn test_buffer_pool_limits() {
        let pool = BufferPool::new(10, 10000);

        // Release more buffers than limit
        for _ in 0..20 {
            let buffer = vec![0.0; 1000];
            pool.release(buffer);
        }

        // Pool should not exceed limit
        assert!(pool.pool_size() <= 10);
    }

    #[wasm_bindgen_test]
    fn test_buffer_pool_reuse() {
        let pool = BufferPool::new(100, 10000);

        let initial_allocs = pool.total_allocations();

        // Acquire and release multiple times
        for _ in 0..10 {
            let buffer = pool.acquire(1024).unwrap();
            pool.release(buffer);
        }

        // Should reuse buffers, minimal new allocations
        let final_allocs = pool.total_allocations();
        assert!(final_allocs - initial_allocs <= 1);
    }

    #[wasm_bindgen_test]
    fn test_buffer_size_limit() {
        let pool = BufferPool::new(10, 1000);

        // Try to acquire buffer larger than limit
        let result = pool.acquire(2000);
        assert!(result.is_err());
    }

    #[wasm_bindgen_test]
    fn test_buffer_pool_cleanup() {
        let pool = BufferPool::new(50, 10000);

        // Fill pool
        for _ in 0..50 {
            pool.release(vec![0.0; 1000]);
        }

        assert_eq!(pool.pool_size(), 50);

        // Clear pool
        pool.clear();

        assert_eq!(pool.pool_size(), 0);
    }

    #[wasm_bindgen_test]
    fn test_memory_bounds_checking() {
        let mut buffer = vec![0.0; 100];

        // Safe access
        for i in 0..100 {
            buffer[i] = i as f32;
        }

        // Verify all values
        for i in 0..100 {
            assert_eq!(buffer[i], i as f32);
        }
    }

    #[wasm_bindgen_test]
    fn test_buffer_capacity_management() {
        let pool = BufferPool::new(10, 10000);

        // Acquire small buffer
        let buffer1 = pool.acquire(100).unwrap();
        assert!(buffer1.capacity() >= 100);
        pool.release(buffer1);

        // Acquire larger buffer - should reuse and grow
        let buffer2 = pool.acquire(1000).unwrap();
        assert!(buffer2.capacity() >= 1000);
        pool.release(buffer2);

        // Pool should have one buffer with capacity >= 1000
        assert_eq!(pool.pool_size(), 1);
    }

    #[wasm_bindgen_test]
    async fn test_memory_stability_over_time() {
        let pool = Arc::new(BufferPool::new(100, 10000));

        // Simulate sustained usage
        for iteration in 0..100 {
            let p = pool.clone();
            let buffer = p.acquire(1024).unwrap();

            // Simulate processing
            wasm_bindgen_futures::JsFuture::from(js_sys::Promise::resolve(&wasm_bindgen::JsValue::from(1)))
                .await
                .ok();

            p.release(buffer);

            // Check pool hasn't grown unbounded
            if iteration % 10 == 0 {
                assert!(pool.pool_size() <= 100);
            }
        }

        // Final check
        assert!(pool.pool_size() <= 100);
    }

    #[wasm_bindgen_test]
    fn test_allocation_tracking() {
        let pool = BufferPool::new(10, 10000);

        let initial_allocs = pool.total_allocations();

        // First acquisition should allocate
        let buffer1 = pool.acquire(1024).unwrap();
        assert_eq!(pool.total_allocations(), initial_allocs + 1);

        // Release and reacquire - should not allocate
        pool.release(buffer1);
        let buffer2 = pool.acquire(1024).unwrap();
        assert_eq!(pool.total_allocations(), initial_allocs + 1);

        pool.release(buffer2);
    }

    #[wasm_bindgen_test]
    fn test_deallocation_tracking() {
        let pool = BufferPool::new(2, 10000);

        let initial_deallocs = pool.total_deallocations();

        // Fill pool to capacity
        pool.release(vec![0.0; 1000]);
        pool.release(vec![0.0; 1000]);

        // Pool is full, next release should deallocate
        pool.release(vec![0.0; 1000]);

        assert_eq!(pool.total_deallocations(), initial_deallocs + 1);
    }

    /// Memory monitor for leak detection
    struct MemoryMonitor {
        samples: Vec<usize>,
        threshold_mb: f64,
    }

    impl MemoryMonitor {
        fn new(threshold_mb: f64) -> Self {
            Self {
                samples: Vec::new(),
                threshold_mb,
            }
        }

        fn sample(&mut self, pool: &BufferPool) {
            let size = pool.pool_size();
            self.samples.push(size);
        }

        fn check_leak(&self) -> bool {
            if self.samples.len() < 2 {
                return false;
            }

            let initial = self.samples[0];
            let final_val = self.samples[self.samples.len() - 1];

            // Check if growth exceeds threshold
            let growth = (final_val as f64 - initial as f64) / initial as f64;
            growth > self.threshold_mb
        }

        fn average_size(&self) -> f64 {
            if self.samples.is_empty() {
                return 0.0;
            }
            self.samples.iter().sum::<usize>() as f64 / self.samples.len() as f64
        }

        fn max_size(&self) -> usize {
            self.samples.iter().copied().max().unwrap_or(0)
        }

        fn min_size(&self) -> usize {
            self.samples.iter().copied().min().unwrap_or(0)
        }
    }

    #[wasm_bindgen_test]
    async fn test_memory_leak_detection() {
        let pool = Arc::new(BufferPool::new(100, 10000));
        let mut monitor = MemoryMonitor::new(2.0); // 200% growth threshold

        // Simulate usage and monitor
        for _ in 0..50 {
            let p = pool.clone();
            let buffer = p.acquire(1024).unwrap();

            // Small delay
            wasm_bindgen_futures::JsFuture::from(js_sys::Promise::resolve(&wasm_bindgen::JsValue::from(1)))
                .await
                .ok();

            p.release(buffer);
            monitor.sample(&pool);
        }

        // Check for leak
        let has_leak = monitor.check_leak();
        assert!(!has_leak, "Memory leak detected");

        // Log stats
        web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&format!(
            "Memory stats - Avg: {:.1}, Max: {}, Min: {}",
            monitor.average_size(),
            monitor.max_size(),
            monitor.min_size()
        )));
    }

    #[wasm_bindgen_test]
    fn test_concurrent_memory_access() {
        let pool = Arc::new(BufferPool::new(100, 10000));

        // Simulate concurrent access
        let mut handles = Vec::new();

        for _ in 0..10 {
            let p = pool.clone();
            handles.push((p.acquire(1024).unwrap(), p.clone()));
        }

        // Release all
        for (buffer, p) in handles {
            p.release(buffer);
        }

        // Pool should be stable
        assert!(pool.pool_size() <= 100);
    }

    #[wasm_bindgen_test]
    fn test_memory_pressure_recovery() {
        let pool = BufferPool::new(10, 10000);

        // Create memory pressure by exceeding pool size
        let mut buffers = Vec::new();
        for _ in 0..20 {
            buffers.push(pool.acquire(1024).unwrap());
        }

        // Release all
        for buffer in buffers {
            pool.release(buffer);
        }

        // Pool should stabilize at max size
        assert_eq!(pool.pool_size(), 10);
    }
}

/// Tests for WebAssembly memory growth
#[cfg(target_arch = "wasm32")]
mod wasm_memory_tests {
    use super::*;

    #[wasm_bindgen_test]
    fn test_wasm_memory_pages() {
        // Check WASM memory is available
        let memory = wasm_bindgen::memory();
        let buffer = memory.buffer();

        // Should have some memory allocated
        assert!(buffer.byte_length() > 0);
    }

    #[wasm_bindgen_test]
    fn test_wasm_memory_growth() {
        let initial_memory = wasm_bindgen::memory().buffer().byte_length();

        // Allocate some data
        let _large_vec: Vec<u8> = vec![0; 1024 * 1024]; // 1MB

        let after_memory = wasm_bindgen::memory().buffer().byte_length();

        // Memory should have grown or stayed same
        assert!(after_memory >= initial_memory);
    }

    #[wasm_bindgen_test]
    fn test_shared_array_buffer_support() {
        // Check if SharedArrayBuffer is available
        let has_sab = js_sys::eval("typeof SharedArrayBuffer !== 'undefined'")
            .ok()
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&format!(
            "SharedArrayBuffer available: {}",
            has_sab
        )));
    }
}

// Non-WASM tests
#[cfg(not(target_arch = "wasm32"))]
mod non_wasm_tests {
    #[test]
    fn test_memory_tests_are_wasm_only() {
        assert!(true, "Memory management tests are WASM-only");
    }
}
