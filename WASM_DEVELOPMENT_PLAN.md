# WASM Development Plan - Rusty Audio

**Date:** 2025-11-15
**Status:** Ready for Implementation
**Target:** Feature parity with desktop application for WASM deployment

---

## Executive Summary

This document provides a comprehensive development plan for expanding the Rusty Audio WASM implementation to include features currently available only in the desktop version. All code references include proper citations to source files.

**Current State:**
- ✅ WASM build infrastructure operational
- ✅ Basic signal generator working
- ✅ Web Audio API backend integrated
- ✅ PWA infrastructure complete

**Target State:**
- 🎯 Full equalizer with 8-band control
- 🎯 Real-time spectrum visualization
- 🎯 Settings panel with theme selection
- 🎯 Tab-based navigation
- 🎯 Enhanced UI components

**Timeline:** 5 weeks
**Estimated Lines of Code:** 1,350-2,100 new lines

---

## Reference Documents

This plan references three comprehensive analysis documents:

1. **WASM_IMPLEMENTATION_ANALYSIS.md** (25KB, 850 lines)
   - Current infrastructure status
   - Feature gap analysis
   - Build configuration details
   - Effort estimates

2. **WASM_CODE_BORROWING_GUIDE.md** (25KB+)
   - Specific code to borrow from desktop app
   - Proper citations with file paths and line numbers
   - Adaptation notes for WASM compatibility
   - Implementation guidance

3. **WASM_DEVELOPMENT_PLAN.md** (this document)
   - Week-by-week implementation plan
   - Acceptance criteria for each feature
   - Testing strategy

---

## Development Phases

### Phase 1: Foundation - Tab Navigation
**Duration:** Week 1
**Effort:** Low
**Complexity:** Low
**Lines of Code:** ~150

#### Goals
- Add tab-based navigation UI
- Switch between Generator, Equalizer, Effects, and Settings tabs
- Refactor existing signal generator into tab panel

#### Tasks

##### Task 1.1: Create Tab System
**Source Reference:** `src/main.rs:93-160` (AudioPlayerApp struct with Tab enum)

```rust
// Add to src/web.rs
#[derive(Debug, Clone, PartialEq)]
enum AppTab {
    Generator,
    Equalizer,
    Effects,
    Settings,
}

// Add to WasmAudioApp struct
struct WasmAudioApp {
    // ... existing fields ...
    active_tab: AppTab,
}
```

**Files to modify:**
- `src/web.rs` (add AppTab enum and field)

**Acceptance Criteria:**
- ✅ Four tabs visible in top panel
- ✅ Clicking tab switches content
- ✅ Active tab is visually highlighted
- ✅ Tab state persists during session

##### Task 1.2: Refactor Generator Panel
**Source Reference:** `src/web.rs:150-221` (current generator code)

Extract signal generator code into separate method:

```rust
fn draw_generator_panel(&mut self, ui: &mut egui::Ui, colors: &ThemeColors) {
    // Move existing generator code here
}
```

**Files to modify:**
- `src/web.rs` (refactor update() method)

**Acceptance Criteria:**
- ✅ Generator panel works in tab view
- ✅ No regression in functionality

##### Task 1.3: Add Placeholder Tabs

```rust
fn draw_eq_panel(&mut self, ui: &mut egui::Ui, colors: &ThemeColors) {
    ui.heading("Equalizer");
    ui.label("Coming soon...");
}

fn draw_effects_panel(&mut self, ui: &mut egui::Ui, colors: &ThemeColors) {
    ui.heading("Effects & Spectrum");
    ui.label("Coming soon...");
}

fn draw_settings_panel(&mut self, ui: &mut egui::Ui, colors: &ThemeColors) {
    ui.heading("Settings");
    ui.label("Coming soon...");
}
```

**Files to modify:**
- `src/web.rs` (add placeholder methods)

**Acceptance Criteria:**
- ✅ All tabs render without errors
- ✅ Smooth tab switching

#### Testing Checklist
- [ ] Visual: Tabs render correctly on desktop
- [ ] Visual: Tabs render correctly on mobile
- [ ] Interaction: Tab switching works
- [ ] Performance: No lag when switching tabs
- [ ] Persistence: Active tab doesn't reset on repaint

---

### Phase 2: Equalizer Implementation
**Duration:** Week 2
**Effort:** Medium
**Complexity:** Medium
**Lines of Code:** ~400-500

#### Goals
- Add 8-band parametric equalizer UI
- Connect EQ to Web Audio API BiquadFilterNode
- Enable EQ for signal generator output
- Add reset functionality

#### Tasks

##### Task 2.1: Add EQ UI Components
**Source Reference:** `src/main.rs:889-982` (draw_eq_panel function)
**Guide Reference:** WASM_CODE_BORROWING_GUIDE.md (Priority 1)

Copy EQ panel code from desktop app:

```rust
// Add to WasmAudioApp struct
struct WasmAudioApp {
    // ... existing fields ...
    accessible_eq_knobs: Vec<AccessibleKnob>,
}

// Initialize in Default impl
impl Default for WasmAudioApp {
    fn default() -> Self {
        // ... existing code ...

        let accessible_eq_knobs = (0..8)
            .map(|_| {
                AccessibleKnob::new(-12.0, 12.0, 0.0)
                    .with_label("EQ Band")
                    .with_step(0.5)
            })
            .collect();

        Self {
            // ... existing fields ...
            accessible_eq_knobs,
        }
    }
}
```

**Files to modify:**
- `src/web.rs` (add EQ UI)

**Dependencies:**
- Import `AccessibleKnob` from `src/ui/controls.rs`
- Import `EnhancedButton` from `src/ui/enhanced_button.rs`

**Acceptance Criteria:**
- ✅ 8 EQ knobs render correctly
- ✅ Frequency labels show correct values (60Hz to 7.7kHz)
- ✅ Knobs respond to mouse interaction
- ✅ Reset button sets all bands to 0 dB

##### Task 2.2: Implement Web Audio EQ Backend
**Source Reference:** `src/audio/web_audio_backend.rs:1-277`

Add BiquadFilterNode support to Web Audio backend:

```rust
// Add to WebAudioBackend or IntegratedAudioManager
use web_sys::{BiquadFilterNode, BiquadFilterType};

pub struct WebAudioBackend {
    // ... existing fields ...
    eq_filters: Vec<BiquadFilterNode>,
}

impl WebAudioBackend {
    pub fn create_eq_chain(&mut self) -> Result<(), String> {
        if let Some(ref context) = self.context {
            let mut filters = Vec::new();

            // Create 8 peaking filters
            for i in 0..8 {
                let filter = context
                    .create_biquad_filter()
                    .map_err(|e| format!("Failed to create filter: {:?}", e))?;

                filter.set_type(BiquadFilterType::Peaking);

                // Set center frequency (60Hz * 2^i)
                let freq = 60.0 * 2.0_f32.powi(i);
                filter.frequency().set_value(freq);

                // Q factor for peaking filter
                filter.q().set_value(1.0);

                // Initial gain: 0 dB
                filter.gain().set_value(0.0);

                filters.push(filter);
            }

            self.eq_filters = filters;
            Ok(())
        } else {
            Err("Audio context not initialized".to_string())
        }
    }

    pub fn set_eq_band(&mut self, band: usize, gain_db: f32) -> Result<(), String> {
        if band >= self.eq_filters.len() {
            return Err(format!("Invalid EQ band: {}", band));
        }

        // Clamp gain to safe range
        let clamped_gain = gain_db.clamp(-12.0, 12.0);

        self.eq_filters[band].gain().set_value(clamped_gain);
        Ok(())
    }
}
```

**Files to modify:**
- `src/audio/web_audio_backend.rs` (add EQ methods)
- `src/integrated_audio_manager.rs` (expose EQ control)

**web-sys features required:**
Add to `Cargo.toml`:
```toml
[target.'cfg(target_arch = "wasm32")'.dependencies.web-sys]
features = [
    # ... existing features ...
    "BiquadFilterNode",
    "BiquadFilterType",
]
```

**Acceptance Criteria:**
- ✅ 8 BiquadFilterNodes created on init
- ✅ Filters correctly chained in audio graph
- ✅ set_eq_band() updates filter gain
- ✅ Gain clamped to ±12 dB

##### Task 2.3: Connect UI to Backend

Wire up EQ knobs to backend:

```rust
// In draw_eq_panel()
let knob_response = knob.show(ui, colors);
if knob_response.changed() {
    let gain_value = knob.value();
    if let Some(ref mut audio_manager) = self.audio_manager {
        if let Err(e) = audio_manager.set_eq_band(i, gain_value) {
            self.error_message = Some(format!("EQ band {} update failed: {}", i, e));
        }
    }
}
```

**Files to modify:**
- `src/web.rs` (connect UI to backend)
- `src/integrated_audio_manager.rs` (add set_eq_band method)

**Acceptance Criteria:**
- ✅ Moving knob updates audio in real-time
- ✅ No audio glitches or pops
- ✅ Error handling works correctly

##### Task 2.4: Add Reset Functionality

```rust
if ui.button("Reset All").clicked() {
    for (i, knob) in self.accessible_eq_knobs.iter_mut().enumerate() {
        if let Some(ref mut audio_manager) = self.audio_manager {
            if let Err(e) = audio_manager.set_eq_band(i, 0.0) {
                self.error_message = Some(format!("EQ reset failed: {}", e));
            }
        }
        knob.set_value(0.0);
    }
}
```

**Acceptance Criteria:**
- ✅ Reset button sets all bands to 0 dB
- ✅ UI knobs update to reflect reset

#### Testing Checklist
- [ ] Visual: 8 EQ knobs render correctly
- [ ] Interaction: Knobs respond to drag
- [ ] Audio: EQ changes affect signal generator output
- [ ] Audio: Boost and cut work correctly for each band
- [ ] Audio: Frequency bands are correct (60Hz to 7.7kHz)
- [ ] Functional: Reset button works
- [ ] Performance: No lag when adjusting EQ
- [ ] Error: Graceful handling if backend fails

---

### Phase 3: Spectrum Visualization
**Duration:** Week 3
**Effort:** Medium
**Complexity:** Medium
**Lines of Code:** ~300-400

#### Goals
- Add real-time spectrum analyzer
- Support multiple visualization modes (Bars, Line, Filled, Circular)
- Connect to Web Audio AnalyserNode
- Display frequency data from signal generator and EQ output

#### Tasks

##### Task 3.1: Add SpectrumVisualizer Component
**Source Reference:** `src/ui/spectrum.rs:1-700+`
**Guide Reference:** WASM_CODE_BORROWING_GUIDE.md (Priority 2)

```rust
// Add to WasmAudioApp struct
use rusty_audio::ui::spectrum::{SpectrumVisualizer, SpectrumMode};

struct WasmAudioApp {
    // ... existing fields ...
    spectrum_visualizer: SpectrumVisualizer,
    spectrum_data: Vec<f32>,
}

// Initialize in Default impl
impl Default for WasmAudioApp {
    fn default() -> Self {
        // ... existing code ...

        let spectrum_visualizer = SpectrumVisualizer::default();
        let spectrum_data = vec![0.0; 512];  // 512 frequency bins

        Self {
            // ... existing fields ...
            spectrum_visualizer,
            spectrum_data,
        }
    }
}
```

**Files to modify:**
- `src/web.rs` (add spectrum visualizer)

**Dependencies:**
- Import from `src/ui/spectrum.rs` (already exists)

**Acceptance Criteria:**
- ✅ SpectrumVisualizer instance created
- ✅ Default configuration loads

##### Task 3.2: Implement Effects Panel UI
**Source Reference:** `src/main.rs:838-887` (draw_effects_panel)

```rust
fn draw_effects_panel(&mut self, ui: &mut egui::Ui, colors: &ThemeColors) {
    ui.vertical(|ui| {
        ui.heading(RichText::new("🎛️ Audio Effects & Spectrum").color(colors.text));
        ui.add_space(10.0);

        // Spectrum visualizer
        ui.group(|ui| {
            ui.label(RichText::new("Spectrum Analyzer").color(colors.text));
            ui.add_space(5.0);
            let spectrum_rect = ui.available_rect_before_wrap();
            self.spectrum_visualizer.draw(ui, spectrum_rect, colors);
        });

        ui.add_space(15.0);

        // Mode selection
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

**Files to modify:**
- `src/web.rs` (add draw_effects_panel method)

**Acceptance Criteria:**
- ✅ Effects panel renders in tab
- ✅ Spectrum visualizer shows (even with no data)
- ✅ Mode selector works

##### Task 3.3: Add Web Audio AnalyserNode
**Source Reference:** New code for `src/audio/web_audio_backend.rs`

```rust
use web_sys::AnalyserNode;

pub struct WebAudioBackend {
    // ... existing fields ...
    analyser: Option<AnalyserNode>,
}

impl WebAudioBackend {
    pub fn create_analyser(&mut self) -> Result<(), String> {
        if let Some(ref context) = self.context {
            let analyser = context
                .create_analyser()
                .map_err(|e| format!("Failed to create analyser: {:?}", e))?;

            // Configure analyser
            analyser.set_fft_size(1024);  // 512 frequency bins
            analyser.set_smoothing_time_constant(0.8);
            analyser.set_min_decibels(-90.0);
            analyser.set_max_decibels(-10.0);

            self.analyser = Some(analyser);
            Ok(())
        } else {
            Err("Audio context not initialized".to_string())
        }
    }

    pub fn get_frequency_data(&self, buffer: &mut [f32]) -> Result<(), String> {
        if let Some(ref analyser) = self.analyser {
            // Create byte buffer for Web Audio API
            let mut byte_buffer = vec![0u8; analyser.frequency_bin_count() as usize];
            analyser.get_byte_frequency_data(&mut byte_buffer);

            // Convert u8 (0-255) to f32 (0.0-1.0)
            let len = buffer.len().min(byte_buffer.len());
            for i in 0..len {
                buffer[i] = byte_buffer[i] as f32 / 255.0;
            }

            Ok(())
        } else {
            Err("Analyser not initialized".to_string())
        }
    }

    pub fn connect_analyser_to_output(&mut self) -> Result<(), String> {
        // Connect analyser to destination for monitoring
        // This should be called after creating the audio graph
        if let (Some(ref analyser), Some(ref context)) = (&self.analyser, &self.context) {
            // Connect to context destination
            analyser.connect_with_audio_node(&context.destination())
                .map_err(|e| format!("Failed to connect analyser: {:?}", e))?;
            Ok(())
        } else {
            Err("Analyser or context not initialized".to_string())
        }
    }
}
```

**Files to modify:**
- `src/audio/web_audio_backend.rs` (add analyser support)

**web-sys features required:**
Add to `Cargo.toml`:
```toml
[target.'cfg(target_arch = "wasm32")'.dependencies.web-sys]
features = [
    # ... existing features ...
    "AnalyserNode",
]
```

**Acceptance Criteria:**
- ✅ AnalyserNode created successfully
- ✅ FFT size set to 1024 (512 bins)
- ✅ get_frequency_data() returns valid data
- ✅ Analyser connected to audio output

##### Task 3.4: Connect Analyzer to UI

```rust
// In WasmAudioApp::update()
fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    // ... existing code ...

    // Update spectrum data
    if let Some(ref mut audio_manager) = self.audio_manager {
        // Get frequency data from analyser
        if let Ok(_) = audio_manager.get_frequency_data(&mut self.spectrum_data) {
            self.spectrum_visualizer.update_frequency_data(&self.spectrum_data);
        }
    }

    // ... rest of update code ...
}
```

**Files to modify:**
- `src/web.rs` (add spectrum data update)
- `src/integrated_audio_manager.rs` (expose get_frequency_data)

**Acceptance Criteria:**
- ✅ Spectrum updates in real-time
- ✅ Bars respond to audio from signal generator
- ✅ Bars respond to EQ changes
- ✅ Smooth 60 FPS animation

#### Testing Checklist
- [ ] Visual: Spectrum bars render correctly
- [ ] Visual: All 4 modes work (Bars, Line, Filled, Circular)
- [ ] Audio: Spectrum responds to signal generator
- [ ] Audio: Spectrum responds to EQ changes
- [ ] Audio: Spectrum shows correct frequency distribution
- [ ] Performance: 60 FPS with spectrum updates
- [ ] Interaction: Mode selector works
- [ ] Error: Graceful handling if analyser fails

---

### Phase 4: Settings Panel
**Duration:** Week 4
**Effort:** Low
**Complexity:** Low
**Lines of Code:** ~250-350

#### Goals
- Add settings panel with theme selector
- Implement localStorage persistence
- Add display settings (zoom, FPS counter)
- Add "About" section

#### Tasks

##### Task 4.1: Implement Settings Panel UI
**Source Reference:** `src/main.rs:1863-1950` (draw_settings_panel_main)
**Guide Reference:** WASM_CODE_BORROWING_GUIDE.md (Priority 3)

```rust
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
                        let current_theme = self.theme_manager.current_theme().clone();
                        if ui.selectable_value(
                            &mut self.theme_manager.current_theme_mut(),
                            theme.clone(),
                            theme.display_name()
                        ).clicked() {
                            self.theme_manager.set_theme(theme);
                            // Save to localStorage
                            self.save_settings();
                        }
                    }
                });
        });

        ui.add_space(15.0);

        // Display Settings
        ui.group(|ui| {
            ui.label(RichText::new("🖥️ Display").strong());
            ui.add_space(5.0);

            ui.horizontal(|ui| {
                if ui.checkbox(&mut self.show_fps, "Show FPS counter").changed() {
                    self.save_settings();
                }
            });
        });

        ui.add_space(15.0);

        // About
        ui.group(|ui| {
            ui.label(RichText::new("ℹ️ About").strong());
            ui.add_space(5.0);
            ui.label("Rusty Audio Player (WASM)");
            ui.label(format!("Version: {}", env!("CARGO_PKG_VERSION")));
            ui.label("Built with Rust + egui + Web Audio API");
        });
    });
}
```

**Files to modify:**
- `src/web.rs` (add draw_settings_panel method)

**Acceptance Criteria:**
- ✅ Settings panel renders
- ✅ Theme selector works
- ✅ FPS counter toggle works

##### Task 4.2: Add LocalStorage Persistence

```rust
use web_sys::Storage;

impl WasmAudioApp {
    fn save_settings(&self) {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                // Save theme
                let theme_name = format!("{:?}", self.theme_manager.current_theme());
                let _ = storage.set_item("rusty_audio_theme", &theme_name);

                // Save FPS setting
                let _ = storage.set_item("rusty_audio_show_fps", &self.show_fps.to_string());

                log::info!("Settings saved to localStorage");
            }
        }
    }

    fn load_settings() -> (Theme, bool) {
        let mut theme = Theme::StudioDark;
        let mut show_fps = false;

        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                // Load theme
                if let Ok(Some(theme_str)) = storage.get_item("rusty_audio_theme") {
                    if let Some(loaded_theme) = Theme::from_name(&theme_str) {
                        theme = loaded_theme;
                    }
                }

                // Load FPS setting
                if let Ok(Some(fps_str)) = storage.get_item("rusty_audio_show_fps") {
                    show_fps = fps_str.parse().unwrap_or(false);
                }

                log::info!("Settings loaded from localStorage");
            }
        }

        (theme, show_fps)
    }
}

// Update Default impl to load settings
impl Default for WasmAudioApp {
    fn default() -> Self {
        let (theme, show_fps) = Self::load_settings();

        // ... rest of initialization ...

        Self {
            theme_manager: ThemeManager::new(theme),
            show_fps,
            // ... other fields ...
        }
    }
}
```

**Files to modify:**
- `src/web.rs` (add persistence methods)
- `src/ui/theme.rs` (add Theme::from_name if not exists)

**Acceptance Criteria:**
- ✅ Settings persist across page reloads
- ✅ Theme loads correctly on startup
- ✅ FPS counter setting loads correctly

##### Task 4.3: Add FPS Counter

```rust
// Add to WasmAudioApp struct
struct WasmAudioApp {
    // ... existing fields ...
    show_fps: bool,
    frame_count: u32,
    fps: f32,
    fps_timer: Instant,
}

// In update() method
fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    // FPS calculation
    self.frame_count += 1;
    let elapsed = self.fps_timer.elapsed().as_secs_f32();
    if elapsed >= 1.0 {
        self.fps = self.frame_count as f32 / elapsed;
        self.frame_count = 0;
        self.fps_timer = Instant::now();
    }

    // ... existing update code ...

    // Show FPS if enabled
    if self.show_fps {
        egui::Window::new("FPS")
            .fixed_pos([10.0, 10.0])
            .resizable(false)
            .show(ctx, |ui| {
                ui.label(format!("FPS: {:.1}", self.fps));
            });
    }
}
```

**Files to modify:**
- `src/web.rs` (add FPS counter)

**Acceptance Criteria:**
- ✅ FPS counter displays when enabled
- ✅ FPS updates every second
- ✅ Toggle works in settings

#### Testing Checklist
- [ ] Visual: Settings panel renders correctly
- [ ] Interaction: Theme selector changes theme
- [ ] Persistence: Settings persist across reload
- [ ] Persistence: Theme loads correctly on startup
- [ ] Functional: FPS counter shows/hides correctly
- [ ] Functional: FPS counter shows accurate values

---

### Phase 5: Polish & Testing
**Duration:** Week 5
**Effort:** Medium
**Complexity:** Medium
**Lines of Code:** ~200-350

#### Goals
- Add mobile-responsive layouts
- Optimize performance
- Comprehensive cross-browser testing
- Fix bugs and edge cases
- Update documentation

#### Tasks

##### Task 5.1: Mobile-Responsive Layouts

**Implement responsive detection:**

```rust
// In WasmAudioApp
fn is_mobile(&self, ctx: &egui::Context) -> bool {
    ctx.screen_rect().width() < 600.0
}

// Use mobile layouts
fn draw_eq_panel(&mut self, ui: &mut egui::Ui, colors: &ThemeColors) {
    if self.is_mobile(ui.ctx()) {
        self.draw_mobile_eq_panel(ui, colors);
    } else {
        self.draw_desktop_eq_panel(ui, colors);
    }
}
```

**Source Reference:** `src/main.rs:1014-1038` (draw_mobile_eq_panel)

**Files to modify:**
- `src/web.rs` (add mobile detection and layouts)

**Acceptance Criteria:**
- ✅ Desktop layout on wide screens
- ✅ Mobile layout on narrow screens
- ✅ Smooth transition between layouts

##### Task 5.2: Performance Optimization

**Optimize spectrum updates:**

```rust
// Only update spectrum at target rate (30-60 Hz)
const SPECTRUM_UPDATE_RATE: f32 = 30.0;  // Hz
let spectrum_interval = 1.0 / SPECTRUM_UPDATE_RATE;

if self.spectrum_update_timer.elapsed().as_secs_f32() >= spectrum_interval {
    if let Ok(_) = audio_manager.get_frequency_data(&mut self.spectrum_data) {
        self.spectrum_visualizer.update_frequency_data(&self.spectrum_data);
    }
    self.spectrum_update_timer = Instant::now();
}
```

**Reduce repaints:**

```rust
// Only request repaint when needed
if self.active_tab == AppTab::Effects || audio_manager.is_playing() {
    ctx.request_repaint();
} else {
    ctx.request_repaint_after(Duration::from_millis(100));
}
```

**Acceptance Criteria:**
- ✅ Consistent 60 FPS
- ✅ Low CPU usage when idle
- ✅ Smooth animations

##### Task 5.3: Cross-Browser Testing

**Test Matrix:**

| Browser | Version | Desktop | Mobile | Notes |
|---------|---------|---------|--------|-------|
| Chrome | Latest | ✅ | ✅ | Primary target |
| Firefox | Latest | ✅ | ✅ | Test Web Audio API |
| Safari | Latest | ✅ | ✅ | Test iOS compatibility |
| Edge | Latest | ✅ | ✅ | Chromium-based |

**Test Scenarios:**
1. Signal generator playback
2. EQ adjustment in real-time
3. Spectrum visualization
4. Theme switching
5. Settings persistence
6. Mobile touch interactions
7. PWA installation
8. Offline functionality (via service worker)

**Acceptance Criteria:**
- ✅ Works on all major browsers
- ✅ No console errors
- ✅ PWA installs correctly

##### Task 5.4: Bug Fixes & Edge Cases

**Common issues to address:**
- Audio context suspended on page load (requires user interaction)
- Race conditions in audio initialization
- Memory leaks in spectrum analyzer
- Theme switching glitches
- LocalStorage quota exceeded
- Mobile keyboard covering UI

**Acceptance Criteria:**
- ✅ Audio context resumes after user interaction
- ✅ No memory leaks after extended use
- ✅ Graceful error handling

##### Task 5.5: Documentation Updates

**Update files:**
- `CLAUDE.md` - Add WASM development section
- `README.md` - Add WASM deployment instructions
- `www/README.md` - Update icon generation guide
- Inline code documentation

**Acceptance Criteria:**
- ✅ Documentation reflects new features
- ✅ Build instructions updated
- ✅ Screenshots updated

#### Testing Checklist
- [ ] Performance: Consistent 60 FPS
- [ ] Performance: Low CPU when idle
- [ ] Cross-browser: Chrome ✅
- [ ] Cross-browser: Firefox ✅
- [ ] Cross-browser: Safari ✅
- [ ] Cross-browser: Edge ✅
- [ ] Mobile: Touch interactions work
- [ ] Mobile: Responsive layouts work
- [ ] PWA: Installs correctly
- [ ] PWA: Works offline
- [ ] Bugs: No console errors
- [ ] Bugs: No memory leaks
- [ ] Documentation: Updated

---

## Build & Deployment

### Development Build

```bash
# Install trunk if not already installed
cargo install trunk

# Serve locally with hot reload
trunk serve

# Open in browser
# http://localhost:8080
```

### Production Build

```bash
# Build optimized WASM
trunk build --release

# Output in dist/
ls -lh dist/

# Expected files:
# - index.html
# - rusty_audio.js
# - rusty_audio_bg.wasm
# - rusty_audio_bg.wasm.gz (compressed)
# - manifest.json
# - sw.js
```

### Deployment to GitHub Pages

```bash
# Build release
trunk build --release

# Deploy to gh-pages branch
git subtree push --prefix dist origin gh-pages
```

### Testing Deployment

```bash
# Test locally with production build
cd dist
python3 -m http.server 8000

# Open in browser
# http://localhost:8000
```

---

## Success Metrics

### Phase 1 Success
- ✅ Tab navigation works
- ✅ 4 tabs render without errors
- ✅ Smooth tab switching

### Phase 2 Success
- ✅ 8-band EQ fully functional
- ✅ Real-time EQ affects audio
- ✅ Reset functionality works

### Phase 3 Success
- ✅ Spectrum displays in real-time
- ✅ 4 visualization modes work
- ✅ 60 FPS performance

### Phase 4 Success
- ✅ Settings panel complete
- ✅ Theme switching works
- ✅ Settings persist across reloads

### Phase 5 Success
- ✅ Mobile-responsive
- ✅ Cross-browser compatible
- ✅ No critical bugs
- ✅ Documentation updated

---

## Risk Assessment

### Low Risk
- ✅ Tab navigation (simple UI change)
- ✅ Settings panel (mostly UI)
- ✅ Theme system (already working)

### Medium Risk
- ⚠️ Equalizer backend (Web Audio API complexity)
- ⚠️ Spectrum analyser (performance concerns)
- ⚠️ Mobile layouts (testing required)

### High Risk
- ❌ Web Audio API browser compatibility
- ❌ Performance on low-end devices
- ❌ Memory leaks with continuous spectrum updates

### Mitigation Strategies
1. **Browser compatibility:** Test early and often on all target browsers
2. **Performance:** Implement frame rate limiting and update throttling
3. **Memory leaks:** Use browser profiling tools to detect leaks
4. **Fallbacks:** Provide graceful degradation if features fail

---

## Code Quality Standards

### Rust Style
- ✅ Run `cargo fmt` before commits
- ✅ Run `cargo clippy -- -D warnings`
- ✅ No `unwrap()` in library code
- ✅ Proper error handling with `Result`

### Documentation
- ✅ Module-level documentation
- ✅ Function documentation for public APIs
- ✅ Inline comments for complex logic
- ✅ Code citations when borrowing from desktop app

### Testing
- ✅ Manual testing on all browsers
- ✅ Visual regression testing
- ✅ Performance benchmarking
- ✅ Error path testing

---

## Dependencies Summary

### Existing (No Changes Needed)
- ✅ `egui` - UI framework
- ✅ `eframe` - App framework with WASM support
- ✅ `web-sys` - Web API bindings
- ✅ `wasm-bindgen` - JS interop
- ✅ UI components in `src/ui/`

### New web-sys Features Required
```toml
[target.'cfg(target_arch = "wasm32")'.dependencies.web-sys]
features = [
    # Existing features...
    "AudioContext",
    "AudioNode",
    "AudioParam",

    # NEW - For EQ
    "BiquadFilterNode",
    "BiquadFilterType",

    # NEW - For Spectrum
    "AnalyserNode",

    # NEW - For Settings
    "Storage",  # localStorage
]
```

### No External Crates Needed
- ✅ All UI components already exist in `src/ui/`
- ✅ No new Rust dependencies required
- ✅ Only web-sys feature additions

---

## Final Deliverables

### Code
- ✅ Enhanced `src/web.rs` with all features
- ✅ Updated `src/audio/web_audio_backend.rs` with EQ and analyser
- ✅ Updated `src/integrated_audio_manager.rs` with new methods
- ✅ Updated `Cargo.toml` with new web-sys features

### Documentation
- ✅ WASM_IMPLEMENTATION_ANALYSIS.md (existing)
- ✅ WASM_CODE_BORROWING_GUIDE.md (existing)
- ✅ WASM_DEVELOPMENT_PLAN.md (this document)
- ✅ Updated CLAUDE.md
- ✅ Updated README.md

### Deployment
- ✅ Production-ready WASM build
- ✅ PWA manifest and service worker
- ✅ Deployment to GitHub Pages (optional)

---

## Timeline Summary

| Week | Phase | Deliverable | LOC |
|------|-------|-------------|-----|
| 1 | Tab Navigation | Working tab system | 150 |
| 2 | Equalizer | 8-band EQ with Web Audio backend | 450 |
| 3 | Spectrum | Real-time spectrum analyzer | 350 |
| 4 | Settings | Settings panel with persistence | 300 |
| 5 | Polish | Mobile layouts, testing, docs | 250 |
| **Total** | | **Complete WASM app** | **1,500** |

**End Result:** Feature-rich WASM audio application with EQ, spectrum visualization, and comprehensive settings, deployable as a PWA.

---

## Next Steps

1. **Immediate:** Review this plan and adjust priorities if needed
2. **Week 1:** Begin Phase 1 - Tab Navigation
3. **Weekly:** Review progress and update plan
4. **Week 5:** Deploy and announce

**Questions or concerns?** Review the detailed code citations in WASM_CODE_BORROWING_GUIDE.md for implementation specifics.
