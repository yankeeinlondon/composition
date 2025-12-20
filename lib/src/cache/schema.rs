use surrealdb::engine::local::Db;
use surrealdb::Surreal;
use crate::error::Result;
use tracing::{info, instrument};

/// SQL schema definitions for the database
pub const SCHEMA_SQL: &str = r#"
-- Document node
DEFINE TABLE document SCHEMAFULL;
DEFINE FIELD resource_hash ON document TYPE string;
DEFINE FIELD content_hash ON document TYPE string;
DEFINE FIELD file_path ON document TYPE option<string>;
DEFINE FIELD url ON document TYPE option<string>;
DEFINE FIELD last_validated ON document TYPE datetime;
DEFINE INDEX idx_resource_hash ON document FIELDS resource_hash UNIQUE;

-- Dependency edge (using SurrealDB graph relations)
DEFINE TABLE depends_on SCHEMAFULL;
DEFINE FIELD in ON depends_on TYPE record<document>;
DEFINE FIELD out ON depends_on TYPE record<document>;
DEFINE FIELD reference_type ON depends_on TYPE string;
DEFINE FIELD required ON depends_on TYPE bool DEFAULT false;

-- Image cache
DEFINE TABLE image_cache SCHEMAFULL;
DEFINE FIELD resource_hash ON image_cache TYPE string;
DEFINE FIELD content_hash ON image_cache TYPE string;
DEFINE FIELD created_at ON image_cache TYPE datetime DEFAULT time::now();
DEFINE FIELD expires_at ON image_cache TYPE option<datetime>;
DEFINE FIELD source_type ON image_cache TYPE string;
DEFINE FIELD source ON image_cache TYPE string;
DEFINE FIELD has_transparency ON image_cache TYPE bool;
DEFINE FIELD original_width ON image_cache TYPE int;
DEFINE FIELD original_height ON image_cache TYPE int;
DEFINE INDEX idx_image_resource ON image_cache FIELDS resource_hash UNIQUE;
DEFINE INDEX idx_image_lookup ON image_cache FIELDS resource_hash, content_hash;

-- LLM cache
DEFINE TABLE llm_cache SCHEMAFULL;
DEFINE FIELD operation ON llm_cache TYPE string;
DEFINE FIELD input_hash ON llm_cache TYPE string;
DEFINE FIELD model ON llm_cache TYPE string;
DEFINE FIELD response ON llm_cache TYPE string;
DEFINE FIELD created_at ON llm_cache TYPE datetime DEFAULT time::now();
DEFINE FIELD expires_at ON llm_cache TYPE datetime;
DEFINE FIELD tokens_used ON llm_cache TYPE option<int>;
DEFINE INDEX idx_llm_lookup ON llm_cache FIELDS operation, input_hash, model;
DEFINE INDEX idx_llm_expires ON llm_cache FIELDS expires_at;

-- Vector embedding (HNSW index syntax for Phase 6 - may need SurrealDB 2.x)
DEFINE TABLE embedding SCHEMAFULL;
DEFINE FIELD resource_hash ON embedding TYPE string;
DEFINE FIELD content_hash ON embedding TYPE string;
DEFINE FIELD model ON embedding TYPE string;
DEFINE FIELD vector ON embedding TYPE array<float>;
DEFINE FIELD created_at ON embedding TYPE datetime DEFAULT time::now();
-- Note: HNSW vector index syntax varies by SurrealDB version
-- DEFINE INDEX idx_embedding_vector ON embedding FIELDS vector HNSW DIMENSION 1536 DISTANCE COSINE;
DEFINE INDEX idx_embedding_resource ON embedding FIELDS resource_hash UNIQUE;
"#;

/// Apply the database schema
#[instrument(skip(db))]
pub async fn apply_schema(db: &Surreal<Db>) -> Result<()> {
    info!("Applying database schema");

    // Execute schema definition
    db.query(SCHEMA_SQL)
        .await
        .map_err(|e| crate::error::CacheError::QueryFailed(format!("Schema application failed: {}", e)))?;

    info!("Schema applied successfully");
    Ok(())
}
