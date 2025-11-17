# Audio Backend Architecture - Visual Reference

## System Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Application Layer                           │
│  ┌─────────────┐  ┌──────────────┐  ┌────────────────────────┐    │
│  │   UI (egui) │──│ Main Window  │──│ Audio Controls Panel   │    │
│  └──────┬──────┘  └──────────────┘  └────────────────────────┘    │
└─────────┼──────────────────────────────────────────────────────────┘
          │
          │ Commands (play, stop, adjust EQ, etc.)
          │
┌─────────▼──────────────────────────────────────────────────────────┐
│              IntegratedAudioManager                                 │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │  State Management:                                           │  │
│  │  - Playback state (playing/paused/stopped)                   │  │
│  │  - Active routes (source → destination mappings)             │  │
│  │  - Device configuration                                      │  │
│  └──────────────────────────────────────────────────────────────┘  │
└─────────┬────────────────────────────────┬─────────────────────────┘
          │                                │
          │                                │
┌─────────▼─────────────┐        ┌─────────▼──────────────────┐
│    AudioRouter        │        │   Box<dyn AudioBackend>    │
│  ┌─────────────────┐  │        │  ┌──────────────────────┐  │
│  │ Sources:        │  │        │  │ Backend Selection:   │  │
│  │ - File player   │  │        │  │ - ASIO (Windows)     │  │
│  │ - Microphone    │  │        │  │ - CPAL (Cross-plat)  │  │
│  │ - Generator     │  │        │  │ - WebAudio (WASM)    │  │
│  └─────────────────┘  │        │  │ - Hybrid (Fallback)  │  │
│                       │        │  └──────────────────────┘  │
│  ┌─────────────────┐  │        └─────────┬──────────────────┘
│  │ Destinations:   │  │                  │
│  │ - Hardware out  │  │                  │ Device I/O
│  │ - File recorder │  │                  │
│  │ - Level meter   │  │        ┌─────────▼──────────────────┐
│  └─────────────────┘  │        │   Hardware/Browser Audio   │
│                       │        │  ┌──────────────────────┐  │
│  ┌─────────────────┐  │        │  │ Physical Devices:    │  │
│  │ Routes:         │  │        │  │ - Sound card         │  │
│  │ Source → Dest   │  │        │  │ - ASIO interface     │  │
│  │ with gain       │  │        │  │ - Browser audio      │  │
│  └─────────────────┘  │        │  └──────────────────────┘  │
└───────────────────────┘        └────────────────────────────┘
```

---

## Correct Audio Signal Flow (PR #5 Fix)

### BROKEN Flow (Current PR #5 Bug)
```
┌──────────┐
│  Source  │─────────────────────────────┐
└──────────┘                             │
                                         │
┌────────────────┐                       │
│ EQ (8 bands)   │  ← NOT CONNECTED!    │
│ (Created but   │                       │
│  not in path)  │                       │
└────────────────┘                       │
                                         │
┌─────────────┐                          │
│  Analyser   │  ← NOT CONNECTED!       │
│  (No data)  │                          │
└─────────────┘                          │
                                         │
                                         ▼
                                  ┌──────────┐
                                  │  Output  │
                                  └──────────┘
Result: EQ adjustments have NO EFFECT!
```

### FIXED Flow (Required Connection Pattern)
```
┌──────────┐
│  Source  │
└────┬─────┘
     │
     ▼
┌────────────────────────────────────┐
│  8-Band Parametric Equalizer       │
│  ┌──────┐ ┌──────┐ ... ┌──────┐  │
│  │ 60Hz │→│120Hz │ ... │7680Hz│  │
│  └──────┘ └──────┘     └──────┘  │
│  Each band: Gain ±12dB, Q=1.0     │
└────┬───────────────────────────────┘
     │
     ▼
┌─────────────────────────────────────┐
│  Spectrum Analyser                  │
│  - 512 FFT bins                     │
│  - Post-EQ signal                   │
│  - Real-time frequency display      │
└────┬────────────────────────────────┘
     │
     ▼
┌────────────────────────────┐
│  Master Gain (Volume)      │
│  Range: 0.0 to 1.0 (linear)│
└────┬───────────────────────┘
     │
     ▼
┌──────────┐
│  Output  │
└──────────┘

Result: EQ adjustments AFFECT audio output ✅
```

---

## Backend Trait Hierarchy

### BROKEN Design (Not Dyn-Safe)
```
pub trait AudioBackend {
    fn create_stream<F>(callback: F) -> Result<Stream>
    where F: FnMut(&mut [f32]);
        ↑
        └─── Generic parameter prevents trait object!
}

❌ CANNOT DO THIS:
let backend: Box<dyn AudioBackend> = ...;
              ^^^^^^^^^^^^^^^^^^^
              Error: trait not dyn compatible
```

### FIXED Design (Dyn-Safe)
```
pub type OutputCallback = Box<dyn FnMut(&mut [f32]) + Send>;

pub trait AudioBackend {
    fn create_stream(callback: OutputCallback) -> Result<Stream>;
                              ^^^^^^^^^^^^^^
                              Concrete type - vtable-compatible!

    fn as_any(&self) -> &dyn Any;  // Downcasting support
}

✅ CAN DO THIS:
let backend: Box<dyn AudioBackend> = Box::new(CpalBackend::new());
              ^^^^^^^^^^^^^^^^^^^
              Works! Trait is dyn-safe
```

---

## Backend Implementation Relationships

```
┌──────────────────────────────────────────────────────────┐
│              AudioBackend (trait)                        │
│  + create_output_stream()                                │
│  + create_input_stream()                                 │
│  + create_output_stream_with_callback()                  │
│  + enumerate_devices()                                   │
│  + as_any() / as_any_mut()  ← Downcasting support       │
└────────────┬─────────────────────────────────────────────┘
             │ implements
             │
   ┌─────────┼────────────┬───────────────┬─────────────┐
   │         │            │               │             │
   ▼         ▼            ▼               ▼             ▼
┌────────┐ ┌─────────┐ ┌──────────┐ ┌──────────┐ ┌─────────┐
│ Cpal   │ │  Asio   │ │ WebAudio │ │  Hybrid  │ │ Future  │
│Backend │ │Backend  │ │ Backend  │ │  Backend │ │Backends │
└────────┘ └─────────┘ └──────────┘ └──────────┘ └─────────┘
    │           │            │             │            │
    │           │            │             │            │
    ▼           ▼            ▼             ▼            ▼
┌────────┐ ┌─────────┐ ┌──────────┐ ┌──────────┐ ┌─────────┐
│ WASAPI │ │  ASIO   │ │ Web      │ │ Dual     │ │ JACK    │
│CoreAudio│ │DirectSound│ Audio   │ │ Backend  │ │PipeWire │
│  ALSA  │ │ Drivers │ │   API    │ │ System   │ │  etc.   │
└────────┘ └─────────┘ └──────────┘ └──────────┘ └─────────┘
```

---

## Thread Safety Architecture

### Native Platforms (CPAL/ASIO)
```
┌──────────────────────────────────────────────────────┐
│                  Main Thread                         │
│  ┌────────────────────────────────────────────┐     │
│  │  Create callback:                          │     │
│  │  let cb: OutputCallback = Box::new(...)    │     │
│  └────────────────────┬───────────────────────┘     │
└───────────────────────┼──────────────────────────────┘
                        │
                        │ Wrap in Arc<Mutex<>>
                        │
                        ▼
              ┌──────────────────────┐
              │ Arc<Mutex<Callback>> │
              └──────────┬───────────┘
                         │
              ┌──────────┴────────────┐
              │                       │
              │ clone                 │ clone
              │                       │
     ┌────────▼────────┐    ┌────────▼────────┐
     │   Main Thread   │    │  Audio Thread   │
     │   (owns Arc)    │    │  (owns Arc)     │
     └─────────────────┘    └────────┬────────┘
                                     │
                         ┌───────────▼───────────┐
                         │ Audio Callback:       │
                         │ let mut cb = arc.lock()│
                         │ cb(buffer)            │
                         └───────────────────────┘

Key: Arc<Mutex<>> allows sharing between threads
```

### WASM Platform (Web Audio)
```
┌──────────────────────────────────────────────────────┐
│                  Main Thread (ONLY)                  │
│  ┌────────────────────────────────────────────┐     │
│  │  Create callback:                          │     │
│  │  let cb: OutputCallback = Box::new(...)    │     │
│  └────────────────────┬───────────────────────┘     │
│                       │                              │
│                       │ Wrap in Rc<RefCell<>>       │
│                       │ (No Send/Sync required!)    │
│                       ▼                              │
│              ┌──────────────────────┐               │
│              │ Rc<RefCell<Callback>>│               │
│              └──────────┬───────────┘               │
│                         │                            │
│              ┌──────────┴────────────┐              │
│              │                       │              │
│              │ clone                 │ clone        │
│              │                       │              │
│     ┌────────▼────────┐    ┌────────▼────────┐    │
│     │   Rust Code     │    │ JS Closure      │    │
│     │   (owns Rc)     │    │ (owns Rc)       │    │
│     └─────────────────┘    └────────┬────────┘    │
│                                     │              │
│                         ┌───────────▼───────────┐ │
│                         │ onaudioprocess:       │ │
│                         │ let mut cb = rc.borrow_mut()│
│                         │ cb(buffer)            │ │
│                         └───────────────────────┘ │
└──────────────────────────────────────────────────────┘

Key: Single-threaded, no Arc/Mutex needed
```

---

## Hybrid Backend Mode Switching

```
┌────────────────────────────────────────────────────────┐
│           HybridAudioBackend State Machine             │
└────────────────────────────────────────────────────────┘

Mode: WebAudioOnly
┌──────────────┐
│ Web Audio    │──→ Browser Audio Output
│ Context      │
└──────────────┘
  ↑
  └── WASM target only


Mode: CpalOnly
┌──────────────┐
│ CPAL Backend │──→ Hardware Audio Output
└──────────────┘
  ↑
  └── Native platforms, direct hardware


Mode: HybridNative ⭐ (Most Complex)
┌───────────────┐      ┌─────────────┐      ┌──────────────┐
│ Web Audio     │──→   │ Ring Buffer │──→   │ CPAL Stream  │──→ Hardware
│ Processing    │      │ (lock-free) │      │              │
│ (EQ, effects) │      │ 8x buffered │      │ (native I/O) │
└───────────────┘      └─────────────┘      └──────────────┘
                              │
                              │ Health Monitoring
                              │
                       ┌──────▼────────┐
                       │ BackendHealth │
                       │ - Healthy     │
                       │ - Degraded    │
                       │ - Failed      │
                       └───────────────┘
                              │
                       ┌──────▼────────────┐
                       │ Fallback Trigger  │
                       │ - 3x underrun →   │
                       │   Degraded        │
                       │ - 10x underrun →  │
                       │   Switch mode     │
                       └───────────────────┘
```

---

## Device Enumeration Flow

```
┌─────────────────┐
│ User Action:    │
│ "List devices"  │
└────────┬────────┘
         │
         ▼
┌───────────────────────────────────────┐
│ backend.enumerate_devices(Output)     │
└────────┬──────────────────────────────┘
         │
         ▼
┌───────────────────────────────────────┐
│ Platform-specific device query:       │
│                                       │
│ Windows: WASAPI/ASIO/DirectSound      │
│ macOS:   CoreAudio                    │
│ Linux:   ALSA/PulseAudio              │
│ WASM:    Web Audio API (single device)│
└────────┬──────────────────────────────┘
         │
         ▼
┌───────────────────────────────────────┐
│ For each device, build DeviceInfo:   │
│ ┌───────────────────────────────────┐│
│ │ DeviceInfo {                      ││
│ │   id: "Device-123",               ││
│ │   name: "Speakers (Realtek)",     ││
│ │   is_default: true,               ││
│ │   supported_configs: [            ││
│ │     AudioConfig {                 ││
│ │       sample_rate: 48000,         ││
│ │       channels: 2,                ││
│ │       buffer_size: 512,           ││
│ │       ...                         ││
│ │     }                             ││
│ │   ],                              ││
│ │   min_sample_rate: 44100,         ││
│ │   max_sample_rate: 192000,        ││
│ │   ...                             ││
│ │ }                                 ││
│ └───────────────────────────────────┘│
└────────┬──────────────────────────────┘
         │
         ▼
┌───────────────────────────────────────┐
│ Return Vec<DeviceInfo> to caller      │
└───────────────────────────────────────┘
         │
         ▼
┌───────────────────────────────────────┐
│ UI displays device list:              │
│ ┌───────────────────────────────────┐│
│ │ ● Speakers (Realtek) [Default]    ││
│ │   ASIO Interface (Focusrite)      ││
│ │   Headphones (USB)                ││
│ └───────────────────────────────────┘│
└───────────────────────────────────────┘
```

---

## Stream Creation with Callback Flow

```
┌─────────────────────────────────────────┐
│ UI Thread: Create audio stream          │
└────────┬────────────────────────────────┘
         │
         ▼
┌────────────────────────────────────────────────────────┐
│ 1. Define callback closure                             │
│    let callback: OutputCallback = Box::new(|data| {   │
│        // Fill buffer with audio samples               │
│        for sample in data.iter_mut() {                 │
│            *sample = generate_sample();                │
│        }                                               │
│    });                                                 │
└────────┬───────────────────────────────────────────────┘
         │
         ▼
┌────────────────────────────────────────────────────────┐
│ 2. Call backend trait method                           │
│    backend.create_output_stream_with_callback(         │
│        device_id,                                      │
│        config,                                         │
│        callback  ← Ownership transferred               │
│    )                                                   │
└────────┬───────────────────────────────────────────────┘
         │
         ▼
┌────────────────────────────────────────────────────────┐
│ 3. Backend wraps callback for thread safety            │
│    let callback = Arc::new(Mutex::new(callback));      │
│    let callback_clone = callback.clone();              │
└────────┬───────────────────────────────────────────────┘
         │
         ▼
┌────────────────────────────────────────────────────────┐
│ 4. Build platform-specific stream                      │
│    device.build_output_stream(                         │
│        &config,                                        │
│        move |data: &mut [f32], _| {  ← Audio callback │
│            let mut cb = callback_clone.lock();         │
│            cb(data);  ← Call user's closure           │
│        },                                              │
│        error_callback,                                 │
│        None                                            │
│    )                                                   │
└────────┬───────────────────────────────────────────────┘
         │
         ▼
┌────────────────────────────────────────────────────────┐
│ 5. Return stream wrapped in trait object               │
│    Ok(Box::new(CpalOutputStream {                      │
│        stream,  ← Platform stream                     │
│        config,                                         │
│        status: Stopped,                                │
│    }))                                                 │
└────────┬───────────────────────────────────────────────┘
         │
         ▼
┌────────────────────────────────────────────────────────┐
│ 6. UI calls stream.play()                              │
│    stream.play()?;                                     │
└────────┬───────────────────────────────────────────────┘
         │
         ▼
┌────────────────────────────────────────────────────────┐
│ 7. Audio thread starts calling callback at buffer rate │
│    Every ~11.6ms (512 samples @ 44.1kHz):              │
│    ┌────────────────────────────────┐                 │
│    │ Audio Thread                   │                 │
│    │ → Lock callback                │                 │
│    │ → Call closure                 │                 │
│    │ → Fill buffer                  │                 │
│    │ → Unlock                       │                 │
│    │ → Send to hardware             │                 │
│    └────────────────────────────────┘                 │
└────────────────────────────────────────────────────────┘
```

---

## Error Recovery and Fallback Flow

```
┌──────────────────────────┐
│ Initial State:           │
│ Backend = ASIO           │
│ Health = Healthy         │
└────────┬─────────────────┘
         │
         │ ❌ Error: Stream underrun
         │
         ▼
┌──────────────────────────┐
│ Underrun detected        │
│ underrun_count++         │
└────────┬─────────────────┘
         │
         │ underrun_count == 3?
         │
         ▼
    ┌────────┐
    │  Yes   │
    └───┬────┘
        │
        ▼
┌────────────────────────────┐
│ Health → Degraded          │
│ Log warning                │
└────────┬───────────────────┘
         │
         │ underrun_count == 10?
         │
         ▼
    ┌────────┐
    │  Yes   │
    └───┬────┘
        │
        ▼
┌──────────────────────────────┐
│ Health → Failed              │
│ Trigger fallback             │
└────────┬─────────────────────┘
         │
         ▼
┌──────────────────────────────┐
│ FallbackPolicy check:        │
│ - Manual → Return error      │
│ - AutoOnError → Continue     │
└────────┬─────────────────────┘
         │
         ▼
┌──────────────────────────────┐
│ Determine fallback backend:  │
│ ASIO → WASAPI                │
│ WASAPI → DirectSound         │
│ DirectSound → WebAudio       │
└────────┬─────────────────────┘
         │
         ▼
┌──────────────────────────────┐
│ Switch to fallback backend   │
│ backend.set_mode(WASAPI)     │
└────────┬─────────────────────┘
         │
         ▼
┌──────────────────────────────┐
│ Reinitialize stream          │
│ Reset health → Healthy       │
│ Reset underrun_count = 0     │
└────────┬─────────────────────┘
         │
         ▼
┌──────────────────────────────┐
│ Continue playback            │
│ Monitor for new errors       │
└──────────────────────────────┘
```

---

## Backend Downcasting Pattern

```
┌─────────────────────────────────────────────┐
│ Generic backend reference                   │
│ let backend: Box<dyn AudioBackend> = ...;   │
└────────┬────────────────────────────────────┘
         │
         │ Need ASIO-specific features?
         │
         ▼
┌─────────────────────────────────────────────┐
│ Attempt downcast to AsioBackend             │
│ if let Some(asio) =                         │
│     backend.downcast_ref::<AsioBackend>() { │
└────────┬────────────────────────────────────┘
         │
    ┌────┴────┐
    │         │
   Yes       No
    │         │
    ▼         ▼
┌─────────┐ ┌──────────────────────┐
│ Success │ │ Try other types or   │
└────┬────┘ │ use generic interface│
     │      └──────────────────────┘
     ▼
┌──────────────────────────────────────┐
│ Access ASIO-specific methods:        │
│ - asio.backend_type()                │
│ - asio.set_exclusive_mode(true)      │
│ - asio.available_backends()          │
└──────────────────────────────────────┘

Example:
┌──────────────────────────────────────────────┐
│ let backend: Box<dyn AudioBackend> =         │
│     Box::new(AsioBackend::new());            │
│                                              │
│ // Generic trait methods work               │
│ backend.enumerate_devices(Output)?;          │
│                                              │
│ // Downcast for ASIO-specific features      │
│ if let Some(asio) =                          │
│     backend.downcast_ref::<AsioBackend>() {  │
│     println!("Using: {}",                    │
│         asio.backend_type().name());         │
│     asio.set_exclusive_mode(true);           │
│ }                                            │
└──────────────────────────────────────────────┘
```

---

## Memory Layout: Trait Objects vs Concrete Types

### Concrete Type (Stack Allocated)
```
┌─────────────────────────────────┐
│ CpalBackend instance            │ ← On stack
│ ┌─────────────────────────────┐ │
│ │ host: cpal::Host            │ │
│ │ initialized: bool           │ │
│ └─────────────────────────────┘ │
└─────────────────────────────────┘

Size: ~24 bytes (known at compile time)
```

### Trait Object (Heap Allocated)
```
Stack:
┌─────────────────────────────────┐
│ Box<dyn AudioBackend>           │
│ ┌─────────────────────────────┐ │
│ │ data_ptr   ───┐             │ │ ← Points to heap
│ │ vtable_ptr ───┼──┐          │ │ ← Points to vtable
│ └───────────────┼──┼──────────┘ │
└─────────────────┼──┼────────────┘
                  │  │
Heap:             │  │
┌─────────────────▼──┼────────────┐
│ CpalBackend instance│            │
│ ┌─────────────────┐ │            │
│ │ host: cpal::Host│ │            │
│ │ initialized     │ │            │
│ └─────────────────┘ │            │
└─────────────────────┼────────────┘
                      │
VTable:               │
┌─────────────────────▼────────────┐
│ Function pointers for:           │
│ - create_output_stream()         │
│ - create_input_stream()          │
│ - enumerate_devices()            │
│ - etc.                           │
└──────────────────────────────────┘

Size: 16 bytes on stack (2 pointers)
      + concrete type size on heap
      + vtable (shared across instances)
```

---

## Compilation Dependency Graph

```
┌───────────────────────────────────────────────────┐
│               Cargo Dependencies                  │
└───────────────────────────────────────────────────┘

Required for all platforms:
┌────────┐   ┌─────────┐   ┌───────────┐
│ anyhow │   │thiserror│   │parking_lot│
└────────┘   └─────────┘   └───────────┘
     │            │              │
     └────────────┴──────────────┘
                  │
                  ▼
         ┌──────────────────┐
         │ audio::backend   │ ← Core trait definitions
         └────────┬─────────┘
                  │
    ┌─────────────┼─────────────┐
    │             │             │
    ▼             ▼             ▼
┌────────┐  ┌──────────┐  ┌──────────┐
│  cpal  │  │   web-   │  │  hybrid  │
│        │  │  audio-  │  │          │
└────────┘  │    api   │  └──────────┘
    │       └──────────┘       │
    │             │            │
    │             │            │
    └─────────────┴────────────┘
                  │
                  ▼
         ┌──────────────────┐
         │ IntegratedAudio  │
         │    Manager       │
         └────────┬─────────┘
                  │
                  ▼
         ┌──────────────────┐
         │    Main App      │
         └──────────────────┘

Windows-specific:
┌──────────────────────────────────┐
│ [target.'cfg(windows)']          │
│ windows = { features = [         │
│   "Win32_Foundation",            │ ← For HANDLE
│   "Win32_System_Threading",      │ ← For MMCSS
│ ]}                               │
└──────────────────────────────────┘
```

---

## Summary: Key Architectural Points

1. **Dyn-Safe Trait Design**
   - Use `Box<dyn FnMut>` instead of generic `F`
   - Enables `Box<dyn AudioBackend>` polymorphism

2. **Correct Audio Graph**
   - Source → EQ → Analyser → Gain → Output
   - NOT Source → Output (PR #5 bug)

3. **Thread Safety**
   - Native: `Arc<Mutex<>>` for callback sharing
   - WASM: `Rc<RefCell<>>` for single-threaded

4. **Error Recovery**
   - Automatic fallback on persistent errors
   - Health monitoring with degradation levels

5. **Platform Support**
   - Windows: ASIO/WASAPI/DirectSound
   - macOS: CoreAudio
   - Linux: ALSA/PulseAudio
   - WASM: Web Audio API

---

For detailed implementation, see:
- **AUDIO_BACKEND_ARCHITECTURE.md** - Full specification
- **IMPLEMENTATION_CHECKLIST.md** - Step-by-step fixes
- **ARCHITECTURE_SUMMARY.md** - Executive summary

