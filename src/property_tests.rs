//! Property-Based Testing for Audio Processing
//!
//! This module provides comprehensive property-based tests using proptest
//! to verify correctness of audio processing algorithms.

use proptest::prelude::*;
use quickcheck::{Arbitrary, Gen, QuickCheck, TestResult};
use approx::assert_abs_diff_eq;
use std::sync::Arc;

use crate::audio_performance_optimized::*;
use crate::optimized_signal_generators::*;
use crate::testing::signal_generators::*;

/// Property tests for SIMD vector operations
mod simd_tests {
    use super::*;

    proptest! {
        #[test]
        fn test_simd_vector_addition_commutative(
            a in prop::collection::vec(any::<f32>(), 8..1024),
            b in prop::collection::vec(any::<f32>(), 8..1024)
        ) {
            prop_assume!(a.len() == b.len());
            prop_assume!(a.iter().all(|&x| x.is_finite()));
            prop_assume!(b.iter().all(|&x| x.is_finite()));

            let mut result1 = vec![0.0; a.len()];
            let mut result2 = vec![0.0; a.len()];

            simd_ops::add_vectors_simd(&a, &b, &mut result1);
            simd_ops::add_vectors_simd(&b, &a, &mut result2);

            for (r1, r2) in result1.iter().zip(result2.iter()) {
                assert_abs_diff_eq!(r1, r2, epsilon = 1e-6);
            }
        }

        #[test]
        fn test_simd_vector_addition_identity(
            a in prop::collection::vec(-1000.0..1000.0f32, 8..1024)
        ) {
            let zeros = vec![0.0; a.len()];
            let mut result = vec![0.0; a.len()];

            simd_ops::add_vectors_simd(&a, &zeros, &mut result);

            for (original, result_val) in a.iter().zip(result.iter()) {
                assert_abs_diff_eq!(original, result_val, epsilon = 1e-6);
            }
        }

        #[test]
        fn test_simd_scalar_multiplication_distributive(
            a in prop::collection::vec(-100.0..100.0f32, 8..1024),
            scalar1 in -10.0..10.0f32,
            scalar2 in -10.0..10.0f32
        ) {
            let mut result1 = vec![0.0; a.len()];
            let mut result2 = vec![0.0; a.len()];
            let mut temp = vec![0.0; a.len()];

            // (scalar1 + scalar2) * a
            simd_ops::mul_scalar_simd(&a, scalar1 + scalar2, &mut result1);

            // scalar1 * a + scalar2 * a
            simd_ops::mul_scalar_simd(&a, scalar1, &mut temp);
            simd_ops::mul_scalar_simd(&a, scalar2, &mut result2);
            simd_ops::add_vectors_simd(&temp, &result2, &mut result2);

            for (r1, r2) in result1.iter().zip(result2.iter()) {
                assert_abs_diff_eq!(r1, r2, epsilon = 1e-5);
            }
        }
    }

    /// QuickCheck tests for additional coverage
    #[derive(Clone, Debug)]
    struct FiniteF32(f32);

    impl Arbitrary for FiniteF32 {
        fn arbitrary(g: &mut Gen) -> Self {
            loop {
                let val = f32::arbitrary(g);
                if val.is_finite() && val.abs() < 1000.0 {
                    return FiniteF32(val);
                }
            }
        }
    }

    #[quickcheck_macros::quickcheck]
    fn qc_simd_multiplication_by_zero_is_zero(values: Vec<FiniteF32>) -> bool {
        if values.is_empty() {
            return true;
        }

        let input: Vec<f32> = values.iter().map(|x| x.0).collect();
        let mut output = vec![0.0; input.len()];

        simd_ops::mul_scalar_simd(&input, 0.0, &mut output);

        output.iter().all(|&x| x.abs() < 1e-10)
    }

    #[quickcheck_macros::quickcheck]
    fn qc_simd_multiplication_by_one_is_identity(values: Vec<FiniteF32>) -> bool {
        if values.is_empty() {
            return true;
        }

        let input: Vec<f32> = values.iter().map(|x| x.0).collect();
        let mut output = vec![0.0; input.len()];

        simd_ops::mul_scalar_simd(&input, 1.0, &mut output);

        input.iter().zip(output.iter()).all(|(a, b)| (a - b).abs() < 1e-6)
    }
}

/// Property tests for ring buffer operations
mod ring_buffer_tests {
    use super::*;

    proptest! {
        #[test]
        fn test_ring_buffer_fifo_order(
            data in prop::collection::vec(-1000.0..1000.0f32, 1..256),
            buffer_size in 256..2048usize
        ) {
            let buffer = LockFreeRingBuffer::new(buffer_size);

            // Write data
            let written = buffer.write(&data);
            prop_assert_eq!(written, data.len());

            // Read back
            let mut output = vec![0.0; data.len()];
            let read = buffer.read(&mut output);
            prop_assert_eq!(read, data.len());

            // Verify order preservation
            for (original, read_back) in data.iter().zip(output.iter()) {
                prop_assert_eq!(*original, *read_back);
            }
        }

        #[test]
        fn test_ring_buffer_wraparound(
            chunk_size in 1..100usize,
            num_chunks in 1..50usize,
            buffer_size in 128..512usize
        ) {
            let buffer = LockFreeRingBuffer::new(buffer_size);
            let mut all_written_data = Vec::new();
            let mut all_read_data = Vec::new();

            for i in 0..num_chunks {
                // Generate chunk
                let chunk: Vec<f32> = (0..chunk_size)
                    .map(|j| (i * chunk_size + j) as f32)
                    .collect();

                // Write chunk
                let mut to_write = &chunk[..];
                while !to_write.is_empty() {
                    let written = buffer.write(to_write);
                    all_written_data.extend_from_slice(&to_write[..written]);
                    to_write = &to_write[written..];

                    // Read some data back to make space
                    if buffer.available() > buffer_size / 2 {
                        let mut read_chunk = vec![0.0; chunk_size.min(buffer.available())];
                        let read = buffer.read(&mut read_chunk);
                        all_read_data.extend_from_slice(&read_chunk[..read]);
                    }
                }
            }

            // Read remaining data
            while buffer.available() > 0 {
                let mut read_chunk = vec![0.0; buffer.available()];
                let read = buffer.read(&mut read_chunk);
                all_read_data.extend_from_slice(&read_chunk[..read]);
            }

            // Verify all data was preserved in order
            prop_assert_eq!(all_written_data.len(), all_read_data.len());
            for (written, read) in all_written_data.iter().zip(all_read_data.iter()) {
                prop_assert_eq!(*written, *read);
            }
        }
    }
}

/// Property tests for audio generators
mod generator_tests {
    use super::*;

    proptest! {
        #[test]
        fn test_sine_generator_amplitude_bounds(
            frequency in 20.0..20000.0f32,
            amplitude in 0.0..2.0f32,
            sample_rate in 8000.0..96000.0f32,
            duration in 0.001..1.0f32
        ) {
            let gen = SineGenerator::new(frequency).with_amplitude(amplitude);
            let samples = gen.generate(duration, sample_rate);

            for &sample in &samples {
                prop_assert!(sample.abs() <= amplitude + 1e-6);
            }
        }

        #[test]
        fn test_sine_generator_frequency_consistency(
            frequency in 100.0..1000.0f32,
            sample_rate in 44100.0..48000.0f32
        ) {
            let gen = SineGenerator::new(frequency);
            let duration = 1.0; // 1 second
            let samples = gen.generate(duration, sample_rate);

            // Count zero crossings to estimate frequency
            let zero_crossings = samples.windows(2)
                .filter(|pair| pair[0] * pair[1] < 0.0)
                .count();

            let estimated_frequency = zero_crossings as f32 / (2.0 * duration);
            let error = (estimated_frequency - frequency).abs() / frequency;

            // Allow 5% error due to discretization
            prop_assert!(error < 0.05);
        }

        #[test]
        fn test_white_noise_generator_statistics(
            amplitude in 0.1..2.0f32,
            seed in 1u64..1000000u64
        ) {
            let gen = WhiteNoiseGenerator::new()
                .with_amplitude(amplitude)
                .with_seed(seed);

            let samples = gen.generate(1.0, 44100.0);

            // Calculate statistics
            let mean = samples.iter().sum::<f32>() / samples.len() as f32;
            let variance = samples.iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f32>() / samples.len() as f32;

            // Mean should be close to zero
            prop_assert!(mean.abs() < 0.1);

            // Variance should be reasonable for white noise
            prop_assert!(variance > 0.01);
            prop_assert!(variance < amplitude * amplitude);

            // All samples should be within bounds
            for &sample in &samples {
                prop_assert!(sample.abs() <= amplitude);
            }
        }

        #[test]
        fn test_multi_tone_generator_superposition(
            freq1 in 100.0..500.0f32,
            freq2 in 600.0..1000.0f32,
            amp1 in 0.1..0.5f32,
            amp2 in 0.1..0.5f32
        ) {
            // Generate individual tones
            let gen1 = SineGenerator::new(freq1).with_amplitude(amp1);
            let gen2 = SineGenerator::new(freq2).with_amplitude(amp2);

            let duration = 0.1;
            let sample_rate = 44100.0;

            let samples1 = gen1.generate(duration, sample_rate);
            let samples2 = gen2.generate(duration, sample_rate);

            // Generate multi-tone
            let multi_gen = MultiToneGenerator::new(vec![freq1, freq2])
                .with_amplitudes(vec![amp1, amp2]);
            let multi_samples = multi_gen.generate(duration, sample_rate);

            // Verify superposition principle
            for ((s1, s2), multi) in samples1.iter()
                .zip(samples2.iter())
                .zip(multi_samples.iter()) {
                let expected = s1 + s2;
                prop_assert!((multi - expected).abs() < 1e-5);
            }
        }
    }
}

/// Property tests for EQ processing
mod eq_tests {
    use super::*;

    proptest! {
        #[test]
        fn test_eq_processor_unity_gain(
            input in prop::collection::vec(-1.0..1.0f32, 128..1024)
        ) {
            let mut eq = OptimizedEqProcessor::new(8, 48000.0);
            eq.prepare(input.len());

            // Set all bands to 0 dB gain (unity)
            for i in 0..8 {
                let freq = 60.0 * 2.0_f32.powi(i as i32);
                eq.update_band(i, freq, 1.0, 0.0); // 0 dB gain
            }

            let mut output = vec![0.0; input.len()];
            eq.process(&input, &mut output);

            // Output should be approximately equal to input
            for (inp, out) in input.iter().zip(output.iter()) {
                prop_assert!((inp - out).abs() < 0.1); // Allow for filter transients
            }
        }

        #[test]
        fn test_eq_processor_linearity(
            input in prop::collection::vec(-0.5..0.5f32, 256..512),
            gain_db in -20.0..20.0f32
        ) {
            let mut eq1 = OptimizedEqProcessor::new(1, 48000.0);
            let mut eq2 = OptimizedEqProcessor::new(1, 48000.0);

            eq1.prepare(input.len());
            eq2.prepare(input.len());

            // Configure both EQs identically
            eq1.update_band(0, 1000.0, 1.0, gain_db);
            eq2.update_band(0, 1000.0, 1.0, gain_db);

            // Process input and 2*input
            let input2: Vec<f32> = input.iter().map(|&x| x * 2.0).collect();

            let mut output1 = vec![0.0; input.len()];
            let mut output2 = vec![0.0; input.len()];

            eq1.process(&input, &mut output1);
            eq2.process(&input2, &mut output2);

            // output2 should be approximately 2 * output1 (linearity)
            for (out1, out2) in output1.iter().zip(output2.iter()) {
                if out1.abs() > 1e-6 { // Avoid division by very small numbers
                    let ratio = out2 / out1;
                    prop_assert!((ratio - 2.0).abs() < 0.1);
                }
            }
        }
    }
}

/// Property tests for fast mathematical functions
mod math_tests {
    use super::*;

    proptest! {
        #[test]
        fn test_fast_pow10_accuracy(x in -5.0..1.0f32) {
            let fast_result = fast_pow10(x);
            let accurate_result = 10.0_f32.powf(x);

            let relative_error = (fast_result - accurate_result).abs() / accurate_result;
            prop_assert!(relative_error < 0.01); // 1% error tolerance
        }

        #[test]
        fn test_fast_pow10_monotonic(x1 in -5.0..0.9f32, x2 in -5.0..0.9f32) {
            prop_assume!(x1 < x2);

            let result1 = fast_pow10(x1);
            let result2 = fast_pow10(x2);

            // Should be monotonically increasing
            prop_assert!(result1 <= result2);
        }
    }
}

/// Property tests for buffer pool
mod pool_tests {
    use super::*;

    proptest! {
        #[test]
        fn test_buffer_pool_consistency(
            pool_size in 1..20usize,
            buffer_size in 64..2048usize,
            operations in prop::collection::vec(0..2u8, 10..100)
        ) {
            let pool = OptimizedBufferPool::new(pool_size, buffer_size);
            let mut acquired_buffers = Vec::new();

            for &op in &operations {
                match op {
                    0 => { // Acquire
                        if let Some(buffer) = pool.acquire() {
                            prop_assert_eq!(buffer.len(), buffer_size);
                            acquired_buffers.push(buffer);
                        }
                    },
                    1 => { // Release
                        if let Some(buffer) = acquired_buffers.pop() {
                            pool.release(buffer);
                        }
                    },
                    _ => unreachable!(),
                }

                // Pool should never exceed initial size
                prop_assert!(acquired_buffers.len() <= pool_size);
            }

            // Release all remaining buffers
            for buffer in acquired_buffers {
                pool.release(buffer);
            }

            let (available, capacity) = pool.stats();
            prop_assert!(available <= capacity);
        }
    }
}

/// Stress tests for concurrent operations
#[cfg(test)]
mod stress_tests {
    use super::*;
    use std::thread;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn stress_test_ring_buffer_concurrent() {
        let buffer = Arc::new(LockFreeRingBuffer::new(1024));
        let iterations = 1000;
        let write_count = Arc::new(AtomicUsize::new(0));
        let read_count = Arc::new(AtomicUsize::new(0));

        let buffer_writer = buffer.clone();
        let write_count_clone = write_count.clone();
        let writer = thread::spawn(move || {
            for i in 0..iterations {
                let data = vec![i as f32; 10];
                while buffer_writer.write(&data) < data.len() {
                    thread::yield_now();
                }
                write_count_clone.fetch_add(data.len(), Ordering::Relaxed);
            }
        });

        let buffer_reader = buffer.clone();
        let read_count_clone = read_count.clone();
        let reader = thread::spawn(move || {
            let mut total_read = 0;
            while total_read < iterations * 10 {
                let mut output = vec![0.0; 10];
                let read = buffer_reader.read(&mut output);
                total_read += read;
                read_count_clone.fetch_add(read, Ordering::Relaxed);

                if read == 0 {
                    thread::yield_now();
                }
            }
        });

        writer.join().unwrap();
        reader.join().unwrap();

        assert_eq!(write_count.load(Ordering::Relaxed), iterations * 10);
        assert_eq!(read_count.load(Ordering::Relaxed), iterations * 10);
    }

    #[test]
    fn stress_test_simd_operations() {
        const ITERATIONS: usize = 1000;
        const VECTOR_SIZE: usize = 1024;

        for _ in 0..ITERATIONS {
            let a: Vec<f32> = (0..VECTOR_SIZE).map(|i| i as f32 / 100.0).collect();
            let b: Vec<f32> = (0..VECTOR_SIZE).map(|i| (i + 1000) as f32 / 100.0).collect();
            let mut result = vec![0.0; VECTOR_SIZE];

            simd_ops::add_vectors_simd(&a, &b, &mut result);

            // Verify results
            for i in 0..VECTOR_SIZE {
                let expected = a[i] + b[i];
                assert!((result[i] - expected).abs() < 1e-6);
            }
        }
    }
}

/// Performance regression tests
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn benchmark_simd_vs_scalar() {
        const SIZE: usize = 10000;
        const ITERATIONS: usize = 1000;

        let a: Vec<f32> = (0..SIZE).map(|i| i as f32).collect();
        let b: Vec<f32> = (0..SIZE).map(|i| (i + 1) as f32).collect();
        let mut result_simd = vec![0.0; SIZE];
        let mut result_scalar = vec![0.0; SIZE];

        // SIMD version
        let start = Instant::now();
        for _ in 0..ITERATIONS {
            simd_ops::add_vectors_simd(&a, &b, &mut result_simd);
        }
        let simd_time = start.elapsed();

        // Scalar version
        let start = Instant::now();
        for _ in 0..ITERATIONS {
            for i in 0..SIZE {
                result_scalar[i] = a[i] + b[i];
            }
        }
        let scalar_time = start.elapsed();

        println!("SIMD time: {:?}, Scalar time: {:?}", simd_time, scalar_time);
        println!("Speedup: {:.2}x", scalar_time.as_nanos() as f64 / simd_time.as_nanos() as f64);

        // Results should be identical
        for (simd_val, scalar_val) in result_simd.iter().zip(result_scalar.iter()) {
            assert_abs_diff_eq!(simd_val, scalar_val, epsilon = 1e-6);
        }

        // SIMD should be faster (or at least not significantly slower)
        assert!(simd_time <= scalar_time * 2); // Allow some overhead on different architectures
    }
}

/// Utility functions for property tests
mod test_utils {
    use super::*;

    /// Generate a realistic audio signal for testing
    pub fn generate_test_signal(length: usize, sample_rate: f32) -> Vec<f32> {
        let mut signal = vec![0.0; length];
        let freq1 = 440.0; // A4
        let freq2 = 880.0; // A5

        for (i, sample) in signal.iter_mut().enumerate() {
            let t = i as f32 / sample_rate;
            *sample = 0.5 * (2.0 * std::f32::consts::PI * freq1 * t).sin()
                    + 0.3 * (2.0 * std::f32::consts::PI * freq2 * t).sin();
        }

        signal
    }

    /// Calculate RMS value of a signal
    pub fn calculate_rms(signal: &[f32]) -> f32 {
        if signal.is_empty() {
            return 0.0;
        }

        let sum_squares = signal.iter().map(|&x| x * x).sum::<f32>();
        (sum_squares / signal.len() as f32).sqrt()
    }

    /// Calculate signal-to-noise ratio
    pub fn calculate_snr(signal: &[f32], noise: &[f32]) -> f32 {
        let signal_power = calculate_rms(signal).powi(2);
        let noise_power = calculate_rms(noise).powi(2);

        if noise_power == 0.0 {
            return f32::INFINITY;
        }

        10.0 * (signal_power / noise_power).log10()
    }
}

pub use test_utils::*;