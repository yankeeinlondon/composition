---
name: rust-ai
description: Rust libraries and frameworks for LLM integration - provider SDKs, unified interfaces, orchestration frameworks, and local inference engines
hash: "49f5bcd134defefd"
---

# Rust AI/LLM Integration

Comprehensive guide to Rust's ecosystem for Large Language Model integration, covering provider-specific SDKs, multi-provider unified interfaces, orchestration frameworks, and local inference engines.

## Quick Reference

| Category | Libraries | Use Case |
|----------|-----------|----------|
| Provider SDKs | `async-openai`, `anthropic-rs`, `ollama-rs` | Direct API access to specific providers |
| Unified SDKs | `llm`, `rllm`, `genai`, `allms` | Single interface for multiple providers |
| Orchestration | `rig`, `llm-chain`, `kalosm`, `langchain-rust` | Complex workflows, RAG, agents |
| Local Inference | `candle`, `mistral.rs`, `llama_cpp` | On-device model execution |

## Provider-Specific SDKs

### OpenAI
- **`async-openai`**: Full-featured async client based on OpenAPI spec
- **`openai-api-rs`**: Alternative client library
- Supports chat completions, embeddings, images, streaming

### Anthropic (Claude)
- **`anthropic-rs`**: Unofficial SDK with async support
- Uses `HUMAN_PROMPT` and `AI_PROMPT` delimiters
- Supports streaming responses

### Ollama (Local Models)
- **`ollama-rs`**: Client for local Ollama server (localhost:11434)
- Supports model management, streaming, and local LLM inference

See [provider-sdks.md](./provider-sdks.md) for detailed examples.

## Multi-Provider Unified SDKs

### llm Crate (RLLM)
The `llm` crate provides a unified interface across providers:
- Supports: OpenAI, Anthropic, Ollama, Google, DeepSeek, Groq, xAI
- Features: Chat, streaming, embeddings, tool calls, prompt chains
- Feature flags for backend selection

### genai
Experimental multi-provider SDK covering OpenAI, Anthropic, Google PaLM, Cohere.

See [unified-sdks.md](./unified-sdks.md) for usage patterns.

## High-Level Orchestration Frameworks

### Rig
Modular framework for LLM-powered applications:
- Unified `CompletionModel` and `EmbeddingModel` traits
- Built-in RAG support with vector store integration (Qdrant, MongoDB, LanceDB)
- Type-safe structured data extraction
- Tool calling and agent abstractions

### llm-chain
LangChain-inspired chain orchestration:
- Sequential and map-reduce chains
- Template-based prompts with parameter substitution
- OpenAI and local LLAMA drivers
- Multi-step workflow composition

### Kalosm
Local-first AI meta-framework built on Candle:
- Supports Llama, Mistral, Phi, Zephyr quantized models
- Multimodal: text, audio (Whisper), image (Segment Anything)
- Structured generation with custom parsers
- Integrated vector search with SurrealDB

### LangChain-rust
Community port of LangChain concepts:
- PromptTemplate, Chain, Memory abstractions
- Tool and agent definitions
- Vector store integrations (Qdrant, SQLite, SurrealDB)

See [orchestration-frameworks.md](./orchestration-frameworks.md) for detailed examples.

## Local Inference Engines

### Candle (Hugging Face)
Minimalist ML framework for Rust:
- GPU support (CUDA, Metal), WebAssembly
- Implements LLaMA, Mistral, Phi, Stable Diffusion, Whisper
- PyTorch-like API for tensor operations
- Foundation for Kalosm and other frameworks

### Mistral.rs
High-performance LLM inference engine:
- Text, vision, speech, and image generation models
- ISQ (In-Place Quantization), PagedAttention, FlashAttention
- Per-layer topology optimization for memory/speed tuning
- MCP (Model Context Protocol) for external tool integration
- OpenAI-compatible HTTP server

### llama.cpp Bindings
- **`llama_cpp`**: High-level safe bindings
- **`llama-cpp-2`**: Low-level raw bindings
- **`llm_client`**: High-level wrapper with model downloading
- Optimized CPU inference for GGUF/GGML models

See [local-inference.md](./local-inference.md) for setup and examples.

## Composable Solutions

### RAG Pipeline
Combine `rig` (RAG tooling) + `llm` (multi-provider access):
- Use rig for vector store integration and embedding management
- Use llm for flexible provider switching within rig's workflow

### Agent Orchestration
Combine `langchain-rust` (agent logic) + `llm` (provider abstraction):
- LangChain-rust manages planning, execution, and tool use
- llm provides unified interface for multiple LLM backends

### Fully Local Stack
Combine `llm-chain` or `kalosm` (orchestration) + `llama_cpp` (inference):
- Define prompting logic with high-level framework
- Execute with local llama.cpp for offline operation

### Hybrid Cloud-Local
- Use `ollama-rs` for quick local classifications
- Fall back to `async-openai` for complex queries
- Route based on query complexity

See [combining-solutions.md](./combining-solutions.md) for architecture patterns.

## Decision Guide

**New async application with multiple providers**: `llm` crate or `rig`

**RAG system with vector stores**: `rig` with companion crates

**LangChain-style chains**: `llm-chain` or `langchain-rust`

**Local-first multimodal AI**: `kalosm`

**Maximum inference performance**: `mistral.rs` or `candle`

**Simple local LLM server**: `ollama-rs`

**Single provider integration**: Use provider-specific SDK directly

## Key Dependencies

Most crates require:
- `tokio` for async runtime
- `serde` for serialization
- API keys via environment variables (e.g., `OPENAI_API_KEY`)

For local inference:
- GGUF/GGML model files
- Optional: CUDA toolkit for GPU acceleration
