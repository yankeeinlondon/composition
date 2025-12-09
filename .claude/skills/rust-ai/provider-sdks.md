# Provider-Specific SDKs

Detailed examples for direct integration with specific LLM providers.

## OpenAI Crates

### async-openai
Full-featured async client based on OpenAPI spec.

```rust
use openai_rust::{Client, chat::{ChatArguments, Message}};

#[tokio::main]
async fn main() {
    let api_key = std::env::var("OPENAI_API_KEY").expect("Missing API key");
    let client = Client::new(&api_key);

    let args = ChatArguments::new("gpt-3.5-turbo", vec![
        Message { role: "user".into(), content: "Hello, GPT!".into() }
    ]);

    let response = client.create_chat(args).await.unwrap();
    println!("{}", response);
}
```

### OpenRouter Integration
Use OpenAI SDK with OpenRouter's unified proxy:

```rust
// Set base URL to OpenRouter endpoint
// https://openrouter.ai/api/v1
// Use OpenRouter API key instead of OpenAI key
```

OpenRouter provides access to multiple providers (OpenAI, Anthropic, etc.) through a single OpenAI-compatible API.

**Crate**: `openrouter_api` on crates.io

## Anthropic (Claude)

### anthropic-rs
Unofficial SDK with async support and streaming.

```rust
use anthropic::{Client, AnthropicConfig, CompleteRequestBuilder, HUMAN_PROMPT, AI_PROMPT};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    let cfg = AnthropicConfig::new()?;  // reads ANTHROPIC_API_KEY
    let client = Client::try_from(cfg)?;

    let request = CompleteRequestBuilder::default()
        .model("claude-instant-1".to_string())
        .prompt(format!("{HUMAN_PROMPT}How many toes do dogs have?{AI_PROMPT}"))
        .max_tokens_to_sample(1000)
        .stream_response(false)
        .build()?;

    let response = client.complete(request).await?;
    println!("Claude answered: {}", response.completion);
    Ok(())
}
```

## Ollama (Local LLM Server)

### ollama-rs
Client for local Ollama server running at localhost:11434.

```rust
use ollama_rs::{Ollama, GenerationRequest};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let ollama = Ollama::default();  // connects to localhost:11434

    let model = "llama2:latest".to_string();
    let prompt = "Why is the sky blue?".to_string();
    let result = ollama.generate(GenerationRequest::new(model, prompt)).await?;

    println!("{}", result.response);
    Ok(())
}
```

### Features
- Streaming with `generate_stream` (requires `stream` feature)
- Model management: `list_local_models()`, model info, create models
- Mirrors Ollama HTTP API

## Cargo.toml Dependencies

```toml
[dependencies]
# OpenAI
async-openai = "0.18"
# or
openai-api-rs = "4"

# Anthropic
anthropic = "0.1"

# Ollama
ollama-rs = "0.1"

# Common
tokio = { version = "1", features = ["full"] }
anyhow = "1"
dotenv = "0.15"
```

## Environment Variables

```bash
export OPENAI_API_KEY="sk-..."
export ANTHROPIC_API_KEY="sk-ant-..."
# Ollama uses local server, no API key needed
```

## Sources

- [async-openai](https://crates.io/crates/async-openai)
- [openai-api-rs](https://crates.io/crates/openai-api-rs)
- [anthropic-rs](https://github.com/AbdelStark/anthropic-rs)
- [ollama-rs](https://github.com/pepperoni21/ollama-rs)
- [OpenRouter](https://openrouter.ai)
