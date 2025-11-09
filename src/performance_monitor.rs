// Real-time performance monitoring and telemetry system

use parking_lot::RwLock;
use std::cmp::Ordering;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Performance metrics for audio processing
#[derive(Debug, Clone, Default)]
pub struct AudioMetrics {
    /// Audio callback processing time in microseconds
    pub callback_latency_us: f64,
    /// Buffer underruns in the last second
    pub underruns: u32,
    /// Current CPU usage percentage (0-100)
    pub cpu_usage: f32,
    /// Memory usage in MB
    pub memory_usage_mb: f32,
    /// Spectrum analysis time in microseconds
    pub spectrum_latency_us: f64,
    /// UI frame time in milliseconds
    pub frame_time_ms: f32,
    /// Number of audio dropouts
    pub dropouts: u32,
}

/// Circular buffer for performance history
pub struct MetricsHistory {
    history: VecDeque<AudioMetrics>,
    max_entries: usize,
    timestamps: VecDeque<Instant>,
}

impl MetricsHistory {
    pub fn new(max_entries: usize) -> Self {
        Self {
            history: VecDeque::with_capacity(max_entries),
            max_entries,
            timestamps: VecDeque::with_capacity(max_entries),
        }
    }

    pub fn push(&mut self, metrics: AudioMetrics) {
        if self.history.len() >= self.max_entries {
            self.history.pop_front();
            self.timestamps.pop_front();
        }
        self.history.push_back(metrics);
        self.timestamps.push_back(Instant::now());
    }

    pub fn get_average(&self, duration: Duration) -> Option<AudioMetrics> {
        let cutoff = Instant::now() - duration;
        let recent: Vec<_> = self
            .history
            .iter()
            .zip(self.timestamps.iter())
            .filter(|(_, ts)| **ts >= cutoff)
            .map(|(m, _)| m.clone())
            .collect();

        if recent.is_empty() {
            return None;
        }

        let len = recent.len() as f32;
        Some(AudioMetrics {
            callback_latency_us: recent.iter().map(|m| m.callback_latency_us).sum::<f64>()
                / len as f64,
            underruns: recent.iter().map(|m| m.underruns).sum::<u32>() / len as u32,
            cpu_usage: recent.iter().map(|m| m.cpu_usage).sum::<f32>() / len,
            memory_usage_mb: recent.iter().map(|m| m.memory_usage_mb).sum::<f32>() / len,
            spectrum_latency_us: recent.iter().map(|m| m.spectrum_latency_us).sum::<f64>()
                / len as f64,
            frame_time_ms: recent.iter().map(|m| m.frame_time_ms).sum::<f32>() / len,
            dropouts: recent.iter().map(|m| m.dropouts).sum::<u32>(),
        })
    }

    pub fn get_percentile(&self, percentile: f32) -> Option<AudioMetrics> {
        if self.history.is_empty() || percentile < 0.0 || percentile > 100.0 {
            return None;
        }

        let mut latencies: Vec<_> = self.history.iter().map(|m| m.callback_latency_us).collect();
        latencies.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));

        let idx = ((latencies.len() - 1) as f32 * (percentile / 100.0)) as usize;

        Some(AudioMetrics {
            callback_latency_us: latencies[idx],
            ..self.history.back()?.clone()
        })
    }
}

/// Main performance monitor
pub struct PerformanceMonitor {
    metrics: Arc<RwLock<AudioMetrics>>,
    history: Arc<RwLock<MetricsHistory>>,
    callback_timer: Arc<RwLock<Option<Instant>>>,
    spectrum_timer: Arc<RwLock<Option<Instant>>>,
    frame_timer: Arc<RwLock<Option<Instant>>>,
    alerts: Arc<RwLock<Vec<PerformanceAlert>>>,
}

#[derive(Debug, Clone)]
pub enum PerformanceAlert {
    HighLatency { latency_ms: f32, threshold_ms: f32 },
    BufferUnderrun { count: u32 },
    HighCpuUsage { usage: f32, threshold: f32 },
    MemoryPressure { usage_mb: f32, threshold_mb: f32 },
    FrequentDropouts { count: u32, duration_s: f32 },
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(AudioMetrics::default())),
            history: Arc::new(RwLock::new(MetricsHistory::new(1000))),
            callback_timer: Arc::new(RwLock::new(None)),
            spectrum_timer: Arc::new(RwLock::new(None)),
            frame_timer: Arc::new(RwLock::new(None)),
            alerts: Arc::new(RwLock::new(Vec::new())),
        }
    }

    // Audio callback monitoring
    pub fn start_audio_callback(&self) {
        *self.callback_timer.write() = Some(Instant::now());
    }

    pub fn end_audio_callback(&self) {
        if let Some(start) = *self.callback_timer.read() {
            let elapsed_us = start.elapsed().as_micros() as f64;
            self.metrics.write().callback_latency_us = elapsed_us;

            // Check for high latency alert
            if elapsed_us > 1000.0 {
                // > 1ms
                self.add_alert(PerformanceAlert::HighLatency {
                    latency_ms: elapsed_us as f32 / 1000.0,
                    threshold_ms: 1.0,
                });
            }
        }
    }

    // Spectrum analysis monitoring
    pub fn start_spectrum_analysis(&self) {
        *self.spectrum_timer.write() = Some(Instant::now());
    }

    pub fn end_spectrum_analysis(&self) {
        if let Some(start) = *self.spectrum_timer.read() {
            let elapsed_us = start.elapsed().as_micros() as f64;
            self.metrics.write().spectrum_latency_us = elapsed_us;
        }
    }

    // UI frame monitoring
    pub fn start_frame(&self) {
        *self.frame_timer.write() = Some(Instant::now());
    }

    pub fn end_frame(&self) {
        if let Some(start) = *self.frame_timer.read() {
            let elapsed_ms = start.elapsed().as_secs_f32() * 1000.0;
            self.metrics.write().frame_time_ms = elapsed_ms;
        }
    }

    /// Update system metrics
    ///
    /// Note: System resource metrics (memory/CPU) are disabled on WASM targets
    /// and cross-compilation environments due to platform-specific dependencies
    /// on sys-info crate which requires Windows SDK for native builds.
    pub fn update_system_metrics(&self) {
        let mut metrics = self.metrics.write();

        // Platform-specific memory monitoring (disabled - sys-info crate requires Windows SDK)
        // TODO: Re-enable when sys-info compilation issues are resolved
        // See Cargo.toml for details
        #[cfg(all(
            feature = "system-metrics",
            target_os = "windows",
            not(target_arch = "wasm32")
        ))]
        {
            // Get memory usage statistics
            // if let Ok(mem_info) = sys_info::mem_info() {
            //     let used_kb = mem_info.total - mem_info.free;
            //     metrics.memory_usage_mb = (used_kb as f32) / 1024.0;
            //
            //     // Alert on high memory pressure (>80% usage)
            //     let usage_percent = (used_kb as f32 / mem_info.total as f32) * 100.0;
            //     if usage_percent > 80.0 {
            //         self.add_alert(PerformanceAlert::MemoryPressure {
            //             usage_mb: metrics.memory_usage_mb,
            //             threshold_mb: (mem_info.total as f32 * 0.8) / 1024.0,
            //         });
            //     }
            // }
        }

        // Platform-specific CPU monitoring (disabled - sys-info crate requires Windows SDK)
        // TODO: Re-enable when sys-info compilation issues are resolved
        #[cfg(all(
            feature = "system-metrics",
            target_os = "windows",
            not(target_arch = "wasm32")
        ))]
        {
            // if let Ok(loadavg) = sys_info::loadavg() {
            //     // Normalize load average by CPU count to get percentage
            //     metrics.cpu_usage = (loadavg.one as f32) * 100.0 / num_cpus::get() as f32;
            //
            //     // Alert on sustained high CPU usage (>80%)
            //     if metrics.cpu_usage > 80.0 {
            //         self.add_alert(PerformanceAlert::HighCpuUsage {
            //             usage: metrics.cpu_usage,
            //             threshold: 80.0,
            //         });
            //     }
            // }
        }

        // Store current metrics snapshot in performance history
        self.history.write().push(metrics.clone());
    }

    pub fn record_underrun(&self) {
        self.metrics.write().underruns += 1;
        self.add_alert(PerformanceAlert::BufferUnderrun {
            count: self.metrics.read().underruns,
        });
    }

    pub fn record_dropout(&self) {
        self.metrics.write().dropouts += 1;

        // Check for frequent dropouts
        if let Some(avg) = self.history.read().get_average(Duration::from_secs(10)) {
            if avg.dropouts > 5 {
                self.add_alert(PerformanceAlert::FrequentDropouts {
                    count: avg.dropouts,
                    duration_s: 10.0,
                });
            }
        }
    }

    fn add_alert(&self, alert: PerformanceAlert) {
        let mut alerts = self.alerts.write();
        alerts.push(alert);

        // Keep only last 100 alerts
        if alerts.len() > 100 {
            let len = alerts.len();
            alerts.drain(0..len - 100);
        }
    }

    pub fn get_current_metrics(&self) -> AudioMetrics {
        self.metrics.read().clone()
    }

    pub fn get_average_metrics(&self, duration: Duration) -> Option<AudioMetrics> {
        self.history.read().get_average(duration)
    }

    pub fn get_p99_latency(&self) -> Option<f64> {
        self.history
            .read()
            .get_percentile(99.0)
            .map(|m| m.callback_latency_us)
    }

    pub fn get_recent_alerts(&self) -> Vec<PerformanceAlert> {
        self.alerts.read().clone()
    }

    pub fn clear_alerts(&self) {
        self.alerts.write().clear();
    }

    /// Generate performance report
    pub fn generate_report(&self) -> PerformanceReport {
        let current = self.get_current_metrics();
        let avg_1s = self.get_average_metrics(Duration::from_secs(1));
        let avg_10s = self.get_average_metrics(Duration::from_secs(10));
        let avg_60s = self.get_average_metrics(Duration::from_secs(60));

        PerformanceReport {
            timestamp: Instant::now(),
            current_metrics: current,
            avg_1s,
            avg_10s,
            avg_60s,
            p50_latency: self
                .history
                .read()
                .get_percentile(50.0)
                .map(|m| m.callback_latency_us),
            p95_latency: self
                .history
                .read()
                .get_percentile(95.0)
                .map(|m| m.callback_latency_us),
            p99_latency: self.get_p99_latency(),
            recent_alerts: self.get_recent_alerts(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PerformanceReport {
    pub timestamp: Instant,
    pub current_metrics: AudioMetrics,
    pub avg_1s: Option<AudioMetrics>,
    pub avg_10s: Option<AudioMetrics>,
    pub avg_60s: Option<AudioMetrics>,
    pub p50_latency: Option<f64>,
    pub p95_latency: Option<f64>,
    pub p99_latency: Option<f64>,
    pub recent_alerts: Vec<PerformanceAlert>,
}

impl PerformanceReport {
    pub fn format_summary(&self) -> String {
        let mut summary = String::new();

        summary.push_str("=== Performance Report ===\n");
        summary.push_str(&format!(
            "Current Latency: {:.2}μs\n",
            self.current_metrics.callback_latency_us
        ));
        summary.push_str(&format!(
            "CPU Usage: {:.1}%\n",
            self.current_metrics.cpu_usage
        ));
        summary.push_str(&format!(
            "Memory: {:.1}MB\n",
            self.current_metrics.memory_usage_mb
        ));
        summary.push_str(&format!(
            "Frame Time: {:.2}ms\n",
            self.current_metrics.frame_time_ms
        ));

        if let Some(p50) = self.p50_latency {
            summary.push_str(&format!("P50 Latency: {:.2}μs\n", p50));
        }
        if let Some(p95) = self.p95_latency {
            summary.push_str(&format!("P95 Latency: {:.2}μs\n", p95));
        }
        if let Some(p99) = self.p99_latency {
            summary.push_str(&format!("P99 Latency: {:.2}μs\n", p99));
        }

        if !self.recent_alerts.is_empty() {
            summary.push_str("\nRecent Alerts:\n");
            for alert in &self.recent_alerts {
                summary.push_str(&format!("  - {:?}\n", alert));
            }
        }

        summary
    }
}

/// Memory pool monitor
pub struct MemoryPoolMonitor {
    allocations: Arc<RwLock<usize>>,
    deallocations: Arc<RwLock<usize>>,
    peak_usage: Arc<RwLock<usize>>,
    current_usage: Arc<RwLock<usize>>,
}

impl MemoryPoolMonitor {
    pub fn new() -> Self {
        Self {
            allocations: Arc::new(RwLock::new(0)),
            deallocations: Arc::new(RwLock::new(0)),
            peak_usage: Arc::new(RwLock::new(0)),
            current_usage: Arc::new(RwLock::new(0)),
        }
    }

    pub fn record_allocation(&self, size: usize) {
        *self.allocations.write() += 1;
        let mut current = self.current_usage.write();
        *current += size;

        let mut peak = self.peak_usage.write();
        if *current > *peak {
            *peak = *current;
        }
    }

    pub fn record_deallocation(&self, size: usize) {
        *self.deallocations.write() += 1;
        let mut current = self.current_usage.write();
        *current = current.saturating_sub(size);
    }

    pub fn get_stats(&self) -> MemoryPoolStats {
        MemoryPoolStats {
            allocations: *self.allocations.read(),
            deallocations: *self.deallocations.read(),
            peak_usage: *self.peak_usage.read(),
            current_usage: *self.current_usage.read(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MemoryPoolStats {
    pub allocations: usize,
    pub deallocations: usize,
    pub peak_usage: usize,
    pub current_usage: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_metrics_history() {
        let mut history = MetricsHistory::new(10);

        for i in 0..15 {
            let mut metrics = AudioMetrics::default();
            metrics.callback_latency_us = i as f64 * 100.0;
            history.push(metrics);
        }

        // Should only keep last 10
        assert_eq!(history.history.len(), 10);

        // Check average calculation
        let avg = history.get_average(Duration::from_secs(1));
        assert!(avg.is_some());
        if let Some(avg_metrics) = avg {
            assert!(avg_metrics.callback_latency_us > 0.0);
        }
    }

    #[test]
    fn test_performance_monitor() {
        let monitor = PerformanceMonitor::new();

        // Test callback timing
        monitor.start_audio_callback();
        thread::sleep(Duration::from_millis(1));
        monitor.end_audio_callback();

        let metrics = monitor.get_current_metrics();
        assert!(metrics.callback_latency_us > 1000.0); // Should be > 1ms

        // Test underrun recording
        monitor.record_underrun();
        monitor.record_underrun();

        let metrics = monitor.get_current_metrics();
        assert_eq!(metrics.underruns, 2);
    }

    #[test]
    fn test_percentiles() {
        let mut history = MetricsHistory::new(100);

        // Add metrics with known latencies
        for i in 1..=100 {
            let mut metrics = AudioMetrics::default();
            metrics.callback_latency_us = i as f64 * 10.0;
            history.push(metrics);
        }

        let p50 = history.get_percentile(50.0);
        assert!(p50.is_some());
        if let Some(p50_metrics) = p50 {
            assert!((p50_metrics.callback_latency_us - 500.0).abs() < 50.0); // ~500μs
        }

        let p99 = history.get_percentile(99.0);
        assert!(p99.is_some());
        if let Some(p99_metrics) = p99 {
            assert!((p99_metrics.callback_latency_us - 990.0).abs() < 50.0); // ~990μs
        }
    }
}
