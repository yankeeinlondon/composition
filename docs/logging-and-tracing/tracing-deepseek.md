# Using `tracing` in Rust

The `tracing` crate provides a powerful framework for structured, async-aware logging and diagnostics in Rust. It focuses on recording the *context* and *hierarchy* of events in your code, which is essential for debugging complex applications, especially asynchronous ones.

### 📚 Core Concepts

The API is built around three core concepts, which work together to create a complete picture of your program's execution:

| Concept | Description | Analogy |
| :--- | :--- | :--- |
| **Spans** | A period of time with a beginning and end (e.g., handling an HTTP request, a database transaction). | A chapter in a book. |
| **Events** | A moment in time that occurred within a span (e.g., "cache miss," "validation failed"). | A specific sentence or event within that chapter. |
| **Subscribers** | Collects, processes, and outputs the data from spans and events (e.g., to the console, a file, or a remote system). | The publisher that prints the book. |

### 🛠️ Basic Setup and Instrumentation

To start, add the necessary crates to your `Cargo.toml`. For a binary application, you typically need both `tracing` and a subscriber like `tracing-subscriber`.

```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = "0.3"
```

#### Initializing a Subscriber

Before you can record traces, you must initialize a `Subscriber`. The simplest setup writes formatted logs to `stdout`:

```rust
use tracing_subscriber;

fn main() {
    // Sets up a basic subscriber that prints to stdout
    tracing_subscriber::fmt::init();

    // Your instrumented application code here
}
```

#### Creating Spans and Events

You record data using the `span!` and `event!` macros, or their level-specific shortcuts like `info_span!` and `error!`.

```rust
use tracing::{info, info_span, warn};

fn process_order(order_id: u64) {
    // Create and enter a span named "process_order"
    let span = info_span!("process_order", order_id);
    let _enter = span.enter();

    // Record events within this span
    info!("Starting to process order");
    // ... application logic ...
    warn!("Inventory low for order item");
    // Span exits when `_enter` is dropped at the end of the function.
}
```

The `#[instrument]` attribute automatically creates a span for a function, capturing its arguments:

```rust
use tracing::{info, instrument};

#[instrument] // Creates a span named `fetch_user_data` with `user_id` as a field
async fn fetch_user_data(user_id: u64) -> Result<(), ()> {
    info!("Fetching data from database");
    // ... async work ...
    Ok(())
}
```

**Crucial for Async Code**: To avoid incorrect traces, do **not** hold a span guard across an `.await` point. Instead, use `span.in_scope()` for synchronous sections or ensure the guard is dropped before awaiting.

```rust
// SAFE: Using `in_scope` for synchronous work
async fn my_function() {
    let span = info_span!("my_async_span");
    let result = span.in_scope(|| {
        // Do synchronous work inside the span
        some_sync_work()
    });
    // Span is exited before the await
    some_other_async_function(result).await;
}
```

### 🔧 Configuration and Popular Companion Crates

You build a production-ready observability setup by composing `Layers` onto a `Subscriber`. Key configuration involves filtering and output.

#### Filtering with `EnvFilter`

Use `EnvFilter` to dynamically control verbosity, often via the `RUST_LOG` environment variable.

```rust
use tracing_subscriber::{fmt, EnvFilter};

tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::from_default_env()) // e.g., RUST_LOG=info,my_crate=trace
    .init();
```

#### Popular Crates in the Ecosystem

The `tracing` ecosystem is modular. Here are essential crates for different tasks:

| Crate | Primary Use Case | Key Feature |
| :--- | :--- | :--- |
| **`tracing-subscriber`** | Subscriber implementation | Provides utilities for building and composing `Subscriber`s. |
| **`tracing-appender`** | Log file management | Offers non-blocking, rotating file appenders. |
| **`tracing-opentelemetry`** | Distributed tracing | Exports traces to OpenTelemetry collectors (e.g., Jaeger, Honeycomb). |
| **`tower-http`** `trace` feature | HTTP request tracing | Auto-instruments HTTP requests for Axum/Tower-based web services. |
| **`tracing-tree`** | Debugging | Prints a hierarchical, tree-like view of spans. |
| **`tracing-flame`** | Performance profiling | Generates flame graphs from span data. |

#### Common Configuration Patterns

When configuring tracing for production, these are key considerations:

* **Sampling**: In high-throughput services, sample traces (e.g., 10%) to manage volume and cost.
* **Context Propagation**: Ensure trace IDs are passed across service boundaries (e.g., via HTTP headers) for distributed tracing.
* **Structured Fields**: Use the `%` (Display) and `?` (Debug) sigils in macros to record structured data.

    ```rust
    event!(
        Level::ERROR,
        error = ?e, // Record error using Debug format
        url = %request_url, // Record URL using Display format
        "Request failed"
    );
    ```

* **Output Destinations**: Configure layers to send data to multiple destinations simultaneously (e.g., stdout for local dev, OpenTelemetry for production).

A more complex setup for a web service might look like this:

```rust
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Registry};
use tracing_tree::HierarchicalLayer;

// For file logging
let file_appender = tracing_appender::rolling::daily("/var/log", "api.log");
let (non_blocking_writer, _guard) = tracing_appender::non_blocking(file_appender);

// Build the subscriber
Registry::default()
    // Tree view for debugging
    .with(HierarchicalLayer::new(2))
    // Formatted logs to stdout
    .with(tracing_subscriber::fmt::layer().pretty())
    // JSON logs to a file
    .with(tracing_subscriber::fmt::layer().json().with_writer(non_blocking_writer))
    // Filter based on RUST_LOG
    .with(tracing_subscriber::EnvFilter::from_default_env())
    .init(); // Set as the global default subscriber
```

### 🚀 Integration Example: Web Server with Tower-HTTP

Here is a practical example of instrumenting an Axum web server using `tower-http`'s `TraceLayer`, which automatically creates a span for each request:

```rust
use axum::{routing::get, Router};
use tower_http::trace::{TraceLayer, DefaultMakeSpan, DefaultOnResponse};
use tracing::Level;

#[tokio::main]
async fn main() {
    // Initialize the subscriber
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/", get(|| async { "Hello, world!" }))
        // Apply tracing middleware
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(
                    DefaultMakeSpan::new()
                        .level(Level::INFO)
                        .include_headers(true), // Optional: log request headers
                )
                .on_response(
                    DefaultOnResponse::new()
                        .level(Level::INFO)
                        .include_headers(true), // Optional: log response headers
                ),
        );

    axum::serve(
        tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap(),
        app
    ).await.unwrap();
}
```

In summary, start by adding `tracing` and `tracing-subscriber`, and use the `#[instrument]` attribute on key functions. For production, layer in filters, file output, and potentially `tracing-opentelemetry` for distributed tracing.

If you are building a specific type of application, such as a high-performance API server or a background job processor, I can offer more tailored advice on which crates and configuration options would be most beneficial.
