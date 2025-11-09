pub mod components;
pub mod layout;
pub mod dock_layout;
pub mod spectrum;
pub mod controls;
pub mod theme;
pub mod utils;
pub mod signal_generator;
pub mod accessibility;
pub mod enhanced_controls;
pub mod enhanced_button;
pub mod error_handling;
// Recording panel requires native audio module
#[cfg(not(target_arch = "wasm32"))]
pub mod recording_panel;

pub use components::*;
pub use layout::*;
pub use spectrum::*;
pub use controls::*;
pub use theme::*;
pub use utils::*;
pub use signal_generator::*;
pub use accessibility::*;
pub use enhanced_controls::{AccessibleSlider, AccessibleKnob};
pub use enhanced_button::*;
pub use error_handling::*;
#[cfg(not(target_arch = "wasm32"))]
pub use recording_panel::RecordingPanel;
