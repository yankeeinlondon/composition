---
name: rust-ai
description: Comprehensive guide to Rust libraries and frameworks for LLM and AI integration
created: 2025-12-08
hash: 4c6f6791645b55af
tags:
  - rust
  - llm
  - ai
  - machine-learning
  - openai
  - anthropic
  - rag
  - inference
---

# Rust AI: Libraries and Frameworks for LLM Integration

Rust is emerging as a compelling language for building LLM-powered applications, offering performance, reliability, and memory safety. This guide provides a comprehensive overview of the Rust ecosystem for AI and LLM development, from provider-specific SDKs to high-level orchestration frameworks.

## Table of Contents

- [Rust AI: Libraries and Frameworks for LLM Integration](#rust-ai-libraries-and-frameworks-for-llm-integration)
  - [Table of Contents](#table-of-contents)
  - [Overview](#overview)
    - [Choosing the Right Approach](#choosing-the-right-approach)
  - [Provider-Specific SDKs](#provider-specific-sdks)
    - [OpenAI](#openai)
    - [Anthropic](#anthropic)
    - [Ollama](#ollama)
  - [Multi-Provider SDKs](#multi-provider-sdks)
    - [The llm Crate](#the-llm-crate)
    - [RLLM](#rllm)
    - [genai](#genai)
  - [High-Level Orchestration Frameworks](#high-level-orchestration-frameworks)
    - [Rig](#rig)
    - [llm-chain](#llm-chain)
    - [LangChain-rust](#langchain-rust)
    - [Anchor-Chain](#anchor-chain)
    - [Kalosm](#kalosm)
    - [rustchain-community](#rustchain-community)
  - [Local Inference Engines](#local-inference-engines)
    - [Candle](#candle)
    - [Mistral.rs](#mistralrs)
    - [llama.cpp Bindings](#llamacpp-bindings)
  - [Combining Solutions](#combining-solutions)
    - [RAG Pipelines](#rag-pipelines)
    - [Agent Orchestration](#agent-orchestration)
    - [Fully Local Deployments](#fully-local-deployments)
    - [Hybrid Cloud-Local Systems](#hybrid-cloud-local-systems)
  - [Quick Reference](#quick-reference)
    - [Crate Comparison](#crate-comparison)
    - [Feature Matrix by Use Case](#feature-matrix-by-use-case)
  - [Resources](#resources)
    - [Official Documentation](#official-documentation)
    - [Crates.io](#cratesio)
    - [GitHub Repositories](#github-repositories)
    - [Guides and Tutorials](#guides-and-tutorials)

## Overview

The Rust LLM ecosystem spans several categories:

| Category | Purpose | Key Crates |
|----------|---------|------------|
| **Provider SDKs** | Direct API access to specific providers | `async-openai`, `anthropic-rs`, `ollama-rs` |
| **Multi-Provider SDKs** | Unified interface across providers | `llm`, `rllm`, `genai` |
| **Orchestration Frameworks** | Chains, agents, RAG, workflows | `rig`, `llm-chain`, `langchain-rust` |
| **Inference Engines** | Local model execution | `candle`, `mistral.rs`, `llama_cpp` |

### Choosing the Right Approach

- **Rapid prototyping with cloud APIs**: Use multi-provider SDKs like `llm` or `rllm`
- **Production RAG systems**: Use `rig` with vector store integrations
- **Complex multi-step workflows**: Use `llm-chain` or `langchain-rust`
- **Local/private inference**: Use `mistral.rs`, `candle`, or `llama_cpp`
- **Multimodal AI (text, audio, vision)**: Use `kalosm` or `mistral.rs`

## Provider-Specific SDKs

### OpenAI

Rust has several unofficial SDKs for OpenAI's API, providing async, strongly-typed interfaces for chat completions, embeddings, and more.

**Key crates:**

- `async-openai` - Full-featured async client based on OpenAPI spec
- `openai-rust` - Simple, ergonomic client
- `openai-api-rs` - Alternative OpenAI client

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

**OpenRouter Tip:** OpenRouter provides a unified proxy for many LLM providers with an OpenAI-compatible API. Configure any OpenAI SDK to use `https://openrouter.ai/api/v1` as the base URL with your OpenRouter API key.

### Anthropic

The `anthropic-rs` crate provides an async client for Claude models with streaming support.

```rust
use anthropic::{Client, AnthropicConfig, CompleteRequestBuilder, HUMAN_PROMPT, AI_PROMPT};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    let cfg = AnthropicConfig::new()?;
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

### Ollama

`ollama-rs` provides a Rust client for local LLM servers running Ollama. It supports text generation, streaming, and model management.

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

**Features:**

- Streaming with `generate_stream` (requires `stream` feature)
- Model management: `list_local_models()`, model info, model creation

## Multi-Provider SDKs

### The llm Crate

The `llm` crate (v1.0.7+) provides a unified interface for multiple LLM providers including OpenAI, Anthropic, Ollama, DeepSeek, xAI, Phind, Groq, and Google.

**Functional footprint:**

- Chat-based interactions and streaming
- Text completion and embeddings
- Tool calling
- Request validation and retry logic
- Agent building modules
- Conversation history management

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

### RLLM

RLLM provides a builder-based interface supporting OpenAI, Anthropic, Ollama, and more through feature flags.

```rust
use rllm::{LLMBuilder, backend::LLMBackend};

#[tokio::main]
async fn main() -> rllm::Result<()> {
    let llm = LLMBuilder::new()
        .backend(LLMBackend::OpenAI)
        .api_key(std::env::var("OPENAI_API_KEY")?)
        .model("gpt-3.5-turbo")
        .build()?;

    let reply = llm.chat(["Hello, how are you?"]).await?;
    println!("Model reply: {}", reply);
    Ok(())
}
```

Switching providers is as simple as changing `LLMBackend::OpenAI` to `LLMBackend::Anthropic`.

### genai

The `genai` crate is an experimental multi-provider SDK covering OpenAI, Anthropic, Google PaLM, Cohere, and others with an ergonomic API.

## High-Level Orchestration Frameworks

### Rig

Rig is a comprehensive framework for building LLM-powered applications with a focus on RAG systems and type-safe development.

**Key features:**

- Unified API across providers (OpenAI, Cohere, etc.)
- Built-in RAG support with embedding and vector store abstractions
- Type-safe extraction using Rust's type system
- Tool calling support
- Builder pattern for intuitive configuration

**Basic completion:**

```rust
use rig::{completion::Prompt, providers::openai};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let openai_client = openai::Client::from_env();
    let gpt4 = openai_client.model("gpt-4").build();

    let response = gpt4.prompt("Explain quantum computing in one sentence.").await?;
    println!("GPT-4: {}", response);
    Ok(())
}
```

**RAG agent:**

```rust
use rig::{
    completion::Prompt,
    embeddings::EmbeddingsBuilder,
    providers::openai::Client,
    vector_store::{in_memory_store::InMemoryVectorStore, VectorStore},
};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let openai_client = Client::from_env();
    let embedding_model = openai_client.embedding_model("text-embedding-ada-002");

    let mut vector_store = InMemoryVectorStore::default();

    let embeddings = EmbeddingsBuilder::new(embedding_model.clone())
        .simple_document("doc1", "Rig is a Rust library for building LLM applications.")
        .simple_document("doc2", "Rig supports OpenAI and Cohere as LLM providers.")
        .build()
        .await?;
    vector_store.add_documents(embeddings).await?;

    let rag_agent = openai_client.context_rag_agent("gpt-4")
        .preamble("You are an assistant that answers questions about Rig.")
        .dynamic_context(1, vector_store.index(embedding_model))
        .build();

    let response = rag_agent.prompt("What is Rig?").await?;
    println!("RAG Agent: {}", response);
    Ok(())
}
```

**Type-safe extraction:**

```rust
use serde::Deserialize;
use rig::providers::openai::Client;

#[derive(Deserialize, rig::JsonSchema)]
struct Person {
    name: String,
    age: u8,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let openai_client = Client::from_env();
    let extractor = openai_client.extractor::<Person>("gpt-4").build();

    let person: Person = extractor.extract("John Doe is 30 years old").await?;
    println!("{:?}", person);
    Ok(())
}
```

### llm-chain

`llm-chain` provides a framework for building sequential chains of LLM operations, inspired by Python's LangChain.

**Key features:**

- Sequential chains where output flows to input
- Template-based prompts with parameter substitution
- Map-reduce patterns for parallel processing
- Multiple driver support (OpenAI, local LLAMA)

```rust
use llm_chain::parameters;
use llm_chain::step::Step;
use llm_chain::traits::Executor as ExecutorTrait;
use llm_chain::{chains::sequential::Chain, prompt};
use llm_chain_openai::chatgpt::Executor;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let exec = Executor::new()?;

    let chain: Chain = Chain::new(vec![
        Step::for_prompt_template(
            prompt!("You are a bot for travel assistance research",
                "Find good places to visit in {{city}}, {{country}}.")
        ),
        Step::for_prompt_template(
            prompt!("You are a social media assistant",
                "Format into 5 bullet points:\n--\n{{text}}")
        ),
        Step::for_prompt_template(
            prompt!("You are a social media assistant",
                "Create a LinkedIn post with emojis:\n--\n{{text}}")
        )
    ]);

    let result = chain
        .run(parameters!("city" => "Rome", "country" => "Italy"), &exec)
        .await?;

    println!("{}", result.to_immediate().await?.as_content());
    Ok(())
}
```

### LangChain-rust

A community port of LangChain concepts to Rust, providing composable primitives for AI applications.

**Key features:**

- PromptTemplate and Chain abstractions
- Tool abstractions and agent logic
- Memory interfaces for chat history
- Vector store integrations (Qdrant, SQLite, SurrealDB)
- Support for OpenAI, Azure OpenAI, Ollama, Claude

```rust
use langchain_rust::llm::OpenAI;
use langchain_rust::prompt::PromptTemplate;
use langchain_rust::chain::LLMChain;

let llm = OpenAI::new().with_api_key("...").model("gpt-3.5-turbo");
let template = PromptTemplate::new("Translate to French: {input}")?;
let chain = LLMChain::new(llm, template);

let result = chain.run(&serde_json::json!({"input": "Hello, world!"}))?;
println!("{}", result);
```

### Anchor-Chain

An async framework for LLM workflows emphasizing static typing and compile-time error checking.

**Key features:**

- YAML or Rust-based workflow definition
- Statically-typed tool input/output schemas
- Multi-step workflow orchestration

### Kalosm

A "local first AI meta-framework" built on Candle, supporting language, audio, and vision models.

**Key features:**

- Local text generation with streaming
- Structured generation (parse LLM output into Rust types)
- Context gathering from RSS, websites, files, search
- Embedding-powered search with SurrealDB integration
- Audio transcription (Whisper)
- Image generation (Wuerstchen) and segmentation (Segment Anything)

**Basic text generation:**

```rust
use kalosm::language::*;

#[tokio::main]
async fn main() {
    let mut llm = Llama::new().await.unwrap();
    let prompt = "The following is a 300 word essay about Paris:";
    print!("{}", prompt);

    let mut stream = llm(prompt);
    stream.to_std_out().await.unwrap();
}
```

**Structured generation:**

```rust
use kalosm::language::*;
use std::sync::Arc;

#[derive(Parse, Clone, Debug)]
enum Class { Thing, Person, Animal }

#[derive(Parse, Clone, Debug)]
struct Response { classification: Class }

#[tokio::main]
async fn main() {
    let llm = Llama::new_chat().await.unwrap();
    let task = llm.task("Classify as person, animal, or thing in JSON format")
        .with_constraints(Arc::new(Response::new_parser()));

    let response = task("The Kalosm library").await.unwrap();
    println!("{:?}", response);
}
```

**RAG with SurrealDB:**

```rust
use kalosm::language::*;
use surrealdb::{engine::local::SurrealKv, Surreal};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let db = Surreal::new::<SurrealKv>("./db/temp.db").await?;
    db.use_ns("test").use_db("test").await?;

    let document_table = db
        .document_table_builder("documents")
        .at("./db/embeddings.db")
        .build::<Document>()
        .await?;

    // Add documents...
    document_table.add_context(urls).await?;

    let model = Llama::new_chat().await?;
    let mut chat = model.chat()
        .with_system_prompt("Answer based on provided context.");

    // Search and respond...
    let context = document_table.search(&question).with_results(1).await?;
    // Use context in prompt...
    Ok(())
}
```

### rustchain-community

A memory-safe AI agent framework with workflow transpilation capabilities.

**Key features:**

- YAML-based workflow (mission) definition
- Core agent orchestration
- Performance optimization
- Transpilation between workflow formats (LangChain, Kubernetes configs)

```yaml
version: '1.0'
name: document_summary_mission
steps:
  - id: load_document
    step_type: command
    parameters:
      command: cat document.txt
      output_key: document_content
  - id: summarize
    step_type: llm_completion
    parameters:
      model: gpt-3.5-turbo
      prompt: "Summarize: {{document_content}}"
      output_key: summary_result
```

## Local Inference Engines

### Candle

Candle is Hugging Face's minimalist ML framework for Rust, providing the foundation for many higher-level libraries.

**Key features:**

- Tensor operations and neural network modules (`candle-nn`)
- GPU support (CUDA with FlashAttention, cuDNN)
- WebAssembly support for browser deployment
- Implementations of LLaMA, Falcon, Phi, Mistral, Stable Diffusion, Whisper, and more
- PyTorch-like API

```rust
use candle_core::{Device, Tensor};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let device = Device::Cpu;

    let a = Tensor::randn(0f32, 1., (2, 3), &device)?;
    let b = Tensor::randn(0f32, 1., (3, 4), &device)?;
    let c = a.matmul(&b)?;

    println!("{c}");
    Ok(())
}
```

For LLM inference, use `candle-transformers` which provides higher-level model implementations.

### Mistral.rs

A high-performance LLM inference engine with extensive model support.

**Key features:**

- Text models: Llama, Mistral, Mixtral, Gemma, Qwen, Phi
- Vision models: LLaVA, Qwen-VL, Phi-3V
- Speech models: Dia
- Image generation: FLUX
- **MCP (Model Context Protocol) Client** for external tool integration
- Advanced quantization: ISQ, PagedAttention, FlashAttention
- Per-layer topology optimization
- Multiple quantization formats: GGML/GGUF, GPTQ, AWQ, FP8, BNB
- LoRA/X-LoRA adapters
- AnyMoE (Mixture of Experts on any base model)
- Rust, Python, and OpenAI-compatible HTTP APIs

**MCP configuration for tool use:**

```json
{
  "servers": [{
    "name": "Filesystem Tools",
    "source": {
      "type": "Process",
      "command": "npx",
      "args": ["@modelcontextprotocol/server-filesystem", "/tmp", "-y"]
    }
  }],
  "auto_register_tools": true
}
```

Run with: `./mistralrs-server --mcp-config mcp-config.json --port 1234 run -m Qwen/Qwen3-4B`

### llama.cpp Bindings

Rust bindings to the optimized llama.cpp C library for efficient local inference.

**Key crates:**

- `llama_cpp` - High-level, safe bindings
- `llama-cpp-2` - Lower-level bindings closer to C API
- `llm_client` - High-level wrapper with model downloading
- `drama_llama` - Ergonomic wrapper for chat-style completions

```rust
use llama_cpp::{LlamaModel, LlamaParams, SessionParams};
use llama_cpp::standard_sampler::StandardSampler;
use std::io::{self, Write};

fn main() -> anyhow::Result<()> {
    let model = LlamaModel::load_from_file(
        "models/7B/ggml-model.gguf",
        LlamaParams::default()
    ).expect("Could not load model");

    let mut session = model.create_session(SessionParams::default())
        .expect("Failed to create session");

    session.advance_context("This is the story of a curious Rust programmer.").unwrap();

    let mut outputs = session
        .start_completing_with(StandardSampler::default(), 512)
        .into_strings();

    while let Some(token) = outputs.next() {
        print!("{}", token);
        io::stdout().flush().unwrap();
    }
    Ok(())
}
```

**Simplified approach with llm_client:**

```rust
// Downloads Mistral-7B automatically if needed
let client = LlmClient::llama_cpp()
    .mistral7b_instruct_v0_3()
    .init()
    .await?;

let response = client.ask("What is Rust?").await?;
```

## Combining Solutions

No single crate does everything. The most powerful applications combine multiple libraries.

### RAG Pipelines

**Goal:** High-performance, memory-safe Retrieval-Augmented Generation

**Recommended combination:** `rig` + `llm`

- **rig** provides the superior framework for vector store integration, embeddings, and RAG workflow architecture
- **llm** can serve as an underlying `CompletionModel` implementation, allowing easy provider switching without changing RAG logic

**Alternative:** `llm-chain` + `llm-chain-hnsw` + `qdrant-client`

Encode documents, store in Qdrant, retrieve at query time, inject into prompts.

### Agent Orchestration

**Goal:** Complex multi-step agentic workflows

**Recommended combination:** `rustchain-community` + `llm`

- **rustchain-community** structures high-level pipeline logic via YAML missions
- **llm** handles the actual LLM/tool calling within mission steps

**Alternative:** `langchain-rust` + `rllm`

- **langchain-rust** manages agent planning/execution and tool usage
- **rllm** provides flexible provider switching

### Fully Local Deployments

**Goal:** No external API calls, all inference on local hardware

**Recommended combination:** `anchor-chain` or `llm-chain` + `llama_cpp` or `llm_client`

- Use the orchestration crate for workflow logic
- Use llama.cpp bindings for in-process inference
- Write an adapter so chain "LLM calls" use the local model

Ideal for offline applications, edge deployments, or IoT devices with smaller models.

### Hybrid Cloud-Local Systems

**Goal:** Route queries to appropriate backends based on complexity

**Recommended combination:** `ollama-rs` + `async-openai` + custom routing logic

- Use Ollama for quick, domain-specific local queries
- Fall back to cloud APIs (OpenAI, Claude) for complex tasks
- Orchestrate decision logic in Rust

## Quick Reference

### Crate Comparison

| Crate | Type | Local | Cloud | RAG | Agents | Streaming |
|-------|------|-------|-------|-----|--------|-----------|
| `llm` | SDK | Ollama | Yes | - | Basic | Yes |
| `rig` | Framework | - | Yes | Yes | Yes | Yes |
| `llm-chain` | Framework | Yes | Yes | Yes | - | Yes |
| `langchain-rust` | Framework | Yes | Yes | Yes | Yes | Yes |
| `kalosm` | Framework | Yes | - | Yes | - | Yes |
| `mistral.rs` | Engine | Yes | - | - | MCP | Yes |
| `candle` | Engine | Yes | - | - | - | - |
| `llama_cpp` | Bindings | Yes | - | - | - | Yes |

### Feature Matrix by Use Case

| Use Case | Recommended Crate(s) |
|----------|---------------------|
| Quick cloud API access | `llm`, `async-openai` |
| Production RAG | `rig` |
| Complex chains | `llm-chain`, `langchain-rust` |
| Local inference (performance) | `mistral.rs` |
| Local inference (flexibility) | `candle` |
| Multimodal (text+audio+vision) | `kalosm`, `mistral.rs` |
| YAML-defined workflows | `rustchain-community` |
| Tool-using agents | `mistral.rs` (MCP), `langchain-rust` |

## Resources

### Official Documentation

- [Rig](https://rig.rs/) - Official website and [documentation](https://docs.rig.rs/)
- [Candle](https://github.com/huggingface/candle) - Hugging Face's ML framework
- [Mistral.rs](https://github.com/EricLBuehler/mistral.rs) - High-performance inference
- [Kalosm](https://docs.rs/kalosm) - Local-first AI meta-framework

### Crates.io

- [llm](https://crates.io/crates/llm) - Multi-provider SDK
- [rllm](https://crates.io/crates/rllm) - Unified LLM interface
- [llm-chain](https://crates.io/crates/llm-chain) - Chain orchestration
- [langchain-rust](https://crates.io/crates/langchain-rust) - LangChain port
- [ollama-rs](https://crates.io/crates/ollama-rs) - Ollama client
- [async-openai](https://crates.io/crates/async-openai) - OpenAI client
- [anthropic](https://crates.io/crates/anthropic) - Anthropic client
- [llama_cpp](https://crates.io/crates/llama_cpp) - llama.cpp bindings
- [candle-core](https://crates.io/crates/candle-core) - Candle ML framework
- [candle-transformers](https://crates.io/crates/candle-transformers) - Transformer utilities

### GitHub Repositories

- [0xPlaygrounds/rig](https://github.com/0xPlaygrounds/rig)
- [sobelio/llm-chain](https://github.com/sobelio/llm-chain)
- [Abraxas-365/langchain-rust](https://github.com/Abraxas-365/langchain-rust)
- [pepperoni21/ollama-rs](https://github.com/pepperoni21/ollama-rs)
- [graniet/rllm](https://github.com/graniet/rllm)
- [AbdelStark/anthropic-rs](https://github.com/AbdelStark/anthropic-rs)
- [emersonmde/anchor-chain](https://github.com/emersonmde/anchor-chain)
- [edgenai/llama_cpp-rs](https://github.com/edgenai/llama_cpp-rs)

### Guides and Tutorials

- [Rust Ecosystem for AI & LLMs](https://hackmd.io/@Hamze/Hy5LiRV1gg) - Comprehensive HackMD guide
- [Shuttle.dev llm-chain guide](https://www.shuttle.dev/blog/2024/06/06/llm-chain-langchain-rust)
- [OpenRouter](https://openrouter.ai/docs/quickstart) - Unified LLM proxy documentation
