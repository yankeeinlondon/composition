# Rust LSP Crates

## Core LSP Frameworks

### tower-lsp (Recommended)
The most popular framework for building LSP servers in Rust. Built on the Tower middleware pattern.

```toml
[dependencies]
tower-lsp = "0.20"
tokio = { version = "1", features = ["full"] }
```

**Key Features:**
- High-level `LanguageServer` trait with async methods
- Handles JSON-RPC framing, protocol handshake, message dispatch
- Built-in concurrency via Tokio
- Supports stdio and TCP transports

### lsp-types
De-facto standard for LSP type definitions in Rust. Used by virtually all Rust LSP projects.

```toml
[dependencies]
lsp-types = "0.95"
```

Provides complete structs/enums for LSP v3.17+ spec: requests, notifications, capabilities, diagnostics.

### lsp-server (rust-analyzer team)
Lower-level scaffold with explicit control over the main loop.

```toml
[dependencies]
lsp-server = "0.7"
```

**Use when:**
- You need tight control over threading and scheduling
- Building heavy-duty servers like rust-analyzer
- tower-lsp feels too opinionated

### async-lsp
Tower-style, middleware-centric LSP core. Modern alternative to lsp-server.

```toml
[dependencies]
async-lsp = "0.2"
```

**Features:**
- Tower Layers for pluggable middleware
- Built-in concurrency control, tracing, cancellation
- Integrates with lsp-types and standard async/futures

## Decision Matrix

| Crate | Type | Async | Best For |
|-------|------|-------|----------|
| tower-lsp | Full Framework | Tokio | Most new projects |
| lsp-types | Type Definitions | N/A | All projects (foundational) |
| lsp-server | Low-level | crossbeam | Custom concurrency models |
| async-lsp | Middleware Framework | async-std/tokio | Modular architectures |

## Supporting Crates

### Text Handling
```toml
ropey = "1.6"          # Efficient text buffer for large files
text-size = "1.1"      # UTF-8 aware text positions
```

### Parsing
```toml
tree-sitter = "0.22"   # Incremental parsing for programming languages
pulldown-cmark = "0.10" # Fast CommonMark parser
markdown = "1.0.0-alpha" # markdown-rs with AST and extension points
comrak = "0.21"        # Full CommonMark + GFM with AST
```

### Utilities
```toml
dashmap = "5"          # Concurrent hashmap for document caching
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tracing = "0.1"        # Structured logging
anyhow = "1"           # Error handling
```

## Minimal tower-lsp Setup

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
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::INCREMENTAL,
                )),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![".".into(), ":".into()]),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client.log_message(MessageType::INFO, "Server initialized!").await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn completion(&self, _: CompletionParams) -> Result<Option<CompletionResponse>> {
        Ok(Some(CompletionResponse::Array(vec![
            CompletionItem::new_simple("example".into(), "An example completion".into()),
        ])))
    }

    async fn hover(&self, _: HoverParams) -> Result<Option<Hover>> {
        Ok(Some(Hover {
            contents: HoverContents::Scalar(MarkedString::String("Hover info".into())),
            range: None,
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
