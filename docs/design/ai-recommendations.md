# AI Integration Recommendations

This document provides detailed recommendations for Rust crates to use for AI/LLM integration in the Composition project. Based on the project's requirements for summarization, consolidation, topic extraction, and vector embeddings across multiple providers.

## Table of Contents

- [Overview](#overview)
- [Provider Support Matrix](#provider-support-matrix)
- [Primary Recommendation: rig-core](#primary-recommendation-rig-core)
- [Alternative: llm Crate](#alternative-llm-crate)
- [Embeddings Strategy](#embeddings-strategy)
- [Provider-Specific SDKs](#provider-specific-sdks)
- [Local Inference with Ollama](#local-inference-with-ollama)
- [SurrealDB Integration](#surrealdb-integration)
- [Architecture Patterns](#architecture-patterns)
- [Cargo Dependencies](#cargo-dependencies)
- [Decision Matrix](#decision-matrix)
- [Sources](#sources)

---

## Overview

The Composition project requires AI capabilities for:

| Use Case | Description | Model Type |
|----------|-------------|------------|
| **Summarization** | Condense Markdown/HTML/PDF content | `MODEL` or `MODEL_FAST` |
| **Consolidation** | Merge multiple documents into a cohesive whole | `MODEL` or `MODEL_STRONG` |
| **Topic Extraction** | Extract and structure content around a topic | `MODEL_STRONG` |
| **Vector Embeddings** | Encode/decode for semantic search | `EMBEDDINGS` |

### Required Providers

| Provider | Priority | API Compatibility |
|----------|----------|-------------------|
| OpenAI | Required | Native |
| Anthropic | Required | Native |
| Gemini | Required | Native |
| Open Router | Required | OpenAI-compatible |
| Ollama | Required (local fallback) | OpenAI-compatible |

### Nice-to-Have Providers

| Provider | Notes |
|----------|-------|
| [Kimi K2](https://kimi.com) | Moonshot AI |
| [Zai/GLM](https://z.ai) | Zhipu AI |
| [DeepSeek](https://deepseek.com) | Cost-effective reasoning |
| [ZenMux](https://zenmux.ai/) | Aggregator (less expensive than Open Router) |

---

## Provider Support Matrix

| Crate | OpenAI | Anthropic | Gemini | OpenRouter | Ollama | DeepSeek | Embeddings |
|-------|--------|-----------|--------|------------|--------|----------|------------|
| **rig-core** | Yes | Yes | Yes | Via OpenAI | Yes | No | Yes |
| **llm/rllm** | Yes | Yes | Yes | Yes | Yes | Yes | Yes |
| **async-openai** | Yes | No | No | Yes* | No | No | Yes |
| **ollama-rs** | No | No | No | No | Yes | No | Yes |
| **openrouter_api** | Via proxy | Via proxy | Via proxy | Native | No | Via proxy | Via proxy |
| **fastembed** | Local | Local | Local | Local | Local | Local | Yes (local) |

*OpenRouter uses OpenAI-compatible API

---

## Primary Recommendation: rig-core

**Recommendation**: Use [`rig-core`](https://crates.io/crates/rig-core) as the primary framework for LLM operations.

### Why rig-core?

1. **Unified API**: Single interface for completions and embeddings across providers
2. **RAG-Ready**: Built-in vector store abstractions and retrieval patterns
3. **Type-Safe**: Rust-native with strong typing via serde
4. **Extensible**: Trait-based design allows custom provider implementations
5. **SurrealDB Compatible**: Works with vector stores including SurrealDB
6. **Production-Ready**: Used by VT Code, Dria, Cairnify in production

### Basic Completion Example

```rust
use rig::{completion::Prompt, providers::openai};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let openai_client = openai::Client::from_env();  // Uses OPENAI_API_KEY
    let model = openai_client.model("gpt-4o").build();

    let response = model.prompt("Summarize the key points of this text...").await?;
    println!("{}", response);
    Ok(())
}
```

### Provider Abstraction Pattern

```rust
use rig::completion::{CompletionModel, Prompt};
use rig::providers::{openai, anthropic, ollama};

pub enum ModelProvider {
    OpenAI(openai::Client),
    Anthropic(anthropic::Client),
    Ollama(ollama::Client),
}

pub struct CompositionAI {
    client: Box<dyn CompletionModel>,
    embedding_model: Box<dyn rig::embeddings::EmbeddingModel>,
}

impl CompositionAI {
    pub fn from_env_model(model_str: &str) -> Result<Self, anyhow::Error> {
        // Parse "provider/model" format
        let parts: Vec<&str> = model_str.split('/').collect();
        let (provider, model_name) = (parts[0], parts[1]);

        let client: Box<dyn CompletionModel> = match provider {
            "openai" => {
                let client = openai::Client::from_env();
                Box::new(client.model(model_name).build())
            }
            "anthropic" => {
                let client = anthropic::Client::from_env();
                Box::new(client.model(model_name).build())
            }
            "ollama" => {
                let client = ollama::Client::from_env();
                Box::new(client.model(model_name).build())
            }
            "openrouter" => {
                // OpenRouter uses OpenAI-compatible API
                let client = openai::Client::new(
                    &std::env::var("OPENROUTER_API_KEY")?,
                    "https://openrouter.ai/api/v1"
                );
                Box::new(client.model(model_name).build())
            }
            _ => anyhow::bail!("Unsupported provider: {}", provider),
        };

        // Similar pattern for embeddings...
        Ok(Self { client, embedding_model: todo!() })
    }

    pub async fn summarize(&self, content: &str) -> Result<String, anyhow::Error> {
        let prompt = format!(
            "Summarize the following content concisely:\n\n{}\n\nSummary:",
            content
        );
        Ok(self.client.prompt(&prompt).await?)
    }

    pub async fn consolidate(&self, documents: &[&str]) -> Result<String, anyhow::Error> {
        let combined = documents.join("\n\n---\n\n");
        let prompt = format!(
            "Consolidate these documents into a cohesive whole, \
             restructuring and supplementing as needed:\n\n{}\n\nConsolidated document:",
            combined
        );
        Ok(self.client.prompt(&prompt).await?)
    }
}
```

### Embeddings with rig-core

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

### Type-Safe Structured Extraction

Useful for topic extraction where we want structured output:

```rust
use serde::Deserialize;
use rig::providers::openai::Client;

#[derive(Debug, Deserialize, rig::JsonSchema)]
pub struct ExtractedTopic {
    pub topic_name: String,
    pub summary: String,
    pub key_points: Vec<String>,
    pub related_concepts: Vec<String>,
}

pub async fn extract_topic(
    documents: &[&str],
    topic: &str,
) -> Result<ExtractedTopic, anyhow::Error> {
    let client = Client::from_env();
    let extractor = client.extractor::<ExtractedTopic>("gpt-4o")
        .preamble(&format!(
            "You are an expert at extracting information about '{}' from documents. \
             Analyze the provided documents and extract all relevant information about this topic.",
            topic
        ))
        .build();

    let combined = documents.join("\n\n---\n\n");
    let extracted: ExtractedTopic = extractor.extract(&combined).await?;
    Ok(extracted)
}
```

---

## Alternative: llm Crate

For simpler use cases or when more providers are needed, the [`llm`](https://crates.io/crates/llm) crate provides a unified interface.

### Why llm/rllm?

1. **More Providers**: Supports DeepSeek, xAI, Phind, Groq, Cohere natively
2. **Simpler API**: Builder pattern for quick setup
3. **Feature Flags**: Include only needed backends
4. **Parallel Evaluation**: Test multiple providers simultaneously

### Basic Usage

```rust
use llm::{Chat, Result, llm_chat};
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");

    // Format: "provider:model"
    let chat = llm_chat!("openai:gpt-4o", &api_key)
        .await?
        .with_system_message("You are a document summarization expert.");

    let response = chat
        .send_user_message("Summarize this document...")
        .await?;

    println!("{}", response.content);
    Ok(())
}
```

### Multi-Provider with llm

```rust
use rllm::{LLMBuilder, backend::LLMBackend};

pub struct MultiProviderLLM {
    model: String,
    backend: LLMBackend,
}

impl MultiProviderLLM {
    /// Parse "provider/model" format
    pub fn from_model_string(model_str: &str) -> Result<Self, anyhow::Error> {
        let parts: Vec<&str> = model_str.split('/').collect();
        let (provider, model) = (parts[0], parts[1]);

        let backend = match provider.to_lowercase().as_str() {
            "openai" => LLMBackend::OpenAI,
            "anthropic" => LLMBackend::Anthropic,
            "ollama" => LLMBackend::Ollama,
            "deepseek" => LLMBackend::DeepSeek,
            "google" | "gemini" => LLMBackend::Google,
            "groq" => LLMBackend::Groq,
            "openrouter" => LLMBackend::OpenRouter,
            _ => anyhow::bail!("Unknown provider: {}", provider),
        };

        Ok(Self {
            model: model.to_string(),
            backend,
        })
    }

    pub async fn complete(&self, prompt: &str) -> rllm::Result<String> {
        let api_key = self.get_api_key()?;

        let llm = LLMBuilder::new()
            .backend(self.backend.clone())
            .api_key(api_key)
            .model(&self.model)
            .build()?;

        llm.chat([prompt]).await
    }

    fn get_api_key(&self) -> Result<String, anyhow::Error> {
        let env_var = match self.backend {
            LLMBackend::OpenAI => "OPENAI_API_KEY",
            LLMBackend::Anthropic => "ANTHROPIC_API_KEY",
            LLMBackend::Google => "GOOGLE_API_KEY",
            LLMBackend::DeepSeek => "DEEPSEEK_API_KEY",
            LLMBackend::Groq => "GROQ_API_KEY",
            LLMBackend::OpenRouter => "OPENROUTER_API_KEY",
            LLMBackend::Ollama => return Ok(String::new()), // No key needed
            _ => anyhow::bail!("Unknown backend"),
        };
        std::env::var(env_var).map_err(|_| anyhow::anyhow!("{} not set", env_var))
    }
}
```

### Embeddings with llm

```rust
use rllm::{LLMBuilder, backend::LLMBackend};

pub async fn generate_embeddings(text: &str) -> rllm::Result<Vec<f32>> {
    let llm = LLMBuilder::new()
        .backend(LLMBackend::OpenAI)
        .api_key(std::env::var("OPENAI_API_KEY")?)
        .model("text-embedding-3-small")
        .build()?;

    let embedding = llm.embed(text).await?;
    Ok(embedding)
}
```

---

## Embeddings Strategy

For embeddings, we have two main approaches:

### 1. Cloud Embeddings (via rig-core or llm)

Best for consistency with the same provider as completions:

```rust
use rig::providers::openai;

pub async fn cloud_embeddings(text: &str) -> Result<Vec<f32>, anyhow::Error> {
    let client = openai::Client::from_env();
    let model = client.embedding_model("text-embedding-3-small");
    let embedding = model.embed_text(text).await?;
    Ok(embedding.vec)
}
```

### 2. Local Embeddings with fastembed

For offline/local fallback or to reduce API costs:

```rust
use fastembed::{TextEmbedding, InitOptions, EmbeddingModel};

pub struct LocalEmbedder {
    model: TextEmbedding,
}

impl LocalEmbedder {
    pub fn new() -> Result<Self, anyhow::Error> {
        // Uses BGE-Small-EN-v1.5 by default (384 dimensions)
        let model = TextEmbedding::try_new(
            InitOptions::new(EmbeddingModel::BGESmallENV15)
                .with_show_download_progress(true)
        )?;
        Ok(Self { model })
    }

    pub fn new_with_model(model_type: EmbeddingModel) -> Result<Self, anyhow::Error> {
        let model = TextEmbedding::try_new(
            InitOptions::new(model_type)
                .with_show_download_progress(true)
        )?;
        Ok(Self { model })
    }

    pub fn embed(&self, texts: Vec<&str>) -> Result<Vec<Vec<f32>>, anyhow::Error> {
        let embeddings = self.model.embed(texts, None)?;
        Ok(embeddings)
    }

    pub fn embed_single(&self, text: &str) -> Result<Vec<f32>, anyhow::Error> {
        let embeddings = self.model.embed(vec![text], None)?;
        Ok(embeddings.into_iter().next().unwrap())
    }
}

// Usage example
fn main() -> Result<(), anyhow::Error> {
    let embedder = LocalEmbedder::new()?;

    // Batch embedding (more efficient)
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

### Hybrid Embedding Strategy

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
                // Use cloud embeddings
                cloud_embeddings(text).await
            }
            EmbeddingProvider::Local if self.local.is_some() => {
                // Use local embeddings
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
}
```

### rig + fastembed Integration

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

    // Store in vector store (SurrealDB, etc.)
    Ok(())
}
```

---

## Provider-Specific SDKs

For cases where you need direct provider access:

### OpenAI with async-openai

```rust
use async_openai::{
    Client,
    types::{
        CreateChatCompletionRequestArgs,
        ChatCompletionRequestUserMessageArgs,
        CreateEmbeddingRequestArgs,
    },
};

pub async fn openai_summarize(content: &str) -> Result<String, anyhow::Error> {
    let client = Client::new();  // Uses OPENAI_API_KEY

    let request = CreateChatCompletionRequestArgs::default()
        .model("gpt-4o")
        .messages([
            ChatCompletionRequestUserMessageArgs::default()
                .content(format!("Summarize: {}", content))
                .build()?
                .into(),
        ])
        .build()?;

    let response = client.chat().create(request).await?;
    let content = response.choices[0].message.content.clone()
        .unwrap_or_default();

    Ok(content)
}

pub async fn openai_embeddings(texts: Vec<&str>) -> Result<Vec<Vec<f32>>, anyhow::Error> {
    let client = Client::new();

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

### Anthropic with anthropic-rs

```rust
use anthropic::{Client, AnthropicConfig, CompleteRequestBuilder, HUMAN_PROMPT, AI_PROMPT};

pub async fn anthropic_summarize(content: &str) -> Result<String, anyhow::Error> {
    let cfg = AnthropicConfig::new()?;  // Uses ANTHROPIC_API_KEY
    let client = Client::try_from(cfg)?;

    let request = CompleteRequestBuilder::default()
        .model("claude-3-5-sonnet-20241022")
        .prompt(format!(
            "{HUMAN_PROMPT}Summarize the following content:\n\n{content}{AI_PROMPT}"
        ))
        .max_tokens_to_sample(2000)
        .stream_response(false)
        .build()?;

    let response = client.complete(request).await?;
    Ok(response.completion)
}
```

### OpenRouter with openrouter_api

```rust
use openrouter_api::{Client, ChatRequest, Message, Role};

pub async fn openrouter_complete(
    model: &str,
    prompt: &str,
) -> Result<String, anyhow::Error> {
    let client = Client::new()?;  // Uses OPENROUTER_API_KEY

    let request = ChatRequest::new(model)
        .with_message(Message::new(Role::User, prompt));

    let response = client.chat(request).await?;
    let content = response.choices[0].message.content.clone();

    Ok(content)
}

// Example: Using Claude via OpenRouter
pub async fn claude_via_openrouter(prompt: &str) -> Result<String, anyhow::Error> {
    openrouter_complete("anthropic/claude-3.5-sonnet", prompt).await
}

// Example: Using DeepSeek via OpenRouter
pub async fn deepseek_via_openrouter(prompt: &str) -> Result<String, anyhow::Error> {
    openrouter_complete("deepseek/deepseek-r1", prompt).await
}
```

---

## Local Inference with Ollama

### Using ollama-rs

```rust
use ollama_rs::{Ollama, generation::completion::GenerationRequest};

pub struct OllamaClient {
    client: Ollama,
    model: String,
}

impl OllamaClient {
    pub fn new(model: &str) -> Self {
        Self {
            client: Ollama::default(),  // localhost:11434
            model: model.to_string(),
        }
    }

    pub fn with_host(host: &str, port: u16, model: &str) -> Self {
        Self {
            client: Ollama::new(format!("http://{}:{}", host, port)),
            model: model.to_string(),
        }
    }

    pub async fn complete(&self, prompt: &str) -> Result<String, anyhow::Error> {
        let request = GenerationRequest::new(self.model.clone(), prompt.to_string());
        let response = self.client.generate(request).await?;
        Ok(response.response)
    }

    pub async fn summarize(&self, content: &str) -> Result<String, anyhow::Error> {
        let prompt = format!(
            "Summarize the following content concisely:\n\n{}\n\nSummary:",
            content
        );
        self.complete(&prompt).await
    }

    pub async fn is_available(&self) -> bool {
        self.client.list_local_models().await.is_ok()
    }
}

// Usage with fallback
pub async fn summarize_with_fallback(
    content: &str,
    local_model: &str,
) -> Result<String, anyhow::Error> {
    let ollama = OllamaClient::new(local_model);

    if ollama.is_available().await {
        ollama.summarize(content).await
    } else {
        // Fall back to cloud provider
        cloud_summarize(content).await
    }
}
```

### Ollama Embeddings

```rust
use ollama_rs::{Ollama, generation::embeddings::request::GenerateEmbeddingsRequest};

pub async fn ollama_embeddings(
    texts: Vec<String>,
    model: &str,
) -> Result<Vec<Vec<f32>>, anyhow::Error> {
    let ollama = Ollama::default();

    let mut embeddings = Vec::new();
    for text in texts {
        let request = GenerateEmbeddingsRequest::new(model.to_string(), text.into());
        let response = ollama.generate_embeddings(request).await?;
        embeddings.push(response.embeddings);
    }

    Ok(embeddings)
}
```

---

## SurrealDB Integration

The Composition project uses SurrealDB for caching. Here's how to integrate AI operations with the database.

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

### RAG with SurrealDB

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

---

## Architecture Patterns

### Model Selection from Environment

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

### Complete Composition AI Service

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

---

## Cargo Dependencies

### Recommended Cargo.toml

```toml
[dependencies]
# Primary framework
rig-core = { version = "0.9", features = ["derive"] }

# Provider integrations for rig
rig-fastembed = "0.1"  # Local embeddings

# Alternative unified SDK (if needed)
llm = { version = "1.0", features = ["openai", "anthropic", "ollama"], optional = true }

# Direct provider SDKs (for specific needs)
async-openai = "0.28"
ollama-rs = { version = "0.2", features = ["stream"] }

# OpenRouter support
openrouter_api = "0.7"

# Local embeddings
fastembed = "4"

# Database
surrealdb = { version = "2", features = ["protocol-ws", "kv-mem"] }

# Async runtime
tokio = { version = "1", features = ["full"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Error handling (per tech-stack.md)
thiserror = "2"
anyhow = "1"

# Utilities
chrono = { version = "0.4", features = ["serde"] }
xxhash-rust = { version = "0.8", features = ["xxh3"] }

[features]
default = ["rig"]
rig = []
llm-crate = ["llm"]
all-providers = ["rig", "llm-crate"]
```

---

## Decision Matrix

| Use Case | Primary | Alternative | Notes |
|----------|---------|-------------|-------|
| **Multi-provider completions** | `rig-core` | `llm` | rig for RAG, llm for more providers |
| **Cloud embeddings** | `rig-core` | `async-openai` | Consistent with completion model |
| **Local embeddings** | `fastembed` | `rig-fastembed` | ONNX-based, no API needed |
| **Ollama local models** | `ollama-rs` | `llm` (ollama backend) | Direct control vs unified API |
| **OpenRouter access** | `openrouter_api` | `async-openai` with custom URL | openrouter_api has native support |
| **DeepSeek models** | `llm` | OpenRouter proxy | llm has native DeepSeek support |
| **RAG pipelines** | `rig-core` | `langchain-rust` | rig has better vector store support |
| **SurrealDB vectors** | Custom + `surrealdb` | `kalosm` | kalosm has built-in SurrealDB support |

---

## Sources

### Crates and Documentation

- [rig-core](https://crates.io/crates/rig-core) - [Documentation](https://docs.rs/rig-core) - [Website](https://rig.rs/)
- [llm/rllm](https://crates.io/crates/llm) - [GitHub](https://github.com/graniet/rllm)
- [fastembed](https://crates.io/crates/fastembed) - [GitHub](https://github.com/Anush008/fastembed-rs)
- [async-openai](https://crates.io/crates/async-openai)
- [ollama-rs](https://crates.io/crates/ollama-rs) - [GitHub](https://github.com/pepperoni21/ollama-rs)
- [openrouter_api](https://crates.io/crates/openrouter_api) - [GitHub](https://github.com/socrates8300/openrouter_api)
- [surrealdb](https://crates.io/crates/surrealdb) - [Documentation](https://surrealdb.com/docs)

### Guides and Tutorials

- [Rig RAG Guide](https://docs.rig.rs/guides/rag/rag_system)
- [Local Embeddings with Fastembed, Rig & Rust](https://dev.to/joshmo_dev/local-embeddings-with-fastembed-rig-rust-3581)
- [SurrealDB RAG can be Rigged](https://surrealdb.com/blog/rag-can-be-rigged)
- [Rust Ecosystem for AI & LLMs](https://hackmd.io/@Hamze/Hy5LiRV1gg)
- [Building a Simple RAG System with Rust](https://masteringbackend.com/posts/building-a-simple-rag-system-application-with-rust)
