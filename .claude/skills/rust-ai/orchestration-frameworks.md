# High-Level Orchestration Frameworks

Frameworks for complex LLM workflows, RAG systems, and agent development.

## Rig (Recommended)

Modular framework for LLM-powered applications with focus on RAG and type safety.

### Why Rig?

1. **Unified API**: Single interface for completions and embeddings across providers
2. **RAG-Ready**: Built-in vector store abstractions and retrieval patterns
3. **Type-Safe**: Structured extraction via `#[derive(JsonSchema)]`
4. **Extensible**: Trait-based design for custom provider implementations
5. **Production-Ready**: Used by VT Code, Dria, Cairnify in production

### Basic Completion

```rust
use rig::{completion::Prompt, providers::openai};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let openai_client = openai::Client::from_env();
    let gpt4 = openai_client.model("gpt-4o").build();

    let response = gpt4.prompt("Explain quantum computing in one sentence.").await?;
    println!("GPT-4: {}", response);
    Ok(())
}
```

### Provider Abstraction Pattern

Create a unified interface that works with multiple providers:

```rust
use rig::completion::{CompletionModel, Prompt};
use rig::providers::{openai, anthropic, ollama};

pub enum ModelProvider {
    OpenAI(openai::Client),
    Anthropic(anthropic::Client),
    Ollama(ollama::Client),
}

pub struct CompositionAI {
    client: Box<dyn CompletionModel>,
    embedding_model: Box<dyn rig::embeddings::EmbeddingModel>,
}

impl CompositionAI {
    /// Parse "provider/model" format from environment
    pub fn from_env_model(model_str: &str) -> Result<Self, anyhow::Error> {
        let parts: Vec<&str> = model_str.split('/').collect();
        let (provider, model_name) = (parts[0], parts[1]);

        let client: Box<dyn CompletionModel> = match provider {
            "openai" => {
                let client = openai::Client::from_env();
                Box::new(client.model(model_name).build())
            }
            "anthropic" => {
                let client = anthropic::Client::from_env();
                Box::new(client.model(model_name).build())
            }
            "ollama" => {
                let client = ollama::Client::from_env();
                Box::new(client.model(model_name).build())
            }
            "openrouter" => {
                // OpenRouter uses OpenAI-compatible API
                let client = openai::Client::new(
                    &std::env::var("OPENROUTER_API_KEY")?,
                    "https://openrouter.ai/api/v1"
                );
                Box::new(client.model(model_name).build())
            }
            _ => anyhow::bail!("Unsupported provider: {}", provider),
        };

        // Similar pattern for embeddings...
        Ok(Self { client, embedding_model: todo!() })
    }

    pub async fn summarize(&self, content: &str) -> Result<String, anyhow::Error> {
        let prompt = format!(
            "Summarize the following content concisely:\n\n{}\n\nSummary:",
            content
        );
        Ok(self.client.prompt(&prompt).await?)
    }

    pub async fn consolidate(&self, documents: &[&str]) -> Result<String, anyhow::Error> {
        let combined = documents.join("\n\n---\n\n");
        let prompt = format!(
            "Consolidate these documents into a cohesive whole, \
             restructuring and supplementing as needed:\n\n{}\n\nConsolidated document:",
            combined
        );
        Ok(self.client.prompt(&prompt).await?)
    }
}
```

### Type-Safe Structured Extraction

Use rig's `#[derive(JsonSchema)]` for structured output:

```rust
use serde::Deserialize;
use rig::providers::openai::Client;

#[derive(Debug, Deserialize, rig::JsonSchema)]
pub struct ExtractedTopic {
    pub topic_name: String,
    pub summary: String,
    pub key_points: Vec<String>,
    pub related_concepts: Vec<String>,
}

pub async fn extract_topic(
    documents: &[&str],
    topic: &str,
) -> Result<ExtractedTopic, anyhow::Error> {
    let client = Client::from_env();
    let extractor = client.extractor::<ExtractedTopic>("gpt-4o")
        .preamble(&format!(
            "You are an expert at extracting information about '{}' from documents. \
             Analyze the provided documents and extract all relevant information about this topic.",
            topic
        ))
        .build();

    let combined = documents.join("\n\n---\n\n");
    let extracted: ExtractedTopic = extractor.extract(&combined).await?;
    Ok(extracted)
}
```

### RAG Agent with Vector Store

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
    let embedding_model = openai_client.embedding_model("text-embedding-3-small");

    let mut vector_store = InMemoryVectorStore::default();

    let embeddings = EmbeddingsBuilder::new(embedding_model.clone())
        .simple_document("doc1", "Rig is a Rust library for LLM applications.")
        .simple_document("doc2", "Rig supports OpenAI and Anthropic providers.")
        .build()
        .await?;
    vector_store.add_documents(embeddings).await?;

    let rag_agent = openai_client.context_rag_agent("gpt-4o")
        .preamble("You answer questions based on the provided context.")
        .dynamic_context(3, vector_store.index(embedding_model))
        .build();

    let response = rag_agent.prompt("What is Rig?").await?;
    println!("RAG Agent: {}", response);
    Ok(())
}
```

### Embeddings with rig-core

```rust
use rig::embeddings::{EmbeddingsBuilder, EmbeddingModel};
use rig::providers::openai;

pub struct DocumentEmbedder {
    embedding_model: openai::EmbeddingModel,
}

impl DocumentEmbedder {
    pub fn new() -> Self {
        let client = openai::Client::from_env();
        let embedding_model = client.embedding_model("text-embedding-3-small");
        Self { embedding_model }
    }

    pub async fn embed_documents(
        &self,
        documents: Vec<(&str, &str)>,  // (id, content)
    ) -> Result<Vec<rig::embeddings::Embedding>, anyhow::Error> {
        let mut builder = EmbeddingsBuilder::new(self.embedding_model.clone());

        for (id, content) in documents {
            builder = builder.simple_document(id, content);
        }

        let embeddings = builder.build().await?;
        Ok(embeddings)
    }

    pub async fn embed_query(&self, query: &str) -> Result<Vec<f32>, anyhow::Error> {
        let embedding = self.embedding_model.embed_text(query).await?;
        Ok(embedding.vec)
    }
}
```

## llm-chain

LangChain-inspired sequential chain orchestration.

### Multi-Step Chain

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
        // Step 1: Find places to visit
        Step::for_prompt_template(
            prompt!("You are a travel assistant",
                "Find good places to visit in {{city}}, {{country}}.")
        ),
        // Step 2: Format as bullet points
        Step::for_prompt_template(
            prompt!("You manage social media for a travel company",
                "Format into 5 bullet points:\n--\n{{text}}")
        ),
        // Step 3: Create LinkedIn post
        Step::for_prompt_template(
            prompt!("You manage social media for a travel company",
                "Summarize into a LinkedIn post:\n--\n{{text}}")
        )
    ]);

    let result = chain
        .run(parameters!("city" => "Rome", "country" => "Italy"), &exec)
        .await?;

    println!("{}", result.to_immediate().await?.as_content());
    Ok(())
}
```

### Features
- `prompt!` macro for templates
- `parameters!` macro for variable substitution
- `{{text}}` auto-substituted with previous step output
- Sequential and map-reduce chains

## Kalosm

Local-first AI meta-framework built on Candle.

### Local Text Generation

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

### Structured Generation

```rust
use kalosm::language::*;
use std::sync::Arc;

#[derive(Parse, Clone, Debug)]
enum Class {
    Thing,
    Person,
    Animal,
}

#[derive(Parse, Clone, Debug)]
struct Response {
    classification: Class,
}

#[tokio::main]
async fn main() {
    let llm = Llama::new_chat().await.unwrap();
    let task = llm.task("Classify the message as person, animal, or thing in JSON")
        .with_constraints(Arc::new(Response::new_parser()));

    let response = task("The Kalosm library").await.unwrap();
    println!("{:?}", response);
}
```

### RAG with SurrealDB

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

    // Add documents
    let context = ["https://example.com/docs"]
        .iter()
        .map(|url| Url::parse(url).unwrap());
    document_table.add_context(context).await?;

    // Query with RAG
    let model = Llama::new_chat().await?;
    let mut chat = model.chat().with_system_prompt("Answer based on context.");

    let user_question = "What is this about?";
    let context = document_table.search(&user_question).with_results(1).await?;

    let prompt = format!("{}\n{}", context_text, user_question);
    let mut output = chat(&prompt);
    output.to_std_out().await?;
    Ok(())
}
```

## LangChain-rust

Community port of Python LangChain.

### Basic Chain

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

### Features
- PromptTemplate, Chain, Memory abstractions
- Tool and agent definitions
- Vector stores: Qdrant, SQLite, SurrealDB

## Comparison

| Framework | Focus | Local Models | RAG Built-in | Agent Support |
|-----------|-------|--------------|--------------|---------------|
| **Rig** | RAG, structured extraction | Via providers | Yes | Yes |
| llm-chain | Sequential chains | Via drivers | With extensions | Limited |
| Kalosm | Local-first, multimodal | Native | Yes | Limited |
| LangChain-rust | LangChain parity | Via Ollama | With integrations | Yes |

## Sources

- [Rig](https://rig.rs/) / [GitHub](https://github.com/0xPlaygrounds/rig) / [Docs](https://docs.rig.rs/)
- [Rig RAG Guide](https://docs.rig.rs/guides/rag/rag_system)
- [llm-chain](https://github.com/sobelio/llm-chain)
- [Kalosm](https://docs.rs/kalosm)
- [LangChain-rust](https://github.com/Abraxas-365/langchain-rust)
