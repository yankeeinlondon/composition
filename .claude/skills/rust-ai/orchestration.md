# Orchestration Frameworks

Higher-level frameworks for building complex LLM applications: chains, agents, RAG, and workflows.

## Rig

Comprehensive framework for LLM-powered applications with excellent RAG support.

**Cargo.toml:**
```toml
[dependencies]
rig-core = "0.5"
tokio = { version = "1", features = ["full"] }
anyhow = "1"
```

### Basic Completion

```rust
use rig::{completion::Prompt, providers::openai};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let client = openai::Client::from_env();
    let gpt4 = client.model("gpt-4").build();

    let response = gpt4.prompt("Explain Rust in one sentence.").await?;
    println!("{}", response);
    Ok(())
}
```

### Embeddings

```rust
use rig::{
    embeddings::EmbeddingsBuilder,
    providers::openai::Client,
    vector_store::in_memory_store::InMemoryVectorStore,
};

let client = Client::from_env();
let embedding_model = client.embedding_model("text-embedding-ada-002");

let embeddings = EmbeddingsBuilder::new(embedding_model.clone())
    .simple_document("doc1", "Rig is a Rust LLM framework.")
    .simple_document("doc2", "Rust is a systems programming language.")
    .build()
    .await?;

let mut store = InMemoryVectorStore::default();
store.add_documents(embeddings).await?;
```

### RAG Agent

```rust
use rig::{
    completion::Prompt,
    embeddings::EmbeddingsBuilder,
    providers::openai::Client,
    vector_store::{in_memory_store::InMemoryVectorStore, VectorStore},
};

let client = Client::from_env();
let embedding_model = client.embedding_model("text-embedding-ada-002");

// Build vector store with documents
let mut vector_store = InMemoryVectorStore::default();
let embeddings = EmbeddingsBuilder::new(embedding_model.clone())
    .simple_document("doc1", "Rig supports OpenAI and Cohere.")
    .simple_document("doc2", "Rig has built-in RAG support.")
    .build()
    .await?;
vector_store.add_documents(embeddings).await?;

// Create RAG agent
let rag_agent = client.context_rag_agent("gpt-4")
    .preamble("Answer questions about Rig based on the provided context.")
    .dynamic_context(2, vector_store.index(embedding_model))
    .build();

let response = rag_agent.prompt("What providers does Rig support?").await?;
```

### Type-Safe Extraction

```rust
use serde::Deserialize;
use rig::providers::openai::Client;

#[derive(Deserialize, rig::JsonSchema, Debug)]
struct Person {
    name: String,
    age: u8,
    occupation: String,
}

let client = Client::from_env();
let extractor = client.extractor::<Person>("gpt-4").build();

let person: Person = extractor
    .extract("John Doe is a 30-year-old software engineer.")
    .await?;

println!("{:?}", person);
// Person { name: "John Doe", age: 30, occupation: "software engineer" }
```

### Tool Calling

```rust
use rig::{tool::Tool, completion::ToolDefinition};

#[derive(Tool)]
#[tool(description = "Get current weather for a city")]
struct WeatherTool;

impl WeatherTool {
    async fn call(&self, city: String) -> String {
        format!("Weather in {}: Sunny, 72°F", city)
    }
}

let agent = client.agent("gpt-4")
    .preamble("You are a helpful assistant with weather access.")
    .tool(WeatherTool)
    .build();
```

## llm-chain

LangChain-inspired framework for sequential LLM workflows.

**Cargo.toml:**
```toml
[dependencies]
llm-chain = "0.13"
llm-chain-openai = "0.13"
tokio = { version = "1", features = ["full"] }
```

### Basic Chain

```rust
use llm_chain::{parameters, prompt};
use llm_chain::step::Step;
use llm_chain::chains::sequential::Chain;
use llm_chain_openai::chatgpt::Executor;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let exec = Executor::new()?;

    let chain = Chain::new(vec![
        Step::for_prompt_template(
            prompt!("You are a travel expert",
                   "Recommend 3 places to visit in {{city}}")
        ),
        Step::for_prompt_template(
            prompt!("You are a content writer",
                   "Format these as bullet points:\n{{text}}")
        ),
    ]);

    let result = chain
        .run(parameters!("city" => "Tokyo"), &exec)
        .await?;

    println!("{}", result.to_immediate().await?.as_content());
    Ok(())
}
```

### Multi-Step Workflow

```rust
let chain = Chain::new(vec![
    // Step 1: Research
    Step::for_prompt_template(
        prompt!("Research assistant",
               "Find key facts about {{topic}}")
    ),
    // Step 2: Summarize
    Step::for_prompt_template(
        prompt!("Summarizer",
               "Summarize in 3 sentences:\n{{text}}")
    ),
    // Step 3: Format
    Step::for_prompt_template(
        prompt!("Editor",
               "Make this engaging for social media:\n{{text}}")
    ),
]);

let result = chain
    .run(parameters!("topic" => "quantum computing"), &exec)
    .await?;
```

### Available Drivers

- `llm-chain-openai` - OpenAI GPT models
- `llm-chain-llama` - Local LLAMA models
- Custom drivers possible

## Kalosm

Local-first AI framework with language, audio, and vision support.

**Cargo.toml:**
```toml
[dependencies]
kalosm = { version = "0.3", features = ["full"] }
tokio = { version = "1", features = ["full"] }
```

### Text Generation

```rust
use kalosm::language::*;

#[tokio::main]
async fn main() {
    let mut llm = Llama::new().await.unwrap();

    let mut stream = llm("Write a poem about Rust:");
    stream.to_std_out().await.unwrap();
}
```

### Structured Generation

```rust
use kalosm::language::*;
use std::sync::Arc;

#[derive(Parse, Clone, Debug)]
enum Sentiment {
    Positive,
    Negative,
    Neutral,
}

#[derive(Parse, Clone, Debug)]
struct Analysis {
    sentiment: Sentiment,
    confidence: f32,
}

let llm = Llama::new_chat().await?;
let task = llm
    .task("Analyze the sentiment of the user's message")
    .with_constraints(Arc::new(Analysis::new_parser()));

let result: Analysis = task("I love this product!").await?;
println!("{:?}", result);
```

### RAG with SurrealDB

```rust
use kalosm::language::*;
use surrealdb::{engine::local::SurrealKv, Surreal};

let db = Surreal::new::<SurrealKv>("./db/temp.db").await?;
db.use_ns("rag").use_db("docs").await?;

let document_table = db
    .document_table_builder("documents")
    .at("./db/embeddings.db")
    .build::<Document>()
    .await?;

// Add documents
document_table.add_context([
    Url::parse("https://example.com/docs")?
]).await?;

// Query with context
let model = Llama::new_chat().await?;
let context = document_table
    .search("How do I use Kalosm?")
    .with_results(3)
    .await?;

let prompt = format!("{}\n\nQuestion: How do I use Kalosm?",
    context.iter().map(|d| d.record.body()).collect::<Vec<_>>().join("\n"));

let mut stream = model.chat()(&prompt);
stream.to_std_out().await?;
```

### Audio Transcription

```rust
use kalosm::audio::*;

let model = Whisper::new().await?;
let audio = Audio::from_file("speech.wav")?;
let transcription = model.transcribe(audio).await?;
println!("{}", transcription);
```

### Image Generation

```rust
use kalosm::vision::*;

let model = Wuerstchen::new().await?;
let image = model.generate("A rusty robot in a garden").await?;
image.save("output.png")?;
```

## LangChain-rust

Community port of LangChain concepts to Rust.

**Cargo.toml:**
```toml
[dependencies]
langchain-rust = "4"
tokio = { version = "1", features = ["full"] }
serde_json = "1"
```

### Basic Chain

```rust
use langchain_rust::llm::OpenAI;
use langchain_rust::prompt::PromptTemplate;
use langchain_rust::chain::LLMChain;

let llm = OpenAI::new()
    .with_api_key("...")
    .model("gpt-3.5-turbo");

let template = PromptTemplate::new("Translate to French: {input}")?;
let chain = LLMChain::new(llm, template);

let result = chain.run(&serde_json::json!({"input": "Hello!"}))?;
println!("{}", result);
```

### With Vector Store

Supports Qdrant, SQLite, SurrealDB via feature flags:
```toml
langchain-rust = { version = "4", features = ["qdrant"] }
```

## Comparison

| Framework | Best For | RAG | Agents | Local Models | Multimodal |
|:----------|:---------|:----|:-------|:-------------|:-----------|
| Rig | RAG apps, type-safe extraction | Excellent | Good | Via providers | No |
| llm-chain | Sequential workflows | Basic | Limited | Yes (llama) | No |
| Kalosm | Local-first, multimodal | Good | Basic | Excellent | Yes |
| LangChain-rust | LangChain familiarity | Good | Good | Via Ollama | No |

## Choosing a Framework

**Choose Rig when:**
- Building RAG applications
- Need type-safe structured extraction
- Want provider flexibility (OpenAI, Cohere)
- Building production applications

**Choose llm-chain when:**
- Need multi-step sequential workflows
- Familiar with LangChain patterns
- Want declarative chain definitions

**Choose Kalosm when:**
- Local/private inference is required
- Need multimodal (audio, vision)
- Building edge/offline applications
- Want integrated vector store

**Choose LangChain-rust when:**
- Coming from Python LangChain
- Need agent/tool patterns
- Want familiar abstractions
