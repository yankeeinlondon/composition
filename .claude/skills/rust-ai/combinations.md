# Strategic Library Combinations

How to combine Rust LLM libraries for comprehensive solutions.

## Combination Patterns

### 1. Rig + Candle: Production RAG with Custom ML

**Use case:** RAG applications needing custom embedding models or preprocessing.

```
┌─────────────────────────────────────────────────┐
│                  Application                     │
├─────────────────────────────────────────────────┤
│  Rig (Orchestration)                            │
│  - Agent logic, RAG workflow                    │
│  - Provider abstraction (OpenAI, Cohere)        │
│  - Type-safe extraction                         │
├─────────────────────────────────────────────────┤
│  Candle (ML Backend)                            │
│  - Custom embeddings                            │
│  - Document preprocessing                       │
│  - Fine-tuned models                            │
└─────────────────────────────────────────────────┘
```

**Example:**
```rust
use rig::{completion::Prompt, providers::openai::Client};
use candle_core::{Device, Tensor};
use candle_transformers::models::bert::BertModel;

// Use Candle for custom document preprocessing
fn preprocess_with_candle(text: &str) -> Vec<f32> {
    // Custom embedding or feature extraction
    // ...
}

// Use Rig for the RAG workflow
async fn rag_query(query: &str) -> anyhow::Result<String> {
    let client = Client::from_env();
    let agent = client.context_rag_agent("gpt-4")
        .preamble("Answer using the context")
        .dynamic_context(3, custom_index)
        .build();

    agent.prompt(query).await
}
```

**Benefits:**
- Rig handles high-level orchestration
- Candle provides low-level ML control
- Custom preprocessing without external services

---

### 2. Rig + Mistral.rs: Cloud Orchestration + Local Inference

**Use case:** Hybrid cloud/local deployment with fallback.

```
┌─────────────────────────────────────────────────┐
│                  Application                     │
├─────────────────────────────────────────────────┤
│  Rig (Orchestration)                            │
│  - Workflow management                          │
│  - Agent definitions                            │
│  - Retry/fallback logic                         │
├──────────────────────┬──────────────────────────┤
│  OpenAI/Anthropic    │   Mistral.rs             │
│  (Cloud, Primary)    │   (Local, Fallback)      │
│  - High capability   │   - Privacy              │
│  - Cost per token    │   - No network           │
└──────────────────────┴──────────────────────────┘
```

**Example:**
```rust
use rig::{completion::Prompt, providers::openai};

struct HybridProvider {
    cloud: openai::Client,
    local_endpoint: String,  // Mistral.rs OpenAI-compatible API
}

impl HybridProvider {
    async fn chat(&self, prompt: &str) -> anyhow::Result<String> {
        // Try cloud first
        match self.cloud_chat(prompt).await {
            Ok(response) => Ok(response),
            Err(_) => {
                // Fallback to local Mistral.rs
                self.local_chat(prompt).await
            }
        }
    }

    async fn local_chat(&self, prompt: &str) -> anyhow::Result<String> {
        // Call Mistral.rs server at self.local_endpoint
        // Uses OpenAI-compatible API
        todo!()
    }
}
```

**Benefits:**
- Cost optimization (use local for simple queries)
- Privacy for sensitive data
- Resilience against API outages

---

### 3. Kalosm + External APIs: Local-First with Cloud Fallback

**Use case:** Privacy-focused applications with cloud enhancement.

```
┌─────────────────────────────────────────────────┐
│                  Application                     │
├─────────────────────────────────────────────────┤
│  Kalosm (Primary)                               │
│  - Local Llama/Mistral/Phi                      │
│  - Audio transcription (Whisper)                │
│  - Structured generation                        │
├─────────────────────────────────────────────────┤
│  llm crate (Fallback/Enhancement)               │
│  - Complex queries → GPT-4                      │
│  - Vision tasks → Claude                        │
└─────────────────────────────────────────────────┘
```

**Example:**
```rust
use kalosm::language::*;
use llm::{LLMBuilder, backend::LLMBackend};

async fn smart_query(prompt: &str, complexity: f32) -> anyhow::Result<String> {
    if complexity < 0.5 {
        // Simple query: use local model
        let mut llm = Llama::phi_3().await?;
        let mut stream = llm(prompt);
        stream.collect_string().await
    } else {
        // Complex query: use cloud
        let llm = LLMBuilder::new()
            .backend(LLMBackend::OpenAI)
            .api_key(std::env::var("OPENAI_API_KEY")?)
            .model("gpt-4")
            .build()?;

        llm.chat([prompt]).await
    }
}
```

---

### 4. llm-chain + llm crate: Chains with Provider Flexibility

**Use case:** Multi-step workflows that may need different models per step.

```
┌─────────────────────────────────────────────────┐
│  llm-chain (Workflow)                           │
├─────────────────────────────────────────────────┤
│  Step 1: Research     → llm(Anthropic/Claude)   │
│  Step 2: Summarize    → llm(OpenAI/GPT-3.5)     │
│  Step 3: Format       → llm(Local/Ollama)       │
└─────────────────────────────────────────────────┘
```

**Benefits:**
- Right model for each task
- Cost optimization (cheaper models for simple steps)
- Chain framework handles flow

---

### 5. Full RAG Stack: Rig + Vector DB + Local Embeddings

**Use case:** Production RAG with external vector database.

```
┌─────────────────────────────────────────────────┐
│  Application Layer                              │
├─────────────────────────────────────────────────┤
│  Rig (Orchestration)                            │
│  - RAG agent                                    │
│  - Query handling                               │
├──────────────────────┬──────────────────────────┤
│  Qdrant/Pinecone     │   OpenAI/Anthropic       │
│  (Vector Store)      │   (LLM Provider)         │
├──────────────────────┴──────────────────────────┤
│  Candle or llm crate (Embeddings)               │
│  - Local: Candle + sentence-transformers        │
│  - Cloud: OpenAI text-embedding-ada-002         │
└─────────────────────────────────────────────────┘
```

```toml
[dependencies]
rig-core = "0.5"
rig-qdrant = "0.5"  # Qdrant integration
candle-core = { version = "0.8", optional = true }
```

---

## Decision Matrix

| Goal | Primary | Secondary | Notes |
|:-----|:--------|:----------|:------|
| Simple API calls | `llm` | - | Unified interface sufficient |
| RAG application | `rig` | Vector DB crate | Best RAG abstractions |
| Multi-step chains | `llm-chain` | `llm` | Chain framework + provider flex |
| Local-first | `kalosm` | `llm` | Local primary, cloud fallback |
| Max performance | `mistral.rs` | - | Direct inference engine |
| Custom ML | `candle` | `rig` | Low-level control + orchestration |
| Edge deployment | `kalosm` or `llama_cpp` | - | Minimal dependencies |

## Integration Patterns

### Pattern 1: Adapter

Create a common trait and implement for each library:

```rust
trait LLMProvider: Send + Sync {
    async fn complete(&self, prompt: &str) -> Result<String>;
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;
}

struct RigProvider { client: rig::providers::openai::Client }
struct LLMCrateProvider { llm: llm::LLM }
struct KalosmProvider { model: kalosm::language::Llama }

// Implement LLMProvider for each
```

### Pattern 2: Service Layer

Wrap libraries behind a service abstraction:

```rust
struct AIService {
    orchestrator: Rig,
    inference: MistralRs,
    embeddings: Candle,
}

impl AIService {
    async fn query(&self, prompt: &str) -> Result<String> {
        // Decide which backend based on query type
    }

    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        // Use local Candle embeddings
    }
}
```

### Pattern 3: Configuration-Driven

Select libraries at runtime based on configuration:

```rust
#[derive(Deserialize)]
struct Config {
    llm_provider: String,  // "openai", "anthropic", "local"
    embedding_provider: String,
    use_local_fallback: bool,
}

fn create_llm(config: &Config) -> Box<dyn LLMProvider> {
    match config.llm_provider.as_str() {
        "openai" => Box::new(OpenAIProvider::new()),
        "local" => Box::new(KalosmProvider::new()),
        _ => panic!("Unknown provider"),
    }
}
```

## Best Practices

1. **Start simple**: Use `llm` crate for most applications
2. **Add complexity incrementally**: Don't over-engineer from the start
3. **Separate concerns**: Keep orchestration, inference, and storage modular
4. **Test combinations**: Integration tests for library interactions
5. **Monitor costs**: Track which providers are called and optimize
6. **Plan for failure**: Always have fallback strategies
7. **Version carefully**: LLM libraries evolve rapidly
