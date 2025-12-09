# Combining Solutions for Comprehensive Applications

Strategic combinations of Rust LLM libraries to build full-featured applications.

## Architecture Patterns

### 1. Unified RAG Pipeline

**Goal**: High-performance, memory-safe RAG system with flexible provider switching.

**Combination**: `rig` + `llm` crate

**Architecture**:
```
User Query
    |
    v
[Embedding Model] --> [Vector Store (Qdrant/MongoDB)]
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

**Rationale**:
- `rig` provides superior RAG framework with vector store integration
- `llm` crate serves as underlying `CompletionModel` implementation
- Swap providers (OpenAI to Anthropic) without changing RAG logic

**Code Pattern**:
```rust
// Use rig for RAG workflow
let rag_agent = rig_client.context_rag_agent("gpt-4")
    .dynamic_context(3, vector_store.index(embedding_model))
    .build();

// The underlying LLM call uses the unified llm crate interface
// allowing easy provider switching via configuration
```

### 2. Complex Agent Orchestration

**Goal**: Multi-step agentic workflows with tool integration.

**Combination**: `langchain-rust` + `llm` crate

**Architecture**:
```
Agent Definition
    |
    v
[LangChain-rust Agent] --> [Tool Registry]
    |                           |
    v                           v
[Planning Loop] ------> [Tool Execution]
    |                           |
    v                           v
[llm crate Backend] <-------|
    |
    v
Final Response
```

**Rationale**:
- LangChain-rust manages planning, execution, tool selection
- `llm` provides unified interface for multiple LLM backends
- Tools can use different models (fast local for classification, cloud for complex)

### 3. Fully Local Stack

**Goal**: Offline operation with high-level abstractions.

**Combination**: `llm-chain` or `kalosm` + `llama_cpp`

**Architecture**:
```
Prompt Template
    |
    v
[llm-chain Orchestration]
    |
    v
[llama_cpp Inference] --> Local GGUF Model
    |
    v
Response
```

**Implementation**:
```rust
// Define chain with llm-chain
let chain = Chain::new(vec![
    Step::for_prompt_template(prompt!("Summarize: {{text}}")),
    Step::for_prompt_template(prompt!("Extract key points: {{text}}"))
]);

// Execute with local llama_cpp adapter
let local_executor = LocalLlamaExecutor::new("path/to/model.gguf")?;
let result = chain.run(params, &local_executor).await?;
```

**Benefits**:
- Privacy: Data never leaves device
- Latency: No network round-trips
- Cost: No API fees
- Reliability: No external dependencies

### 4. Hybrid Cloud-Local Deployment

**Goal**: Optimize for cost, speed, and capability.

**Combination**: `ollama-rs` + `async-openai`

**Architecture**:
```
Incoming Query
    |
    v
[Query Classifier] --> Local Ollama (fast, cheap)
    |
    +--> Simple Query --> [ollama-rs] --> Local Model
    |
    +--> Complex Query --> [async-openai] --> GPT-4
    |
    v
Response
```

**Implementation**:
```rust
async fn route_query(query: &str) -> String {
    let complexity = classify_complexity(query).await;

    match complexity {
        Complexity::Simple => {
            // Use local Ollama for simple queries
            let ollama = Ollama::default();
            let result = ollama.generate(
                GenerationRequest::new("llama2:7b".into(), query.into())
            ).await?;
            result.response
        }
        Complexity::Complex => {
            // Route to GPT-4 for complex queries
            let client = OpenAIClient::new(&api_key);
            let response = client.create_chat(ChatArguments::new(
                "gpt-4",
                vec![Message::user(query)]
            )).await?;
            response.content
        }
    }
}
```

### 5. Performance-Critical Pipeline

**Goal**: Maximum inference speed with orchestration.

**Combination**: `rig` + `mistral.rs`

**Architecture**:
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

**Benefits**:
- Rig's high-level abstractions for complex workflows
- Mistral.rs's extreme performance optimizations
- OpenAI-compatible API makes integration seamless

### 6. Multimodal Application

**Goal**: Text, audio, and image processing in one application.

**Combination**: `kalosm` (primary) + `candle` (custom components)

**Architecture**:
```
Multimodal Input
    |
    +-> Text  --> [Kalosm LLM]
    |
    +-> Audio --> [Kalosm Whisper]
    |
    +-> Image --> [Kalosm SAM / Custom Candle Model]
    |
    v
Unified Processing
    |
    v
Response
```

**Benefits**:
- Kalosm handles common multimodal tasks
- Candle allows custom model integration
- Shared foundation reduces dependencies

## Decision Matrix

| Requirement | Primary Library | Secondary | Notes |
|-------------|-----------------|-----------|-------|
| RAG with vector stores | rig | llm | Best vector store support |
| LangChain-style chains | llm-chain | langchain-rust | Template-based workflows |
| Local-first multimodal | kalosm | candle | Built-in audio/image |
| Max inference speed | mistral.rs | llama_cpp | Advanced optimizations |
| Multi-provider flexibility | llm crate | rig | Easy provider switching |
| Offline operation | kalosm/llama_cpp | llm-chain | No API dependencies |
| Agent with tools | langchain-rust | rig | Tool abstractions |

## Common Integration Points

### Vector Stores
```rust
// Rig with Qdrant
use rig_qdrant::QdrantVectorStore;

// Kalosm with SurrealDB
use kalosm::language::*;
use surrealdb::Surreal;

// LangChain-rust with SQLite
use langchain_rust::vectorstore::sqlite::SqliteVectorStore;
```

### Embeddings
```rust
// Shared embedding model across components
let embedding_model = openai_client.embedding_model("text-embedding-ada-002");

// Use with rig
let rig_embeddings = EmbeddingsBuilder::new(embedding_model.clone()).build();

// Use with custom vector store
let vectors = embedding_model.embed(texts).await?;
custom_store.insert(vectors).await?;
```

### Error Handling
```rust
// Unified error handling across libraries
use anyhow::Result;

async fn hybrid_query(query: &str) -> Result<String> {
    // Try local first
    match local_llm.generate(query).await {
        Ok(response) => Ok(response),
        Err(_) => {
            // Fallback to cloud
            cloud_llm.chat(query).await
        }
    }
}
```

## Best Practices

1. **Start with highest abstraction needed**: Begin with rig/llm-chain, drop down to Candle only when required

2. **Use feature flags**: Enable only needed backends to reduce compile time and binary size

3. **Implement fallback strategies**: Always have backup paths for LLM calls

4. **Monitor costs**: Track API usage when combining cloud and local

5. **Benchmark your stack**: Performance varies significantly by model and hardware

## Sources

- [Rig documentation](https://docs.rig.rs/)
- [llm-chain guide](https://www.shuttle.dev/blog/2024/06/06/llm-chain-langchain-rust)
- [Kalosm examples](https://docs.rs/kalosm)
- [Mistral.rs README](https://github.com/EricLBuehler/mistral.rs)
