# Code Examples

Complete, working examples for common LLM tasks in Rust.

## Chat Completion

### Using llm (Unified SDK)

```rust
use llm::{LLMBuilder, backend::LLMBackend};

#[tokio::main]
async fn main() -> llm::Result<()> {
    let llm = LLMBuilder::new()
        .backend(LLMBackend::OpenAI)
        .api_key(std::env::var("OPENAI_API_KEY")?)
        .model("gpt-4")
        .build()?;

    let response = llm.chat(["What is Rust programming language?"]).await?;
    println!("{}", response);
    Ok(())
}
```

### Using async-openai

```rust
use openai_rust::{Client, chat::{ChatArguments, Message}};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::new(&std::env::var("OPENAI_API_KEY")?);

    let messages = vec![
        Message { role: "system".into(), content: "You are helpful.".into() },
        Message { role: "user".into(), content: "Hello!".into() },
    ];

    let args = ChatArguments::new("gpt-4", messages);
    let response = client.create_chat(args).await?;

    println!("{}", response);
    Ok(())
}
```

### Using Rig

```rust
use rig::{completion::Prompt, providers::openai};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = openai::Client::from_env();
    let model = client.model("gpt-4").build();

    let response = model.prompt("Explain quantum computing simply.").await?;
    println!("{}", response);
    Ok(())
}
```

## Streaming Responses

### Using llm

```rust
use llm::{LLMBuilder, backend::LLMBackend};
use futures::StreamExt;

#[tokio::main]
async fn main() -> llm::Result<()> {
    let llm = LLMBuilder::new()
        .backend(LLMBackend::OpenAI)
        .api_key(std::env::var("OPENAI_API_KEY")?)
        .model("gpt-4")
        .build()?;

    let mut stream = llm.chat_stream(["Write a haiku about Rust"]).await?;

    while let Some(chunk) = stream.next().await {
        print!("{}", chunk?);
        std::io::Write::flush(&mut std::io::stdout())?;
    }
    println!();
    Ok(())
}
```

### Using Kalosm (Local)

```rust
use kalosm::language::*;

#[tokio::main]
async fn main() {
    let mut llm = Llama::new().await.unwrap();

    print!("Response: ");
    let mut stream = llm("Tell me a joke:");
    stream.to_std_out().await.unwrap();
}
```

## Embeddings

### Using llm

```rust
use llm::{LLMBuilder, backend::LLMBackend};

#[tokio::main]
async fn main() -> llm::Result<()> {
    let llm = LLMBuilder::new()
        .backend(LLMBackend::OpenAI)
        .api_key(std::env::var("OPENAI_API_KEY")?)
        .model("text-embedding-ada-002")
        .build()?;

    let texts = vec!["Hello world", "Rust is great", "AI is the future"];
    let embeddings = llm.embed(texts).await?;

    for (i, emb) in embeddings.iter().enumerate() {
        println!("Text {}: {} dimensions", i, emb.len());
    }
    Ok(())
}
```

### Using Rig

```rust
use rig::{embeddings::EmbeddingsBuilder, providers::openai::Client};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::from_env();
    let model = client.embedding_model("text-embedding-ada-002");

    let embeddings = EmbeddingsBuilder::new(model)
        .simple_document("id1", "First document")
        .simple_document("id2", "Second document")
        .build()
        .await?;

    println!("Generated {} embeddings", embeddings.len());
    Ok(())
}
```

## RAG (Retrieval-Augmented Generation)

### Complete RAG with Rig

```rust
use rig::{
    completion::Prompt,
    embeddings::EmbeddingsBuilder,
    providers::openai::Client,
    vector_store::{in_memory_store::InMemoryVectorStore, VectorStore},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::from_env();
    let embedding_model = client.embedding_model("text-embedding-ada-002");

    // Create and populate vector store
    let mut store = InMemoryVectorStore::default();

    let docs = EmbeddingsBuilder::new(embedding_model.clone())
        .simple_document("rust", "Rust is a systems programming language focused on safety.")
        .simple_document("cargo", "Cargo is Rust's package manager and build tool.")
        .simple_document("crates", "Crates.io is the Rust community's package registry.")
        .build()
        .await?;

    store.add_documents(docs).await?;

    // Create RAG agent
    let agent = client
        .context_rag_agent("gpt-4")
        .preamble("Answer questions about Rust using only the provided context.")
        .dynamic_context(2, store.index(embedding_model))
        .build();

    // Query
    let response = agent.prompt("What is Cargo?").await?;
    println!("{}", response);
    Ok(())
}
```

### RAG with Kalosm and SurrealDB

```rust
use kalosm::language::*;
use surrealdb::{engine::local::SurrealKv, Surreal};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Setup database
    let db = Surreal::new::<SurrealKv>("./rag.db").await?;
    db.use_ns("app").use_db("docs").await?;

    let table = db
        .document_table_builder("knowledge")
        .at("./embeddings.db")
        .build::<Document>()
        .await?;

    // Index documents (only needed once)
    if !std::path::Path::new("./rag.db").exists() {
        table.add_context([
            "https://doc.rust-lang.org/book/ch01-00-getting-started.html".parse()?
        ]).await?;
    }

    // Create model and chat
    let model = Llama::new_chat().await?;

    // Search for context
    let query = "How do I install Rust?";
    let context = table.search(query).with_results(2).await?;

    let context_text: String = context
        .iter()
        .map(|d| d.record.body())
        .collect::<Vec<_>>()
        .join("\n\n");

    // Generate response with context
    let prompt = format!(
        "Context:\n{}\n\nQuestion: {}\n\nAnswer based on the context:",
        context_text, query
    );

    let mut stream = model.chat()(&prompt);
    stream.to_std_out().await?;
    Ok(())
}
```

## Multi-Step Chains

### Using llm-chain

```rust
use llm_chain::{parameters, prompt, step::Step, chains::sequential::Chain};
use llm_chain_openai::chatgpt::Executor;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let exec = Executor::new()?;

    let chain = Chain::new(vec![
        // Step 1: Generate ideas
        Step::for_prompt_template(prompt!(
            "You are a creative writer",
            "Generate 3 blog post ideas about {{topic}}"
        )),
        // Step 2: Select and outline
        Step::for_prompt_template(prompt!(
            "You are an editor",
            "Pick the best idea and create an outline:\n{{text}}"
        )),
        // Step 3: Write intro
        Step::for_prompt_template(prompt!(
            "You are a content writer",
            "Write an engaging introduction based on this outline:\n{{text}}"
        )),
    ]);

    let result = chain
        .run(parameters!("topic" => "Rust programming"), &exec)
        .await?;

    println!("{}", result.to_immediate().await?.as_content());
    Ok(())
}
```

## Structured Output

### Type-Safe Extraction with Rig

```rust
use serde::Deserialize;
use rig::providers::openai::Client;

#[derive(Deserialize, rig::JsonSchema, Debug)]
struct Recipe {
    name: String,
    ingredients: Vec<String>,
    prep_time_minutes: u32,
    instructions: Vec<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::from_env();
    let extractor = client.extractor::<Recipe>("gpt-4").build();

    let recipe: Recipe = extractor.extract(
        "Make a simple pasta: boil water, cook spaghetti 8 mins, \
         drain, add olive oil, garlic, parmesan. Takes 15 minutes."
    ).await?;

    println!("Recipe: {}", recipe.name);
    println!("Prep time: {} minutes", recipe.prep_time_minutes);
    println!("Ingredients: {:?}", recipe.ingredients);
    Ok(())
}
```

### Constrained Generation with Kalosm

```rust
use kalosm::language::*;
use std::sync::Arc;

#[derive(Parse, Clone, Debug)]
struct MovieReview {
    title: String,
    rating: u8,  // 1-10
    summary: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let llm = Llama::new_chat().await?;

    let task = llm
        .task("Extract movie review details from the user's text")
        .with_constraints(Arc::new(MovieReview::new_parser()));

    let review: MovieReview = task(
        "I just watched Inception, it was mind-blowing! \
         Definitely a 9 out of 10. The dream-within-a-dream \
         concept was brilliantly executed."
    ).await?;

    println!("{:?}", review);
    Ok(())
}
```

## Local Inference

### Using Ollama

```rust
use ollama_rs::{Ollama, GenerationRequest};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let ollama = Ollama::default();

    let request = GenerationRequest::new(
        "llama2".to_string(),
        "Explain the borrow checker in Rust".to_string(),
    );

    let response = ollama.generate(request).await?;
    println!("{}", response.response);
    Ok(())
}
```

### Using llama_cpp

```rust
use llama_cpp::{LlamaModel, LlamaParams, SessionParams};
use llama_cpp::standard_sampler::StandardSampler;

fn main() -> anyhow::Result<()> {
    let model = LlamaModel::load_from_file(
        "models/llama-7b.gguf",
        LlamaParams::default()
    )?;

    let mut session = model.create_session(SessionParams::default())?;
    session.advance_context("The meaning of life is")?;

    let mut tokens = session
        .start_completing_with(StandardSampler::default(), 100)
        .into_strings();

    while let Some(token) = tokens.next() {
        print!("{}", token);
    }
    Ok(())
}
```

## Tool Calling

### With Rig

```rust
use rig::{tool::Tool, completion::Prompt, providers::openai::Client};

#[derive(Tool)]
#[tool(description = "Calculate the result of a math expression")]
struct Calculator;

impl Calculator {
    async fn call(&self, expression: String) -> String {
        // Simple eval (use a proper math parser in production)
        match expression.as_str() {
            "2+2" => "4".to_string(),
            _ => format!("Cannot evaluate: {}", expression),
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::from_env();

    let agent = client.agent("gpt-4")
        .preamble("You can use the calculator tool for math.")
        .tool(Calculator)
        .build();

    let response = agent.prompt("What is 2+2?").await?;
    println!("{}", response);
    Ok(())
}
```

## Error Handling

### Robust API Client

```rust
use llm::{LLMBuilder, backend::LLMBackend};

#[tokio::main]
async fn main() {
    let result = async {
        let llm = LLMBuilder::new()
            .backend(LLMBackend::OpenAI)
            .api_key(std::env::var("OPENAI_API_KEY")?)
            .model("gpt-4")
            .build()?;

        llm.chat(["Hello"]).await
    }.await;

    match result {
        Ok(response) => println!("Success: {}", response),
        Err(e) => {
            eprintln!("Error: {}", e);
            // Handle specific error types
            // - Rate limiting
            // - Invalid API key
            // - Model not found
            // - Network errors
        }
    }
}
```

### Retry with Backoff

```rust
use std::time::Duration;
use tokio::time::sleep;

async fn chat_with_retry(llm: &impl ChatProvider, prompt: &str, max_retries: u32) -> Result<String> {
    let mut attempts = 0;

    loop {
        match llm.chat([prompt]).await {
            Ok(response) => return Ok(response),
            Err(e) if attempts < max_retries => {
                attempts += 1;
                let backoff = Duration::from_millis(100 * 2u64.pow(attempts));
                eprintln!("Retry {} after {:?}: {}", attempts, backoff, e);
                sleep(backoff).await;
            }
            Err(e) => return Err(e),
        }
    }
}
```
