---
name: lsp
description: Expert knowledge for building Language Server Protocol (LSP) implementations in Rust or TypeScript. Use when creating language servers, implementing LSP features like completion, diagnostics, hover, or go-to-definition, choosing LSP crates/libraries, or working with editor integrations via LSP.
---

# Language Server Protocol (LSP)

The LSP is a JSON-RPC-based protocol that enables communication between code editors and language tools. It standardizes how language intelligence (completions, diagnostics, navigation) is delivered across different development environments.

## Core Principles

- **tower-lsp + lsp-types** is the recommended starting point for Rust LSP servers
- **vscode-languageserver** is the canonical approach for TypeScript/Node.js servers
- Always announce capabilities in the `initialize` response before implementing features
- Use async/concurrent processing - LSP clients send multiple requests simultaneously
- Maintain document state efficiently using text ropes (e.g., `ropey` crate) for incremental updates
- Cache parse results and invalidate on `textDocument/didChange` notifications
- Source maps are critical - always track positions back to original source for accurate LSP features
- Implement `textDocument/didOpen`, `didChange`, `didClose` for proper document synchronization
- Test with multiple editors (VS Code, Neovim, Helix) as implementations vary

## Quick Reference: Rust LSP Server

```rust
use tower_lsp::{LspService, Server, LanguageServer, Client};
use tower_lsp::lsp_types::*;

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

## Topics

### Rust Implementation
- [Rust LSP Crates](./rust-crates.md) - Core crates: tower-lsp, lsp-types, lsp-server, async-lsp
- [Rust Architecture](./rust-architecture.md) - Document management, caching, incremental parsing

### TypeScript Implementation
- [TypeScript LSP](./typescript-lsp.md) - vscode-languageserver, tsserver integration, vtsls

### LSP Features
- [Core Capabilities](./capabilities.md) - Completion, diagnostics, hover, navigation, formatting
- [Editor Integration](./editors.md) - VS Code, Neovim, Helix configuration differences

### Markdown-Specific
- [Markdown LSPs](./markdown-lsp.md) - Existing implementations: markdown-oxide, marksman, quickmark
- [Extending Markdown](./extending-markdown.md) - Plugin architecture with markdown-rs, custom syntax

## Common Patterns

### Document State Management
```rust
use dashmap::DashMap;
use ropey::Rope;

struct DocumentManager {
    documents: DashMap<Url, DocumentState>,
}

struct DocumentState {
    content: Rope,
    version: i32,
    // Cached parse results
    ast: Option<SyntaxTree>,
}
```

### Publishing Diagnostics
```rust
async fn publish_diagnostics(&self, uri: Url, diagnostics: Vec<Diagnostic>) {
    self.client.publish_diagnostics(uri, diagnostics, None).await;
}
```

## Resources

- [LSP Specification](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/)
- [VS Code Language Server Guide](https://code.visualstudio.com/api/language-extensions/language-server-extension-guide)
- [tower-lsp docs](https://docs.rs/tower-lsp/)
- [lsp-types docs](https://docs.rs/lsp-types/)
