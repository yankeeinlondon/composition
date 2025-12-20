//! Audio processing module for the DarkMatter DSL
//!
//! This module provides functionality for processing audio files in markdown documents,
//! including format detection, metadata extraction, caching, and HTML player generation.
//!
//! # Examples
//!
//! ```no_run
//! use lib::audio::types::{AudioSource, AudioInput, AudioFormat};
//! use std::path::PathBuf;
//!
//! let input = AudioInput {
//!     source: AudioSource::Local(PathBuf::from("podcast.mp3")),
//!     name: Some("Episode 1".to_string()),
//! };
//!
//! let hash = input.source.resource_hash();
//! println!("Resource hash: {}", hash);
//! ```

pub mod cache;
pub mod html;
pub mod metadata;
pub mod processor;
pub mod types;

// Re-export commonly used types
pub use cache::{AudioCache, AudioCacheEntry, NewAudioCacheEntry};
pub use html::{generate_audio_html, html_escape, AudioHtmlOptions};
pub use metadata::{
    compute_content_hash, detect_audio_format, extract_audio_metadata, load_audio_bytes,
};
pub use processor::process_audio;
pub use types::{
    AudioFormat, AudioInput, AudioMetadata, AudioOutput, AudioProcessingConfig, AudioSource,
};
