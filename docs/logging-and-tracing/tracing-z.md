# Using `tracing` in Rust

### 1. Why `tracing`? The Problem It Solves

Before `tracing`, the `log` crate was the de facto standard for logging. It's simple and effective for basic use cases:

```rust
// Using the `log` crate
log::info!("User {} logged in.", user_id);
log::error!("Failed to connect to database: {}", error);
```

However, `log` has limitations, especially in modern, complex applications:

1. **Lack of Context:** It's difficult to correlate log messages that belong to a single logical operation, especially across asynchronous tasks (like handling a web request that involves multiple database queries and cache calls). You end up manually passing context IDs around.
2. **Unstructured Data:** While you can format strings, it's not truly structured. Parsing log output to find all errors related to a specific `user_id` is difficult.
3. **Not Async-Aware:** `log` has no concept of an asynchronous task's lifecycle. A log message is just a point in time, disconnected from the larger operation.

`tracing` was created to solve these problems. It's a framework for **structured, concurrent, and context-aware diagnostics**. It treats diagnostics as a core part of your application's logic.

### 2. Core Concepts of `tracing`

To understand `tracing`, you need to know its three main components:

| Concept | Analogy | Description |
| :--- | :--- | :--- |
| **`Span`** | A "To-Do List" Item | A unit of work or a period of time in your program. For example, "handling an HTTP request" or "executing a database query." Spans can be nested to form a tree, representing the call stack of an operation. |
| **`Event`** | A "Note" on a "To-Do List" Item | A point-in-time occurrence that happens within a span. For example, "user authenticated successfully" or "cache miss." Events are the things you actually log. |
| **`Subscriber`** | The "Notebook" | The entity that collects spans and events. It's the destination for all your diagnostic data. A subscriber can do anything with this data: print it to the console, send it to a logging service, or write it to a file. |
| **`Layer`** | A "Notebook Section" | A composable component that processes data from spans and events. A `Subscriber` can be composed of multiple `Layer`s. For example, one layer might format logs for the console, while another sends them to Jaeger for distributed tracing. |

### 3. Getting Started: Basic Instrumentation

Let's add `tracing` to a project.

**`Cargo.toml`**

```toml
[dependencies]
# The core tracing crate with macros
tracing = "0.1"

# A subscriber that prints to stdout. It's the most common way to get started.
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

The `env-filter` feature is crucial. It allows us to control the log level using an environment variable, which is a standard practice.

#### 3.1 The Easiest Way: `#[instrument]`

The `#[instrument]` attribute is the magic sauce of `tracing`. You can add it to any function to automatically create a span for that function's execution.

```rust
use tracing::{info, instrument};

// This function will now be traced!
#[instrument]
fn greet(name: &str) {
    // This `info!` event is part of the `greet` span.
    info!("Saying hello to {}", name);
}

fn main() {
    // Initialize the global subscriber. This should be done once in your application.
    tracing_subscriber::fmt::init();

    greet("Alice");
    greet("Bob");
}
```

**Running this code (with default `RUST_LOG=info`):**

```
$ cargo run
   Compiling tracing-example v0.1.0 (...)
    Finished dev [unoptimized + debuginfo] target(s) in ...
     Running `target/debug/tracing-example`
 INFO  tracing_example{name="Alice"}: tracing_example: Saying hello to Alice
 INFO  tracing_example{name="Bob"}: tracing_example: Saying hello to Bob
```

**What's happening here?**

1. `tracing_subscriber::fmt::init()` sets up a global subscriber that formats and prints spans and events to the console.
2. `#[instrument]` on `greet` creates a span named `greet`.
3. It automatically records the function's arguments (`name`) as fields on the span.
4. The `info!` macro creates an `Event` inside the active `greet` span. The subscriber prints both the span's context (`{name="Alice"}`) and the event's message.

#### 3.2 Creating Spans and Events Manually

While `#[instrument]` is great, you sometimes need more control.

```rust
use tracing::{info, span, Level, Instrument};

fn process_data(data: &[u8]) {
    // Manually create a span
    let span = span!(Level::DEBUG, "process_data", size = data.len());
    // "Enter" the span to make it the active context
    let _enter = span.enter();

    // This event will be associated with the `process_data` span
    info!("Data received, beginning processing.");

    // ... do some work ...

    info!("Processing complete.");
}

// You can also use the `Instrument` trait to attach a span to a Future
use std::time::Duration;
async fn background_job(job_id: u32) {
    let span = tracing::info_span!("background_job", job_id);
    background_job_logic(job_id)
        .instrument(span) // Attach the span to the future
        .await
}

async fn background_job_logic(job_id: u32) {
    // This `info!` will be inside the `background_job` span
    tracing::info!("Executing background job");
    tokio::time::sleep(Duration::from_millis(100)).await;
    tracing::info!("Job {} finished", job_id);
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    process_data(b"hello world");

    background_job(42).await;
}
```

**Running with `RUST_LOG=debug`:**

```
$ RUST_LOG=debug cargo run
  DEBUG tracing_example{size=11}: process_data: Data received, beginning processing.
  DEBUG tracing_example{size=11}: process_data: Processing complete.
 INFO  tracing_example{job_id=42}: background_job: Executing background job
 INFO  tracing_example{job_id=42}: background_job: Job 42 finished
```

### 4. The Power of `tracing-subscriber` and Layers

`tracing-subscriber` is the library you'll use to configure *how* your diagnostics are collected and displayed. Its power comes from its composable `Layer`s.

#### 4.1 The `Registry` and `EnvFilter`

The most common setup is a `Registry` subscriber with multiple layers.

* **`Registry`**: A subscriber that can hold multiple layers. It's the central hub.
* **`EnvFilter`**: A layer that filters spans and events based on their level and target (the crate/module path). This is controlled by the `RUST_LOG` environment variable.

```rust
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

fn main() {
    // Create a filter from the RUST_LOG environment variable
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info")); // Default to "info" if RUST_LOG is not set

    // Create a subscriber with multiple layers
    tracing_subscriber::registry()
        .with(env_filter) // Apply the filter
        .with(tracing_subscriber::fmt::layer()) // Add a layer that formats for the console
        .init(); // Initialize the global subscriber

    // ... your application code ...
}
```

**Common `RUST_LOG` settings:**

* `RUST_LOG=info`: Show `info`, `warn`, and `error` level events from all crates.
* `RUST_LOG=debug`: Show `debug`, `info`, `warn`, and `error` from all crates (very verbose!).
* `RUST_LOG=error`: Show only `error` level events.
* `RUST_LOG=my_crate=trace,warn`: Show `trace` level events from your crate (`my_crate`) and only `warn` and `error` from all other crates.
* `RUST_LOG=my_crate::web=info,my_crate::db=debug`: Set different levels for different modules within your crate.

#### 4.2 Other Popular Layers

* **`fmt::layer`**: The console formatter. It can be configured further.

    ```rust
    tracing_subscriber::fmt::layer()
        .json() // Output logs in JSON format (great for machines!)
        .with_target(false) // Don't print the crate/module target
        .with_thread_ids(true) // Print the thread ID
        .with_file(true) // Print the source file
    ```

* **`tracing-appender`**: A layer for writing logs to a file or a rolling file appender.

    ```toml
    # Cargo.toml
    tracing-appender = "0.2"
    ```

    ```rust
    use tracing_appender::rolling;

    let file_appender = rolling::daily("/var/log/my-app", "prefix.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(non_blocking) // Write to the file instead of stdout
        )
        .init();
    // _guard must be kept alive for the duration of your program.
    ```

### 5. Advanced Usage: Per-Request Tracing in a Web Server

This is where `tracing` truly shines. You can create a root span for each incoming request and have that context automatically propagate through all your handler's logic, even across `.await` points and into other functions.

Here's an example using `axum`, a popular web framework built by Tokio.

**`Cargo.toml`**

```toml
[dependencies]
axum = "0.7"
tokio = { version = "1.0", features = ["full"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
uuid = { version = "1.0", features = ["v4"] }
```

**`src/main.rs`**

```rust
use axum::{
    extract::Request,
    http::StatusCode,
    middleware::{self, Next},
    response::Response,
    routing::get,
    Router,
};
use std::time::Duration;
use tracing::{info, Instrument, Span};
use uuid::Uuid;

// Middleware to create a root span for each request
async fn request_tracer(req: Request, next: Next) -> Result<Response, StatusCode> {
    // Generate a unique ID for the request
    let request_id = Uuid::new_v4().to_string();

    // Create a span for this request
    let span = tracing::info_span!(
        "request",
        method = %req.method(),
        path = %req.uri().path(),
        request_id = %request_id,
    );

    // Run the rest of the request handling within this span's context
    let response = next.run(req).instrument(span).await;

    Ok(response)
}

// A handler that does some work
async fn process_request() -> String {
    // This event will be part of the request span created by the middleware
    info!("Starting to process the request logic.");

    // Simulate a database call
    simulate_db_call().await;

    info!("Finished processing request logic.");
    "Hello, World!".to_string()
}

// A simulated database function
#[tracing::instrument]
async fn simulate_db_call() {
    info!("Connecting to the database...");
    tokio::time::sleep(Duration::from_millis(50)).await;
    info!("Database query successful.");
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_target(false) // Less verbose for the example
        .init();

    // Build the application
    let app = Router::new()
        .route("/", get(process_request))
        .layer(middleware::from_fn(request_tracer));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
```

Now, run this and make a request with `curl`:

```bash
$ curl http://localhost:3000
Hello, World!
```

**Server output (with `RUST_LOG=info`):**

```sh
$ RUST_LOG=info cargo run
 INFO  request{method=GET path="/":request_id="..."}: request_tracer: Starting to process the request logic.
 INFO  simulate_db_call: Connecting to the database...
 INFO  simulate_db_call: Database query successful.
 INFO  request{method=GET path="/":request_id="..."}: request_tracer: Finished processing request logic.
```

Notice how the `process_request` logs are part of the `request` span, and the `simulate_db_call` logs are part of their own `simulate_db_call` span, which is a child of the `request` span. This hierarchy is incredibly powerful for debugging complex flows.

### 6. Popular Crates in the `tracing` Ecosystem

* **Subscribers & Layers:**
  * `tracing-subscriber`: The essential, all-in-one subscriber and layer collection. Includes `fmt`, `EnvFilter`, `Registry`, and more.
  * `tracing-appender`: For non-blocking file writers and rolling file appenders.
  * `tracing-wasm`: For sending traces to the browser's console in WebAssembly.
  * `tracing-bunyan-formatter`: A layer for Bunyan-style JSON logging.
  * `tracing-opentelemetry`: A bridge to the OpenTelemetry standard for distributed tracing. This sends your trace data to systems like Jaeger, Zipkin, or Honeycomb.

* **Integrations (Crates that natively support `tracing`):**
  * `tokio`: The async runtime is fully `tracing`-aware.
  * `axum`, `actix-web`, `warp`, `poem`: Most modern web frameworks have middleware or built-in support for `tracing`.
  * `tonic`: The gRPC library for Rust.
  * `sqlx`: The async SQL toolkit.
  * `reqwest`: The HTTP client.

* **Utilities:**
  * `tracing-futures`: Provides helpers for instrumenting futures (now largely superseded by the `Instrument` trait being in `tracing` itself, but still has some useful tools).
  * `tracing-error`: Provides utilities for creating and handling error events, making it easier to attach error context to spans.

### Summary

`tracing` is more than a logging library; it's a comprehensive instrumentation framework for building observable, resilient, and debuggable applications in Rust.

* **Start with `#[instrument]`** to get immediate benefits with minimal effort.
* **Use `tracing-subscriber`** to configure your logging output, especially with `EnvFilter` to control verbosity.
* **Leverage spans** to provide context for your operations, especially in async code and web servers.
* **Explore the ecosystem** for integrations with your favorite libraries and for advanced features like distributed tracing with OpenTelemetry.
