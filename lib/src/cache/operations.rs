use crate::error::{CacheError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb::engine::local::Db;
use surrealdb::sql::Datetime as SurrealDatetime;
use surrealdb::Surreal;
use tracing::{debug, instrument};

/// Convert chrono DateTime to SurrealDB Datetime
fn to_surreal_datetime(dt: DateTime<Utc>) -> SurrealDatetime {
    SurrealDatetime::from(dt)
}

/// Convert SurrealDB Datetime to chrono DateTime
fn from_surreal_datetime(dt: &SurrealDatetime) -> DateTime<Utc> {
    DateTime::from(dt.0)
}

/// Document cache entry (internal representation using SurrealDB types)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DocumentCacheEntryInternal {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<surrealdb::sql::Thing>,
    pub resource_hash: String,
    pub content_hash: String,
    pub file_path: Option<String>,
    pub url: Option<String>,
    pub last_validated: SurrealDatetime,
}

/// Document cache entry (public API using chrono types)
#[derive(Debug, Clone)]
pub struct DocumentCacheEntry {
    pub id: Option<surrealdb::sql::Thing>,
    pub resource_hash: String,
    pub content_hash: String,
    pub file_path: Option<String>,
    pub url: Option<String>,
    pub last_validated: DateTime<Utc>,
}

impl From<DocumentCacheEntryInternal> for DocumentCacheEntry {
    fn from(internal: DocumentCacheEntryInternal) -> Self {
        Self {
            id: internal.id,
            resource_hash: internal.resource_hash,
            content_hash: internal.content_hash,
            file_path: internal.file_path,
            url: internal.url,
            last_validated: from_surreal_datetime(&internal.last_validated),
        }
    }
}

impl From<DocumentCacheEntry> for DocumentCacheEntryInternal {
    fn from(entry: DocumentCacheEntry) -> Self {
        Self {
            id: entry.id,
            resource_hash: entry.resource_hash,
            content_hash: entry.content_hash,
            file_path: entry.file_path,
            url: entry.url,
            last_validated: to_surreal_datetime(entry.last_validated),
        }
    }
}

/// Image cache entry (internal representation)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ImageCacheEntryInternal {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<surrealdb::sql::Thing>,
    pub resource_hash: String,
    pub content_hash: String,
    pub created_at: SurrealDatetime,
    pub expires_at: Option<SurrealDatetime>,
    pub source_type: String,
    pub source: String,
    pub has_transparency: bool,
    pub original_width: i64,
    pub original_height: i64,
}

/// Image cache entry (public API)
#[derive(Debug, Clone)]
pub struct ImageCacheEntry {
    pub id: Option<surrealdb::sql::Thing>,
    pub resource_hash: String,
    pub content_hash: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub source_type: String,
    pub source: String,
    pub has_transparency: bool,
    pub original_width: i64,
    pub original_height: i64,
}

impl From<ImageCacheEntryInternal> for ImageCacheEntry {
    fn from(internal: ImageCacheEntryInternal) -> Self {
        Self {
            id: internal.id,
            resource_hash: internal.resource_hash,
            content_hash: internal.content_hash,
            created_at: from_surreal_datetime(&internal.created_at),
            expires_at: internal.expires_at.as_ref().map(from_surreal_datetime),
            source_type: internal.source_type,
            source: internal.source,
            has_transparency: internal.has_transparency,
            original_width: internal.original_width,
            original_height: internal.original_height,
        }
    }
}

impl From<ImageCacheEntry> for ImageCacheEntryInternal {
    fn from(entry: ImageCacheEntry) -> Self {
        Self {
            id: entry.id,
            resource_hash: entry.resource_hash,
            content_hash: entry.content_hash,
            created_at: to_surreal_datetime(entry.created_at),
            expires_at: entry.expires_at.map(to_surreal_datetime),
            source_type: entry.source_type,
            source: entry.source,
            has_transparency: entry.has_transparency,
            original_width: entry.original_width,
            original_height: entry.original_height,
        }
    }
}

/// LLM cache entry (internal representation)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct LlmCacheEntryInternal {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<surrealdb::sql::Thing>,
    pub operation: String,
    pub input_hash: String,
    pub model: String,
    pub response: String,
    pub created_at: SurrealDatetime,
    pub expires_at: SurrealDatetime,
    pub tokens_used: Option<i64>,
}

/// LLM cache entry (public API)
#[derive(Debug, Clone)]
pub struct LlmCacheEntry {
    pub id: Option<surrealdb::sql::Thing>,
    pub operation: String,
    pub input_hash: String,
    pub model: String,
    pub response: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub tokens_used: Option<i64>,
}

impl From<LlmCacheEntryInternal> for LlmCacheEntry {
    fn from(internal: LlmCacheEntryInternal) -> Self {
        Self {
            id: internal.id,
            operation: internal.operation,
            input_hash: internal.input_hash,
            model: internal.model,
            response: internal.response,
            created_at: from_surreal_datetime(&internal.created_at),
            expires_at: from_surreal_datetime(&internal.expires_at),
            tokens_used: internal.tokens_used,
        }
    }
}

impl From<LlmCacheEntry> for LlmCacheEntryInternal {
    fn from(entry: LlmCacheEntry) -> Self {
        Self {
            id: entry.id,
            operation: entry.operation,
            input_hash: entry.input_hash,
            model: entry.model,
            response: entry.response,
            created_at: to_surreal_datetime(entry.created_at),
            expires_at: to_surreal_datetime(entry.expires_at),
            tokens_used: entry.tokens_used,
        }
    }
}

/// Cache operations trait for different cache types
pub struct CacheOperations {
    db: Surreal<Db>,
}

impl CacheOperations {
    pub fn new(db: Surreal<Db>) -> Self {
        Self { db }
    }

    /// Get a document cache entry by resource hash
    #[instrument(skip(self))]
    pub async fn get_document(&self, resource_hash: &str) -> Result<Option<DocumentCacheEntry>> {
        debug!("Getting document cache entry for hash: {}", resource_hash);

        let mut result = self
            .db
            .query("SELECT * FROM document WHERE resource_hash = $hash")
            .bind(("hash", resource_hash))
            .await
            .map_err(|e| CacheError::QueryFailed(e.to_string()))?;

        let entry: Option<DocumentCacheEntryInternal> = result
            .take(0)
            .map_err(|e| CacheError::DeserializationError(e.to_string()))?;

        Ok(entry.map(DocumentCacheEntry::from))
    }

    /// Upsert a document cache entry
    #[instrument(skip(self, entry))]
    pub async fn upsert_document(&self, entry: DocumentCacheEntry) -> Result<()> {
        debug!("Upserting document cache entry for hash: {}", entry.resource_hash);

        let internal: DocumentCacheEntryInternal = entry.into();
        let _created: Vec<DocumentCacheEntryInternal> = self.db
            .create("document")
            .content(internal)
            .await
            .map_err(|e| CacheError::QueryFailed(e.to_string()))?;

        Ok(())
    }

    /// Get an image cache entry by resource hash
    #[instrument(skip(self))]
    pub async fn get_image(&self, resource_hash: &str) -> Result<Option<ImageCacheEntry>> {
        debug!("Getting image cache entry for hash: {}", resource_hash);

        let mut result = self
            .db
            .query("SELECT * FROM image_cache WHERE resource_hash = $hash")
            .bind(("hash", resource_hash))
            .await
            .map_err(|e| CacheError::QueryFailed(e.to_string()))?;

        let entry: Option<ImageCacheEntryInternal> = result
            .take(0)
            .map_err(|e| CacheError::DeserializationError(e.to_string()))?;

        Ok(entry.map(ImageCacheEntry::from))
    }

    /// Upsert an image cache entry
    #[instrument(skip(self, entry))]
    pub async fn upsert_image(&self, entry: ImageCacheEntry) -> Result<()> {
        debug!("Upserting image cache entry for hash: {}", entry.resource_hash);

        let internal: ImageCacheEntryInternal = entry.into();
        let _created: Vec<ImageCacheEntryInternal> = self.db
            .create("image_cache")
            .content(internal)
            .await
            .map_err(|e| CacheError::QueryFailed(e.to_string()))?;

        Ok(())
    }

    /// Get an LLM cache entry
    #[instrument(skip(self))]
    pub async fn get_llm(
        &self,
        operation: &str,
        input_hash: &str,
        model: &str,
    ) -> Result<Option<LlmCacheEntry>> {
        debug!("Getting LLM cache entry for operation: {}, model: {}", operation, model);

        let mut result = self
            .db
            .query(
                r#"
                SELECT * FROM llm_cache
                WHERE operation = $operation
                AND input_hash = $input_hash
                AND model = $model
                AND expires_at > $now
                "#,
            )
            .bind(("operation", operation))
            .bind(("input_hash", input_hash))
            .bind(("model", model))
            .bind(("now", to_surreal_datetime(Utc::now())))
            .await
            .map_err(|e| CacheError::QueryFailed(e.to_string()))?;

        let entry: Option<LlmCacheEntryInternal> = result
            .take(0)
            .map_err(|e| CacheError::DeserializationError(e.to_string()))?;

        Ok(entry.map(LlmCacheEntry::from))
    }

    /// Upsert an LLM cache entry
    #[instrument(skip(self, entry))]
    pub async fn upsert_llm(&self, entry: LlmCacheEntry) -> Result<()> {
        debug!("Upserting LLM cache entry for operation: {}", entry.operation);

        let internal: LlmCacheEntryInternal = entry.into();
        let _created: Vec<LlmCacheEntryInternal> = self.db
            .create("llm_cache")
            .content(internal)
            .await
            .map_err(|e| CacheError::QueryFailed(e.to_string()))?;

        Ok(())
    }

    /// Invalidate a document and cascade to dependents
    #[instrument(skip(self))]
    pub async fn invalidate_document_cascade(&self, resource_hash: &str) -> Result<Vec<String>> {
        debug!("Invalidating document cascade for hash: {}", resource_hash);

        // Find all documents that depend on this one (transitively)
        let mut result = self
            .db
            .query(
                r#"
                SELECT resource_hash FROM (
                    SELECT ->depends_on->document AS dependents
                    FROM document
                    WHERE resource_hash = $hash
                ).dependents.*
                "#,
            )
            .bind(("hash", resource_hash))
            .await
            .map_err(|e| CacheError::QueryFailed(e.to_string()))?;

        #[derive(Deserialize)]
        struct HashResult {
            resource_hash: String,
        }

        let dependents: Vec<HashResult> = result
            .take(0)
            .map_err(|e| CacheError::DeserializationError(e.to_string()))?;

        let invalidated_hashes: Vec<String> = dependents
            .into_iter()
            .map(|h| h.resource_hash)
            .collect();

        // Delete the document and its dependents
        self.db
            .query("DELETE FROM document WHERE resource_hash = $hash")
            .bind(("hash", resource_hash))
            .await
            .map_err(|e| CacheError::InvalidationFailed(e.to_string()))?;

        for dep_hash in &invalidated_hashes {
            self.db
                .query("DELETE FROM document WHERE resource_hash = $hash")
                .bind(("hash", dep_hash))
                .await
                .map_err(|e| CacheError::InvalidationFailed(e.to_string()))?;
        }

        Ok(invalidated_hashes)
    }

    /// Invalidate an image cache entry
    #[instrument(skip(self))]
    pub async fn invalidate_image(&self, resource_hash: &str) -> Result<()> {
        debug!("Invalidating image cache entry for hash: {}", resource_hash);

        self.db
            .query("DELETE FROM image_cache WHERE resource_hash = $hash")
            .bind(("hash", resource_hash))
            .await
            .map_err(|e| CacheError::InvalidationFailed(e.to_string()))?;

        Ok(())
    }

    /// Clean expired LLM cache entries
    #[instrument(skip(self))]
    pub async fn clean_expired_llm_cache(&self) -> Result<usize> {
        debug!("Cleaning expired LLM cache entries");

        let mut result = self
            .db
            .query("DELETE FROM llm_cache WHERE expires_at < $now RETURN BEFORE")
            .bind(("now", to_surreal_datetime(Utc::now())))
            .await
            .map_err(|e| CacheError::QueryFailed(e.to_string()))?;

        // Count deleted entries
        let deleted: Vec<LlmCacheEntryInternal> = result
            .take(0)
            .unwrap_or_default();

        Ok(deleted.len())
    }
}

/// Legacy wrapper function for get_image_cache (to be removed after refactoring)
pub async fn get_image_cache(
    db: &Surreal<Db>,
    resource_hash: &str,
    _content_hash: &str,
) -> Result<Option<ImageCacheEntry>> {
    let ops = CacheOperations::new(db.clone());
    ops.get_image(resource_hash).await
}

/// Legacy wrapper function for upsert_image_cache (to be removed after refactoring)
#[allow(clippy::too_many_arguments)]
pub async fn upsert_image_cache(
    db: &Surreal<Db>,
    resource_hash: &str,
    content_hash: &str,
    source_type: &str,
    source: &str,
    has_transparency: bool,
    original_width: u32,
    original_height: u32,
    expires_at: Option<DateTime<Utc>>,
) -> Result<()> {
    let ops = CacheOperations::new(db.clone());
    let entry = ImageCacheEntry {
        id: None,
        resource_hash: resource_hash.to_string(),
        content_hash: content_hash.to_string(),
        created_at: Utc::now(),
        expires_at,
        source_type: source_type.to_string(),
        source: source.to_string(),
        has_transparency,
        original_width: original_width as i64,
        original_height: original_height as i64,
    };
    ops.upsert_image(entry).await
}
