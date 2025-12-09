# LSP Framework Comparison

## tower-lsp (Recommended)

**"Batteries-included framework built on Tower"**

```rust
use tower_lsp::{LspService, Server, LanguageServer, Client};

struct Backend { client: Client }

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    // Implement trait methods for each LSP feature
}
```

**Pros:**
- High-level abstraction, minimal boilerplate
- Async-first design using Tokio
- Handles JSON-RPC, message dispatch, lifecycle
- Built on Tower middleware pattern (composable layers)
- Active maintenance, good documentation

**Cons:**
- Less control over server main loop
- Tied to Tower/Tokio ecosystem

**Best For:** Most new LSP projects

## lsp-server

**"Lower-level scaffold from rust-analyzer team"**

```rust
use lsp_server::{Connection, Message, Request, Response};

fn main() -> Result<()> {
    let (connection, io_threads) = Connection::stdio();

    // Manual initialization handshake
    let capabilities = /* ... */;
    connection.initialize(capabilities)?;

    // Manual main loop
    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => handle_request(req),
            Message::Notification(not) => handle_notification(not),
            _ => {}
        }
    }
    io_threads.join()?;
    Ok(())
}
```

**Pros:**
- Full control over main loop and threading
- Synchronous API using crossbeam channels
- Battle-tested in rust-analyzer
- No async runtime required

**Cons:**
- More boilerplate
- Manual message routing
- Must implement concurrency yourself

**Best For:** Complex servers needing fine-grained control, or when avoiding async

## async-lsp

**"Tower-style middleware-centric LSP core"**

```rust
use async_lsp::{LspService, MainLoop};

// Uses Tower Layers for middleware
let service = tower::ServiceBuilder::new()
    .layer(TracingLayer::default())
    .layer(ConcurrencyLayer::new(4))
    .service(MyLanguageServer::new());
```

**Pros:**
- Modular middleware system
- Built-in concurrency control, tracing, cancellation
- Modern async design
- Works with standard futures ecosystem

**Cons:**
- Less mature than tower-lsp
- Smaller community

**Best For:** Projects wanting maximum middleware composability

## lsp-types

**"De-facto standard Rust types for LSP"**

Not a framework - provides type definitions for all LSP structures:
- Request/response types (`InitializeParams`, `CompletionItem`, etc.)
- Capabilities enums
- Protocol version tracking

**Used by all frameworks.** You'll always depend on this.

```rust
use lsp_types::{
    InitializeParams, InitializeResult, ServerCapabilities,
    CompletionParams, CompletionItem, CompletionItemKind,
    Diagnostic, DiagnosticSeverity, Position, Range, Url,
};
```

## Comparison Table

| Feature | tower-lsp | lsp-server | async-lsp |
|:--------|:----------|:-----------|:----------|
| Abstraction Level | High | Low | Medium |
| Async Model | Tokio | Sync (crossbeam) | Tower/futures |
| Main Loop | Managed | Manual | Managed |
| Middleware | Limited | DIY | Tower Layers |
| Learning Curve | Easy | Medium | Medium |
| Control | Less | Full | Medium |
| Maturity | High | High | Growing |

## Choosing a Framework

```
Start here:
    ↓
Need maximum control? → lsp-server
    ↓ no
Using Tokio already? → tower-lsp
    ↓ no
Want middleware composability? → async-lsp
    ↓ no
Default choice → tower-lsp
```

## Migration Path

All frameworks use `lsp-types`, so the language-specific logic (parsing, analysis) is portable. Only the server scaffolding differs.

## Legacy Note

**`languageserver-types`** is an older crate superseded by `lsp-types`. Avoid for new projects.
