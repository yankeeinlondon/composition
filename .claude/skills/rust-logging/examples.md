# Code Examples

Working examples for common logging and tracing scenarios.

## Simple CLI Application

```toml
[dependencies]
log = "0.4"
env_logger = "0.11"
```

```rust
use log::{info, warn, error, debug};

fn main() {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info")
    ).init();

    info!("CLI tool starting");

    match run_command() {
        Ok(result) => {
            info!("Command succeeded: {}", result);
        }
        Err(e) => {
            error!("Command failed: {}", e);
            std::process::exit(1);
        }
    }
}

fn run_command() -> Result<String, String> {
    debug!("Executing command logic");
    Ok("done".to_string())
}
```

## Async Web Service (Axum + tracing)

```toml
[dependencies]
axum = "0.7"
tokio = { version = "1", features = ["full"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
tower-http = { version = "0.6", features = ["trace"] }
uuid = { version = "1", features = ["v4"] }
```

```rust
use axum::{Router, routing::get, extract::Request, middleware::{self, Next}, response::Response};
use tower_http::trace::TraceLayer;
use tracing::{info, info_span, Instrument};
use uuid::Uuid;

// Request tracing middleware
async fn request_tracer(req: Request, next: Next) -> Response {
    let request_id = Uuid::new_v4();
    let span = info_span!(
        "request",
        method = %req.method(),
        path = %req.uri().path(),
        request_id = %request_id,
    );

    next.run(req).instrument(span).await
}

async fn handler() -> &'static str {
    info!("Processing request");
    simulate_db_call().await;
    "Hello, World!"
}

#[tracing::instrument]
async fn simulate_db_call() {
    info!("Querying database");
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    info!("Query complete");
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("info,tower_http=debug")
        .init();

    let app = Router::new()
        .route("/", get(handler))
        .layer(middleware::from_fn(request_tracer))
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    info!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
```

## Production Service with File Logging

```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
tracing-appender = "0.2"
tokio = { version = "1", features = ["full"] }
```

```rust
use tracing::{info, Level};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use tracing_appender::rolling::{RollingFileAppender, Rotation};

fn init_logging() {
    // File logging with daily rotation
    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("app")
        .filename_suffix("log")
        .build("/var/log/myapp")
        .expect("Failed to create log file");

    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        // Console: human-readable
        .with(fmt::layer().with_target(true))
        // File: JSON for machine processing
        .with(fmt::layer().json().with_writer(non_blocking))
        .init();

    // IMPORTANT: _guard must be kept alive
    Box::leak(Box::new(_guard));
}

#[tokio::main]
async fn main() {
    init_logging();
    info!("Service started");
    // ... application code ...
}
```

## Library with log Facade

```toml
# Library Cargo.toml - only log, no implementation
[dependencies]
log = "0.4"
```

```rust
// src/lib.rs
use log::{debug, info, warn, error, trace};

pub struct MyClient {
    url: String,
}

impl MyClient {
    pub fn new(url: &str) -> Self {
        debug!("Creating client for {}", url);
        Self { url: url.to_string() }
    }

    pub fn fetch(&self, path: &str) -> Result<String, String> {
        trace!(target: "my_lib::http", "Fetching {}{}", self.url, path);

        // Simulate request
        if path.contains("error") {
            warn!("Request to {} may fail", path);
            error!("Fetch failed: simulated error");
            return Err("Not found".to_string());
        }

        info!("Fetch successful: {}{}", self.url, path);
        Ok("response body".to_string())
    }
}
```

Users of this library choose their own logger:
```rust
// In application using the library
fn main() {
    env_logger::init();  // Or any other log implementation
    let client = my_lib::MyClient::new("https://api.example.com");
    client.fetch("/data").unwrap();
}
```

## Bridging log and tracing

When your app uses `tracing` but dependencies use `log`:

```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
tracing-log = "0.2"
log = "0.4"  # For demo
```

```rust
use tracing::{info, instrument};
use tracing_log::LogTracer;

// Simulates a library using log crate
fn legacy_library_call() {
    log::info!("Message from log crate");
    log::debug!("Debug from legacy code");
}

#[instrument]
fn modern_code() {
    info!("Message from tracing crate");
    legacy_library_call();  // These logs appear in tracing output
}

fn main() {
    // Install LogTracer to forward log records to tracing
    LogTracer::init().expect("Failed to set logger");

    tracing_subscriber::fmt()
        .with_env_filter("debug")
        .init();

    modern_code();
}
```

## Structured Logging with slog

```toml
[dependencies]
slog = "2"
slog-term = "2"
slog-async = "2"
slog-json = "2"
```

```rust
use slog::{info, warn, o, Drain, Logger};
use std::sync::Mutex;

fn main() {
    // Console output
    let decorator = slog_term::TermDecorator::new().build();
    let console_drain = slog_term::FullFormat::new(decorator).build().fuse();

    // JSON file output
    let file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("app.json.log")
        .unwrap();
    let json_drain = slog_json::Json::new(file).build().fuse();

    // Combine both outputs
    let combined = slog::Duplicate::new(console_drain, json_drain).fuse();
    let drain = slog_async::Async::new(combined).build().fuse();

    let log = Logger::root(drain, o!(
        "app" => "my-service",
        "version" => env!("CARGO_PKG_VERSION")
    ));

    info!(log, "Service started"; "port" => 8080);

    // Child logger with context
    let request_log = log.new(o!("request_id" => "abc-123"));
    info!(request_log, "Processing request"; "user_id" => 42);
    warn!(request_log, "Slow query"; "duration_ms" => 1500);
}
```

## Custom Formatting with fern

```toml
[dependencies]
log = "0.4"
fern = "0.7"
chrono = "0.4"
colored = "2"
```

```rust
use log::{info, warn, error, debug, Level};
use colored::*;

fn setup_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            let level_color = match record.level() {
                Level::Error => "red",
                Level::Warn => "yellow",
                Level::Info => "green",
                Level::Debug => "blue",
                Level::Trace => "white",
            };

            out.finish(format_args!(
                "{} {} {} > {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level().to_string().color(level_color),
                record.target().dimmed(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .chain(fern::log_file("output.log")?)
        .apply()?;
    Ok(())
}

fn main() {
    setup_logger().unwrap();

    info!("Application started");
    debug!("Debug information");
    warn!("Warning message");
    error!("Error occurred");
}
```
