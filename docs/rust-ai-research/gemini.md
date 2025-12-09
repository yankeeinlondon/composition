# AI Crates for Rust

## 💡 Rust LLM/Agent Development Crates

| Crate Name | Functional Footprint | Core Features |
| :--- | :--- | :--- |
| **`llm`** | **Unified Provider SDK & Basic Orchestration** | Provides a unified, high-level interface across multiple major LLM providers (OpenAI, Anthropic, Google, Ollama, etc.). Includes built-in support for **tool calling**, **streaming**, **embeddings**, and multi-step **prompt chains**. This is the closest analog to a feature-rich, multi-provider SDK. |
| **`rig`** | **LLM/RAG Abstraction Framework** | A modular framework designed for building scalable LLM applications, particularly focusing on **Retrieval-Augmented Generation (RAG)** systems. It offers unified interfaces for LLM access, embeddings, and deep integration with various **vector stores** (e.g., Qdrant, MongoDB) via companion crates. |
| **`rustchain-community`** | **AI Agent Framework/Workflow Orchestration** | Positioned as a memory-safe, high-performance AI agent framework with a focus on **workflow transpilation** (e.g., converting between formats like LangChain, Kubernetes configs). Aims to provide the performance benefits of Rust for multi-step agent and pipeline creation. |
| **`runagent`** | **Multi-Framework Agent SDK (Client/Server)** | Primarily a Rust **SDK for deploying and managing AI agents** built using *other* frameworks (like Python's LangChain or LangGraph). It includes a client/server architecture for interaction and history storage. Its primary footprint is on the operational side, interacting with agents defined elsewhere, but it brings powerful agent management tools to Rust. |

-----

## 🛠️ Detailed Solutions and Examples

### 1\. `llm` (Unified Provider SDK & Chains)

The `llm` crate abstracts away the differences between major LLM APIs, providing a coherent set of traits and structs for chat, streaming, and advanced features like tool calling. It provides the base layer of "AI SDK" functionality and includes basic "chain" functionality.

* **Functional Footprint:** Unified, high-level client for over a dozen LLM and AI-related backends (including text-to-speech/speech-to-text), structured output via JSON schema, conversational memory, and chaining multiple steps/providers.
* **Example: Basic Chat Completion**

<!-- end list -->

```rust
use llm::{Chat, Result, llm_chat};
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    // Requires setting the OPENAI_API_KEY environment variable
    let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");

    let chat = llm_chat!("openai:gpt-4o", &api_key)
        .await?
        .with_system_message("You are a concise Rust programming expert.");

    let response = chat
        .send_user_message("Write a short example of a Rust struct and its associated method.")
        .await?;

    println!("LLM Response:\n{}", response.content);

    Ok(())
}
```

### 2\. `rig` (LLM/RAG Abstraction Framework)

`rig` is designed to be a building block for more complex applications, focusing on the core components of RAG: LLM interface, embedding generation, and vector store integration.

* **Functional Footprint:** Unified `CompletionModel` and `EmbeddingModel` traits. Decoupled, feature-flagged integrations for numerous vector stores (Qdrant, MongoDB, LanceDB, etc.) and embedding providers. Facilitates building robust RAG pipelines.
* **Example: Embedding Generation**

<!-- end list -->

```rust
use rig_core::{
    embedding::{EmbeddingModel, EmbeddingRequest, Vector},
    provider::openai::OpenAIClient,
};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("OPENAI_API_KEY")?;
    
    // Initialize the OpenAI client and specify the embedding model
    let client = OpenAIClient::new(&api_key)?;
    let embedding_model = client.embedding_model("text-embedding-ada-002");

    let request = EmbeddingRequest::new(vec![
        "The quick brown fox jumps over the lazy dog.".to_string(),
        "Rust is a systems programming language.".to_string(),
    ]);

    let response = embedding_model.generate_embeddings(request).await?;

    for (i, vector) in response.embeddings.iter().enumerate() {
        println!("Embedding {} (Dim: {}): {:?}", i + 1, vector.len(), &vector[0..5]);
    }

    Ok(())
}
```

### 3\. `rustchain-community` (AI Agent Framework)

This crate positions itself as a high-performance alternative to Python's LangChain, with an emphasis on creating complex, multi-step workflows/missions and high-speed execution.

* **Functional Footprint:** YAML-based workflow definition (missions), core agent orchestration, performance optimization, and transpilation between various workflow formats. Ideal for defining agent logic in a structured, external format.
* **Example: Basic Mission Definition (YAML)**

While the crate's usage often involves a CLI tool for execution, here's how a basic multi-step *mission* (workflow) is defined, which is central to its footprint:

```yaml
version: '1.0'
name: document_summary_mission
description: Summarize a given document using an LLM.
steps:
  - id: load_document
    name: Read File
    step_type: command
    parameters:
      command: cat document.txt # Hypothetical: loads content into context
      output_key: document_content
  - id: summarize
    name: LLM Summarization
    step_type: llm_completion
    parameters:
      model: gpt-3.5-turbo
      prompt: "Summarize the following text in three bullet points: {{document_content}}"
      output_key: summary_result
  - id: final_output
    name: Print Summary
    step_type: print
    parameters:
      content: "{{summary_result}}"
```

-----

## 🤝 Composable Solutions

Since a single crate hasn't reached the dominance of LangChain, combining a foundational library with an application-level library is the best approach for comprehensive LLM applications in Rust.

### 1\. Unified RAG Pipeline

* **Goal:** Build a high-performance, memory-safe RAG (Retrieval-Augmented Generation) system.
* **Combination:** **`rig`** (for RAG tooling) + **`llm`** (for multi-provider LLM access).
* **Rationale:** `rig` provides the superior framework for handling vector store integration, embeddings, and the overall RAG workflow architecture. `llm` can be used as one of the underlying `CompletionModel` implementations within the `rig` structure, allowing you to easily swap out LLM providers (OpenAI, Anthropic, Google) without changing your RAG logic.

### 2\. Complex Agent Orchestration

* **Goal:** Define and execute complex, multi-step agentic workflows and chains.
* **Combination:** **`rustchain-community`** (for workflow definition and execution) + **`llm`** (for core LLM/Tool calling).
* **Rationale:** Use `rustchain-community` to structure your high-level pipeline logic (the "LangGraph" part) via its YAML missions, taking advantage of its performance and defined execution steps. Use `llm` internally within the `llm_completion` or `tool_calling` steps of the mission to interact with the external LLM APIs, leveraging its unified interface for chat and tool management.

Would you like to explore a specific feature in one of these crates, such as **tool calling** or **RAG implementation details**?
