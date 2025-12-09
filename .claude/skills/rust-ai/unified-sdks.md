# Unified Multi-Provider SDKs

Libraries that provide a single interface across multiple LLM providers. Use these when you need provider flexibility or want to avoid vendor lock-in.

## llm (RLLM)

The primary unified SDK for Rust. Supports OpenAI, Anthropic, Ollama, DeepSeek, xAI, Phind, Groq, and Google.

**Cargo.toml:**
```toml
[dependencies]
llm = { version = "1.0", features = ["openai", "anthropic", "ollama"] }
tokio = { version = "1", features = ["full"] }
```

### Basic Chat

```rust
use llm::{LLMBuilder, backend::LLMBackend};

#[tokio::main]
async fn main() -> llm::Result<()> {
    let llm = LLMBuilder::new()
        .backend(LLMBackend::OpenAI)
        .api_key(std::env::var("OPENAI_API_KEY")?)
        .model("gpt-4")
        .build()?;

    let reply = llm.chat(["Hello, how are you?"]).await?;
    println!("Response: {}", reply);
    Ok(())
}
```

### Streaming

```rust
use llm::{LLMBuilder, backend::LLMBackend};
use futures::StreamExt;

let llm = LLMBuilder::new()
    .backend(LLMBackend::Anthropic)
    .api_key(std::env::var("ANTHROPIC_API_KEY")?)
    .model("claude-3-sonnet-20240229")
    .build()?;

let mut stream = llm.chat_stream(["Explain quantum computing"]).await?;
while let Some(chunk) = stream.next().await {
    print!("{}", chunk?);
}
```

### Tool Calling

```rust
use llm::{LLMBuilder, Tool, ToolParameter};

let weather_tool = Tool::new("get_weather")
    .description("Get current weather for a location")
    .parameter(ToolParameter::new("location", "string")
        .description("City name")
        .required(true));

let llm = LLMBuilder::new()
    .backend(LLMBackend::OpenAI)
    .api_key(api_key)
    .model("gpt-4")
    .tools(vec![weather_tool])
    .build()?;

let response = llm.chat(["What's the weather in Paris?"]).await?;
// Handle tool calls in response
```

### Embeddings

```rust
let llm = LLMBuilder::new()
    .backend(LLMBackend::OpenAI)
    .api_key(api_key)
    .model("text-embedding-ada-002")
    .build()?;

let embeddings = llm.embed(["Hello world", "Rust is great"]).await?;
```

### Available Backends

| Backend | Feature Flag | Models |
|:--------|:-------------|:-------|
| `LLMBackend::OpenAI` | `openai` | gpt-4, gpt-3.5-turbo, etc. |
| `LLMBackend::Anthropic` | `anthropic` | claude-3-opus, claude-3-sonnet, etc. |
| `LLMBackend::Ollama` | `ollama` | Any Ollama-served model |
| `LLMBackend::DeepSeek` | `deepseek` | deepseek-chat, deepseek-coder |
| `LLMBackend::Groq` | `groq` | llama2-70b, mixtral-8x7b |
| `LLMBackend::Google` | `google` | gemini-pro, gemini-ultra |

## genai (rust-genai)

Experimental multi-provider SDK with ergonomic API.

```rust
use genai::{Client, Model};

let client = Client::new()?;
let response = client
    .model(Model::OpenAI("gpt-4"))
    .chat("What is Rust?")
    .await?;
```

**Supported providers:** OpenAI, Anthropic, Google PaLM, Cohere

## allms

Type-safe interactions across providers with shared interface.

```rust
use allms::{Client, Provider};

let client = Client::new(Provider::OpenAI, api_key)?;
let response = client.complete("Hello").await?;

// Switch provider with same interface
let client = Client::new(Provider::Anthropic, different_key)?;
let response = client.complete("Hello").await?;
```

## llmclient

Rust client for Gemini, OpenAI, Anthropic, and Mistral.

## Comparison

| Crate | Providers | Streaming | Embeddings | Tool Calls | Agent Support |
|:------|:----------|:----------|:-----------|:-----------|:--------------|
| `llm` | 8+ | Yes | Yes | Yes | Yes (basic) |
| `genai` | 4 | Yes | Partial | Partial | No |
| `allms` | 4 | Yes | Yes | Partial | No |
| `llmclient` | 4 | Yes | Yes | No | No |

## Best Practices

### Feature Flag Optimization
Only enable the backends you need to minimize compile time and binary size:

```toml
# Only OpenAI and Anthropic
llm = { version = "1.0", default-features = false, features = ["openai", "anthropic"] }
```

### Environment-Based Provider Selection

```rust
use llm::{LLMBuilder, backend::LLMBackend};

let backend = match std::env::var("LLM_PROVIDER").as_deref() {
    Ok("openai") => LLMBackend::OpenAI,
    Ok("anthropic") => LLMBackend::Anthropic,
    Ok("ollama") => LLMBackend::Ollama,
    _ => LLMBackend::OpenAI, // default
};

let llm = LLMBuilder::new()
    .backend(backend)
    .api_key(std::env::var("LLM_API_KEY")?)
    .model(std::env::var("LLM_MODEL")?)
    .build()?;
```

### Fallback Pattern

```rust
async fn chat_with_fallback(prompt: &str) -> Result<String> {
    // Try primary provider
    if let Ok(response) = primary_llm.chat([prompt]).await {
        return Ok(response);
    }

    // Fallback to secondary
    fallback_llm.chat([prompt]).await
}
```

## When to Use Unified SDKs

**Advantages:**
- Single API for multiple providers
- Easy provider switching
- Reduced learning curve
- Good for A/B testing models

**Disadvantages:**
- May lag behind provider-specific features
- Abstraction adds some overhead
- Not all provider features exposed

**Recommendation:** Use `llm` crate for most multi-provider applications. It has the best coverage and most active development.
