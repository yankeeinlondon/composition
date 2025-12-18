---
name: lsp
description: Comprehensive guide to the Language Server Protocol (LSP) - architecture, implementation in Rust and TypeScript, integration with editors, and building custom language servers
created: 2025-12-08
_fixed: true
sources:
  - docs/lsp/articles.md
  - docs/lsp/deepseek.md
  - docs/lsp/openai.md
  - docs/lsp/videos.md
  - docs/lsp/for-markdown-zai.md
  - docs/lsp/for-markdown-gemini.md
  - docs/lsp/lsp-linters-and-formatters.md
  - docs/lsp/ts.md
  - docs/lsp/z.md
  - docs/lsp/strat-for-extending-markdown.md
tags:
  - lsp
  - language-server
  - rust
  - typescript
  - ide
  - tooling
---

# Language Server Protocol (LSP)

The Language Server Protocol (LSP) is a standardized JSON-RPC-based communication protocol that enables integration between code editors and language tools. Developed by Microsoft in 2016, LSP has revolutionized how language intelligence is delivered across development environments by solving the "M x N problem" - where M languages previously needed N separate editor integrations.

## Table of Contents

1. [Introduction and Core Concepts](#1-introduction-and-core-concepts)
2. [LSP Architecture](#2-lsp-architecture)
3. [Core Capabilities](#3-core-capabilities)
4. [Building Language Servers in Rust](#4-building-language-servers-in-rust)
5. [Building Language Servers in TypeScript](#5-building-language-servers-in-typescript)
6. [Editor Integration](#6-editor-integration)
7. [LSP and Linters/Formatters](#7-lsp-and-lintersformatters)
8. [Markdown Language Servers](#8-markdown-language-servers)
9. [Extending Markdown with Custom Syntax](#9-extending-markdown-with-custom-syntax)
10. [Web Development Language Servers](#10-web-development-language-servers)
11. [Implementation Patterns and Best Practices](#11-implementation-patterns-and-best-practices)
12. [Learning Resources](#12-learning-resources)
13. [Resources](#resources)

---

## 1. Introduction and Core Concepts

### What is LSP?

The Language Server Protocol defines a standardized interface between a **language server** (which provides language features) and a **language client** (integrated into editors). This separation means:

- Language support is implemented once and works across all LSP-compatible editors
- Editors don't need language-specific code
- Language tools can be written in any programming language
- Resource-intensive operations don't block the editor UI

### The M x N Problem

Before LSP, providing language support for M languages in N editors required M x N integrations. With LSP, this becomes M + N - each language implements one server, and each editor implements one client.

### Protocol Basics

LSP uses JSON-RPC 2.0 for communication. Messages are exchanged over various transports (stdio, TCP, pipes) with a simple header format:

```txt
Content-Length: <length>\r\n
\r\n
<JSON-RPC message>
```

---

## 2. LSP Architecture

### Client-Server Model

```txt
+------------------+     JSON-RPC      +------------------+
|   Editor/IDE     | <--------------> | Language Server  |
| (Language Client)|                   |                  |
+------------------+                   +------------------+
        |                                      |
        v                                      v
   User Interface                    Parser & Analyzer
   File Operations                   Symbol Tables
   UI Notifications                  Diagnostics Engine
```

### Three Main Components

1. **Workspace**: The directory containing files being worked on
2. **Editor (Client)**: Reads/writes files and communicates with the language server
3. **Language Server**: Provides language intelligence features

### Communication Flow

1. Client sends `initialize` request with capabilities
2. Server responds with its capabilities
3. Client sends `initialized` notification
4. Normal operation: requests, responses, and notifications
5. Client sends `shutdown` request followed by `exit` notification

---

## 3. Core Capabilities

Language servers implement various capabilities that enhance the coding experience:

| Capability | LSP Method | Description |
|------------|------------|-------------|
| **Completions** | `textDocument/completion` | Intelligent code completion based on context |
| **Hover** | `textDocument/hover` | Documentation and type information on hover |
| **Go to Definition** | `textDocument/definition` | Navigate to symbol definitions |
| **Find References** | `textDocument/references` | Locate all usages of a symbol |
| **Diagnostics** | `textDocument/publishDiagnostics` | Report errors, warnings, and hints |
| **Code Actions** | `textDocument/codeAction` | Quick fixes and refactoring suggestions |
| **Formatting** | `textDocument/formatting` | Code formatting |
| **Document Symbols** | `textDocument/documentSymbol` | Outline and symbol navigation |
| **Rename** | `textDocument/rename` | Symbol renaming across files |
| **Signature Help** | `textDocument/signatureHelp` | Function parameter information |

### Text Synchronization

Documents are synchronized between client and server through:

- `textDocument/didOpen` - Document opened
- `textDocument/didChange` - Document modified
- `textDocument/didClose` - Document closed
- `textDocument/didSave` - Document saved

Synchronization modes:

- **Full**: Entire document sent on each change
- **Incremental**: Only changes sent (more efficient)

---

## 4. Building Language Servers in Rust

### Core Crates

#### lsp-types

The de facto standard for LSP type definitions in Rust. Provides complete structs/enums for the LSP v3.16+ specification.

```toml
[dependencies]
lsp-types = "0.95"
```

Used by virtually all Rust LSP projects for protocol compliance and type safety.

#### tower-lsp

The recommended high-level framework for building LSP servers. Built on the Tower middleware pattern with Tokio async runtime.

```toml
[dependencies]
tower-lsp = "0.20"
tokio = { version = "1", features = ["full"] }
```

Key features:

- `LanguageServer` trait with async methods
- Handles JSON-RPC message dispatch and lifecycle
- Built-in stdio and TCP transport
- Runtime-agnostic async support

#### lsp-server

Lower-level scaffold from the rust-analyzer team. Synchronous API using crossbeam channels for servers wanting explicit control over threading.

```toml
[dependencies]
lsp-server = "0.7"
```

Best for: Servers needing custom concurrency models or maximum control.

#### async-lsp

Tower-style, middleware-centric alternative with pluggable layers for concurrency control, tracing, and cancellation.

### Choosing a Framework

| Framework | Best For | Complexity |
|-----------|----------|------------|
| **tower-lsp** | Most new projects | Low |
| **lsp-server** | Maximum control, custom threading | Medium |
| **async-lsp** | Middleware-heavy architectures | Medium |

### Basic tower-lsp Example

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

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn completion(&self, _: CompletionParams) -> Result<Option<CompletionResponse>> {
        Ok(Some(CompletionResponse::Array(vec![
            CompletionItem::new_simple("Hello".to_string(), "Detail".to_string()),
        ])))
    }

    async fn hover(&self, _: HoverParams) -> Result<Option<Hover>> {
        Ok(Some(Hover {
            contents: HoverContents::Scalar(
                MarkedString::String("Hover content".to_string())
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

### Supporting Crates for Rust LSP Development

| Crate | Purpose |
|-------|---------|
| `ropey` | Efficient text rope for large documents |
| `dashmap` | Concurrent hashmap for document caching |
| `tree-sitter` | Incremental parsing for programming languages |
| `pulldown-cmark` | CommonMark Markdown parsing |
| `markdown-rs` | Extensible Markdown parsing with AST |
| `comrak` | Full CommonMark + GFM implementation |
| `rowan` | Syntax tree library (used by rust-analyzer) |
| `text-size` | UTF-8 aware text positions |
| `tracing` | Structured logging |

---

## 5. Building Language Servers in TypeScript

### vscode-languageserver

The official Microsoft SDK for building language servers in Node.js/TypeScript.

```bash
npm install vscode-languageserver vscode-languageserver-textdocument
```

Key components:

- `vscode-languageserver`: Server-side protocol implementation
- `vscode-languageclient`: Client-side for VS Code extensions
- `vscode-languageserver-textdocument`: Text document management

### Basic TypeScript LSP Structure

```typescript
import {
    createConnection,
    TextDocuments,
    ProposedFeatures,
    InitializeParams,
    TextDocumentSyncKind,
    InitializeResult
} from 'vscode-languageserver/node';

import { TextDocument } from 'vscode-languageserver-textdocument';

const connection = createConnection(ProposedFeatures.all);
const documents: TextDocuments<TextDocument> = new TextDocuments(TextDocument);

connection.onInitialize((params: InitializeParams) => {
    const result: InitializeResult = {
        capabilities: {
            textDocumentSync: TextDocumentSyncKind.Incremental,
            completionProvider: {
                resolveProvider: true
            }
        }
    };
    return result;
});

connection.onCompletion((_textDocumentPosition) => {
    return [
        {
            label: 'TypeScript',
            kind: 1, // Text
            data: 1
        }
    ];
});

documents.listen(connection);
connection.listen();
```

---

## 6. Editor Integration

### Visual Studio Code

VS Code has built-in LSP support through its extension API:

- **Language Client**: Normal VS Code extension in JavaScript/TypeScript
- **Language Server**: Separate process providing intelligence
- **Extension Host**: Manages extension and server lifecycle

Configuration is done through `settings.json` and extension settings.

### Neovim

Neovim has native LSP support built into the editor core:

```lua
-- Enable a language server
vim.lsp.enable('rust_analyzer')

-- Configure a language server
vim.lsp.config('rust_analyzer', {
    cmd = {'rust-analyzer'},
    filetypes = {'rust'},
    root_dir = vim.fs.root(0, {'Cargo.toml', '.git'}),
    settings = {
        ['rust-analyzer'] = {
            checkOnSave = {
                command = 'clippy'
            }
        }
    }
})
```

Key differences from VS Code:

- Built-in LSP client (no extensions needed)
- Lua configuration instead of JSON
- Manual server management
- null-ls/none-ls for non-LSP tool integration

### Other Editors

| Editor | LSP Support |
|--------|-------------|
| **Vim** | `vim-lsp`, `coc.nvim` plugins |
| **Emacs** | `lsp-mode`, `eglot` packages |
| **Helix** | Built-in native support |
| **Sublime Text** | LSP package |
| **Kakoune** | `kak-lsp` plugin |

### Editor Comparison

| Feature | VS Code | Neovim |
|---------|---------|--------|
| LSP Client | Extension-based | Built-in |
| Configuration | GUI + JSON | Lua |
| Server Management | Automatic | Manual |
| External Tools | Extensions | null-ls/none-ls |
| Performance | Higher overhead | Lightweight |

---

## 7. LSP and Linters/Formatters

### Understanding the Relationship

| Feature | LSP | Linter | Formatter |
|---------|-----|--------|-----------|
| Error Detection | Yes (diagnostics) | Primary function | No |
| Style Checking | Sometimes | Sometimes | Primary function |
| Code Fixes | Yes (code actions) | Sometimes | Yes (automatic) |
| Navigation | Yes | No | No |
| Completion | Yes | No | No |
| Real-time Feedback | Yes | Usually manual | Usually manual |

### Linter Integration

Linters integrate with LSP in two ways:

1. **Native LSP**: Linter built into the language server (e.g., Ruff LSP)
2. **External wrapper**: Traditional linters wrapped for LSP compatibility (e.g., via null-ls)

### Formatter Integration

LSP formatting methods:

- `textDocument/formatting` - Format entire document
- `textDocument/rangeFormatting` - Format selection
- `textDocument/onTypeFormatting` - Format while typing

### Best Practices

- Avoid enabling duplicate functionality in multiple places
- Understand which features your LSP provides vs. external tools
- Consider performance impact of multiple servers
- Ensure team-wide configuration consistency

---

## 8. Markdown Language Servers

### Rust-based Markdown LSPs

#### Markdown-Oxide

PKM-focused LSP for wiki-style Markdown workflows.

- Wiki-link completion and validation
- Cross-references and backlinks
- Go-to-definition for links
- Obsidian/Logseq-inspired features
- Editor-agnostic via LSP

**Best for**: Personal knowledge management, note-taking systems

#### Quickmark

Markdown linter with first-class LSP support.

- Diagnostics (lint errors/warnings)
- Workspace-wide linting
- VS Code and JetBrains integrations

**Best for**: Markdown quality/style enforcement

#### mdBook-LS

Domain-specific LSP for mdBook documentation projects.

- Live preview integration
- Book-aware features
- Chapter navigation

**Best for**: mdBook documentation authors

#### zeta-note

Note-taking focused LSP inspired by rust-analyzer.

- Broken link diagnostics
- Duplicate header detection
- File path completion

**Best for**: Markdown-based note-taking

### TypeScript-based Markdown LSPs

#### remark-language-server

LSP built on the remark/unified ecosystem.

- Integrates with remark plugins
- Wide editor support
- Extensive Markdown processing capabilities

#### unified-language-server

Generic LSP for the unified pipeline (Markdown and related formats).

### Comparison

| LSP | Focus | Language |
|-----|-------|----------|
| Markdown-Oxide | PKM/Wiki | Rust |
| Quickmark | Linting | Rust |
| mdBook-LS | Documentation | Rust |
| zeta-note | Note-taking | Rust |
| remark-language-server | General Markdown | TypeScript |

---

## 9. Extending Markdown with Custom Syntax

### Strategy 1: Parser Extension (Recommended)

Use a parser with built-in extension points like `markdown-rs`:

```rust
use markdown::{to_mdast_with_options, Options, ParseOptions};

// Configure custom parsers
let mut parse_options = ParseOptions::default();
parse_options.constructs.text = Some(Box::new(parse_wikilink));

// Parse with extensions
let ast = to_mdast_with_options(text, &Options {
    parse: Some(parse_options),
    ..Options::default()
}).ok();
```

**Advantages**:

- Clean, isolated extension process
- Custom nodes are first-class AST citizens
- Easy to implement LSP features for custom syntax

### Strategy 2: Pre-processor Approach

Transform custom syntax to standard Markdown before parsing:

```rust
fn preprocess_custom_syntax(text: &str) -> (String, SourceMap) {
    // Convert [[wikilink]] to standard [link](url) syntax
    // Maintain source map for position translation
}
```

**Advantages**:

- Works with any parser
- Conceptually simple

**Disadvantages**:

- Source map management is complex
- Custom features lose semantic meaning

### Architecture for Extended Markdown LSP

```txt
+------------------+
|   Editor Client  |
+--------+---------+
         |
    JSON-RPC
         |
+--------v---------+
|   tower-lsp      |
|   LSP Server     |
+--------+---------+
         |
+--------v---------+
| Document Manager |
+--------+---------+
         |
+--------v---------+
|  markdown-rs     |
|  + Custom Parsers|
+--------+---------+
         |
+--------v---------+
|   Unified AST    |
| (Standard+Custom)|
+--------+---------+
         |
+--------v---------+
| LSP Handlers     |
| (hover, complete)|
+------------------+
```

### Markdown Parsing Crates

| Crate | Style | Best For |
|-------|-------|----------|
| `pulldown-cmark` | Event-based iterator | Speed, streaming |
| `comrak` | Full AST | GFM, rich structure |
| `markdown-rs` | Extensible AST | Custom syntax |

---

## 10. Web Development Language Servers

### TypeScript Language Servers

#### typescript-language-server (Community)

Thin LSP wrapper around TypeScript's tsserver.

```bash
npm install -g typescript-language-server typescript
```

Features: Code actions on save, workspace commands, inlay hints

#### vtsls

LSP wrapper around VS Code's TypeScript extension.

```bash
npm install -g @vtsls/language-server
```

Features: Near-identical VS Code experience, advanced refactoring

#### Comparison

| Feature | typescript-language-server | vtsls |
|---------|---------------------------|-------|
| VSCode Parity | Good | Excellent |
| Performance | Excellent | Very Good |
| Setup | Simple | Simple |
| Refactoring | Basic | Advanced |

### HTML Language Servers

- **vscode-html-language-server**: Official Microsoft server
- **SuperHTML**: Reports syntax errors (unlike VS Code's)

### CSS Language Servers

- **vscode-css-language-server**: CSS, LESS, SCSS support
- **Biome**: Fast Rust-based toolchain with CSS support

### Neovim TypeScript Configuration Example

```lua
vim.lsp.enable('vtsls')
vim.lsp.config('vtsls', {
    cmd = {'vtsls', '--stdio'},
    filetypes = {'javascript', 'typescript', 'typescriptreact'},
    root_dir = vim.fs.root(0, {'package.json', '.git'}),
    settings = {
        vtsls = {
            enableMoveToFileCodeAction = true,
            autoUseWorkspaceTsdk = true,
        },
        typescript = {
            updateImportsOnFileMove = { enabled = "always" },
            inlayHints = {
                parameterNames = { enabled = "literals" },
                parameterTypes = { enabled = true },
                variableTypes = { enabled = false },
            },
        },
    },
})
```

---

## 11. Implementation Patterns and Best Practices

### Document State Management

```rust
use dashmap::DashMap;
use ropey::Rope;

struct DocumentManager {
    documents: DashMap<Url, DocumentState>,
}

struct DocumentState {
    content: Rope,       // Efficient text buffer
    ast: SyntaxNode,     // Cached parse result
    version: i32,        // Document version
}
```

### Incremental Updates

Handle `textDocument/didChange` efficiently:

1. Apply changes to rope buffer
2. Determine affected regions
3. Re-parse only affected portions
4. Update cached analysis

### Caching Strategy

- Cache parse results and analysis data
- Invalidate on document changes
- Use incremental analysis where possible
- Consider lazy computation for expensive features

### Async Processing

```rust
// Use async/await for concurrent request handling
async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
    // Non-blocking analysis
    let document = self.documents.get(&params.text_document.uri);
    // ...
}
```

### Error Handling

- Use `anyhow` or `color-eyre` for rich error context
- Log errors with `tracing` for debugging
- Return graceful LSP errors to clients

### Testing Strategy

1. **Unit tests**: Individual parsing and analysis components
2. **Integration tests**: Full LSP server with mock client
3. **Performance tests**: Large document handling
4. **Specification tests**: Protocol compliance

---

## 12. Learning Resources

### Video Tutorials

#### Rust LSP

- [Build your own LSP using RUST](https://www.youtube.com/watch?v=dRxbqca6p60) - Step-by-step implementation
- [Let's read the Rust LSP server source code](https://www.youtube.com/watch?v=PNCldjGJTcU) - Source code walkthrough
- [Writing A Language Server Part 2](https://www.youtube.com/watch?v=JC4vmHVt59s) - TheVimeagen series

#### TypeScript LSP

- [LSP: Building a Language Server From Scratch](https://www.youtube.com/watch?v=Xo5VXTRoL6Q) - Comprehensive TypeScript tutorial
- [Language Server Shenanigans](https://www.youtube.com/watch?v=VUXROd82Ljk) - Real-world architecture

### Articles and Guides

#### General LSP

- [Language Server Extension Guide](https://code.visualstudio.com/api/language-extensions/language-server-extension-guide) - Official VS Code docs
- [Understanding the Language Server Protocol](https://www.freecodecamp.org/news/what-is-the-language-server-protocol-easier-code-editing-across-languages/) - freeCodeCamp overview

#### Rust-Specific

- [LSP outside the editor](https://medium.com/@selfint/lsp-outside-the-editor-431f77a9a4be) - JSON-RPC deep dive
- [Rust LSP server forum discussion](https://users.rust-lang.org/t/rust-tutorial-on-writing-a-lsp-server/75570) - Community resources

#### TypeScript-Specific

- [A Practical Guide for Language Server Protocol](https://medium.com/ballerina-techblog/practical-guide-for-the-language-server-protocol-3091a122b750) - Multi-part series
- [Getting Started with LSP](https://nabeelvalley.co.za/blog/2025/26-03/the-language-server-protocol/) - Ground-up implementation

---

## Resources

### Official Documentation

- [LSP Specification](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/)
- [VS Code Language Extensions](https://code.visualstudio.com/api/language-extensions/overview)
- [tower-lsp Documentation](https://docs.rs/tower-lsp/)
- [lsp-types Documentation](https://docs.rs/lsp-types/)

### Repositories

- [tower-lsp](https://github.com/ebkalderon/tower-lsp)
- [lsp-types](https://github.com/gluon-lang/lsp-types)
- [rust-analyzer](https://github.com/rust-lang/rust-analyzer) - Reference implementation
- [typescript-language-server](https://github.com/typescript-language-server/typescript-language-server)
- [vtsls](https://github.com/yioneko/vtsls)
- [markdown-oxide](https://github.com/Feel-ix-343/markdown-oxide)

### Tools

- [LSP Inspector](https://microsoft.github.io/language-server-protocol/inspector/) - Protocol debugging
- [nvim-lspconfig](https://github.com/neovim/nvim-lspconfig) - Neovim LSP configurations

---

## Sources

1. LSP Implementation Articles and Tutorials (docs/lsp/articles.md)
2. Building a Language Server in Rust - Core Crates (docs/lsp/deepseek.md)
3. Rust LSP Implementation Guide - Detailed Analysis (docs/lsp/openai.md)
4. LSP Server Implementation Videos (docs/lsp/videos.md)
5. Building an LSP Server for Extended Markdown (docs/lsp/for-markdown-zai.md)
6. Building a Custom Markdown LSP in Rust (docs/lsp/for-markdown-gemini.md)
7. LSP, Linters and Formatters Analysis (docs/lsp/lsp-linters-and-formatters.md)
8. TypeScript Language Servers Guide (docs/lsp/ts.md)
9. Comprehensive Guide to Rust Language Servers (docs/lsp/z.md)
10. Strategies for Extending Markdown LSP (docs/lsp/strat-for-extending-markdown.md)
