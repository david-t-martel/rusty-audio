//! Web Audio API backend for WASM
//!
//! Implements the AudioBackend trait using the browser's Web Audio API.
//! This provides audio functionality for WASM builds.

#[cfg(target_arch = "wasm32")]
use super::backend::{
    AudioBackend, AudioBackendError, AudioConfig, AudioStream, DeviceInfo, Result, SampleFormat,
    StreamDirection, StreamStatus,
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use web_sys::{
    AnalyserNode, AudioContext, AudioDestinationNode, BiquadFilterNode, BiquadFilterType,
};

/// Web Audio API backend with EQ and spectrum analysis support
#[cfg(target_arch = "wasm32")]
pub struct WebAudioBackend {
    context: Option<AudioContext>,
    initialized: bool,
    /// 8-band parametric EQ using BiquadFilterNode (peaking filters)
    /// Frequencies: 60Hz, 120Hz, 240Hz, 480Hz, 960Hz, 1.9kHz, 3.8kHz, 7.7kHz
    eq_filters: Vec<BiquadFilterNode>,
    /// Spectrum analyser for real-time frequency analysis
    analyser: Option<AnalyserNode>,
}

// SAFETY: WebAudioBackend is safe to Send/Sync in WASM context
// WASM runs single-threaded in the browser, so no actual multi-threading occurs
#[cfg(target_arch = "wasm32")]
unsafe impl Send for WebAudioBackend {}
#[cfg(target_arch = "wasm32")]
unsafe impl Sync for WebAudioBackend {}

#[cfg(target_arch = "wasm32")]
impl WebAudioBackend {
    /// Create a new Web Audio API backend
    pub fn new() -> Self {
        Self {
            context: None,
            initialized: false,
            eq_filters: Vec::new(),
            analyser: None,
        }
    }

    /// Get or create the audio context
    fn get_context(&mut self) -> Result<&AudioContext> {
        if self.context.is_none() {
            let context = AudioContext::new().map_err(|e| {
                AudioBackendError::InitializationFailed(format!(
                    "Failed to create AudioContext: {:?}",
                    e
                ))
            })?;

            self.context = Some(context);
        }

        Ok(self.context.as_ref().unwrap())
    }

    /// Get sample rate from context
    fn get_sample_rate(&mut self) -> Result<u32> {
        let context = self.get_context()?;
        Ok(context.sample_rate() as u32)
    }

    /// Create 8-band parametric EQ filter chain
    /// Source: Based on WASM_CODE_BORROWING_GUIDE.md Phase 2 recommendations
    pub fn create_eq_chain(&mut self) -> Result<()> {
        // Get context and clone to avoid borrow checker issues
        let context = self.get_context()?.clone();

        // Clear existing filters if any
        self.eq_filters.clear();

        // Create 8 peaking filters (one per frequency band)
        for i in 0..8 {
            let filter = context.create_biquad_filter().map_err(|e| {
                AudioBackendError::InitializationFailed(format!(
                    "Failed to create biquad filter: {:?}",
                    e
                ))
            })?;

            // Set filter type to peaking (parametric EQ)
            filter.set_type(BiquadFilterType::Peaking);

            // Set center frequency: 60Hz × 2^i
            let freq = 60.0 * 2.0_f32.powi(i);
            filter.frequency().set_value(freq);

            // Set Q factor (bandwidth) - typical value for parametric EQ
            filter.q().set_value(1.0);

            // Initial gain: 0 dB (flat response)
            filter.gain().set_value(0.0);

            self.eq_filters.push(filter);
        }

        log::info!("Created 8-band parametric EQ chain");
        Ok(())
    }

    /// Set EQ band gain
    /// Source: Based on WASM_CODE_BORROWING_GUIDE.md Phase 2 recommendations
    ///
    /// # Arguments
    /// * `band` - Band index (0-7)
    /// * `gain_db` - Gain in decibels (±12 dB range)
    pub fn set_eq_band(&mut self, band: usize, gain_db: f32) -> Result<()> {
        if band >= self.eq_filters.len() {
            return Err(AudioBackendError::UnsupportedFormat(format!(
                "Invalid EQ band: {}. Must be 0-7",
                band
            )));
        }

        // Clamp gain to safe range (±12 dB)
        let clamped_gain = gain_db.clamp(-12.0, 12.0);

        // Set the gain on the BiquadFilterNode
        self.eq_filters[band].gain().set_value(clamped_gain);

        log::debug!("EQ band {} set to {:.1} dB", band, clamped_gain);
        Ok(())
    }

    /// Get EQ band gain
    ///
    /// # Arguments
    /// * `band` - Band index (0-7)
    pub fn get_eq_band(&self, band: usize) -> Result<f32> {
        if band >= self.eq_filters.len() {
            return Err(AudioBackendError::UnsupportedFormat(format!(
                "Invalid EQ band: {}. Must be 0-7",
                band
            )));
        }

        Ok(self.eq_filters[band].gain().value())
    }

    /// Reset all EQ bands to 0 dB (flat response)
    pub fn reset_eq(&mut self) -> Result<()> {
        for (i, filter) in self.eq_filters.iter().enumerate() {
            filter.gain().set_value(0.0);
            log::debug!("Reset EQ band {} to 0 dB", i);
        }

        log::info!("All EQ bands reset to flat response");
        Ok(())
    }

    /// Get reference to EQ filters for audio graph connection
    pub fn eq_filters(&self) -> &[BiquadFilterNode] {
        &self.eq_filters
    }

    /// Create spectrum analyser
    /// Source: Based on WASM_CODE_BORROWING_GUIDE.md Phase 3 recommendations
    ///
    /// # Arguments
    /// * `fft_size` - FFT size (must be power of 2, typically 512, 1024, 2048, 4096)
    /// * `smoothing_time_constant` - Smoothing (0.0 to 1.0, default 0.8)
    pub fn create_analyser(&mut self, fft_size: u32, smoothing_time_constant: f32) -> Result<()> {
        let context = self.get_context()?.clone();

        let analyser = context.create_analyser().map_err(|e| {
            AudioBackendError::InitializationFailed(format!(
                "Failed to create AnalyserNode: {:?}",
                e
            ))
        })?;

        // Set FFT size (must be power of 2)
        analyser.set_fft_size(fft_size);

        // Set smoothing time constant (0.0 = no smoothing, 1.0 = max smoothing)
        analyser.set_smoothing_time_constant(smoothing_time_constant.clamp(0.0, 1.0) as f64);

        // Store min/max decibels for proper scaling
        analyser.set_min_decibels(-100.0);
        analyser.set_max_decibels(0.0);

        self.analyser = Some(analyser);
        log::info!(
            "Created spectrum analyser: FFT size {}, smoothing {:.2}",
            fft_size,
            smoothing_time_constant
        );
        Ok(())
    }

    /// Get frequency data from analyser
    ///
    /// Returns frequency bin amplitudes in decibels
    pub fn get_frequency_data(&self) -> Result<Vec<f32>> {
        let analyser = self.analyser.as_ref().ok_or_else(|| {
            AudioBackendError::UnsupportedFormat(
                "Analyser not created. Call create_analyser() first".to_string(),
            )
        })?;

        let frequency_bin_count = analyser.frequency_bin_count();
        let mut data = vec![0u8; frequency_bin_count as usize];

        // Get frequency data as unsigned bytes (0-255)
        analyser.get_byte_frequency_data(&mut data);

        // Convert to normalized float values (0.0 to 1.0)
        let float_data: Vec<f32> = data.iter().map(|&b| b as f32 / 255.0).collect();

        Ok(float_data)
    }

    /// Get analyser reference for audio graph connection
    pub fn analyser(&self) -> Option<&AnalyserNode> {
        self.analyser.as_ref()
    }

    /// Get frequency bin count (half of FFT size)
    pub fn frequency_bin_count(&self) -> Option<u32> {
        self.analyser.as_ref().map(|a| a.frequency_bin_count())
    }
}

#[cfg(target_arch = "wasm32")]
impl Default for WebAudioBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(target_arch = "wasm32")]
impl AudioBackend for WebAudioBackend {
    fn name(&self) -> &'static str {
        "Web Audio API"
    }

    fn is_available(&self) -> bool {
        // Web Audio API is always available in browsers
        true
    }

    fn initialize(&mut self) -> Result<()> {
        if self.initialized {
            return Ok(());
        }

        // Create audio context to verify it works
        self.get_context()?;
        self.initialized = true;

        log::info!("Web Audio API backend initialized");
        Ok(())
    }

    fn enumerate_devices(&self, _direction: StreamDirection) -> Result<Vec<DeviceInfo>> {
        // Web Audio API doesn't expose device enumeration in the same way
        // Return a single "default" device
        let sample_rate = if let Some(ref ctx) = self.context {
            ctx.sample_rate() as u32
        } else {
            48000 // Default assumption
        };

        Ok(vec![DeviceInfo {
            id: "default".to_string(),
            name: "Web Audio Default".to_string(),
            is_default: true,
            supported_configs: vec![AudioConfig {
                sample_rate,
                channels: 2,
                sample_format: SampleFormat::F32,
                buffer_size: 512,
                exclusive_mode: false,
            }],
            min_sample_rate: 44100,
            max_sample_rate: 48000,
            max_input_channels: 2,
            max_output_channels: 2,
        }])
    }

    fn default_device(&self, direction: StreamDirection) -> Result<DeviceInfo> {
        let devices = self.enumerate_devices(direction)?;
        devices
            .into_iter()
            .next()
            .ok_or_else(|| AudioBackendError::DeviceNotFound("No default device".to_string()))
    }

    fn test_device(&self, _device_id: &str) -> Result<bool> {
        // In Web Audio API, if the context exists, the device works
        Ok(self.context.is_some())
    }

    fn supported_configs(
        &self,
        _device_id: &str,
        _direction: StreamDirection,
    ) -> Result<Vec<AudioConfig>> {
        let sample_rate = if let Some(ref ctx) = self.context {
            ctx.sample_rate() as u32
        } else {
            48000
        };

        Ok(vec![
            AudioConfig {
                sample_rate,
                channels: 2,
                sample_format: SampleFormat::F32,
                buffer_size: 512,
                exclusive_mode: false,
            },
            AudioConfig {
                sample_rate,
                channels: 1,
                sample_format: SampleFormat::F32,
                buffer_size: 512,
                exclusive_mode: false,
            },
        ])
    }

    fn create_output_stream(
        &mut self,
        _device_id: &str,
        config: AudioConfig,
    ) -> Result<Box<dyn AudioStream>> {
        let context = self.get_context()?.clone();

        Ok(Box::new(WebAudioOutputStream {
            context,
            config,
            status: StreamStatus::Stopped,
        }))
    }

    fn create_input_stream(
        &mut self,
        _device_id: &str,
        _config: AudioConfig,
    ) -> Result<Box<dyn AudioStream>> {
        // Input streams require getUserMedia, which is more complex
        // For now, return an error
        Err(AudioBackendError::UnsupportedFormat(
            "Input streams not yet supported in Web Audio API backend".to_string(),
        ))
    }

    fn create_output_stream_with_callback(
        &mut self,
        device_id: &str,
        config: AudioConfig,
        _callback: Box<dyn FnMut(&mut [f32]) + Send + 'static>,
    ) -> Result<Box<dyn AudioStream>> {
        // For now, just create a regular output stream
        // Callback-based streams are not yet implemented for Web Audio
        self.create_output_stream(device_id, config)
    }

    fn create_input_stream_with_callback(
        &mut self,
        _device_id: &str,
        _config: AudioConfig,
        _callback: Box<dyn FnMut(&[f32]) + Send + 'static>,
    ) -> Result<Box<dyn AudioStream>> {
        Err(AudioBackendError::UnsupportedFormat(
            "Input streams with callbacks not yet supported in Web Audio API backend".to_string(),
        ))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Web Audio output stream
#[cfg(target_arch = "wasm32")]
struct WebAudioOutputStream {
    context: AudioContext,
    config: AudioConfig,
    status: StreamStatus,
}

// SAFETY: WebAudioOutputStream is safe to Send in WASM context
// WASM runs single-threaded in the browser
#[cfg(target_arch = "wasm32")]
unsafe impl Send for WebAudioOutputStream {}

#[cfg(target_arch = "wasm32")]
impl AudioStream for WebAudioOutputStream {
    fn play(&mut self) -> Result<()> {
        // Resume the audio context if suspended
        let promise = self.context.resume().map_err(|e| {
            AudioBackendError::StreamError(format!("Failed to resume context: {:?}", e))
        })?;

        self.status = StreamStatus::Playing;
        Ok(())
    }

    fn pause(&mut self) -> Result<()> {
        let promise = self.context.suspend().map_err(|e| {
            AudioBackendError::StreamError(format!("Failed to suspend context: {:?}", e))
        })?;

        self.status = StreamStatus::Paused;
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        let promise = self.context.suspend().map_err(|e| {
            AudioBackendError::StreamError(format!("Failed to stop context: {:?}", e))
        })?;

        self.status = StreamStatus::Stopped;
        Ok(())
    }

    fn status(&self) -> StreamStatus {
        self.status
    }

    fn config(&self) -> &AudioConfig {
        &self.config
    }

    fn latency_samples(&self) -> Option<usize> {
        // Web Audio API has built-in buffering
        Some(self.config.buffer_size)
    }
}

// Stub implementation for non-WASM platforms
#[cfg(not(target_arch = "wasm32"))]
pub struct WebAudioBackend;

#[cfg(not(target_arch = "wasm32"))]
impl WebAudioBackend {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(test)]
#[cfg(target_arch = "wasm32")]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_web_audio_backend_creation() {
        let backend = WebAudioBackend::new();
        assert!(backend.is_available());
        assert_eq!(backend.name(), "Web Audio API");
    }

    #[wasm_bindgen_test]
    fn test_web_audio_backend_initialization() {
        let mut backend = WebAudioBackend::new();
        assert!(backend.initialize().is_ok());
        assert!(backend.initialized);
    }

    #[wasm_bindgen_test]
    fn test_device_enumeration() {
        let mut backend = WebAudioBackend::new();
        backend.initialize().unwrap();

        let devices = backend.enumerate_devices(StreamDirection::Output).unwrap();
        assert!(!devices.is_empty());
        assert_eq!(devices[0].id, "default");
    }
}
