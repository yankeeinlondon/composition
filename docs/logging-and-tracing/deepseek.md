# Logging and Tracing

For modern Rust projects, especially those using async frameworks like Tokio, `tracing` has become the recommended go-to crate for structured logging and diagnostics. For simpler use cases, the traditional `log` crate ecosystem offers a straightforward, widely supported alternative. The choice between them depends on your project's complexity and your need for advanced features like structured data, async support, and distributed tracing.

Here’s a comparison to help you evaluate the main options at a glance:

| Feature | `tracing` Ecosystem | `log` Facade Ecosystem |
| :--- | :--- | :--- |
| **Core Design** | Structured, async-aware instrumentation with spans & events. | Simple, unstructured logging facade. |
| **Async Support** | Designed for async; spans handle `await` points correctly. | Not inherently async-friendly; may cause issues. |
| **Data Model** | **Spans** (periods of time) and **Events** (moments). Both support key-value fields. | Single log events with a level, target, and text message. |
| **Key Crates** | `tracing`, `tracing-subscriber`, `tracing-appender`, `tracing-opentelemetry`. | `log`, `env-logger`, `fern`, `log4rs`. |
| **Ease of Use** | Steeper initial learning curve, powerful for complex systems. | Very simple to start with; good for prototypes and CLI tools. |
| **Best For** | Async applications (Tokio, Axum), microservices, production systems requiring detailed observability. | Libraries (for broad compatibility), simple CLI tools, synchronous applications. |

### 🛠️ Implementing `tracing` in a Project

`tracing`'s power comes from its structured data and concept of spans, which represent a period of time during program execution.

**1. Basic Setup and Configuration:**
First, add the necessary crates:

```toml
[dependencies]
tracing = { version = "0.1", features = ["attributes"] }
tracing-subscriber = "0.3"
tracing-appender = "0.2" # For file logging
```

Then, initialize a subscriber early in your `main` function. Here's an example that logs to stdout with a filter:

```rust
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

fn main() {
    // Initialize a subscriber that logs to stdout
    tracing_subscriber::registry()
        .with(fmt::layer()) // Format and print logs
        .with(EnvFilter::from_default_env()) // Use RUST_LOG env var for filtering
        .init();

    // Your application code starts here
    info!("Application starting");
}
```

You can control verbosity at runtime with the `RUST_LOG` environment variable (e.g., `RUST_LOG=info cargo run`). The `EnvFilter` also lets you mute noisy dependencies.

**2. Instrumenting Your Code:**
The easiest way to add instrumentation is with the `#[instrument]` attribute, which automatically creates a span for a function, logging its arguments and execution.

```rust
use tracing::{info, instrument};
use axum::{Json, extract::State};

#[instrument(skip(db_pool))] // `skip` omits the db_pool from the span fields
async fn create_user(
    State(db_pool): State<PgPool>,
    Json(payload): Json<CreateUser>,
) -> Result<impl IntoResponse, StatusCode> {
    info!("Attempting to insert user into database");
    // ... database logic
    Ok(StatusCode::CREATED)
}
```

For more control, you can manually create events and spans:

```rust
use tracing::{event, span, Level, warn};

fn process_item(item: &Item) {
    // Create a span
    let span = span!(Level::INFO, "process_item", item_id = item.id);
    let _guard = span.enter(); // Enter the span

    if item.is_invalid() {
        // Record an event within the current span
        warn!("Encountered invalid item");
        event!(Level::ERROR, "error_details" = ?item.errors); // Structured field
    }
    // `_guard` is dropped here, exiting the span
}
```

Remember to use `.in_scope()` for spans that cross `await` points to avoid incorrect traces.

**3. Logging to Files:**
For production, you'll want to write logs to files using `tracing-appender`, which supports log rotation.

```rust
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::fmt;

fn setup_file_logging() {
    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY) // Rotate logs daily
        .filename_prefix("myapp.log")
        .build("/var/log/myapp")
        .expect("Failed to create file appender");

    let (non_blocking_writer, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_writer(non_blocking_writer)
        .init();
}
```

### 📝 Implementing the `log` Facade

The `log` crate provides a simple, universal API. Libraries use `log` macros, while executables choose a compatible logging implementation like `env-logger`.

**1. Basic Setup with `env-logger`:**
Add the crates to your `Cargo.toml`:

```toml
[dependencies]
log = "0.4"
env_logger = "0.11"
```

Initialize the logger in your application:

```rust
use log::info;

fn main() {
    // Initialize the logger, reading the filter from the RUST_LOG environment variable
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    info!("Application started with log facade");
    warn!("This is a warning");
    error!("This is an error");
}
```

Run with `RUST_LOG=debug cargo run` to see debug messages.

**2. Using the `log` Macros in Libraries:**
Libraries should only depend on `log` to remain compatible with any logging backend the user chooses.

```rust
use log::{debug, error, trace};

pub fn complex_operation(input: &str) -> Result<(), MyError> {
    trace!("Starting complex_operation with input: {}", input);
    
    let result = some_fallible_step(input);
    match result {
        Ok(val) => {
            debug!("Intermediate step successful: {:?}", val);
            Ok(())
        }
        Err(e) => {
            error!("Operation failed: {}", e);
            Err(e.into())
        }
    }
}
```

### 🤔 How to Choose and Next Steps

To decide which approach is best for you, consider your project type:

* **Choose `tracing` if:** You are building a **new async application** (e.g., with Tokio or Axum), a **production microservice** requiring distributed tracing, or need deep, **structured observability** to debug complex flows.
* **Choose the `log` facade if:** You are writing a **library** that needs maximum compatibility, building a **simple CLI tool or script**, or working on a **small, synchronous application** where advanced features are unnecessary.

For advanced `tracing` use cases, you can explore:

* **Distributed Tracing**: Use the `tracing-opentelemetry` crate to send traces to OpenTelemetry-compatible backends like Jaeger or Honeycomb.
* **Custom Layers**: The `tracing-subscriber` crate's `Layer` trait lets you compose custom behavior, like sending metrics and logs to different destinations.

I hope this deep dive helps you get started. If you can share more about your specific project (e.g., an async web service or a utility library), I can offer more tailored advice on configuration or architecture.
