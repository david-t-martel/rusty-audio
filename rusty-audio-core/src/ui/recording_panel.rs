//! Recording Panel UI Component
//!
//! Professional recording interface with level meters, device selection,
//! and monitoring controls

use chrono::Local;
use egui::{Color32, RichText, Ui, Vec2};
use std::time::{Duration, Instant};

use super::{theme::ThemeColors, utils::ColorUtils};
use crate::audio::backend::DeviceInfo;
use crate::audio::manager::AudioDeviceManager;
use crate::audio::recorder::{
    AudioRecorder, MonitoringMode, RecordingConfig, RecordingFormat, RecordingState,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TakeSource {
    Live,
    Generated,
}

impl TakeSource {
    fn label(&self) -> &'static str {
        match self {
            TakeSource::Live => "Live",
            TakeSource::Generated => "Generated",
        }
    }
}

#[derive(Debug, Clone)]
struct RecordedTake {
    id: usize,
    label: String,
    source: TakeSource,
    duration: Duration,
    peak: f32,
    rms: f32,
    clip_events: u32,
    timestamp_label: String,
    waveform: Vec<f32>,
    notes: String,
}

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
    peak_levels: Vec<f32>,      // Per channel
    rms_levels: Vec<f32>,       // Per channel
    clip_indicators: Vec<bool>, // Per channel
    last_meter_update: Instant,
    takes: Vec<RecordedTake>,
    selected_take: Option<usize>,
    last_state: RecordingState,
    next_take_id: usize,
}

impl Default for RecordingPanel {
    fn default() -> Self {
        // Try to create device manager
        let device_manager = AudioDeviceManager::new().ok();

        // Enumerate input devices if we have a device manager
        let available_input_devices = device_manager
            .as_ref()
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
            peak_levels: vec![0.0; 2], // Stereo default
            rms_levels: vec![0.0; 2],
            clip_indicators: vec![false; 2],
            last_meter_update: Instant::now(),
            takes: Vec::new(),
            selected_take: None,
            last_state: RecordingState::Idle,
            next_take_id: 1,
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
        let channels = config.channels as usize;
        self.recorder = Some(AudioRecorder::new(config));
        self.peak_levels = vec![0.0; channels];
        self.rms_levels = vec![0.0; channels];
        self.clip_indicators = vec![false; channels];
    }

    pub fn current_state(&self) -> RecordingState {
        self.recorder
            .as_ref()
            .map(|rec| rec.state())
            .unwrap_or(RecordingState::Idle)
    }

    pub fn is_recording(&self) -> bool {
        matches!(self.current_state(), RecordingState::Recording)
    }

    pub fn toggle_recording(&mut self) {
        if let Some(recorder) = &mut self.recorder {
            match recorder.state() {
                RecordingState::Recording => {
                    let _ = recorder.stop();
                }
                RecordingState::Paused => {
                    let _ = recorder.resume();
                }
                _ => {
                    let _ = recorder.start();
                }
            }
        }
    }

    pub fn status_badge(&self) -> (&'static str, Color32) {
        match self.current_state() {
            RecordingState::Recording => ("REC", Color32::from_rgb(255, 80, 80)),
            RecordingState::Paused => ("Paused", Color32::from_rgb(255, 200, 120)),
            RecordingState::Stopped => ("Stopped", Color32::from_rgb(170, 170, 170)),
            RecordingState::Idle => ("Idle", Color32::from_rgb(120, 160, 200)),
        }
    }

    /// Update level meters from recorder
    pub fn update_levels(&mut self) {
        let mut current_state = RecordingState::Idle;
        if let Some(recorder) = &self.recorder {
            current_state = recorder.state();
            let buffer = recorder.buffer();
            // Lock-free buffer - direct access, no .lock() needed

            for ch in 0..self.peak_levels.len() {
                self.peak_levels[ch] = buffer.peak_level(ch);
                self.rms_levels[ch] = buffer.rms_level(ch);

                // Detect clipping (> 0.99)
                if self.peak_levels[ch] > 0.99 {
                    self.clip_indicators[ch] = true;
                }
            }
        }

        self.handle_state_transition(current_state);
    }

    /// Clear clip indicators
    pub fn clear_clips(&mut self) {
        self.clip_indicators.fill(false);
    }

    fn handle_state_transition(&mut self, current_state: RecordingState) {
        if self.last_state == RecordingState::Recording && current_state == RecordingState::Stopped
        {
            // Capture data from recorder before calling capture_live_take
            if let Some(recorder) = &self.recorder {
                let buffer = recorder.buffer();
                let mut samples = Vec::new();
                buffer.get_samples(&mut samples);

                if !samples.is_empty() {
                    let channels = recorder.config().channels.max(1) as usize;
                    let sample_rate = recorder.config().sample_rate as f32;

                    // Now call the method with extracted data (no borrow conflict)
                    self.add_take_from_samples(
                        format!("Take {}", self.next_take_id),
                        TakeSource::Live,
                        &samples,
                        sample_rate,
                        channels,
                    );
                }
            }
        }
        self.last_state = current_state;
    }

    fn add_take_from_samples(
        &mut self,
        label: String,
        source: TakeSource,
        samples: &[f32],
        sample_rate: f32,
        channels: usize,
    ) {
        if samples.is_empty() || channels == 0 {
            return;
        }

        let frames = samples.len() / channels;
        if frames == 0 {
            return;
        }

        let duration_secs = frames as f32 / sample_rate.max(1.0);
        let duration = Duration::from_secs_f32(duration_secs);
        let (peak, rms, clip_events) = Self::analyze_samples(samples);
        let waveform = Self::downsample_waveform(samples, channels, 256);

        if self.takes.len() >= 32 {
            self.takes.remove(0);
            if let Some(selected) = self.selected_take {
                self.selected_take = selected.checked_sub(1);
            }
        }

        let take = RecordedTake {
            id: self.next_take_id,
            label,
            source,
            duration,
            peak,
            rms,
            clip_events,
            timestamp_label: Local::now().format("%H:%M:%S").to_string(),
            waveform,
            notes: String::new(),
        };

        self.next_take_id += 1;
        self.takes.push(take);
        self.selected_take = Some(self.takes.len().saturating_sub(1));
    }

    fn analyze_samples(samples: &[f32]) -> (f32, f32, u32) {
        if samples.is_empty() {
            return (0.0, 0.0, 0);
        }

        let mut peak: f32 = 0.0;
        let mut sum_squares = 0.0;
        let mut clip_events = 0;

        for &sample in samples {
            let abs = sample.abs();
            peak = peak.max(abs);
            sum_squares += sample * sample;
            if abs >= 0.99 {
                clip_events += 1;
            }
        }

        let rms = (sum_squares / samples.len() as f32).sqrt();
        (peak, rms, clip_events)
    }

    fn downsample_waveform(samples: &[f32], channels: usize, target: usize) -> Vec<f32> {
        if samples.is_empty() || channels == 0 || target == 0 {
            return Vec::new();
        }

        let frames = samples.len() / channels;
        if frames == 0 {
            return Vec::new();
        }

        let step = (frames as f32 / target as f32).ceil() as usize;
        let mut waveform = Vec::with_capacity(target);

        for frame in (0..frames).step_by(step) {
            let mut sum = 0.0;
            for ch in 0..channels {
                let idx = frame * channels + ch;
                if idx < samples.len() {
                    sum += samples[idx];
                }
            }
            waveform.push(sum / channels as f32);
            if waveform.len() >= target {
                break;
            }
        }

        Self::normalize_waveform(waveform)
    }

    fn normalize_waveform(mut waveform: Vec<f32>) -> Vec<f32> {
        let max = waveform
            .iter()
            .fold(0.0f32, |acc, &sample| acc.max(sample.abs()));
        if max > 0.0 {
            for sample in &mut waveform {
                *sample /= max;
            }
        }
        waveform
    }

    fn linear_to_db(value: f32) -> f32 {
        20.0 * value.max(1e-6).log10()
    }

    /// Log a virtual take produced by the signal generator
    pub fn log_generated_take(
        &mut self,
        label: String,
        samples: &[f32],
        sample_rate: f32,
        channels: usize,
    ) {
        self.add_take_from_samples(label, TakeSource::Generated, samples, sample_rate, channels);
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

            ui.add_space(15.0);

            self.draw_take_manager(ui, colors);
        });
    }

    /// Draw recording control buttons
    fn draw_recording_controls(&mut self, ui: &mut Ui, colors: &ThemeColors) {
        ui.group(|ui| {
            ui.label(RichText::new("Transport").strong());
            ui.add_space(5.0);

            let state = self
                .recorder
                .as_ref()
                .map(|r| r.state())
                .unwrap_or(RecordingState::Idle);

            ui.horizontal(|ui| {
                // Record button
                let record_button_color = if state == RecordingState::Recording {
                    Color32::from_rgb(200, 50, 50) // Red when recording
                } else {
                    Color32::from_rgb(100, 100, 100) // Gray when idle
                };

                let record_text = if state == RecordingState::Recording {
                    "⏺ Recording..."
                } else {
                    "⏺ Record"
                };

                if ui
                    .button(
                        RichText::new(record_text)
                            .color(record_button_color)
                            .size(16.0),
                    )
                    .clicked()
                    && state != RecordingState::Recording
                {
                    if let Some(recorder) = &mut self.recorder {
                        let _ = recorder.start();
                    }
                }

                // Stop button
                if ui.button(RichText::new("⏹ Stop").size(16.0)).clicked() {
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

                if ui.button(RichText::new(pause_text).size(16.0)).clicked() {
                    if let Some(recorder) = &mut self.recorder {
                        match state {
                            RecordingState::Recording => {
                                let _ = recorder.pause();
                            }
                            RecordingState::Paused => {
                                let _ = recorder.resume();
                            }
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
                let duration_text = format!(
                    "Duration: {:02}:{:02}.{:01}",
                    duration.as_secs() / 60,
                    duration.as_secs() % 60,
                    duration.subsec_millis() / 100
                );
                ui.label(RichText::new(duration_text).size(14.0));

                // Buffer info (lock-free access)
                let buffer = recorder.buffer();
                let buffer_duration = buffer.duration();
                let buffer_text = format!("Recorded: {:.1}s", buffer_duration.as_secs_f32());
                ui.label(
                    RichText::new(buffer_text)
                        .size(12.0)
                        .color(colors.text_secondary),
                );
            }
        });
    }

    /// Draw level meters
    fn draw_level_meters(&mut self, ui: &mut Ui, colors: &ThemeColors) {
        ui.group(|ui| {
            ui.label(RichText::new("📊 Input Levels").strong());
            ui.add_space(5.0);

            for (ch, (&peak, &rms)) in self
                .peak_levels
                .iter()
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
                        egui::Sense::hover(),
                    );

                    if ui.is_rect_visible(rect) {
                        let painter = ui.painter();

                        // Background
                        painter.rect_filled(rect, 2.0, Color32::from_gray(40));

                        // RMS level (darker green)
                        let rms_width = (rms * meter_width).min(meter_width);
                        let rms_rect =
                            egui::Rect::from_min_size(rect.min, Vec2::new(rms_width, meter_height));
                        let rms_color = self.get_meter_color(rms, false);
                        painter.rect_filled(rms_rect, 2.0, rms_color);

                        // Peak level (brighter)
                        let peak_width = (peak * meter_width).min(meter_width);
                        let peak_height = meter_height * 0.3;
                        let peak_rect = egui::Rect::from_min_size(
                            rect.min + egui::vec2(0.0, meter_height * 0.35),
                            Vec2::new(peak_width, peak_height),
                        );
                        let peak_color = self.get_meter_color(peak, true);
                        painter.rect_filled(peak_rect, 2.0, peak_color);

                        // Clipping indicator at the end
                        if self.clip_indicators[ch] {
                            let clip_rect = egui::Rect::from_min_size(
                                egui::pos2(rect.max.x - 20.0, rect.min.y),
                                Vec2::new(18.0, meter_height),
                            );
                            painter.rect_filled(clip_rect, 2.0, Color32::from_rgb(255, 0, 0));
                        }

                        // Grid lines every 6dB
                        for db in [-6, -12, -18, -24, -30] {
                            let level = 10.0_f32.powf(db as f32 / 20.0);
                            let x = rect.min.x + level * meter_width;
                            painter.line_segment(
                                [egui::pos2(x, rect.min.y), egui::pos2(x, rect.max.y)],
                                (1.0, Color32::from_gray(60)),
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
                        Color32::from_rgb(255, 100, 100) // Red
                    } else if peak_db > -6.0 {
                        Color32::from_rgb(255, 200, 100) // Yellow
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
                (50.0 * brightness) as u8,
            )
        } else if db > -6.0 {
            // Yellow zone
            Color32::from_rgb(
                (255.0 * brightness) as u8,
                (200.0 * brightness) as u8,
                (50.0 * brightness) as u8,
            )
        } else if db > -18.0 {
            // Green zone
            Color32::from_rgb(
                (50.0 * brightness) as u8,
                (255.0 * brightness) as u8,
                (50.0 * brightness) as u8,
            )
        } else {
            // Low level
            Color32::from_rgb(
                (50.0 * brightness) as u8,
                (150.0 * brightness) as u8,
                (50.0 * brightness) as u8,
            )
        }
    }

    /// Draw input device selection
    fn draw_input_device_selection(&mut self, ui: &mut Ui, colors: &ThemeColors) {
        ui.group(|ui| {
            ui.label(RichText::new("🎤 Input Device").strong());
            ui.add_space(5.0);

            // Find selected device name
            let selected_name = self
                .selected_input_device_id
                .as_ref()
                .and_then(|id| {
                    self.available_input_devices
                        .iter()
                        .find(|d| &d.id == id)
                        .map(|d| d.name.clone())
                })
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
                        if ui
                            .selectable_value(&mut device_id_option, Some(device.id.clone()), label)
                            .clicked()
                        {
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
                RichText::new("✓ Device connected and ready")
                    .color(Color32::from_rgb(100, 255, 100))
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
                        ui.selectable_value(
                            &mut self.save_format,
                            RecordingFormat::Wav,
                            "🎵 WAV (32-bit float)",
                        );
                        ui.selectable_value(
                            &mut self.save_format,
                            RecordingFormat::Flac,
                            "📦 FLAC (lossless)",
                        );
                    });
            });

            ui.add_space(5.0);

            // Save button (lock-free access)
            let can_save = self
                .recorder
                .as_ref()
                .map(|r| r.buffer().position() > 0)
                .unwrap_or(false);

            ui.horizontal(|ui| {
                if ui
                    .add_enabled(can_save, egui::Button::new("💾 Save Recording..."))
                    .clicked()
                {
                    // TODO: Open file dialog
                    self.show_save_dialog = true;
                }

                if ui.button("🗑️ Clear Buffer").clicked() {
                    if let Some(recorder) = &self.recorder {
                        recorder.buffer().clear();
                    }
                }
            });

            ui.add_space(5.0);
            ui.label(
                RichText::new("ℹ️ Recording will be saved as multi-channel interleaved audio")
                    .size(11.0)
                    .color(colors.text_secondary),
            );
        });
    }

    fn draw_take_manager(&mut self, ui: &mut Ui, colors: &ThemeColors) {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("🎞️ Takes & Notes")
                        .size(16.0)
                        .color(colors.text),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!("{} total", self.takes.len()))
                            .color(colors.text_secondary),
                    );
                });
            });

            ui.add_space(6.0);

            if self.takes.is_empty() {
                ui.label(
                    RichText::new(
                        "No takes yet. Record or route a generator signal to capture one.",
                    )
                    .color(colors.text_secondary),
                );
                return;
            }

            let mut clicked_take: Option<usize> = None;
            egui::ScrollArea::vertical()
                .max_height(220.0)
                .show(ui, |ui| {
                    for (idx, take) in self.takes.iter().enumerate() {
                        let selected = Some(idx) == self.selected_take;
                        let header = format!(
                            "{} · {} · {:.1}s",
                            take.label,
                            take.source.label(),
                            take.duration.as_secs_f32()
                        );
                        let response = ui.selectable_label(selected, header);
                        if response.clicked() {
                            clicked_take = Some(idx);
                        }
                        if selected {
                            self.draw_take_details(ui, colors, take);
                        }
                        ui.add_space(6.0);
                    }
                });

            if let Some(idx) = clicked_take {
                self.selected_take = Some(idx);
            }

            if let Some(idx) = self.selected_take {
                if let Some(take) = self.takes.get_mut(idx) {
                    ui.add_space(6.0);
                    ui.label(RichText::new("Notes").strong());
                    ui.text_edit_multiline(&mut take.notes)
                        .on_hover_text("Add reminders for this take");
                }
            }
        });
    }

    fn draw_take_details(&self, ui: &mut Ui, colors: &ThemeColors, take: &RecordedTake) {
        ui.indent(format!("take_detail_{}", take.id), |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("Peak {:.1} dBFS", Self::linear_to_db(take.peak)))
                        .color(colors.text_secondary),
                );
                ui.add_space(10.0);
                ui.label(
                    RichText::new(format!("RMS {:.1} dBFS", Self::linear_to_db(take.rms)))
                        .color(colors.text_secondary),
                );
                ui.add_space(10.0);
                ui.label(
                    RichText::new(format!("Clips {}", take.clip_events))
                        .color(colors.text_secondary),
                );
                ui.add_space(10.0);
                ui.label(
                    RichText::new(format!("Captured {}", take.timestamp_label))
                        .color(colors.text_secondary),
                );
            });

            let (rect, _) =
                ui.allocate_exact_size(Vec2::new(ui.available_width(), 48.0), egui::Sense::hover());
            self.draw_take_waveform(ui, colors, rect, &take.waveform);
        });
    }

    fn draw_take_waveform(
        &self,
        ui: &mut Ui,
        colors: &ThemeColors,
        rect: egui::Rect,
        waveform: &[f32],
    ) {
        let painter = ui.painter();
        painter.rect_filled(rect, 6.0, ColorUtils::with_alpha(colors.surface, 0.7));

        if waveform.is_empty() {
            return;
        }

        let step = rect.width() / (waveform.len().max(1) as f32);
        let mut last_point = rect.center();
        for (i, sample) in waveform.iter().enumerate() {
            let x = rect.min.x + i as f32 * step;
            let y = rect.center().y - sample * rect.height() * 0.4;
            let point = egui::pos2(x, y);
            if i > 0 {
                painter.line_segment([last_point, point], egui::Stroke::new(1.0, colors.accent));
            }
            last_point = point;
        }
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
