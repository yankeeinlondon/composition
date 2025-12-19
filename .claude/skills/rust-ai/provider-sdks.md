# Provider-Specific SDKs

Detailed examples for direct integration with specific LLM providers. Use these when you need full control over a single provider's API.

## OpenAI with async-openai

Full-featured async client based on OpenAPI spec. The most complete Rust SDK for OpenAI.

### Chat Completion

```rust
use async_openai::{
    Client,
    types::{
        CreateChatCompletionRequestArgs,
        ChatCompletionRequestUserMessageArgs,
    },
};

pub async fn openai_chat(content: &str) -> Result<String, anyhow::Error> {
    let client = Client::new();  // Uses OPENAI_API_KEY

    let request = CreateChatCompletionRequestArgs::default()
        .model("gpt-4o")
        .messages([
            ChatCompletionRequestUserMessageArgs::default()
                .content(content)
                .build()?
                .into(),
        ])
        .build()?;

    let response = client.chat().create(request).await?;
    let content = response.choices[0].message.content.clone()
        .unwrap_or_default();

    Ok(content)
}
```

### Embeddings

```rust
use async_openai::{
    Client,
    types::CreateEmbeddingRequestArgs,
};

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

### Summarization Example

```rust
pub async fn openai_summarize(content: &str) -> Result<String, anyhow::Error> {
    let client = Client::new();

    let request = CreateChatCompletionRequestArgs::default()
        .model("gpt-4o")
        .messages([
            ChatCompletionRequestUserMessageArgs::default()
                .content(format!("Summarize the following content concisely:\n\n{}", content))
                .build()?
                .into(),
        ])
        .build()?;

    let response = client.chat().create(request).await?;
    let content = response.choices[0].message.content.clone()
        .unwrap_or_default();

    Ok(content)
}
```

## OpenRouter with async-openai

OpenRouter provides access to multiple providers through an OpenAI-compatible API. Use async-openai with a custom base URL:

```rust
use async_openai::{Client, config::OpenAIConfig};

pub fn openrouter_client() -> Client<OpenAIConfig> {
    let config = OpenAIConfig::new()
        .with_api_key(std::env::var("OPENROUTER_API_KEY").expect("OPENROUTER_API_KEY not set"))
        .with_api_base("https://openrouter.ai/api/v1");

    Client::with_config(config)
}

pub async fn openrouter_chat(model: &str, prompt: &str) -> Result<String, anyhow::Error> {
    let client = openrouter_client();

    let request = CreateChatCompletionRequestArgs::default()
        .model(model)  // e.g., "anthropic/claude-3.5-sonnet", "deepseek/deepseek-r1"
        .messages([
            ChatCompletionRequestUserMessageArgs::default()
                .content(prompt)
                .build()?
                .into(),
        ])
        .build()?;

    let response = client.chat().create(request).await?;
    Ok(response.choices[0].message.content.clone().unwrap_or_default())
}

// Example: Using Claude via OpenRouter
pub async fn claude_via_openrouter(prompt: &str) -> Result<String, anyhow::Error> {
    openrouter_chat("anthropic/claude-3.5-sonnet", prompt).await
}

// Example: Using DeepSeek via OpenRouter
pub async fn deepseek_via_openrouter(prompt: &str) -> Result<String, anyhow::Error> {
    openrouter_chat("deepseek/deepseek-r1", prompt).await
}
```

## OpenRouter with openrouter_api

Native OpenRouter crate for simpler API:

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
```

## Anthropic (Claude) with anthropic-rs

Unofficial SDK with async support and streaming:

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

## Ollama (Local LLM Server)

Client for local Ollama server running at localhost:11434.

### Basic Completion

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
```

### Streaming

```rust
use ollama_rs::{Ollama, generation::completion::GenerationRequest};
use futures::StreamExt;

pub async fn ollama_stream(prompt: &str) -> Result<(), anyhow::Error> {
    let ollama = Ollama::default();
    let request = GenerationRequest::new("llama3.2".into(), prompt.into());

    let mut stream = ollama.generate_stream(request).await?;
    while let Some(result) = stream.next().await {
        if let Ok(response) = result {
            print!("{}", response.response);
        }
    }
    Ok(())
}
```

### With Fallback to Cloud

```rust
pub async fn summarize_with_fallback(
    content: &str,
    local_model: &str,
) -> Result<String, anyhow::Error> {
    let ollama = OllamaClient::new(local_model);

    if ollama.is_available().await {
        ollama.summarize(content).await
    } else {
        // Fall back to cloud provider
        openai_summarize(content).await
    }
}
```

## Cargo.toml Dependencies

```toml
[dependencies]
# OpenAI (recommended)
async-openai = "0.28"

# Anthropic
anthropic = "0.1"

# Ollama
ollama-rs = { version = "0.2", features = ["stream"] }

# OpenRouter (native)
openrouter_api = "0.7"

# Common
tokio = { version = "1", features = ["full"] }
anyhow = "1"
futures = "0.3"  # For streaming
```

## Environment Variables

```bash
# OpenAI
export OPENAI_API_KEY="sk-..."

# Anthropic
export ANTHROPIC_API_KEY="sk-ant-..."

# OpenRouter
export OPENROUTER_API_KEY="sk-or-..."

# Ollama uses local server, no API key needed
```

## Provider Selection Guide

| Provider | Best For | Crate |
|----------|----------|-------|
| OpenAI | General use, embeddings | `async-openai` |
| Anthropic | Long context, safety | `anthropic-rs` or via OpenRouter |
| OpenRouter | Multi-provider access, cost optimization | `async-openai` + custom URL |
| Ollama | Local development, privacy | `ollama-rs` |
| DeepSeek | Cost-effective reasoning | Via OpenRouter or `llm` crate |

## Sources

- [async-openai](https://crates.io/crates/async-openai)
- [anthropic-rs](https://github.com/AbdelStark/anthropic-rs)
- [ollama-rs](https://github.com/pepperoni21/ollama-rs)
- [openrouter_api](https://crates.io/crates/openrouter_api)
- [OpenRouter API Docs](https://openrouter.ai/docs)
