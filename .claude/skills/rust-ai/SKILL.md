---
name: rust-ai
description: Rust libraries and frameworks for LLM integration - provider SDKs, unified interfaces, orchestration frameworks, and local inference engines
hash: "49f5bcd134defefd"
---

# Rust AI/LLM Integration

Comprehensive guide to Rust's ecosystem for Large Language Model integration. This skill provides opinionated recommendations based on real-world usage patterns.

## Primary Recommendation: rig-core

For most Rust LLM applications, **[`rig-core`](https://crates.io/crates/rig-core)** is the recommended starting point:

1. **Unified API**: Single interface for completions and embeddings across providers
2. **RAG-Ready**: Built-in vector store abstractions and retrieval patterns
3. **Type-Safe**: Rust-native with structured extraction via `#[derive(JsonSchema)]`
4. **Extensible**: Trait-based design allows custom provider implementations
5. **Production-Ready**: Used by VT Code, Dria, Cairnify in production

```rust
use rig::{completion::Prompt, providers::openai};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let client = openai::Client::from_env();  // Uses OPENAI_API_KEY
    let model = client.model("gpt-4o").build();
    let response = model.prompt("Summarize this...").await?;
    Ok(())
}
```

## Provider Support Matrix

| Crate | OpenAI | Anthropic | Gemini | OpenRouter | Ollama | DeepSeek | Embeddings |
|-------|--------|-----------|--------|------------|--------|----------|------------|
| **rig-core** | Yes | Yes | Yes | Via OpenAI | Yes | No | Yes |
| **llm/rllm** | Yes | Yes | Yes | Yes | Yes | Yes | Yes |
| **async-openai** | Yes | No | No | Yes* | No | No | Yes |
| **ollama-rs** | No | No | No | No | Yes | No | Yes |
| **fastembed** | Local | Local | Local | Local | Local | Local | Yes (local) |

*OpenRouter uses OpenAI-compatible API

## Quick Decision Guide

| Use Case | Primary | Alternative | Notes |
|----------|---------|-------------|-------|
| **Multi-provider completions** | `rig-core` | `llm` | rig for RAG, llm for more providers |
| **Cloud embeddings** | `rig-core` | `async-openai` | Consistent with completion model |
| **Local embeddings** | `fastembed` | `rig-fastembed` | ONNX-based, no API needed |
| **Ollama local models** | `ollama-rs` | `llm` (ollama) | Direct control vs unified API |
| **OpenRouter access** | `async-openai` | `openrouter_api` | Use OpenAI SDK with custom URL |
| **DeepSeek models** | `llm` | OpenRouter | llm has native DeepSeek support |
| **RAG pipelines** | `rig-core` | `langchain-rust` | rig has superior vector store support |
| **SurrealDB vectors** | `rig-core` + `surrealdb` | `kalosm` | kalosm has built-in SurrealDB |

## Library Categories

### Provider-Specific SDKs
Direct API access when you need full provider control:
- **`async-openai`**: Full OpenAI API including embeddings, images, streaming
- **`anthropic-rs`**: Claude models with streaming support
- **`ollama-rs`**: Local Ollama server (localhost:11434)
- **`openrouter_api`**: Native OpenRouter support

See [provider-sdks.md](./provider-sdks.md) for detailed examples.

### Multi-Provider Unified SDKs
Single interface for multiple providers:
- **`llm` (RLLM)**: 8+ providers, chains, agents, REST API
- **`genai`**: Experimental, ergonomic API
- **`allms`**: Type safety focus

See [unified-sdks.md](./unified-sdks.md) for usage patterns.

### Orchestration Frameworks
Complex workflows, RAG, and agents:
- **`rig-core`**: RAG, structured extraction, tool calling (recommended)
- **`llm-chain`**: LangChain-style sequential chains
- **`kalosm`**: Local-first multimodal (Candle-based)
- **`langchain-rust`**: LangChain port with agent support

See [orchestration-frameworks.md](./orchestration-frameworks.md) for detailed examples.

### Local Inference Engines
On-device model execution:
- **`candle`**: Hugging Face ML framework (foundation for others)
- **`mistral.rs`**: High-performance with ISQ, PagedAttention
- **`llama_cpp`**: Bindings to llama.cpp for GGUF models

See [local-inference.md](./local-inference.md) for setup and examples.

### Embeddings
Vector embeddings for semantic search:
- **Cloud**: `rig-core` or `async-openai` with `text-embedding-3-small`
- **Local**: `fastembed` (BGE models, ONNX-based, no API needed)
- **Integration**: `rig-fastembed` bridges fastembed with rig's API

See [embeddings-strategy.md](./embeddings-strategy.md) for hybrid patterns.

## Recommended Cargo.toml

```toml
[dependencies]
# Primary framework
rig-core = { version = "0.9", features = ["derive"] }
rig-fastembed = "0.1"  # Local embeddings

# Alternative unified SDK
llm = { version = "1.0", features = ["openai", "anthropic", "ollama"], optional = true }

# Direct provider SDKs
async-openai = "0.28"
ollama-rs = { version = "0.2", features = ["stream"] }
openrouter_api = "0.7"

# Local embeddings
fastembed = "4"

# Database (for caching/vectors)
surrealdb = { version = "2", features = ["protocol-ws", "kv-mem"] }

# Async runtime
tokio = { version = "1", features = ["full"] }

# Error handling
thiserror = "2"
anyhow = "1"

# Utilities
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }
xxhash-rust = { version = "0.8", features = ["xxh3"] }
```

## Architecture Patterns

See [combining-solutions.md](./combining-solutions.md) for:
- Provider abstraction with environment-based model selection
- Hybrid cloud-local deployment
- SurrealDB integration for embeddings and response caching
- Complete service architectures

## Sources

- [rig-core](https://crates.io/crates/rig-core) - [Docs](https://docs.rs/rig-core) - [Website](https://rig.rs/)
- [llm/rllm](https://crates.io/crates/llm) - [GitHub](https://github.com/graniet/rllm)
- [fastembed](https://crates.io/crates/fastembed) - [GitHub](https://github.com/Anush008/fastembed-rs)
- [async-openai](https://crates.io/crates/async-openai)
- [ollama-rs](https://crates.io/crates/ollama-rs)
- [SurrealDB](https://surrealdb.com/docs)
