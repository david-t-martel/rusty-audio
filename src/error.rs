//! Error types for the rusty-audio application
//!
//! This module defines all the error types that can occur during audio playback,
//! file operations, and UI interactions. Each error type is designed to provide
//! clear context about what went wrong and how to potentially fix it.

use thiserror::Error;

/// Main error type for all application errors
#[derive(Error, Debug)]
pub enum AudioPlayerError {
    /// Errors related to file operations (opening, reading, parsing)
    #[error("File operation failed: {0}")]
    FileOperation(#[from] FileError),

    /// Errors related to audio processing and playback
    #[error("Audio processing failed: {0}")]
    AudioProcessing(#[from] AudioError),

    /// Errors related to metadata extraction
    #[error("Metadata processing failed: {0}")]
    Metadata(#[from] MetadataError),

    /// Errors related to image processing (album art)
    #[error("Image processing failed: {0}")]
    ImageProcessing(#[from] ImageError),

    /// Errors related to UI operations
    #[error("UI operation failed: {0}")]
    UserInterface(#[from] UiError),
}

/// File operation specific errors
#[derive(Error, Debug)]
pub enum FileError {
    #[error("Failed to open file: {path}")]
    OpenFailed { path: String },

    #[error("Failed to read file: {path}")]
    ReadFailed { path: String },

    #[error("File not found: {path}")]
    NotFound { path: String },

    #[error("Invalid file format: expected audio file, got {format}")]
    InvalidFormat { format: String },

    #[error("File access denied: {path}")]
    AccessDenied { path: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Audio processing and playback errors
#[derive(Error, Debug)]
pub enum AudioError {
    #[error("Failed to decode audio data")]
    DecodeFailed,

    #[error("Audio context initialization failed")]
    ContextInitFailed,

    #[error("Audio node connection failed")]
    ConnectionFailed,

    #[error("Playback failed: {reason}")]
    PlaybackFailed { reason: String },

    #[error("Audio buffer creation failed")]
    BufferCreationFailed,

    #[error("Invalid audio parameters: {details}")]
    InvalidParameters { details: String },

    #[error("Audio device not available")]
    DeviceNotAvailable,
}

/// Metadata extraction errors
#[derive(Error, Debug)]
pub enum MetadataError {
    #[error("Failed to read metadata from file")]
    ReadFailed,

    #[error("No metadata found in file")]
    NotFound,

    #[error("Invalid metadata format")]
    InvalidFormat,

    #[error("Lofty library error: {0}")]
    LoftyError(String),
}

/// Image processing errors (album art)
#[derive(Error, Debug)]
pub enum ImageError {
    #[error("Failed to load image data")]
    LoadFailed,

    #[error("Invalid image format")]
    InvalidFormat,

    #[error("Image processing error: {details}")]
    ProcessingFailed { details: String },

    #[error("Image library error: {0}")]
    ImageLibError(String),
}

/// UI operation errors
#[derive(Error, Debug)]
pub enum UiError {
    #[error("Texture loading failed")]
    TextureLoadFailed,

    #[error("Theme application failed")]
    ThemeApplicationFailed,

    #[error("UI rendering failed: {details}")]
    RenderingFailed { details: String },
}

/// Result type alias for convenience
pub type Result<T> = std::result::Result<T, AudioPlayerError>;

/// Helper trait for converting errors with additional context
pub trait ErrorContext<T> {
    fn with_context(self, context: &str) -> Result<T>;
}

impl<T, E> ErrorContext<T> for std::result::Result<T, E>
where
    E: Into<AudioPlayerError>,
{
    fn with_context(self, context: &str) -> Result<T> {
        self.map_err(|e| {
            tracing::error!("Error in {}: {:?}", context, e);
            e.into()
        })
    }
}