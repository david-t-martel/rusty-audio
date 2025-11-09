//! Recording Panel UI Component
//!
//! Professional recording interface with level meters, device selection,
//! and monitoring controls

use egui::{Color32, RichText, Ui, Vec2};

use super::theme::ThemeColors;
use crate::audio::recorder::{
    AudioRecorder, RecordingConfig,
    RecordingFormat, RecordingState, MonitoringMode,
};
use crate::audio::manager::AudioDeviceManager;
use crate::audio::backend::DeviceInfo;

/// Recording panel state
pub struct RecordingPanel {
    recorder: Option<AudioRecorder>,
    device_manager: Option<AudioDeviceManager>,
    available_input_devices: Vec<DeviceInfo>,
    selected_input_device_id: Option<String>,
    monitoring_gain: f32,
    show_save_dialog: bool,
    save_path: String,
    save_format: RecordingFormat,
    
    // Level metering
    peak_levels: Vec<f32>,  // Per channel
    rms_levels: Vec<f32>,   // Per channel
    clip_indicators: Vec<bool>,  // Per channel
    last_meter_update: std::time::Instant,
}

impl Default for RecordingPanel {
    fn default() -> Self {
        // Try to create device manager
        let device_manager = AudioDeviceManager::new().ok();
        
        // Enumerate input devices if we have a device manager
        let available_input_devices = device_manager.as_ref()
            .and_then(|dm| dm.enumerate_input_devices().ok())
            .unwrap_or_default();
        
        Self {
            recorder: None,
            device_manager,
            available_input_devices,
            selected_input_device_id: None,
            monitoring_gain: 1.0,
            show_save_dialog: false,
            save_path: String::new(),
            save_format: RecordingFormat::Wav,
            peak_levels: vec![0.0; 2],  // Stereo default
            rms_levels: vec![0.0; 2],
            clip_indicators: vec![false; 2],
            last_meter_update: std::time::Instant::now(),
        }
    }
}

impl RecordingPanel {
    pub fn new() -> Self {
        let mut panel = Self::default();
        // Initialize recorder with default configuration
        panel.initialize_recorder(RecordingConfig::default());
        panel
    }
    
    /// Initialize recorder with configuration
    pub fn initialize_recorder(&mut self, config: RecordingConfig) {
        self.recorder = Some(AudioRecorder::new(config));
        let channels = self.recorder.as_ref().unwrap().config().channels as usize;
        self.peak_levels = vec![0.0; channels];
        self.rms_levels = vec![0.0; channels];
        self.clip_indicators = vec![false; channels];
    }
    
    /// Update level meters from recorder
    pub fn update_levels(&mut self) {
        if let Some(recorder) = &self.recorder {
            let buffer = recorder.buffer();
            let buffer_lock = buffer.lock().unwrap();
            
            for ch in 0..self.peak_levels.len() {
                self.peak_levels[ch] = buffer_lock.peak_level(ch);
                self.rms_levels[ch] = buffer_lock.rms_level(ch);
                
                // Detect clipping (> 0.99)
                if self.peak_levels[ch] > 0.99 {
                    self.clip_indicators[ch] = true;
                }
            }
        }
    }
    
    /// Clear clip indicators
    pub fn clear_clips(&mut self) {
        self.clip_indicators.fill(false);
    }
    
    /// Draw the recording panel
    pub fn draw(&mut self, ui: &mut Ui, colors: &ThemeColors) {
        ui.vertical(|ui| {
            ui.heading(RichText::new("🎙️ Recording").size(20.0));
            ui.add_space(10.0);
            
            // Recording status and controls
            self.draw_recording_controls(ui, colors);
            
            ui.add_space(15.0);
            
            // Level meters
            self.draw_level_meters(ui, colors);
            
            ui.add_space(15.0);
            
            // Input device selection
            self.draw_input_device_selection(ui, colors);
            
            ui.add_space(15.0);
            
            // Monitoring controls
            self.draw_monitoring_controls(ui, colors);
            
            ui.add_space(15.0);
            
            // File management
            self.draw_file_management(ui, colors);
        });
    }
    
    /// Draw recording control buttons
    fn draw_recording_controls(&mut self, ui: &mut Ui, colors: &ThemeColors) {
        ui.group(|ui| {
            ui.label(RichText::new("Transport").strong());
            ui.add_space(5.0);
            
            let state = self.recorder.as_ref()
                .map(|r| r.state())
                .unwrap_or(RecordingState::Idle);
            
            ui.horizontal(|ui| {
                // Record button
                let record_button_color = if state == RecordingState::Recording {
                    Color32::from_rgb(200, 50, 50)  // Red when recording
                } else {
                    Color32::from_rgb(100, 100, 100)  // Gray when idle
                };
                
                let record_text = if state == RecordingState::Recording {
                    "⏺ Recording..."
                } else {
                    "⏺ Record"
                };
                
                if ui.button(RichText::new(record_text).color(record_button_color).size(16.0))
                    .clicked() && state != RecordingState::Recording
                {
                    if let Some(recorder) = &mut self.recorder {
                        let _ = recorder.start();
                    }
                }
                
                // Stop button
                if ui.button(RichText::new("⏹ Stop").size(16.0))
                    .clicked()
                {
                    if let Some(recorder) = &mut self.recorder {
                        let _ = recorder.stop();
                    }
                }
                
                // Pause button
                let pause_text = if state == RecordingState::Paused {
                    "▶ Resume"
                } else {
                    "⏸ Pause"
                };
                
                if ui.button(RichText::new(pause_text).size(16.0))
                    .clicked()
                {
                    if let Some(recorder) = &mut self.recorder {
                        match state {
                            RecordingState::Recording => { let _ = recorder.pause(); }
                            RecordingState::Paused => { let _ = recorder.resume(); }
                            _ => {}
                        }
                    }
                }
            });
            
            ui.add_space(5.0);
            
            // Status display
            let status_text = match state {
                RecordingState::Idle => "⚪ Idle",
                RecordingState::Recording => "🔴 Recording",
                RecordingState::Paused => "⏸️ Paused",
                RecordingState::Stopped => "⏹️ Stopped",
            };
            
            let status_color = match state {
                RecordingState::Idle => colors.text_secondary,
                RecordingState::Recording => Color32::from_rgb(255, 100, 100),
                RecordingState::Paused => Color32::from_rgb(255, 200, 100),
                RecordingState::Stopped => colors.text_secondary,
            };
            
            ui.label(RichText::new(status_text).color(status_color).strong());
            
            // Duration display
            if let Some(recorder) = &self.recorder {
                let duration = recorder.duration();
                let duration_text = format!("Duration: {:02}:{:02}.{:01}",
                    duration.as_secs() / 60,
                    duration.as_secs() % 60,
                    duration.subsec_millis() / 100
                );
                ui.label(RichText::new(duration_text).size(14.0));
                
                // Buffer info
                let buffer = recorder.buffer();
                let buffer_lock = buffer.lock().unwrap();
                let buffer_duration = buffer_lock.duration();
                let buffer_text = format!("Recorded: {:.1}s", buffer_duration.as_secs_f32());
                ui.label(RichText::new(buffer_text).size(12.0).color(colors.text_secondary));
            }
        });
    }
    
    /// Draw level meters
    fn draw_level_meters(&mut self, ui: &mut Ui, colors: &ThemeColors) {
        ui.group(|ui| {
            ui.label(RichText::new("📊 Input Levels").strong());
            ui.add_space(5.0);
            
            for (ch, (&peak, &rms)) in self.peak_levels.iter()
                .zip(self.rms_levels.iter())
                .enumerate()
            {
                ui.horizontal(|ui| {
                    ui.label(format!("Ch {}", ch + 1));
                    
                    // Level meter bar
                    let meter_width = 200.0;
                    let meter_height = 20.0;
                    
                    let (rect, response) = ui.allocate_exact_size(
                        Vec2::new(meter_width, meter_height),
                        egui::Sense::hover()
                    );
                    
                    if ui.is_rect_visible(rect) {
                        let painter = ui.painter();
                        
                        // Background
                        painter.rect_filled(rect, 2.0, Color32::from_gray(40));
                        
                        // RMS level (darker green)
                        let rms_width = (rms * meter_width).min(meter_width);
                        let rms_rect = egui::Rect::from_min_size(
                            rect.min,
                            Vec2::new(rms_width, meter_height)
                        );
                        let rms_color = self.get_meter_color(rms, false);
                        painter.rect_filled(rms_rect, 2.0, rms_color);
                        
                        // Peak level (brighter)
                        let peak_width = (peak * meter_width).min(meter_width);
                        let peak_height = meter_height * 0.3;
                        let peak_rect = egui::Rect::from_min_size(
                            rect.min + egui::vec2(0.0, meter_height * 0.35),
                            Vec2::new(peak_width, peak_height)
                        );
                        let peak_color = self.get_meter_color(peak, true);
                        painter.rect_filled(peak_rect, 2.0, peak_color);
                        
                        // Clipping indicator at the end
                        if self.clip_indicators[ch] {
                            let clip_rect = egui::Rect::from_min_size(
                                egui::pos2(rect.max.x - 20.0, rect.min.y),
                                Vec2::new(18.0, meter_height)
                            );
                            painter.rect_filled(clip_rect, 2.0, Color32::from_rgb(255, 0, 0));
                        }
                        
                        // Grid lines every 6dB
                        for db in [-6, -12, -18, -24, -30] {
                            let level = 10.0_f32.powf(db as f32 / 20.0);
                            let x = rect.min.x + level * meter_width;
                            painter.line_segment(
                                [egui::pos2(x, rect.min.y), egui::pos2(x, rect.max.y)],
                                (1.0, Color32::from_gray(60))
                            );
                        }
                    }
                    
                    // dB readout
                    let peak_db = if peak > 0.0001 {
                        20.0 * peak.log10()
                    } else {
                        -60.0
                    };
                    
                    let db_text = if peak_db > -60.0 {
                        format!("{:.1} dB", peak_db)
                    } else {
                        "-∞ dB".to_string()
                    };
                    
                    let db_color = if peak_db > -3.0 {
                        Color32::from_rgb(255, 100, 100)  // Red
                    } else if peak_db > -6.0 {
                        Color32::from_rgb(255, 200, 100)  // Yellow
                    } else {
                        colors.text
                    };
                    
                    ui.label(RichText::new(db_text).color(db_color).monospace());
                });
            }
            
            ui.add_space(5.0);
            
            // Clear clips button
            if ui.button("Clear Clips").clicked() {
                self.clear_clips();
            }
        });
    }
    
    /// Get meter color based on level
    fn get_meter_color(&self, level: f32, is_peak: bool) -> Color32 {
        let db = if level > 0.0001 {
            20.0 * level.log10()
        } else {
            -60.0
        };
        
        let brightness = if is_peak { 1.0 } else { 0.7 };
        
        if db > -3.0 {
            // Red zone
            Color32::from_rgb(
                (255.0 * brightness) as u8,
                (50.0 * brightness) as u8,
                (50.0 * brightness) as u8
            )
        } else if db > -6.0 {
            // Yellow zone
            Color32::from_rgb(
                (255.0 * brightness) as u8,
                (200.0 * brightness) as u8,
                (50.0 * brightness) as u8
            )
        } else if db > -18.0 {
            // Green zone
            Color32::from_rgb(
                (50.0 * brightness) as u8,
                (255.0 * brightness) as u8,
                (50.0 * brightness) as u8
            )
        } else {
            // Low level
            Color32::from_rgb(
                (50.0 * brightness) as u8,
                (150.0 * brightness) as u8,
                (50.0 * brightness) as u8
            )
        }
    }
    
    /// Draw input device selection
    fn draw_input_device_selection(&mut self, ui: &mut Ui, colors: &ThemeColors) {
        ui.group(|ui| {
            ui.label(RichText::new("🎤 Input Device").strong());
            ui.add_space(5.0);
            
            // Find selected device name
            let selected_name = self.selected_input_device_id.as_ref()
                .and_then(|id| self.available_input_devices.iter()
                    .find(|d| &d.id == id)
                    .map(|d| d.name.clone()))
                .unwrap_or_else(|| "No device selected".to_string());
            
            // Device dropdown with real devices
            let mut newly_selected_device: Option<String> = None;
            egui::ComboBox::from_label("")
                .selected_text(&selected_name)
                .show_ui(ui, |ui| {
                    for device in &self.available_input_devices {
                        let icon = if device.is_default { "🎤" } else { "🎵" };
                        let label = if device.is_default {
                            format!("{} {} (Default)", icon, device.name)
                        } else {
                            format!("{} {}", icon, device.name)
                        };
                        
                        let mut device_id_option = self.selected_input_device_id.clone();
                        if ui.selectable_value(&mut device_id_option, Some(device.id.clone()), label).clicked() {
                            newly_selected_device = Some(device.id.clone());
                        }
                    }
                });
            
            // Connect to newly selected device after the ComboBox closes
            if let Some(device_id) = newly_selected_device {
                self.selected_input_device_id = Some(device_id.clone());
                self.connect_to_device(&device_id);
            }
            
            ui.add_space(5.0);
            
            // Status message
            let status = if self.available_input_devices.is_empty() {
                RichText::new("⚠️ No input devices found").color(Color32::from_rgb(255, 200, 100))
            } else if self.selected_input_device_id.is_some() {
                RichText::new("✓ Device connected and ready").color(Color32::from_rgb(100, 255, 100))
            } else {
                RichText::new("ℹ️ Select your audio input device").color(colors.text_secondary)
            };
            
            ui.label(status.size(11.0));
        });
    }
    
    /// Draw monitoring controls
    fn draw_monitoring_controls(&mut self, ui: &mut Ui, colors: &ThemeColors) {
        ui.group(|ui| {
            ui.label(RichText::new("🎧 Monitoring").strong());
            ui.add_space(5.0);
            
            let current_mode = self.recorder.as_ref()
                .map(|r| r.monitoring_mode())
                .unwrap_or(MonitoringMode::Off);
            
            ui.horizontal(|ui| {
                ui.label("Mode:");
                
                if ui.radio(current_mode == MonitoringMode::Off, "🔇 Off")
                    .on_hover_text("No monitoring - silent recording")
                    .clicked()
                {
                    if let Some(recorder) = &mut self.recorder {
                        recorder.set_monitoring_mode(MonitoringMode::Off);
                    }
                }
                
                if ui.radio(current_mode == MonitoringMode::Direct, "⚡ Direct")
                    .on_hover_text("Zero-latency direct monitoring (ASIO-style)")
                    .clicked()
                {
                    if let Some(recorder) = &mut self.recorder {
                        recorder.set_monitoring_mode(MonitoringMode::Direct);
                    }
                }
                
                if ui.radio(current_mode == MonitoringMode::Routed, "🎛️ Routed")
                    .on_hover_text("Monitor through effects chain")
                    .clicked()
                {
                    if let Some(recorder) = &mut self.recorder {
                        recorder.set_monitoring_mode(MonitoringMode::Routed);
                    }
                }
            });
            
            ui.add_space(5.0);
            
            // Monitoring gain slider
            if current_mode != MonitoringMode::Off {
                ui.horizontal(|ui| {
                    ui.label("Gain:");
                    if ui.add(
                        egui::Slider::new(&mut self.monitoring_gain, 0.0..=1.0)
                            .text("")
                    ).changed() {
                        if let Some(recorder) = &mut self.recorder {
                            recorder.set_monitoring_gain(self.monitoring_gain);
                        }
                    }
                    ui.label(format!("{:.0}%", self.monitoring_gain * 100.0));
                });
            }
            
            // Mode description
            let description = match current_mode {
                MonitoringMode::Off => "Recording without monitoring. Use for overdubs or when monitoring externally.",
                MonitoringMode::Direct => "Input signal routed directly to output with minimal latency. Professional tracking mode.",
                MonitoringMode::Routed => "Input monitored through the effects chain. Creative recording mode.",
            };
            
            ui.add_space(5.0);
            ui.label(RichText::new(description).size(11.0).color(colors.text_secondary).italics());
        });
    }
    
    /// Draw file management controls
    fn draw_file_management(&mut self, ui: &mut Ui, colors: &ThemeColors) {
        ui.group(|ui| {
            ui.label(RichText::new("💾 File Management").strong());
            ui.add_space(5.0);
            
            // Format selection
            ui.horizontal(|ui| {
                ui.label("Format:");
                egui::ComboBox::from_label("")
                    .selected_text(match self.save_format {
                        RecordingFormat::Wav => "WAV (32-bit float)",
                        RecordingFormat::Flac => "FLAC (lossless)",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.save_format, RecordingFormat::Wav, "🎵 WAV (32-bit float)");
                        ui.selectable_value(&mut self.save_format, RecordingFormat::Flac, "📦 FLAC (lossless)");
                    });
            });
            
            ui.add_space(5.0);
            
            // Save button
            let can_save = self.recorder.as_ref()
                .map(|r| r.buffer().lock().unwrap().position() > 0)
                .unwrap_or(false);
            
            ui.horizontal(|ui| {
                if ui.add_enabled(can_save, egui::Button::new("💾 Save Recording..."))
                    .clicked()
                {
                    // TODO: Open file dialog
                    self.show_save_dialog = true;
                }
                
                if ui.button("🗑️ Clear Buffer").clicked() {
                    if let Some(recorder) = &self.recorder {
                        recorder.buffer().lock().unwrap().clear();
                    }
                }
            });
            
            ui.add_space(5.0);
            ui.label(RichText::new("ℹ️ Recording will be saved as multi-channel interleaved audio")
                .size(11.0)
                .color(colors.text_secondary));
        });
    }
    
    /// Get recorder reference
    pub fn recorder(&self) -> Option<&AudioRecorder> {
        self.recorder.as_ref()
    }
    
    /// Get mutable recorder reference
    pub fn recorder_mut(&mut self) -> Option<&mut AudioRecorder> {
        self.recorder.as_mut()
    }
    
    /// Connect AudioRecorder to an input device
    fn connect_to_device(&mut self, device_id: &str) {
        if let Some(recorder) = &mut self.recorder {
            if let Err(e) = recorder.connect_input_device(device_id) {
                eprintln!("Failed to connect to input device {}: {}", device_id, e);
            }
        }
    }
}
