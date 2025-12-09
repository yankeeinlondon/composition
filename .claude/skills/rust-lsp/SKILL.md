---
name: rust-lsp
description: Build Language Server Protocol (LSP) servers in Rust using tower-lsp, lsp-types, and related crates. Use when implementing IDE features like completion, diagnostics, hover, or go-to-definition for any language or file format.
---

# Rust LSP Development

Expert knowledge for building Language Server Protocol servers in Rust. LSP servers provide IDE features (completion, diagnostics, hover, navigation) to any editor supporting the protocol.

## Core Crates

| Crate | Purpose |
|:------|:--------|
| `tower-lsp` | High-level async LSP framework (recommended) |
| `lsp-types` | Protocol type definitions (always needed) |
| `lsp-server` | Lower-level sync framework (rust-analyzer style) |
| `async-lsp` | Tower-style middleware-centric alternative |

**Recommendation**: Start with `tower-lsp` + `lsp-types` for most projects.

## Quick Start

```rust
use tower_lsp::{LspService, Server, LanguageServer, Client};
use tower_lsp::lsp_types::*;
use tower_lsp::jsonrpc::Result;

struct Backend { client: Client }

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions::default()),
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::INCREMENTAL
                )),
                ..Default::default()
            },
            ..Default::default()
        })
    }
    async fn shutdown(&self) -> Result<()> { Ok(()) }
}

#[tokio::main]
async fn main() {
    let (service, socket) = LspService::new(|client| Backend { client });
    Server::new(tokio::io::stdin(), tokio::io::stdout(), socket)
        .serve(service).await;
}
```

## Key LSP Methods

| Method | Purpose | Trait Method |
|:-------|:--------|:-------------|
| `textDocument/completion` | Autocomplete | `completion()` |
| `textDocument/hover` | Tooltip info | `hover()` |
| `textDocument/definition` | Go to definition | `goto_definition()` |
| `textDocument/references` | Find references | `references()` |
| `textDocument/documentSymbol` | Outline/symbols | `document_symbol()` |
| `textDocument/publishDiagnostics` | Errors/warnings | `client.publish_diagnostics()` |

## Architecture Overview

```
Editor (VS Code, etc.)
    ↓ JSON-RPC via stdio/TCP
LSP Server (tower-lsp)
    ↓ Document sync notifications
Document Manager (state cache)
    ↓ Parse on change
Parser (pulldown-cmark, tree-sitter, etc.)
    ↓ AST/Events with source positions
Feature Handlers (completion, hover, etc.)
```

## Detailed Documentation

- [Core Frameworks](./frameworks.md) - tower-lsp, lsp-server, async-lsp comparison
- [LSP Capabilities](./capabilities.md) - Full list of LSP features and how to implement them
- [Document Management](./document-management.md) - State management, text ropes, incremental updates
- [Markdown LSPs](./markdown-lsps.md) - Existing Rust Markdown LSP implementations
- [Architecture Patterns](./architecture.md) - Plugin systems, event pipelines, caching strategies

## Essential Companion Crates

| Crate | Purpose |
|:------|:--------|
| `ropey` | Efficient text buffer for document state |
| `dashmap` | Concurrent hashmap for document cache |
| `tokio` | Async runtime (required by tower-lsp) |
| `tracing` | Structured logging |

## Cargo.toml

```toml
[dependencies]
tower-lsp = "0.20"
lsp-types = "0.95"
tokio = { version = "1", features = ["full"] }
ropey = "1"
dashmap = "5"
tracing = "0.1"
```

## Common Patterns

### Publishing Diagnostics
```rust
async fn validate(&self, uri: Url, text: &str) {
    let diagnostics = vec![Diagnostic {
        range: Range::new(Position::new(0, 0), Position::new(0, 5)),
        severity: Some(DiagnosticSeverity::ERROR),
        message: "Example error".into(),
        ..Default::default()
    }];
    self.client.publish_diagnostics(uri, diagnostics, None).await;
}
```

### Source Position Tracking
For LSP features, you need byte/line/column positions. Use parsers that provide source maps:
- `pulldown-cmark`: `parser.into_offset_iter()` returns `(Event, Range<usize>)`
- `tree-sitter`: Nodes have `start_position()` and `end_position()`
