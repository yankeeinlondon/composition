# Rust Libraries for LLM Integration

Rust’s growing AI ecosystem offers both provider-specific SDKs and higher-level orchestration frameworks for working with large language models (LLMs). Below we detail several solutions – from simple API clients (e.g. OpenAI, Anthropic) to multi-provider toolkits and LangChain-like frameworks – including their capabilities and usage examples. Each solution’s code snippet illustrates how a Rust developer can integrate or use it in practice.

## OpenAI API Crates (Rust SDKs for OpenAI)

Rust has unofficial SDKs for OpenAI's API, allowing you to call models like GPT-3.5 or GPT-4. Notable crates include [openai](https://crates.io/crates/openai) (aka openai-rust) and [async-openai](https://crates.io/crates/async-openai), which provide asynchronous, strongly-typed interfaces covering chat completions, completions, embeddings, etc. These clients mirror OpenAI's endpoints and support streaming and other features. For example, the openai-rust crate supports listing models, chat completions, edits, embeddings, images, and more.

Using these crates is straightforward: instantiate a client with your API key, build a request (messages, model name, etc.), and call the API. In the example below, we create a chat request to the gpt-3.5-turbo model:

use openai_rust::{Client, chat::{ChatArguments, Message}};

```rust
# [tokio::main]
async fn main() {
    // Initialize OpenAI client with API key from environment
    let api_key = std::env::var("OPENAI_API_KEY").expect("Missing API key");
    let client = Client::new(&api_key);

    // Prepare a chat conversation with one user message
    let args = ChatArguments::new("gpt-3.5-turbo", vec![
        Message { role: "user".into(), content: "Hello, GPT!".into() }
    ]);

    // Send the chat completion request
    let response = client.create_chat(args).await.unwrap();
    println!("{}", response);  // print the assistant's reply
}
```

This uses the `openai_rust::Client` to call OpenAI's chat completion endpoint. The OpenAI crates also allow setting a custom API base URL, which is useful for services like **[OpenRouter](https://openrouter.ai)**. **OpenRouter** acts as a unified proxy for many LLM providers, exposing an OpenAI-compatible API. To use OpenRouter with Rust, you can point an OpenAI SDK client at the **OpenRouter** endpoint (e.g. <https://openrouter.ai/api/v1>) and use your **OpenRouter** API key. For instance, in TypeScript the OpenAI SDK can be configured with baseURL: "<https://openrouter.ai/api/v1>" and the **OpenRouter** key – similarly, in Rust you can override the base URL or use the OpenAI client with an environment variable for the base. This lets you access models from OpenAI, Anthropic, etc. through one interface.

## Anthropic API Client (Claude SDK for Rust)

For Anthropic's Claude models, Rust has an unofficial SDK [anthropic-rs](https://github.com/AbdelStark/anthropic-rs). This crate provides an async client to call Claude's completion API (including streaming support). It is still work-in-progress but active. The usage is similar: configure your API key and model, then send a completion prompt. For example, using anthropic-rs you might do:

use anthropic::{Client, AnthropicConfig, CompleteRequestBuilder, HUMAN_PROMPT, AI_PROMPT};

```rust
# [tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load Anthropic API key from environment (e.g. in .env file)
    dotenv::dotenv().ok();
    let cfg = AnthropicConfig::new()?;              // reads ANTHROPIC_API_KEY
    let client = Client::try_from(cfg)?;            // initialize Anthropic client

    // Build a completion request for Claude
    let request = CompleteRequestBuilder::default()
        .model("claude-instant-1".to_string())      // specify Claude model
        .prompt(format!("{HUMAN_PROMPT}How many toes do dogs have?{AI_PROMPT}"))
        .max_tokens_to_sample(1000)
        .stream_response(false)                     // get full response (not streaming)
        .build()?;

    let response = client.complete(request).await?; 
    println!("Claude answered: {}", response.completion);
    Ok(())
}
```

In this snippet, we use Anthropic's human (HUMAN_PROMPT) and assistant (AI_PROMPT) prompt delimiters and ask Claude a question. The anthropic-rs client handles the HTTP call to Anthropic's API. (Note: The Anthropic crate uses dotenv to load API keys, and it was inspired by the async OpenAI Rust library.)

## Multi-Provider LLM SDKs (Unified Interfaces)

Several community crates aim to unify multiple LLM providers behind one API. These are Rust's answer to an "AI SDK" that can talk to OpenAI, Anthropic, Ollama, and others through a single interface. Notable examples:
 • [RLLM](https://github.com/graniet/rllm) (published as the [llm crate](https://crates.io/crates/llm) on crates.io): Provides a common trait-based interface for chat and completion across providers. It supports OpenAI, Anthropic (Claude), Ollama, and more, using a builder to configure which backend to use at runtime. Developers can enable only the features (backends) they need (via Cargo feature flags for openai, anthropic, ollama, etc.). RLLM includes chat and text completion traits (ChatProvider, CompletionProvider) for a unified experience.
 • Allms and llmclient: Similar unified clients covering OpenAI, Anthropic, and other APIs with a common interface.
 • [rust-genai](https://crates.io/crates/genai) (genAI): Another multi-provider SDK (experimental) covering OpenAI, Anthropic, Google's PaLM, Cohere, and others via one ergonomic API.

Using a unified SDK simplifies switching between models. For example, with RLLM you can configure a backend and send a chat in one fluent builder chain:

```rust
use rllm::{LLMBuilder, backend::LLMBackend};

# [tokio::main]
async fn main() -> rllm::Result<()> {
    // Build an LLM client targeting OpenAI GPT-3.5
    let llm = LLMBuilder::new()
        .backend(LLMBackend::OpenAI)           // choose provider (OpenAI, Anthropic, etc.)
        .api_key(std::env::var("OPENAI_API_KEY")?)
        .model("gpt-3.5-turbo")
        .build()?;

    // Use the unified interface to have a conversation
    let user_message = "Hello, how are you?";
    let reply = llm.chat([user_message]).await?;   // `chat` returns the assistant's response
    println!("Model reply: {}", reply);
    Ok(())
}
```

In the above pseudo-code, LLMBuilder selects the backend and model – e.g. switching to LLMBackend::Anthropic with a Claude model string would call Anthropic instead. Under the hood, RLLM's traits abstract over provider-specific details. This approach is useful for writing provider-agnostic code or implementing fallback logic (e.g., try OpenAI, then Anthropic). Keep in mind that multi-backend crates typically perform API calls (not local inference) and may not support all advanced features of each API, but they cover common functionality like chat completions, completions, and embeddings.

OpenRouter Note: Since OpenRouter itself routes to various models, one could also use a single OpenAI-compatible client with OpenRouter as a backend (as mentioned earlier). However, multi-provider SDKs like RLLM allow using providers directly or even chaining them (RLLM has modules for chains and agents).

## Ollama-rs (Local LLM Server Client)

Ollama is a tool for running local LLMs (like LLaMA 2, Mistral, etc.) via an HTTP API. The crate [ollama-rs](https://github.com/pepperoni21/ollama-rs) provides a Rust client to interact with an Ollama server. This means you can download or serve a model with Ollama (which runs on your machine) and use Rust to send it prompts and get completions, similar to calling a cloud API. Ollama supports both prompt completions and model management (listing models, creating new ones from Modelfiles, etc.).

Using ollama-rs is straightforward. By default it will connect to a local Ollama daemon at localhost:11434. You can then generate text by specifying the model and prompt:

```rust
use ollama_rs::{Ollama, GenerationRequest};

# [tokio::main]
async fn main() -> anyhow::Result<()> {
    let ollama = Ollama::default();  // connect to localhost:11434 by default

    // Choose a model installed in Ollama and send a prompt
    let model = "llama2:latest".to_string();
    let prompt = "Why is the sky blue?".to_string();
    let result = ollama.generate(GenerationRequest::new(model, prompt)).await?;

    println!("{}", result.response);  // print the model's completion
    Ok(())
}
```

This example asks a locally served LLaMA-2 model the question "Why is the sky blue?" and prints its answer. With Ollama, you can also stream results token-by-token (using generate_stream with the stream feature), list available local models, or load new models. The functional footprint of ollama-rs includes model management endpoints like listing models (ollama.list_local_models().await), showing model info, and creating models, mirroring the Ollama HTTP API.

## LLaMA.cpp Integration (Rust bindings for local models)

If you want to run LLMs fully in-process (without an external server), Rust offers bindings to the popular llama.cpp library for efficient local inference on CPU/GPU. There are a few crates in this space:
 • [llama_cpp](https://crates.io/crates/llama_cpp): High-level, safe Rust bindings to llama.cpp's C API. It provides a LlamaModel and LlamaSession abstraction to load a GGML/GGUF model and generate text in a streaming fashion.
 • [llama-cpp-2](https://crates.io/crates/llama-cpp-2): A lower-level binding closely mirroring the raw C API (useful for more control or exposing new llama.cpp features).
 • [llm_client](https://crates.io/crates/llm_client): A higher-level wrapper that uses llama.cpp under the hood to offer a simple interface for local GPT models (with features like model downloading, preset model configurations, etc.).
 • drama_llama: Another ergonomic wrapper, focused on chat-style completions with llama.cpp (work in progress, but aiming to simplify chat interactions).

Using llama_cpp directly typically involves loading a model from file and creating a session. For example:

```rust
use llama_cpp::{LlamaModel, LlamaParams, SessionParams};
use llama_cpp::standard_sampler::StandardSampler;
use std::io::{self, Write};

fn main() -> anyhow::Result<()> {
    // Load the LLaMA model from a GGUF/GGML file (ensure you have the model file locally)
    let model = LlamaModel::load_from_file("models/7B/ggml-model.gguf", LlamaParams::default())
        .expect("Could not load model");

    // Create a new session (context) for generation
    let mut session = model.create_session(SessionParams::default())
        .expect("Failed to create session");

    // Feed an initial prompt into the context
    session.advance_context("This is the story of a curious Rust programmer.").unwrap();

    // Generate text from the model, token by token
    let mut outputs = session.start_completing_with(StandardSampler::default(), 512).into_strings();
    while let Some(token) = outputs.next() {
        // Print each generated token as it streams out
        print!("{}", token);
        io::stdout().flush().unwrap();
    }
    Ok(())
}
```

In this snippet, we load a local model (e.g. a 7B LLaMA variant in GGUF format) and prompt it with some text. We then use a StandardSampler to sample up to 512 tokens of continuation, iterating over the streamed tokens. The llama_cpp crate handles all the heavy lifting in Rust, leveraging llama.cpp's optimized backend for inference. This gives Rust developers a way to run LLMs like LLaMA 2, Mistral, etc. on local hardware with good performance. (Ensure you compile in release mode for speed, as llama.cpp is compute-heavy.)

For an easier high-level approach, [llm_client](https://crates.io/crates/llm_client) can initialize a local model with one line (downloading it if needed). For example, LlmClient::llama_cpp().mistral7b_instruct_v0_3().init().await? will fetch the Mistral-7B model and get it ready to use for completions. You can then call methods like llm_client.ask(prompt) or use provided reasoning/chain utilities. The trade-off is less manual control but faster setup.

Overall, Rust’s **llama.cpp** integrations enable offline, private LLM usage in applications – ideal for edge deployments or when data must stay on-device.

## LLM Chain Frameworks (LangChain-like Libraries)

Higher-level libraries in Rust provide abstractions for prompt orchestration, chains, agents, and integrating tools or memory with LLMs – analogous to Python's LangChain or Typescript's AI SDK/LangGraph. While this area in Rust is newer, there are a few promising projects:
 • [llm-chain](https://github.com/sobelio/llm-chain) – "the ultimate toolbox" for building LLM applications in Rust. It allows you to chain prompts together, manage state and memory between prompts, and includes utilities like prompt templates, output parsers, and even vector store integration. It's macro-heavy (for ergonomics) and comes with integrations for providers (OpenAI, etc.) via sub-crates. For example, llm-chain-openai enables using OpenAI as the backend for the chain. llm-chain supports sequential chains, map-reduce patterns (for summarization over chunks), and more.
 • [LangChain-rust](https://github.com/Abraxas-365/langchain-rust) – a community port of the LangChain concepts to Rust. It provides composable primitives like PromptTemplate, Chain (sequential or branching chains of calls), tool abstractions, memory interfaces for chat history, etc., similar to LangChain.py. It includes integrations for OpenAI, Azure OpenAI, Ollama, Claude, and even some vector stores via feature flags (e.g., support for Qdrant, SQLite, SurrealDB for embedding storage). This library aims to let you build agentic systems and pipelines in a more declarative way, leveraging Rust's performance and safety.
 • [Anchor-Chain](https://github.com/emersonmde/anchor-chain) – an async framework for LLM workflows that emphasizes static typing and even allows defining agents/tools via YAML or Rust code. It is designed to catch errors at compile time where possible (for example, by enforcing that a tool's input/output schema matches what the LLM expects). Anchor-Chain can orchestrate multi-step workflows and tools, similar to LangChain Agents, but with Rust's type guarantees.
 • [Rig](https://rig.rs/) – a modular framework for LLM-powered applications with a focus on Retrieval-Augmented Generation (RAG) and agents. Rig provides a unified interface to different LLMs and encoders, abstractions for tools/agents, and built-in support for retrieval (e.g., pulling relevant context from a vector store to augment prompts). It's meant for building production-ready AI apps, supporting things like knowledge-base Q&A out of the box.

To illustrate how a Rust LLM chain library might be used, here is a basic llm-chain example that runs a simple prompt chain:

```rust
use llm_chain::{prompt, parameters, executor};
use llm_chain_openai::OpenAIExecutor;  // OpenAI integration for llm-chain

# [tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize an OpenAI-based executor (uses OPENAI_API_KEY from env)
    let exec = executor!(OpenAIExecutor)?.with_model("gpt-3.5-turbo");

    // Define a prompt chain with a system and user message
    let chain = prompt!(
        "You are a polite assistant.",
        "User asks: {{question}}"
    );

    // Run the chain with a parameter for {{question}}
    let params = parameters!("question" => "What is the capital of France?");
    let result = chain.run(&params, &exec).await?;

    println!("Assistant answer: {}", result);
    Ok(())
}
```

In this snippet, we configure an OpenAI executor for llm-chain, then create a prompt template with a variable ({{question}}). We supply the variable and run the chain, which under the hood calls the OpenAI API and returns the completion. The prompt!( ... ) macro and the builder-style executor!() macro make it ergonomic. llm-chain would output something like: "Assistant answer: The capital of France is Paris.". From here, you can compose multiple steps: e.g., take the answer and feed it into another prompt, do a transformation (map-reduce), etc., all managed by the chain.

With LangChain-rust, patterns are similar but using that library's API. For instance, you might create an LLMChain with an OpenAI LLM and a PromptTemplate, then call chain.run(input) to get the result (just as in Python). You can also define tools (e.g., a math calculator function) and use an agent that decides when to use a tool. The syntax will differ (LangChain-rust uses macros like template_fstring! for prompt templates, etc.), but conceptually you chain together Chains, Tools, and Memories. A simple use case could be:


```rust
use langchain_rust::llm::OpenAI;
use langchain_rust::prompt::PromptTemplate;
use langchain_rust::chain::LLMChain;

let llm = OpenAI::new().with_api_key("...").model("gpt-3.5-turbo");
let template = PromptTemplate::new("Translate to French: {input}")?;
let chain = LLMChain::new(llm, template);

let result = chain.run(&serde_json::json!({"input": "Hello, world!"}))?;
println!("{}", result);  // outputs the French translation of "Hello, world!"
```

> **Note:** The above is a conceptual example; actual function names may differ, but this demonstrates the typical usage of defining a prompt and running a chain.

These orchestration libraries significantly expand Rust’s “functional footprint” for LLM apps – enabling complex workflows like agents that use tools, conversing with memory, branching logic, and RAG pipelines, all in a high-level manner. They bring Rust closer to parity with Python/TS ecosystems (LangChain, LangGraph, etc.) in terms of building AI applications.

## Combining Solutions for a Full-Featured Stack

No single Rust crate does everything, but you can mix and match the above solutions to build a comprehensive LLM-powered system. Here are a few combination ideas that play well together:

 • Retrieval-Augmented Q&A: Use an LLM orchestration framework (like llm-chain or rig) together with a vector database client for context retrieval. For example, you might combine llm-chain (for prompt management) with [llm-chain-hnsw](https://crates.io/crates/llm-chain-hnsw) (an integration that enables semantic search over embeddings) and a vector store like Qdrant via [qdrant-client](https://crates.io/crates/qdrant-client). In practice, you would encode documents into embeddings (OpenAI or Hugging Face models), store them in Qdrant, then at query time retrieve relevant text and inject it into the prompt (the chain can handle this). This achieves a Rust-based RAG pipeline: a user question triggers a search for context, which is appended to the prompt for a more accurate LLM answer.
 • Multi-Agent System: For building something like an Auto-GPT or LangGraph agent in Rust, you could combine LangChain-rust (for agent logic and tool integration) with RLLM as the LLM backend. LangChain-rust can manage the agent’s planning/execution loop and tool usage, while RLLM gives you the flexibility to switch between providers (OpenAI for one tool, Anthropic for another, etc.) using the same interface. Throw in a vector store (like the Qdrant or LanceDB Rust clients) if your agents need long-term memory or knowledge storage. This combo leverages LangChain-rust’s structured approach and RLLM’s provider abstraction.
 • Fully Local Solution: If you require all-local execution (no API calls), you can pair an orchestration crate with a local inference engine. For instance, use Anchor-Chain or llm-chain to define the prompting logic, and use llama_cpp or llm_client to run a local model (like Llama 2 13B) for each step. You would write an adapter so that the chain’s “LLM call” uses your local model instead of an API – effectively an in-process provider. This way, you benefit from Rust’s strong typing and the chain framework’s features (tool use, multi-step reasoning), while all inference happens on your hardware via llama.cpp. This combination is powerful for offline applications or deploying on the edge (e.g. an IoT device running a smaller model).
 • Hybrid Cloud-Local Deployment: Rust’s strength in performance means you can integrate multiple components efficiently. Consider using Ollama-rs to serve a fine-tuned local model for some tasks (say, quick classifications or domain-specific answers), and fall back to OpenAI via async-openai for more complex queries – orchestrating the decision logic in Rust. Libraries like [SmartGPT](https://github.com/Cormanz/smartgpt) (a Rust framework for multi-step reasoning with multiple models) or custom Rust code can coordinate these calls. For example, a Rust program could analyze a query and decide: if it’s simple, answer with a local model via Ollama; if it’s complex, route to GPT-4 via OpenRouter. The result can then be combined or verified by another local step. Such pipeline demonstrates how the crates can be combined: OpenAI + OpenRouter SDKs, plus Ollama, plus some orchestration for decision making.

## Summary

In summary, the Rust ecosystem now offers a rich toolkit for LLM developers – from direct API bindings (OpenAI, Claude, etc.) to unified SDKs (covering many providers) to full-fledged chaining frameworks akin to LangChain. By mixing these, Rust authors can build robust, high-performance AI applications, choosing the right tool for each part of the stack. And with ongoing work (e.g. Microsoft's [AICI project](https://github.com/microsoft/aici) for prompt-as-code, or community efforts on agents like [AutoGPT-rs](https://github.com/kevin-rs/autogpt)), the gap between Rust and Python in the LLM space is rapidly closing. Each of the solutions above contributes a piece to that puzzle, giving Rust developers the ability to interact with LLMs in flexible and powerful ways.

Sources:

### OpenAI & Anthropic Rust SDKs (unofficial)
- [async-openai](https://crates.io/crates/async-openai) – Async Rust library for OpenAI based on OpenAPI spec
- [openai-api-rs](https://crates.io/crates/openai-api-rs) – OpenAI API client library for Rust
- [openai-rs/openai-api](https://github.com/openai-rs/openai-api) – Simple Rust library for OpenAI API
- [anthropic-rs](https://github.com/AbdelStark/anthropic-rs) – Anthropic Rust SDK with async support
- [anthropic](https://crates.io/crates/anthropic) – Anthropic crate on crates.io

### RLLM Multi-Backend Library
- [graniet/rllm](https://github.com/graniet/rllm) – GitHub repository
- [graniet/llm](https://github.com/graniet/llm) – The underlying llm crate
- [rllm on crates.io](https://crates.io/crates/rllm) – Package registry
- [rllm docs](https://docs.rs/rllm) – API documentation

### Rust LLM Ecosystem Overview
- [Rust Ecosystem for AI & LLMs](https://hackmd.io/@Hamze/Hy5LiRV1gg) – Hamza's comprehensive HackMD guide covering ollama-rs, llama_cpp, and more

### llm-chain
- [sobelio/llm-chain](https://github.com/sobelio/llm-chain) – GitHub repository
- [llm-chain on crates.io](https://crates.io/crates/llm-chain) – Package registry
- [Shuttle.dev guide](https://www.shuttle.dev/blog/2024/06/06/llm-chain-langchain-rust) – Comprehensive guide to llm-chain usage

### LangChain-rust
- [Abraxas-365/langchain-rust](https://github.com/Abraxas-365/langchain-rust) – GitHub repository
- [langchain-rust on crates.io](https://crates.io/crates/langchain-rust) – Package registry
- [langchain-rust docs](https://docs.rs/crate/langchain-rust/latest) – API documentation

### Ollama-rs
- [pepperoni21/ollama-rs](https://github.com/pepperoni21/ollama-rs) – GitHub repository
- [ollama-rs on crates.io](https://crates.io/crates/ollama-rs) – Package registry
- [ollama-rs docs](https://docs.rs/ollama-rs/latest/ollama_rs/) – API documentation

### llama.cpp Rust Bindings
- [edgenai/llama_cpp-rs](https://github.com/edgenai/llama_cpp-rs) – High-level bindings (llama_cpp)
- [llama_cpp on crates.io](https://crates.io/crates/llama_cpp) – Package registry
- [llama_cpp docs](https://docs.rs/llama_cpp) – API documentation
- [llama-cpp-2 on crates.io](https://crates.io/crates/llama-cpp-2) – Lower-level bindings
- [llm_client on crates.io](https://crates.io/crates/llm_client) – High-level wrapper for local LLMs

### Additional Orchestration Frameworks
- [emersonmde/anchor-chain](https://github.com/emersonmde/anchor-chain) – Statically typed LLM workflow framework
- [anchor-chain docs](https://docs.rs/anchor-chain/latest/anchor_chain/) – API documentation
- [Rig](https://rig.rs/) – Official website
- [0xPlaygrounds/rig](https://github.com/0xPlaygrounds/rig) – GitHub repository
- [Rig documentation](https://docs.rig.rs/) – Official docs

### OpenRouter
- [OpenRouter](https://openrouter.ai) – Main website
- [OpenRouter Quickstart](https://openrouter.ai/docs/quickstart) – Getting started guide
- [OpenRouter API Reference](https://openrouter.ai/docs/api/reference/overview) – Complete API documentation
- [openrouter_api on crates.io](https://crates.io/crates/openrouter_api) – Rust client library
