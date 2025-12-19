# Combining Solutions for Comprehensive Applications

Strategic combinations of Rust LLM libraries to build full-featured applications, including caching, SurrealDB integration, and production patterns.

## Model Configuration from Environment

Flexible model selection supporting environment variables and frontmatter overrides:

```rust
use std::env;

#[derive(Debug, Clone)]
pub struct ModelConfig {
    pub model_fast: String,
    pub model: String,
    pub model_strong: String,
    pub local_fallback: String,
    pub embeddings: String,
}

impl ModelConfig {
    pub fn from_env() -> Self {
        Self {
            model_fast: env::var("MODEL_FAST")
                .unwrap_or_else(|_| "openai/gpt-4o-mini".to_string()),
            model: env::var("MODEL")
                .unwrap_or_else(|_| "openai/gpt-4o".to_string()),
            model_strong: env::var("MODEL_STRONG")
                .unwrap_or_else(|_| "anthropic/claude-3-5-sonnet".to_string()),
            local_fallback: env::var("LOCAL_FALLBACK")
                .unwrap_or_else(|_| "ollama/llama3.2".to_string()),
            embeddings: env::var("EMBEDDINGS")
                .unwrap_or_else(|_| "openai/text-embedding-3-small".to_string()),
        }
    }

    /// Override with frontmatter values
    pub fn with_frontmatter(mut self, frontmatter: &serde_json::Value) -> Self {
        if let Some(v) = frontmatter.get("model_fast").and_then(|v| v.as_str()) {
            self.model_fast = v.to_string();
        }
        if let Some(v) = frontmatter.get("model").and_then(|v| v.as_str()) {
            self.model = v.to_string();
        }
        if let Some(v) = frontmatter.get("model_strong").and_then(|v| v.as_str()) {
            self.model_strong = v.to_string();
        }
        if let Some(v) = frontmatter.get("local_fallback").and_then(|v| v.as_str()) {
            self.local_fallback = v.to_string();
        }
        if let Some(v) = frontmatter.get("embeddings").and_then(|v| v.as_str()) {
            self.embeddings = v.to_string();
        }
        self
    }
}
```

## SurrealDB Integration

### Storing Embeddings

```rust
use surrealdb::{Surreal, engine::remote::ws::Ws};
use surrealdb::opt::auth::Root;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentRecord {
    pub id: Option<surrealdb::RecordId>,
    pub resource_hash: String,
    pub content_hash: String,
    pub content: String,
    pub embedding: Vec<f32>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub struct AICache {
    db: Surreal<surrealdb::engine::remote::ws::Client>,
}

impl AICache {
    pub async fn new(url: &str) -> Result<Self, anyhow::Error> {
        let db = Surreal::new::<Ws>(url).await?;
        db.signin(Root {
            username: "root",
            password: "secret",
        }).await?;
        db.use_ns("composition").use_db("cache").await?;

        // Create schema for embeddings
        db.query(r#"
            DEFINE TABLE document SCHEMAFULL;
            DEFINE FIELD resource_hash ON document TYPE string;
            DEFINE FIELD content_hash ON document TYPE string;
            DEFINE FIELD content ON document TYPE string;
            DEFINE FIELD embedding ON document TYPE array<float>;
            DEFINE FIELD created_at ON document TYPE datetime;

            DEFINE INDEX document_resource_hash ON document FIELDS resource_hash UNIQUE;
            DEFINE INDEX document_embedding_idx ON document
                FIELDS embedding HNSW DIMENSION 1536 DISTANCE COSINE;
        "#).await?;

        Ok(Self { db })
    }

    pub async fn store_document(
        &self,
        resource_hash: &str,
        content_hash: &str,
        content: &str,
        embedding: Vec<f32>,
    ) -> Result<(), anyhow::Error> {
        let doc = DocumentRecord {
            id: None,
            resource_hash: resource_hash.to_string(),
            content_hash: content_hash.to_string(),
            content: content.to_string(),
            embedding,
            created_at: chrono::Utc::now(),
        };

        self.db.create("document").content(doc).await?;
        Ok(())
    }

    pub async fn find_similar(
        &self,
        query_embedding: Vec<f32>,
        limit: u32,
    ) -> Result<Vec<DocumentRecord>, anyhow::Error> {
        let mut response = self.db.query(r#"
            SELECT
                id, resource_hash, content_hash, content,
                vector::distance::cosine(embedding, $embedding) AS distance
            FROM document
            WHERE embedding <|$limit, COSINE|> $embedding
            ORDER BY distance
        "#)
        .bind(("embedding", query_embedding))
        .bind(("limit", limit))
        .await?;

        let results: Vec<DocumentRecord> = response.take(0)?;
        Ok(results)
    }

    pub async fn is_cached(&self, resource_hash: &str) -> Result<bool, anyhow::Error> {
        let result: Option<DocumentRecord> = self.db
            .query("SELECT * FROM document WHERE resource_hash = $hash LIMIT 1")
            .bind(("hash", resource_hash))
            .await?
            .take(0)?;

        Ok(result.is_some())
    }
}
```

### Caching LLM Responses

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct LLMCacheEntry {
    pub id: Option<surrealdb::RecordId>,
    pub operation: String,  // "summarize", "consolidate", "topic"
    pub input_hash: String,
    pub model: String,
    pub response: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

impl AICache {
    pub async fn cache_llm_response(
        &self,
        operation: &str,
        input_hash: &str,
        model: &str,
        response: &str,
        ttl_days: i64,
    ) -> Result<(), anyhow::Error> {
        let now = chrono::Utc::now();
        let entry = LLMCacheEntry {
            id: None,
            operation: operation.to_string(),
            input_hash: input_hash.to_string(),
            model: model.to_string(),
            response: response.to_string(),
            created_at: now,
            expires_at: now + chrono::Duration::days(ttl_days),
        };

        self.db.create("llm_cache").content(entry).await?;
        Ok(())
    }

    pub async fn get_cached_response(
        &self,
        operation: &str,
        input_hash: &str,
        model: &str,
    ) -> Result<Option<String>, anyhow::Error> {
        let result: Option<LLMCacheEntry> = self.db
            .query(r#"
                SELECT * FROM llm_cache
                WHERE operation = $operation
                    AND input_hash = $hash
                    AND model = $model
                    AND expires_at > time::now()
                LIMIT 1
            "#)
            .bind(("operation", operation))
            .bind(("hash", input_hash))
            .bind(("model", model))
            .await?
            .take(0)?;

        Ok(result.map(|e| e.response))
    }
}
```

## Complete AI Service

Production-ready service combining all patterns:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AIError {
    #[error("Provider not available: {0}")]
    ProviderUnavailable(String),
    #[error("Model not found: {0}")]
    ModelNotFound(String),
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Cache error: {0}")]
    CacheError(#[from] surrealdb::Error),
}

pub struct CompositionAIService {
    config: ModelConfig,
    cache: AICache,
    embedder: HybridEmbedder,
}

impl CompositionAIService {
    pub async fn new(cache_url: &str) -> Result<Self, AIError> {
        let config = ModelConfig::from_env();
        let cache = AICache::new(cache_url).await
            .map_err(|e| AIError::CacheError(e.into()))?;
        let embedder = HybridEmbedder::new()
            .map_err(|e| AIError::ApiError(e.to_string()))?;

        Ok(Self { config, cache, embedder })
    }

    /// Summarize content, with caching
    pub async fn summarize(
        &self,
        content: &str,
        use_model: Option<&str>,
    ) -> Result<String, AIError> {
        let model = use_model.unwrap_or(&self.config.model);
        let input_hash = xxhash_rust::xxh3::xxh3_64(content.as_bytes()).to_string();

        // Check cache first
        if let Some(cached) = self.cache.get_cached_response("summarize", &input_hash, model).await? {
            return Ok(cached);
        }

        // Generate summary
        let llm = MultiProviderLLM::from_model_string(model)
            .map_err(|e| AIError::ProviderUnavailable(e.to_string()))?;

        let prompt = format!(
            "Summarize the following content concisely:\n\n{}\n\nSummary:",
            content
        );

        let summary = llm.complete(&prompt).await
            .map_err(|e| AIError::ApiError(e.to_string()))?;

        // Cache the result
        self.cache.cache_llm_response("summarize", &input_hash, model, &summary, 1).await?;

        Ok(summary)
    }

    /// Consolidate multiple documents
    pub async fn consolidate(
        &self,
        documents: &[&str],
        use_model: Option<&str>,
    ) -> Result<String, AIError> {
        let model = use_model.unwrap_or(&self.config.model_strong);
        let combined = documents.join("\n---\n");
        let input_hash = xxhash_rust::xxh3::xxh3_64(combined.as_bytes()).to_string();

        // Check cache
        if let Some(cached) = self.cache.get_cached_response("consolidate", &input_hash, model).await? {
            return Ok(cached);
        }

        let llm = MultiProviderLLM::from_model_string(model)
            .map_err(|e| AIError::ProviderUnavailable(e.to_string()))?;

        let prompt = format!(
            "Consolidate these documents into a single cohesive document. \
             Restructure and supplement sections as needed to create a comprehensive whole:\n\n\
             {}\n\nConsolidated document:",
            combined
        );

        let result = llm.complete(&prompt).await
            .map_err(|e| AIError::ApiError(e.to_string()))?;

        self.cache.cache_llm_response("consolidate", &input_hash, model, &result, 1).await?;

        Ok(result)
    }

    /// Extract topic from documents
    pub async fn extract_topic(
        &self,
        topic: &str,
        documents: &[&str],
        use_model: Option<&str>,
    ) -> Result<String, AIError> {
        let model = use_model.unwrap_or(&self.config.model_strong);
        let combined = documents.join("\n---\n");
        let cache_key = format!("{}:{}", topic, combined);
        let input_hash = xxhash_rust::xxh3::xxh3_64(cache_key.as_bytes()).to_string();

        if let Some(cached) = self.cache.get_cached_response("topic", &input_hash, model).await? {
            return Ok(cached);
        }

        let llm = MultiProviderLLM::from_model_string(model)
            .map_err(|e| AIError::ProviderUnavailable(e.to_string()))?;

        let prompt = format!(
            "Review the following documents and extract all information related to the topic '{}'.\n\
             Consolidate the extracted information into a well-structured document.\n\n\
             Documents:\n{}\n\n\
             Extracted content about '{}':",
            topic, combined, topic
        );

        let result = llm.complete(&prompt).await
            .map_err(|e| AIError::ApiError(e.to_string()))?;

        self.cache.cache_llm_response("topic", &input_hash, model, &result, 1).await?;

        Ok(result)
    }

    /// Generate embeddings for content
    pub async fn embed(&self, content: &str) -> Result<Vec<f32>, AIError> {
        // Check if we should use local or cloud
        let use_local = self.config.embeddings.starts_with("local/") ||
                        self.config.embeddings.starts_with("ollama/");

        let provider = if use_local {
            EmbeddingProvider::Local
        } else {
            EmbeddingProvider::OpenAI
        };

        self.embedder.embed(content, provider).await
            .map_err(|e| AIError::ApiError(e.to_string()))
    }
}
```

## RAG with SurrealDB

```rust
use rig::completion::Prompt;
use rig::providers::openai;

pub struct CompositionRAG {
    cache: AICache,
    embedder: DocumentEmbedder,
    completion_model: openai::CompletionModel,
}

impl CompositionRAG {
    pub async fn answer_with_context(
        &self,
        query: &str,
        context_count: u32,
    ) -> Result<String, anyhow::Error> {
        // 1. Generate query embedding
        let query_embedding = self.embedder.embed_query(query).await?;

        // 2. Find similar documents from SurrealDB
        let similar_docs = self.cache.find_similar(query_embedding, context_count).await?;

        // 3. Build context from retrieved documents
        let context: String = similar_docs
            .iter()
            .map(|doc| format!("---\n{}\n", doc.content))
            .collect();

        // 4. Generate answer with context
        let prompt = format!(
            "Based on the following context, answer the question.\n\n\
             Context:\n{}\n\n\
             Question: {}\n\n\
             Answer:",
            context, query
        );

        let response = self.completion_model.prompt(&prompt).await?;
        Ok(response)
    }
}
```

## Architecture Patterns

### 1. Unified RAG Pipeline

**Combination**: `rig-core` + `llm` crate

```
User Query
    |
    v
[Embedding Model] --> [Vector Store (SurrealDB)]
    |                          |
    v                          v
[Context Retrieval] <---------|
    |
    v
[llm crate] --> OpenAI / Anthropic / Ollama
    |
    v
Response
```

### 2. Hybrid Cloud-Local Deployment

**Combination**: `ollama-rs` + `async-openai`

```rust
async fn route_query(query: &str) -> Result<String, anyhow::Error> {
    let complexity = classify_complexity(query).await;

    match complexity {
        Complexity::Simple => {
            // Use local Ollama for simple queries
            let ollama = Ollama::default();
            let result = ollama.generate(
                GenerationRequest::new("llama3.2".into(), query.into())
            ).await?;
            Ok(result.response)
        }
        Complexity::Complex => {
            // Route to GPT-4 for complex queries
            let client = openai::Client::from_env();
            let model = client.model("gpt-4o").build();
            Ok(model.prompt(query).await?)
        }
    }
}
```

### 3. Performance-Critical Pipeline

**Combination**: `rig-core` + `mistral.rs`

```
Application Logic
    |
    v
[Rig Orchestration] --> Agent/RAG/Extraction
    |
    v
[Mistral.rs Server] --> HTTP API (OpenAI-compatible)
    |
    v
[Optimized Model] --> ISQ, PagedAttention, FlashAttention
```

## Decision Matrix

| Requirement | Primary Library | Secondary | Notes |
|-------------|-----------------|-----------|-------|
| RAG with vector stores | rig-core | llm | Best vector store support |
| LangChain-style chains | llm-chain | langchain-rust | Template-based workflows |
| Local-first multimodal | kalosm | candle | Built-in audio/image |
| Max inference speed | mistral.rs | llama_cpp | Advanced optimizations |
| Multi-provider flexibility | llm crate | rig | Easy provider switching |
| Offline operation | kalosm/llama_cpp | llm-chain | No API dependencies |
| Agent with tools | langchain-rust | rig | Tool abstractions |

## Best Practices

1. **Start with highest abstraction needed**: Begin with rig-core, drop down to Candle only when required

2. **Use feature flags**: Enable only needed backends to reduce compile time and binary size

3. **Implement fallback strategies**: Always have backup paths for LLM calls

4. **Monitor costs**: Track API usage when combining cloud and local

5. **Cache aggressively**: LLM calls are expensive, cache by content hash

6. **Use content hashes**: xxhash (xxh3_64) for fast hashing of inputs

## Sources

- [Rig documentation](https://docs.rig.rs/)
- [SurrealDB RAG can be Rigged](https://surrealdb.com/blog/rag-can-be-rigged)
- [Building a Simple RAG System with Rust](https://masteringbackend.com/posts/building-a-simple-rag-system-application-with-rust)
- [llm-chain guide](https://www.shuttle.dev/blog/2024/06/06/llm-chain-langchain-rust)
- [Mistral.rs README](https://github.com/EricLBuehler/mistral.rs)
