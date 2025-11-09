use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use parking_lot::{Mutex, RwLock};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

const SAMPLE_RATE: f32 = 48000.0;
const RENDER_QUANTUM_SIZE: usize = 128;
const BUFFER_SIZE: usize = 4096;

/// Real-time audio metrics
#[derive(Clone, Debug)]
struct RealtimeMetrics {
    total_processed_samples: u64,
    total_processing_time_ns: u64,
    max_processing_time_ns: u64,
    min_processing_time_ns: u64,
    dropped_samples: u64,
    xruns: u64, // Buffer underruns/overruns
}

impl RealtimeMetrics {
    fn new() -> Self {
        Self {
            total_processed_samples: 0,
            total_processing_time_ns: 0,
            max_processing_time_ns: 0,
            min_processing_time_ns: u64::MAX,
            dropped_samples: 0,
            xruns: 0,
        }
    }

    fn update(&mut self, samples: usize, processing_time_ns: u64) {
        self.total_processed_samples += samples as u64;
        self.total_processing_time_ns += processing_time_ns;
        self.max_processing_time_ns = self.max_processing_time_ns.max(processing_time_ns);
        self.min_processing_time_ns = self.min_processing_time_ns.min(processing_time_ns);
    }

    fn add_xrun(&mut self, dropped_samples: usize) {
        self.xruns += 1;
        self.dropped_samples += dropped_samples as u64;
    }

    fn average_processing_time_ns(&self) -> f64 {
        if self.total_processed_samples == 0 {
            0.0
        } else {
            self.total_processing_time_ns as f64
                / (self.total_processed_samples / RENDER_QUANTUM_SIZE as u64) as f64
        }
    }

    fn cpu_usage_percentage(&self, quantum_duration_ns: u64) -> f64 {
        let avg_processing_time = self.average_processing_time_ns();
        (avg_processing_time / quantum_duration_ns as f64) * 100.0
    }
}

/// Lock-free ring buffer for real-time audio
struct LockFreeRingBuffer {
    buffer: Vec<f32>,
    capacity: usize,
    write_pos: AtomicU64,
    read_pos: AtomicU64,
}

impl LockFreeRingBuffer {
    fn new(capacity: usize) -> Self {
        Self {
            buffer: vec![0.0; capacity],
            capacity,
            write_pos: AtomicU64::new(0),
            read_pos: AtomicU64::new(0),
        }
    }

    fn write(&self, data: &[f32]) -> usize {
        let write_pos = self.write_pos.load(Ordering::Acquire);
        let read_pos = self.read_pos.load(Ordering::Acquire);

        let available_space = self.capacity - ((write_pos - read_pos) as usize % self.capacity);
        let to_write = data.len().min(available_space);

        for i in 0..to_write {
            let pos = (write_pos as usize + i) % self.capacity;
            unsafe {
                // Safe because we control access patterns
                *self.buffer.get_unchecked_mut(pos) = data[i];
            }
        }

        self.write_pos
            .store(write_pos + to_write as u64, Ordering::Release);
        to_write
    }

    fn read(&self, data: &mut [f32]) -> usize {
        let read_pos = self.read_pos.load(Ordering::Acquire);
        let write_pos = self.write_pos.load(Ordering::Acquire);

        let available_data = (write_pos - read_pos) as usize % self.capacity;
        let to_read = data.len().min(available_data);

        for i in 0..to_read {
            let pos = (read_pos as usize + i) % self.capacity;
            data[i] = unsafe {
                // Safe because we control access patterns
                *self.buffer.get_unchecked(pos)
            };
        }

        self.read_pos
            .store(read_pos + to_read as u64, Ordering::Release);
        to_read
    }

    fn available_for_read(&self) -> usize {
        let write_pos = self.write_pos.load(Ordering::Acquire);
        let read_pos = self.read_pos.load(Ordering::Acquire);
        (write_pos - read_pos) as usize % self.capacity
    }

    fn available_for_write(&self) -> usize {
        let write_pos = self.write_pos.load(Ordering::Acquire);
        let read_pos = self.read_pos.load(Ordering::Acquire);
        self.capacity - ((write_pos - read_pos) as usize % self.capacity)
    }
}

/// Real-time audio processor
struct RealtimeAudioProcessor {
    input_buffer: LockFreeRingBuffer,
    output_buffer: LockFreeRingBuffer,
    processing_buffer: Vec<f32>,
    metrics: Arc<Mutex<RealtimeMetrics>>,
    is_running: Arc<AtomicBool>,
    gain: f32,
    filter_state: [f32; 4], // Biquad filter coefficients and state
}

impl RealtimeAudioProcessor {
    fn new() -> Self {
        Self {
            input_buffer: LockFreeRingBuffer::new(BUFFER_SIZE),
            output_buffer: LockFreeRingBuffer::new(BUFFER_SIZE),
            processing_buffer: vec![0.0; RENDER_QUANTUM_SIZE],
            metrics: Arc::new(Mutex::new(RealtimeMetrics::new())),
            is_running: Arc::new(AtomicBool::new(false)),
            gain: 1.0,
            filter_state: [0.0; 4],
        }
    }

    fn start(&self) {
        self.is_running.store(true, Ordering::Release);
    }

    fn stop(&self) {
        self.is_running.store(false, Ordering::Release);
    }

    fn process_quantum(&mut self) -> bool {
        if !self.is_running.load(Ordering::Acquire) {
            return false;
        }

        let start_time = Instant::now();

        // Check if we have enough input data
        if self.input_buffer.available_for_read() < RENDER_QUANTUM_SIZE {
            // Buffer underrun
            let mut metrics = self.metrics.lock();
            metrics.add_xrun(RENDER_QUANTUM_SIZE);
            return true;
        }

        // Check if we have enough output space
        if self.output_buffer.available_for_write() < RENDER_QUANTUM_SIZE {
            // Buffer overrun
            let mut metrics = self.metrics.lock();
            metrics.add_xrun(RENDER_QUANTUM_SIZE);
            return true;
        }

        // Read input samples
        let samples_read = self.input_buffer.read(&mut self.processing_buffer);
        if samples_read != RENDER_QUANTUM_SIZE {
            let mut metrics = self.metrics.lock();
            metrics.add_xrun(RENDER_QUANTUM_SIZE - samples_read);
            return true;
        }

        // Process audio (simple gain + basic filtering)
        for sample in &mut self.processing_buffer {
            *sample *= self.gain;

            // Simple low-pass filter
            let output = self.filter_state[0] + *sample;
            self.filter_state[0] = self.filter_state[1] + *sample;
            self.filter_state[1] = *sample;
            *sample = output * 0.5;
        }

        // Write output samples
        let samples_written = self.output_buffer.write(&self.processing_buffer);
        if samples_written != RENDER_QUANTUM_SIZE {
            let mut metrics = self.metrics.lock();
            metrics.add_xrun(RENDER_QUANTUM_SIZE - samples_written);
        }

        // Update metrics
        let processing_time = start_time.elapsed().as_nanos() as u64;
        let mut metrics = self.metrics.lock();
        metrics.update(RENDER_QUANTUM_SIZE, processing_time);

        true
    }

    fn feed_input(&mut self, samples: &[f32]) -> usize {
        self.input_buffer.write(samples)
    }

    fn get_output(&mut self, output: &mut [f32]) -> usize {
        self.output_buffer.read(output)
    }

    fn set_gain(&mut self, gain: f32) {
        self.gain = gain;
    }

    fn get_metrics(&self) -> RealtimeMetrics {
        self.metrics.lock().clone()
    }
}

/// Simulate real-time constraints
struct RealtimeConstraints {
    quantum_duration_ns: u64,
    max_processing_time_ns: u64,
    jitter_tolerance_ns: u64,
}

impl RealtimeConstraints {
    fn new(sample_rate: f32, quantum_size: usize) -> Self {
        let quantum_duration_ns =
            ((quantum_size as f64 / sample_rate as f64) * 1_000_000_000.0) as u64;

        Self {
            quantum_duration_ns,
            max_processing_time_ns: quantum_duration_ns / 2, // 50% CPU budget
            jitter_tolerance_ns: quantum_duration_ns / 10,   // 10% jitter tolerance
        }
    }

    fn check_deadline(&self, processing_time_ns: u64) -> bool {
        processing_time_ns <= self.max_processing_time_ns
    }

    fn calculate_cpu_usage(&self, processing_time_ns: u64) -> f64 {
        (processing_time_ns as f64 / self.quantum_duration_ns as f64) * 100.0
    }
}

/// Benchmark quantum processing under real-time constraints
fn bench_quantum_processing_deadlines(c: &mut Criterion) {
    let mut group = c.benchmark_group("quantum_deadlines");
    group.throughput(Throughput::Elements(RENDER_QUANTUM_SIZE as u64));

    let constraints = RealtimeConstraints::new(SAMPLE_RATE, RENDER_QUANTUM_SIZE);

    group.bench_function("simple_gain", |b| {
        let mut processor = RealtimeAudioProcessor::new();
        processor.start();

        // Fill input buffer
        let input_data: Vec<f32> = (0..BUFFER_SIZE).map(|i| (i as f32 * 0.001).sin()).collect();
        processor.feed_input(&input_data);

        b.iter(|| {
            let start = Instant::now();
            black_box(processor.process_quantum());
            let processing_time = start.elapsed().as_nanos() as u64;

            // Verify deadline compliance
            assert!(
                constraints.check_deadline(processing_time),
                "Deadline missed: {}ns > {}ns",
                processing_time,
                constraints.max_processing_time_ns
            );
        });
    });

    group.bench_function("complex_processing", |b| {
        let mut processor = RealtimeAudioProcessor::new();
        processor.start();

        // Fill input buffer
        let input_data: Vec<f32> = (0..BUFFER_SIZE).map(|i| (i as f32 * 0.001).sin()).collect();
        processor.feed_input(&input_data);

        b.iter(|| {
            let start = Instant::now();

            // Add some computational load
            for _ in 0..10 {
                black_box(processor.process_quantum());
            }

            let processing_time = start.elapsed().as_nanos() as u64;
            let cpu_usage = constraints.calculate_cpu_usage(processing_time / 10);

            // Allow higher CPU usage for complex processing
            assert!(cpu_usage < 80.0, "CPU usage too high: {:.1}%", cpu_usage);
        });
    });

    group.finish();
}

/// Benchmark lock-free data structures
fn bench_lockfree_structures(c: &mut Criterion) {
    let mut group = c.benchmark_group("lockfree_structures");

    let ring_buffer = LockFreeRingBuffer::new(BUFFER_SIZE);
    let test_data: Vec<f32> = (0..RENDER_QUANTUM_SIZE)
        .map(|i| i as f32 / RENDER_QUANTUM_SIZE as f32)
        .collect();

    group.bench_function("ringbuffer_write", |b| {
        b.iter(|| {
            let written = ring_buffer.write(black_box(&test_data));
            black_box(written);
        });
    });

    group.bench_function("ringbuffer_read", |b| {
        // Fill buffer first
        for _ in 0..10 {
            ring_buffer.write(&test_data);
        }

        b.iter(|| {
            let mut output = vec![0.0; RENDER_QUANTUM_SIZE];
            let read = ring_buffer.read(black_box(&mut output));
            black_box(read);
        });
    });

    group.bench_function("ringbuffer_concurrent", |b| {
        let ring_buffer = Arc::new(LockFreeRingBuffer::new(BUFFER_SIZE * 4));
        let stop_flag = Arc::new(AtomicBool::new(false));

        b.iter(|| {
            let rb_writer = ring_buffer.clone();
            let rb_reader = ring_buffer.clone();
            let stop_writer = stop_flag.clone();
            let stop_reader = stop_flag.clone();

            stop_flag.store(false, Ordering::Release);

            let writer_handle = thread::spawn(move || {
                let data = vec![1.0f32; RENDER_QUANTUM_SIZE];
                while !stop_writer.load(Ordering::Acquire) {
                    rb_writer.write(&data);
                    thread::yield_now();
                }
            });

            let reader_handle = thread::spawn(move || {
                let mut data = vec![0.0f32; RENDER_QUANTUM_SIZE];
                while !stop_reader.load(Ordering::Acquire) {
                    rb_reader.read(&mut data);
                    thread::yield_now();
                }
            });

            // Let them run briefly
            thread::sleep(Duration::from_millis(1));

            stop_flag.store(true, Ordering::Release);
            writer_handle.join().unwrap();
            reader_handle.join().unwrap();
        });
    });

    group.finish();
}

/// Benchmark real-time scheduling and latency
fn bench_realtime_latency(c: &mut Criterion) {
    let mut group = c.benchmark_group("realtime_latency");

    group.bench_function("input_to_output_latency", |b| {
        let mut processor = RealtimeAudioProcessor::new();
        processor.start();

        b.iter(|| {
            let start_time = Instant::now();

            // Generate test signal
            let input: Vec<f32> = (0..RENDER_QUANTUM_SIZE)
                .map(|i| if i == 0 { 1.0 } else { 0.0 }) // Impulse signal
                .collect();

            // Feed input
            let fed = processor.feed_input(black_box(&input));
            assert_eq!(fed, RENDER_QUANTUM_SIZE);

            // Process
            processor.process_quantum();

            // Get output
            let mut output = vec![0.0; RENDER_QUANTUM_SIZE];
            let received = processor.get_output(&mut output);

            let latency = start_time.elapsed();

            black_box((received, latency));
        });
    });

    group.bench_function("scheduling_jitter", |b| {
        let mut processor = RealtimeAudioProcessor::new();
        processor.start();

        let mut last_time = Instant::now();
        let mut jitter_measurements = Vec::new();

        b.iter(|| {
            let current_time = Instant::now();
            let interval = current_time.duration_since(last_time);
            jitter_measurements.push(interval.as_nanos() as u64);
            last_time = current_time;

            // Simulate processing
            processor.process_quantum();

            if jitter_measurements.len() > 100 {
                jitter_measurements.remove(0);
            }

            black_box(&jitter_measurements);
        });
    });

    group.finish();
}

/// Benchmark CPU usage under sustained load
fn bench_sustained_cpu_load(c: &mut Criterion) {
    let mut group = c.benchmark_group("sustained_load");

    group.bench_function("continuous_processing", |b| {
        let mut processor = RealtimeAudioProcessor::new();
        processor.start();

        // Fill input buffer with test data
        let input_data: Vec<f32> = (0..BUFFER_SIZE).map(|i| (i as f32 * 0.001).sin()).collect();

        for _ in 0..10 {
            processor.feed_input(&input_data);
        }

        b.iter(|| {
            // Process many quantums continuously
            for _ in 0..100 {
                black_box(processor.process_quantum());
            }

            // Check metrics
            let metrics = processor.get_metrics();
            let constraints = RealtimeConstraints::new(SAMPLE_RATE, RENDER_QUANTUM_SIZE);
            let cpu_usage = metrics.cpu_usage_percentage(constraints.quantum_duration_ns);

            // Ensure we're not exceeding CPU budget
            assert!(
                cpu_usage < 90.0,
                "CPU usage too high under sustained load: {:.1}%",
                cpu_usage
            );
        });
    });

    group.bench_function("burst_processing", |b| {
        let mut processor = RealtimeAudioProcessor::new();
        processor.start();

        let input_data: Vec<f32> = (0..BUFFER_SIZE).map(|i| (i as f32 * 0.001).sin()).collect();

        b.iter(|| {
            // Simulate burst of processing after idle period
            processor.feed_input(&input_data);

            let start = Instant::now();
            for _ in 0..50 {
                processor.process_quantum();
            }
            let burst_duration = start.elapsed();

            // Verify burst handling
            assert!(
                burst_duration.as_millis() < 100,
                "Burst processing took too long: {}ms",
                burst_duration.as_millis()
            );

            black_box(burst_duration);
        });
    });

    group.finish();
}

/// Benchmark memory allocation in real-time context
fn bench_realtime_memory(c: &mut Criterion) {
    let mut group = c.benchmark_group("realtime_memory");

    group.bench_function("zero_allocation_processing", |b| {
        let mut processor = RealtimeAudioProcessor::new();
        processor.start();

        let input_data: Vec<f32> = (0..BUFFER_SIZE).map(|i| (i as f32 * 0.001).sin()).collect();
        processor.feed_input(&input_data);

        b.iter(|| {
            // This should not allocate any memory
            black_box(processor.process_quantum());
        });
    });

    group.bench_function("pre_allocated_buffers", |b| {
        let mut input_buffer = vec![0.0f32; RENDER_QUANTUM_SIZE];
        let mut output_buffer = vec![0.0f32; RENDER_QUANTUM_SIZE];
        let mut temp_buffer = vec![0.0f32; RENDER_QUANTUM_SIZE];

        b.iter(|| {
            // Generate input
            for (i, sample) in input_buffer.iter_mut().enumerate() {
                *sample = (i as f32 * 0.001).sin();
            }

            // Process (gain and copy)
            for i in 0..RENDER_QUANTUM_SIZE {
                temp_buffer[i] = input_buffer[i] * 0.5;
            }

            // Copy to output
            output_buffer.copy_from_slice(&temp_buffer);

            black_box(&output_buffer);
        });
    });

    group.finish();
}

criterion_group! {
    name = realtime_benches;
    config = Criterion::default()
        .sample_size(30)
        .measurement_time(Duration::from_secs(15))
        .warm_up_time(Duration::from_secs(5));
    targets =
        bench_quantum_processing_deadlines,
        bench_lockfree_structures,
        bench_realtime_latency,
        bench_sustained_cpu_load,
        bench_realtime_memory
}

criterion_main!(realtime_benches);
