use crate::ui::{
    theme::ThemeColors,
    spectrum::{SpectrumMode, SpectrumVisualizer, SpectrumVisualizerConfig},
    controls::CircularKnob,
};
use crate::{AudioPlayerApp, PlaybackState};
use egui::{Ui, Vec2, RichText, Context};
use std::time::Duration;
use web_audio_api::param::AudioParam;
use web_audio_api::node::{AudioScheduledSourceNode, AudioNode};
use web_audio_api::context::BaseAudioContext;
use lofty::file::TaggedFileExt;
use lofty::tag::Accessor;
use image::GenericImageView;

impl AudioPlayerApp {
    pub fn draw_effects_panel(&mut self, ui: &mut Ui, colors: &ThemeColors) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.heading("🎛️ Effects & Visualization");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Spectrum mode selector
                    egui::ComboBox::from_label("Mode")
                        .selected_text(self.spectrum_visualizer.config().mode.display_name())
                        .show_ui(ui, |ui| {
                            for mode in SpectrumMode::all() {
                                ui.selectable_value(
                                    &mut self.spectrum_visualizer.config_mut().mode,
                                    mode.clone(),
                                    mode.display_name()
                                );
                            }
                        });
                });
            });

            ui.separator();

            // Enhanced spectrum visualizer
            let spectrum_rect = egui::Rect::from_min_size(
                ui.cursor().min,
                Vec2::new(ui.available_width(), 300.0)
            );
            self.spectrum_visualizer.draw(ui, spectrum_rect, colors);

            ui.add_space(20.0);

            // Spectrum controls
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label("Smoothing");
                    ui.add(egui::Slider::new(
                        &mut self.spectrum_visualizer.config_mut().smoothing,
                        0.0..=1.0
                    ).show_value(true));
                });

                ui.add_space(20.0);

                ui.vertical(|ui| {
                    ui.label("Bars");
                    ui.add(egui::Slider::new(
                        &mut self.spectrum_visualizer.config_mut().num_bars,
                        16..=128
                    ).show_value(true));
                });

                ui.add_space(20.0);

                ui.vertical(|ui| {
                    ui.checkbox(&mut self.spectrum_visualizer.config_mut().gradient_enabled, "Gradient");
                    ui.checkbox(&mut self.spectrum_visualizer.config_mut().glow_enabled, "Glow");
                    ui.checkbox(&mut self.spectrum_visualizer.config_mut().mirror_enabled, "Mirror");
                });
            });
        });
    }

    pub fn draw_mobile_effects_panel(&mut self, ui: &mut Ui, colors: &ThemeColors) {
        ui.vertical(|ui| {
            ui.heading("🎛️ Effects");

            // Compact spectrum visualizer
            let spectrum_rect = egui::Rect::from_min_size(
                ui.cursor().min,
                Vec2::new(ui.available_width(), 200.0)
            );
            self.spectrum_visualizer.draw(ui, spectrum_rect, colors);

            ui.add_space(10.0);

            // Simple controls
            ui.horizontal(|ui| {
                egui::ComboBox::from_label("Mode")
                    .selected_text(self.spectrum_visualizer.config().mode.display_name())
                    .show_ui(ui, |ui| {
                        for mode in SpectrumMode::all() {
                            ui.selectable_value(
                                &mut self.spectrum_visualizer.config_mut().mode,
                                mode.clone(),
                                mode.display_name()
                            );
                        }
                    });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.checkbox(&mut self.spectrum_visualizer.config_mut().gradient_enabled, "Gradient");
                });
            });
        });
    }

    pub fn draw_eq_panel(&mut self, ui: &mut Ui, colors: &ThemeColors) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.heading("📊 Equalizer");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Reset All").clicked() {
                        self.reset_equalizer();
                    }
                });
            });

            ui.separator();

            ui.add_space(10.0);

            // EQ bands with circular knobs
            ui.horizontal_centered(|ui| {
                let eq_count = self.eq_bands.len();
                for i in 0..eq_count {
                    ui.vertical_centered(|ui| {
                        // Frequency label
                        let freq = 60.0 * 2.0_f32.powi(i as i32);
                        let freq_label = if freq < 1000.0 {
                            format!("{:.0} Hz", freq)
                        } else {
                            format!("{:.1}k Hz", freq / 1000.0)
                        };
                        ui.label(RichText::new(freq_label).size(11.0));

                        // Circular knob control
                        let knob_response = self.eq_knobs[i].show(ui, colors);
                        if knob_response.changed() {
                            self.eq_bands[i].gain().set_value(self.eq_knobs[i].value());
                        }

                        // Gain value display
                        ui.label(RichText::new(format!("{:.1} dB", self.eq_knobs[i].value())).size(10.0));
                    });

                    if i < eq_count - 1 {
                        ui.add_space(8.0);
                    }
                }
            });

            ui.add_space(20.0);

            // Master gain and additional controls
            ui.horizontal_centered(|ui| {
                ui.vertical(|ui| {
                    ui.label("Master Gain");
                    let mut master_gain = self.gain_node.gain().value();
                    if ui.add(egui::Slider::new(&mut master_gain, 0.0..=2.0)
                        .show_value(true)
                        .suffix(" dB")).changed() {
                        self.gain_node.gain().set_value(master_gain);
                    }
                });

                ui.add_space(20.0);

                ui.vertical(|ui| {
                    ui.label("EQ Presets");
                    egui::ComboBox::from_label("")
                        .selected_text("Custom")
                        .show_ui(ui, |ui| {
                            if ui.button("Flat").clicked() { self.apply_eq_preset_flat(); }
                            if ui.button("Bass Boost").clicked() { self.apply_eq_preset_bass_boost(); }
                            if ui.button("Treble Boost").clicked() { self.apply_eq_preset_treble_boost(); }
                            if ui.button("Vocal").clicked() { self.apply_eq_preset_vocal(); }
                            if ui.button("Electronic").clicked() { self.apply_eq_preset_electronic(); }
                        });
                });
            });
        });
    }

    pub fn draw_mobile_eq_panel(&mut self, ui: &mut Ui, colors: &ThemeColors) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.heading("📊 EQ");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Reset").clicked() {
                        self.reset_equalizer();
                    }
                });
            });

            ui.separator();

            // Compact EQ knobs
            ui.horizontal_wrapped(|ui| {
                for (i, (band, knob)) in self.eq_bands.iter_mut().zip(self.eq_knobs.iter_mut()).enumerate() {
                    ui.vertical_centered(|ui| {
                        let freq = 60.0 * 2.0_f32.powi(i as i32);
                        let freq_label = if freq < 1000.0 {
                            format!("{:.0}", freq)
                        } else {
                            format!("{:.1}k", freq / 1000.0)
                        };
                        ui.label(RichText::new(freq_label).size(10.0));

                        // Smaller knobs for mobile
                        // Note: radius is set during construction
                        let knob_response = knob.show(ui, colors);
                        if knob_response.changed() {
                            band.gain().set_value(knob.value());
                        }
                    });
                }
            });

            ui.add_space(10.0);

            // Preset selector
            egui::ComboBox::from_label("Presets")
                .selected_text("Custom")
                .show_ui(ui, |ui| {
                    if ui.button("Flat").clicked() { self.apply_eq_preset_flat(); }
                    if ui.button("Bass").clicked() { self.apply_eq_preset_bass_boost(); }
                    if ui.button("Treble").clicked() { self.apply_eq_preset_treble_boost(); }
                    if ui.button("Vocal").clicked() { self.apply_eq_preset_vocal(); }
                });
        });
    }

    pub fn draw_settings_panel(&mut self, ui: &mut Ui, colors: &ThemeColors) {
        ui.vertical(|ui| {
            ui.heading("⚙️ Settings");
            ui.separator();

            ui.add_space(10.0);

            // Theme settings
            ui.horizontal(|ui| {
                ui.label("🎨 Theme:");
                egui::ComboBox::from_label("")
                    .selected_text(self.theme_manager.current_theme().display_name())
                    .show_ui(ui, |ui| {
                        for theme in crate::ui::theme::Theme::all() {
                            let mut current_theme = self.theme_manager.current_theme().clone();
                            if ui.selectable_value(&mut current_theme, theme.clone(), theme.display_name()).clicked() {
                                self.theme_manager.set_theme(theme);
                            }
                        }
                    });
            });

            ui.add_space(15.0);

            // Audio settings
            ui.heading("🔊 Audio");
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Buffer Size:");
                ui.label("(Fixed by web-audio-api)");
            });

            ui.horizontal(|ui| {
                ui.label("Sample Rate:");
                ui.label(format!("{} Hz", self.audio_context.sample_rate()));
            });

            ui.add_space(15.0);

            // Visualization settings
            ui.heading("📊 Visualization");
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Update Rate:");
                ui.add(egui::Slider::new(
                    &mut self.spectrum_visualizer.config_mut().update_rate,
                    30.0..=120.0
                ).suffix(" Hz"));
            });

            ui.horizontal(|ui| {
                ui.label("FFT Size:");
                ui.label(format!("{}", self.spectrum_processor.fft_size()));
                ui.label("(Fixed)");
            });

            ui.add_space(15.0);

            // Performance info
            ui.heading("📈 Performance");
            ui.separator();

            let fps = 1.0 / self.last_frame_time.elapsed().as_secs_f32().max(0.001);
            ui.horizontal(|ui| {
                ui.label("Frame Rate:");
                ui.label(format!("{:.1} FPS", fps));
            });

            ui.horizontal(|ui| {
                ui.label("Screen Size:");
                ui.label(format!("{:?}", self.screen_size));
            });

            ui.add_space(15.0);

            // About section
            ui.heading("ℹ️ About");
            ui.separator();

            ui.label("Rusty Audio Player");
            ui.label("Built with Rust, egui, and web-audio-api");
            ui.label("© 2024");

            if ui.button("Show Keyboard Shortcuts").clicked() {
                self.show_keyboard_shortcuts = !self.show_keyboard_shortcuts;
            }
        });
    }

    pub fn draw_keyboard_shortcuts_overlay(&mut self, ctx: &Context, colors: &ThemeColors) {
        egui::Window::new("⌨️ Keyboard Shortcuts")
            .collapsible(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("Playback Controls").strong());
                    ui.label("Space - Play/Pause");
                    ui.label("S - Stop");
                    ui.label("L - Toggle Loop");

                    ui.add_space(10.0);

                    ui.label(RichText::new("Volume & Seeking").strong());
                    ui.label("↑/↓ - Volume Up/Down");
                    ui.label("←/→ - Seek -5s/+5s");

                    ui.add_space(10.0);

                    ui.label(RichText::new("Interface").strong());
                    ui.label("? - Show/Hide This Help");
                    ui.label("Esc - Close Dialogs");

                    ui.add_space(10.0);

                    if ui.button("Close").clicked() {
                        self.show_keyboard_shortcuts = false;
                    }
                });
            });
    }

    // Enhanced keyboard input handling - replaces the basic version in main.rs
    pub fn handle_keyboard_input_enhanced(&mut self, ctx: &Context) {
        ctx.input(|i| {
            if i.key_pressed(egui::Key::Space) {
                self.play_pause();
            }
            if i.key_pressed(egui::Key::S) {
                self.stop();
            }
            if i.key_pressed(egui::Key::L) {
                self.toggle_loop();
            }
            if i.key_pressed(egui::Key::ArrowUp) {
                self.adjust_volume(0.05);
            }
            if i.key_pressed(egui::Key::ArrowDown) {
                self.adjust_volume(-0.05);
            }
            if i.key_pressed(egui::Key::ArrowLeft) {
                self.seek_relative(-5.0);
            }
            if i.key_pressed(egui::Key::ArrowRight) {
                self.seek_relative(5.0);
            }
            if i.key_pressed(egui::Key::Escape) {
                self.show_keyboard_shortcuts = false;
            }
            if i.key_pressed(egui::Key::F1) ||
               (i.modifiers.ctrl && i.key_pressed(egui::Key::Slash)) ||
               i.key_pressed(egui::Key::Questionmark) {
                self.show_keyboard_shortcuts = !self.show_keyboard_shortcuts;
            }
        });
    }

    // Helper methods for enhanced functionality

    pub fn open_file(&mut self) {
        if let Some(handle) = rfd::FileDialog::new()
            .add_filter("Audio Files", &["mp3", "wav", "flac", "ogg", "m4a", "aac"])
            .add_filter("All Files", &["*"])
            .pick_file()
        {
            let handle = std::sync::Arc::new(rfd::FileHandle::from(handle));
            self.open_file_handle(handle);
        }
    }

    pub fn open_file_handle(&mut self, handle: std::sync::Arc<rfd::FileHandle>) {
        let path = handle.path();

        // Load metadata
        if let Ok(tagged_file) = lofty::read_from_path(path) {
            if let Some(tag) = tagged_file.primary_tag() {
                self.metadata = Some(crate::TrackMetadata {
                    title: tag.title().as_deref().unwrap_or("Unknown Title").into(),
                    artist: tag.artist().as_deref().unwrap_or("Unknown Artist").into(),
                    album: tag.album().as_deref().unwrap_or("Unknown Album").into(),
                    year: tag.year().map(|y| y.to_string()).unwrap_or_else(|| "----".into()),
                });
            }

            // Load album art
            if let Some(picture) = tagged_file.primary_tag().and_then(|t| t.pictures().get(0)) {
                if let Ok(img) = image::load_from_memory(picture.data()) {
                    let (width, height) = img.dimensions();
                    let rgba = img.to_rgba8();
                    let pixels = rgba.into_raw();
                    let image = egui::ColorImage::from_rgba_unmultiplied([width as usize, height as usize], &pixels);
                    // Note: We need access to the egui context to load the texture
                    // This will be handled by the caller
                }
            } else {
                self.album_art = None;
            }
        }

        // Load and decode audio
        if let Ok(file) = std::fs::File::open(path) {
            if let Ok(buffer) = self.audio_context.decode_audio_data_sync(file) {
                self.total_duration = Duration::from_secs_f64(buffer.duration());

                let mut source_node = self.audio_context.create_buffer_source();
                source_node.set_buffer(buffer);

                // Connect audio graph
                source_node.connect(&self.gain_node);
                let mut previous_node: &dyn web_audio_api::node::AudioNode = &self.gain_node;
                for band in &self.eq_bands {
                    previous_node.connect(band);
                    previous_node = band;
                }
                previous_node.connect(&self.analyser);
                self.analyser.connect(&self.audio_context.destination());

                source_node.start();
                self.source_node = Some(source_node);
                self.current_file = Some(handle.clone());
                self.playback_state = PlaybackState::Playing;
                self.playback_pos = Duration::ZERO;
                self.error = None;
            }
        }
    }

    pub fn seek_to_position(&mut self, position_seconds: f32) {
        if let Some(source_node) = &mut self.source_node {
            let new_pos = Duration::from_secs_f32(position_seconds.clamp(0.0, self.total_duration.as_secs_f32()));
            self.playback_pos = new_pos;

            // For proper seeking, we'd need to recreate the source node
            // This is a simplified implementation
            source_node.stop();
            source_node.start_at(self.audio_context.current_time() + new_pos.as_secs_f64());
        }
    }

    pub fn seek_relative(&mut self, seconds: f32) {
        let new_pos = if seconds < 0.0 {
            self.playback_pos.saturating_sub(Duration::from_secs_f32(-seconds))
        } else {
            self.playback_pos.saturating_add(Duration::from_secs_f32(seconds))
        };

        self.seek_to_position(new_pos.as_secs_f32());
    }

    pub fn adjust_volume(&mut self, delta: f32) {
        self.volume = (self.volume + delta).clamp(0.0, 1.0);
        self.gain_node.gain().set_value(self.volume);
        self.volume_slider.set_value(self.volume);
    }

    pub fn toggle_loop(&mut self) {
        self.is_looping = !self.is_looping;
        if let Some(source_node) = &mut self.source_node {
            source_node.set_loop(self.is_looping);
        }
    }

    pub fn reset_equalizer(&mut self) {
        for (band, knob) in self.eq_bands.iter_mut().zip(self.eq_knobs.iter_mut()) {
            band.gain().set_value(0.0);
            knob.set_value(0.0);
        }
    }

    pub fn apply_eq_preset_flat(&mut self) {
        self.reset_equalizer();
    }

    pub fn apply_eq_preset_bass_boost(&mut self) {
        let gains = [6.0, 4.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        for ((band, knob), &gain) in self.eq_bands.iter_mut().zip(self.eq_knobs.iter_mut()).zip(gains.iter()) {
            band.gain().set_value(gain);
            knob.set_value(gain);
        }
    }

    pub fn apply_eq_preset_treble_boost(&mut self) {
        let gains = [0.0, 0.0, 0.0, 0.0, 0.0, 2.0, 4.0, 6.0];
        for ((band, knob), &gain) in self.eq_bands.iter_mut().zip(self.eq_knobs.iter_mut()).zip(gains.iter()) {
            band.gain().set_value(gain);
            knob.set_value(gain);
        }
    }

    pub fn apply_eq_preset_vocal(&mut self) {
        let gains = [-2.0, 0.0, 2.0, 4.0, 4.0, 2.0, 0.0, -1.0];
        for ((band, knob), &gain) in self.eq_bands.iter_mut().zip(self.eq_knobs.iter_mut()).zip(gains.iter()) {
            band.gain().set_value(gain);
            knob.set_value(gain);
        }
    }

    pub fn apply_eq_preset_electronic(&mut self) {
        let gains = [4.0, 2.0, -1.0, 1.0, 0.0, 2.0, 4.0, 5.0];
        for ((band, knob), &gain) in self.eq_bands.iter_mut().zip(self.eq_knobs.iter_mut()).zip(gains.iter()) {
            band.gain().set_value(gain);
            knob.set_value(gain);
        }
    }

    // Audio control methods
    pub fn play_pause(&mut self) {
        match self.playback_state {
            PlaybackState::Playing => {
                self.playback_state = PlaybackState::Paused;
            }
            PlaybackState::Paused | PlaybackState::Stopped => {
                self.playback_state = PlaybackState::Playing;
            }
        }
    }

    pub fn stop(&mut self) {
        self.playback_state = PlaybackState::Stopped;
        self.playback_pos = Duration::ZERO;
    }
}