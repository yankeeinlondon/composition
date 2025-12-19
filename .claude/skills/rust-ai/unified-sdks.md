# Multi-Provider Unified SDKs

Libraries providing a single interface across multiple LLM providers. Use these when you need to support multiple backends without provider-specific code.

## llm Crate (RLLM)

The primary unified SDK for Rust, published as `llm` on crates.io. Best choice when you need broad provider support.

### Supported Backends

| Backend | API Key Env Var | Notes |
|---------|-----------------|-------|
| OpenAI | `OPENAI_API_KEY` | Full support |
| Anthropic | `ANTHROPIC_API_KEY` | Full support |
| Ollama | None (local) | Full support |
| Google | `GOOGLE_API_KEY` | Gemini models |
| DeepSeek | `DEEPSEEK_API_KEY` | Native support |
| OpenRouter | `OPENROUTER_API_KEY` | Multi-provider access |
| xAI | `XAI_API_KEY` | Grok models |
| Phind | `PHIND_API_KEY` | Code-focused |
| Groq | `GROQ_API_KEY` | Fast inference |

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
        .backend(LLMBackend::OpenAI)
        .api_key(std::env::var("OPENAI_API_KEY")?)
        .model("gpt-4o")
        .build()?;

    let reply = llm.chat(["Hello, how are you?"]).await?;
    println!("Model reply: {}", reply);
    Ok(())
}
```

### Multi-Provider with llm

Parse "provider/model" format for flexible model selection:

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

### Switching Providers

```rust
// Simply change the backend and model
let llm = LLMBuilder::new()
    .backend(LLMBackend::Anthropic)
    .api_key(std::env::var("ANTHROPIC_API_KEY")?)
    .model("claude-3-5-sonnet")
    .build()?;
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

### llm Macro Style

```rust
use llm::{Chat, Result, llm_chat};
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");

    // Format: "provider:model"
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

### Feature Flags

Enable only needed backends to reduce compile time:

```toml
[dependencies]
llm = { version = "1.0", features = ["openai", "anthropic", "ollama"] }
```

Available features:
- `openai` - OpenAI API
- `anthropic` - Anthropic Claude
- `ollama` - Local Ollama
- `google` - Google Gemini
- `deepseek` - DeepSeek
- `groq` - Groq
- `openrouter` - OpenRouter proxy

## genai Crate

Experimental multi-provider SDK with ergonomic API.

### Supported Providers
- OpenAI
- Anthropic
- Google PaLM
- Cohere

### Design Philosophy
- Ergonomic, Rust-idiomatic API
- Feature-flagged providers
- Common interface patterns

## allms

Type-safe interactions across providers:
- OpenAI
- Anthropic
- Mistral
- Gemini

Focus on compile-time type safety for provider-specific features.

## llmclient

Focused lightweight client for:
- Gemini
- OpenAI
- Anthropic
- Mistral

## Choosing a Unified SDK

| Crate | Maturity | Provider Count | Special Features | Best For |
|-------|----------|----------------|------------------|----------|
| **llm** (RLLM) | Production | 8+ | Chains, agents, REST API | Most use cases |
| genai | Experimental | 4 | Ergonomic API | Simple projects |
| allms | Active | 4 | Type safety focus | Type-heavy codebases |
| llmclient | Active | 4 | Lightweight | Minimal footprint |

## When to Use Unified vs Provider-Specific

**Use unified SDK (llm)** when:
- Supporting multiple providers
- Provider might change based on config
- Building a framework or library
- Cost optimization across providers

**Use provider-specific SDK** when:
- Only one provider needed
- Need provider-specific features
- Maximum type safety for that provider
- Building provider-specific integrations

## Cargo.toml

```toml
[dependencies]
# RLLM (recommended for multi-provider)
llm = { version = "1.0", features = ["openai", "anthropic", "ollama", "deepseek"] }

# Alternative
genai = "0.1"

# Common
tokio = { version = "1", features = ["full"] }
anyhow = "1"
```

## Sources

- [RLLM GitHub](https://github.com/graniet/rllm)
- [llm crate](https://crates.io/crates/llm)
- [rllm docs](https://docs.rs/rllm)
- [genai crate](https://crates.io/crates/genai)
- [Rust Ecosystem for AI & LLMs](https://hackmd.io/@Hamze/Hy5LiRV1gg)
