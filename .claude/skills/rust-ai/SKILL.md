# Rust AI/LLM Development

Expert knowledge for building LLM-powered applications in Rust, covering provider SDKs, unified interfaces, local inference engines, and orchestration frameworks.

## Ecosystem Overview

The Rust LLM ecosystem spans four categories:

| Category | Purpose | Key Crates |
|:---------|:--------|:-----------|
| **Provider SDKs** | Direct API clients for specific LLM providers | `async-openai`, `anthropic-rs`, `ollama-rs` |
| **Unified SDKs** | Single interface across multiple providers | `llm` (rllm), `genai`, `allms` |
| **Local Inference** | Run models locally without API calls | `candle`, `mistral.rs`, `llama_cpp`, `kalosm` |
| **Orchestration** | Chains, agents, RAG, and workflows | `rig`, `llm-chain`, `langchain-rust` |

## Quick Reference: Choosing a Library

**For API-based development:**
- Single provider (OpenAI/Anthropic) → Use provider-specific SDK
- Multiple providers or flexibility → Use `llm` crate (unified interface)
- OpenRouter compatibility → Configure OpenAI SDK with custom base URL

**For local/private inference:**
- Full ML framework control → `candle` (Hugging Face)
- High-performance inference → `mistral.rs`
- Local-first with multimodal → `kalosm`
- Direct llama.cpp access → `llama_cpp` or `llama-cpp-2`

**For complex workflows:**
- RAG applications → `rig` (best RAG abstractions)
- Multi-step chains → `llm-chain` (LangChain-like)
- Agent systems → `rig` or `langchain-rust`

## Detailed Documentation

For in-depth coverage, see these supplemental documents:

- [Provider SDKs](./providers.md) - OpenAI, Anthropic, Ollama direct clients
- [Unified SDKs](./unified-sdks.md) - Multi-provider interfaces (`llm`, `genai`)
- [Local Inference](./local-inference.md) - Candle, Mistral.rs, llama.cpp bindings
- [Orchestration Frameworks](./orchestration.md) - Rig, llm-chain, Kalosm details
- [Code Examples](./examples.md) - Working examples for each library
- [Strategic Combinations](./combinations.md) - How to combine libraries effectively

## Common Patterns

### Basic Chat Completion (Unified SDK)
```rust
use llm::{LLMBuilder, backend::LLMBackend};

let llm = LLMBuilder::new()
    .backend(LLMBackend::OpenAI)
    .api_key(std::env::var("OPENAI_API_KEY")?)
    .model("gpt-4")
    .build()?;

let reply = llm.chat(["Hello!"]).await?;
```

### RAG with Rig
```rust
use rig::{completion::Prompt, providers::openai};

let client = openai::Client::from_env();
let rag_agent = client.context_rag_agent("gpt-4")
    .preamble("Answer based on context")
    .dynamic_context(1, vector_store.index(embedding_model))
    .build();
```

### Local Inference with Kalosm
```rust
use kalosm::language::*;

let mut llm = Llama::new().await?;
let mut stream = llm("Write a haiku:");
stream.to_std_out().await?;
```

## Key Cargo Features

Most crates use feature flags to minimize dependencies:

```toml
# Only enable needed backends
llm = { version = "1.0", features = ["openai", "anthropic"] }

# Candle with CUDA support
candle-core = { version = "0.8", features = ["cuda"] }

# Kalosm full feature set
kalosm = { version = "0.3", features = ["full"] }
```

## External Resources

- [Rig Documentation](https://docs.rig.rs/)
- [Candle GitHub](https://github.com/huggingface/candle)
- [Mistral.rs GitHub](https://github.com/EricLBuehler/mistral.rs)
- [llm-chain Guide](https://www.shuttle.dev/blog/2024/06/06/llm-chain-langchain-rust)
- [OpenRouter](https://openrouter.ai) - Unified LLM proxy with OpenAI-compatible API
