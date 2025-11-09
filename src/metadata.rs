//! Metadata Module
//!
//! This module handles audio file metadata extraction and album art processing.
//! It follows the Single Responsibility Principle by handling only metadata operations.

use crate::error::{ErrorContext, ImageError, MetadataError, Result};
use image::GenericImageView;
use lofty::{
    file::{AudioFile, TaggedFileExt},
    tag::Accessor,
};
use std::path::Path;
use tracing::{debug, info, warn};

/// Track metadata extracted from audio files
#[derive(Debug, Clone)]
pub struct TrackMetadata {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub year: String,
    pub duration: Option<std::time::Duration>,
    pub genre: Option<String>,
    pub track_number: Option<u32>,
    pub album_artist: Option<String>,
}

impl Default for TrackMetadata {
    fn default() -> Self {
        Self {
            title: "Unknown Title".to_string(),
            artist: "Unknown Artist".to_string(),
            album: "Unknown Album".to_string(),
            year: "----".to_string(),
            duration: None,
            genre: None,
            track_number: None,
            album_artist: None,
        }
    }
}

/// Album art data
#[derive(Debug, Clone)]
pub struct AlbumArt {
    pub data: Vec<u8>,
    pub mime_type: String,
    pub width: u32,
    pub height: u32,
}

/// Metadata extractor trait for dependency inversion
pub trait MetadataExtractorInterface {
    /// Extract metadata from an audio file
    fn extract_metadata(&self, path: &Path) -> Result<TrackMetadata>;

    /// Extract album art from an audio file
    fn extract_album_art(&self, path: &Path) -> Result<Option<AlbumArt>>;

    /// Process album art data into egui texture format
    fn process_album_art_for_ui(&self, album_art: &AlbumArt) -> Result<egui::ColorImage>;
}

/// Lofty-based metadata extractor implementation
pub struct LoftyMetadataExtractor;

impl LoftyMetadataExtractor {
    pub fn new() -> Self {
        info!("Creating Lofty metadata extractor");
        Self
    }
}

impl MetadataExtractorInterface for LoftyMetadataExtractor {
    fn extract_metadata(&self, path: &Path) -> Result<TrackMetadata> {
        debug!("Extracting metadata from: {:?}", path);

        let tagged_file =
            lofty::read_from_path(path).map_err(|e| MetadataError::LoftyError(e.to_string()))?;

        let properties = tagged_file.file_type();
        let duration = tagged_file.properties().duration();

        let metadata = if let Some(tag) = tagged_file.primary_tag() {
            TrackMetadata {
                title: tag
                    .title()
                    .as_deref()
                    .unwrap_or("Unknown Title")
                    .to_string(),
                artist: tag
                    .artist()
                    .as_deref()
                    .unwrap_or("Unknown Artist")
                    .to_string(),
                album: tag
                    .album()
                    .as_deref()
                    .unwrap_or("Unknown Album")
                    .to_string(),
                year: tag
                    .year()
                    .map(|y| y.to_string())
                    .unwrap_or_else(|| "----".to_string()),
                duration: Some(duration),
                genre: tag.genre().as_deref().map(|s| s.to_string()),
                track_number: tag.track().map(|t| t as u32),
                album_artist: tag.artist().as_deref().map(|s| s.to_string()), // Use artist as fallback
            }
        } else {
            warn!("No primary tag found, using defaults");
            TrackMetadata {
                duration: Some(duration),
                ..Default::default()
            }
        };

        info!(
            "Extracted metadata: {} - {} ({})",
            metadata.artist, metadata.title, metadata.album
        );
        Ok(metadata)
    }

    fn extract_album_art(&self, path: &Path) -> Result<Option<AlbumArt>> {
        debug!("Extracting album art from: {:?}", path);

        let tagged_file =
            lofty::read_from_path(path).map_err(|e| MetadataError::LoftyError(e.to_string()))?;

        if let Some(picture) = tagged_file.primary_tag().and_then(|t| t.pictures().get(0)) {
            debug!("Found album art, size: {} bytes", picture.data().len());

            // Try to get dimensions using image crate
            match image::load_from_memory(picture.data()) {
                Ok(img) => {
                    let (width, height) = img.dimensions();
                    Ok(Some(AlbumArt {
                        data: picture.data().to_vec(),
                        mime_type: picture
                            .mime_type()
                            .map(|m| m.to_string())
                            .unwrap_or_else(|| "unknown".to_string()),
                        width,
                        height,
                    }))
                }
                Err(e) => {
                    warn!("Failed to process album art image: {}", e);
                    Err(ImageError::ProcessingFailed {
                        details: e.to_string(),
                    }
                    .into())
                }
            }
        } else {
            debug!("No album art found");
            Ok(None)
        }
    }

    fn process_album_art_for_ui(&self, album_art: &AlbumArt) -> Result<egui::ColorImage> {
        debug!(
            "Processing album art for UI: {}x{}",
            album_art.width, album_art.height
        );

        let img =
            image::load_from_memory(&album_art.data).map_err(|e| ImageError::ProcessingFailed {
                details: e.to_string(),
            })?;

        let (width, height) = img.dimensions();
        let rgba = img.to_rgba8();
        let pixels = rgba.into_raw();

        Ok(egui::ColorImage::from_rgba_unmultiplied(
            [width as usize, height as usize],
            &pixels,
        ))
    }
}

impl Default for LoftyMetadataExtractor {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility functions for metadata handling
pub mod utils {
    use super::*;

    /// Format duration as MM:SS
    pub fn format_duration(duration: std::time::Duration) -> String {
        let total_seconds = duration.as_secs();
        let minutes = total_seconds / 60;
        let seconds = total_seconds % 60;
        format!("{:02}:{:02}", minutes, seconds)
    }

    /// Get a display-friendly file name from a path
    pub fn get_display_filename(path: &Path) -> String {
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Unknown File")
            .to_string()
    }

    /// Check if a file extension is supported for audio
    pub fn is_supported_audio_format(path: &Path) -> bool {
        if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
            matches!(
                extension.to_lowercase().as_str(),
                "mp3" | "wav" | "flac" | "ogg" | "m4a" | "aac" | "wma" | "opus"
            )
        } else {
            false
        }
    }

    /// Sanitize metadata strings for display
    pub fn sanitize_metadata_string(input: &str) -> String {
        input
            .chars()
            .filter(|c| c.is_ascii_graphic() || c.is_whitespace())
            .collect::<String>()
            .trim()
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_format_duration() {
        assert_eq!(utils::format_duration(Duration::from_secs(0)), "00:00");
        assert_eq!(utils::format_duration(Duration::from_secs(65)), "01:05");
        assert_eq!(utils::format_duration(Duration::from_secs(3661)), "61:01");
    }

    #[test]
    fn test_is_supported_audio_format() {
        assert!(utils::is_supported_audio_format(Path::new("test.mp3")));
        assert!(utils::is_supported_audio_format(Path::new("TEST.MP3")));
        assert!(utils::is_supported_audio_format(Path::new("song.flac")));
        assert!(!utils::is_supported_audio_format(Path::new("video.mp4")));
        assert!(!utils::is_supported_audio_format(Path::new("document.txt")));
    }

    #[test]
    fn test_sanitize_metadata_string() {
        assert_eq!(
            utils::sanitize_metadata_string("  Normal Text  "),
            "Normal Text"
        );
        assert_eq!(
            utils::sanitize_metadata_string("Text\x00with\x01control"),
            "Textwithcontrol"
        );
    }
}
