//! Audio metadata caching operations
//!
//! This module provides caching for audio metadata to avoid reprocessing unchanged files.
//! The cache uses SurrealDB to store metadata indexed by resource hash and content hash.

use crate::audio::types::{AudioFormat, AudioMetadata, AudioSource};
use crate::error::{CacheError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb::engine::local::Db;
use surrealdb::sql::Datetime as SurrealDatetime;
use surrealdb::Surreal;
use tracing::{debug, info, instrument};

/// Convert chrono DateTime to SurrealDB Datetime
fn to_surreal_datetime(dt: DateTime<Utc>) -> SurrealDatetime {
    SurrealDatetime::from(dt)
}

/// Convert SurrealDB Datetime to chrono DateTime
fn from_surreal_datetime(dt: &SurrealDatetime) -> DateTime<Utc> {
    dt.0
}

/// Audio cache entry (internal representation using SurrealDB types)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AudioCacheEntryInternal {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<surrealdb::sql::Thing>,
    pub resource_hash: String,
    pub content_hash: String,
    pub created_at: SurrealDatetime,
    pub source_type: String,
    pub source: String,
    pub format: String,
    pub duration_secs: Option<f32>,
    pub bitrate: Option<i64>,
    pub sample_rate: Option<i64>,
    pub channels: Option<i64>,
}

/// Audio cache entry (public API using chrono types and domain types)
#[derive(Debug, Clone)]
pub struct AudioCacheEntry {
    pub id: Option<surrealdb::sql::Thing>,
    pub resource_hash: String,
    pub content_hash: String,
    pub created_at: DateTime<Utc>,
    pub source_type: String,
    pub source: String,
    pub format: AudioFormat,
    pub metadata: AudioMetadata,
}

impl From<AudioCacheEntryInternal> for AudioCacheEntry {
    fn from(internal: AudioCacheEntryInternal) -> Self {
        let format = match internal.format.as_str() {
            "mp3" => AudioFormat::Mp3,
            "wav" => AudioFormat::Wav,
            _ => AudioFormat::Mp3, // Default fallback
        };

        let metadata = AudioMetadata {
            duration_secs: internal.duration_secs,
            bitrate: internal.bitrate.map(|b| b as u32),
            sample_rate: internal.sample_rate.map(|s| s as u32),
            channels: internal.channels.map(|c| c as u16),
            title: None,
            artist: None,
            album: None,
        };

        Self {
            id: internal.id,
            resource_hash: internal.resource_hash,
            content_hash: internal.content_hash,
            created_at: from_surreal_datetime(&internal.created_at),
            source_type: internal.source_type,
            source: internal.source,
            format,
            metadata,
        }
    }
}

impl From<AudioCacheEntry> for AudioCacheEntryInternal {
    fn from(entry: AudioCacheEntry) -> Self {
        Self {
            id: entry.id,
            resource_hash: entry.resource_hash,
            content_hash: entry.content_hash,
            created_at: to_surreal_datetime(entry.created_at),
            source_type: entry.source_type,
            source: entry.source,
            format: entry.format.extension().to_string(),
            duration_secs: entry.metadata.duration_secs,
            bitrate: entry.metadata.bitrate.map(|b| b as i64),
            sample_rate: entry.metadata.sample_rate.map(|s| s as i64),
            channels: entry.metadata.channels.map(|c| c as i64),
        }
    }
}

/// Input for creating a new audio cache entry
#[derive(Debug, Clone)]
pub struct NewAudioCacheEntry {
    pub resource_hash: String,
    pub content_hash: String,
    pub source: AudioSource,
    pub format: AudioFormat,
    pub metadata: AudioMetadata,
}

impl From<NewAudioCacheEntry> for AudioCacheEntry {
    fn from(new_entry: NewAudioCacheEntry) -> Self {
        let (source_type, source) = match &new_entry.source {
            AudioSource::Local(path) => ("local".to_string(), path.to_string_lossy().to_string()),
            AudioSource::Remote(url) => ("remote".to_string(), url.clone()),
        };

        Self {
            id: None,
            resource_hash: new_entry.resource_hash,
            content_hash: new_entry.content_hash,
            created_at: Utc::now(),
            source_type,
            source,
            format: new_entry.format,
            metadata: new_entry.metadata,
        }
    }
}

/// Audio cache operations
#[derive(Clone)]
pub struct AudioCache {
    db: Surreal<Db>,
}

impl AudioCache {
    /// Create a new AudioCache instance with the given database connection
    ///
    /// # Arguments
    ///
    /// * `db` - SurrealDB connection
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use lib::audio::cache::AudioCache;
    /// use surrealdb::Surreal;
    /// use surrealdb::engine::local::Mem;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Surreal::new::<Mem>(()).await?;
    /// let cache = AudioCache::new(db);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(db: Surreal<Db>) -> Self {
        Self { db }
    }

    /// Get an audio cache entry by resource hash and content hash
    ///
    /// Returns `None` if no matching entry is found (cache miss).
    /// Returns `Some(entry)` if a matching entry is found (cache hit).
    ///
    /// # Arguments
    ///
    /// * `resource_hash` - Hash of the resource location (file path or URL)
    /// * `content_hash` - Hash of the audio file content
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use lib::audio::cache::AudioCache;
    /// # use surrealdb::Surreal;
    /// # use surrealdb::engine::local::Mem;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let db = Surreal::new::<Mem>(()).await?;
    /// # let cache = AudioCache::new(db);
    /// let entry = cache.get("abc123", "def456").await?;
    /// if let Some(entry) = entry {
    ///     println!("Cache hit! Duration: {:?}", entry.metadata.duration_secs);
    /// } else {
    ///     println!("Cache miss - need to extract metadata");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self))]
    pub async fn get(
        &self,
        resource_hash: &str,
        content_hash: &str,
    ) -> Result<Option<AudioCacheEntry>> {
        debug!(
            "Getting audio cache entry for resource_hash: {}, content_hash: {}",
            resource_hash, content_hash
        );

        let mut result = self
            .db
            .query(
                r#"
                SELECT * FROM audio_cache
                WHERE resource_hash = $resource_hash
                AND content_hash = $content_hash
                "#,
            )
            .bind(("resource_hash", resource_hash))
            .bind(("content_hash", content_hash))
            .await
            .map_err(|e| CacheError::QueryFailed(e.to_string()))?;

        let entry: Option<AudioCacheEntryInternal> = result
            .take(0)
            .map_err(|e| CacheError::DeserializationError(e.to_string()))?;

        if entry.is_some() {
            info!("Cache hit for resource_hash: {}", resource_hash);
        } else {
            info!("Cache miss for resource_hash: {}", resource_hash);
        }

        Ok(entry.map(AudioCacheEntry::from))
    }

    /// Insert or update an audio cache entry
    ///
    /// If an entry with the same resource_hash already exists, it will be replaced.
    /// Returns the created/updated cache entry.
    ///
    /// # Arguments
    ///
    /// * `new_entry` - The new cache entry to insert
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use lib::audio::cache::{AudioCache, NewAudioCacheEntry};
    /// # use lib::audio::types::{AudioSource, AudioFormat, AudioMetadata};
    /// # use std::path::PathBuf;
    /// # use surrealdb::Surreal;
    /// # use surrealdb::engine::local::Mem;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let db = Surreal::new::<Mem>(()).await?;
    /// # let cache = AudioCache::new(db);
    /// let new_entry = NewAudioCacheEntry {
    ///     resource_hash: "abc123".to_string(),
    ///     content_hash: "def456".to_string(),
    ///     source: AudioSource::Local(PathBuf::from("audio.mp3")),
    ///     format: AudioFormat::Mp3,
    ///     metadata: AudioMetadata {
    ///         duration_secs: Some(180.5),
    ///         bitrate: Some(320000),
    ///         sample_rate: Some(44100),
    ///         channels: Some(2),
    ///         ..Default::default()
    ///     },
    /// };
    /// let entry = cache.upsert(new_entry).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, new_entry))]
    pub async fn upsert(&self, new_entry: NewAudioCacheEntry) -> Result<AudioCacheEntry> {
        debug!("Upserting audio cache entry for resource_hash: {}", new_entry.resource_hash);

        let entry: AudioCacheEntry = new_entry.into();
        let internal: AudioCacheEntryInternal = entry.clone().into();

        // Delete existing entry with same resource_hash to ensure upsert behavior
        self.db
            .query("DELETE FROM audio_cache WHERE resource_hash = $resource_hash")
            .bind(("resource_hash", &internal.resource_hash))
            .await
            .map_err(|e| CacheError::QueryFailed(format!("Failed to delete existing entry: {}", e)))?;

        // Create new entry
        let _created: Vec<AudioCacheEntryInternal> = self
            .db
            .create("audio_cache")
            .content(internal)
            .await
            .map_err(|e| CacheError::QueryFailed(format!("Failed to create entry: {}", e)))?;

        info!("Upserted audio cache entry for resource_hash: {}", entry.resource_hash);

        Ok(entry)
    }

    /// Clear all audio cache entries
    ///
    /// This deletes all entries from the audio_cache table.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use lib::audio::cache::AudioCache;
    /// # use surrealdb::Surreal;
    /// # use surrealdb::engine::local::Mem;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let db = Surreal::new::<Mem>(()).await?;
    /// # let cache = AudioCache::new(db);
    /// cache.clear().await?;
    /// println!("All audio cache entries cleared");
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self))]
    pub async fn clear(&self) -> Result<()> {
        info!("Clearing all audio cache entries");

        self.db
            .query("DELETE FROM audio_cache")
            .await
            .map_err(|e| CacheError::QueryFailed(format!("Failed to clear audio cache: {}", e)))?;

        info!("Audio cache cleared successfully");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use surrealdb::engine::local::Mem;

    async fn setup_test_db() -> Surreal<Db> {
        let db = Surreal::new::<Mem>(()).await.unwrap();
        db.use_ns("test").use_db("test").await.unwrap();

        // Apply schema
        crate::cache::schema::apply_schema(&db).await.unwrap();

        db
    }

    #[tokio::test]
    async fn test_cache_new() {
        let db = setup_test_db().await;
        let _cache = AudioCache::new(db);
        // Success if no panic
    }

    #[tokio::test]
    async fn test_cache_miss() {
        let db = setup_test_db().await;
        let cache = AudioCache::new(db);

        let result = cache.get("nonexistent_resource", "nonexistent_content").await.unwrap();
        assert!(result.is_none(), "Expected cache miss for nonexistent entry");
    }

    #[tokio::test]
    async fn test_cache_hit() {
        let db = setup_test_db().await;
        let cache = AudioCache::new(db);

        // Insert an entry
        let new_entry = NewAudioCacheEntry {
            resource_hash: "test_resource_123".to_string(),
            content_hash: "test_content_456".to_string(),
            source: AudioSource::Local(PathBuf::from("test.mp3")),
            format: AudioFormat::Mp3,
            metadata: AudioMetadata {
                duration_secs: Some(180.5),
                bitrate: Some(320000),
                sample_rate: Some(44100),
                channels: Some(2),
                title: None,
                artist: None,
                album: None,
            },
        };

        cache.upsert(new_entry).await.unwrap();

        // Retrieve the entry
        let result = cache.get("test_resource_123", "test_content_456").await.unwrap();
        assert!(result.is_some(), "Expected cache hit");

        let entry = result.unwrap();
        assert_eq!(entry.resource_hash, "test_resource_123");
        assert_eq!(entry.content_hash, "test_content_456");
        assert_eq!(entry.format, AudioFormat::Mp3);
        assert_eq!(entry.metadata.duration_secs, Some(180.5));
        assert_eq!(entry.metadata.bitrate, Some(320000));
        assert_eq!(entry.metadata.sample_rate, Some(44100));
        assert_eq!(entry.metadata.channels, Some(2));
    }

    #[tokio::test]
    async fn test_upsert_creates_new_entry() {
        let db = setup_test_db().await;
        let cache = AudioCache::new(db);

        let new_entry = NewAudioCacheEntry {
            resource_hash: "new_resource".to_string(),
            content_hash: "new_content".to_string(),
            source: AudioSource::Local(PathBuf::from("new.wav")),
            format: AudioFormat::Wav,
            metadata: AudioMetadata {
                duration_secs: Some(60.0),
                bitrate: Some(1411200),
                sample_rate: Some(44100),
                channels: Some(2),
                title: None,
                artist: None,
                album: None,
            },
        };

        let entry = cache.upsert(new_entry).await.unwrap();
        assert_eq!(entry.resource_hash, "new_resource");
        assert_eq!(entry.format, AudioFormat::Wav);

        // Verify it's in the cache
        let result = cache.get("new_resource", "new_content").await.unwrap();
        assert!(result.is_some());
    }

    #[tokio::test]
    async fn test_upsert_updates_existing_entry() {
        let db = setup_test_db().await;
        let cache = AudioCache::new(db);

        // Insert initial entry
        let entry1 = NewAudioCacheEntry {
            resource_hash: "update_test".to_string(),
            content_hash: "content_v1".to_string(),
            source: AudioSource::Local(PathBuf::from("audio.mp3")),
            format: AudioFormat::Mp3,
            metadata: AudioMetadata {
                duration_secs: Some(100.0),
                bitrate: Some(128000),
                sample_rate: Some(44100),
                channels: Some(2),
                title: None,
                artist: None,
                album: None,
            },
        };
        cache.upsert(entry1).await.unwrap();

        // Update with new content hash
        let entry2 = NewAudioCacheEntry {
            resource_hash: "update_test".to_string(),
            content_hash: "content_v2".to_string(),
            source: AudioSource::Local(PathBuf::from("audio.mp3")),
            format: AudioFormat::Mp3,
            metadata: AudioMetadata {
                duration_secs: Some(120.0),
                bitrate: Some(256000),
                sample_rate: Some(48000),
                channels: Some(2),
                title: None,
                artist: None,
                album: None,
            },
        };
        cache.upsert(entry2).await.unwrap();

        // Old content hash should not exist
        let old_result = cache.get("update_test", "content_v1").await.unwrap();
        assert!(old_result.is_none(), "Old entry should be replaced");

        // New content hash should exist
        let new_result = cache.get("update_test", "content_v2").await.unwrap();
        assert!(new_result.is_some(), "New entry should exist");
        let entry = new_result.unwrap();
        assert_eq!(entry.metadata.duration_secs, Some(120.0));
        assert_eq!(entry.metadata.bitrate, Some(256000));
    }

    #[tokio::test]
    async fn test_clear() {
        let db = setup_test_db().await;
        let cache = AudioCache::new(db);

        // Insert multiple entries
        for i in 0..3 {
            let entry = NewAudioCacheEntry {
                resource_hash: format!("resource_{}", i),
                content_hash: format!("content_{}", i),
                source: AudioSource::Local(PathBuf::from(format!("audio_{}.mp3", i))),
                format: AudioFormat::Mp3,
                metadata: AudioMetadata::default(),
            };
            cache.upsert(entry).await.unwrap();
        }

        // Verify entries exist
        let result = cache.get("resource_0", "content_0").await.unwrap();
        assert!(result.is_some());

        // Clear all entries
        cache.clear().await.unwrap();

        // Verify all entries are gone
        for i in 0..3 {
            let result = cache.get(&format!("resource_{}", i), &format!("content_{}", i)).await.unwrap();
            assert!(result.is_none(), "Entry {} should be cleared", i);
        }
    }

    #[tokio::test]
    async fn test_remote_source() {
        let db = setup_test_db().await;
        let cache = AudioCache::new(db);

        let new_entry = NewAudioCacheEntry {
            resource_hash: "remote_resource".to_string(),
            content_hash: "remote_content".to_string(),
            source: AudioSource::Remote("https://example.com/audio.mp3".to_string()),
            format: AudioFormat::Mp3,
            metadata: AudioMetadata::default(),
        };

        let entry = cache.upsert(new_entry).await.unwrap();
        assert_eq!(entry.source_type, "remote");
        assert_eq!(entry.source, "https://example.com/audio.mp3");
    }
}
