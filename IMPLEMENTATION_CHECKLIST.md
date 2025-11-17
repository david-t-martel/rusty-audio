# Audio Backend Fix Implementation Checklist

**Goal:** Fix broken main branch compilation and PR #5 bugs
**Document:** See AUDIO_BACKEND_ARCHITECTURE.md for full details

## Critical Fixes (Immediate - Must Compile)

### ✅ Fix 1: MMCSS HANDLE Import Error

**File:** `src/audio/mmcss.rs`
**Lines:** 85, 160

```rust
// BEFORE (BROKEN):
use windows::Win32::System::Threading::HANDLE;

// AFTER (FIXED):
use windows::Win32::Foundation::HANDLE;
```

**Changes:**
- Line 85: `task_handle: windows::Win32::Foundation::HANDLE,`
- Line 160: `pub fn handle(&self) -> windows::Win32::Foundation::HANDLE {`

**Verification:**
```bash
cargo check --features audio-optimizations
```

---

### ✅ Fix 2: AudioBackend Trait Dyn-Safety

**File:** `src/audio/backend.rs`
**Lines:** After line 7, lines 182-201

**Step 1:** Add type aliases after imports (after line 7):
```rust
/// Dyn-safe callback type aliases (no generics!)
pub type OutputCallback = Box<dyn FnMut(&mut [f32]) + Send + 'static>;
pub type InputCallback = Box<dyn FnMut(&[f32]) + Send + 'static>;
```

**Step 2:** Replace generic methods (lines 182-201):
```rust
// BEFORE (BROKEN):
fn create_output_stream_with_callback<F>(
    &mut self,
    device_id: &str,
    config: AudioConfig,
    callback: F,
) -> Result<Box<dyn AudioStream>>
where
    F: FnMut(&mut [f32]) + Send + 'static;

// AFTER (FIXED):
fn create_output_stream_with_callback(
    &mut self,
    device_id: &str,
    config: AudioConfig,
    callback: OutputCallback,
) -> Result<Box<dyn AudioStream>>;

fn create_input_stream_with_callback(
    &mut self,
    device_id: &str,
    config: AudioConfig,
    callback: InputCallback,
) -> Result<Box<dyn AudioStream>>;
```

**Step 3:** Add downcasting methods (after line 201):
```rust
/// Downcasting support for backend-specific features
fn as_any(&self) -> &dyn std::any::Any;
fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
```

**Verification:**
```bash
cargo check
# Should now compile without "trait AudioBackend is not dyn compatible" errors
```

---

### ✅ Fix 3: CpalBackend Implementation

**File:** `src/audio/device.rs`

**Step 1:** Add import (line 6):
```rust
use super::backend::{
    AudioBackend, AudioBackendError, AudioConfig, AudioStream, DeviceInfo, Result, SampleFormat,
    InputCallback, OutputCallback,  // ← Add these
    StreamDirection, StreamStatus,
};
```

**Step 2:** Add helper method (before `impl AudioBackend for CpalBackend`):
```rust
impl CpalBackend {
    fn find_output_device(&self, device_id: &str) -> Result<cpal::Device> {
        self.host
            .output_devices()
            .map_err(|e| AudioBackendError::DeviceUnavailable(format!("Cannot enumerate: {}", e)))?
            .find(|d| d.name().ok().as_deref() == Some(device_id))
            .ok_or_else(|| AudioBackendError::DeviceNotFound(device_id.to_string()))
    }

    fn find_input_device(&self, device_id: &str) -> Result<cpal::Device> {
        self.host
            .input_devices()
            .map_err(|e| AudioBackendError::DeviceUnavailable(format!("Cannot enumerate: {}", e)))?
            .find(|d| d.name().ok().as_deref() == Some(device_id))
            .ok_or_else(|| AudioBackendError::DeviceNotFound(device_id.to_string()))
    }
}
```

**Step 3:** Update trait implementation (around line 327-452):
```rust
impl AudioBackend for CpalBackend {
    // ... existing methods ...

    fn create_output_stream_with_callback(
        &mut self,
        device_id: &str,
        config: AudioConfig,
        callback: OutputCallback,  // ← Changed from generic F
    ) -> Result<Box<dyn AudioStream>> {
        let device = self.find_output_device(device_id)?;

        let stream_config = cpal::StreamConfig {
            channels: config.channels,
            sample_rate: cpal::SampleRate(config.sample_rate),
            buffer_size: cpal::BufferSize::Fixed(config.buffer_size as u32),
        };

        let callback = Arc::new(Mutex::new(callback));
        let callback_clone = callback.clone();

        let stream = device
            .build_output_stream(
                &stream_config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let mut cb = callback_clone.lock();
                    cb(data);
                },
                |err| eprintln!("Stream error: {}", err),
                None,
            )
            .map_err(|e| AudioBackendError::StreamError(format!("Build failed: {}", e)))?;

        Ok(Box::new(CpalOutputStream {
            stream,
            config,
            status: StreamStatus::Stopped,
        }))
    }

    fn create_input_stream_with_callback(
        &mut self,
        device_id: &str,
        config: AudioConfig,
        callback: InputCallback,  // ← Changed from generic F
    ) -> Result<Box<dyn AudioStream>> {
        let device = self.find_input_device(device_id)?;

        let stream_config = cpal::StreamConfig {
            channels: config.channels,
            sample_rate: cpal::SampleRate(config.sample_rate),
            buffer_size: cpal::BufferSize::Fixed(config.buffer_size as u32),
        };

        let callback = Arc::new(Mutex::new(callback));
        let callback_clone = callback.clone();

        let stream = device
            .build_input_stream(
                &stream_config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    let mut cb = callback_clone.lock();
                    cb(data);
                },
                |err| eprintln!("Input stream error: {}", err),
                None,
            )
            .map_err(|e| AudioBackendError::StreamError(format!("Build failed: {}", e)))?;

        Ok(Box::new(CpalInputStream {
            stream,
            config,
            status: StreamStatus::Stopped,
        }))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
```

**Note:** Remove the old `create_output_stream_with_callback` and `create_input_stream_with_callback`
methods that are currently defined as inherent methods (lines 30-150). They should only exist in the
trait implementation.

---

### ✅ Fix 4: AsioBackend Implementation

**File:** `src/audio/asio_backend.rs`

**Step 1:** Add import (line 26):
```rust
use super::backend::{
    AudioBackend, AudioBackendError, AudioConfig, AudioStream, DeviceInfo, Result, SampleFormat,
    InputCallback, OutputCallback,  // ← Add these
    StreamDirection, StreamStatus,
};
```

**Step 2:** Update trait implementation (lines 744-768):
```rust
impl AudioBackend for AsioBackend {
    // ... existing methods ...

    fn create_output_stream_with_callback(
        &mut self,
        device_id: &str,
        config: AudioConfig,
        callback: OutputCallback,  // ← Changed from generic F
    ) -> Result<Box<dyn AudioStream>> {
        #[cfg(target_os = "windows")]
        {
            let host = self.get_or_create_host()?;

            let device = host
                .output_devices()
                .map_err(|e| AudioBackendError::DeviceUnavailable(format!("Cannot enumerate: {}", e)))?
                .find(|d| d.name().ok().as_deref() == Some(device_id))
                .ok_or_else(|| AudioBackendError::DeviceNotFound(device_id.to_string()))?;

            let stream_config = cpal::StreamConfig {
                channels: config.channels,
                sample_rate: cpal::SampleRate(config.sample_rate),
                buffer_size: cpal::BufferSize::Fixed(config.buffer_size as u32),
            };

            let callback = Arc::new(Mutex::new(callback));
            let callback_clone = callback.clone();

            let stream = device
                .build_output_stream(
                    &stream_config,
                    move |data: &mut [f32], _| {
                        let mut cb = callback_clone.lock();
                        cb(data);
                    },
                    |err| log::error!("ASIO stream error: {}", err),
                    None,
                )
                .map_err(|e| AudioBackendError::StreamError(e.to_string()))?;

            Ok(Box::new(AsioOutputStream {
                stream,
                config,
                status: StreamStatus::Stopped,
            }))
        }

        #[cfg(not(target_os = "windows"))]
        Err(AudioBackendError::BackendNotAvailable(
            "ASIO only available on Windows".to_string(),
        ))
    }

    fn create_input_stream_with_callback(
        &mut self,
        device_id: &str,
        config: AudioConfig,
        callback: InputCallback,  // ← Changed from generic F
    ) -> Result<Box<dyn AudioStream>> {
        // Similar implementation for input
        #[cfg(target_os = "windows")]
        {
            let host = self.get_or_create_host()?;
            let device = host
                .input_devices()
                .map_err(|e| AudioBackendError::DeviceUnavailable(format!("Cannot enumerate: {}", e)))?
                .find(|d| d.name().ok().as_deref() == Some(device_id))
                .ok_or_else(|| AudioBackendError::DeviceNotFound(device_id.to_string()))?;

            let stream_config = cpal::StreamConfig {
                channels: config.channels,
                sample_rate: cpal::SampleRate(config.sample_rate),
                buffer_size: cpal::BufferSize::Fixed(config.buffer_size as u32),
            };

            let callback = Arc::new(Mutex::new(callback));
            let callback_clone = callback.clone();

            let stream = device
                .build_input_stream(
                    &stream_config,
                    move |data: &[f32], _| {
                        let mut cb = callback_clone.lock();
                        cb(data);
                    },
                    |err| log::error!("ASIO input stream error: {}", err),
                    None,
                )
                .map_err(|e| AudioBackendError::StreamError(e.to_string()))?;

            Ok(Box::new(AsioInputStream {
                stream,
                config,
                status: StreamStatus::Stopped,
            }))
        }

        #[cfg(not(target_os = "windows"))]
        Err(AudioBackendError::BackendNotAvailable(
            "ASIO only available on Windows".to_string(),
        ))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
```

**Note:** Remove or update the old inherent method implementations (lines 215-358) to avoid duplication.

---

### ✅ Fix 5: WebAudioBackend Implementation

**File:** `src/audio/web_audio_backend.rs`

**Step 1:** Add import (line 7):
```rust
use super::backend::{
    AudioBackend, AudioBackendError, AudioConfig, AudioStream, DeviceInfo, Result, SampleFormat,
    InputCallback, OutputCallback,  // ← Add these
    StreamDirection, StreamStatus,
};
```

**Step 2:** Add to trait implementation (after line 179):
```rust
#[cfg(target_arch = "wasm32")]
impl AudioBackend for WebAudioBackend {
    // ... existing methods ...

    fn create_output_stream_with_callback(
        &mut self,
        _device_id: &str,
        config: AudioConfig,
        callback: OutputCallback,
    ) -> Result<Box<dyn AudioStream>> {
        // For now, return error - full implementation requires ScriptProcessorNode
        Err(AudioBackendError::UnsupportedFormat(
            "Callback-based Web Audio streams not yet implemented".to_string(),
        ))
    }

    fn create_input_stream_with_callback(
        &mut self,
        _device_id: &str,
        _config: AudioConfig,
        _callback: InputCallback,
    ) -> Result<Box<dyn AudioStream>> {
        Err(AudioBackendError::UnsupportedFormat(
            "Input streams require getUserMedia API".to_string(),
        ))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
```

---

### ✅ Fix 6: HybridAudioBackend Implementation

**File:** `src/audio/hybrid.rs`

**Step 1:** Add import (line 16):
```rust
use super::backend::{
    AudioBackend, AudioBackendError, AudioConfig, AudioStream, Result, StreamStatus,
    InputCallback, OutputCallback,  // ← Add these
};
```

**Step 2:** Add to trait implementation (after line 543):
```rust
#[cfg(not(target_arch = "wasm32"))]
impl AudioBackend for HybridAudioBackend {
    // ... existing methods ...

    fn create_output_stream_with_callback(
        &mut self,
        device_id: &str,
        config: AudioConfig,
        callback: OutputCallback,
    ) -> Result<Box<dyn AudioStream>> {
        match self.mode {
            HybridMode::WebAudioOnly => {
                Err(AudioBackendError::UnsupportedFormat(
                    "Callback streams not supported in WebAudioOnly mode".to_string(),
                ))
            }
            HybridMode::HybridNative | HybridMode::CpalOnly => {
                if let Some(backend) = &mut self.cpal_backend {
                    backend.create_output_stream_with_callback(device_id, config, callback)
                } else {
                    Err(AudioBackendError::BackendNotAvailable(
                        "CPAL backend not initialized".to_string(),
                    ))
                }
            }
        }
    }

    fn create_input_stream_with_callback(
        &mut self,
        device_id: &str,
        config: AudioConfig,
        callback: InputCallback,
    ) -> Result<Box<dyn AudioStream>> {
        match self.mode {
            HybridMode::WebAudioOnly => {
                Err(AudioBackendError::UnsupportedFormat(
                    "Input callback streams not supported in WebAudioOnly mode".to_string(),
                ))
            }
            HybridMode::HybridNative | HybridMode::CpalOnly => {
                if let Some(backend) = &mut self.cpal_backend {
                    backend.create_input_stream_with_callback(device_id, config, callback)
                } else {
                    Err(AudioBackendError::BackendNotAvailable(
                        "CPAL backend not initialized".to_string(),
                    ))
                }
            }
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
```

---

## Verification Steps

### Step 1: Clean Build
```bash
cargo clean
cargo check
```

**Expected:** No compilation errors

### Step 2: Check Dyn-Safety
```bash
# This should compile without "trait not dyn compatible" errors
cargo check 2>&1 | grep -i "dyn compatible"
```

**Expected:** No output (no dyn-compatibility errors)

### Step 3: Feature Compilation
```bash
cargo check --all-features
cargo check --target wasm32-unknown-unknown
```

**Expected:** Both succeed

### Step 4: Test Trait Object Usage
Add this test to `src/audio/backend.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_dyn_safety() {
        use crate::audio::device::CpalBackend;

        // This MUST compile to verify dyn-safety
        let backend: Box<dyn AudioBackend> = Box::new(CpalBackend::new());

        assert_eq!(backend.name(), "cpal");
        assert!(backend.is_available());
    }
}
```

Run:
```bash
cargo test test_backend_dyn_safety
```

**Expected:** Test compiles and passes

---

## PR #5 Bug Fix (Non-Functional EQ)

**Status:** Requires separate investigation
**File:** Likely in Web Audio API integration code

**Issue:** EQ and Analyser nodes created but not connected to audio graph

**Fix Required:**
1. Find where Web Audio nodes are created (search for `BiquadFilterNode`)
2. Ensure connection chain: `Source → EQ[0] → ... → EQ[7] → Analyser → Gain → Output`
3. Add test to verify EQ actually affects output

**Verification:**
```rust
// Test that EQ affects output
#[test]
fn test_eq_in_signal_path() {
    // 1. Generate 1kHz sine wave
    // 2. Set 1kHz EQ band to -20dB
    // 3. Process audio
    // 4. Measure output level
    // 5. Assert output is ~10x quieter (-20dB)
}
```

---

## Post-Fix Testing

### Unit Tests
```bash
cargo test audio::backend
cargo test audio::device
cargo test audio::asio_backend
cargo test audio::hybrid
```

### Integration Test
```bash
cargo test integrated_audio_manager
```

### Manual Test
```bash
cargo run --release
# Load audio file
# Adjust EQ sliders
# Verify audio changes
# Check spectrum analyser updates
```

---

## Common Issues & Solutions

### Issue: "trait `AudioBackend` is not dyn compatible"
**Solution:** Ensure no generic type parameters in trait methods

### Issue: "HANDLE not found in Threading"
**Solution:** Change import to `windows::Win32::Foundation::HANDLE`

### Issue: "method not found in trait"
**Solution:** Ensure all backends implement `as_any()` and `as_any_mut()`

### Issue: Callback not executing
**Solution:** Verify callback is wrapped in `Arc<Mutex<>>` for native backends

### Issue: EQ has no effect
**Solution:** Check audio node connections in Web Audio implementation

---

## Success Criteria

✅ `cargo check` compiles without errors
✅ `cargo test` passes all tests
✅ `Box<dyn AudioBackend>` usage compiles
✅ Audio plays through all backends (CPAL, ASIO, Web Audio)
✅ EQ adjustments affect audio output
✅ Spectrum analyser displays post-EQ signal

---

## Timeline Estimate

- **Critical Fixes (1-2 hours):** Fixes 1-6
- **Testing (30 min):** Verify compilation and basic functionality
- **PR #5 Investigation (1-2 hours):** Find and fix EQ connection
- **Full Testing (1 hour):** Manual testing on all backends

**Total:** 3.5-5.5 hours

