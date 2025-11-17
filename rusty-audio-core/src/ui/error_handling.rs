use super::accessibility::{AccessibilityManager, AnnouncementPriority};
use super::theme::ThemeColors;
use super::utils::{AnimationState, ColorUtils};
use egui::{Button, Color32, RichText, Ui, Vec2};
use std::time::{Duration, Instant};

/// Enhanced error handling system with user-friendly messages and recovery options
#[derive(Debug, Clone)]
pub struct ErrorManager {
    errors: Vec<ErrorInfo>,
    last_error_time: Option<Instant>,
    error_animation: AnimationState,
    auto_dismiss_timeout: Duration,
}

#[derive(Debug, Clone)]
pub struct ErrorInfo {
    pub id: String,
    pub error_type: ErrorType,
    pub title: String,
    pub message: String,
    pub details: Option<String>,
    pub recovery_actions: Vec<RecoveryAction>,
    pub timestamp: Instant,
    pub severity: ErrorSeverity,
    pub auto_dismiss: bool,
    pub dismissed: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorType {
    FileLoad,
    AudioDecode,
    AudioPlayback,
    Network,
    Permission,
    Configuration,
    General,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone)]
pub struct RecoveryAction {
    pub label: String,
    pub description: String,
    pub action_type: RecoveryActionType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RecoveryActionType {
    Retry,
    SelectDifferentFile,
    CheckPermissions,
    ResetSettings,
    ContactSupport,
    Dismiss,
}

impl ErrorManager {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            last_error_time: None,
            error_animation: AnimationState::new(0.0, 8.0),
            auto_dismiss_timeout: Duration::from_secs(10),
        }
    }

    pub fn add_error(
        &mut self,
        error_type: ErrorType,
        title: impl Into<String>,
        message: impl Into<String>,
    ) {
        let error_info = ErrorInfo {
            id: format!("error_{}", self.errors.len()),
            error_type: error_type.clone(),
            title: title.into(),
            message: message.into(),
            details: None,
            recovery_actions: self.get_default_recovery_actions(&error_type),
            timestamp: Instant::now(),
            severity: self.get_severity_for_type(&error_type),
            auto_dismiss: matches!(error_type, ErrorType::General | ErrorType::Network),
            dismissed: false,
        };

        self.errors.push(error_info);
        self.last_error_time = Some(Instant::now());
        self.error_animation.set_target(1.0);
    }

    pub fn add_detailed_error(&mut self, mut error_info: ErrorInfo) {
        error_info.id = format!("error_{}", self.errors.len());
        error_info.timestamp = Instant::now();

        self.errors.push(error_info);
        self.last_error_time = Some(Instant::now());
        self.error_animation.set_target(1.0);
    }

    pub fn update(&mut self, dt: f32) {
        self.error_animation.update(dt);

        // Auto-dismiss errors that have expired
        let now = Instant::now();
        for error in &mut self.errors {
            if error.auto_dismiss
                && !error.dismissed
                && error.timestamp.elapsed() > self.auto_dismiss_timeout
            {
                error.dismissed = true;
            }
        }

        // Remove very old dismissed errors
        self.errors.retain(|error| {
            !error.dismissed || error.timestamp.elapsed() < Duration::from_secs(60)
        });

        // If no active errors, fade out animation
        if self.errors.iter().all(|e| e.dismissed) {
            self.error_animation.set_target(0.0);
        }
    }

    pub fn show_errors(
        &mut self,
        ui: &mut Ui,
        colors: &ThemeColors,
        accessibility: &mut AccessibilityManager,
    ) -> Vec<RecoveryActionType> {
        let mut actions_to_execute = Vec::new();

        let active_errors: Vec<_> = self
            .errors
            .iter()
            .filter(|e| !e.dismissed)
            .cloned()
            .collect();
        if active_errors.is_empty() {
            return actions_to_execute;
        }

        let animation_value = self.error_animation.value();
        if animation_value < 0.01 {
            return actions_to_execute;
        }

        for (index, error) in active_errors.iter().enumerate() {
            let error_id = format!("error_display_{}", index);

            egui::Window::new(&error.title)
                .id(egui::Id::new(&error_id))
                .anchor(
                    egui::Align2::CENTER_CENTER,
                    Vec2::new(0.0, index as f32 * 50.0),
                )
                .collapsible(false)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    ui.set_max_width(500.0);

                    // Error icon and severity indicator
                    ui.horizontal(|ui| {
                        let (icon, icon_color) = match error.severity {
                            ErrorSeverity::Info => ("ℹ️", colors.primary),
                            ErrorSeverity::Warning => ("⚠️", colors.warning),
                            ErrorSeverity::Error => ("❌", colors.error),
                            ErrorSeverity::Critical => ("🚨", colors.error),
                        };

                        ui.label(RichText::new(icon).size(20.0));
                        ui.label(
                            RichText::new(&error.title)
                                .strong()
                                .size(16.0)
                                .color(icon_color),
                        );
                    });

                    ui.separator();

                    // Error message
                    ui.label(&error.message);

                    // Details (if available)
                    if let Some(details) = &error.details {
                        ui.add_space(8.0);
                        ui.collapsing("Technical Details", |ui| {
                            ui.label(RichText::new(details).family(egui::FontFamily::Monospace));
                        });
                    }

                    ui.add_space(12.0);

                    // Recovery actions
                    if !error.recovery_actions.is_empty() {
                        ui.label(RichText::new("What would you like to do?").strong());
                        ui.add_space(4.0);

                        for action in &error.recovery_actions {
                            ui.horizontal(|ui| {
                                let button_text = &action.label;
                                if ui.button(button_text).clicked() {
                                    actions_to_execute.push(action.action_type.clone());

                                    // Mark error as dismissed
                                    if let Some(error_mut) =
                                        self.errors.iter_mut().find(|e| e.id == error.id)
                                    {
                                        error_mut.dismissed = true;
                                    }

                                    // Announce action for screen readers
                                    accessibility.announce(
                                        format!("Executing recovery action: {}", action.label),
                                        AnnouncementPriority::Medium,
                                    );
                                }

                                if !action.description.is_empty() {
                                    ui.label(
                                        RichText::new(&action.description)
                                            .color(colors.text_secondary),
                                    );
                                }
                            });
                        }
                    }

                    ui.add_space(8.0);

                    // Dismiss button
                    ui.horizontal(|ui| {
                        if ui.button("Dismiss").clicked() {
                            if let Some(error_mut) =
                                self.errors.iter_mut().find(|e| e.id == error.id)
                            {
                                error_mut.dismissed = true;
                            }
                        }

                        // Auto-dismiss indicator
                        if error.auto_dismiss {
                            let remaining = self
                                .auto_dismiss_timeout
                                .saturating_sub(error.timestamp.elapsed());
                            ui.label(
                                RichText::new(format!(
                                    "Auto-dismiss in {:.0}s",
                                    remaining.as_secs_f32()
                                ))
                                .size(10.0)
                                .color(colors.text_secondary),
                            );
                        }
                    });
                });
        }

        actions_to_execute
    }

    pub fn clear_all_errors(&mut self) {
        self.errors.clear();
        self.error_animation.set_target(0.0);
    }

    pub fn has_active_errors(&self) -> bool {
        self.errors.iter().any(|e| !e.dismissed)
    }

    pub fn get_error_count(&self) -> usize {
        self.errors.iter().filter(|e| !e.dismissed).count()
    }

    fn get_default_recovery_actions(&self, error_type: &ErrorType) -> Vec<RecoveryAction> {
        match error_type {
            ErrorType::FileLoad => vec![
                RecoveryAction {
                    label: "Try Another File".to_string(),
                    description: "Select a different audio file".to_string(),
                    action_type: RecoveryActionType::SelectDifferentFile,
                },
                RecoveryAction {
                    label: "Retry".to_string(),
                    description: "Attempt to load the file again".to_string(),
                    action_type: RecoveryActionType::Retry,
                },
            ],
            ErrorType::AudioDecode => vec![
                RecoveryAction {
                    label: "Try Another File".to_string(),
                    description: "This file format may not be supported".to_string(),
                    action_type: RecoveryActionType::SelectDifferentFile,
                },
                RecoveryAction {
                    label: "Check Format".to_string(),
                    description: "Ensure the file is a valid audio format".to_string(),
                    action_type: RecoveryActionType::ContactSupport,
                },
            ],
            ErrorType::AudioPlayback => vec![
                RecoveryAction {
                    label: "Retry Playback".to_string(),
                    description: "Try playing the audio again".to_string(),
                    action_type: RecoveryActionType::Retry,
                },
                RecoveryAction {
                    label: "Reset Audio Settings".to_string(),
                    description: "Reset all audio settings to defaults".to_string(),
                    action_type: RecoveryActionType::ResetSettings,
                },
            ],
            ErrorType::Permission => vec![
                RecoveryAction {
                    label: "Check Permissions".to_string(),
                    description: "Verify file and folder permissions".to_string(),
                    action_type: RecoveryActionType::CheckPermissions,
                },
                RecoveryAction {
                    label: "Try Another Location".to_string(),
                    description: "Select a file from a different folder".to_string(),
                    action_type: RecoveryActionType::SelectDifferentFile,
                },
            ],
            ErrorType::Network => vec![RecoveryAction {
                label: "Retry".to_string(),
                description: "Try the network operation again".to_string(),
                action_type: RecoveryActionType::Retry,
            }],
            ErrorType::Configuration => vec![RecoveryAction {
                label: "Reset Settings".to_string(),
                description: "Reset all settings to defaults".to_string(),
                action_type: RecoveryActionType::ResetSettings,
            }],
            ErrorType::General => vec![RecoveryAction {
                label: "Retry".to_string(),
                description: "Try the operation again".to_string(),
                action_type: RecoveryActionType::Retry,
            }],
        }
    }

    fn get_severity_for_type(&self, error_type: &ErrorType) -> ErrorSeverity {
        match error_type {
            ErrorType::FileLoad => ErrorSeverity::Warning,
            ErrorType::AudioDecode => ErrorSeverity::Warning,
            ErrorType::AudioPlayback => ErrorSeverity::Error,
            ErrorType::Network => ErrorSeverity::Warning,
            ErrorType::Permission => ErrorSeverity::Error,
            ErrorType::Configuration => ErrorSeverity::Warning,
            ErrorType::General => ErrorSeverity::Info,
        }
    }
}

/// Helper functions for creating specific error types
impl ErrorManager {
    pub fn add_file_load_error(&mut self, filename: &str, details: Option<String>) {
        let has_details = details.is_some();
        let mut error = ErrorInfo {
            id: String::new(), // Will be set by add_detailed_error
            error_type: ErrorType::FileLoad,
            title: "Failed to Load Audio File".to_string(),
            message: format!("Could not load the file '{}'.", filename),
            details,
            recovery_actions: vec![
                RecoveryAction {
                    label: "🔄 Try Again".to_string(),
                    description: "Attempt to load the file again".to_string(),
                    action_type: RecoveryActionType::Retry,
                },
                RecoveryAction {
                    label: "📁 Choose Different File".to_string(),
                    description: "Select a different audio file".to_string(),
                    action_type: RecoveryActionType::SelectDifferentFile,
                },
            ],
            timestamp: Instant::now(),
            severity: ErrorSeverity::Warning,
            auto_dismiss: false,
            dismissed: false,
        };

        if has_details {
            error
                .message
                .push_str(" Check the technical details below for more information.");
        }

        self.add_detailed_error(error);
    }

    pub fn add_audio_decode_error(&mut self, filename: &str, format_hint: Option<&str>) {
        let message = if let Some(format) = format_hint {
            format!("The file '{}' appears to be a {} file, but could not be decoded. The file may be corrupted or use an unsupported variant of the format.", filename, format)
        } else {
            format!("The file '{}' could not be decoded. It may be corrupted or in an unsupported format.", filename)
        };

        let error = ErrorInfo {
            id: String::new(),
            error_type: ErrorType::AudioDecode,
            title: "Audio Format Not Supported".to_string(),
            message,
            details: Some(format!(
                "Supported formats: MP3, WAV, FLAC, OGG, M4A\nFile: {}",
                filename
            )),
            recovery_actions: vec![
                RecoveryAction {
                    label: "📁 Choose Different File".to_string(),
                    description: "Select a file in a supported format".to_string(),
                    action_type: RecoveryActionType::SelectDifferentFile,
                },
                RecoveryAction {
                    label: "❓ Get Help".to_string(),
                    description: "Learn about supported audio formats".to_string(),
                    action_type: RecoveryActionType::ContactSupport,
                },
            ],
            timestamp: Instant::now(),
            severity: ErrorSeverity::Warning,
            auto_dismiss: false,
            dismissed: false,
        };

        self.add_detailed_error(error);
    }

    pub fn add_playback_error(&mut self, details: Option<String>) {
        let error = ErrorInfo {
            id: String::new(),
            error_type: ErrorType::AudioPlayback,
            title: "Audio Playback Error".to_string(),
            message:
                "An error occurred during audio playback. The audio may have stopped unexpectedly."
                    .to_string(),
            details,
            recovery_actions: vec![
                RecoveryAction {
                    label: "▶️ Try Playing Again".to_string(),
                    description: "Restart audio playback".to_string(),
                    action_type: RecoveryActionType::Retry,
                },
                RecoveryAction {
                    label: "🔧 Reset Audio Settings".to_string(),
                    description: "Reset equalizer and audio settings".to_string(),
                    action_type: RecoveryActionType::ResetSettings,
                },
            ],
            timestamp: Instant::now(),
            severity: ErrorSeverity::Error,
            auto_dismiss: false,
            dismissed: false,
        };

        self.add_detailed_error(error);
    }

    pub fn add_permission_error(&mut self, operation: &str, path: &str) {
        let error = ErrorInfo {
            id: String::new(),
            error_type: ErrorType::Permission,
            title: "Permission Denied".to_string(),
            message: format!("Cannot {} '{}' due to insufficient permissions.", operation, path),
            details: Some(format!("Operation: {}\nPath: {}\nThis may be due to file or folder permissions, or the file being in use by another application.", operation, path)),
            recovery_actions: vec![
                RecoveryAction {
                    label: "🔐 Check Permissions".to_string(),
                    description: "Verify you have access to this file".to_string(),
                    action_type: RecoveryActionType::CheckPermissions,
                },
                RecoveryAction {
                    label: "📁 Try Different Location".to_string(),
                    description: "Select a file from a different folder".to_string(),
                    action_type: RecoveryActionType::SelectDifferentFile,
                },
            ],
            timestamp: Instant::now(),
            severity: ErrorSeverity::Error,
            auto_dismiss: false,
            dismissed: false,
        };

        self.add_detailed_error(error);
    }
}
