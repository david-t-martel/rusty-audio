//! Security monitoring and alerting module
//!
//! Tracks security events, violations, and provides alerting capabilities
//! for security-critical incidents.

use parking_lot::RwLock;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

/// Security monitor for tracking and alerting on security events
pub struct SecurityMonitor {
    violation_count: Arc<AtomicUsize>,
    events: Arc<RwLock<VecDeque<SecurityEvent>>>,
    last_violation: Arc<RwLock<Option<Instant>>>,
    is_active: Arc<AtomicBool>,
    lockdown_mode: Arc<AtomicBool>,
    config: MonitorConfig,
}

/// Security event structure
#[derive(Debug, Clone)]
pub struct SecurityEvent {
    pub timestamp: Instant,
    pub severity: Severity,
    pub category: EventCategory,
    pub message: String,
    pub details: Option<String>,
}

/// Event severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

/// Event categories
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventCategory {
    FileAccess,
    PathTraversal,
    BufferOverflow,
    AudioSafety,
    InputValidation,
    Authentication,
    ResourceLimit,
    SystemIntegrity,
}

/// Monitor configuration
#[derive(Debug, Clone)]
pub struct MonitorConfig {
    pub max_events: usize,
    pub violation_threshold: usize,
    pub violation_window: Duration,
    pub enable_alerts: bool,
    pub enable_logging: bool,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            max_events: 1000,
            violation_threshold: 10,
            violation_window: Duration::from_secs(300), // 5 minutes
            enable_alerts: true,
            enable_logging: true,
        }
    }
}

impl SecurityMonitor {
    /// Create a new security monitor
    pub fn new() -> Self {
        Self::with_config(MonitorConfig::default())
    }

    /// Create with custom configuration
    pub fn with_config(config: MonitorConfig) -> Self {
        Self {
            violation_count: Arc::new(AtomicUsize::new(0)),
            events: Arc::new(RwLock::new(VecDeque::with_capacity(config.max_events))),
            last_violation: Arc::new(RwLock::new(None)),
            is_active: Arc::new(AtomicBool::new(true)),
            lockdown_mode: Arc::new(AtomicBool::new(false)),
            config,
        }
    }

    /// Log a security event
    pub fn log_event(&self, event: SecurityEvent) {
        if !self.is_active.load(Ordering::Acquire) {
            return;
        }

        // Log to tracing
        if self.config.enable_logging {
            match event.severity {
                Severity::Critical => {
                    error!(
                        category = ?event.category,
                        "SECURITY CRITICAL: {} - {:?}",
                        event.message,
                        event.details
                    );
                }
                Severity::High => {
                    warn!(
                        category = ?event.category,
                        "SECURITY HIGH: {} - {:?}",
                        event.message,
                        event.details
                    );
                }
                Severity::Medium => {
                    warn!(
                        category = ?event.category,
                        "SECURITY MEDIUM: {} - {:?}",
                        event.message,
                        event.details
                    );
                }
                Severity::Low => {
                    info!(
                        category = ?event.category,
                        "SECURITY LOW: {} - {:?}",
                        event.message,
                        event.details
                    );
                }
            }
        }

        // Track violations
        if event.severity >= Severity::High {
            self.track_violation();
        }

        // Store event
        let mut events = self.events.write();
        if events.len() >= self.config.max_events {
            events.pop_front();
        }
        events.push_back(event.clone());

        // Check for lockdown conditions
        if event.severity == Severity::Critical {
            self.check_lockdown_conditions();
        }

        // Trigger alerts if needed
        if self.config.enable_alerts && event.severity >= Severity::High {
            self.trigger_alert(event);
        }
    }

    /// Track a security violation
    fn track_violation(&self) {
        let count = self.violation_count.fetch_add(1, Ordering::SeqCst) + 1;
        *self.last_violation.write() = Some(Instant::now());

        debug!("Security violation #{}", count);

        // Check if we should enter lockdown
        if count >= self.config.violation_threshold {
            self.check_lockdown_conditions();
        }
    }

    /// Check if lockdown should be triggered
    fn check_lockdown_conditions(&self) {
        let violation_count = self.violation_count.load(Ordering::Acquire);

        // Check violation threshold
        if violation_count >= self.config.violation_threshold {
            // Check if violations are within the time window
            if let Some(last_violation) = *self.last_violation.read() {
                if last_violation.elapsed() < self.config.violation_window {
                    self.enter_lockdown();
                }
            }
        }
    }

    /// Enter lockdown mode
    fn enter_lockdown(&self) {
        if !self.lockdown_mode.swap(true, Ordering::SeqCst) {
            error!("ENTERING SECURITY LOCKDOWN MODE");

            // Log critical event
            self.log_event(SecurityEvent {
                timestamp: Instant::now(),
                severity: Severity::Critical,
                category: EventCategory::SystemIntegrity,
                message: "Security lockdown activated due to excessive violations".to_string(),
                details: Some(format!(
                    "Violation count: {}",
                    self.violation_count.load(Ordering::Acquire)
                )),
            });
        }
    }

    /// Exit lockdown mode
    pub fn exit_lockdown(&self) {
        if self.lockdown_mode.swap(false, Ordering::SeqCst) {
            info!("Exiting security lockdown mode");
            self.reset_violations();
        }
    }

    /// Check if in lockdown mode
    pub fn is_lockdown(&self) -> bool {
        self.lockdown_mode.load(Ordering::Acquire)
    }

    /// Reset violation counter
    pub fn reset_violations(&self) {
        self.violation_count.store(0, Ordering::Release);
        *self.last_violation.write() = None;
    }

    /// Trigger security alert
    fn trigger_alert(&self, event: SecurityEvent) {
        // In a real application, this would:
        // - Send notifications to administrators
        // - Log to security audit file
        // - Trigger incident response procedures
        // - Send metrics to monitoring system

        warn!("SECURITY ALERT: {:?} - {}", event.category, event.message);
    }

    /// Get recent events
    pub fn get_recent_events(&self, count: usize) -> Vec<SecurityEvent> {
        let events = self.events.read();
        events.iter().rev().take(count).cloned().collect()
    }

    /// Get events by severity
    pub fn get_events_by_severity(&self, min_severity: Severity) -> Vec<SecurityEvent> {
        let events = self.events.read();
        events
            .iter()
            .filter(|e| e.severity >= min_severity)
            .cloned()
            .collect()
    }

    /// Get violation count
    pub fn get_violation_count(&self) -> usize {
        self.violation_count.load(Ordering::Acquire)
    }

    /// Check if monitor is active
    pub fn is_active(&self) -> bool {
        self.is_active.load(Ordering::Acquire)
    }

    /// Activate monitor
    pub fn activate(&self) {
        self.is_active.store(true, Ordering::Release);
        info!("Security monitor activated");
    }

    /// Deactivate monitor
    pub fn deactivate(&self) {
        self.is_active.store(false, Ordering::Release);
        info!("Security monitor deactivated");
    }

    /// Get security summary
    pub fn get_summary(&self) -> SecuritySummary {
        let events = self.events.read();

        let mut critical_count = 0;
        let mut high_count = 0;
        let mut medium_count = 0;
        let mut low_count = 0;

        for event in events.iter() {
            match event.severity {
                Severity::Critical => critical_count += 1,
                Severity::High => high_count += 1,
                Severity::Medium => medium_count += 1,
                Severity::Low => low_count += 1,
            }
        }

        SecuritySummary {
            total_events: events.len(),
            critical_count,
            high_count,
            medium_count,
            low_count,
            violation_count: self.violation_count.load(Ordering::Acquire),
            is_lockdown: self.is_lockdown(),
            is_active: self.is_active(),
        }
    }

    /// Clear all events
    pub fn clear_events(&self) {
        let mut events = self.events.write();
        events.clear();
        info!("Security events cleared");
    }
}

/// Security summary statistics
#[derive(Debug, Clone)]
pub struct SecuritySummary {
    pub total_events: usize,
    pub critical_count: usize,
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
    pub violation_count: usize,
    pub is_lockdown: bool,
    pub is_active: bool,
}

/// Helper functions for creating common security events
impl SecurityMonitor {
    /// Log a file access violation
    pub fn log_file_access_violation(&self, path: &str, reason: &str) {
        self.log_event(SecurityEvent {
            timestamp: Instant::now(),
            severity: Severity::High,
            category: EventCategory::FileAccess,
            message: format!("File access violation: {}", path),
            details: Some(reason.to_string()),
        });
    }

    /// Log a path traversal attempt
    pub fn log_path_traversal(&self, attempted_path: &str) {
        self.log_event(SecurityEvent {
            timestamp: Instant::now(),
            severity: Severity::Critical,
            category: EventCategory::PathTraversal,
            message: "Path traversal attempt detected".to_string(),
            details: Some(format!("Attempted path: {}", attempted_path)),
        });
    }

    /// Log an audio safety violation
    pub fn log_audio_safety_violation(&self, violation_type: &str, value: f32) {
        self.log_event(SecurityEvent {
            timestamp: Instant::now(),
            severity: Severity::High,
            category: EventCategory::AudioSafety,
            message: format!("Audio safety violation: {}", violation_type),
            details: Some(format!("Value: {:.3}", value)),
        });
    }

    /// Log an input validation failure
    pub fn log_input_validation_failure(&self, parameter: &str, value: &str) {
        self.log_event(SecurityEvent {
            timestamp: Instant::now(),
            severity: Severity::Medium,
            category: EventCategory::InputValidation,
            message: format!("Input validation failed for: {}", parameter),
            details: Some(format!("Invalid value: {}", value)),
        });
    }

    /// Log a resource limit exceeded
    pub fn log_resource_limit_exceeded(&self, resource: &str, limit: &str) {
        self.log_event(SecurityEvent {
            timestamp: Instant::now(),
            severity: Severity::Medium,
            category: EventCategory::ResourceLimit,
            message: format!("Resource limit exceeded: {}", resource),
            details: Some(format!("Limit: {}", limit)),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_event_logging() {
        let monitor = SecurityMonitor::new();

        // Log various severity events
        monitor.log_event(SecurityEvent {
            timestamp: Instant::now(),
            severity: Severity::Low,
            category: EventCategory::FileAccess,
            message: "Test low event".to_string(),
            details: None,
        });

        monitor.log_event(SecurityEvent {
            timestamp: Instant::now(),
            severity: Severity::Critical,
            category: EventCategory::PathTraversal,
            message: "Test critical event".to_string(),
            details: Some("Details".to_string()),
        });

        let summary = monitor.get_summary();
        assert_eq!(summary.total_events, 2);
        assert_eq!(summary.low_count, 1);
        assert_eq!(summary.critical_count, 1);
    }

    #[test]
    fn test_violation_tracking() {
        let config = MonitorConfig {
            violation_threshold: 3,
            violation_window: Duration::from_secs(1),
            ..Default::default()
        };

        let monitor = SecurityMonitor::with_config(config);

        // Should not trigger lockdown
        for _ in 0..2 {
            monitor.log_event(SecurityEvent {
                timestamp: Instant::now(),
                severity: Severity::High,
                category: EventCategory::AudioSafety,
                message: "Test violation".to_string(),
                details: None,
            });
        }

        assert!(!monitor.is_lockdown());

        // Third violation should trigger lockdown
        monitor.log_event(SecurityEvent {
            timestamp: Instant::now(),
            severity: Severity::High,
            category: EventCategory::AudioSafety,
            message: "Test violation".to_string(),
            details: None,
        });

        assert!(monitor.is_lockdown());
    }

    #[test]
    fn test_lockdown_mode() {
        let monitor = SecurityMonitor::new();

        assert!(!monitor.is_lockdown());

        // Enter lockdown
        monitor.enter_lockdown();
        assert!(monitor.is_lockdown());

        // Exit lockdown
        monitor.exit_lockdown();
        assert!(!monitor.is_lockdown());
        assert_eq!(monitor.get_violation_count(), 0);
    }

    #[test]
    fn test_event_filtering() {
        let monitor = SecurityMonitor::new();

        // Add events of different severities
        monitor.log_event(SecurityEvent {
            timestamp: Instant::now(),
            severity: Severity::Low,
            category: EventCategory::FileAccess,
            message: "Low".to_string(),
            details: None,
        });

        monitor.log_event(SecurityEvent {
            timestamp: Instant::now(),
            severity: Severity::High,
            category: EventCategory::AudioSafety,
            message: "High".to_string(),
            details: None,
        });

        monitor.log_event(SecurityEvent {
            timestamp: Instant::now(),
            severity: Severity::Critical,
            category: EventCategory::PathTraversal,
            message: "Critical".to_string(),
            details: None,
        });

        // Filter by severity
        let high_events = monitor.get_events_by_severity(Severity::High);
        assert_eq!(high_events.len(), 2); // High and Critical
    }
}
