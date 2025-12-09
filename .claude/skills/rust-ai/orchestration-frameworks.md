# High-Level Orchestration Frameworks

Frameworks for complex LLM workflows, RAG systems, and agent development.

## Rig

Modular framework for LLM-powered applications with focus on RAG.

### Core Features
- Unified `CompletionModel` and `EmbeddingModel` traits
- Vector store integrations (Qdrant, MongoDB, LanceDB)
- Type-safe structured data extraction
- Tool calling and agents

### Basic Completion

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

### RAG Agent

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
        .simple_document("doc1", "Rig is a Rust library for LLM applications.")
        .simple_document("doc2", "Rig supports OpenAI and Cohere.")
        .build()
        .await?;
    vector_store.add_documents(embeddings).await?;

    let rag_agent = openai_client.context_rag_agent("gpt-4")
        .preamble("You answer questions about Rig.")
        .dynamic_context(1, vector_store.index(embedding_model))
        .build();

    let response = rag_agent.prompt("What is Rig?").await?;
    println!("RAG Agent: {}", response);
    Ok(())
}
```

### Type-Safe Extraction

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
| Rig | RAG, structured extraction | Via providers | Yes | Yes |
| llm-chain | Sequential chains | Via drivers | With extensions | Limited |
| Kalosm | Local-first, multimodal | Native | Yes | Limited |
| LangChain-rust | LangChain parity | Via Ollama | With integrations | Yes |

## Sources

- [Rig](https://rig.rs/) / [GitHub](https://github.com/0xPlaygrounds/rig)
- [llm-chain](https://github.com/sobelio/llm-chain)
- [Kalosm](https://docs.rs/kalosm)
- [LangChain-rust](https://github.com/Abraxas-365/langchain-rust)
