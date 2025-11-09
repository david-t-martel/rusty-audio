// Phase 2.1: PanelContent trait implementation for dock layout
// This file contains the implementation of panel rendering for the docking system

use super::*;

impl PanelContent for AudioPlayerApp {
    fn show_file_browser(&mut self, ui: &mut egui::Ui) {
        let colors = self.theme_manager.get_colors();
        let colors = self.accessibility_manager.get_accessible_colors(&colors);

        ui.vertical(|ui| {
            ui.heading(RichText::new("📁 File Browser").color(colors.text));
            ui.add_space(10.0);

            if ui.button("Open Audio File...").clicked() {
                self.open_file_dialog();
            }

            ui.add_space(10.0);

            if let Some(file) = &self.current_file {
                ui.label(RichText::new("Current File:").color(colors.text));
                ui.label(
                    RichText::new(file.file_name())
                        .color(colors.accent)
                        .strong(),
                );
            } else {
                ui.label(RichText::new("No file loaded").color(colors.text_secondary));
            }
        });
    }
    fn show_waveform(&mut self, ui: &mut egui::Ui) {
        let colors = self.theme_manager.get_colors();
        let colors = self.accessibility_manager.get_accessible_colors(&colors);

        ui.vertical(|ui| {
            ui.heading(RichText::new("🌊 Waveform Display").color(colors.text));
            ui.add_space(10.0);

            // Enhanced progress bar serves as waveform placeholder
            ui.allocate_ui(egui::Vec2::new(ui.available_width(), 60.0), |ui| {
                let progress_response = self.progress_bar.show(ui, &colors);
                if progress_response.changed() {
                    let position_seconds =
                        self.progress_bar.progress * self.total_duration.as_secs_f32();
                    self.seek_to_position_main(position_seconds);
                }
            });

            ui.add_space(10.0);

            // Playback time display
            let current_time = format_duration(self.playback_pos);
            let total_time = format_duration(self.total_duration);
            ui.label(
                RichText::new(format!("{} / {}", current_time, total_time)).color(colors.text),
            );
        });
    }
    fn show_spectrum(&mut self, ui: &mut egui::Ui) {
        let colors = self.theme_manager.get_colors();
        let colors = self.accessibility_manager.get_accessible_colors(&colors);

        ui.vertical(|ui| {
            ui.heading(RichText::new("📊 Spectrum Analyzer").color(colors.text));
            ui.add_space(10.0);

            // Spectrum visualizer
            let spectrum_rect = ui.available_rect_before_wrap();
            self.spectrum_visualizer.draw(ui, spectrum_rect, &colors);

            ui.add_space(10.0);

            // Spectrum mode controls
            ui.horizontal(|ui| {
                ui.label("Mode:");
                let current_mode = self.spectrum_visualizer.config().mode.clone();
                egui::ComboBox::from_label("")
                    .selected_text(format!("{:?}", current_mode))
                    .show_ui(ui, |ui| {
                        for mode in &[SpectrumMode::Bars, SpectrumMode::Line] {
                            if ui
                                .selectable_value(
                                    &mut self.spectrum_visualizer.config_mut().mode,
                                    mode.clone(),
                                    format!("{:?}", mode),
                                )
                                .clicked()
                            {
                                // Mode changed
                            }
                        }
                    });
            });
        });
    }
    fn show_generator(&mut self, ui: &mut egui::Ui) {
        let colors = self.theme_manager.get_colors();
        let colors = self.accessibility_manager.get_accessible_colors(&colors);

        // Use the existing signal generator panel's show method
        self.signal_generator_panel.show(ui, &colors);
    }

    fn show_effects(&mut self, ui: &mut egui::Ui) {
        let colors = self.theme_manager.get_colors();
        let colors = self.accessibility_manager.get_accessible_colors(&colors);

        ui.vertical(|ui| {
            ui.heading(RichText::new("🎛️ Effects").color(colors.text));
            ui.add_space(10.0);

            ui.label(RichText::new("Panning Control").color(colors.text));
            ui.horizontal(|ui| {
                ui.label("L");
                let panning_slider =
                    egui::Slider::new(&mut self.panning, 0.0..=1.0).show_value(false);
                if ui.add(panning_slider).changed() {
                    // Update panning (would need stereo panner node)
                }
                ui.label("R");
            });

            ui.add_space(15.0);

            ui.label(
                RichText::new("Future effects will be added here:").color(colors.text_secondary),
            );
            ui.label("• Reverb");
            ui.label("• Delay");
            ui.label("• Compression");
        });
    }
    fn show_equalizer(&mut self, ui: &mut egui::Ui) {
        let colors = self.theme_manager.get_colors();
        let colors = self.accessibility_manager.get_accessible_colors(&colors);

        ui.vertical(|ui| {
            ui.heading(RichText::new("🎚️ Equalizer").color(colors.text));
            ui.add_space(10.0);

            // EQ bands in a horizontal layout
            ui.horizontal_wrapped(|ui| {
                for (i, knob) in self.accessible_eq_knobs.iter_mut().enumerate() {
                    ui.vertical(|ui| {
                        let response = knob.show(ui, &colors, &mut self.accessibility_manager);
                        if response.changed() {
                            if let Some(band) = self.eq_bands.get(i) {
                                band.gain().set_value(knob.value());
                            }
                        }
                    });
                }
            });

            ui.add_space(15.0);

            if ui.button("Reset All Bands").clicked() {
                for (band, knob) in self
                    .eq_bands
                    .iter_mut()
                    .zip(self.accessible_eq_knobs.iter_mut())
                {
                    band.gain().set_value(0.0);
                    knob.set_value(0.0);
                }
            }
        });
    }
    fn show_inspector(&mut self, ui: &mut egui::Ui) {
        let colors = self.theme_manager.get_colors();
        let colors = self.accessibility_manager.get_accessible_colors(&colors);

        ui.vertical(|ui| {
            ui.heading(RichText::new("🔍 Inspector").color(colors.text));
            ui.add_space(10.0);

            ui.group(|ui| {
                ui.label(RichText::new("Audio Info").color(colors.accent).strong());
                ui.separator();
                ui.label(format!(
                    "Sample Rate: {} Hz",
                    self.audio_context.sample_rate()
                ));
                ui.label(format!("Playback State: {:?}", self.playback_state));
                ui.label(format!("Volume: {:.1}%", self.volume * 100.0));
                ui.label(format!(
                    "Loop: {}",
                    if self.is_looping { "On" } else { "Off" }
                ));
            });

            ui.add_space(10.0);

            ui.group(|ui| {
                ui.label(RichText::new("Performance").color(colors.accent).strong());
                ui.separator();
                ui.label(format!("FPS: ~60"));
                ui.label(format!("Spectrum Size: {} bins", self.spectrum.len()));
            });
        });
    }

    fn show_mixer(&mut self, ui: &mut egui::Ui) {
        let colors = self.theme_manager.get_colors();
        let colors = self.accessibility_manager.get_accessible_colors(&colors);

        ui.vertical(|ui| {
            ui.heading(RichText::new("🎚️ Mixer").color(colors.text));
            ui.add_space(10.0);

            // Master volume control
            ui.group(|ui| {
                ui.label(RichText::new("Master Volume").color(colors.accent).strong());
                ui.add_space(5.0);

                let volume_response = self.accessible_volume_slider.show(
                    ui,
                    &colors,
                    &mut self.accessibility_manager,
                );
                if volume_response.changed() {
                    self.volume = self.accessible_volume_slider.value();
                    self.gain_node.gain().set_value(self.volume);

                    if !self.accessibility_manager.is_volume_safe(self.volume) {
                        self.accessibility_manager.announce(
                            "Warning: Volume level may be harmful to hearing".to_string(),
                            ui::accessibility::AnnouncementPriority::High,
                        );
                    }
                }

                ui.label(format!("{:.0}%", self.volume * 100.0));
            });

            ui.add_space(15.0);

            // Volume safety indicator
            self.volume_safety_indicator.show(ui, &colors);
        });
    }
    fn show_transport(&mut self, ui: &mut egui::Ui) {
        let colors = self.theme_manager.get_colors();
        let colors = self.accessibility_manager.get_accessible_colors(&colors);

        ui.vertical_centered(|ui| {
            ui.heading(RichText::new("⏯️ Transport Controls").color(colors.text));
            ui.add_space(10.0);

            // Album art
            ui.add_space(5.0);
            self.album_art_display.show(ui, &colors);
            ui.add_space(10.0);

            // Metadata
            self.metadata_display.show(ui, &colors);
            ui.add_space(15.0);

            // Transport buttons
            ui.horizontal_centered(|ui| {
                let button_size = egui::Vec2::new(60.0, 35.0);

                if ui
                    .add_sized(button_size, egui::Button::new("📁 Open"))
                    .clicked()
                {
                    self.open_file_dialog();
                }

                ui.add_space(5.0);

                let play_pause_text = if self.playback_state == PlaybackState::Playing {
                    "⏸️ Pause"
                } else {
                    "▶️ Play"
                };
                if ui
                    .add_sized(button_size, egui::Button::new(play_pause_text))
                    .clicked()
                {
                    self.play_pause_main();
                }

                ui.add_space(5.0);

                if ui
                    .add_sized(button_size, egui::Button::new("⏹️ Stop"))
                    .clicked()
                {
                    self.stop_playback_main();
                }
            });

            ui.add_space(10.0);

            // Loop control
            let loop_text = if self.is_looping {
                "🔁 Loop: On"
            } else {
                "🔁 Loop: Off"
            };
            if ui.button(loop_text).clicked() {
                self.is_looping = !self.is_looping;
            }
        });
    }

    fn show_settings(&mut self, ui: &mut egui::Ui) {
        let colors = self.theme_manager.get_colors();
        let colors = self.accessibility_manager.get_accessible_colors(&colors);

        self.draw_settings_panel_main(ui, &colors);
    }
}

// Helper function to format duration
fn format_duration(duration: std::time::Duration) -> String {
    let total_seconds = duration.as_secs();
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    format!("{:02}:{:02}", minutes, seconds)
}
