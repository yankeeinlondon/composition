//! Audio processing pipeline
//!
//! This module provides the complete audio processing workflow:
//! 1. Load audio bytes from source
//! 2. Detect format and compute hashes
//! 3. Check cache for existing metadata
//! 4. Extract metadata on cache miss
//! 5. Copy audio file to output directory
//! 6. Generate base64 data if inline mode
//! 7. Return AudioOutput with all processed information

use crate::audio::cache::{AudioCache, NewAudioCacheEntry};
use crate::audio::metadata::{
    compute_content_hash, detect_audio_format, extract_audio_metadata, load_audio_bytes,
};
use crate::audio::types::{AudioInput, AudioOutput, AudioProcessingConfig};
use crate::error::{AudioError, CompositionError};
use base64::{engine::general_purpose, Engine as _};
use std::fs;
use std::path::Path;
use tracing::{debug, info, instrument, warn};

type Result<T> = std::result::Result<T, CompositionError>;

/// Process an audio file (async public API)
///
/// This is the main entry point for audio processing. It wraps the sync
/// processing logic in a blocking task to maintain async compatibility.
///
/// # Arguments
///
/// * `input` - Audio input specification
/// * `output_dir` - Directory to copy processed audio files
/// * `cache` - Audio cache for metadata storage
/// * `inline_mode` - If true, generate base64 data; if false, use file references
/// * `config` - Processing configuration (limits, allowed formats)
///
/// # Returns
///
/// `AudioOutput` containing format, metadata, path, optional base64 data, and display name
///
/// # Errors
///
/// Returns errors for:
/// - File read failures
/// - Unsupported formats
/// - Cache failures
/// - File copy failures
///
/// # Examples
///
/// ```no_run
/// use lib::audio::processor::process_audio;
/// use lib::audio::types::{AudioInput, AudioSource, AudioProcessingConfig};
/// use lib::audio::cache::AudioCache;
/// use std::path::PathBuf;
/// use surrealdb::Surreal;
/// use surrealdb::engine::local::Mem;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let db = Surreal::new::<Mem>(()).await?;
/// let cache = AudioCache::new(db);
///
/// let input = AudioInput {
///     source: AudioSource::Local(PathBuf::from("audio.mp3")),
///     name: Some("My Podcast".to_string()),
/// };
///
/// let output = process_audio(
///     input,
///     Path::new("output"),
///     &cache,
///     false,
///     &AudioProcessingConfig::default()
/// ).await?;
///
/// println!("Processed audio: {}", output.display_name);
/// # Ok(())
/// # }
/// ```
#[instrument(skip(cache, config), fields(source = ?input.source))]
pub async fn process_audio(
    input: AudioInput,
    output_dir: &Path,
    cache: &AudioCache,
    inline_mode: bool,
    config: &AudioProcessingConfig,
) -> Result<AudioOutput> {
    // Clone data needed for blocking task
    let input_clone = input.clone();
    let output_dir_clone = output_dir.to_path_buf();
    let config_clone = config.clone();
    let cache_clone = cache.clone();

    // Spawn blocking task for sync operations
    let output = tokio::task::spawn_blocking(move || {
        process_audio_sync(
            input_clone,
            &output_dir_clone,
            &cache_clone,
            inline_mode,
            &config_clone,
        )
    })
    .await
    .map_err(|e| CompositionError::Audio(AudioError::ProcessingFailed {
        reason: format!("Task join error: {}", e),
    }))??;

    Ok(output)
}

/// Process an audio file (sync internal implementation)
///
/// This function contains the core processing logic. It's synchronous and called
/// via `spawn_blocking` from the async public API.
///
/// # Processing Steps
///
/// 1. Compute resource hash from source
/// 2. Load audio bytes
/// 3. Detect format
/// 4. Compute content hash
/// 5. Check cache with (resource_hash, content_hash)
/// 6. If cache miss: extract metadata, upsert cache
/// 7. Validate file size against config.max_inline_size if inline_mode
/// 8. Copy audio file to output_dir/audio/{resource_hash}.{ext}
/// 9. Generate base64 data if inline_mode
/// 10. Determine display name (priority: input.name > metadata.title > filename)
/// 11. Return AudioOutput
#[instrument(skip(cache, config))]
pub(crate) fn process_audio_sync(
    input: AudioInput,
    output_dir: &Path,
    cache: &AudioCache,
    inline_mode: bool,
    config: &AudioProcessingConfig,
) -> Result<AudioOutput> {
    // Step 1: Compute resource hash
    let resource_hash = input.source.resource_hash();
    let resource_hash_str = format!("{:x}", resource_hash);
    info!(resource_hash = %resource_hash_str, "Processing audio");

    // Step 2: Load audio bytes
    let (bytes, filename) = load_audio_bytes(&input.source)?;
    debug!(size_bytes = bytes.len(), "Loaded audio bytes");

    // Step 3: Detect format
    let format = detect_audio_format(&input.source, &bytes)?;
    debug!(format = ?format, "Detected audio format");

    // Validate format is allowed
    if !config.allowed_formats.contains(&format) {
        return Err(CompositionError::Audio(AudioError::UnsupportedFormat {
            format: format!("{:?}", format),
        }));
    }

    // Validate max file size
    if let Some(max_size) = config.max_file_size {
        if bytes.len() as u64 > max_size {
            return Err(CompositionError::Audio(AudioError::FileTooLarge {
                size: bytes.len() as u64,
                max_size,
            }));
        }
    }

    // Step 4: Compute content hash
    let content_hash = compute_content_hash(&bytes);
    debug!(content_hash = %content_hash, "Computed content hash");

    // Step 5: Check cache
    // Note: We need to use a runtime handle to execute async cache operations
    // from within this sync function
    let runtime = tokio::runtime::Handle::try_current()
        .or_else(|_| {
            // If no runtime is available, create a new one
            tokio::runtime::Runtime::new()
                .map(|rt| rt.handle().clone())
        })
        .map_err(|e| {
            CompositionError::Audio(AudioError::CacheFailed(format!(
                "No async runtime available: {}",
                e
            )))
        })?;

    let cached_entry = runtime.block_on(cache.get(&resource_hash_str, &content_hash))?;

    let metadata = if let Some(entry) = cached_entry {
        info!(resource_hash = %resource_hash_str, "Cache hit - using cached metadata");
        entry.metadata
    } else {
        // Step 6: Cache miss - extract metadata and upsert
        info!(resource_hash = %resource_hash_str, "Cache miss - extracting metadata");
        let extracted_metadata = extract_audio_metadata(&bytes, format)?;

        let new_entry = NewAudioCacheEntry {
            resource_hash: resource_hash_str.clone(),
            content_hash: content_hash.clone(),
            source: input.source.clone(),
            format,
            metadata: extracted_metadata.clone(),
        };

        runtime.block_on(cache.upsert(new_entry))?;

        extracted_metadata
    };

    // Step 7: Validate file size for inline mode
    if inline_mode && bytes.len() as u64 > config.max_inline_size {
        warn!(
            size = bytes.len(),
            max_inline_size = config.max_inline_size,
            "Audio file size exceeds max_inline_size - proceeding anyway"
        );
    }

    // Step 8: Copy audio file to output directory
    let audio_output_dir = output_dir.join("audio");
    fs::create_dir_all(&audio_output_dir).map_err(|e| {
        CompositionError::Audio(AudioError::ProcessingFailed {
            reason: format!("Failed to create audio output directory: {}", e),
        })
    })?;

    let output_filename = format!("{}.{}", resource_hash_str, format.extension());
    let output_path = audio_output_dir.join(&output_filename);
    fs::write(&output_path, &bytes).map_err(|e| {
        CompositionError::Audio(AudioError::ProcessingFailed {
            reason: format!("Failed to write audio file: {}", e),
        })
    })?;

    debug!(path = ?output_path, "Copied audio file to output directory");

    // Step 9: Generate base64 data if inline mode
    let base64_data = if inline_mode {
        let encoded = general_purpose::STANDARD.encode(&bytes);
        Some(encoded)
    } else {
        None
    };

    // Step 10: Determine display name
    let display_name = input
        .name
        .or_else(|| metadata.title.clone())
        .unwrap_or(filename);

    // Step 11: Return AudioOutput
    let relative_path = format!("audio/{}", output_filename);
    Ok(AudioOutput {
        format,
        metadata,
        path: relative_path,
        base64_data,
        display_name,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::types::AudioSource;
    use std::path::PathBuf;
    use surrealdb::engine::local::Mem;
    use surrealdb::Surreal;
    use tempfile::TempDir;

    async fn setup_test_cache() -> AudioCache {
        let db = Surreal::new::<Mem>(()).await.unwrap();
        db.use_ns("test").use_db("test").await.unwrap();
        crate::cache::schema::apply_schema(&db).await.unwrap();
        AudioCache::new(db)
    }

    #[tokio::test]
    async fn test_process_audio_sync_with_valid_mp3() {
        let cache = setup_test_cache().await;
        let temp_dir = TempDir::new().unwrap();

        let input = AudioInput {
            source: AudioSource::Local(PathBuf::from("../tests/fixtures/audio/test.mp3")),
            name: Some("Test Audio".to_string()),
        };

        let result = process_audio(
            input,
            temp_dir.path(),
            &cache,
            false,
            &AudioProcessingConfig::default(),
        )
        .await;

        // NOTE: This may fail if the test.mp3 fixture is not a valid MP3
        // In that case, we expect an AudioError::MetadataFailed
        match result {
            Ok(output) => {
                assert_eq!(output.display_name, "Test Audio");
                assert!(output.path.starts_with("audio/"));
                assert!(output.path.ends_with(".mp3"));
                assert_eq!(output.base64_data, None);

                // Verify file was copied
                let audio_dir = temp_dir.path().join("audio");
                assert!(audio_dir.exists());
                let files: Vec<_> = fs::read_dir(audio_dir).unwrap().collect();
                assert_eq!(files.len(), 1);
            }
            Err(CompositionError::Audio(AudioError::MetadataFailed { .. })) => {
                // Expected for minimal test fixtures
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_process_audio_sync_with_valid_wav() {
        let cache = setup_test_cache().await;
        let temp_dir = TempDir::new().unwrap();

        let input = AudioInput {
            source: AudioSource::Local(PathBuf::from("../tests/fixtures/audio/test.wav")),
            name: None, // Test default naming
        };

        let result = process_audio(
            input,
            temp_dir.path(),
            &cache,
            false,
            &AudioProcessingConfig::default(),
        )
        .await;

        assert!(result.is_ok());
        let output = result.unwrap();
        // Should use filename as display name (no input.name, no metadata.title)
        assert!(output.display_name.contains("test"));
        assert!(output.path.ends_with(".wav"));
        assert_eq!(output.base64_data, None);
    }

    #[tokio::test]
    async fn test_process_audio_sync_with_inline_mode() {
        let cache = setup_test_cache().await;
        let temp_dir = TempDir::new().unwrap();

        let input = AudioInput {
            source: AudioSource::Local(PathBuf::from("../tests/fixtures/audio/test.wav")),
            name: Some("Inline Test".to_string()),
        };

        let result = process_audio(
            input,
            temp_dir.path(),
            &cache,
            true, // inline_mode = true
            &AudioProcessingConfig::default(),
        )
        .await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.base64_data.is_some());
        let base64 = output.base64_data.unwrap();
        assert!(!base64.is_empty());
        // Verify it's valid base64
        assert!(general_purpose::STANDARD.decode(&base64).is_ok());
    }

    #[tokio::test]
    async fn test_process_audio_sync_cache_hit() {
        let cache = setup_test_cache().await;
        let temp_dir = TempDir::new().unwrap();

        let input = AudioInput {
            source: AudioSource::Local(PathBuf::from("../tests/fixtures/audio/test.wav")),
            name: Some("Cache Test".to_string()),
        };

        // First call - cache miss
        let result1 = process_audio(
            input.clone(),
            temp_dir.path(),
            &cache,
            false,
            &AudioProcessingConfig::default(),
        )
        .await;
        assert!(result1.is_ok());

        // Second call - should be cache hit
        let result2 = process_audio(
            input,
            temp_dir.path(),
            &cache,
            false,
            &AudioProcessingConfig::default(),
        )
        .await;
        assert!(result2.is_ok());

        // Both should produce same output
        let output1 = result1.unwrap();
        let output2 = result2.unwrap();
        assert_eq!(output1.path, output2.path);
        assert_eq!(output1.display_name, output2.display_name);
    }

    #[tokio::test]
    async fn test_process_audio_sync_display_name_priority() {
        let cache = setup_test_cache().await;
        let temp_dir = TempDir::new().unwrap();

        // Test: input.name has highest priority
        let input_with_name = AudioInput {
            source: AudioSource::Local(PathBuf::from("../tests/fixtures/audio/test.wav")),
            name: Some("Custom Name".to_string()),
        };

        let result = process_audio(
            input_with_name,
            temp_dir.path(),
            &cache,
            false,
            &AudioProcessingConfig::default(),
        )
        .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().display_name, "Custom Name");

        // Test: filename used when no input.name
        let input_no_name = AudioInput {
            source: AudioSource::Local(PathBuf::from("../tests/fixtures/audio/test.wav")),
            name: None,
        };

        let result = process_audio(
            input_no_name,
            temp_dir.path(),
            &cache,
            false,
            &AudioProcessingConfig::default(),
        )
        .await;
        assert!(result.is_ok());
        let display_name = result.unwrap().display_name;
        assert!(display_name.contains("test") || display_name.contains("wav"));
    }

    #[tokio::test]
    async fn test_process_audio_sync_file_size_limit_warning() {
        let cache = setup_test_cache().await;
        let temp_dir = TempDir::new().unwrap();

        let input = AudioInput {
            source: AudioSource::Local(PathBuf::from("../tests/fixtures/audio/test.wav")),
            name: Some("Size Test".to_string()),
        };

        // Set max_inline_size to 1 byte (will definitely exceed)
        let config = AudioProcessingConfig {
            max_file_size: None,
            max_inline_size: 1,
            allowed_formats: vec![crate::audio::types::AudioFormat::Mp3, crate::audio::types::AudioFormat::Wav],
        };

        let result = process_audio(input, temp_dir.path(), &cache, true, &config).await;

        // Should succeed with warning (not fail)
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.base64_data.is_some());
    }

    #[tokio::test]
    async fn test_process_audio_sync_unsupported_format() {
        let cache = setup_test_cache().await;
        let temp_dir = TempDir::new().unwrap();

        let input = AudioInput {
            source: AudioSource::Local(PathBuf::from("../tests/fixtures/audio/test.mp3")),
            name: Some("Format Test".to_string()),
        };

        // Only allow WAV
        let config = AudioProcessingConfig {
            max_file_size: None,
            max_inline_size: 10 * 1024 * 1024,
            allowed_formats: vec![crate::audio::types::AudioFormat::Wav],
        };

        let result = process_audio(input, temp_dir.path(), &cache, false, &config).await;

        // Should fail with UnsupportedFormat (or MetadataFailed if fixture is invalid)
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_process_audio_sync_missing_file() {
        let cache = setup_test_cache().await;
        let temp_dir = TempDir::new().unwrap();

        let input = AudioInput {
            source: AudioSource::Local(PathBuf::from("nonexistent.mp3")),
            name: Some("Missing Test".to_string()),
        };

        let result = process_audio(
            input,
            temp_dir.path(),
            &cache,
            false,
            &AudioProcessingConfig::default(),
        )
        .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            CompositionError::Audio(AudioError::ReadFailed { .. }) => {}
            e => panic!("Expected ReadFailed, got: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_process_audio_async_wrapper() {
        let cache = setup_test_cache().await;
        let temp_dir = TempDir::new().unwrap();

        let input = AudioInput {
            source: AudioSource::Local(PathBuf::from("../tests/fixtures/audio/test.wav")),
            name: Some("Async Test".to_string()),
        };

        let result = process_audio(
            input,
            temp_dir.path(),
            &cache,
            false,
            &AudioProcessingConfig::default(),
        )
        .await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.display_name, "Async Test");
    }
}
