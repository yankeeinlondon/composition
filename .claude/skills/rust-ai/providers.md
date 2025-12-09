# Provider-Specific SDKs

Direct API clients for individual LLM providers. Use these when you only need one provider or require provider-specific features.

## OpenAI SDKs

### async-openai
The most feature-complete OpenAI SDK for Rust. Fully async, strongly typed.

```rust
use openai_rust::{Client, chat::{ChatArguments, Message}};

#[tokio::main]
async fn main() {
    let api_key = std::env::var("OPENAI_API_KEY").expect("Missing API key");
    let client = Client::new(&api_key);

    let args = ChatArguments::new("gpt-4", vec![
        Message { role: "user".into(), content: "Hello!".into() }
    ]);

    let response = client.create_chat(args).await.unwrap();
    println!("{}", response);
}
```

**Cargo.toml:**
```toml
[dependencies]
openai-rust = "1"
tokio = { version = "1", features = ["full"] }
```

### openai-api-rs
Alternative OpenAI client with similar functionality.

**Features:**
- Chat completions, completions, embeddings
- Image generation (DALL-E)
- Audio transcription/translation (Whisper)
- Streaming support
- Custom base URL (for OpenRouter, Azure, etc.)

### Using with OpenRouter

OpenRouter provides a unified proxy for multiple providers with an OpenAI-compatible API:

```rust
// Configure the client with OpenRouter's base URL
let client = Client::new_with_base_url(
    &std::env::var("OPENROUTER_API_KEY").unwrap(),
    "https://openrouter.ai/api/v1"
);

// Use any model available on OpenRouter
let args = ChatArguments::new("anthropic/claude-3-opus", messages);
```

## Anthropic SDK

### anthropic-rs
Unofficial but actively maintained SDK for Claude models.

```rust
use anthropic::{Client, AnthropicConfig, CompleteRequestBuilder, HUMAN_PROMPT, AI_PROMPT};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    let cfg = AnthropicConfig::new()?;
    let client = Client::try_from(cfg)?;

    let request = CompleteRequestBuilder::default()
        .model("claude-3-opus-20240229".to_string())
        .prompt(format!("{HUMAN_PROMPT}What is Rust?{AI_PROMPT}"))
        .max_tokens_to_sample(1000)
        .stream_response(false)
        .build()?;

    let response = client.complete(request).await?;
    println!("{}", response.completion);
    Ok(())
}
```

**Features:**
- Async completion API
- Streaming support
- Environment-based configuration
- Claude-specific prompt formatting helpers

**Cargo.toml:**
```toml
[dependencies]
anthropic = "0.1"
dotenv = "0.15"
anyhow = "1"
tokio = { version = "1", features = ["full"] }
```

## Ollama SDK

### ollama-rs
Client for the Ollama local LLM server.

```rust
use ollama_rs::{Ollama, GenerationRequest};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let ollama = Ollama::default(); // localhost:11434

    let model = "llama2:latest".to_string();
    let prompt = "Why is the sky blue?".to_string();

    let result = ollama.generate(GenerationRequest::new(model, prompt)).await?;
    println!("{}", result.response);
    Ok(())
}
```

**Features:**
- Text generation with streaming (`generate_stream`)
- Chat conversations
- Model management (list, pull, create, delete)
- Custom endpoint configuration
- Embeddings generation

**Streaming example:**
```rust
use ollama_rs::{Ollama, GenerationRequest};
use futures::StreamExt;

let ollama = Ollama::default();
let mut stream = ollama.generate_stream(
    GenerationRequest::new("llama2".into(), "Tell me a story".into())
).await?;

while let Some(chunk) = stream.next().await {
    print!("{}", chunk?.response);
}
```

**Cargo.toml:**
```toml
[dependencies]
ollama-rs = { version = "0.2", features = ["stream"] }
tokio = { version = "1", features = ["full"] }
futures = "0.3"
```

## Comparison Table

| Crate | Provider | Streaming | Embeddings | Tool Calling | Maturity |
|:------|:---------|:----------|:-----------|:-------------|:---------|
| `async-openai` | OpenAI | Yes | Yes | Yes | High |
| `openai-rust` | OpenAI | Yes | Yes | Yes | High |
| `anthropic-rs` | Anthropic | Yes | No | Partial | Medium |
| `ollama-rs` | Ollama (local) | Yes | Yes | No | Medium |

## When to Use Provider SDKs

**Advantages:**
- Full access to provider-specific features
- Typically more up-to-date with API changes
- Lower abstraction overhead
- Better TypeScript/IDE support for specific APIs

**Disadvantages:**
- Vendor lock-in
- Different APIs for each provider
- More work to switch providers

**Recommendation:** Start with provider SDKs for simple applications or when you need cutting-edge features. Move to unified SDKs when you need multi-provider support.
