use super::{
    controls::{ButtonStyle, EnhancedButton, EnhancedSlider, SliderOrientation, SliderStyle},
    theme::ThemeColors,
    utils::{AnimationState, ColorUtils},
};
use crate::testing::signal_generators::*;
use egui::{ComboBox, Pos2, Rect, Response, RichText, Sense, Stroke, Ui, Vec2};
#[cfg(not(target_arch = "wasm32"))]
use web_audio_api::{
    context::{AudioContext, BaseAudioContext},
    node::{AudioBufferSourceNode, AudioScheduledSourceNode},
};

#[derive(Debug, Clone, PartialEq)]
pub enum SignalType {
    Sine,
    WhiteNoise,
    PinkNoise,
    Square,
    Sawtooth,
    Sweep,
    Impulse,
    MultiTone,
}

impl SignalType {
    pub fn display_name(&self) -> &str {
        match self {
            SignalType::Sine => "Sine Wave",
            SignalType::WhiteNoise => "White Noise",
            SignalType::PinkNoise => "Pink Noise",
            SignalType::Square => "Square Wave",
            SignalType::Sawtooth => "Sawtooth Wave",
            SignalType::Sweep => "Frequency Sweep",
            SignalType::Impulse => "Impulse",
            SignalType::MultiTone => "Multi-Tone",
        }
    }

    pub fn all() -> Vec<SignalType> {
        vec![
            SignalType::Sine,
            SignalType::WhiteNoise,
            SignalType::PinkNoise,
            SignalType::Square,
            SignalType::Sawtooth,
            SignalType::Sweep,
            SignalType::Impulse,
            SignalType::MultiTone,
        ]
    }
}

#[derive(Debug, Clone)]
pub struct SignalParameters {
    pub frequency: f32,
    pub amplitude_db: f32,
    pub duration: f32,
    pub sample_rate: f32,
    pub duty_cycle: f32,
    pub start_frequency: f32,
    pub end_frequency: f32,
    pub phase: f32,
    pub seed: u64,
}

impl Default for SignalParameters {
    fn default() -> Self {
        Self {
            frequency: 1000.0,
            amplitude_db: -6.0,
            duration: 2.0,
            sample_rate: 44100.0,
            duty_cycle: 0.5,
            start_frequency: 20.0,
            end_frequency: 20000.0,
            phase: 0.0,
            seed: 42,
        }
    }
}

impl SignalParameters {
    pub fn amplitude(&self) -> f32 {
        10.0_f32.powf(self.amplitude_db / 20.0)
    }

    pub fn validate_frequency(&mut self) {
        self.frequency = self.frequency.clamp(20.0, 20000.0);
    }

    pub fn validate_amplitude(&mut self) {
        self.amplitude_db = self.amplitude_db.clamp(-60.0, 0.0);
    }

    pub fn validate_duration(&mut self) {
        self.duration = self.duration.clamp(0.1, 60.0);
    }

    pub fn validate_duty_cycle(&mut self) {
        self.duty_cycle = self.duty_cycle.clamp(0.05, 0.95);
    }

    pub fn validate_sweep_frequencies(&mut self) {
        self.start_frequency = self.start_frequency.clamp(10.0, 20000.0);
        self.end_frequency = self
            .end_frequency
            .clamp(self.start_frequency + 1.0, 20000.0);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum GeneratorState {
    Stopped,
    Playing,
    Generating,
}

pub struct SignalGeneratorPanel {
    pub signal_type: SignalType,
    pub parameters: SignalParameters,
    pub state: GeneratorState,
    pub generated_samples: Vec<f32>,
    pub preview_enabled: bool,
    pub spectrum_analysis_enabled: bool,

    // Audio nodes (native only)
    #[cfg(not(target_arch = "wasm32"))]
    pub source_node: Option<AudioBufferSourceNode>,

    // UI controls
    frequency_slider: EnhancedSlider,
    amplitude_slider: EnhancedSlider,
    duration_slider: EnhancedSlider,
    duty_cycle_slider: EnhancedSlider,
    start_freq_slider: EnhancedSlider,
    end_freq_slider: EnhancedSlider,
    phase_slider: EnhancedSlider,

    // UI state
    show_advanced_params: bool,
    waveform_samples: Vec<f32>,
    spectrum_data: Vec<f32>,

    // Animations
    generation_animation: AnimationState,
    preview_animation: AnimationState,
}

impl SignalGeneratorPanel {
    pub fn new() -> Self {
        let parameters = SignalParameters::default();

        Self {
            signal_type: SignalType::Sine,
            parameters: parameters.clone(),
            state: GeneratorState::Stopped,
            generated_samples: Vec::new(),
            preview_enabled: true,
            spectrum_analysis_enabled: false,
            #[cfg(not(target_arch = "wasm32"))]
            source_node: None,

            // Initialize sliders with parameter values
            frequency_slider: EnhancedSlider::new(parameters.frequency, 20.0..=20000.0)
                .orientation(SliderOrientation::Horizontal)
                .style(SliderStyle {
                    show_value: true,
                    show_ticks: true,
                    tick_count: 5,
                    gradient: true,
                    ..Default::default()
                }),
            amplitude_slider: EnhancedSlider::new(parameters.amplitude_db, -60.0..=0.0)
                .orientation(SliderOrientation::Horizontal)
                .style(SliderStyle {
                    show_value: true,
                    gradient: true,
                    ..Default::default()
                }),
            duration_slider: EnhancedSlider::new(parameters.duration, 0.1..=60.0)
                .orientation(SliderOrientation::Horizontal)
                .style(SliderStyle {
                    show_value: true,
                    ..Default::default()
                }),
            duty_cycle_slider: EnhancedSlider::new(parameters.duty_cycle, 0.05..=0.95)
                .orientation(SliderOrientation::Horizontal)
                .style(SliderStyle {
                    show_value: true,
                    ..Default::default()
                }),
            start_freq_slider: EnhancedSlider::new(parameters.start_frequency, 10.0..=20000.0)
                .orientation(SliderOrientation::Horizontal)
                .style(SliderStyle {
                    show_value: true,
                    ..Default::default()
                }),
            end_freq_slider: EnhancedSlider::new(parameters.end_frequency, 10.0..=20000.0)
                .orientation(SliderOrientation::Horizontal)
                .style(SliderStyle {
                    show_value: true,
                    ..Default::default()
                }),
            phase_slider: EnhancedSlider::new(parameters.phase, 0.0..=std::f32::consts::TAU)
                .orientation(SliderOrientation::Horizontal)
                .style(SliderStyle {
                    show_value: true,
                    ..Default::default()
                }),

            show_advanced_params: false,
            waveform_samples: Vec::new(),
            spectrum_data: Vec::new(),

            generation_animation: AnimationState::new(0.0, 8.0),
            preview_animation: AnimationState::new(0.0, 5.0),
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.generation_animation.update(dt);
        self.preview_animation.update(dt);

        // Update preview if enabled and parameters changed
        if self.preview_enabled && self.state != GeneratorState::Generating {
            self.update_preview();
        }
    }

    pub fn show(&mut self, ui: &mut Ui, colors: &ThemeColors) -> Response {
        let available_rect = ui.available_rect_before_wrap();
        let response = ui.allocate_rect(available_rect, Sense::hover());

        ui.vertical(|ui| {
            self.draw_signal_type_selector(ui, colors);
            ui.add_space(16.0);

            self.draw_parameter_controls(ui, colors);
            ui.add_space(16.0);

            self.draw_control_buttons(ui, colors);
            ui.add_space(16.0);

            if self.preview_enabled {
                self.draw_waveform_preview(ui, colors);
                ui.add_space(12.0);
            }

            if self.spectrum_analysis_enabled {
                self.draw_spectrum_analysis(ui, colors);
                ui.add_space(12.0);
            }

            self.draw_mathematical_verification(ui, colors);
        });

        response
    }

    fn draw_signal_type_selector(&mut self, ui: &mut Ui, colors: &ThemeColors) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Signal Type:").size(14.0).color(colors.text));
            ui.add_space(10.0);

            ComboBox::from_id_source("signal_type")
                .selected_text(self.signal_type.display_name())
                .show_ui(ui, |ui| {
                    for signal_type in SignalType::all() {
                        ui.selectable_value(
                            &mut self.signal_type,
                            signal_type.clone(),
                            signal_type.display_name(),
                        );
                    }
                });
        });
    }

    fn draw_parameter_controls(&mut self, ui: &mut Ui, colors: &ThemeColors) {
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Parameters").size(16.0).color(colors.text));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .small_button(if self.show_advanced_params {
                                "▼ Advanced"
                            } else {
                                "▶ Advanced"
                            })
                            .clicked()
                        {
                            self.show_advanced_params = !self.show_advanced_params;
                        }
                    });
                });

                ui.separator();
                ui.add_space(8.0);

                // Common parameters for all signal types
                self.draw_common_parameters(ui, colors);

                // Signal-specific parameters
                match self.signal_type {
                    SignalType::Sine => self.draw_sine_parameters(ui, colors),
                    SignalType::WhiteNoise | SignalType::PinkNoise => {
                        self.draw_noise_parameters(ui, colors)
                    }
                    SignalType::Square => self.draw_square_parameters(ui, colors),
                    SignalType::Sawtooth => self.draw_sawtooth_parameters(ui, colors),
                    SignalType::Sweep => self.draw_sweep_parameters(ui, colors),
                    SignalType::Impulse => self.draw_impulse_parameters(ui, colors),
                    SignalType::MultiTone => self.draw_multitone_parameters(ui, colors),
                }

                if self.show_advanced_params {
                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);
                    self.draw_advanced_parameters(ui, colors);
                }
            });
        });
    }

    fn draw_common_parameters(&mut self, ui: &mut Ui, colors: &ThemeColors) {
        // Amplitude control
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Amplitude (dB):")
                    .size(12.0)
                    .color(colors.text_secondary),
            );
            ui.add_space(10.0);

            let amplitude_response = self.amplitude_slider.show(ui, colors);
            if amplitude_response.changed() {
                self.parameters.amplitude_db = self.amplitude_slider.value();
                self.parameters.validate_amplitude();
            }

            ui.label(format!(
                "{:.1} dB ({:.3})",
                self.parameters.amplitude_db,
                self.parameters.amplitude()
            ));
        });

        ui.add_space(6.0);

        // Duration control
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Duration (s):")
                    .size(12.0)
                    .color(colors.text_secondary),
            );
            ui.add_space(10.0);

            let duration_response = self.duration_slider.show(ui, colors);
            if duration_response.changed() {
                self.parameters.duration = self.duration_slider.value();
                self.parameters.validate_duration();
            }

            ui.label(format!("{:.2} seconds", self.parameters.duration));
        });

        ui.add_space(6.0);
    }

    fn draw_sine_parameters(&mut self, ui: &mut Ui, colors: &ThemeColors) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Frequency (Hz):")
                    .size(12.0)
                    .color(colors.text_secondary),
            );
            ui.add_space(10.0);

            let freq_response = self.frequency_slider.show(ui, colors);
            if freq_response.changed() {
                self.parameters.frequency = self.frequency_slider.value();
                self.parameters.validate_frequency();
            }

            ui.label(format!("{:.0} Hz", self.parameters.frequency));
        });
    }

    fn draw_noise_parameters(&mut self, ui: &mut Ui, colors: &ThemeColors) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Seed:")
                    .size(12.0)
                    .color(colors.text_secondary),
            );
            ui.add_space(10.0);

            let mut seed_text = self.parameters.seed.to_string();
            if ui.text_edit_singleline(&mut seed_text).changed() {
                if let Ok(seed) = seed_text.parse::<u64>() {
                    self.parameters.seed = seed;
                }
            }
        });
    }

    fn draw_square_parameters(&mut self, ui: &mut Ui, colors: &ThemeColors) {
        self.draw_sine_parameters(ui, colors);
        ui.add_space(6.0);

        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Duty Cycle:")
                    .size(12.0)
                    .color(colors.text_secondary),
            );
            ui.add_space(10.0);

            let duty_response = self.duty_cycle_slider.show(ui, colors);
            if duty_response.changed() {
                self.parameters.duty_cycle = self.duty_cycle_slider.value();
                self.parameters.validate_duty_cycle();
            }

            ui.label(format!("{:.1}%", self.parameters.duty_cycle * 100.0));
        });
    }

    fn draw_sawtooth_parameters(&mut self, ui: &mut Ui, colors: &ThemeColors) {
        self.draw_sine_parameters(ui, colors);
    }

    fn draw_sweep_parameters(&mut self, ui: &mut Ui, colors: &ThemeColors) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Start Frequency (Hz):")
                    .size(12.0)
                    .color(colors.text_secondary),
            );
            ui.add_space(10.0);

            let start_response = self.start_freq_slider.show(ui, colors);
            if start_response.changed() {
                self.parameters.start_frequency = self.start_freq_slider.value();
                self.parameters.validate_sweep_frequencies();
            }

            ui.label(format!("{:.0} Hz", self.parameters.start_frequency));
        });

        ui.add_space(6.0);

        ui.horizontal(|ui| {
            ui.label(
                RichText::new("End Frequency (Hz):")
                    .size(12.0)
                    .color(colors.text_secondary),
            );
            ui.add_space(10.0);

            let end_response = self.end_freq_slider.show(ui, colors);
            if end_response.changed() {
                self.parameters.end_frequency = self.end_freq_slider.value();
                self.parameters.validate_sweep_frequencies();
            }

            ui.label(format!("{:.0} Hz", self.parameters.end_frequency));
        });
    }

    fn draw_impulse_parameters(&mut self, ui: &mut Ui, colors: &ThemeColors) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Delay (s):")
                    .size(12.0)
                    .color(colors.text_secondary),
            );
            ui.add_space(10.0);

            let phase_response = self.phase_slider.show(ui, colors);
            if phase_response.changed() {
                self.parameters.phase = self.phase_slider.value();
            }

            ui.label(format!("{:.3} seconds", self.parameters.phase));
        });
    }

    fn draw_multitone_parameters(&mut self, ui: &mut Ui, colors: &ThemeColors) {
        ui.label(
            RichText::new("Multi-tone: 1kHz + 2kHz + 3kHz harmonics")
                .size(12.0)
                .color(colors.text_secondary),
        );
    }

    fn draw_advanced_parameters(&mut self, ui: &mut Ui, colors: &ThemeColors) {
        ui.label(
            RichText::new("Advanced Parameters")
                .size(14.0)
                .color(colors.text),
        );
        ui.add_space(6.0);

        // Sample rate selection
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Sample Rate:")
                    .size(12.0)
                    .color(colors.text_secondary),
            );
            ui.add_space(10.0);

            ComboBox::from_id_source("sample_rate")
                .selected_text(format!("{} Hz", self.parameters.sample_rate as i32))
                .show_ui(ui, |ui| {
                    for &sr in &[44100.0, 48000.0, 96000.0, 192000.0] {
                        ui.selectable_value(
                            &mut self.parameters.sample_rate,
                            sr,
                            format!("{} Hz", sr as i32),
                        );
                    }
                });
        });

        ui.add_space(6.0);

        // Phase control (for applicable signals)
        if matches!(
            self.signal_type,
            SignalType::Sine | SignalType::Square | SignalType::Sawtooth
        ) {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Phase (radians):")
                        .size(12.0)
                        .color(colors.text_secondary),
                );
                ui.add_space(10.0);

                let phase_response = self.phase_slider.show(ui, colors);
                if phase_response.changed() {
                    self.parameters.phase = self.phase_slider.value();
                }

                ui.label(format!(
                    "{:.2} rad ({:.0}°)",
                    self.parameters.phase,
                    self.parameters.phase.to_degrees()
                ));
            });
        }
    }

    fn draw_control_buttons(&mut self, ui: &mut Ui, colors: &ThemeColors) {
        ui.horizontal_centered(|ui| {
            // Generate button
            let generate_text = match self.state {
                GeneratorState::Generating => "⏳ Generating...",
                _ => "🔧 Generate",
            };

            let mut generate_button = EnhancedButton::new(generate_text).style(ButtonStyle {
                gradient: true,
                glow: self.state == GeneratorState::Generating,
                ..Default::default()
            });

            if generate_button.show(ui, colors).clicked()
                && self.state != GeneratorState::Generating
            {
                self.generate_signal();
            }

            ui.add_space(15.0);

            // Play/Pause button
            let play_text = match self.state {
                GeneratorState::Playing => "⏸️ Pause",
                _ => "▶️ Play",
            };

            let mut play_button = EnhancedButton::new(play_text).style(ButtonStyle {
                gradient: true,
                glow: self.state == GeneratorState::Playing,
                ..Default::default()
            });

            if play_button.show(ui, colors).clicked() {
                self.toggle_playback();
            }

            ui.add_space(15.0);

            // Stop button
            let mut stop_button = EnhancedButton::new("⏹️ Stop");
            if stop_button.show(ui, colors).clicked() {
                self.stop_playback();
            }

            ui.add_space(15.0);

            // Preview toggle
            let preview_text = if self.preview_enabled {
                "👁️ Preview: On"
            } else {
                "👁️ Preview: Off"
            };
            let mut preview_button = EnhancedButton::new(preview_text).style(ButtonStyle {
                gradient: self.preview_enabled,
                ..Default::default()
            });

            if preview_button.show(ui, colors).clicked() {
                self.preview_enabled = !self.preview_enabled;
            }
        });
    }

    fn draw_waveform_preview(&mut self, ui: &mut Ui, colors: &ThemeColors) {
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Waveform Preview")
                            .size(14.0)
                            .color(colors.text),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("📊 Spectrum").clicked() {
                            self.spectrum_analysis_enabled = !self.spectrum_analysis_enabled;
                        }
                    });
                });

                ui.separator();
                ui.add_space(4.0);

                let (rect, _) =
                    ui.allocate_exact_size(Vec2::new(ui.available_width(), 120.0), Sense::hover());
                self.draw_waveform(ui, rect, colors);
            });
        });
    }

    fn draw_spectrum_analysis(&mut self, ui: &mut Ui, colors: &ThemeColors) {
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label(
                    RichText::new("Spectrum Analysis")
                        .size(14.0)
                        .color(colors.text),
                );
                ui.separator();
                ui.add_space(4.0);

                let (rect, _) =
                    ui.allocate_exact_size(Vec2::new(ui.available_width(), 100.0), Sense::hover());
                self.draw_spectrum(ui, rect, colors);
            });
        });
    }

    fn draw_mathematical_verification(&mut self, ui: &mut Ui, colors: &ThemeColors) {
        if !self.generated_samples.is_empty() {
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("Mathematical Verification")
                            .size(14.0)
                            .color(colors.text),
                    );
                    ui.separator();
                    ui.add_space(4.0);

                    let verification_results = self.perform_mathematical_verification();

                    ui.horizontal(|ui| {
                        ui.label("Samples Generated:");
                        ui.label(
                            RichText::new(format!("{}", self.generated_samples.len()))
                                .color(colors.accent),
                        );
                    });

                    ui.horizontal(|ui| {
                        ui.label("RMS Level:");
                        ui.label(
                            RichText::new(format!("{:.4}", verification_results.rms))
                                .color(colors.accent),
                        );
                    });

                    ui.horizontal(|ui| {
                        ui.label("Peak Level:");
                        ui.label(
                            RichText::new(format!("{:.4}", verification_results.peak))
                                .color(colors.accent),
                        );
                    });

                    ui.horizontal(|ui| {
                        ui.label("DC Offset:");
                        ui.label(
                            RichText::new(format!("{:.6}", verification_results.dc_offset))
                                .color(colors.accent),
                        );
                    });

                    if let Some(thd) = verification_results.thd {
                        ui.horizontal(|ui| {
                            ui.label("THD+N:");
                            ui.label(
                                RichText::new(format!("{:.2}%", thd * 100.0)).color(colors.accent),
                            );
                        });
                    }
                });
            });
        }
    }

    fn draw_waveform(&self, ui: &Ui, rect: Rect, colors: &ThemeColors) {
        let painter = ui.painter();

        // Draw background
        painter.rect_filled(rect, 4.0, ColorUtils::with_alpha(colors.surface, 0.8));

        if self.waveform_samples.is_empty() {
            // Draw placeholder
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "Generate a signal to see waveform preview",
                egui::FontId::proportional(12.0),
                colors.text_secondary,
            );
            return;
        }

        // Draw waveform
        let samples_to_show = self.waveform_samples.len().min(1024); // Show max 1024 samples for performance
        let step = self.waveform_samples.len() / samples_to_show.max(1);

        let mut points = Vec::with_capacity(samples_to_show);
        for i in 0..samples_to_show {
            let sample_idx = i * step;
            if sample_idx < self.waveform_samples.len() {
                let x = rect.min.x + (i as f32 / (samples_to_show - 1) as f32) * rect.width();
                let y = rect.center().y - self.waveform_samples[sample_idx] * rect.height() * 0.4;
                points.push(Pos2::new(x, y));
            }
        }

        if points.len() > 1 {
            // Draw waveform line
            painter.add(egui::Shape::line(points, Stroke::new(1.5, colors.primary)));
        }

        // Draw zero line
        let zero_y = rect.center().y;
        painter.line_segment(
            [Pos2::new(rect.min.x, zero_y), Pos2::new(rect.max.x, zero_y)],
            (1.0, ColorUtils::with_alpha(colors.text_secondary, 0.3)),
        );

        // Draw border
        painter.rect_stroke(
            rect,
            4.0,
            Stroke::new(1.0, ColorUtils::with_alpha(colors.text_secondary, 0.2)),
            egui::epaint::StrokeKind::Outside,
        );
    }

    fn draw_spectrum(&self, ui: &Ui, rect: Rect, colors: &ThemeColors) {
        let painter = ui.painter();

        // Draw background
        painter.rect_filled(rect, 4.0, ColorUtils::with_alpha(colors.surface, 0.8));

        if self.spectrum_data.is_empty() {
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "Spectrum analysis will appear here",
                egui::FontId::proportional(12.0),
                colors.text_secondary,
            );
            return;
        }

        // Draw spectrum bars
        let num_bars = self.spectrum_data.len().min(256);
        let bar_width = rect.width() / num_bars as f32;

        for i in 0..num_bars {
            let magnitude = self.spectrum_data[i];
            let bar_height = magnitude * rect.height();

            let bar_rect = Rect::from_min_size(
                Pos2::new(rect.min.x + i as f32 * bar_width, rect.max.y - bar_height),
                Vec2::new(bar_width - 1.0, bar_height),
            );

            let color = ColorUtils::lerp_color32(colors.primary, colors.accent, magnitude);
            painter.rect_filled(bar_rect, 0.0, color);
        }

        // Draw border
        painter.rect_stroke(
            rect,
            4.0,
            Stroke::new(1.0, ColorUtils::with_alpha(colors.text_secondary, 0.2)),
            egui::epaint::StrokeKind::Outside,
        );
    }

    fn update_preview(&mut self) {
        if self.state == GeneratorState::Generating {
            return;
        }

        // Generate a short preview (100ms max)
        let preview_duration = self.parameters.duration.min(0.1);
        let samples = self.generate_samples(preview_duration);

        self.waveform_samples = samples;

        if self.spectrum_analysis_enabled {
            self.update_spectrum_analysis();
        }
    }

    fn update_spectrum_analysis(&mut self) {
        use rustfft::{num_complex::Complex, FftPlanner};

        if self.waveform_samples.is_empty() {
            return;
        }

        let fft_size = 1024.min(self.waveform_samples.len());
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(fft_size);

        let mut buffer: Vec<Complex<f32>> = self.waveform_samples[..fft_size]
            .iter()
            .map(|&x| Complex::new(x, 0.0))
            .collect();

        fft.process(&mut buffer);

        // Calculate magnitudes and convert to dB
        self.spectrum_data = buffer[..fft_size / 2]
            .iter()
            .map(|c| {
                let magnitude = c.norm();
                let db = 20.0 * magnitude.log10().max(-120.0);
                (db + 120.0) / 120.0 // Normalize to 0-1 range
            })
            .collect();
    }

    pub fn generate_signal(&mut self) {
        if self.state == GeneratorState::Generating {
            return;
        }

        self.state = GeneratorState::Generating;
        self.generation_animation.set_target(1.0);

        // Generate the full signal
        self.generated_samples = self.generate_samples(self.parameters.duration);

        // Update preview
        if self.preview_enabled {
            self.update_preview();
        }

        self.state = GeneratorState::Stopped;
        self.generation_animation.set_target(0.0);
    }

    fn generate_samples(&self, duration: f32) -> Vec<f32> {
        let amplitude = self.parameters.amplitude();

        match self.signal_type {
            SignalType::Sine => {
                let generator = SineGenerator::new(self.parameters.frequency)
                    .with_amplitude(amplitude)
                    .with_phase(self.parameters.phase);
                generator.generate(duration, self.parameters.sample_rate)
            }
            SignalType::WhiteNoise => {
                let generator = WhiteNoiseGenerator::new()
                    .with_amplitude(amplitude)
                    .with_seed(self.parameters.seed);
                generator.generate(duration, self.parameters.sample_rate)
            }
            SignalType::PinkNoise => {
                let generator = PinkNoiseGenerator::new()
                    .with_amplitude(amplitude)
                    .with_seed(self.parameters.seed);
                generator.generate(duration, self.parameters.sample_rate)
            }
            SignalType::Square => {
                let generator = SquareGenerator::new(self.parameters.frequency)
                    .with_amplitude(amplitude)
                    .with_duty_cycle(self.parameters.duty_cycle);
                generator.generate(duration, self.parameters.sample_rate)
            }
            SignalType::Sawtooth => {
                let generator =
                    SawtoothGenerator::new(self.parameters.frequency).with_amplitude(amplitude);
                generator.generate(duration, self.parameters.sample_rate)
            }
            SignalType::Sweep => {
                let generator = SweepGenerator::new(
                    self.parameters.start_frequency,
                    self.parameters.end_frequency,
                )
                .with_amplitude(amplitude);
                generator.generate(duration, self.parameters.sample_rate)
            }
            SignalType::Impulse => {
                let generator = ImpulseGenerator::new()
                    .with_amplitude(amplitude)
                    .with_delay(self.parameters.phase); // Using phase as delay
                generator.generate(duration, self.parameters.sample_rate)
            }
            SignalType::MultiTone => {
                let generator = MultiToneGenerator::new(vec![1000.0, 2000.0, 3000.0])
                    .with_amplitudes(vec![amplitude, amplitude * 0.5, amplitude * 0.33]);
                generator.generate(duration, self.parameters.sample_rate)
            }
        }
    }

    pub fn toggle_playback(&mut self) {
        match self.state {
            GeneratorState::Playing => self.pause_playback(),
            _ => self.start_playback(),
        }
    }

    fn start_playback(&mut self) {
        if self.generated_samples.is_empty() {
            self.generate_signal();
        }
        self.state = GeneratorState::Playing;
    }

    fn pause_playback(&mut self) {
        self.state = GeneratorState::Stopped;
    }

    pub fn stop_playback(&mut self) {
        self.state = GeneratorState::Stopped;
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(source) = &mut self.source_node {
                source.stop();
            }
            self.source_node = None;
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn create_audio_buffer(
        &self,
        audio_context: &AudioContext,
    ) -> Option<web_audio_api::AudioBuffer> {
        if self.generated_samples.is_empty() {
            return None;
        }

        let mut buffer = audio_context.create_buffer(
            1, // mono
            self.generated_samples.len(),
            self.parameters.sample_rate,
        );

        buffer.copy_to_channel(&self.generated_samples, 0);
        Some(buffer)
    }

    fn perform_mathematical_verification(&self) -> VerificationResults {
        if self.generated_samples.is_empty() {
            return VerificationResults::default();
        }

        let samples = &self.generated_samples;
        let n = samples.len() as f32;

        // Calculate RMS
        let rms = (samples.iter().map(|&x| x * x).sum::<f32>() / n).sqrt();

        // Calculate peak
        let peak = samples.iter().map(|&x| x.abs()).fold(0.0, f32::max);

        // Calculate DC offset
        let dc_offset = samples.iter().sum::<f32>() / n;

        // Calculate THD+N for sine waves
        let thd = if matches!(self.signal_type, SignalType::Sine) {
            Some(self.calculate_thd())
        } else {
            None
        };

        VerificationResults {
            rms,
            peak,
            dc_offset,
            thd,
        }
    }

    fn calculate_thd(&self) -> f32 {
        // Simplified THD calculation using FFT
        use rustfft::{num_complex::Complex, FftPlanner};

        if self.generated_samples.len() < 1024 {
            return 0.0;
        }

        let fft_size = 1024;
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(fft_size);

        let mut buffer: Vec<Complex<f32>> = self.generated_samples[..fft_size]
            .iter()
            .map(|&x| Complex::new(x, 0.0))
            .collect();

        fft.process(&mut buffer);

        // Find fundamental frequency bin
        let fundamental_bin =
            (self.parameters.frequency * fft_size as f32 / self.parameters.sample_rate) as usize;
        let fundamental_power = buffer[fundamental_bin].norm_sqr();

        // Calculate harmonic power
        let mut harmonic_power = 0.0;
        for harmonic in 2..=5 {
            let harmonic_bin = fundamental_bin * harmonic;
            if harmonic_bin < buffer.len() / 2 {
                harmonic_power += buffer[harmonic_bin].norm_sqr();
            }
        }

        if fundamental_power > 0.0 {
            (harmonic_power / fundamental_power).sqrt()
        } else {
            0.0
        }
    }
}

#[derive(Debug, Clone, Default)]
struct VerificationResults {
    rms: f32,
    peak: f32,
    dc_offset: f32,
    thd: Option<f32>,
}

impl Default for SignalGeneratorPanel {
    fn default() -> Self {
        Self::new()
    }
}
