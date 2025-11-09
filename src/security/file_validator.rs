//! File validation and sandboxing module
//!
//! Provides secure file path validation to prevent directory traversal attacks
//! and enforce file type and size restrictions.

use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// File validator for secure file operations
pub struct FileValidator {
    max_file_size: u64,
    allowed_extensions: Vec<String>,
    sandbox_root: PathBuf,
    validate_content: bool,
}

impl FileValidator {
    /// Create a new file validator with the specified sandbox root
    pub fn new(sandbox_root: PathBuf) -> Self {
        Self {
            max_file_size: 500 * 1024 * 1024, // 500MB default limit
            allowed_extensions: vec![
                "mp3".to_string(),
                "wav".to_string(),
                "flac".to_string(),
                "ogg".to_string(),
                "m4a".to_string(),
                "aac".to_string(),
            ],
            sandbox_root,
            validate_content: true,
        }
    }

    /// Set maximum file size in bytes
    pub fn set_max_file_size(&mut self, size: u64) {
        self.max_file_size = size;
    }

    /// Validate a file path for security
    pub fn validate_file_path(&self, path: &Path) -> Result<PathBuf, SecurityError> {
        // Step 1: Canonicalize the path to resolve any symlinks and relative components
        let canonical = path
            .canonicalize()
            .map_err(|e| SecurityError::InvalidPath {
                path: path.display().to_string(),
                reason: e.to_string(),
            })?;

        // Step 2: Ensure the path is within the sandbox
        if !self.is_within_sandbox(&canonical) {
            tracing::error!(
                "Path traversal attempt detected: {:?} (sandbox: {:?})",
                canonical,
                self.sandbox_root
            );
            return Err(SecurityError::PathTraversal {
                attempted_path: canonical.display().to_string(),
            });
        }

        // Step 3: Check file extension
        let ext = canonical
            .extension()
            .and_then(|s| s.to_str())
            .ok_or_else(|| SecurityError::InvalidFileType {
                path: canonical.display().to_string(),
                reason: "No file extension".to_string(),
            })?;

        if !self.is_allowed_extension(ext) {
            return Err(SecurityError::InvalidFileType {
                path: canonical.display().to_string(),
                reason: format!("Extension '{}' not allowed", ext),
            });
        }

        // Step 4: Check file size
        let metadata = fs::metadata(&canonical).map_err(|e| SecurityError::FileAccessError {
            path: canonical.display().to_string(),
            reason: e.to_string(),
        })?;

        if metadata.len() > self.max_file_size {
            return Err(SecurityError::FileTooLarge {
                size: metadata.len(),
                max_size: self.max_file_size,
            });
        }

        // Step 5: Optional content validation (check file magic numbers)
        if self.validate_content {
            self.validate_file_content(&canonical)?;
        }

        Ok(canonical)
    }

    /// Check if a path is within the sandbox
    fn is_within_sandbox(&self, path: &Path) -> bool {
        path.starts_with(&self.sandbox_root)
    }

    /// Check if a file extension is allowed
    fn is_allowed_extension(&self, ext: &str) -> bool {
        self.allowed_extensions.contains(&ext.to_lowercase())
    }

    /// Validate file content by checking magic numbers
    fn validate_file_content(&self, path: &Path) -> Result<(), SecurityError> {
        use std::io::Read;

        let mut file = fs::File::open(path).map_err(|e| SecurityError::FileAccessError {
            path: path.display().to_string(),
            reason: e.to_string(),
        })?;

        let mut magic_bytes = [0u8; 16];
        file.read_exact(&mut magic_bytes)
            .map_err(|e| SecurityError::InvalidContent {
                path: path.display().to_string(),
                reason: format!("Cannot read file header: {}", e),
            })?;

        // Check for known audio file magic numbers
        let is_valid = match &magic_bytes[..4] {
            // MP3 with ID3v2
            [0x49, 0x44, 0x33, _] => true,
            // MP3 without ID3 (frame sync)
            [0xFF, 0xFB, _, _] | [0xFF, 0xFA, _, _] | [0xFF, 0xF3, _, _] => true,
            // WAV/RIFF
            [0x52, 0x49, 0x46, 0x46] => {
                // Check for WAVE signature at offset 8
                &magic_bytes[8..12] == b"WAVE"
            }
            // FLAC
            [0x66, 0x4C, 0x61, 0x43] => true,
            // OGG
            [0x4F, 0x67, 0x67, 0x53] => true,
            // M4A/MP4
            _ if &magic_bytes[4..8] == b"ftyp" => true,
            _ => false,
        };

        if !is_valid {
            return Err(SecurityError::InvalidContent {
                path: path.display().to_string(),
                reason: "File content does not match expected audio format".to_string(),
            });
        }

        Ok(())
    }

    /// Create a safe filename from user input
    pub fn sanitize_filename(filename: &str) -> String {
        filename
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect::<String>()
            .chars()
            .take(255) // Max filename length
            .collect()
    }
}

/// Security errors that can occur during file validation
#[derive(Debug, Error)]
pub enum SecurityError {
    #[error("Invalid file path: {path} ({reason})")]
    InvalidPath { path: String, reason: String },

    #[error("Path traversal detected: {attempted_path}")]
    PathTraversal { attempted_path: String },

    #[error("Invalid file type: {path} ({reason})")]
    InvalidFileType { path: String, reason: String },

    #[error("File too large: {size} bytes (max: {max_size} bytes)")]
    FileTooLarge { size: u64, max_size: u64 },

    #[error("File access error: {path} ({reason})")]
    FileAccessError { path: String, reason: String },

    #[error("Invalid file content: {path} ({reason})")]
    InvalidContent { path: String, reason: String },

    #[error("Sandbox directory not found")]
    SandboxNotFound,

    #[error("Audio limiter not operational")]
    LimiterNotOperational,

    #[error("Security monitoring inactive")]
    MonitoringInactive,

    #[error("Configuration error: {0}")]
    ConfigError(String),
}

impl From<super::secure_config::ConfigError> for SecurityError {
    fn from(err: super::secure_config::ConfigError) -> Self {
        SecurityError::ConfigError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_path_traversal_prevention() {
        let temp_dir = TempDir::new().unwrap();
        let validator = FileValidator::new(temp_dir.path().to_path_buf());

        // Various path traversal attempts
        let attacks = vec![
            "../../../etc/passwd",
            "..\\..\\..\\windows\\system32\\config\\sam",
            "./../../sensitive.txt",
            "audio/../../../etc/shadow",
            "audio/./../../etc/hosts",
        ];

        for attack in attacks {
            let result = validator.validate_file_path(Path::new(attack));
            assert!(result.is_err(), "Path traversal not caught: {}", attack);
        }
    }

    #[test]
    fn test_file_extension_validation() {
        let temp_dir = TempDir::new().unwrap();
        let validator = FileValidator::new(temp_dir.path().to_path_buf());

        // Create test files
        let valid_file = temp_dir.path().join("test.mp3");
        let invalid_file = temp_dir.path().join("test.exe");

        fs::write(&valid_file, b"ID3\x03test").unwrap();
        fs::write(&invalid_file, b"MZ\x90\x00").unwrap();

        // Valid extension should pass
        assert!(validator.validate_file_path(&valid_file).is_ok());

        // Invalid extension should fail
        assert!(validator.validate_file_path(&invalid_file).is_err());
    }

    #[test]
    fn test_filename_sanitization() {
        let test_cases: Vec<(&str, &str)> = vec![
            ("../../etc/passwd", "______etc_passwd"),
            ("file<script>.mp3", "file_script_.mp3"),
            ("file|pipe.wav", "file_pipe.wav"),
        ];

        for (input, expected) in test_cases {
            let sanitized = FileValidator::sanitize_filename(input);
            assert_eq!(sanitized, expected, "Failed to sanitize: {}", input);
        }

        // Test very long filename separately
        let long_input = format!("very*long*name*{}", "x".repeat(300));
        let expected_long = format!("very_long_name_{}", "x".repeat(238));
        let sanitized = FileValidator::sanitize_filename(&long_input);
        assert_eq!(sanitized, expected_long, "Failed to sanitize long filename");
    }
}
