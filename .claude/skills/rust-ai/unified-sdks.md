# Multi-Provider Unified SDKs

Libraries providing a single interface across multiple LLM providers.

## llm Crate (RLLM)

The primary unified SDK for Rust, published as `llm` on crates.io.

### Supported Backends
- OpenAI
- Anthropic (Claude)
- Ollama
- Google
- DeepSeek
- xAI
- Phind
- Groq

### Features
- Chat-based interactions
- Streaming responses
- Usage metadata
- Tool calls
- Text completion
- Embeddings generation
- Request validation/retry logic
- Agent modules
- REST API exposure
- Conversation history management

### Basic Usage

```rust
use rllm::{LLMBuilder, backend::LLMBackend};

#[tokio::main]
async fn main() -> rllm::Result<()> {
    let llm = LLMBuilder::new()
        .backend(LLMBackend::OpenAI)  // or Anthropic, Ollama, etc.
        .api_key(std::env::var("OPENAI_API_KEY")?)
        .model("gpt-3.5-turbo")
        .build()?;

    let reply = llm.chat(["Hello, how are you?"]).await?;
    println!("Model reply: {}", reply);
    Ok(())
}
```

### Switching Providers

```rust
// Simply change the backend and model
let llm = LLMBuilder::new()
    .backend(LLMBackend::Anthropic)
    .api_key(std::env::var("ANTHROPIC_API_KEY")?)
    .model("claude-3-sonnet")
    .build()?;
```

### Feature Flags
Enable only needed backends:

```toml
[dependencies]
llm = { version = "1.0", features = ["openai", "anthropic", "ollama"] }
```

## llm Crate Detailed Example

```rust
use llm::{Chat, Result, llm_chat};
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");

    let chat = llm_chat!("openai:gpt-4o", &api_key)
        .await?
        .with_system_message("You are a concise Rust programming expert.");

    let response = chat
        .send_user_message("Write a short example of a Rust struct.")
        .await?;

    println!("LLM Response:\n{}", response.content);
    Ok(())
}
```

## genai Crate

Experimental multi-provider SDK.

### Supported Providers
- OpenAI
- Anthropic
- Google PaLM
- Cohere

### Design Philosophy
- Ergonomic API
- Feature-flagged providers
- Common interface patterns

## allms

Type-safe interactions across providers:
- OpenAI
- Anthropic
- Mistral
- Gemini

Attempts to share a common interface for all backends.

## llmclient

Focused Rust client for:
- Gemini
- OpenAI
- Anthropic
- Mistral

## Choosing a Unified SDK

| Crate | Maturity | Provider Count | Special Features |
|-------|----------|----------------|------------------|
| `llm` (RLLM) | Production | 8+ | Chains, agents, REST API |
| `genai` | Experimental | 4 | Ergonomic API |
| `allms` | Active | 4 | Type safety focus |
| `llmclient` | Active | 4 | Lightweight |

## Cargo.toml

```toml
[dependencies]
# RLLM (recommended)
llm = { version = "1.0", features = ["openai", "anthropic"] }

# Alternative
genai = "0.1"
```

## Sources

- [RLLM GitHub](https://github.com/graniet/rllm)
- [llm crate](https://crates.io/crates/llm)
- [rllm docs](https://docs.rs/rllm)
- [genai crate](https://crates.io/crates/genai)
