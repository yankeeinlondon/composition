# Embedding Strategy for Composition Library

**Date:** 2025-12-19
**Phase:** 0.3 - Validation Spike
**Status:** Decided

## Executive Summary

The Composition library will standardize on **1536-dimensional embeddings** with a **single HNSW vector index** in SurrealDB. This decision ensures compatibility with industry-standard embedding models while avoiding the complexity of managing multiple indexes.

## Decision

### Embedding Dimension: 1536

**Rationale:**
1. **Industry Standard:** OpenAI's `text-embedding-3-small` and `text-embedding-3-large` models use 1536 dimensions
2. **Cloud Provider Compatibility:** Most major cloud embedding APIs support 1536 dimensions
3. **Performance:** 1536 provides excellent semantic search quality while remaining computationally efficient
4. **Future-Proof:** Compatible with most current and upcoming embedding models

### Index Strategy: Single HNSW Index

**Rationale:**
1. **Simplicity:** One index is easier to maintain and reason about
2. **Storage Efficiency:** Avoids duplication of vector data across multiple indexes
3. **Query Performance:** Single lookup path reduces complexity
4. **Model Flexibility:** Users can choose any 1536-dim model via configuration

**Trade-offs Considered:**
- ✅ Simpler codebase and schema
- ✅ Easier to test and debug
- ⚠️ Requires all embedding models to output 1536 dimensions (acceptable constraint)

## Recommended Embedding Models

### Cloud Models (via rig-core)

1. **OpenAI** (Default)
   - Model: `text-embedding-3-small` or `text-embedding-3-large`
   - Dimensions: 1536
   - Quality: Excellent
   - Cost: Moderate

2. **Anthropic (via OpenAI-compatible API)**
   - Model: `voyage-2` (via Voyage AI)
   - Dimensions: 1536
   - Quality: Excellent
   - Cost: Moderate

3. **Cohere**
   - Model: `embed-english-v3.0`
   - Dimensions: 1536 (can be configured)
   - Quality: Excellent
   - Cost: Low

### Local Models (via fastembed or rig-fastembed)

For users who want offline/private embeddings:

1. **BGE-Large-EN-v1.5** (Recommended)
   - Dimensions: 1024 (requires padding to 1536 or separate workflow)
   - Quality: Very Good
   - Speed: Fast
   - License: MIT

2. **Alternative Approach:** Use cloud models with caching
   - Cache embeddings aggressively in SurrealDB
   - Reduces API calls after initial computation
   - Better quality than local models

## Implementation Notes

### SurrealDB Schema

```sql
DEFINE TABLE embedding SCHEMAFULL;
DEFINE FIELD resource_hash ON embedding TYPE string;
DEFINE FIELD content_hash ON embedding TYPE string;
DEFINE FIELD model ON embedding TYPE string;
DEFINE FIELD vector ON embedding TYPE array<float>;
DEFINE FIELD created_at ON embedding TYPE datetime DEFAULT time::now();

-- HNSW vector index for 1536 dimensions
DEFINE INDEX idx_embedding_vector ON embedding
    FIELDS vector HNSW DIMENSION 1536 DISTANCE COSINE;

DEFINE INDEX idx_embedding_resource ON embedding FIELDS resource_hash UNIQUE;
```

### Embedding Generation API

```rust
pub async fn generate_embedding(
    &self,
    content: &str,
    model: Option<&str>,
) -> Result<Vec<f32>> {
    let model = model.unwrap_or("text-embedding-3-small");

    // Check cache first
    let content_hash = compute_content_hash(content);
    if let Some(cached) = self.get_cached_embedding(&content_hash, model).await? {
        return Ok(cached.vector);
    }

    // Generate new embedding
    let embedding = self.embedding_provider
        .embed(content, model)
        .await?;

    // Verify dimension
    assert_eq!(embedding.len(), 1536, "Embedding must be 1536 dimensions");

    // Cache result
    self.cache_embedding(&content_hash, model, &embedding).await?;

    Ok(embedding)
}
```

### Handling Non-1536 Models

For models that don't natively output 1536 dimensions:

**Option 1: Zero-Padding** (Not Recommended)
- Pad shorter vectors to 1536 with zeros
- ⚠️ Reduces semantic quality

**Option 2: Projection** (Better)
- Use dimensionality reduction/expansion techniques
- ⚠️ Adds complexity

**Option 3: Separate Workflow** (Recommended)
- Use cloud models (1536) for semantic search
- Use local models for other tasks (if needed)
- Keep embeddings in separate table if required

## Configuration

Users can configure embedding models via frontmatter:

```yaml
---
embedding_model: "text-embedding-3-large"
embedding_provider: "openai"
---
```

## Testing Strategy

### Validation Test (Phase 0.3)

```rust
#[tokio::test]
async fn test_embedding_storage_and_retrieval() {
    let db = setup_test_db().await;

    // Generate test embedding
    let test_vector: Vec<f32> = (0..1536)
        .map(|i| (i as f32) / 1536.0)
        .collect();

    // Store embedding
    db.query("INSERT INTO embedding {
        resource_hash: 'test_hash',
        content_hash: 'content_hash',
        model: 'text-embedding-3-small',
        vector: $vector
    }")
    .bind(("vector", &test_vector))
    .await?;

    // Verify retrieval
    let result: Vec<Embedding> = db.select("embedding").await?;
    assert_eq!(result[0].vector.len(), 1536);

    // Test vector search
    let similar = db.query("
        SELECT *, vector::similarity::cosine(vector, $query_vector) AS similarity
        FROM embedding
        ORDER BY similarity DESC
        LIMIT 10
    ")
    .bind(("query_vector", &test_vector))
    .await?;

    // Should find itself as most similar
    assert!(similar[0].similarity > 0.99);
}
```

## Migration Path

If we need to support multiple dimensions in the future:

1. Add `dimension` field to embedding table
2. Create dimension-specific indexes: `idx_embedding_1536`, `idx_embedding_384`, etc.
3. Route queries to appropriate index based on model dimension
4. Maintain 1536 as default/recommended

This keeps the migration path open while standardizing on 1536 for now.

## Conclusion

**Decision:** Use 1536-dimensional embeddings with a single HNSW index in SurrealDB.

**Next Steps:**
1. ✅ Document decision (this file)
2. Implement embedding storage/retrieval in Phase 6
3. Add embedding generation tests
4. Integrate with rig-core providers

**Success Criteria:**
- ✅ Single, clear embedding dimension standard
- ✅ Compatible with major cloud providers
- ✅ Simple schema and implementation
- ✅ Clear migration path if needed
