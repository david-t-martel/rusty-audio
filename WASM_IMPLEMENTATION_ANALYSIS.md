# WASM Implementation Analysis Report - Rusty Audio

## Executive Summary

The rusty-audio project has a **partially implemented WASM deployment** with foundational infrastructure in place but limited feature porting. The desktop application (main.rs, ~2371 lines) is fully featured with 6 major tabs, but the WASM version (web.rs, ~335 lines) only implements signal generation and basic UI.

**Current Status:**
- ✅ WASM build infrastructure operational (Trunk.toml, .cargo config)
- ✅ Web Audio API backend integrated
- ✅ PWA infrastructure with service workers and manifests
- ✅ Basic signal generator working in WASM
- ❌ File playback not ported to WASM
- ❌ Audio recording not in WASM
- ❌ Equalizer UI not in WASM
- ❌ Spectrum visualizer not in WASM
- ❌ Full desktop audio processing pipeline missing

---

## 1. WASM-Specific Files and Directories

### Directory Structure

```
rusty-audio/
├── www/                          # PWA web assets
│   ├── index.html               # Main HTML entry (5.9KB) - with detailed WASM loader
│   ├── manifest.json            # PWA manifest (2.8KB)
│   ├── sw.js                    # Service worker (9.9KB) - full offline support
│   └── README.md                # Icon generation guide
│
├── static/                       # Trunk-managed static assets
│   ├── manifest.webmanifest     # Alternative manifest (612B)
│   ├── service-worker.js        # Simpler service worker (1.8KB)
│   ├── _headers                 # HTTP headers for deployment
│   └── icons/                   # Icon directory (placeholder)
│
├── index.html                    # Trunk build template (36 lines)
├── Trunk.toml                    # Trunk build configuration
├── src/
│   ├── web.rs                   # WASM entry point (335 lines)
│   ├── lib.rs                   # Library root (87 lines)
│   └── audio/
│       ├── web_audio_backend.rs # Web Audio API backend (277 lines)
│       └── web_audio_destination.rs # Web Audio output handler
│
└── .cargo/
    └── config.toml              # WASM build profiles and rustflags
```

### File Purposes

| File | Size | Purpose |
|------|------|---------|
| `www/index.html` | 5.9KB | Complex WASM loader with progress UI, PWA install prompt, error handling |
| `www/sw.js` | 9.9KB | Full-featured service worker with cache management, size limits, offline sync |
| `static/service-worker.js` | 1.8KB | Simplified service worker (network-first for HTML) |
| `src/web.rs` | 335 lines | WASM app entry point using IntegratedAudioManager |
| `src/audio/web_audio_backend.rs` | 277 lines | Web Audio API backend implementation |
| `Trunk.toml` | 18 lines | Build configuration (LTO disabled, release=true) |

---

## 2. WASM Build Configuration

### Trunk.toml (WASM Builder Configuration)

```toml
[build]
dist = "dist"
release = true
filehash = false

[build.rust]
release = true
rustflags = ["-C", "lto=off"]  # Disable LTO for cdylib compatibility
cargo-args = ["--lib"]          # Build library only (no binary)

[serve]
open = true
```

**Key Points:**
- LTO disabled for WASM (cdylib targets don't support LTO)
- Builds library only (not the native binary)
- Output goes to `dist/` directory
- Auto-opens browser on serve

### .cargo/config.toml (WASM-specific rustflags)

```toml
[target.wasm32-unknown-unknown]
rustflags = [
    "--cfg", "getrandom_backend=\"wasm_js\"",
    "-C", "embed-bitcode=yes",        # Required for wasm-opt
    "-C", "opt-level=z",              # Maximum size optimization
]

[profile.wasm-release]
inherits = "release"
opt-level = "z"                       # Maximum size reduction
lto = false                           # Must be false for cdylib
codegen-units = 1
```

### Cargo.toml (WASM-specific dependencies)

**WASM-only dependencies** (target-gated):
```rust
[target.'cfg(target_arch = "wasm32")'.dependencies]
wgpu = { version = "27", features = ["webgpu", "webgl"] }
wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"
js-sys = "0.3"
getrandom_02 = { package = "getrandom", version = "0.2", features = ["js"] }
getrandom = { version = "0.3", features = ["wasm_js"] }
web-sys = { version = "0.3", features = [
    "AudioContext",
    "AudioDestinationNode",
    "AudioNode",
    "AudioParam",
    "AudioBuffer",
    "AudioBufferSourceNode",
    "BaseAudioContext",
    "Window",
    "Document",
    "HtmlCanvasElement",
    "Navigator",
    "Performance",
    "console",
] }
console_error_panic_hook = "0.1"
console_log = "1"
```

**Desktop-only dependencies** (not in WASM):
```rust
[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
rfd = "0.14.1"                        # File dialogs
cpal = { version = "0.15", features = [] }  # Audio I/O
symphonia = { version = "0.5", features = ["all", "opt-simd"] }  # Audio decoding
rodio = { version = "0.17" }          # Audio playback
midir = "0.9"                         # MIDI I/O
hound = "3.5"                         # WAV recording
```

---

## 3. Desktop Application Architecture

### Main.rs Structure (2371 lines)

**Desktop-only App Structure:**
```rust
#[cfg(not(target_arch = "wasm32"))]
struct AudioPlayerApp {
    // Audio Engine (abstraction layer)
    audio_engine: Box<dyn AudioEngineInterface>,
    
    // Backend Systems (Phase 3.1)
    audio_backend: Option<HybridAudioBackend>,
    device_manager: Option<AudioDeviceManager>,
    web_audio_bridge: Option<WebAudioBridge>,
    
    // Playback State
    playback_state: PlaybackState,
    current_file: Option<Arc<FileHandle>>,
    metadata: Option<TrackMetadata>,
    volume: f32,
    panning: f32,
    is_looping: bool,
    playback_pos: Duration,
    total_duration: Duration,
    
    // UI Components (6 tabs)
    active_tab: Tab,
    signal_generator_panel: SignalGeneratorPanel,
    recording_panel: RecordingPanel,
    spectrum_visualizer: SpectrumVisualizer,
    
    // Accessibility & Enhanced Controls
    accessibility_manager: AccessibilityManager,
    accessible_volume_slider: AccessibleSlider,
    accessible_eq_knobs: Vec<AccessibleKnob>,
    volume_safety_indicator: VolumeSafetyIndicator,
    
    // Async Loading (Phase 1.4)
    async_loader: AsyncAudioLoader,
    tokio_runtime: Arc<tokio::runtime::Runtime>,
    load_progress: Option<f32>,
    
    // Dock Layout System (Phase 2.1)
    dock_layout_manager: DockLayoutManager,
    enable_dock_layout: bool,
    
    // Theme & UI State
    theme_manager: ThemeManager,
    layout_manager: LayoutManager,
    screen_size: ScreenSize,
}
```

**Desktop App Features (6 Tabs):**

| Tab | Purpose | Features | Portable? |
|-----|---------|----------|-----------|
| **Playback** | File loading & playback | File dialogs, metadata display, album art, progress bar | ⚠️ Partial (no file I/O in WASM) |
| **Effects** | Audio visualization | Real-time spectrum visualizer with FFT analysis | ✅ Yes (uses signal data) |
| **EQ** | 8-band parametric equalizer | Frequency bands 60Hz-7680Hz, gain sliders | ✅ Yes (UI portable) |
| **Generator** | Signal generation testing | Sine, square, sawtooth, noise, frequency sweep | ✅ Yes (implemented in WASM) |
| **Recording** | Audio capture & saving | Input device selection, format selection, monitoring | ❌ No (requires native I/O) |
| **Settings** | Configuration | Theme selection, accessibility options | ✅ Mostly yes |

**Key Desktop-Only Functions:**
```
fn draw_playback_panel()         - Uses rfd file dialogs
fn draw_recording_panel()        - Uses hound WAV recording
fn draw_effects_panel()          - Uses FFT analysis data
fn draw_eq_panel()               - Uses BiquadFilterNode
fn draw_generator_panel()        - Uses signal generators
fn draw_settings_panel_main()    - Theme & accessibility
```

---

## 4. WASM Entry Point Analysis

### src/web.rs Structure (335 lines)

**WASM-specific App:**
```rust
#[cfg(target_arch = "wasm32")]
struct WasmAudioApp {
    // Audio management (minimal)
    audio_manager: Option<IntegratedAudioManager>,
    initialization_error: Option<String>,
    
    // UI state (limited)
    signal_generator_panel: SignalGeneratorPanel,
    theme_manager: ThemeManager,
    volume: f32,
    error_message: Option<String>,
    last_update: Instant,
}
```

**Current WASM Features:**
1. ✅ Signal generator with routing
2. ✅ Master volume control
3. ✅ Basic status display
4. ✅ Error handling
5. ✅ System information display

**Missing WASM Features:**
1. ❌ File loading and playback
2. ❌ Spectrum visualizer
3. ❌ Equalizer controls
4. ❌ Recording capability
5. ❌ Metadata display
6. ❌ Settings panel
7. ❌ Accessibility features
8. ❌ Theme switching
9. ❌ Dock layout system
10. ❌ Advanced controls

**WebHandle (JavaScript Bridge):**
```rust
#[wasm_bindgen]
pub struct WebHandle {
    runner: eframe::WebRunner,
}

#[wasm_bindgen]
impl WebHandle {
    pub async fn start(&self, canvas: HtmlCanvasElement) -> Result<(), JsValue>
    pub fn destroy(&self)
    pub fn has_panicked(&self) -> bool
    pub fn panic_message(&self) -> Option<String>
}
```

---

## 5. Audio Backend Comparison

### Architecture: Trait-Based Backend Selection

**AudioBackend Trait (backend.rs):**
```rust
pub trait AudioBackend: Send + Sync {
    fn name(&self) -> &'static str;
    fn is_available(&self) -> bool;
    fn initialize(&mut self) -> Result<()>;
    fn enumerate_devices(&self, direction: StreamDirection) -> Result<Vec<DeviceInfo>>;
    fn create_output_stream(&mut self, device_id: &str, config: AudioConfig) 
        -> Result<Box<dyn AudioStream>>;
    fn create_input_stream(&mut self, device_id: &str, config: AudioConfig) 
        -> Result<Box<dyn AudioStream>>;
}
```

### Web Audio API Backend (web_audio_backend.rs)

**Implementation Status:**
- ✅ Output streams (AudioContext, AudioDestinationNode)
- ✅ Device enumeration (single "default" device)
- ✅ Initialization and context creation
- ❌ Input streams (requires getUserMedia - complex async)
- ❌ Device selection (Web Audio API abstraction)

**Key Limitation:**
Web Audio API doesn't expose device enumeration like native backends. WASM version simulates a single "default" device.

**Code Sample:**
```rust
#[cfg(target_arch = "wasm32")]
pub struct WebAudioBackend {
    context: Option<AudioContext>,
    initialized: bool,
}

impl AudioBackend for WebAudioBackend {
    fn enumerate_devices(&self, _direction: StreamDirection) -> Result<Vec<DeviceInfo>> {
        // Returns single "Web Audio Default" device
        Ok(vec![DeviceInfo {
            id: "default".to_string(),
            name: "Web Audio Default".to_string(),
            is_default: true,
            supported_configs: vec![/* 2-channel, 48kHz */],
        }])
    }
}
```

### Desktop Backends (for comparison)

**Native Backend Options:**
1. **CPAL Backend** (Cross-platform) - Linux, macOS, Windows (shared mode)
2. **ASIO Backend** (Windows-only) - Professional audio, ultra-low latency
3. **Hybrid Mode** - Automatic fallback between CPAL and Web Audio API

**Integrated Audio Manager (both platforms):**
```rust
pub struct IntegratedAudioManager {
    router: AudioRouter,
    backend: Box<dyn AudioBackend>,  // Polymorphic backend
    config: AudioConfig,
    
    // Sources and destinations
    signal_generator_source: Option<SourceId>,
    input_device_source: Option<SourceId>,
    file_playback_source: Option<SourceId>,
    output_device: Option<DestId>,
}
```

---

## 6. PWA Infrastructure (Partially Implemented)

### Service Workers (Two Versions)

**Option 1: www/sw.js (Production-Ready)**
- **Size:** 9.9KB
- **Caching Strategy:** Granular (separate caches for WASM, audio, static, runtime)
- **Cache Size Limits:** 
  - Audio cache: 100MB
  - Runtime cache: 50MB
- **Features:**
  - Cache-first for WASM
  - Network-first for HTML
  - Audio file caching with LRU eviction
  - Background sync for metadata
  - Push notification support
  - Cache management API

**Option 2: static/service-worker.js (Simplified)**
- **Size:** 1.8KB
- **Caching Strategy:** Simple (single cache)
- **Cache Size Limits:** None
- **Features:**
  - Network-first for HTML
  - Cache-first for static assets
  - Basic offline support

### PWA Manifests

**www/manifest.json (Full-Featured)**
- 127 lines
- Share target API (audio file sharing)
- Shortcuts (Quick access to "Play Music")
- Screenshots (desktop & mobile)
- Protocol handlers (web+audio://)
- Display overrides (window-controls-overlay)
- Edge side panel support

**static/manifest.webmanifest (Minimal)**
- 18 lines
- Basic app metadata
- Two icon sizes only

### HTML Entry Points

**www/index.html (Feature-Rich)**
- **Size:** 5.9KB
- **Features:**
  - Loading screen with progress bar
  - PWA install prompt
  - Browser compatibility checks
  - Service worker registration
  - Performance monitoring (Web Vitals API)
  - Panic handling
  - Online/offline handling
  - Custom JavaScript bootstrap

**index.html (Trunk Template)**
- **Size:** 36 lines
- **Features:**
  - Trunk directives for build
  - WASM-opt configuration
  - Static asset copying
  - Simple canvas setup

---

## 7. Desktop Features Not Yet Ported to WASM

### Critical Feature Gaps

| Feature | Desktop | WASM | Reason | Complexity |
|---------|---------|------|--------|------------|
| File Loading | ✅ | ❌ | No file dialog API in browser | High |
| File Playback | ✅ | ❌ | No native audio decoding API | High |
| Audio Recording | ✅ | ❌ | Requires getUserMedia + encoding | Very High |
| Spectrum Visualization | ✅ | ❌ | Requires input audio data | Medium |
| EQ Filters | ✅ (UI only) | ❌ | UI portable but needs audio data | Low |
| Metadata Display | ✅ | ❌ | Requires file loading | Medium |
| Album Art | ✅ | ❌ | Requires file loading | Medium |
| Accessibility Features | ✅ | ❌ | UI portable but complex | Medium |
| MIDI Support | ✅ | ❌ | Requires Web MIDI API | High |
| Settings Panel | ✅ | ❌ | UI portable but storage needed | Low |
| Dock Layout System | ✅ | ❌ | Complex state management | Medium |
| Async File Loading | ✅ | ❌ | Tokio runtime not applicable | N/A |

### Desktop-Only Code Sections

**In src/main.rs:**
- Lines 7-37: Platform-specific imports
- Lines 93-160: AudioPlayerApp struct (desktop only)
- Lines 162-290: Default impl (desktop initialization)
- Lines 295-450: eframe::App impl (desktop update loop)
- Lines 452-2307: All tab drawing functions
- Lines 1484-1862: Playback control functions
- Lines 2308: fn main() entry point

**In src/lib.rs:**
- audio_engine (uses web_audio_api)
- async_audio_loader (uses tokio)
- audio_pipeline_integration (uses AnalyserNode)
- audio_performance_integration (uses AnalyserNode)

---

## 8. Code Examples: Desktop vs WASM Differences

### Example 1: File Playback

**Desktop Version (src/main.rs):**
```rust
// File dialog (desktop only)
let file_handle = rfd::AsyncFileDialog::new()
    .add_filter("audio", &["mp3", "wav", "flac", "ogg"])
    .pick_file()
    .await;

// Load with async loader
if let Some(handle) = file_handle {
    self.async_loader.load_audio_file(handle.path(), config);
}
```

**WASM Version (src/web.rs):**
```rust
// No file dialog in WASM
// Would need HTML5 file input instead
// Browser doesn't provide automatic audio decoding API
// Would need JavaScript integration or Web Audio API workaround
```

### Example 2: Recording

**Desktop Version (src/main.rs):**
```rust
// Uses hound crate
let recording = AudioRecorder::new(RecordingConfig {
    format: RecordingFormat::Wav,
    bit_depth: 16,
    channels: 2,
    sample_rate: 48000,
});

// Records to file
recording.start_recording(output_path)?;
```

**WASM Version (src/web.rs):**
```rust
// Would need Web Audio API MediaRecorder API
// Complex async flow with getUserMedia
// Requires JavaScript integration
// Not currently implemented
```

### Example 3: Spectrum Visualization

**Desktop Version (src/main.rs):**
```rust
// Uses AnalyserNode from web_audio_api
fn draw_effects_panel(&mut self, ui: &mut egui::Ui, colors: &ThemeColors) {
    // Draws spectrum from audio_backend.analyser_node
    self.spectrum_visualizer.show(ui);
}
```

**WASM Version (src/web.rs):**
```rust
// Signal generator only - no spectrum data from source audio
// Could be added by:
// 1. Routing signal generator through Web Audio AnalyserNode
// 2. Reading frequency data from analyser
// 3. Passing to spectrum visualizer
// Currently not implemented
```

### Example 4: Audio Backend Selection

**Desktop Version (src/lib.rs):**
```rust
// Supports multiple backends
#[cfg(not(target_arch = "wasm32"))]
pub mod audio_engine;

// Can choose CPAL or Web Audio API or ASIO
let backend = BackendSelector::select_best_backend()?;
```

**WASM Version:**
```rust
// Only Web Audio API backend
#[cfg(target_arch = "wasm32")]
pub mod web_audio_backend;

// Single choice - no fallback needed
let backend = WebAudioBackend::new();
```

---

## 9. Build and Deployment Infrastructure

### Build Scripts Available

**Build Process:**
1. **Cargo.toml** - Main configuration
2. **Trunk.toml** - WASM-specific build (uses `trunk build`)
3. **.cargo/config.toml** - Compiler flags and profiles
4. **Makefile.toml** - Task automation (likely)

### Build Commands

```bash
# Native desktop build
cargo build --release
cargo run --release

# WASM build (via Trunk)
trunk build
trunk build --release
trunk serve  # Local testing with hot reload

# WASM via Cargo
cargo build --target wasm32-unknown-unknown --profile wasm-release --lib
```

### Output Structure (dist/)

```
dist/
├── index.html                 # Trunk-generated HTML
├── rusty_audio.js            # JavaScript bindings
├── rusty_audio_bg.wasm       # WebAssembly binary
├── rusty_audio_bg.wasm.gz    # Compressed WASM
├── rusty_audio_bg.wasm.br    # Brotli WASM
├── manifest.json             # PWA manifest
├── sw.js                      # Service worker
└── icons/                     # PWA icons (not yet present)
```

---

## 10. Recommendations for Next Porting Steps

### Priority 1: Core Playback (Medium Effort, High Impact)

**Goal:** Enable basic audio playback in WASM

**Tasks:**
1. Add HTML5 file input element for audio files
2. Use Web Audio API to decode audio (API limitations - limited format support)
3. Create file-based audio source in audio routing system
4. Implement play/pause/stop controls
5. Add progress tracking

**Est. Effort:** 2-3 weeks
**Depends on:** Web Audio API limitations for decoding

**Blockers:**
- Web Audio API doesn't natively decode MP3/FLAC (only WAV/raw PCM)
- Would need JavaScript integration with decoder library (e.g., id3.js)
- Cross-origin limitations

### Priority 2: Equalizer UI (Low Effort, Medium Impact)

**Goal:** Port EQ tab to WASM

**Tasks:**
1. Copy eq_panel drawing code from main.rs
2. Create BiquadFilterNode chains in Web Audio API
3. Connect to signal generator source
4. Add filter visualizer (can use egui slider visualization)
5. Test with generated test signals

**Est. Effort:** 1 week
**Depends on:** Signal generator (already in WASM)
**Complexity:** Low - mostly UI copying

### Priority 3: Spectrum Visualization (Medium Effort, High Impact)

**Goal:** Add real-time spectrum display

**Tasks:**
1. Access Web Audio API AnalyserNode from output
2. Get frequency data from analyser
3. Implement FFT visualization in spectrum_visualizer.rs
4. Add to WASM UI update loop
5. Sync with signal generator

**Est. Effort:** 1-2 weeks
**Depends on:** Audio routing system
**Complexity:** Medium - FFT integration

### Priority 4: Recording (High Effort, Very High Impact)

**Goal:** Enable audio recording in WASM

**Tasks:**
1. Request user permission via getUserMedia
2. Create audio track from input device
3. Use MediaRecorder API for recording
4. Implement WAV/Ogg export
5. Add download functionality

**Est. Effort:** 3-4 weeks
**Depends on:** User permissions, browser APIs
**Complexity:** Very High - async flows, browser APIs

### Priority 5: Settings & Accessibility (Low Effort, Low Impact)

**Goal:** Port settings panel and accessibility features

**Tasks:**
1. Copy theme manager code
2. Add localStorage persistence
3. Port accessibility features
4. Add keyboard shortcut support
5. Test with screen readers (limited in WASM)

**Est. Effort:** 1-2 weeks
**Depends on:** Nothing blocking
**Complexity:** Low

### Priority 6: File Loading UI (Low Effort, Low Impact)

**Goal:** Add file selection interface

**Tasks:**
1. Create file input HTML element
2. Handle file drop events
3. Add progress indicator
4. Display file metadata
5. Integration with playback system

**Est. Effort:** 1 week (but depends on playback implementation)
**Depends on:** Priority 1
**Complexity:** Low

---

## 11. Build Configuration Details

### Cargo.toml Feature Flags

```rust
[features]
default = ["audio-optimizations", "native-binary"]
native-binary = []             # Gates native binary (disabled for WASM)
audio-optimizations = []       # Real-time audio thread priority
property-testing = ["proptest", "quickcheck"]  # Property tests
```

**How WASM uses these:**
```bash
# Native build (includes native-binary feature)
cargo build --release

# WASM build (excludes native-binary feature)
cargo build --target wasm32-unknown-unknown --lib --no-default-features \
  --features=audio-optimizations

# Via Trunk (handles this automatically)
trunk build --release
```

### Profile Configuration

```toml
[profile.release]
opt-level = 3
lto = "fat"              # Full LTO for native
codegen-units = 1
panic = "abort"
strip = true

[profile.wasm-release]
inherits = "release"
lto = false              # Must be false for WASM cdylib
opt-level = "z"          # Size optimization
codegen-units = 1
```

---

## 12. Summary: What Exists vs What's Missing

### Infrastructure Status
- ✅ **Build System:** Trunk + Cargo fully configured
- ✅ **Backend Abstraction:** AudioBackend trait with Web Audio impl
- ✅ **PWA Setup:** Manifests, service workers, install prompt
- ✅ **WASM Initialization:** WebHandle with proper lifecycle
- ✅ **Error Handling:** Console logging, panic hooks
- ⚠️ **Audio Routing:** IntegratedAudioManager supports both platforms but limited on WASM

### Feature Porting Status
- ✅ **Signal Generator:** Fully implemented
- ✅ **Theme System:** Basic theming available
- ✅ **Volume Control:** Master volume slider
- ⚠️ **UI Components:** Most portable but not integrated
- ❌ **File I/O:** No file loading capability
- ❌ **Audio Decoding:** Limited format support
- ❌ **Recording:** No recording infrastructure
- ❌ **Spectrum Analysis:** Not connected to audio input
- ❌ **MIDI:** No Web MIDI API integration

### Effort Estimate (Remaining Work)

| Feature | Lines Est. | Weeks | Priority |
|---------|-----------|-------|----------|
| File Playback | 300-500 | 3 | P1 |
| EQ Tab | 200-300 | 1 | P2 |
| Spectrum Viz | 150-250 | 1.5 | P3 |
| Recording | 400-600 | 4 | P4 |
| Settings Panel | 200-300 | 1.5 | P5 |
| File Loading UI | 100-150 | 1 | P6 |
| **Total** | **1350-2100** | **12 weeks** | |

---

## 13. Critical Blockers and Considerations

### Browser API Limitations

1. **No Native Audio Decoding**
   - Web Audio API: Accepts PCM only
   - Must use JavaScript library (ffmpeg.js, decoder libraries)
   - Adds 500KB-2MB to bundle size

2. **Limited Format Support**
   - MP3: Requires external decoder
   - FLAC: Requires external decoder  
   - WAV: Natively supported (PCM only)
   - OGG: Requires external decoder

3. **Input Access (getUserMedia)**
   - Requires user permission
   - HTTPS only (except localhost)
   - Cross-origin restrictions
   - Async permission flow

4. **File System Access**
   - No direct file access (security)
   - File input only
   - Download only (via Blob API)

### WASM Bundle Size

**Current Estimates:**
- Unoptimized WASM: 2-3MB
- Optimized (wasm-opt): 1-1.5MB
- Compressed (brotli): 300-500KB
- With decoder libs: +500KB-2MB

**Mitigation:**
- Enable feature flags for optional decoders
- Use dynamic imports for recording feature
- Lazy load visualization libraries

### Testing Challenges

1. **No native file access** - Cannot test file loading easily
2. **No real audio devices** - Cannot test recording without user permission
3. **Browser specific** - Performance varies by browser
4. **ServiceWorker debugging** - Limited tooling

---

## 14. Next Immediate Steps

### For the Next Sprint

1. **Integrate EQ UI into WASM** (1 week)
   - Copy `draw_eq_panel()` to web.rs
   - Connect BiquadFilterNodes to signal generator
   - Update WASM app struct to include eq_knobs

2. **Add Spectrum Visualization** (1-2 weeks)
   - Enable AnalyserNode access in Web Audio backend
   - Update spectrum visualizer to accept external frequency data
   - Connect to signal generator output monitoring

3. **Create Feature Checklist**
   - Document each feature's browser API requirements
   - Assess bundle size impact
   - Plan async code boundaries

4. **Establish Testing Matrix**
   - Browser versions to test (Chrome, Firefox, Safari, Edge)
   - Device types (desktop, tablet, mobile)
   - Network conditions (fast, slow, offline)

