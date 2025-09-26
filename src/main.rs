use eframe::{egui, NativeOptions};
use egui::{Color32, RichText, Vec2};
use kira::manager::{backend::DefaultBackend, AudioManager, AudioManagerSettings};
use kira::sound::static_sound::{StaticSoundData, StaticSoundSettings};
use lofty::file::TaggedFileExt;
use lofty::tag::Accessor;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use rfd::FileHandle;
use std::sync::Arc;
use std::time::Duration;

// Statically create the audio manager.
static AUDIO_MANAGER: Lazy<
    Option<Arc<Mutex<kira::manager::AudioManager<DefaultBackend>>>>,
> = Lazy::new(|| {
    AudioManager::new(AudioManagerSettings::default())
        .ok()
        .map(|manager| Arc::new(Mutex::new(manager)))
});

struct AudioPlayerApp {
    sound_handle: Option<kira::sound::static_sound::StaticSoundHandle>,
    playback_state: PlaybackState,
    current_file: Option<Arc<FileHandle>>,
    metadata: Option<TrackMetadata>,
    volume: f64,
    is_looping: bool,
    playback_pos: Duration,
    total_duration: Duration,
    is_seeking: bool,
    error: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
enum PlaybackState {
    Stopped,
    Playing,
    Paused,
}

#[derive(Debug, Clone)]
struct TrackMetadata {
    title: String,
    artist: String,
    album: String,
    year: String,
}

impl Default for AudioPlayerApp {
    fn default() -> Self {
        Self {
            sound_handle: None,
            playback_state: PlaybackState::Stopped,
            current_file: None,
            metadata: None,
            volume: 0.5,
            is_looping: false,
            playback_pos: Duration::ZERO,
            total_duration: Duration::ZERO,
            is_seeking: false,
            error: None,
        }
    }
}

impl eframe::App for AudioPlayerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Rusty Audio");
            ui.separator();

            // Display Area
            let filename = self
                .current_file
                .as_ref()
                .map(|p| p.file_name())
                .unwrap_or("No file selected".to_string());

            let metadata = self.metadata.as_ref();
            ui.label(RichText::new(filename).size(20.0));
            ui.label(format!(
                "{} - {}",
                metadata.map_or("----", |m| &m.artist),
                metadata.map_or("----", |m| &m.title)
            ));
            ui.label(format!(
                "{} ({})",
                metadata.map_or("----", |m| &m.album),
                metadata.map_or("----", |m| &m.year)
            ));

            ui.add_space(10.0);

            // Time Display & Playback Slider
            let format_duration = |d: Duration| format!("{:02}:{:02}", d.as_secs() / 60, d.as_secs() % 60);
            ui.horizontal(|ui| {
                ui.label(format_duration(self.playback_pos));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format_duration(self.total_duration));
                });
            });

            let mut playback_slider_pos = if self.total_duration.is_zero() {
                0.0
            } else {
                (self.playback_pos.as_secs_f32() / self.total_duration.as_secs_f32()).clamp(0.0, 1.0)
            };
            if ui.add(egui::Slider::new(&mut playback_slider_pos, 0.0..=1.0).show_value(false)).changed() {
                self.is_seeking = true;
                self.playback_pos = self.total_duration.mul_f32(playback_slider_pos);
                if let Some(sound_handle) = &mut self.sound_handle {
                    let _ = sound_handle.seek_to(self.playback_pos.as_secs_f64());
                }
                self.is_seeking = false;
            }

            ui.add_space(10.0);

            // Control Buttons
            ui.horizontal(|ui| {
                if ui.button("Open").clicked() {
                    self.open_file();
                }

                let play_pause_text = if self.playback_state == PlaybackState::Playing { "⏸" } else { "▶" };
                if ui.button(play_pause_text).clicked() {
                    self.play_pause();
                }

                if ui.button("⏹").clicked() {
                    self.stop();
                }

                let loop_style = if self.is_looping {
                    ui.style_mut().visuals.widgets.active.bg_fill = Color32::from_rgb(0, 128, 255);
                    RichText::new("Loop").color(Color32::WHITE)
                } else {
                    RichText::new("Loop")
                };
                if ui.button(loop_style).clicked() {
                    self.is_looping = !self.is_looping;
                }
            });

            // Volume Control
            ui.horizontal(|ui| {
                ui.label("Vol:");
                if ui.add(egui::Slider::new(&mut self.volume, 0.0..=1.0).show_value(false)).changed() {
                    if let Some(sound_handle) = &mut self.sound_handle {
                        let _ = sound_handle.set_volume(self.volume, Default::default());
                    }
                }
            });

            // Error Display
            if let Some(error) = &self.error {
                ui.label(RichText::new(error).color(Color32::RED));
            }

            // Keyboard shortcuts
            self.handle_keyboard_input(ui);

            // Tick
            ctx.request_repaint_after(Duration::from_millis(250));
            self.tick();
        });
    }
}

impl AudioPlayerApp {
    fn open_file(&mut self) {
        if let Some(handle) = rfd::FileDialog::new()
            .add_filter("Audio Files", &["mp3", "wav", "flac", "ogg"])
            .pick_file()
        {
            let handle = Arc::new(FileHandle::from(handle));
            let path = handle.path();
            if let Ok(tagged_file) = lofty::read_from_path(path) {
                if let Some(tag) = tagged_file.primary_tag() {
                    self.metadata = Some(TrackMetadata {
                        title: tag.title().as_deref().unwrap_or("Unknown Title").into(),
                        artist: tag.artist().as_deref().unwrap_or("Unknown Artist").into(),
                        album: tag.album().as_deref().unwrap_or("Unknown Album").into(),
                        year: tag.year().map(|y| y.to_string()).unwrap_or_else(|| "----".into()),
                    });
                }
            }

            if let Some(manager) = AUDIO_MANAGER.as_ref() {
                if let Ok(sound_data) = StaticSoundData::from_file(path, StaticSoundSettings::default()) {
                    self.total_duration = sound_data.duration();
                    let mut manager = manager.lock();
                    if let Ok(sound_handle) = manager.play(sound_data) {
                        self.sound_handle = Some(sound_handle);
                        self.current_file = Some(handle.clone());
                        self.playback_state = PlaybackState::Playing;
                        self.playback_pos = Duration::ZERO;
                        self.error = None;
                    } else {
                        self.error = Some("Failed to play audio file".to_string());
                    }
                } else {
                    self.error = Some("Failed to decode audio file".to_string());
                }
            } else {
                self.error = Some("Failed to initialize audio device".to_string());
            }
        }
    }

    fn play_pause(&mut self) {
        if let Some(sound_handle) = &mut self.sound_handle {
            match self.playback_state {
                PlaybackState::Playing => {
                    let _ = sound_handle.pause(Default::default());
                    self.playback_state = PlaybackState::Paused;
                }
                PlaybackState::Paused => {
                    let _ = sound_handle.resume(Default::default());
                    self.playback_state = PlaybackState::Playing;
                }
                PlaybackState::Stopped => {
                    if let Some(handle) = self.current_file.clone() {
                        self.open_file_handle(handle);
                    }
                }
            }
        }
    }

    fn stop(&mut self) {
        if let Some(sound_handle) = &mut self.sound_handle {
            let _ = sound_handle.stop(Default::default());
            self.playback_state = PlaybackState::Stopped;
            self.playback_pos = Duration::ZERO;
        }
    }

    fn tick(&mut self) {
        if let Some(sound_handle) = &self.sound_handle {
            if sound_handle.state() == kira::sound::PlaybackState::Stopped {
                if self.is_looping {
                    if let Some(handle) = self.current_file.clone() {
                        self.open_file_handle(handle);
                    }
                } else {
                    self.playback_state = PlaybackState::Stopped;
                    self.playback_pos = self.total_duration;
                }
            } else if self.playback_state == PlaybackState::Playing && !self.is_seeking {
                self.playback_pos = Duration::from_secs_f64(sound_handle.position());
            }
        }
    }

    fn handle_keyboard_input(&mut self, ui: &mut egui::Ui) {
        ui.input(|i| {
            if i.key_pressed(egui::Key::Space) {
                self.play_pause();
            }
            if i.key_pressed(egui::Key::S) {
                self.stop();
            }
            if i.key_pressed(egui::Key::L) {
                self.is_looping = !self.is_looping;
            }
            if i.key_pressed(egui::Key::ArrowUp) {
                self.volume = (self.volume + 0.05).min(1.0);
                if let Some(sound_handle) = &mut self.sound_handle {
                    let _ = sound_handle.set_volume(self.volume, Default::default());
                }
            }
            if i.key_pressed(egui::Key::ArrowDown) {
                self.volume = (self.volume - 0.05).max(0.0);
                if let Some(sound_handle) = &mut self.sound_handle {
                    let _ = sound_handle.set_volume(self.volume, Default::default());
                }
            }
            if i.key_pressed(egui::Key::ArrowLeft) {
                if let Some(sound_handle) = &mut self.sound_handle {
                    let new_pos = self.playback_pos.saturating_sub(Duration::from_secs(5));
                    let _ = sound_handle.seek_to(new_pos.as_secs_f64());
                    self.playback_pos = new_pos;
                }
            }
            if i.key_pressed(egui::Key::ArrowRight) {
                if let Some(sound_handle) = &mut self.sound_handle {
                    let new_pos = self.playback_pos.saturating_add(Duration::from_secs(5));
                    let _ = sound_handle.seek_to(new_pos.as_secs_f64());
                    self.playback_pos = new_pos;
                }
            }
        });
    }

    fn open_file_handle(&mut self, handle: Arc<FileHandle>) {
        let path = handle.path();
        if let Some(manager) = AUDIO_MANAGER.as_ref() {
            if let Ok(sound_data) = StaticSoundData::from_file(path, StaticSoundSettings::default()) {
                self.total_duration = sound_data.duration();
                let mut manager = manager.lock();
                if let Ok(sound_handle) = manager.play(sound_data) {
                    self.sound_handle = Some(sound_handle);
                    self.current_file = Some(handle.clone());
                    self.playback_state = PlaybackState::Playing;
                    self.playback_pos = Duration::ZERO;
                    self.error = None;
                } else {
                    self.error = Some("Failed to play audio file".to_string());
                }
            } else {
                self.error = Some("Failed to decode audio file".to_string());
            }
        } else {
            self.error = Some("Failed to initialize audio device".to_string());
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(Vec2::new(450.0, 400.0))
            .with_resizable(false),
        ..Default::default()
    };
    eframe::run_native(
        "Rusty Audio",
        options,
        Box::new(|_cc| Box::new(AudioPlayerApp::default())),
    )
}