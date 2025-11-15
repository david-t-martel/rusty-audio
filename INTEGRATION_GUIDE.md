# Audio Router UI Integration Guide

This guide explains how to integrate the new `IntegratedAudioManager` into the existing UI code in `src/main.rs`.

## Overview

The new audio routing system provides a unified interface for:
- Signal generator playback
- Input device monitoring
- File playback (future)
- Recording with monitoring

## Step 1: Add IntegratedAudioManager to AudioPlayerApp

Replace the current audio backend fields with the integrated manager:

```rust
#[cfg(not(target_arch = "wasm32"))]
struct AudioPlayerApp {
    // Replace these fields:
    // audio_backend: Option<HybridAudioBackend>,
    // device_manager: Option<AudioDeviceManager>,
    // web_audio_bridge: Option<WebAudioBridge>,

    // With this:
    integrated_audio_manager: Option<IntegratedAudioManager>,

    // Keep existing audio_engine for now (will be migrated later)
    audio_engine: Box<dyn rusty_audio::audio_engine::AudioEngineInterface>,

    // ... rest of fields unchanged
}
```

## Step 2: Initialize IntegratedAudioManager

In the `Default` implementation or initialization code:

```rust
impl Default for AudioPlayerApp {
    fn default() -> Self {
        // Create integrated audio manager
        let config = AudioConfig {
            sample_rate: 48000,
            channels: 2,
            buffer_size: 512,
            exclusive_mode: false,
            ..Default::default()
        };

        let mut integrated_audio_manager = IntegratedAudioManager::new(512, config)
            .ok(); // Option for graceful fallback

        // Initialize default output device
        if let Some(ref mut manager) = integrated_audio_manager {
            let _ = manager.initialize_output_device(None); // None = use default
        }

        Self {
            integrated_audio_manager,
            // ... rest of initialization
        }
    }
}
```

## Step 3: Update Signal Generator Routing

Replace the `handle_signal_generator_routing` method:

```rust
fn handle_signal_generator_routing(&mut self) {
    if let Some(intent) = self.signal_generator_panel.take_route_intent() {
        let Some(output) = self.signal_generator_panel.output_snapshot() else {
            self.error = Some("Generate a signal before routing it.".to_string());
            return;
        };

        match intent.mode {
            GeneratorRoutingMode::Recorder => {
                // Route to recorder (existing code)
                self.recording_panel.log_generated_take(
                    intent.label.clone(),
                    output.samples,
                    output.sample_rate,
                    output.channels,
                );
                self.audio_status_message = Some((
                    format!("Saved {} as a virtual take", intent.label),
                    Instant::now(),
                ));
            }
            GeneratorRoutingMode::Output => {
                // NEW: Route through integrated audio manager
                if let Some(ref mut manager) = self.integrated_audio_manager {
                    match manager.play_signal_generator(
                        output.samples,
                        output.sample_rate,
                        false, // not looping
                    ) {
                        Ok(_) => {
                            self.signal_generator_panel.state = GeneratorState::Playing;
                            self.audio_status_message = Some((
                                format!("Playing {}", intent.label),
                                Instant::now(),
                            ));
                        }
                        Err(e) => {
                            self.error = Some(format!("Failed to play signal: {}", e));
                        }
                    }
                } else {
                    self.error = Some("Audio manager not initialized".to_string());
                }
            }
            GeneratorRoutingMode::Both => {
                // Route to both recorder and output
                // 1. Save to recorder
                self.recording_panel.log_generated_take(
                    intent.label.clone(),
                    output.samples.clone(),
                    output.sample_rate,
                    output.channels,
                );

                // 2. Play through output
                if let Some(ref mut manager) = self.integrated_audio_manager {
                    let _ = manager.play_signal_generator(
                        output.samples,
                        output.sample_rate,
                        false,
                    );
                }
            }
        }
    }
}
```

## Step 4: Add Signal Generator Stop Button Handler

In the signal generator UI code, add a stop button:

```rust
// In the Generator tab rendering
if self.signal_generator_panel.state == GeneratorState::Playing {
    // Add stop button
    let stop_button = EnhancedButton::new("Stop")
        .with_style(ButtonStyle::Destructive);

    if ui.add(stop_button).clicked() {
        if let Some(ref mut manager) = self.integrated_audio_manager {
            let _ = manager.stop_signal_generator();
            self.signal_generator_panel.state = GeneratorState::Stopped;
        }
    }
}
```

## Step 5: Add Audio Processing Loop

The audio router needs to be processed regularly. Add this to the `update` method:

```rust
fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    // Process audio routing
    if let Some(ref manager) = self.integrated_audio_manager {
        // Process audio in the main update loop
        // In production, this should be in a dedicated audio thread
        let _ = manager.process();
    }

    // ... rest of update code

    // Request continuous repaints while audio is active
    if let Some(ref manager) = self.integrated_audio_manager {
        if manager.is_signal_generator_playing() {
            ctx.request_repaint();
        }
    }
}
```

## Step 6: Device Selection UI

Update the Settings tab to use the integrated manager for device selection:

```rust
// In Settings tab
if let Some(ref mut manager) = self.integrated_audio_manager {
    ui.heading("Output Device");

    if let Ok(devices) = manager.enumerate_output_devices() {
        for device in devices {
            ui.label(&device.name);
            if device.is_default {
                ui.label("(Default)");
            }

            // TODO: Add selection UI
        }
    }
}
```

## Step 7: Recording Monitoring (Future)

To implement recording monitoring, add these methods:

```rust
// In IntegratedAudioManager
impl IntegratedAudioManager {
    /// Start input monitoring
    pub fn start_input_monitoring(&mut self, device_id: &str, gain: f32) -> Result<()> {
        // Create input device source
        // Create route to output with gain
        // Store route in active_routes
        todo!("Implement in Phase 3.2")
    }

    /// Stop input monitoring
    pub fn stop_input_monitoring(&mut self) -> Result<()> {
        // Remove monitoring route
        todo!("Implement in Phase 3.2")
    }
}
```

## Complete Example: Signal Generator Integration

Here's a complete example of integrating the signal generator:

```rust
// In src/main.rs

// 1. Add import
use rusty_audio::integrated_audio_manager::{
    IntegratedAudioManager, PlaybackState, RouteType
};

// 2. Add to AudioPlayerApp
struct AudioPlayerApp {
    integrated_audio_manager: Option<IntegratedAudioManager>,
    // ... other fields
}

// 3. Initialize
impl Default for AudioPlayerApp {
    fn default() -> Self {
        let config = AudioConfig::default();
        let mut manager = IntegratedAudioManager::new(512, config).ok();

        if let Some(ref mut m) = manager {
            let _ = m.initialize_output_device(None);
        }

        Self {
            integrated_audio_manager: manager,
            // ... other fields
        }
    }
}

// 4. Handle routing
fn handle_signal_generator_routing(&mut self) {
    if let Some(intent) = self.signal_generator_panel.take_route_intent() {
        if let Some(output) = self.signal_generator_panel.output_snapshot() {
            if let Some(ref mut manager) = self.integrated_audio_manager {
                match intent.mode {
                    GeneratorRoutingMode::Output => {
                        manager.play_signal_generator(
                            output.samples,
                            output.sample_rate,
                            false,
                        ).ok();
                        self.signal_generator_panel.state = GeneratorState::Playing;
                    }
                    _ => { /* handle other modes */ }
                }
            }
        }
    }
}

// 5. Process audio
fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    if let Some(ref manager) = self.integrated_audio_manager {
        manager.process().ok();

        if manager.is_signal_generator_playing() {
            ctx.request_repaint();
        }
    }
}
```

## Migration Strategy

### Phase 3.1: Signal Generator (Complete)
- ✅ Create IntegratedAudioManager
- ✅ Integrate signal generator playback
- ✅ Add Play/Stop button handling

### Phase 3.2: Recording Monitoring (Next)
- Add InputDeviceSource to manager
- Implement monitoring modes (Direct, Routed)
- Wire monitoring UI controls

### Phase 3.3: File Playback (Future)
- Create FilePlaybackSource using Symphonia
- Migrate from WebAudioEngine to router
- Maintain EQ/effects through router

## Testing

To test the integration:

1. **Signal Generator Test:**
   ```
   - Open Generator tab
   - Generate a sine wave
   - Click "Route to Output"
   - Verify audio plays through speakers
   - Click "Stop"
   - Verify audio stops
   ```

2. **Device Selection Test:**
   ```
   - Open Settings tab
   - View available output devices
   - Select different device
   - Generate signal and verify it plays through selected device
   ```

3. **Error Handling Test:**
   ```
   - Try to play without generating signal
   - Verify error message appears
   - Disconnect audio device
   - Verify graceful fallback
   ```

## Troubleshooting

### No Audio Output
- Check that `initialize_output_device()` succeeded
- Verify default audio device is available
- Check audio device permissions

### Playback Doesn't Stop
- Ensure `stop_signal_generator()` is called
- Check that route is being removed from router
- Verify generator state is set to Stopped

### High Latency
- Reduce buffer size (256 or 128 samples)
- Enable exclusive mode in AudioConfig
- Use ASIO backend on Windows

## Next Steps

1. Test the signal generator integration
2. Implement recording monitoring
3. Add routing matrix UI visualization
4. Migrate file playback to router

## API Reference

See:
- `src/integrated_audio_manager.rs` - High-level audio manager
- `src/audio/router.rs` - Core routing engine
- `src/audio/sources.rs` - Audio source implementations
- `src/audio/destinations.rs` - Audio destination implementations
