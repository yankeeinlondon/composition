# Local Inference Engines

Libraries for running LLM inference on local hardware without external API calls.

## Candle (Hugging Face)

Minimalist ML framework for Rust, serving as foundation for many other libraries.

### Features
- GPU support: CUDA (FlashAttention, cuDNN), Metal
- WebAssembly support for browser deployment
- PyTorch-like tensor API
- Lightweight binaries for serverless

### Supported Models
- LLaMA v1, v2, v3
- Mistral, Mixtral
- Falcon
- Phi series
- Gemma
- Stable Diffusion
- Whisper

### Basic Tensor Operations

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

### Running LLM Examples

```bash
# Run LLaMA example
cargo run --example llama --release

# With CUDA
cargo run --example llama --release --features cuda
```

### Crates
- `candle-core`: Tensor operations
- `candle-nn`: Neural network modules
- `candle-transformers`: Transformer model utilities

## Mistral.rs

High-performance LLM inference engine with advanced optimizations.

### Features
- ISQ (In-Place Quantization)
- PagedAttention, FlashAttention
- Per-layer topology optimization
- MCP (Model Context Protocol) for external tools
- OpenAI-compatible HTTP server

### Supported Model Types
- **Text**: Llama, Mistral, Mixtral, Gemma, Qwen, Phi
- **Vision**: LLaVA, Qwen-VL, Phi-3V
- **Speech**: Dia
- **Image Generation**: FLUX

### Quantization Formats
- GGML/GGUF
- GPTQ, AWQ, AFQ
- HQQ, FP8, BNB

### CLI Usage

```bash
# Interactive chat with quantized model
./mistralrs-server -i --isq 8 run -m meta-llama/Llama-3.2-3B-Instruct

# With MCP tools
./mistralrs-server --mcp-config mcp-config.json --port 1234 run -m Qwen/Qwen3-4B
```

### MCP Configuration

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

### Per-Layer Topology Optimization
Fine-tune quantization and device placement per layer for:
- Memory optimization (fitting large models in limited VRAM)
- Speed optimization (GPU/CPU splitting)
- Multi-GPU distribution

## llama.cpp Bindings

Rust bindings to the popular C++ inference library.

### llama_cpp (High-Level)
Safe, high-level bindings with Rust-idiomatic API.

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

    let mut outputs = session.start_completing_with(StandardSampler::default(), 512).into_strings();
    while let Some(token) = outputs.next() {
        print!("{}", token);
        io::stdout().flush().unwrap();
    }
    Ok(())
}
```

### llama-cpp-2 (Low-Level)
Thin wrapper around raw C API for maximum control.

### llm_client (High-Level Wrapper)
Simplified interface with automatic model downloading.

```rust
// Downloads and initializes Mistral-7B automatically
let client = LlmClient::llama_cpp()
    .mistral7b_instruct_v0_3()
    .init()
    .await?;

let response = client.ask("What is Rust?").await?;
```

### drama_llama
High-level, Rust-idiomatic wrapper focusing on ease of use.

## Comparison

| Engine | Focus | GPU Support | Quantization | Ease of Use |
|--------|-------|-------------|--------------|-------------|
| Candle | ML framework | CUDA, Metal, WASM | Limited | Medium |
| Mistral.rs | Performance | CUDA | Extensive | High (CLI) |
| llama_cpp | llama.cpp binding | Via llama.cpp | GGUF/GGML | Medium |
| llm_client | Convenience | Via llama.cpp | GGUF | High |

## Model Files

### GGUF/GGML Format
- Quantized models optimized for CPU/GPU inference
- Download from Hugging Face (TheBloke, etc.)
- Sizes: Q4_K_M (balanced), Q5_K_M (higher quality), Q8_0 (near-original)

### Common Model Sources
- [TheBloke on Hugging Face](https://huggingface.co/TheBloke)
- [Ollama model library](https://ollama.ai/library)

## Performance Tips

1. **Compile in release mode**: `cargo run --release`
2. **Enable GPU features**: `--features cuda` or `--features metal`
3. **Use appropriate quantization**: Q4 for speed, Q8 for quality
4. **Batch prompts when possible**: Reduces per-request overhead

## Cargo.toml

```toml
[dependencies]
# Candle
candle-core = "0.4"
candle-nn = "0.4"
candle-transformers = "0.4"

# llama.cpp bindings
llama_cpp = "0.1"
# or
llama-cpp-2 = "0.1"

# High-level wrapper
llm_client = "0.1"

# Common
tokio = { version = "1", features = ["full"] }
anyhow = "1"
```

## Sources

- [Candle](https://github.com/huggingface/candle)
- [Mistral.rs](https://github.com/EricLBuehler/mistral.rs)
- [llama_cpp](https://crates.io/crates/llama_cpp)
- [llm_client](https://crates.io/crates/llm_client)
