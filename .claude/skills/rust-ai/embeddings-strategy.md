# Embeddings Strategy

Comprehensive guide to vector embeddings in Rust for semantic search, RAG systems, and document similarity.

## Overview

Two primary approaches for generating embeddings:

| Approach | Library | Pros | Cons |
|----------|---------|------|------|
| **Cloud** | `rig-core`, `async-openai` | High quality, no local resources | API costs, network dependency |
| **Local** | `fastembed`, `rig-fastembed` | Free, offline, private | Requires model download, CPU/GPU |

## Cloud Embeddings with rig-core

Best for consistency when using rig for completions:

```rust
use rig::embeddings::{EmbeddingsBuilder, EmbeddingModel};
use rig::providers::openai;

pub struct DocumentEmbedder {
    embedding_model: openai::EmbeddingModel,
}

impl DocumentEmbedder {
    pub fn new() -> Self {
        let client = openai::Client::from_env();
        let embedding_model = client.embedding_model("text-embedding-3-small");
        Self { embedding_model }
    }

    pub async fn embed_documents(
        &self,
        documents: Vec<(&str, &str)>,  // (id, content)
    ) -> Result<Vec<rig::embeddings::Embedding>, anyhow::Error> {
        let mut builder = EmbeddingsBuilder::new(self.embedding_model.clone());

        for (id, content) in documents {
            builder = builder.simple_document(id, content);
        }

        let embeddings = builder.build().await?;
        Ok(embeddings)
    }

    pub async fn embed_query(&self, query: &str) -> Result<Vec<f32>, anyhow::Error> {
        let embedding = self.embedding_model.embed_text(query).await?;
        Ok(embedding.vec)
    }
}
```

## Cloud Embeddings with async-openai

Direct OpenAI SDK for fine-grained control:

```rust
use async_openai::{
    Client,
    types::{CreateEmbeddingRequestArgs, EmbeddingInput},
};

pub async fn openai_embeddings(texts: Vec<&str>) -> Result<Vec<Vec<f32>>, anyhow::Error> {
    let client = Client::new();  // Uses OPENAI_API_KEY

    let request = CreateEmbeddingRequestArgs::default()
        .model("text-embedding-3-small")
        .input(texts)
        .build()?;

    let response = client.embeddings().create(request).await?;
    let embeddings: Vec<Vec<f32>> = response.data
        .into_iter()
        .map(|e| e.embedding)
        .collect();

    Ok(embeddings)
}
```

## Local Embeddings with fastembed

ONNX-based local embeddings, no API needed:

```rust
use fastembed::{TextEmbedding, InitOptions, EmbeddingModel};

pub struct LocalEmbedder {
    model: TextEmbedding,
}

impl LocalEmbedder {
    /// Default: BGE-Small-EN-v1.5 (384 dimensions)
    pub fn new() -> Result<Self, anyhow::Error> {
        let model = TextEmbedding::try_new(
            InitOptions::new(EmbeddingModel::BGESmallENV15)
                .with_show_download_progress(true)
        )?;
        Ok(Self { model })
    }

    /// Custom model selection
    pub fn new_with_model(model_type: EmbeddingModel) -> Result<Self, anyhow::Error> {
        let model = TextEmbedding::try_new(
            InitOptions::new(model_type)
                .with_show_download_progress(true)
        )?;
        Ok(Self { model })
    }

    /// Batch embedding (more efficient)
    pub fn embed(&self, texts: Vec<&str>) -> Result<Vec<Vec<f32>>, anyhow::Error> {
        let embeddings = self.model.embed(texts, None)?;
        Ok(embeddings)
    }

    pub fn embed_single(&self, text: &str) -> Result<Vec<f32>, anyhow::Error> {
        let embeddings = self.model.embed(vec![text], None)?;
        Ok(embeddings.into_iter().next().unwrap())
    }
}

// Usage
fn main() -> Result<(), anyhow::Error> {
    let embedder = LocalEmbedder::new()?;

    let texts = vec![
        "This is the first document",
        "This is the second document",
    ];
    let embeddings = embedder.embed(texts)?;

    // Each embedding is Vec<f32> with 384 dimensions
    println!("Embedding dimensions: {}", embeddings[0].len());
    Ok(())
}
```

### Available fastembed Models

| Model | Dimensions | Quality | Speed |
|-------|------------|---------|-------|
| `BGESmallENV15` | 384 | Good | Fast |
| `BGEBaseENV15` | 768 | Better | Medium |
| `BGELargeENV15` | 1024 | Best | Slower |
| `AllMiniLML6V2` | 384 | Good | Fast |

## rig + fastembed Integration

The `rig-fastembed` crate provides native integration:

```rust
use rig_fastembed::FastEmbedEmbedder;
use rig::embeddings::EmbeddingsBuilder;

pub async fn local_rag_embeddings() -> Result<(), anyhow::Error> {
    let embedder = FastEmbedEmbedder::default();

    let embeddings = EmbeddingsBuilder::new(embedder)
        .simple_document("doc1", "Content of first document...")
        .simple_document("doc2", "Content of second document...")
        .build()
        .await?;

    // Store in vector store (SurrealDB, Qdrant, etc.)
    Ok(())
}
```

## Hybrid Embedding Strategy

Flexible strategy that uses cloud or local based on availability and preference:

```rust
pub enum EmbeddingProvider {
    OpenAI,
    Local,
}

pub struct HybridEmbedder {
    local: Option<LocalEmbedder>,
    openai_key: Option<String>,
}

impl HybridEmbedder {
    pub fn new() -> Result<Self, anyhow::Error> {
        let local = LocalEmbedder::new().ok();
        let openai_key = std::env::var("OPENAI_API_KEY").ok();

        Ok(Self { local, openai_key })
    }

    pub async fn embed(
        &self,
        text: &str,
        prefer: EmbeddingProvider,
    ) -> Result<Vec<f32>, anyhow::Error> {
        match prefer {
            EmbeddingProvider::OpenAI if self.openai_key.is_some() => {
                cloud_embeddings(text).await
            }
            EmbeddingProvider::Local if self.local.is_some() => {
                self.local.as_ref().unwrap().embed_single(text)
            }
            _ => {
                // Fallback to whatever is available
                if let Some(ref local) = self.local {
                    local.embed_single(text)
                } else {
                    cloud_embeddings(text).await
                }
            }
        }
    }

    /// Check what's available
    pub fn available_providers(&self) -> Vec<EmbeddingProvider> {
        let mut providers = Vec::new();
        if self.openai_key.is_some() {
            providers.push(EmbeddingProvider::OpenAI);
        }
        if self.local.is_some() {
            providers.push(EmbeddingProvider::Local);
        }
        providers
    }
}

async fn cloud_embeddings(text: &str) -> Result<Vec<f32>, anyhow::Error> {
    use rig::providers::openai;
    let client = openai::Client::from_env();
    let model = client.embedding_model("text-embedding-3-small");
    let embedding = model.embed_text(text).await?;
    Ok(embedding.vec)
}
```

## Ollama Embeddings

For fully local operation with Ollama:

```rust
use ollama_rs::{Ollama, generation::embeddings::request::GenerateEmbeddingsRequest};

pub async fn ollama_embeddings(
    texts: Vec<String>,
    model: &str,
) -> Result<Vec<Vec<f32>>, anyhow::Error> {
    let ollama = Ollama::default();  // localhost:11434

    let mut embeddings = Vec::new();
    for text in texts {
        let request = GenerateEmbeddingsRequest::new(model.to_string(), text.into());
        let response = ollama.generate_embeddings(request).await?;
        embeddings.push(response.embeddings);
    }

    Ok(embeddings)
}
```

## Storing Embeddings in SurrealDB

```rust
use surrealdb::{Surreal, engine::remote::ws::Ws};
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

pub async fn setup_vector_index(db: &Surreal<surrealdb::engine::remote::ws::Client>) -> Result<(), anyhow::Error> {
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

    Ok(())
}

pub async fn find_similar(
    db: &Surreal<surrealdb::engine::remote::ws::Client>,
    query_embedding: Vec<f32>,
    limit: u32,
) -> Result<Vec<DocumentRecord>, anyhow::Error> {
    let mut response = db.query(r#"
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
```

## Dimension Considerations

| Embedding Model | Dimensions | SurrealDB Index Setting |
|-----------------|------------|-------------------------|
| `text-embedding-3-small` | 1536 | `DIMENSION 1536` |
| `text-embedding-3-large` | 3072 | `DIMENSION 3072` |
| `BGESmallENV15` | 384 | `DIMENSION 384` |
| `BGEBaseENV15` | 768 | `DIMENSION 768` |
| Ollama (varies) | Model-specific | Check model docs |

**Important**: Match your vector index dimension to your embedding model. Mismatched dimensions cause errors or poor search results.

## Performance Tips

1. **Batch embeddings**: Process multiple texts at once when possible
2. **Cache embeddings**: Store computed embeddings in database
3. **Use content hashes**: Skip re-embedding unchanged content
4. **Choose appropriate model**: Smaller models for speed, larger for quality
5. **Consider local for bulk**: Local embeddings avoid API rate limits

## Sources

- [fastembed-rs](https://github.com/Anush008/fastembed-rs)
- [rig-fastembed](https://crates.io/crates/rig-fastembed)
- [OpenAI Embeddings](https://platform.openai.com/docs/guides/embeddings)
- [SurrealDB Vector Search](https://surrealdb.com/docs/surrealql/functions/vector)
- [Local Embeddings with Fastembed, Rig & Rust](https://dev.to/joshmo_dev/local-embeddings-with-fastembed-rig-rust-3581)
