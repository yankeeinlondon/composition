---
_fixed: true
---

# Comprehensive Guide to Rust Language Servers: Frameworks and Markdown Implementations

## 1 Introduction to Rust LSP Ecosystem

The **Language Server Protocol (LSP)** has become a foundational technology for providing intelligent language features in code editors and IDEs. In the Rust ecosystem, several powerful crates and frameworks have emerged to simplify the development of language servers. These tools enable developers to implement features like **code completion**, **diagnostics**, **hover information**, and **go-to-definition** for various programming languages and file formats. The Rust programming language, with its emphasis on performance and reliability, is particularly well-suited for implementing high-performance language servers that can handle large codebases efficiently.

The **Rust LSP ecosystem** consists of several key components: foundational crates that provide LSP protocol implementations, specialized libraries for handling language-specific parsing and analysis, and complete language server implementations for specific languages or file types. This ecosystem leverages Rust's **strong type system**, **memory safety guarantees**, and **asynchronous programming capabilities** to create robust and efficient language servers that can integrate with a wide range of development tools through the standardized LSP interface.

## 2 Popular Crates for Creating Language Servers in Rust

### 2.1 Core LSP Frameworks

When developing a language server in Rust, several foundational crates provide the necessary implementation of the Language Server Protocol. These frameworks handle the **protocol communication**, **message serialization**, and **server lifecycle management**, allowing developers to focus on the language-specific logic.

- **tower-lsp**: This is arguably the most popular and well-maintained framework for building language servers in Rust. It provides a **high-level abstraction** over the LSP using the Tower middleware pattern, which simplifies handling of asynchronous requests and responses. The framework includes comprehensive support for all LSP features including **code completion**, **hover**, **diagnostics**, and **text synchronization**. tower-lsp is built on Tokio for asynchronous runtime support and provides both stdio-based and TCP-based transport mechanisms.

- **lsp-types**: While not a complete framework itself, lsp-types is an essential crate that provides **type definitions** for all structures defined in the Language Server Protocol specification. It is maintained by the Rust community and stays up-to-date with the latest LSP versions. This crate is used by virtually all Rust-based language servers to ensure **protocol compliance** and **type safety** when handling LSP messages.

- **gen-lsp**: Another option that provides **code generation** tools for creating language servers from LSP specifications. While less commonly used than tower-lsp, it offers an alternative approach for developers who prefer code generation over framework abstractions.

### 2.2 Comparison of LSP Frameworks

*Table: Comparison of Popular Rust LSP Frameworks*

| **Feature** | **tower-lsp** | **lsp-types** | **gen-lsp** |
| :--- | :--- | :--- | :--- |
| **Type** | Full Framework | Type Definitions | Code Generator |
| **Async Support** | Excellent (Tokio-based) | N/A | Depends on generated code |
| **Ease of Use** | High (abstracts complexity) | N/A (used by others) | Moderate (requires generation step) |
| **Maintenance** | Very Active | Very Active | Less Active |
| **Key Feature** | Tower middleware pattern, full protocol handling | Complete, up-to-date LSP types | Generates code from spec |
| **Best For** | Most server implementations | Foundation for any LSP tool | Projects preferring generated code |

### 2.3 Deep Dive: `tower-lsp`

The `tower-lsp` crate distinguishes itself through its elegant use of the **Tower middleware pattern**, a common paradigm in the Rust networking ecosystem. This approach allows developers to compose layers of functionality to handle LSP requests. For example, you could have a middleware layer for logging, another for authentication, and a final layer that dispatches requests to the appropriate language-specific handler.

The crate provides a `LanguageServer` trait that developers must implement for their backend. This trait has methods corresponding to LSP requests like `initialize`, `completion`, `hover`, `goto_definition`, etc. The asynchronous nature of these methods, powered by the `async-trait` crate, allows the server to handle multiple concurrent requests efficiently without blocking.

Here is a basic example of a `tower-lsp` backend implementation, which demonstrates the core structure:

```rust
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

struct Backend {
    client: Client,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                // Announce which features the server supports
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions::default()),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    // Handler for textDocument/completion
    async fn completion(&self, _: CompletionParams) -> Result<Option<CompletionResponse>> {
        Ok(Some(CompletionResponse::Array(vec![
            CompletionItem::new_simple("Hello".to_string(), "Some detail".to_string()),
            CompletionItem::new_simple("Bye".to_string(), "More detail".to_string())
        ])))
    }

    // Handler for textDocument/hover
    async fn hover(&self, _: HoverParams) -> Result<Option<Hover>> {
        Ok(Some(Hover {
            contents: HoverContents::Scalar(
                MarkedString::String("You're hovering!".to_string())
            ),
            range: None
        }))
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let (service, socket) = LspService::new(|client| Backend { client });
    Server::new(stdin, stdout, socket).serve(service).await;
}
```

*Source: [`tower-lsp` documentation](https://docs.rs/tower-lsp/)*

This example shows how `tower-lsp` abstracts away the underlying JSON-RPC communication and transport layer, allowing the developer to focus purely on the language logic. The `LspService::new` function creates the service, and `Server::new` handles the I/O, connecting standard input/output to the language client.

## 3 Markdown Language Servers Written in Rust

While LSP was originally designed for programming languages, it has proven to be an excellent tool for enhancing authoring experiences for structured text formats like Markdown. Several specialized Markdown language servers have been developed in Rust, each catering to slightly different use cases such as note-taking, technical documentation, and personal knowledge management (PKM).

### 3.1 `markdown-oxide`

**markdown-oxide** is a feature-rich Language Server designed specifically for Personal Knowledge Management (PKM) workflows with Markdown. It aims to provide a powerful, IDE-like experience for managing interconnected notes and documentation.

- **Key Features**:
  - **Wiki-style link completion**: Autocomplete for links to other files in the workspace.
  - **Cross-references and diagnostics**: Validates that links point to existing files and headers, providing warnings for broken links.
  - **Go-to-definition**: Allows jumping to the file or header that a link points to.
  - **Live preview integration**: Designed to work with preview tools for a seamless editing experience.
- **Implementation**: It is written in Rust, leveraging its performance for quickly analyzing large collections of Markdown files. The project is actively maintained and positions itself as a "first-of-its-kind PKM anywhere tool".
- **Target Audience**: Users of PKM systems like Obsidian, Logseq, or those who manage a personal wiki using plain Markdown files.

### 3.2 `zeta-note`

**zeta-note** is another Markdown Language Server focused on making note-taking with Markdown more efficient and intelligent. It provides diagnostics and smart features to improve the writing and linking process.

- **Key Features**:
  - **Diagnostics**: Reports issues like broken links and duplicate headers within the workspace.
  - **Cross-references**: Provides features like go-to-definition for links and hover information to preview linked content.
  - **Completion**: Offers completion for file paths and headers when creating links.
- **Implementation**: The server is written in Rust and was inspired by the architecture of `rust-analyzer`, the premier language server for Rust. This suggests a focus on performance and robust incremental analysis.
- **Target Audience**: Developers, writers, and anyone who uses Markdown for extensive note-taking or documentation.

### 3.3 `notemancy-lsp`

**notemancy-lsp** is a companion language server for "Notemancy," a specific PKM application. While more specialized, it serves as an example of how LSPs can be tailored to particular tools.

- **Key Features**: As a companion tool, its features are likely tightly integrated with the Notemancy application's data model and workflow.
- **Implementation**: Written in Rust, it is listed on `crates.io`, indicating it's a packaged library.
- **Target Audience**: Users of the Notemancy PKM system.

*Table: Comparison of Rust-based Markdown LSPs*

| **Feature** | **markdown-oxide** | **zeta-note** | **notemancy-lsp** |
| :--- | :--- | :--- | :--- |
| **Primary Focus** | PKM and Wikis | Note-taking & Docs | Notemancy App |
| **Key Features** | Wiki-links, diagnostics, go-to-def | Diagnostics, cross-references | App-specific integration |
| **Status** | Active | Active | Likely Active |
| **Best For** | Obsidian-like workflows | General Markdown projects | Notemancy users |

### 3.4 A Note on `crates-lsp`

While not a Markdown LSP, **`crates-lsp`** is an excellent example of a specialized language server written in Rust. It provides language features for `Cargo.toml` files, such as **completion for crate names**, **diagnostics for outdated dependencies**, and **inlay hints** showing the latest version of a crate. It is a testament to the flexibility of the LSP and the power of Rust in building these tools.

## 4 Guidance for Developing a Language Server in Rust

For developers looking to create their own language server in Rust, the ecosystem provides a mature and powerful toolchain. Here is a recommended approach and key considerations:

1. **Choose Your Foundation**:
    - For most use cases, start with **`tower-lsp`**. It provides a robust, async-first framework that handles the boilerplate of the LSP, allowing you to focus on your language's logic.
    - You will inevitably use **`lsp-types`** as well, as `tower-lsp` re-exports it. It contains all the type definitions for the protocol's messages.

2. **Implement the `LanguageServer` Trait**:
    - The core of your server will be a struct that implements the `LanguageServer` trait from `tower-lsp`.
    - Start by implementing only the `initialize`, `initialized`, and `shutdown` methods to get a basic server running.
    - Then, add capabilities one by one (e.g., `completion`, `hover`, `text_document_definition`) by implementing the corresponding trait methods. Be sure to announce these capabilities in the `initialize` response.

3. **Language Parsing and Analysis**:
    - This is the most language-specific part. For structured languages, use a parser library.
    - **For programming languages**, `tree-sitter` is an excellent choice. It's a parser generator tool and incremental parsing library that is fast, robust, and produces a concrete syntax tree.
    - **For structured data like Markdown or TOML**, use dedicated crates like `markdown-rs` or `toml`. `markdown-rs` is a CommonMark compliant parser that can be used to build a Markdown LSP.

4. **State Management and Caching**:
    - An efficient language server must maintain state about the workspace, including the content of open files, parse results, and analysis data.
    - Design your backend to be stateless where possible, but maintain a central cache (e.g., in a `RwLock<HashMap<PathBuf, ParsedFile>>`) to store analysis results.
    - Use LSP's text synchronization notifications (`textDocument/didChange`, `textDocument/didOpen`, `textDocument/didClose`) to keep this cache up-to-date incrementally, avoiding re-parsing entire files on every keystroke.

5. **Concurrency**:
    - The LSP is inherently concurrent. A client can send multiple requests at once.
    - `tower-lsp` and Tokio handle this well. Ensure your language-specific analysis logic can be called concurrently without data races. Use Rust's ownership and concurrency primitives (`Arc`, `Mutex`, `RwLock`) to manage shared state safely.

## 5 Conclusion

The Rust ecosystem offers a powerful and mature set of tools for building high-performance Language Servers. The **`tower-lsp`** and **`lsp-types`** crates provide a solid foundation that abstracts away the complexities of the protocol, allowing developers to concentrate on language-specific analysis and features. The success of specialized servers like **`rust-analyzer`**, **`crates-lsp`**, and the emerging Markdown servers like **`markdown-oxide`** demonstrates the viability and strength of this approach. Whether you are looking to add intelligence to a programming language, a configuration file format, or a structured text language like Markdown, Rust provides the performance, safety, and tooling to build a best-in-class language server.
