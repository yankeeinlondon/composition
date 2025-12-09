# Local Inference Engines

Run LLMs locally without API calls. Essential for privacy, offline use, edge deployment, or cost optimization.

## Candle

Hugging Face's minimalist ML framework for Rust. The foundation for many other Rust AI libraries.

**Cargo.toml:**
```toml
[dependencies]
candle-core = "0.8"
candle-nn = "0.8"
candle-transformers = "0.8"
tokenizers = "0.20"

# Optional: GPU support
# candle-core = { version = "0.8", features = ["cuda"] }
```

### Basic Tensor Operations

```rust
use candle_core::{Device, Tensor};

fn main() -> candle_core::Result<()> {
    let device = Device::Cpu; // or Device::cuda_if_available(0)?

    let a = Tensor::randn(0f32, 1., (2, 3), &device)?;
    let b = Tensor::randn(0f32, 1., (3, 4), &device)?;
    let c = a.matmul(&b)?;

    println!("{c}");
    Ok(())
}
```

### Loading Pre-trained Models

Candle includes implementations for many model architectures:
- **LLMs:** LLaMA (v1, v2, v3), Mistral, Mixtral, Phi, Gemma, Falcon, Qwen
- **Vision:** Stable Diffusion, Segment Anything
- **Audio:** Whisper
- **Embeddings:** Various sentence transformers

### Features

| Feature | Description |
|:--------|:------------|
| `cuda` | NVIDIA GPU support with cuDNN |
| `metal` | Apple Silicon GPU support |
| `mkl` | Intel MKL for optimized CPU ops |
| `accelerate` | Apple Accelerate framework |

### Use Cases
- Building custom ML models in Rust
- Fine-tuning or modifying existing architectures
- Maximum control over inference pipeline
- WASM deployment for browser-based AI

## Mistral.rs

High-performance LLM inference engine with broad model support and advanced features.

**Installation:**
```bash
# Build from source for best performance
git clone https://github.com/EricLBuehler/mistral.rs
cd mistral.rs
cargo build --release --features cuda  # or metal, mkl
```

### Supported Models

| Category | Models |
|:---------|:-------|
| **Text** | Llama (1-3), Mistral, Mixtral, Gemma, Qwen, Phi, DeepSeek |
| **Vision** | LLaVA, Qwen-VL, Phi-3V, Idefics |
| **Audio** | Dia (speech) |
| **Image Gen** | FLUX |

### CLI Usage

```bash
# Interactive chat with quantized model
./mistralrs-server -i --isq Q4_0 run -m meta-llama/Llama-3.2-3B-Instruct

# Start OpenAI-compatible server
./mistralrs-server --port 8080 run -m mistralai/Mistral-7B-Instruct-v0.2
```

### Rust API

```rust
use mistralrs::{Runner, Which};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let runner = Runner::new(Which::Plain("meta-llama/Llama-3.2-3B".into())).await?;

    let response = runner.generate("Write a haiku about Rust:").await?;
    println!("{}", response);
    Ok(())
}
```

### Key Features

**Quantization:**
- In-place quantization (ISQ): Q4_0, Q4_1, Q5_0, Q5_1, Q8_0, Q8_1
- GGUF/GGML format support
- GPTQ, AWQ, HQQ formats
- Per-layer topology optimization

**Performance:**
- PagedAttention for efficient memory
- FlashAttention support
- Multi-GPU distribution
- Per-layer device placement

**MCP (Model Context Protocol):**
```json
{
  "servers": [{
    "name": "Filesystem",
    "source": {
      "type": "Process",
      "command": "npx",
      "args": ["@modelcontextprotocol/server-filesystem", "/tmp"]
    }
  }],
  "auto_register_tools": true
}
```

```bash
./mistralrs-server --mcp-config mcp-config.json run -m Qwen/Qwen3-4B
```

## llama.cpp Bindings

Rust wrappers around the highly optimized llama.cpp C++ library.

### llama_cpp (High-level)

```rust
use llama_cpp::{LlamaModel, LlamaParams, SessionParams};
use llama_cpp::standard_sampler::StandardSampler;

fn main() -> anyhow::Result<()> {
    let model = LlamaModel::load_from_file(
        "models/llama-7b.gguf",
        LlamaParams::default()
    )?;

    let mut session = model.create_session(SessionParams::default())?;
    session.advance_context("Once upon a time")?;

    let mut outputs = session
        .start_completing_with(StandardSampler::default(), 256)
        .into_strings();

    while let Some(token) = outputs.next() {
        print!("{}", token);
    }
    Ok(())
}
```

### llama-cpp-2 (Low-level)

Closer to the raw C API for maximum control:
```rust
use llama_cpp_2::llama_backend;

// Initialize backend
llama_backend::init()?;

// Direct access to llama.cpp functions
```

### llm_client (High-level Wrapper)

Simplest interface, handles model downloading:

```rust
use llm_client::LlmClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Automatically downloads and caches the model
    let client = LlmClient::llama_cpp()
        .mistral7b_instruct_v0_3()
        .init()
        .await?;

    let response = client.ask("What is Rust?").await?;
    println!("{}", response);
    Ok(())
}
```

## Kalosm

Local-first AI meta-framework built on Candle. Supports language, audio, and vision models.

```rust
use kalosm::language::*;

#[tokio::main]
async fn main() {
    let mut llm = Llama::phi_3().await.unwrap();

    let mut stream = llm("Explain quantum computing:");
    stream.to_std_out().await.unwrap();
}
```

See [Orchestration Frameworks](./orchestration.md) for full Kalosm coverage.

## Comparison

| Engine | Ease of Use | Performance | Model Support | GPU Support | WASM |
|:-------|:------------|:------------|:--------------|:------------|:-----|
| Candle | Medium | High | Many | CUDA, Metal | Yes |
| Mistral.rs | Easy | Very High | Many | CUDA, Metal | No |
| llama_cpp | Medium | Very High | GGUF only | CUDA, Metal, OpenCL | No |
| llm_client | Very Easy | High | Limited | Via llama.cpp | No |
| Kalosm | Easy | High | Quantized | Via Candle | No |

## Performance Tips

### Quantization
Use quantized models for faster inference and lower memory:
```bash
# 4-bit quantization typically offers best speed/quality tradeoff
./mistralrs-server --isq Q4_0 run -m meta-llama/Llama-3-8B
```

### Batch Size
Increase batch size for throughput (not latency):
```rust
let params = LlamaParams::default()
    .with_n_batch(512);
```

### GPU Selection
```rust
// Candle
let device = Device::cuda_if_available(0)?; // GPU 0

// Or specific GPU
let device = Device::new_cuda(1)?; // GPU 1
```

### Memory Management
For large models that don't fit in VRAM:
```bash
# Mistral.rs: split across GPU and CPU
./mistralrs-server --topology mixed run -m large-model
```

## When to Use Local Inference

**Advantages:**
- Privacy: data never leaves your machine
- Cost: no per-token API charges
- Latency: no network round-trip
- Offline: works without internet
- Customization: fine-tune or modify models

**Disadvantages:**
- Hardware requirements (GPU recommended)
- Model quality may be lower than frontier APIs
- More setup complexity
- You manage updates and security

**Recommendation:** Use Mistral.rs for production local inference, Candle for custom ML work, llm_client for quick prototyping.
