use crate::ai::traits::EmbeddingModel;
use crate::error::{AIError, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use surrealdb::engine::local::Db;
use surrealdb::sql::Datetime as SurrealDatetime;
use surrealdb::Surreal;
use tracing::{debug, instrument};
use xxhash_rust::xxh3::xxh3_64;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EmbeddingEntryInternal {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<surrealdb::sql::Thing>,
    pub resource_hash: String,
    pub content_hash: String,
    pub model: String,
    pub vector: Vec<f32>,
    pub created_at: SurrealDatetime,
}

#[derive(Debug, Clone)]
pub struct EmbeddingEntry {
    pub id: Option<surrealdb::sql::Thing>,
    pub resource_hash: String,
    pub content_hash: String,
    pub model: String,
    pub vector: Vec<f32>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<EmbeddingEntryInternal> for EmbeddingEntry {
    fn from(internal: EmbeddingEntryInternal) -> Self {
        Self {
            id: internal.id,
            resource_hash: internal.resource_hash,
            content_hash: internal.content_hash,
            model: internal.model,
            vector: internal.vector,
            created_at: internal.created_at.0,
        }
    }
}

impl From<EmbeddingEntry> for EmbeddingEntryInternal {
    fn from(entry: EmbeddingEntry) -> Self {
        Self {
            id: entry.id,
            resource_hash: entry.resource_hash,
            content_hash: entry.content_hash,
            model: entry.model,
            vector: entry.vector,
            created_at: SurrealDatetime::from(entry.created_at),
        }
    }
}

#[instrument(skip(db, model, text))]
pub async fn generate_embedding(
    db: Arc<Surreal<Db>>,
    model: Arc<dyn EmbeddingModel>,
    resource_hash: &str,
    text: &str,
) -> Result<Vec<f32>> {
    let content_hash = format!("{:x}", xxh3_64(text.as_bytes()));
    let model_name = model.model_name();

    debug!(
        "Generating embedding for resource {} (content hash: {}, model: {})",
        resource_hash, content_hash, model_name
    );

    if let Some(existing) = get_embedding(&db, resource_hash, &content_hash, model_name).await? {
        debug!("Using existing embedding");
        return Ok(existing.vector);
    }

    debug!("Generating new embedding");

    let embeddings = model
        .embed(&[text.to_string()])
        .await
        .map_err(|e| AIError::EmbeddingFailed(e.to_string()))?;

    if embeddings.is_empty() {
        return Err(
            AIError::EmbeddingFailed("No embeddings returned from model".to_string()).into(),
        );
    }

    let vector = embeddings[0].clone();

    let expected_dims = model.dimensions();
    if vector.len() != expected_dims {
        return Err(AIError::EmbeddingFailed(format!(
            "Embedding dimension mismatch: expected {}, got {}",
            expected_dims,
            vector.len()
        ))
        .into());
    }

    let entry = EmbeddingEntry {
        id: None,
        resource_hash: resource_hash.to_string(),
        content_hash,
        model: model_name.to_string(),
        vector: vector.clone(),
        created_at: Utc::now(),
    };

    store_embedding(&db, entry).await?;

    Ok(vector)
}

#[instrument(skip(db))]
async fn get_embedding(
    db: &Surreal<Db>,
    resource_hash: &str,
    content_hash: &str,
    model: &str,
) -> Result<Option<EmbeddingEntry>> {
    let mut result = db
        .query(
            r#"
            SELECT * FROM embedding
            WHERE resource_hash = $resource_hash
            AND content_hash = $content_hash
            AND model = $model
            "#,
        )
        .bind(("resource_hash", resource_hash))
        .bind(("content_hash", content_hash))
        .bind(("model", model))
        .await
        .map_err(|e| AIError::EmbeddingFailed(e.to_string()))?;

    let entry: Option<EmbeddingEntryInternal> = result
        .take(0)
        .map_err(|e| AIError::EmbeddingFailed(e.to_string()))?;

    Ok(entry.map(EmbeddingEntry::from))
}

#[instrument(skip(db, entry))]
async fn store_embedding(db: &Surreal<Db>, entry: EmbeddingEntry) -> Result<()> {
    debug!("Storing embedding for resource {}", entry.resource_hash);

    let internal: EmbeddingEntryInternal = entry.into();
    let _created: Vec<EmbeddingEntryInternal> = db
        .create("embedding")
        .content(internal)
        .await
        .map_err(|e| AIError::EmbeddingFailed(e.to_string()))?;

    Ok(())
}

#[instrument(skip(db, query_vector))]
pub async fn find_similar(
    db: Arc<Surreal<Db>>,
    query_vector: &[f32],
    limit: usize,
    model: Option<&str>,
) -> Result<Vec<(EmbeddingEntry, f32)>> {
    debug!("Searching for similar embeddings (limit: {})", limit);

    let model_filter = if let Some(m) = model {
        format!("AND model = '{}'", m)
    } else {
        String::new()
    };

    let mut result = db
        .query(format!(
            r#"
            SELECT *, vector::similarity::cosine(vector, $query) AS score
            FROM embedding
            WHERE 1=1 {}
            ORDER BY score DESC
            LIMIT $limit
            "#,
            model_filter
        ))
        .bind(("query", query_vector))
        .bind(("limit", limit))
        .await
        .map_err(|e| AIError::EmbeddingFailed(e.to_string()))?;

    #[derive(Deserialize)]
    struct ScoredEntry {
        #[serde(flatten)]
        entry: EmbeddingEntryInternal,
        score: f32,
    }

    let scored_entries: Vec<ScoredEntry> = result
        .take(0)
        .map_err(|e| AIError::EmbeddingFailed(e.to_string()))?;

    let results = scored_entries
        .into_iter()
        .map(|se| (EmbeddingEntry::from(se.entry), se.score))
        .collect();

    Ok(results)
}
