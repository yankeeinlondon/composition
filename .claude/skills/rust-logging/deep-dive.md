# Rust Logging and Tracing: A Comprehensive Deep Dive

This document provides an authoritative, in-depth guide to logging and tracing in Rust, covering the complete ecosystem from simple logging facades to distributed tracing with OpenTelemetry.

## Table of Contents

1. [Foundational Concepts](#foundational-concepts)
2. [The log Crate Ecosystem](#the-log-crate-ecosystem)
3. [The tracing Framework](#the-tracing-framework)
4. [Advanced Logging Solutions](#advanced-logging-solutions)
5. [OpenTelemetry Integration](#opentelemetry-integration)
6. [Decision Framework](#decision-framework)
7. [Production Patterns](#production-patterns)
8. [Best Practices](#best-practices)

---

## Foundational Concepts

### Logging vs Tracing: Understanding the Distinction

In Rust's observability ecosystem, "logging" and "tracing" serve related but distinct purposes:

| Aspect | Traditional Logging | Structured Tracing |
|:-------|:-------------------|:-------------------|
| **Data Model** | Discrete, timestamped log records | Spans (time periods) + Events (moments) |
| **Context** | Minimal - each line is independent | Rich - events occur within hierarchical spans |
| **Async Support** | Poor - lines interleave unpredictably | Excellent - context propagates across await points |
| **Structure** | Primarily text-based | Typed key-value fields |
| **Use Case** | Simple apps, libraries, CLIs | Async services, microservices, distributed systems |

**Traditional Logging** records discrete events as they occur. Each log line stands alone, making it difficult to correlate related events in concurrent systems.

**Structured Tracing** captures both *what* happened and *when* it happened relative to other operations. Spans represent periods of time (like "handling this HTTP request"), while events are moments within those spans (like "cache miss occurred").

### The Facade Pattern

Both `log` and `tracing` follow the facade pattern:

- **Libraries** depend on the facade crate (`log` or `tracing`)
- **Applications** choose and configure an implementation
- This decoupling allows libraries to emit diagnostics without imposing specific backend choices

```
Library A ─────┐
Library B ─────┼──> Facade (log/tracing) ──> Implementation ──> Output
Library C ─────┘                              (env_logger,       (console,
                                               tracing-subscriber) file, etc.)
```

---

## The log Crate Ecosystem

### Core Concepts

The `log` crate provides Rust's de-facto standard logging facade. It defines five severity levels:

| Level | Purpose | When to Use |
|:------|:--------|:------------|
| `error!` | Critical failures | Operation failed, may require intervention |
| `warn!` | Potential problems | Unexpected but recoverable situations |
| `info!` | Important events | Normal application milestones |
| `debug!` | Development details | Detailed diagnostic information |
| `trace!` | Very verbose | Function entry/exit, loop iterations |

### Basic Usage

```toml
[dependencies]
log = "0.4"
env_logger = "0.11"
```

```rust
use log::{error, warn, info, debug, trace};

fn main() {
    // Initialize logger - must happen before any log calls
    env_logger::init();

    trace!("Entering main function");
    debug!("Configuration loaded: {:?}", config);
    info!("Application starting on port {}", port);
    warn!("Cache size approaching limit");
    error!("Failed to connect to database: {}", err);
}
```

### Target-Based Filtering

Every log record has a "target" (defaults to the module path). This enables fine-grained filtering:

```rust
// Custom target
info!(target: "security", "User {} authenticated", username);
info!(target: "database", "Query executed in {}ms", duration);

// Filter with: RUST_LOG=security=info,database=debug
```

### env_logger Configuration

The most common `log` implementation, configured via the `RUST_LOG` environment variable:

```bash
# Basic levels
RUST_LOG=info                        # Info and above globally
RUST_LOG=debug                       # Debug and above globally
RUST_LOG=my_crate=debug              # Debug for specific crate
RUST_LOG=my_crate::module=trace      # Trace for specific module
RUST_LOG=info,hyper=warn,h2=warn     # Multiple filters
```

```rust
// Default level if RUST_LOG not set
env_logger::Builder::from_env(
    env_logger::Env::default().default_filter_or("info")
).init();

// Custom format with timestamps
env_logger::Builder::from_default_env()
    .format(|buf, record| {
        use std::io::Write;
        writeln!(
            buf,
            "{} [{}] {}",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            record.level(),
            record.args()
        )
    })
    .init();
```

### Structured Logging (kv feature)

The `log` crate supports key-value data through an unstable feature:

```toml
[dependencies]
log = { version = "0.4", features = ["kv_unstable"] }
```

```rust
use log::info;

fn handle_request(user_id: u64, action: &str) {
    info!(
        user_id = user_id,
        action = action;
        "Handling user request"
    );
}
```

### Library Best Practice

Libraries should **only depend on `log`**, never on an implementation:

```toml
# Library Cargo.toml
[dependencies]
log = "0.4"
# NO env_logger or other implementation!
```

```rust
// src/lib.rs
use log::{debug, error, trace};

pub fn complex_operation(input: &str) -> Result<(), MyError> {
    trace!("Starting operation with: {}", input);

    match some_step(input) {
        Ok(val) => {
            debug!("Step succeeded: {:?}", val);
            Ok(())
        }
        Err(e) => {
            error!("Operation failed: {}", e);
            Err(e.into())
        }
    }
}
```

### Compile-Time Filtering

Remove log calls entirely from release builds:

```toml
[dependencies]
log = { version = "0.4", features = ["release_max_level_info"] }
```

Available features:
- `max_level_off`, `max_level_error`, ..., `max_level_trace`
- `release_max_level_*` - Only for release builds

---

## The tracing Framework

### Core Concepts

`tracing` is designed for structured, async-aware diagnostics with three fundamental concepts:

| Concept | Description | Analogy |
|:--------|:------------|:--------|
| **Span** | A period of time with beginning and end | A chapter in a book |
| **Event** | A moment within a span | A sentence in that chapter |
| **Subscriber** | Collects and processes span/event data | The printer/publisher |
| **Layer** | Composable processing component | A step in the printing process |

### Basic Setup

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

### The #[instrument] Attribute

Automatically creates spans for functions, capturing arguments:

```rust
use tracing::instrument;

#[instrument]
fn process_order(order_id: u64, user_id: u64) {
    tracing::info!("Processing order");
    // Creates span: "process_order{order_id=123, user_id=456}"
}

#[instrument(skip(password))]  // Don't log sensitive data
async fn authenticate(user: &str, password: &str) -> bool {
    tracing::debug!("Authenticating user");
    true
}

#[instrument(name = "custom_span_name", level = "debug")]
fn with_custom_options() {}

#[instrument(ret, err)]  // Log return value and errors
fn parse_config(path: &str) -> Result<Config, Error> {
    // ...
}

#[instrument(fields(request_id = %uuid::Uuid::new_v4()))]
async fn handle(req: Request) {
    // Add custom fields to the span
}
```

### Manual Spans

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

### Async Spans with .instrument()

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

**Critical Pattern**: Never hold span guards across `.await` points:

```rust
// WRONG - span guard held across await
async fn bad_example() {
    let span = info_span!("my_span");
    let _guard = span.enter();
    some_async_work().await;  // Guard still held - context may be wrong!
}

// CORRECT - use .instrument()
async fn good_example() {
    let span = info_span!("my_span");
    async { some_async_work().await }.instrument(span).await;
}
```

### Structured Fields

```rust
use tracing::{info, warn, Level, event};

// Basic fields
info!(user_id = 123, "User logged in");

// Debug formatting with ?
debug!(config = ?app_config, "Loaded configuration");

// Display formatting with %
warn!(error = %e, "Operation failed");

// Multiple fields
info!(
    user_id = user.id,
    username = %user.name,
    roles = ?user.roles,
    "User authenticated"
);

// Dynamic level
event!(Level::ERROR, error = ?e, "Request failed");
```

### Subscriber Configuration

#### Basic with EnvFilter

```rust
use tracing_subscriber::{fmt, EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

tracing_subscriber::registry()
    .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
    .with(fmt::layer())
    .init();
```

#### JSON Output (Production)

```rust
tracing_subscriber::fmt()
    .json()
    .with_target(false)
    .with_thread_ids(true)
    .with_current_span(true)
    .init();
```

#### Multiple Layers

```rust
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
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

### File Logging with Rotation

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

### Bridging log to tracing

When your app uses `tracing` but dependencies use `log`:

```rust
use tracing_log::LogTracer;

fn main() {
    // Forward log records to tracing
    LogTracer::init().expect("Failed to set logger");

    tracing_subscriber::fmt::init();

    // Now both work through tracing
    log::info!("From log crate");
    tracing::info!("From tracing crate");
}
```

### EnvFilter Directives

```bash
# Basic levels
RUST_LOG=trace                       # Everything
RUST_LOG=info                        # Info and above

# Per-crate
RUST_LOG=my_crate=debug              # Debug for my_crate only
RUST_LOG=info,my_crate=debug         # Info global, debug for my_crate

# Per-module
RUST_LOG=my_crate::database=trace

# Suppress noisy crates
RUST_LOG=info,hyper=warn,h2=warn,tower=warn

# Span-based filtering (tracing-specific)
RUST_LOG="[request]=debug"           # Debug only inside "request" spans
```

---

## Advanced Logging Solutions

### fern: Builder-Style Logger

Good balance of simplicity and power for the `log` ecosystem.

```toml
[dependencies]
log = "0.4"
fern = "0.7"
chrono = "0.4"
```

```rust
use fern::colors::{Color, ColoredLevelConfig};

fn setup_logging() -> Result<(), fern::InitError> {
    let colors = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        .info(Color::Green)
        .debug(Color::Blue)
        .trace(Color::Magenta);

    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{} [{}] [{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                colors.color(record.level()),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .level_for("hyper", log::LevelFilter::Warn)
        .level_for("my_crate::verbose", log::LevelFilter::Debug)
        .chain(std::io::stderr())
        .chain(fern::log_file("app.log")?)
        .apply()?;

    Ok(())
}
```

**Best for**: Apps needing custom formatting + file output without full `tracing` complexity.

### flexi_logger: Flexible with Rotation

Feature-rich with file rotation and runtime reconfiguration.

```toml
[dependencies]
log = "0.4"
flexi_logger = "0.29"
```

```rust
use flexi_logger::{Logger, Cleanup, Criterion, Duplicate, Naming};

fn main() {
    Logger::try_with_env_or_str("info, my_crate::db = debug")
        .unwrap()
        .log_to_file(flexi_logger::FileSpec::default()
            .directory("logs")
            .basename("app"))
        .duplicate_to_stderr(Duplicate::Info)
        .rotate(
            Criterion::Size(10_000_000),  // 10MB
            Naming::Numbers,
            Cleanup::KeepLogFiles(7),
        )
        .start()
        .unwrap();

    log::info!("Logging initialized");
}
```

**Best for**: Production apps needing rotation without `tracing`.

### log4rs: Config-File Driven

Enterprise-style configuration via YAML/TOML files.

```yaml
# log4rs.yaml
refresh_rate: 30 seconds

appenders:
  stdout:
    kind: console
    encoder:
      pattern: "{d(%Y-%m-%d %H:%M:%S)} [{l}] {t} - {m}{n}"

  file:
    kind: rolling_file
    path: "logs/app.log"
    encoder:
      pattern: "{d} [{l}] {t} - {m}{n}"
    policy:
      kind: compound
      trigger:
        kind: size
        limit: 10 mb
      roller:
        kind: fixed_window
        base: 1
        count: 5
        pattern: "logs/app.{}.log.gz"

root:
  level: info
  appenders:
    - stdout
    - file

loggers:
  my_crate::database:
    level: debug
```

```rust
fn main() {
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();
    log::info!("Application started");
}
```

**Best for**: Enterprise environments where ops needs to tweak logging via config files.

### slog: Structured Logging Ecosystem

A mature alternative with composable drains and explicit structured logging.

```toml
[dependencies]
slog = "2"
slog-term = "2"
slog-async = "2"
```

```rust
use slog::{o, info, warn, Drain, Logger};

fn main() {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();

    let log = Logger::root(drain, o!(
        "version" => env!("CARGO_PKG_VERSION"),
        "app" => "my_service"
    ));

    info!(log, "Application started"; "port" => 8080);

    // Child logger with additional context
    let db_log = log.new(o!("component" => "database"));
    info!(db_log, "Connected"; "host" => "localhost");
}
```

**Note**: Most new projects prefer `tracing` for structured logging. `slog` is primarily found in legacy codebases.

### Comparison Table

| Feature | fern | flexi_logger | log4rs | slog |
|:--------|:-----|:-------------|:-------|:-----|
| File rotation | Via code | Built-in | Config-driven | Via slog-* crates |
| Config files | No | Optional | Primary | No |
| Runtime reconfig | No | Yes | Yes | Limited |
| Structured K-V | Via log's kv | Via log's kv | Via log's kv | Native |
| Async writing | No | Yes | Yes | Via slog-async |
| Learning curve | Low | Medium | Medium | Higher |

---

## OpenTelemetry Integration

### Architecture Overview

```
┌─────────────────────────────────────────────────┐
│  Your Rust Application                          │
│  tracing spans & events                         │
├─────────────────────────────────────────────────┤
│  tracing-opentelemetry Layer                    │
│  Converts tracing spans -> OTel spans           │
├─────────────────────────────────────────────────┤
│  OpenTelemetry SDK                              │
│  Batching, sampling, processing                 │
├─────────────────────────────────────────────────┤
│  Exporter (OTLP, Jaeger, Stdout, etc.)          │
│  Sends to backend                               │
└─────────────────────────────────────────────────┘
```

### Basic Setup with OTLP

```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
tracing-opentelemetry = "0.28"
opentelemetry = { version = "0.24", features = ["trace"] }
opentelemetry_sdk = { version = "0.24", features = ["rt-tokio"] }
opentelemetry-otlp = "0.17"
```

```rust
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{runtime, trace::TracerProvider};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

fn init_tracing() -> Result<(), Box<dyn std::error::Error>> {
    // Create OTLP exporter
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint("http://localhost:4317")
        .build()?;

    // Create tracer provider
    let provider = TracerProvider::builder()
        .with_batch_exporter(exporter, runtime::Tokio)
        .build();

    let tracer = provider.tracer("my-service");
    opentelemetry::global::set_tracer_provider(provider);

    // Create OpenTelemetry layer
    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .with(otel_layer)
        .init();

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing()?;

    tracing::info!("Application starting");
    // Your application code...

    // Graceful shutdown - flush pending spans
    opentelemetry::global::shutdown_tracer_provider();
    Ok(())
}
```

### Context Propagation

For distributed tracing across services:

```rust
use opentelemetry::propagation::TextMapPropagator;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use tracing_opentelemetry::OpenTelemetrySpanExt;

// Set global propagator (in init)
opentelemetry::global::set_text_map_propagator(
    TraceContextPropagator::new()
);

// Extract context from incoming request
fn extract_context(headers: &HeaderMap) -> opentelemetry::Context {
    opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.extract(&HeaderMapCarrier(headers))
    })
}

// Inject context into outgoing request
async fn call_downstream(client: &Client) {
    let current_span = tracing::Span::current();
    let context = current_span.context();

    let mut headers = HeaderMap::new();
    opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.inject_context(&context, &mut HeaderMapCarrier(&mut headers));
    });

    client.get("http://downstream/api")
        .headers(headers)
        .send()
        .await;
}
```

### Sampling Strategies

```rust
use opentelemetry_sdk::trace::Sampler;

let provider = TracerProvider::builder()
    .with_batch_exporter(exporter, runtime::Tokio)
    // Sample 10% of traces
    .with_sampler(Sampler::TraceIdRatioBased(0.1))
    .build();
```

**Sampler options**:
- `AlwaysOn` - Export all traces (development)
- `AlwaysOff` - Export nothing (still propagates context)
- `TraceIdRatioBased(f64)` - Probabilistic sampling (production)
- `ParentBased` - Follow parent's decision (distributed systems)

### Resource Attributes

```rust
use opentelemetry::KeyValue;
use opentelemetry_sdk::Resource;

let resource = Resource::new(vec![
    KeyValue::new("service.name", "my-service"),
    KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
    KeyValue::new("deployment.environment", "production"),
]);

let provider = TracerProvider::builder()
    .with_batch_exporter(exporter, runtime::Tokio)
    .with_resource(resource)
    .build();
```

### Common Backends

| Backend | Protocol | Endpoint |
|:--------|:---------|:---------|
| Jaeger | OTLP gRPC | localhost:4317 |
| Tempo (Grafana) | OTLP gRPC | localhost:4317 |
| Honeycomb | OTLP HTTP | api.honeycomb.io:443 |
| Datadog | OTLP gRPC | localhost:4317 (agent) |
| Zipkin | Zipkin | localhost:9411 |

---

## Decision Framework

### Quick Decision Guide

| Scenario | Recommended Stack |
|:---------|:------------------|
| Simple CLI or script | `log` + `env_logger` |
| Library crate | `log` only (let consumers choose) |
| Sync app with file logging | `log` + `fern` or `flexi_logger` |
| Config-file driven logging | `log4rs` |
| Async service (Tokio, Axum) | `tracing` + `tracing-subscriber` |
| Production microservice | `tracing` + `tracing-appender` |
| Distributed tracing | `tracing` + `tracing-opentelemetry` |
| Legacy code using `log` | Add `tracing-log` for compatibility |

### When to Use log

- Building a **library** (maximum compatibility)
- Simple **CLI tool** or script
- **Synchronous** application
- Dependencies primarily use `log`
- Minimal setup is priority

### When to Use tracing

- Building **async applications** (Tokio, Axum, Actix)
- Need **span-based context** propagation
- Want **structured, machine-parseable** output
- Need **performance monitoring** with timing
- Planning for **distributed tracing**
- Complex multi-service architectures

### Migration Path

```rust
// Step 1: Add tracing-log to forward log -> tracing
use tracing_log::LogTracer;
LogTracer::init().unwrap();

// Step 2: Initialize tracing subscriber
tracing_subscriber::fmt::init();

// Step 3: Gradually replace log::info! with tracing::info!
// Both now work through the same subscriber
```

---

## Production Patterns

### Web Service with Request Tracing (Axum)

```rust
use axum::{Router, routing::get, extract::Request, middleware::{self, Next}, response::Response};
use tower_http::trace::TraceLayer;
use tracing::{info, info_span, Instrument};
use uuid::Uuid;

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

### Production Service with File Logging

```rust
use tracing::{info, Level};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use tracing_appender::rolling::{RollingFileAppender, Rotation};

fn init_logging() {
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
}
```

### Complete Production OpenTelemetry Setup

```rust
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    propagation::TraceContextPropagator,
    runtime,
    trace::{Sampler, TracerProvider},
    Resource,
};
use opentelemetry::KeyValue;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn init_observability(
    service_name: &str,
    otlp_endpoint: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Set up context propagation
    opentelemetry::global::set_text_map_propagator(
        TraceContextPropagator::new()
    );

    // Build resource with service info
    let resource = Resource::new(vec![
        KeyValue::new("service.name", service_name.to_string()),
        KeyValue::new("service.version", env!("CARGO_PKG_VERSION").to_string()),
    ]);

    // Create OTLP exporter
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(otlp_endpoint)
        .build()?;

    // Create provider with batching and sampling
    let provider = TracerProvider::builder()
        .with_batch_exporter(exporter, runtime::Tokio)
        .with_resource(resource)
        .with_sampler(Sampler::TraceIdRatioBased(0.1))  // 10% sampling
        .build();

    let tracer = provider.tracer(service_name);
    opentelemetry::global::set_tracer_provider(provider);

    // Create tracing layers
    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "info,hyper=warn,h2=warn".into());

    let fmt_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_current_span(true);

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .with(otel_layer)
        .init();

    Ok(())
}

pub fn shutdown_observability() {
    opentelemetry::global::shutdown_tracer_provider();
}
```

---

## Best Practices

### General Guidelines

1. **Libraries use `log` facade** - Don't force dependencies on consumers
2. **Applications use `tracing`** - Especially for async code
3. **Initialize logging early** - Before any log calls
4. **Use `#[instrument]`** - Automatic span creation for functions
5. **Add context with spans** - Not just isolated log lines
6. **Filter noisy crates** - `RUST_LOG=info,hyper=warn,h2=warn`
7. **JSON for production** - Machine-parseable output

### Security

- **Never log secrets**: Passwords, API keys, tokens, PII
- **Use `skip` in #[instrument]**: `#[instrument(skip(password, token))]`
- **Sanitize user input**: Before including in log messages
- **Consider log access**: Who can read production logs?

### Performance

1. **Use `enabled!`** before expensive debug formatting
2. **Lazy evaluation**: Fields only formatted if level enabled
3. **Compile-time filtering**: Use `max_level_*` features
4. **Non-blocking appenders**: For file output
5. **Sampling in production**: 10% is often sufficient

```toml
# Compile out trace/debug in release
[dependencies]
tracing = { version = "0.1", features = ["release_max_level_info"] }
```

### Log Levels Usage

| Level | When to Use | Example |
|:------|:------------|:--------|
| `error!` | Operation failed, may require intervention | Database connection lost |
| `warn!` | Unexpected but recoverable | Retry attempt, deprecation warning |
| `info!` | Important milestones | Service started, request completed |
| `debug!` | Development diagnostics | Variable values, flow decisions |
| `trace!` | Very verbose details | Function entry/exit, loop iterations |

### Structured Data

Prefer structured fields over string interpolation:

```rust
// Good - structured, queryable
info!(user_id = 123, action = "login", duration_ms = 45, "User action");

// Less good - harder to parse
info!("User 123 performed login in 45ms");
```

---

## References

### Official Documentation

- [log crate](https://docs.rs/log)
- [tracing crate](https://docs.rs/tracing)
- [tracing-subscriber](https://docs.rs/tracing-subscriber)
- [tracing-appender](https://docs.rs/tracing-appender)
- [tracing-opentelemetry](https://docs.rs/tracing-opentelemetry)

### Implementations

- [env_logger](https://docs.rs/env_logger)
- [fern](https://docs.rs/fern)
- [flexi_logger](https://docs.rs/flexi_logger)
- [log4rs](https://docs.rs/log4rs)
- [slog](https://docs.rs/slog)

### OpenTelemetry

- [OpenTelemetry Rust](https://github.com/open-telemetry/opentelemetry-rust)
- [OpenTelemetry Specification](https://opentelemetry.io/docs/specs/otel/)
- [Jaeger](https://www.jaegertracing.io/)
- [OTel Collector](https://opentelemetry.io/docs/collector/)

### Tutorials

- [Tokio Tracing Guide](https://tokio.rs/tokio/topics/tracing)
- [Shuttle Tracing Tutorial](https://www.shuttle.dev/blog/2024/01/09/getting-started-tracing-rust)
