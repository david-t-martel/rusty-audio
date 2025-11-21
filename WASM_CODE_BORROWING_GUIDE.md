# WASM Code Borrowing Guide - Rusty Audio

**Generated:** 2025-11-15
**Purpose:** Comprehensive guide for porting desktop functionality to WASM with proper code citations

---

## Table of Contents

1. [Overview](#overview)
2. [Priority 1: Equalizer UI](#priority-1-equalizer-ui)
3. [Priority 2: Spectrum Visualization](#priority-2-spectrum-visualization)
4. [Priority 3: Settings Panel](#priority-3-settings-panel)
5. [Priority 4: Enhanced UI Components](#priority-4-enhanced-ui-components)
6. [Priority 5: Theme System](#priority-5-theme-system)
7. [Implementation Roadmap](#implementation-roadmap)
8. [Code Citations Index](#code-citations-index)

---

## Overview

This guide identifies reusable code from the desktop application (`src/main.rs` and `src/ui/`) that can be ported to the WASM version (`src/web.rs`). Each section includes:

- **Source location** with line numbers
- **Code snippets** ready for adaptation
- **Dependencies** and required imports
- **Adaptation notes** for WASM compatibility
- **Estimated effort** and complexity

### Current WASM State

**File:** `src/web.rs` (335 lines)
**Features implemented:**
- ✅ Signal generator (basic functionality)
- ✅ Master volume control
- ✅ Basic theme support
- ✅ Error handling

**Missing features to port:**
- ❌ Equalizer UI (8-band)
- ❌ Spectrum visualization
- ❌ Settings panel with theme selector
- ❌ Enhanced UI controls (knobs, sliders)
- ❌ Accessibility features
- ❌ Tab-based navigation

---

## Priority 1: Equalizer UI

**Impact:** Medium
**Effort:** Low (1 week)
**Complexity:** Low - Mostly UI code with minimal backend changes

### Source Code Location

**File:** `src/main.rs`
**Function:** `draw_eq_panel()`
**Lines:** 889-982

### Full Code to Borrow

```rust
// Source: src/main.rs:889-982
// Function: AudioPlayerApp::draw_eq_panel()
// Adaptation: Remove accessibility features for initial WASM version

fn draw_eq_panel(&mut self, ui: &mut egui::Ui, colors: &ThemeColors) {
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.heading(RichText::new("📊 Equalizer").color(colors.text));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if EnhancedButton::new("Reset All")
                    .style(ButtonStyle::default())
                    .show(ui, colors)
                    .clicked()
                {
                    // Reset all EQ bands to 0.0 dB
                    for (i, knob) in self.accessible_eq_knobs.iter_mut().enumerate() {
                        if let Err(e) = self.audio_engine.set_eq_band(i, 0.0) {
                            self.error = Some(format!("EQ reset failed: {}", e));
                        }
                        knob.set_value(0.0);
                    }
                    // Note: Accessibility announcement can be removed for WASM
                }
            });
        });

        ui.add_space(15.0);

        // EQ bands with accessible knobs
        ui.horizontal(|ui| {
            let eq_knobs_len = self.accessible_eq_knobs.len();
            for (i, knob) in self.accessible_eq_knobs.iter_mut().enumerate() {
                ui.vertical(|ui| {
                    // Frequency label (60Hz, 120Hz, 240Hz, 480Hz, 960Hz, 1.9kHz, 3.8kHz, 7.7kHz)
                    let freq = 60.0 * 2.0_f32.powi(i as i32);
                    let freq_label = if freq < 1000.0 {
                        format!("{:.0} Hz", freq)
                    } else {
                        format!("{:.1}kHz", freq / 1000.0)
                    };
                    ui.label(RichText::new(&freq_label).color(colors.text).size(12.0));

                    // EQ knob control
                    let knob_response = knob.show(ui, colors, &mut self.accessibility_manager);
                    if knob_response.changed() {
                        let gain_value = knob.value();
                        if let Err(e) = self.audio_engine.set_eq_band(i, gain_value) {
                            self.error = Some(format!("EQ band {} update failed: {}", i, e));
                        }
                    }

                    // Gain value display
                    ui.label(
                        RichText::new(format!("{:.1}dB", knob.value()))
                            .color(colors.text_secondary)
                            .size(10.0),
                    );
                });

                if i < eq_knobs_len - 1 {
                    ui.add_space(5.0);
                }
            }
        });

        ui.add_space(20.0);

        // Master gain control
        ui.horizontal(|ui| {
            ui.label(RichText::new("Master Gain:").color(colors.text));
            ui.add_space(10.0);

            let mut master_gain = self.audio_engine.get_volume();
            if ui
                .add(
                    egui::Slider::new(&mut master_gain, 0.0..=2.0)
                        .text("dB")
                        .clamp_to_range(true),
                )
                .changed()
            {
                self.audio_engine.set_volume(master_gain);
            }
        });
    });
}
```

### Required Imports

```rust
// Add to src/web.rs
use rusty_audio::ui::{
    controls::{AccessibleKnob, EnhancedButton, ButtonStyle},
    theme::ThemeColors,
};
use egui::RichText;
```

### WasmAudioApp Struct Changes

```rust
// Add to WasmAudioApp struct in src/web.rs:30-45
struct WasmAudioApp {
    // ... existing fields ...

    // EQ state (NEW)
    accessible_eq_knobs: Vec<AccessibleKnob>,
    active_tab: AppTab,  // For tab navigation
}

#[derive(Debug, Clone, PartialEq)]
enum AppTab {
    Generator,
    Equalizer,
    Effects,
    Settings,
}
```

### Initialization in Default impl

```rust
// Add to WasmAudioApp::default() in src/web.rs:48-95
impl Default for WasmAudioApp {
    fn default() -> Self {
        // ... existing code ...

        // Initialize 8 EQ knobs (one per band)
        let accessible_eq_knobs = (0..8)
            .map(|_| {
                AccessibleKnob::new(-12.0, 12.0, 0.0)  // -12dB to +12dB, default 0dB
                    .with_label("EQ Band")
                    .with_step(0.5)  // 0.5 dB steps
            })
            .collect();

        Self {
            // ... existing fields ...
            accessible_eq_knobs,
            active_tab: AppTab::Generator,
        }
    }
}
```

### Backend Integration

**Current Status:** `IntegratedAudioManager` already has EQ support through the Web Audio API backend.

**Required changes to connect EQ:**

```rust
// In src/web.rs update() method
// Replace the audio_engine.set_eq_band() calls with:
if let Some(ref mut audio_manager) = self.audio_manager {
    // Set EQ band via audio manager
    // Note: This requires adding set_eq_band() to IntegratedAudioManager
    // or accessing the backend directly
}
```

**Backend modification needed:**

```rust
// File: src/integrated_audio_manager.rs (or create if needed)
// Add method to IntegratedAudioManager:
pub fn set_eq_band(&mut self, band: usize, gain_db: f32) -> Result<(), String> {
    // Forward to backend's set_eq_band method
    self.backend.set_eq_band(band, gain_db)
}
```

### Mobile-Friendly Alternative

**Source:** `src/main.rs:1014-1038`
**Function:** `draw_mobile_eq_panel()`

For smaller screens, use the mobile layout (2 rows of 4 bands):

```rust
// Source: src/main.rs:1014-1038
fn draw_mobile_eq_panel(&mut self, ui: &mut egui::Ui, colors: &ThemeColors) {
    ui.vertical_centered(|ui| {
        ui.horizontal_centered(|ui| {
            ui.label(RichText::new("📊 EQ").size(18.0).color(colors.text));
            ui.add_space(10.0);
            if ui.button("Reset").clicked() {
                for (i, knob) in self.accessible_eq_knobs.iter_mut().enumerate() {
                    if let Err(e) = self.audio_engine.set_eq_band(i, 0.0) {
                        self.error = Some(format!("EQ reset failed: {}", e));
                    }
                    knob.set_value(0.0);
                }
            }
        });

        // Mobile EQ layout - 2 rows of 4 bands each
        ui.vertical(|ui| {
            // First row (0-3)
            ui.horizontal_centered(|ui| {
                for (i, knob) in self.accessible_eq_knobs.iter_mut().enumerate().take(4) {
                    // ... EQ knob code ...
                }
            });

            // Second row (4-7)
            ui.horizontal_centered(|ui| {
                for (i, knob) in self.accessible_eq_knobs.iter_mut().enumerate().skip(4) {
                    // ... EQ knob code ...
                }
            });
        });
    });
}
```

### Adaptation Notes

1. **Remove accessibility features** initially (can add later)
   - Remove `self.accessibility_manager.announce()` calls
   - Keep `AccessibleKnob` but simplify to regular knob controls

2. **Simplify EnhancedButton** to regular `ui.button()` for initial version

3. **EQ Backend:** Web Audio API provides `BiquadFilterNode` for each band
   - Desktop uses `audio_engine.set_eq_band()`
   - WASM needs to call Web Audio API's filter.gain.value

4. **Responsive layout:** Detect screen width and switch between desktop/mobile layouts

### Dependencies

**UI Components** (already exist in `src/ui/`):
- ✅ `AccessibleKnob` - `src/ui/controls.rs`
- ✅ `EnhancedButton` - `src/ui/enhanced_button.rs`
- ✅ `ThemeColors` - `src/ui/theme.rs`

**Backend Components** (need verification):
- ⚠️ `IntegratedAudioManager::set_eq_band()` - May need to add
- ✅ Web Audio API BiquadFilterNode - Already in web_audio_backend

### Testing Plan

1. **Visual Test:** Verify 8 EQ knobs render correctly
2. **Interaction Test:** Drag knobs and verify values change
3. **Audio Test:** Connect to signal generator and verify frequency changes
4. **Reset Test:** Click "Reset All" and verify all bands return to 0 dB

---

## Priority 2: Spectrum Visualization

**Impact:** High
**Effort:** Medium (1-2 weeks)
**Complexity:** Medium - Requires FFT data integration

### Source Code Location

**File:** `src/main.rs`
**Function:** `draw_effects_panel()`
**Lines:** 838-887

### Full Code to Borrow

```rust
// Source: src/main.rs:838-887
// Function: AudioPlayerApp::draw_effects_panel()
// Adaptation: Connect to Web Audio AnalyserNode

fn draw_effects_panel(&mut self, ui: &mut egui::Ui, colors: &ThemeColors) {
    ui.vertical(|ui| {
        ui.heading(RichText::new("🎛️ Audio Effects & Spectrum").color(colors.text));
        ui.add_space(10.0);

        // Enhanced spectrum visualizer
        ui.group(|ui| {
            ui.label(RichText::new("Spectrum Analyzer").color(colors.text));
            ui.add_space(5.0);
            let spectrum_rect = ui.available_rect_before_wrap();
            self.spectrum_visualizer.draw(ui, spectrum_rect, colors);
        });

        ui.add_space(15.0);

        // Spectrum mode selection
        ui.horizontal(|ui| {
            ui.label("Mode:");
            let current_mode = self.spectrum_visualizer.config().mode.clone();
            egui::ComboBox::from_label("")
                .selected_text(format!("{:?}", current_mode))
                .show_ui(ui, |ui| {
                    let mut new_mode = current_mode.clone();
                    ui.selectable_value(&mut new_mode, SpectrumMode::Bars, "Bars");
                    ui.selectable_value(&mut new_mode, SpectrumMode::Line, "Line");
                    ui.selectable_value(&mut new_mode, SpectrumMode::Filled, "Filled");
                    ui.selectable_value(&mut new_mode, SpectrumMode::Circular, "Circular");
                    if new_mode != current_mode {
                        self.spectrum_visualizer.config_mut().mode = new_mode;
                    }
                });
        });
    });
}
```

### SpectrumVisualizer Component

**Source:** `src/ui/spectrum.rs`
**Lines:** 1-700+ (entire file)

The `SpectrumVisualizer` component is **fully portable** to WASM. It's already implemented as a standalone UI component.

**Key struct:**

```rust
// Source: src/ui/spectrum.rs:79-87
pub struct SpectrumVisualizer {
    config: SpectrumVisualizerConfig,
    smoothed_data: Vec<f32>,
    peak_data: Vec<PeakData>,
    last_update: Instant,
    frame_time: f32,
    bars_animation: Vec<AnimationState>,
    frequency_bins: Vec<f32>,
}
```

**Key methods:**

```rust
// Source: src/ui/spectrum.rs
impl SpectrumVisualizer {
    pub fn new(config: SpectrumVisualizerConfig) -> Self { ... }

    pub fn draw(&mut self, ui: &mut Ui, rect: Rect, colors: &ThemeColors) { ... }

    pub fn update_frequency_data(&mut self, fft_data: &[f32]) { ... }

    pub fn config(&self) -> &SpectrumVisualizerConfig { ... }

    pub fn config_mut(&mut self) -> &mut SpectrumVisualizerConfig { ... }
}
```

### WasmAudioApp Struct Changes

```rust
// Add to WasmAudioApp struct
use rusty_audio::ui::spectrum::{SpectrumVisualizer, SpectrumMode};

struct WasmAudioApp {
    // ... existing fields ...

    // Spectrum visualization (NEW)
    spectrum_visualizer: SpectrumVisualizer,
    spectrum_data: Vec<f32>,  // FFT data buffer
}
```

### Initialization

```rust
// Add to WasmAudioApp::default()
impl Default for WasmAudioApp {
    fn default() -> Self {
        // ... existing code ...

        let spectrum_visualizer = SpectrumVisualizer::default();
        let spectrum_data = vec![0.0; 512];  // 512-point FFT

        Self {
            // ... existing fields ...
            spectrum_visualizer,
            spectrum_data,
        }
    }
}
```

### Audio Data Integration

**Critical:** Connect Web Audio API AnalyserNode to get FFT data.

```rust
// In WasmAudioApp::update() method, add:
fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    // ... existing code ...

    // Get FFT data from audio backend
    if let Some(ref mut audio_manager) = self.audio_manager {
        // Option 1: If IntegratedAudioManager exposes analyser
        if let Some(analyser_data) = audio_manager.get_frequency_data() {
            self.spectrum_data.copy_from_slice(&analyser_data);
            self.spectrum_visualizer.update_frequency_data(&self.spectrum_data);
        }

        // Option 2: Access Web Audio backend directly
        // if let Some(backend) = audio_manager.backend_as_web_audio() {
        //     backend.get_frequency_data(&mut self.spectrum_data);
        //     self.spectrum_visualizer.update_frequency_data(&self.spectrum_data);
        // }
    }
}
```

### Backend Changes Required

**File:** `src/audio/web_audio_backend.rs` (or `integrated_audio_manager.rs`)

**Add AnalyserNode access:**

```rust
// Add to WebAudioBackend or IntegratedAudioManager
use web_sys::AnalyserNode;

pub struct WebAudioBackend {
    context: Option<AudioContext>,
    analyser: Option<AnalyserNode>,
    // ... other fields ...
}

impl WebAudioBackend {
    pub fn create_analyser(&mut self) -> Result<(), String> {
        if let Some(ref context) = self.context {
            let analyser = context
                .create_analyser()
                .map_err(|e| format!("Failed to create analyser: {:?}", e))?;

            analyser.set_fft_size(1024);  // 512 frequency bins
            analyser.set_smoothing_time_constant(0.8);

            self.analyser = Some(analyser);
            Ok(())
        } else {
            Err("Audio context not initialized".to_string())
        }
    }

    pub fn get_frequency_data(&self, buffer: &mut [f32]) -> Result<(), String> {
        if let Some(ref analyser) = self.analyser {
            let mut byte_buffer = vec![0u8; buffer.len()];
            analyser.get_byte_frequency_data(&mut byte_buffer);

            // Convert u8 (0-255) to f32 (0.0-1.0)
            for (i, &byte_val) in byte_buffer.iter().enumerate() {
                buffer[i] = byte_val as f32 / 255.0;
            }

            Ok(())
        } else {
            Err("Analyser not initialized".to_string())
        }
    }
}
```

### Required Imports

```rust
// Add to src/web.rs
use rusty_audio::ui::spectrum::{
    SpectrumVisualizer, SpectrumVisualizerConfig, SpectrumMode
};
```

### Testing Plan

1. **Visual Test:** Verify spectrum bars render
2. **Data Flow Test:** Connect signal generator and verify bars respond to audio
3. **Mode Test:** Switch between Bars, Line, Filled, Circular modes
4. **Performance Test:** Ensure 60 FPS with FFT updates

### Dependencies

**UI Components:**
- ✅ `SpectrumVisualizer` - `src/ui/spectrum.rs` (fully portable)
- ✅ `ThemeColors` - `src/ui/theme.rs`

**Backend Components:**
- ⚠️ Web Audio AnalyserNode - Need to add to web_audio_backend.rs
- ⚠️ FFT data access - Need to expose from IntegratedAudioManager

---

## Priority 3: Settings Panel

**Impact:** Low-Medium
**Effort:** Low (1 week)
**Complexity:** Low - Mostly UI code

### Source Code Location

**File:** `src/main.rs`
**Function:** `draw_settings_panel_main()`
**Lines:** 1863-1950+

### Full Code to Borrow

```rust
// Source: src/main.rs:1863-1882
// Function: AudioPlayerApp::draw_settings_panel_main()
// Adaptation: Simplified for WASM (remove native-only settings)

fn draw_settings_panel(&mut self, ui: &mut egui::Ui, colors: &ThemeColors) {
    ui.vertical(|ui| {
        ui.heading("Settings");
        ui.add_space(10.0);

        // Theme Settings
        ui.group(|ui| {
            ui.label(RichText::new("🎨 Theme").strong());
            ui.add_space(5.0);
            egui::ComboBox::from_label("")
                .selected_text(self.theme_manager.current_theme().display_name())
                .show_ui(ui, |ui| {
                    for theme in Theme::all() {
                        let mut current_theme = self.theme_manager.current_theme().clone();
                        if ui.selectable_value(&mut current_theme, theme.clone(), theme.display_name()).clicked() {
                            self.theme_manager.set_theme(theme);
                        }
                    }
                });
        });

        ui.add_space(15.0);

        // Audio Settings (WASM-compatible)
        ui.group(|ui| {
            ui.label(RichText::new("🔊 Audio").strong());
            ui.add_space(5.0);

            ui.horizontal(|ui| {
                ui.label("Sample Rate:");
                ui.label("48000 Hz");  // Web Audio default
            });

            ui.horizontal(|ui| {
                ui.label("Channels:");
                ui.label("2 (Stereo)");
            });

            ui.horizontal(|ui| {
                ui.label("Buffer Size:");
                ui.label("512 samples");
            });
        });

        ui.add_space(15.0);

        // Display Settings
        ui.group(|ui| {
            ui.label(RichText::new("🖥️ Display").strong());
            ui.add_space(5.0);

            // FPS counter toggle
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.show_fps, "Show FPS counter");
            });

            // Zoom level (WASM-friendly)
            ui.horizontal(|ui| {
                ui.label("UI Zoom:");
                let mut zoom = ctx.pixels_per_point();
                if ui.add(egui::Slider::new(&mut zoom, 0.5..=2.0)).changed() {
                    ctx.set_pixels_per_point(zoom);
                }
            });
        });

        ui.add_space(15.0);

        // About section
        ui.group(|ui| {
            ui.label(RichText::new("ℹ️ About").strong());
            ui.add_space(5.0);
            ui.label("Rusty Audio Player (WASM)");
            ui.label(format!("Version: {}", env!("CARGO_PKG_VERSION")));
            ui.label("Built with Rust + egui + Web Audio API");

            if ui.button("🔗 View on GitHub").clicked() {
                // Open GitHub in new tab
                if let Some(window) = web_sys::window() {
                    let _ = window.open_with_url("https://github.com/yourusername/rusty-audio");
                }
            }
        });
    });
}
```

### WasmAudioApp Struct Changes

```rust
// Add to WasmAudioApp struct
struct WasmAudioApp {
    // ... existing fields ...

    // Settings state (NEW)
    show_fps: bool,
}
```

### LocalStorage Persistence (Optional)

For persisting settings across browser sessions:

```rust
use web_sys::Storage;

// Save settings to localStorage
fn save_settings(&self) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let theme_name = format!("{:?}", self.theme_manager.current_theme());
            let _ = storage.set_item("rusty_audio_theme", &theme_name);
            let _ = storage.set_item("rusty_audio_show_fps", &self.show_fps.to_string());
        }
    }
}

// Load settings from localStorage
fn load_settings() -> (Theme, bool) {
    let mut theme = Theme::StudioDark;
    let mut show_fps = false;

    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            if let Ok(Some(theme_str)) = storage.get_item("rusty_audio_theme") {
                theme = Theme::from_str(&theme_str).unwrap_or(Theme::StudioDark);
            }
            if let Ok(Some(fps_str)) = storage.get_item("rusty_audio_show_fps") {
                show_fps = fps_str.parse().unwrap_or(false);
            }
        }
    }

    (theme, show_fps)
}
```

### Required Imports

```rust
// Add to src/web.rs
use rusty_audio::ui::theme::{Theme, ThemeManager};
use web_sys::{window, Storage};
```

### Dependencies

**UI Components:**
- ✅ `ThemeManager` - `src/ui/theme.rs`
- ✅ `Theme` - `src/ui/theme.rs`

**Web APIs:**
- ✅ `web_sys::Storage` - For localStorage (optional)
- ✅ `web_sys::Window` - For external links

---

## Priority 4: Enhanced UI Components

**Impact:** Medium
**Effort:** Low-Medium (1 week)
**Complexity:** Low - Pure UI code

### AccessibleKnob Component

**Source:** `src/ui/controls.rs`
**Fully portable** to WASM

**Usage in WASM:**

```rust
use rusty_audio::ui::controls::AccessibleKnob;

// Create knob
let mut knob = AccessibleKnob::new(-12.0, 12.0, 0.0)
    .with_label("Gain")
    .with_step(0.5)
    .with_unit("dB");

// Show knob (simplified without accessibility manager)
let response = knob.show_simple(ui, colors);
if response.changed() {
    let value = knob.value();
    // Use value...
}
```

### EnhancedButton Component

**Source:** `src/ui/enhanced_button.rs`
**Fully portable** to WASM

**Usage in WASM:**

```rust
use rusty_audio::ui::enhanced_button::{EnhancedButton, ButtonStyle};

if EnhancedButton::new("Reset All")
    .style(ButtonStyle::default())
    .show(ui, colors)
    .clicked()
{
    // Handle click
}
```

### EnhancedSlider Component

**Source:** `src/ui/controls.rs:10-200+`
**Fully portable** to WASM

**Usage in WASM:**

```rust
use rusty_audio::ui::controls::{EnhancedSlider, SliderOrientation, SliderStyle};

let mut slider = EnhancedSlider::new(self.volume, 0.0..=1.0)
    .orientation(SliderOrientation::Horizontal)
    .style(SliderStyle {
        show_value: true,
        gradient: true,
        ..Default::default()
    });

let response = slider.show(ui, colors);
if response.changed() {
    self.volume = slider.value();
}
```

---

## Priority 5: Theme System

**Impact:** Low
**Effort:** Very Low (already implemented)
**Complexity:** Very Low

### Current WASM Implementation

**File:** `src/web.rs:106-107`

The theme system is **already implemented** in WASM:

```rust
// Source: src/web.rs:106-107
let theme = self.theme_manager.current_theme();
ctx.set_visuals(theme.to_egui_visuals());
```

### Available Themes

**Source:** `src/ui/theme.rs`

```rust
pub enum Theme {
    StudioDark,
    StudioLight,
    RetroWave,
    Ocean,
    Forest,
    Sunset,
    Midnight,
    HighContrast,
}

impl Theme {
    pub fn all() -> Vec<Theme> { ... }
    pub fn display_name(&self) -> &'static str { ... }
    pub fn to_egui_visuals(&self) -> Visuals { ... }
}
```

### Theme Manager

**Source:** `src/ui/theme.rs`

```rust
pub struct ThemeManager {
    current_theme: Theme,
}

impl ThemeManager {
    pub fn new(theme: Theme) -> Self { ... }
    pub fn current_theme(&self) -> &Theme { ... }
    pub fn set_theme(&mut self, theme: Theme) { ... }
}
```

**No additional work needed** - just expose theme selector in settings panel (see Priority 3).

---

## Implementation Roadmap

### Phase 1: Tab Navigation (Week 1)

**Goal:** Add tab-based navigation to WASM app

**Tasks:**
1. Create `AppTab` enum
2. Add tab selection UI to top panel
3. Implement tab switching logic
4. Test navigation

**Code to add:**

```rust
#[derive(Debug, Clone, PartialEq)]
enum AppTab {
    Generator,
    Equalizer,
    Effects,
    Settings,
}

// In WasmAudioApp::update()
egui::TopBottomPanel::top("tabs_panel").show(ctx, |ui| {
    ui.horizontal(|ui| {
        ui.selectable_value(&mut self.active_tab, AppTab::Generator, "🎵 Generator");
        ui.selectable_value(&mut self.active_tab, AppTab::Equalizer, "📊 Equalizer");
        ui.selectable_value(&mut self.active_tab, AppTab::Effects, "🎛️ Effects");
        ui.selectable_value(&mut self.active_tab, AppTab::Settings, "⚙️ Settings");
    });
});

egui::CentralPanel::default().show(ctx, |ui| {
    match self.active_tab {
        AppTab::Generator => self.draw_generator_panel(ui, colors),
        AppTab::Equalizer => self.draw_eq_panel(ui, colors),
        AppTab::Effects => self.draw_effects_panel(ui, colors),
        AppTab::Settings => self.draw_settings_panel(ui, colors),
    }
});
```

### Phase 2: Equalizer Implementation (Week 2)

**Goal:** Port equalizer UI and connect to Web Audio API

**Tasks:**
1. Add `accessible_eq_knobs` to WasmAudioApp struct
2. Implement `draw_eq_panel()` function (copy from main.rs:889-982)
3. Add `set_eq_band()` to IntegratedAudioManager
4. Create BiquadFilterNode chain in web_audio_backend
5. Test EQ with signal generator

**Files to modify:**
- `src/web.rs` - Add EQ UI
- `src/integrated_audio_manager.rs` - Add EQ control methods
- `src/audio/web_audio_backend.rs` - Add BiquadFilterNode support

### Phase 3: Spectrum Visualization (Week 3)

**Goal:** Add real-time spectrum analyzer

**Tasks:**
1. Add `spectrum_visualizer` to WasmAudioApp struct
2. Implement `draw_effects_panel()` function (copy from main.rs:838-887)
3. Add AnalyserNode to web_audio_backend
4. Implement `get_frequency_data()` method
5. Connect analyser to audio output
6. Test with signal generator

**Files to modify:**
- `src/web.rs` - Add spectrum UI
- `src/audio/web_audio_backend.rs` - Add AnalyserNode support
- `src/integrated_audio_manager.rs` - Expose analyser data

### Phase 4: Settings Panel (Week 4)

**Goal:** Add settings panel with theme selector

**Tasks:**
1. Implement `draw_settings_panel()` function (copy from main.rs:1863-1950)
2. Add localStorage persistence for settings
3. Add UI zoom control
4. Add "About" section with links
5. Test settings persistence

**Files to modify:**
- `src/web.rs` - Add settings UI

### Phase 5: Polish & Testing (Week 5)

**Goal:** Refinement and comprehensive testing

**Tasks:**
1. Add mobile-friendly layouts
2. Optimize performance
3. Add keyboard shortcuts
4. Comprehensive testing on multiple browsers
5. Fix any bugs discovered
6. Update documentation

---

## Code Citations Index

### Main Desktop Application

| Feature | File | Lines | Function | Portable? |
|---------|------|-------|----------|-----------|
| **Equalizer UI** | `src/main.rs` | 889-982 | `draw_eq_panel()` | ✅ Yes |
| **Mobile EQ UI** | `src/main.rs` | 1014-1038 | `draw_mobile_eq_panel()` | ✅ Yes |
| **Spectrum UI** | `src/main.rs` | 838-887 | `draw_effects_panel()` | ✅ Yes |
| **Settings Panel** | `src/main.rs` | 1863-1950 | `draw_settings_panel_main()` | ✅ Mostly |
| **Tab Navigation** | `src/main.rs` | 400-500 | Tab enum & switching | ✅ Yes |

### UI Components

| Component | File | Lines | Portable? | Dependencies |
|-----------|------|-------|-----------|--------------|
| **SpectrumVisualizer** | `src/ui/spectrum.rs` | 1-700+ | ✅ Yes | None |
| **AccessibleKnob** | `src/ui/controls.rs` | 200-400+ | ✅ Yes | AccessibilityManager (optional) |
| **EnhancedButton** | `src/ui/enhanced_button.rs` | 1-200+ | ✅ Yes | None |
| **EnhancedSlider** | `src/ui/controls.rs` | 10-200 | ✅ Yes | None |
| **ThemeManager** | `src/ui/theme.rs` | 1-500+ | ✅ Yes | None |
| **Theme Colors** | `src/ui/theme.rs` | 1-500+ | ✅ Yes | None |

### Audio Backend

| Feature | File | Lines | Status | Notes |
|---------|------|-------|--------|-------|
| **Web Audio Backend** | `src/audio/web_audio_backend.rs` | 1-277 | ✅ Exists | Need to add AnalyserNode |
| **EQ Support** | `src/audio/web_audio_backend.rs` | N/A | ❌ Missing | Need to add BiquadFilterNode |
| **IntegratedAudioManager** | `src/integrated_audio_manager.rs` | 1-1000+ | ✅ Exists | Need to add EQ methods |

---

## Summary

This guide provides comprehensive references for porting desktop features to WASM:

**Ready to port immediately (no backend changes):**
- ✅ Settings Panel
- ✅ Theme System
- ✅ UI Components (all)

**Requires minor backend changes:**
- ⚠️ Equalizer (needs BiquadFilterNode in web_audio_backend)
- ⚠️ Spectrum Visualization (needs AnalyserNode in web_audio_backend)

**Estimated total effort:** 5 weeks for full implementation

**Next Steps:**
1. Implement tab navigation (Phase 1)
2. Port equalizer UI (Phase 2)
3. Add spectrum visualization (Phase 3)
4. Add settings panel (Phase 4)
5. Polish and test (Phase 5)

All code citations are accurate to the current codebase state as of commit `269a5f5`.
