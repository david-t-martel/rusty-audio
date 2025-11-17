//! Audio routing system
//!
//! This module provides a flexible audio routing matrix that allows connecting
//! any audio source to any audio destination with independent gain control.
//!
//! # Architecture
//!
//! ```text
//!                  ┌──────────────────┐
//!                  │   AudioRouter    │
//!                  └──────────────────┘
//!                          │
//!        ┌─────────────────┼─────────────────┐
//!        │                 │                 │
//!    ┌───▼───┐         ┌───▼───┐        ┌───▼───┐
//!    │Source │         │Source │        │Source │
//!    │  1    │         │  2    │        │  3    │
//!    └───┬───┘         └───┬───┘        └───┬───┘
//!        │                 │                 │
//!        └─────────────────┼─────────────────┘
//!                          │
//!                    ┌─────▼─────┐
//!                    │   Mixer   │
//!                    └─────┬─────┘
//!                          │
//!        ┌─────────────────┼─────────────────┐
//!        │                 │                 │
//!    ┌───▼───┐         ┌───▼───┐        ┌───▼───┐
//!    │ Dest  │         │ Dest  │        │ Dest  │
//!    │  1    │         │  2    │        │  3    │
//!    └───────┘         └───────┘        └───────┘
//! ```
//!
//! # Usage
//!
//! ```rust,no_run
//! use rusty_audio::audio::router::{AudioRouter, AudioSource, AudioDestination};
//!
//! let mut router = AudioRouter::new();
//!
//! // Add sources and destinations
//! let input_id = router.add_source(Box::new(input_device));
//! let output_id = router.add_destination(Box::new(output_device));
//!
//! // Create route with gain
//! let route_id = router.create_route(input_id, output_id, 0.8);
//!
//! // Process audio
//! router.process();
//! ```

use super::backend::{AudioBackendError, AudioConfig, Result};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// Unique identifier for audio sources
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourceId(u64);

/// Unique identifier for audio destinations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DestId(u64);

/// Unique identifier for routes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RouteId(pub u64);  // Make field public for construction

/// Audio source trait
///
/// Implementors provide audio samples that can be routed to destinations.
pub trait AudioSource: Send {
    /// Read samples from this source into the provided buffer
    ///
    /// # Arguments
    /// * `buffer` - Buffer to fill with samples
    ///
    /// # Returns
    /// Number of samples actually read (may be less than buffer size if source is depleted)
    fn read_samples(&mut self, buffer: &mut [f32]) -> usize;

    /// Get the sample rate of this source
    fn sample_rate(&self) -> u32;

    /// Get the number of channels
    fn channels(&self) -> u16;

    /// Check if this source has more samples available
    fn has_more_samples(&self) -> bool {
        true // Most sources are continuous
    }

    /// Seek to a specific sample position (optional, for file sources)
    fn seek(&mut self, _sample: u64) -> Result<()> {
        Err(AudioBackendError::UnsupportedFormat(
            "Seeking not supported".to_string(),
        ))
    }

    /// Get current position in samples (optional, for file sources)
    fn position(&self) -> Option<u64> {
        None
    }

    /// Get total length in samples (optional, for file sources)
    fn length(&self) -> Option<u64> {
        None
    }
}

/// Audio destination trait
///
/// Implementors receive audio samples from the routing system.
pub trait AudioDestination: Send {
    /// Write samples to this destination
    ///
    /// # Arguments
    /// * `buffer` - Buffer containing samples to write
    ///
    /// # Returns
    /// Result indicating success or failure
    fn write_samples(&mut self, buffer: &[f32]) -> Result<()>;

    /// Get the sample rate of this destination
    fn sample_rate(&self) -> u32;

    /// Get the number of channels
    fn channels(&self) -> u16;

    /// Flush any buffered samples (optional)
    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

/// A route connecting a source to a destination
#[derive(Debug, Clone)]
pub struct Route {
    pub id: RouteId,
    pub source: SourceId,
    pub destination: DestId,
    pub gain: f32,
    pub enabled: bool,
    pub muted: bool,
}

impl Route {
    /// Get effective gain (0.0 if muted, otherwise gain)
    pub fn effective_gain(&self) -> f32 {
        if self.muted || !self.enabled {
            0.0
        } else {
            self.gain
        }
    }
}

/// Audio router state
struct RouterState {
    sources: HashMap<SourceId, Box<dyn AudioSource>>,
    destinations: HashMap<DestId, Box<dyn AudioDestination>>,
    routes: HashMap<RouteId, Route>,
    next_source_id: u64,
    next_dest_id: u64,
    next_route_id: u64,
}

impl RouterState {
    fn new() -> Self {
        Self {
            sources: HashMap::new(),
            destinations: HashMap::new(),
            routes: HashMap::new(),
            next_source_id: 1,
            next_dest_id: 1,
            next_route_id: 1,
        }
    }
}

/// Central audio routing system
///
/// Manages audio sources, destinations, and routes between them.
/// Supports multiple simultaneous routes with independent gain control.
pub struct AudioRouter {
    state: Arc<RwLock<RouterState>>,
    buffer_size: usize,
}

impl AudioRouter {
    /// Create a new audio router
    ///
    /// # Arguments
    /// * `buffer_size` - Size of internal processing buffers
    pub fn new(buffer_size: usize) -> Self {
        Self {
            state: Arc::new(RwLock::new(RouterState::new())),
            buffer_size,
        }
    }

    /// Add an audio source to the router
    ///
    /// # Arguments
    /// * `source` - The audio source to add
    ///
    /// # Returns
    /// Unique identifier for the added source
    pub fn add_source(&self, source: Box<dyn AudioSource>) -> SourceId {
        let mut state = self.state.write();
        let id = SourceId(state.next_source_id);
        state.next_source_id += 1;
        state.sources.insert(id, source);
        id
    }

    /// Add an audio destination to the router
    ///
    /// # Arguments
    /// * `destination` - The audio destination to add
    ///
    /// # Returns
    /// Unique identifier for the added destination
    pub fn add_destination(&self, destination: Box<dyn AudioDestination>) -> DestId {
        let mut state = self.state.write();
        let id = DestId(state.next_dest_id);
        state.next_dest_id += 1;
        state.destinations.insert(id, destination);
        id
    }

    /// Remove an audio source
    ///
    /// # Arguments
    /// * `id` - ID of the source to remove
    ///
    /// # Returns
    /// true if source was removed, false if not found
    pub fn remove_source(&self, id: SourceId) -> bool {
        let mut state = self.state.write();

        // Remove all routes using this source
        let routes_to_remove: Vec<RouteId> = state
            .routes
            .iter()
            .filter(|(_, route)| route.source == id)
            .map(|(route_id, _)| *route_id)
            .collect();

        for route_id in routes_to_remove {
            state.routes.remove(&route_id);
        }

        state.sources.remove(&id).is_some()
    }

    /// Remove an audio destination
    ///
    /// # Arguments
    /// * `id` - ID of the destination to remove
    ///
    /// # Returns
    /// true if destination was removed, false if not found
    pub fn remove_destination(&self, id: DestId) -> bool {
        let mut state = self.state.write();

        // Remove all routes using this destination
        let routes_to_remove: Vec<RouteId> = state
            .routes
            .iter()
            .filter(|(_, route)| route.destination == id)
            .map(|(route_id, _)| *route_id)
            .collect();

        for route_id in routes_to_remove {
            state.routes.remove(&route_id);
        }

        state.destinations.remove(&id).is_some()
    }

    /// Create a route from source to destination
    ///
    /// # Arguments
    /// * `source` - Source ID
    /// * `destination` - Destination ID
    /// * `gain` - Gain factor (0.0 to 1.0, can exceed 1.0 for amplification)
    ///
    /// # Returns
    /// Route ID if successful, error if source or destination not found
    pub fn create_route(
        &self,
        source: SourceId,
        destination: DestId,
        gain: f32,
    ) -> Result<RouteId> {
        let mut state = self.state.write();

        // Verify source and destination exist
        if !state.sources.contains_key(&source) {
            return Err(AudioBackendError::DeviceNotFound(format!(
                "Source {:?} not found",
                source
            )));
        }

        if !state.destinations.contains_key(&destination) {
            return Err(AudioBackendError::DeviceNotFound(format!(
                "Destination {:?} not found",
                destination
            )));
        }

        let route_id = RouteId(state.next_route_id);
        state.next_route_id += 1;

        let route = Route {
            id: route_id,
            source,
            destination,
            gain: gain.max(0.0), // Clamp to non-negative
            enabled: true,
            muted: false,
        };

        state.routes.insert(route_id, route);
        Ok(route_id)
    }

    /// Remove a route
    ///
    /// # Arguments
    /// * `id` - Route ID to remove
    ///
    /// # Returns
    /// true if route was removed, false if not found
    pub fn remove_route(&self, id: RouteId) -> bool {
        let mut state = self.state.write();
        state.routes.remove(&id).is_some()
    }

    /// Set route gain
    ///
    /// # Arguments
    /// * `id` - Route ID
    /// * `gain` - New gain value (0.0 to 1.0+)
    pub fn set_route_gain(&self, id: RouteId, gain: f32) -> Result<()> {
        let mut state = self.state.write();
        if let Some(route) = state.routes.get_mut(&id) {
            route.gain = gain.max(0.0);
            Ok(())
        } else {
            Err(AudioBackendError::DeviceNotFound(format!(
                "Route {:?} not found",
                id
            )))
        }
    }

    /// Set route enabled state
    pub fn set_route_enabled(&self, id: RouteId, enabled: bool) -> Result<()> {
        let mut state = self.state.write();
        if let Some(route) = state.routes.get_mut(&id) {
            route.enabled = enabled;
            Ok(())
        } else {
            Err(AudioBackendError::DeviceNotFound(format!(
                "Route {:?} not found",
                id
            )))
        }
    }

    /// Set route muted state
    pub fn set_route_muted(&self, id: RouteId, muted: bool) -> Result<()> {
        let mut state = self.state.write();
        if let Some(route) = state.routes.get_mut(&id) {
            route.muted = muted;
            Ok(())
        } else {
            Err(AudioBackendError::DeviceNotFound(format!(
                "Route {:?} not found",
                id
            )))
        }
    }

    /// Get route information
    pub fn get_route(&self, id: RouteId) -> Option<Route> {
        let state = self.state.read();
        state.routes.get(&id).cloned()
    }

    /// Get all routes
    pub fn get_routes(&self) -> Vec<Route> {
        let state = self.state.read();
        state.routes.values().cloned().collect()
    }

    /// Get all routes for a specific source
    pub fn get_routes_for_source(&self, source: SourceId) -> Vec<Route> {
        let state = self.state.read();
        state
            .routes
            .values()
            .filter(|route| route.source == source)
            .cloned()
            .collect()
    }

    /// Get all routes for a specific destination
    pub fn get_routes_for_destination(&self, destination: DestId) -> Vec<Route> {
        let state = self.state.read();
        state
            .routes
            .values()
            .filter(|route| route.destination == destination)
            .cloned()
            .collect()
    }

    /// Get source IDs
    pub fn get_source_ids(&self) -> Vec<SourceId> {
        let state = self.state.read();
        state.sources.keys().copied().collect()
    }

    /// Get destination IDs
    pub fn get_destination_ids(&self) -> Vec<DestId> {
        let state = self.state.read();
        state.destinations.keys().copied().collect()
    }

    /// Process audio routing for one buffer
    ///
    /// This reads from all sources, applies routing and gain, and writes to destinations.
    /// Should be called regularly (typically in an audio callback).
    pub fn process(&self) -> Result<()> {
        let mut state = self.state.write();

        // Temporary buffers for mixing
        let mut source_buffer = vec![0.0f32; self.buffer_size];
        let mut dest_buffers: HashMap<DestId, Vec<f32>> = HashMap::new();

        // Initialize destination buffers
        for dest_id in state.destinations.keys() {
            dest_buffers.insert(*dest_id, vec![0.0f32; self.buffer_size]);
        }

        // Cache for source samples (read each source once for proper fan-out)
        // Key: source_id, Value: (samples_read, Vec<f32>)
        let mut source_cache: HashMap<SourceId, (usize, Vec<f32>)> = HashMap::new();

        // Collect unique source IDs from enabled routes first (to avoid borrow conflicts)
        let unique_source_ids: Vec<SourceId> = state.routes.values()
            .filter(|route| route.enabled)
            .map(|route| route.source)
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        // First pass: Read from each unique source once
        for source_id in unique_source_ids {
            if let Some(source) = state.sources.get_mut(&source_id) {
                let samples_read = source.read_samples(&mut source_buffer);
                // Store a copy of the samples
                source_cache.insert(source_id, (samples_read, source_buffer.clone()));
            }
        }

        // Second pass: Route cached samples to destinations
        for route in state.routes.values() {
            if !route.enabled {
                continue;
            }

            let gain = route.effective_gain();
            if gain == 0.0 {
                continue;
            }

            // Use cached samples from source
            if let Some((samples_read, source_samples)) = source_cache.get(&route.source) {
                if *samples_read > 0 {
                    // Mix into destination buffer with gain
                    if let Some(dest_buffer) = dest_buffers.get_mut(&route.destination) {
                        for i in 0..*samples_read.min(&self.buffer_size) {
                            dest_buffer[i] += source_samples[i] * gain;
                        }
                    }
                }
            }
        }

        // Write to destinations
        for (dest_id, buffer) in dest_buffers.iter() {
            if let Some(destination) = state.destinations.get_mut(dest_id) {
                // Apply soft clipping to prevent severe distortion
                let clipped_buffer: Vec<f32> =
                    buffer.iter().map(|&sample| soft_clip(sample)).collect();

                destination.write_samples(&clipped_buffer)?;
            }
        }

        Ok(())
    }

    /// Clear all routes (keep sources and destinations)
    pub fn clear_routes(&self) {
        let mut state = self.state.write();
        state.routes.clear();
    }

    /// Clear everything (sources, destinations, routes)
    pub fn clear_all(&self) {
        let mut state = self.state.write();
        state.sources.clear();
        state.destinations.clear();
        state.routes.clear();
    }
}

/// Soft clipping function to prevent harsh distortion
///
/// Uses a tanh-based soft clipping curve that smoothly compresses
/// signals exceeding [-1.0, 1.0] range.
fn soft_clip(sample: f32) -> f32 {
    if sample.abs() <= 1.0 {
        sample
    } else {
        // Tanh soft clipping for smoother distortion
        sample.signum() * (1.0 - (1.0 / (1.0 + sample.abs())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock source for testing
    struct MockSource {
        data: Vec<f32>,
        position: usize,
    }

    impl MockSource {
        fn new(data: Vec<f32>) -> Self {
            Self { data, position: 0 }
        }
    }

    impl AudioSource for MockSource {
        fn read_samples(&mut self, buffer: &mut [f32]) -> usize {
            let available = self.data.len() - self.position;
            let to_read = available.min(buffer.len());

            buffer[..to_read].copy_from_slice(&self.data[self.position..self.position + to_read]);
            self.position += to_read;
            to_read
        }

        fn sample_rate(&self) -> u32 {
            44100
        }

        fn channels(&self) -> u16 {
            1
        }

        fn has_more_samples(&self) -> bool {
            self.position < self.data.len()
        }
    }

    // Mock destination for testing
    struct MockDestination {
        received: Vec<f32>,
    }

    impl MockDestination {
        fn new() -> Self {
            Self {
                received: Vec::new(),
            }
        }
    }

    impl AudioDestination for MockDestination {
        fn write_samples(&mut self, buffer: &[f32]) -> Result<()> {
            self.received.extend_from_slice(buffer);
            Ok(())
        }

        fn sample_rate(&self) -> u32 {
            44100
        }

        fn channels(&self) -> u16 {
            1
        }
    }

    #[test]
    fn test_router_creation() {
        let router = AudioRouter::new(512);
        assert_eq!(router.get_source_ids().len(), 0);
        assert_eq!(router.get_destination_ids().len(), 0);
    }

    #[test]
    fn test_add_source_and_destination() {
        let router = AudioRouter::new(512);

        let source = MockSource::new(vec![1.0, 2.0, 3.0]);
        let dest = MockDestination::new();

        let source_id = router.add_source(Box::new(source));
        let dest_id = router.add_destination(Box::new(dest));

        assert_eq!(router.get_source_ids().len(), 1);
        assert_eq!(router.get_destination_ids().len(), 1);
        assert!(router.get_source_ids().contains(&source_id));
        assert!(router.get_destination_ids().contains(&dest_id));
    }

    #[test]
    fn test_create_route() {
        let router = AudioRouter::new(512);

        let source = MockSource::new(vec![1.0, 2.0, 3.0]);
        let dest = MockDestination::new();

        let source_id = router.add_source(Box::new(source));
        let dest_id = router.add_destination(Box::new(dest));

        let route_id = router.create_route(source_id, dest_id, 1.0).unwrap();

        let route = router.get_route(route_id).unwrap();
        assert_eq!(route.source, source_id);
        assert_eq!(route.destination, dest_id);
        assert_eq!(route.gain, 1.0);
        assert!(route.enabled);
    }

    #[test]
    fn test_route_gain_control() {
        let router = AudioRouter::new(512);

        let source = MockSource::new(vec![1.0; 512]);
        let dest = MockDestination::new();

        let source_id = router.add_source(Box::new(source));
        let dest_id = router.add_destination(Box::new(dest));
        let route_id = router.create_route(source_id, dest_id, 1.0).unwrap();

        // Change gain
        router.set_route_gain(route_id, 0.5).unwrap();
        let route = router.get_route(route_id).unwrap();
        assert_eq!(route.gain, 0.5);
    }

    #[test]
    fn test_soft_clip() {
        assert_eq!(soft_clip(0.0), 0.0);
        assert_eq!(soft_clip(0.5), 0.5);
        assert_eq!(soft_clip(1.0), 1.0);
        assert_eq!(soft_clip(-1.0), -1.0);

        // Values > 1.0 should be clipped
        assert!(soft_clip(2.0) < 2.0);
        assert!(soft_clip(2.0) > 1.0);
        assert!(soft_clip(-2.0) > -2.0);
        assert!(soft_clip(-2.0) < -1.0);
    }
}
