# The tracing Framework

`tracing` is a structured, async-aware framework for diagnostics. Unlike traditional logging, it captures **spans** (periods of time) and **events** (moments), providing rich context for understanding async program flow.

## Core Concepts

| Concept | Description | Analogy |
|:--------|:------------|:--------|
| **Span** | A period of time (e.g., handling a request) | A chapter in a book |
| **Event** | A moment within a span (e.g., "cache miss") | A sentence in that chapter |
| **Subscriber** | Collects and processes span/event data | The printer |
| **Layer** | Composable component for processing | A processing step |

## Basic Setup

```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

```rust
use tracing::{info, warn, error, debug, span, Level};
use tracing_subscriber::{fmt, EnvFilter};

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    info!("Application started");
}
```

## The #[instrument] Attribute

Automatically create spans for functions, capturing arguments:

```rust
use tracing::instrument;

#[instrument]
fn process_order(order_id: u64, user_id: u64) {
    tracing::info!("Processing order");
    // Arguments automatically logged as span fields
}

#[instrument(skip(password))]  // Don't log sensitive data
async fn authenticate(user: &str, password: &str) -> bool {
    tracing::debug!("Authenticating user");
    true
}

#[instrument(name = "custom_span_name", level = "debug")]
fn with_custom_options() {}
```

## Manual Spans

```rust
use tracing::{span, Level, info};

fn process_data(data: &[u8]) {
    let span = span!(Level::INFO, "process_data", size = data.len());
    let _guard = span.enter();  // Span active until _guard dropped

    info!("Processing started");
    // ... work ...
    info!("Processing complete");
}
```

## Async Spans with .instrument()

For async code, use `.instrument()` to attach spans to futures:

```rust
use tracing::{info_span, Instrument};

async fn handle_request(req_id: u64) {
    let span = info_span!("request", %req_id);

    async move {
        tracing::info!("Processing request");
        some_async_work().await;
    }
    .instrument(span)
    .await;
}
```

**Important:** Don't hold span guards across `.await` points:
```rust
// BAD - span guard held across await
async fn bad_example() {
    let span = info_span!("my_span");
    let _guard = span.enter();
    some_async_work().await;  // Guard still held!
}

// GOOD - use .instrument() or in_scope()
async fn good_example() {
    let span = info_span!("my_span");
    async { some_async_work().await }.instrument(span).await;
}
```

## Structured Fields

```rust
use tracing::{info, warn, Level, event};

// Shorthand macros
info!(user_id = 123, action = "login", "User logged in");
warn!(retries = 3, "Connection unstable");

// Display vs Debug formatting
info!(url = %request_url, "Fetching URL");      // Uses Display
info!(error = ?err, "Operation failed");        // Uses Debug

// event! macro for dynamic levels
event!(Level::ERROR, error = ?e, "Request failed");
```

## Subscriber Configuration

### Basic with EnvFilter

```rust
use tracing_subscriber::{fmt, EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

tracing_subscriber::registry()
    .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
    .with(fmt::layer())
    .init();
```

### JSON Output

```rust
tracing_subscriber::fmt()
    .json()
    .with_target(false)
    .with_thread_ids(true)
    .init();
```

### Multiple Layers

```rust
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, Registry, EnvFilter};
use tracing_appender::rolling;

// File appender with daily rotation
let file_appender = rolling::daily("/var/log/app", "myapp.log");
let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

tracing_subscriber::registry()
    .with(EnvFilter::from_default_env())
    .with(fmt::layer())  // Console output
    .with(fmt::layer().json().with_writer(non_blocking))  // JSON to file
    .init();

// IMPORTANT: Keep _guard alive for program duration
```

## File Logging with Rotation

```toml
[dependencies]
tracing-appender = "0.2"
```

```rust
use tracing_appender::rolling::{RollingFileAppender, Rotation};

let file_appender = RollingFileAppender::builder()
    .rotation(Rotation::DAILY)
    .filename_prefix("myapp.log")
    .build("/var/log/myapp")
    .expect("Failed to create appender");

let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

tracing_subscriber::fmt()
    .with_writer(non_blocking)
    .init();
```

## Bridging log to tracing

Capture `log` crate records as tracing events:

```rust
use tracing_log::LogTracer;

fn main() {
    LogTracer::init().expect("Failed to set logger");

    tracing_subscriber::fmt::init();

    // Now both work through tracing
    log::info!("From log crate");
    tracing::info!("From tracing crate");
}
```

## Web Server Integration (Axum)

```rust
use axum::{Router, routing::get, middleware};
use tower_http::trace::TraceLayer;
use tracing::Level;

let app = Router::new()
    .route("/", get(handler))
    .layer(
        TraceLayer::new_for_http()
            .make_span_with(|request: &Request| {
                info_span!(
                    "request",
                    method = %request.method(),
                    path = %request.uri().path(),
                )
            })
    );
```

## RUST_LOG Filtering

```bash
# Global level
RUST_LOG=info

# Per-crate
RUST_LOG=warn,my_app=debug,hyper=info

# Span-based (tracing-specific)
RUST_LOG=my_app[handle_request]=trace

# Field-based
RUST_LOG=my_app[{user_id=123}]=debug
```

## Ecosystem Crates

| Crate | Purpose |
|-------|---------|
| `tracing-subscriber` | Subscriber implementation, formatting, filtering |
| `tracing-appender` | Non-blocking file writers, rotation |
| `tracing-log` | Bridge log records to tracing |
| `tracing-opentelemetry` | Export to OpenTelemetry backends |
| `tracing-tree` | Hierarchical tree-like output |
| `tracing-flame` | Generate flame graphs from spans |
| `tower-http` | HTTP middleware with tracing integration |
