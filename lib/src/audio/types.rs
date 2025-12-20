//! Core types for audio processing
//!
//! This module defines the foundational types for audio processing in the DarkMatter DSL,
//! including source types, format detection, metadata structures, and processing I/O types.

use std::path::PathBuf;
use xxhash_rust::xxh3::xxh3_64;

/// Audio source location (local file or remote URL)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AudioSource {
    /// Local file path
    Local(PathBuf),
    /// Remote URL (HTTP/HTTPS)
    Remote(String),
}

impl AudioSource {
    /// Compute a stable hash for the resource location
    ///
    /// This hash is used for cache lookups and file naming. It's based on the
    /// resource path/URL, not the content.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use lib::audio::types::AudioSource;
    ///
    /// let source = AudioSource::Local(PathBuf::from("audio/podcast.mp3"));
    /// let hash1 = source.resource_hash();
    /// let hash2 = source.resource_hash();
    /// assert_eq!(hash1, hash2); // Deterministic
    /// ```
    pub fn resource_hash(&self) -> u64 {
        match self {
            AudioSource::Local(path) => {
                let path_str = path.to_string_lossy();
                xxh3_64(path_str.as_bytes())
            }
            AudioSource::Remote(url) => xxh3_64(url.as_bytes()),
        }
    }
}

/// Supported audio formats
///
/// This enum is marked `#[non_exhaustive]` to allow adding new formats
/// in the future without breaking existing code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum AudioFormat {
    /// MP3 audio format
    Mp3,
    /// WAV audio format
    Wav,
}

impl AudioFormat {
    /// Detect audio format from file extension
    ///
    /// # Arguments
    ///
    /// * `extension` - File extension (with or without leading dot)
    ///
    /// # Returns
    ///
    /// `Some(AudioFormat)` if the extension is recognized, `None` otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use lib::audio::types::AudioFormat;
    ///
    /// assert_eq!(AudioFormat::from_extension("mp3"), Some(AudioFormat::Mp3));
    /// assert_eq!(AudioFormat::from_extension(".wav"), Some(AudioFormat::Wav));
    /// assert_eq!(AudioFormat::from_extension("MP3"), Some(AudioFormat::Mp3));
    /// assert_eq!(AudioFormat::from_extension("ogg"), None);
    /// ```
    pub fn from_extension(ext: &str) -> Option<Self> {
        let normalized = ext.trim_start_matches('.').to_lowercase();
        match normalized.as_str() {
            "mp3" => Some(AudioFormat::Mp3),
            "wav" => Some(AudioFormat::Wav),
            _ => None,
        }
    }

    /// Get the MIME type for this audio format
    ///
    /// # Examples
    ///
    /// ```
    /// use lib::audio::types::AudioFormat;
    ///
    /// assert_eq!(AudioFormat::Mp3.mime_type(), "audio/mpeg");
    /// assert_eq!(AudioFormat::Wav.mime_type(), "audio/wav");
    /// ```
    pub fn mime_type(&self) -> &'static str {
        match self {
            AudioFormat::Mp3 => "audio/mpeg",
            AudioFormat::Wav => "audio/wav",
        }
    }

    /// Get the file extension for this audio format (without leading dot)
    ///
    /// # Examples
    ///
    /// ```
    /// use lib::audio::types::AudioFormat;
    ///
    /// assert_eq!(AudioFormat::Mp3.extension(), "mp3");
    /// assert_eq!(AudioFormat::Wav.extension(), "wav");
    /// ```
    pub fn extension(&self) -> &'static str {
        match self {
            AudioFormat::Mp3 => "mp3",
            AudioFormat::Wav => "wav",
        }
    }
}

/// Audio metadata extracted from files
///
/// This includes technical metadata (duration, bitrate, etc.) and
/// ID3 tags (title, artist, album) if present.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AudioMetadata {
    /// Duration in seconds (None if unknown)
    pub duration_secs: Option<f32>,
    /// Bitrate in bits per second (None if unknown)
    pub bitrate: Option<u32>,
    /// Sample rate in Hz (None if unknown)
    pub sample_rate: Option<u32>,
    /// Number of audio channels (None if unknown)
    pub channels: Option<u16>,
    /// Title from ID3 tags (None if not present)
    pub title: Option<String>,
    /// Artist from ID3 tags (None if not present)
    pub artist: Option<String>,
    /// Album from ID3 tags (None if not present)
    pub album: Option<String>,
}

/// Input specification for audio processing
#[derive(Debug, Clone)]
pub struct AudioInput {
    /// Source location of the audio file
    pub source: AudioSource,
    /// Optional display name (overrides metadata title)
    pub name: Option<String>,
}

/// Output from audio processing
#[derive(Debug, Clone)]
pub struct AudioOutput {
    /// Detected audio format
    pub format: AudioFormat,
    /// Extracted metadata
    pub metadata: AudioMetadata,
    /// Path to processed audio file (relative to output directory)
    pub path: String,
    /// Base64-encoded audio data (populated only in inline mode)
    pub base64_data: Option<String>,
    /// Display name (from input.name, metadata.title, or filename)
    pub display_name: String,
}

/// Configuration for audio processing
#[derive(Debug, Clone)]
pub struct AudioProcessingConfig {
    /// Maximum file size allowed for processing (None = unlimited)
    pub max_file_size: Option<u64>,
    /// Maximum file size for inline mode (base64 encoding)
    /// Default: 10MB (10 * 1024 * 1024 bytes)
    pub max_inline_size: u64,
    /// Allowed audio formats
    pub allowed_formats: Vec<AudioFormat>,
}

impl Default for AudioProcessingConfig {
    fn default() -> Self {
        Self {
            max_file_size: None, // No limit by default
            max_inline_size: 10 * 1024 * 1024, // 10MB default
            allowed_formats: vec![AudioFormat::Mp3, AudioFormat::Wav],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audio_source_local_hash_is_deterministic() {
        let source = AudioSource::Local(PathBuf::from("test/audio.mp3"));
        let hash1 = source.resource_hash();
        let hash2 = source.resource_hash();
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn audio_source_remote_hash_is_deterministic() {
        let source = AudioSource::Remote("https://example.com/audio.mp3".to_string());
        let hash1 = source.resource_hash();
        let hash2 = source.resource_hash();
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn audio_source_different_paths_produce_different_hashes() {
        let source1 = AudioSource::Local(PathBuf::from("test/audio1.mp3"));
        let source2 = AudioSource::Local(PathBuf::from("test/audio2.mp3"));
        assert_ne!(source1.resource_hash(), source2.resource_hash());
    }

    #[test]
    fn audio_format_from_extension_recognizes_mp3() {
        assert_eq!(AudioFormat::from_extension("mp3"), Some(AudioFormat::Mp3));
        assert_eq!(AudioFormat::from_extension(".mp3"), Some(AudioFormat::Mp3));
        assert_eq!(AudioFormat::from_extension("MP3"), Some(AudioFormat::Mp3));
        assert_eq!(AudioFormat::from_extension(".MP3"), Some(AudioFormat::Mp3));
    }

    #[test]
    fn audio_format_from_extension_recognizes_wav() {
        assert_eq!(AudioFormat::from_extension("wav"), Some(AudioFormat::Wav));
        assert_eq!(AudioFormat::from_extension(".wav"), Some(AudioFormat::Wav));
        assert_eq!(AudioFormat::from_extension("WAV"), Some(AudioFormat::Wav));
        assert_eq!(AudioFormat::from_extension(".WAV"), Some(AudioFormat::Wav));
    }

    #[test]
    fn audio_format_from_extension_rejects_unsupported() {
        assert_eq!(AudioFormat::from_extension("ogg"), None);
        assert_eq!(AudioFormat::from_extension("flac"), None);
        assert_eq!(AudioFormat::from_extension("aac"), None);
        assert_eq!(AudioFormat::from_extension("m4a"), None);
        assert_eq!(AudioFormat::from_extension(""), None);
    }

    #[test]
    fn audio_format_mp3_mime_type() {
        assert_eq!(AudioFormat::Mp3.mime_type(), "audio/mpeg");
    }

    #[test]
    fn audio_format_wav_mime_type() {
        assert_eq!(AudioFormat::Wav.mime_type(), "audio/wav");
    }

    #[test]
    fn audio_format_mp3_extension() {
        assert_eq!(AudioFormat::Mp3.extension(), "mp3");
    }

    #[test]
    fn audio_format_wav_extension() {
        assert_eq!(AudioFormat::Wav.extension(), "wav");
    }

    #[test]
    fn audio_metadata_default_has_none_values() {
        let metadata = AudioMetadata::default();
        assert_eq!(metadata.duration_secs, None);
        assert_eq!(metadata.bitrate, None);
        assert_eq!(metadata.sample_rate, None);
        assert_eq!(metadata.channels, None);
        assert_eq!(metadata.title, None);
        assert_eq!(metadata.artist, None);
        assert_eq!(metadata.album, None);
    }

    #[test]
    fn audio_input_construction() {
        let input = AudioInput {
            source: AudioSource::Local(PathBuf::from("audio.mp3")),
            name: Some("My Audio".to_string()),
        };
        assert_eq!(input.name, Some("My Audio".to_string()));
    }

    #[test]
    fn audio_output_construction() {
        let output = AudioOutput {
            format: AudioFormat::Mp3,
            metadata: AudioMetadata::default(),
            path: "audio/12345.mp3".to_string(),
            base64_data: None,
            display_name: "Test Audio".to_string(),
        };
        assert_eq!(output.format, AudioFormat::Mp3);
        assert_eq!(output.path, "audio/12345.mp3");
        assert_eq!(output.display_name, "Test Audio");
        assert_eq!(output.base64_data, None);
    }

    #[test]
    fn audio_processing_config_default() {
        let config = AudioProcessingConfig::default();
        assert_eq!(config.max_file_size, None);
        assert_eq!(config.max_inline_size, 10 * 1024 * 1024);
        assert_eq!(config.allowed_formats.len(), 2);
        assert!(config.allowed_formats.contains(&AudioFormat::Mp3));
        assert!(config.allowed_formats.contains(&AudioFormat::Wav));
    }

    #[test]
    fn audio_processing_config_custom() {
        let config = AudioProcessingConfig {
            max_file_size: Some(100 * 1024 * 1024), // 100MB
            max_inline_size: 5 * 1024 * 1024,       // 5MB
            allowed_formats: vec![AudioFormat::Mp3],
        };
        assert_eq!(config.max_file_size, Some(100 * 1024 * 1024));
        assert_eq!(config.max_inline_size, 5 * 1024 * 1024);
        assert_eq!(config.allowed_formats, vec![AudioFormat::Mp3]);
    }
}
