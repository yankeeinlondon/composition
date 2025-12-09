# Rust Logging and Tracing

Here’s how the Rust logging/tracing ecosystem hangs together today and what’s worth using.

⸻

0. Mental model: logging vs tracing in Rust
 • log = facade for classic log records (error!, warn!, info!, debug!, trace!). Libraries depend on log, binaries pick an implementation.  ￼
 • tracing = structured, span-based, async-aware diagnostics (logs + traces + a lot of context).  ￼
 • Implementation crates:
 • for log: env_logger, fern, flexi_logger, log4rs, etc.  ￼
 • for tracing: tracing-subscriber (+ optional tracing-appender, tracing-log).  ￼
 • Distributed tracing / observability: tracing-opentelemetry + opentelemetry (Jaeger, OTLP, etc.).  ￼

Broadly: new async services → tracing stack. Simple CLI or “classic logging” → log + env_logger/flexi_logger/fern/log4rs. Slog is still around but tracing has become the de-facto “modern” choice.

⸻

## 1. Core facade: log

What it is

log is a lightweight logging facade; it defines macros and a global logger, but does not decide where logs go. Libraries just use log; binaries call log::set_logger via an implementation crate.  ￼

Log levels: Error, Warn, Info, Debug, Trace.

Minimal example

# Cargo.toml

```toml
[dependencies]
log = "0.4"
env_logger = "0.11" # for example
```

```rust
use log::{debug, error, info, trace, warn};

fn main() {
    // Implementation crate installs itself as the global logger.
    env_logger::init();

    error!("something bad happened: code={}", 42);
    warn!("careful, this might fail");
    info!("starting up");
    debug!("state = {:?}", (1, 2, 3));
    trace!("super noisy details");
}
```

Key gotcha from the docs: avoid expressions with side effects in log macros, because they might be compiled but not evaluated depending on filtering.  ￼

⸻

## 2. log ecosystem implementations

2.1 env_logger: simple, env-driven logger

What it is
 • Implementation of log configured (mostly) via RUST_LOG env var.
 • Writes to stderr by default; can output to stdout.  ￼

Quick usage

```toml
[dependencies]
log = "0.4"
env_logger = "0.11"
````

```rust
fn main() {
    // RUST_LOG=debug ./my-app
    env_logger::init();

    log::info!("app started");
    log::debug!(target: "db", "connected, pool_size={}", 10);
}
```

RUST_LOG example:

RUST_LOG=my_crate=debug,hyper=info ./my-app

Pros
 • Dead simple, no config files.
 • Plays nicely with any log-using dependency.
 • Good choice for CLIs and small services.

Cons
 • Limited formatting / routing options (no built-in file rotation, etc.).
 • Env-var driven (great for 12-factor, less so for complex setups).

⸻

2.2 fern: builder-style log implementation

What it is
 • “Simple, efficient logging” crate.
 • Provides a builder (fern::Dispatch) for composing outputs (stdout, stderr, files, syslog) with different filters and formats.  ￼

Example: console + file, different levels

```toml
[dependencies]
log = "0.4"
fern = "0.7"
chrono = "0.4"
colored = "2"
```

```rust
use chrono::Local;
use colored::Colorize;

fn setup_logging() -> Result<(), Box<dyn std::error::Error>> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{time} [{level}] {target}: {msg}",
                time = Local::now().format("%Y-%m-%dT%H:%M:%S"),
                level = record.level(),
                target = record.target(),
                msg = message,
            ))
        })
        .level(log::LevelFilter::Info)
        .level_for("my_crate::subsystem::noisy", log::LevelFilter::Debug)
        .chain(std::io::stderr())
        .chain(fern::log_file("app.log")?)
        .apply()?; // installs global logger

    Ok(())
}

fn main() {
    setup_logging().unwrap();
    log::info!("hello from fern");
}
```

Pros
 • Nice builder API; multi-target output.
 • Good for apps needing file + console with some complexity.
 • Mature and widely used.  ￼

Cons
 • Still “classic logs”, not spans/structured events.
 • Runtime reconfiguration is limited vs flexi_logger/log4rs.

⸻

2.3 flexi_logger: flexible, rotating, reconfigurable

What it is
 • Easy-to-configure logger for log.
 • Supports stderr/stdout and/or files, rotation, dynamic log spec changes, additional streams (e.g. security, alerts).  ￼

Example: env-driven filtering + file rotation

```toml
[dependencies]
log = "0.4"
flexi_logger = "0.29"
```

```rust
use flexi_logger::{Age, Cleanup, Criterion, Duplicate, Logger, Naming};

fn main() {
    Logger::try_with_env_or_str("info, my_crate::db=debug")
        .unwrap()
        .log_to_file()
        .duplicate_to_stderr(Duplicate::Info)
        .rotate(
            Criterion::Size(10_000_000),
            Naming::Numbers,
            Cleanup::KeepLogFiles(7),
        )
        .start()
        .unwrap();

    log::info!("flexi_logger up");
}
```

Pros
 • Very featureful if you want log files + rotation + dynamic level control.
 • Uses an env-style filter syntax similar to env_logger.  ￼

Cons
 • Overkill for small tools.
 • Slightly more “frameworky” than env_logger/fern.

⸻

2.4 log4rs: log4j-style, config-file-driven

What it is
 • Highly configurable logging framework modeled after Logback/log4j.
 • Config via YAML/TOML or programmatic APIs: appenders, encoders, filters, loggers.  ￼

YAML config example

[dependencies]
log = "0.4"
log4rs = "1"

log4rs.yaml:

refresh_rate: 30 seconds

appenders:
  stdout:
    kind: console
    encoder:
      pattern: "{d} {l} {t} - {m}{n}"

  rolling_file:
    kind: rolling_file
    path: "logs/app.log"
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
    - rolling_file

fn main() {
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();
    log::info!("hello from log4rs");
}

Pros
 • Very powerful/enterprise-y; great when ops wants to tweak logging via config.
 • Familiar model if you’re from Java land.

Cons
 • More moving pieces; heavier than env_logger/fern/flexi_logger.
 • For many greenfield Rust apps, tracing is preferred now.

⸻

## 3. Structured, async-aware diagnostics: tracing ecosystem

3.1 tracing core

What it is

Framework for instrumenting Rust code with spans and events, optimized for async code and structured data.  ￼

Think:
 • event! ~ log record (with fields).
 • span! ~ scoped context that can be entered/exited, and outlive stack frames (great for futures).  ￼

3.2 tracing-subscriber

Implements subscribers that collect trace data, format it, filter it, and ship it to sinks (stdout, journald, OpenTelemetry, etc.). EnvFilter, fmt layer, and composition via layers.  ￼

Basic setup example

```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = "0.3"
```

```rust
use tracing::{debug, error, info, span, Level};
use tracing_subscriber::{fmt, EnvFilter};

fn main() {
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(fmt::layer().pretty())
        .init();

    info!(message = "service starting", version = env!("CARGO_PKG_VERSION"));

    let span = span!(Level::INFO, "request", method = "GET", path = "/health");
    let _enter = span.enter();

    debug!(db_query = "SELECT 1", "pinging database");
    // ...
    error!(error = %"oops", "failed to ping db");
}
```

With:

```rust
RUST_LOG="info,my_app=debug" ./my-app

Per-request spans in async code

use tracing::{info_span, Instrument};

async fn handle_request(req_id: u64) {
    let span = info_span!("request", %req_id);
    async move {
        tracing::info!("processing request");
        // do stuff ...
    }
    .instrument(span)
    .await;
}
```

Tokio’s docs show this pattern for diagnosing async apps.  ￼

3.3 Integrating log with tracing (tracing-log)

tracing-log lets tracing subscribers consume log records as tracing events, so dependencies that use log show up in your span tree.  ￼

```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = "0.3"
tracing-log = "0.2"
log = "0.4"
```

```rust
use tracing_log::LogTracer;
use tracing_subscriber::{fmt, EnvFilter};

fn main() {
    // Forward `log` records into `tracing`.
    LogTracer::init().unwrap();

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(fmt::layer())
        .init();

    // Now both `log::info!` and `tracing::info!` go through tracing.
    log::info!("from log");
    tracing::info!("from tracing");
}
```

Pros of tracing stack
 • Native support for async and structured fields.
 • Spans → easy per-request timing and correlation.
 • Integrates with OpenTelemetry, metrics, etc.
 • tracing-log lets you keep your log-dependent ecosystem.

Cons
 • More concepts (spans, events, subscribers/layers).
 • Slightly more boilerplate than env_logger for tiny tools.
 • Only one subscriber per thread (by design) – multiple sinks require composing layers, not separate subscribers.  ￼

⸻

## 4. slog: structured logging ecosystem

What it is

Older but still solid ecosystem for structured, contextual, extensible logging. Events are key-value based, with composable drains (sinks).  ￼

Today, many folks pick tracing instead, but slog still appears in long-lived projects.

Example

```toml
[dependencies]
slog = "2"
slog-term = "2"
slog-async = "2"
```

```rust
use slog::{o, Drain};

fn main() {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let log = slog::Logger::root(drain, o!("version" => env!("CARGO_PKG_VERSION")));

    slog::info!(log, "service starting"; "port" => 8080);
    slog::warn!(log, "something odd"; "user_id" => 123);
}
```

Pros
 • Very explicit, structured log records.
 • Async drains, lots of ecosystem crates.

Cons
 • More friction/boilerplate than tracing for async code.
 • Ecosystem momentum has largely shifted toward tracing.

⸻

## 5. OpenTelemetry + tracing-opentelemetry

If you want distributed tracing (Jaeger, Tempo, OTLP, etc.), you typically:
 • Instrument with tracing.
 • Use tracing-opentelemetry layer to convert spans to OTel.
 • Use opentelemetry crate to pick an exporter.  ￼

Minimal example (stdout exporter just to see data)

```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = "0.3"
tracing-opentelemetry = "0.28"
opentelemetry = { version = "0.24", features = ["trace"] }
opentelemetry-stdout = { version = "0.7", features = ["trace"] }
```

```rust
use opentelemetry::global;
use opentelemetry_stdout::SpanExporter;
use tracing::{info_span, Instrument};
use tracing_subscriber::{layer::SubscriberExt, Registry};
use tracing_opentelemetry::OpenTelemetryLayer;

fn init_tracing() -> anyhow::Result<()> {
    let exporter = SpanExporter::new(std::io::stdout());
    let provider = opentelemetry::sdk::trace::TracerProvider::builder()
        .with_simple_exporter(exporter)
        .build();
    let tracer = provider.tracer("my-service");
    global::set_tracer_provider(provider);

    let otel_layer = OpenTelemetryLayer::new(tracer);

    let subscriber = Registry::default().with(otel_layer);
    tracing::subscriber::set_global_default(subscriber)?;

    Ok(())
}
```

and ...

```rust
# [tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing()?;

    let span = info_span!("request", request_id = 42);
    async {
        tracing::info!("processing...");
    }
    .instrument(span)
    .await;

    global::shutdown_tracer_provider(); // flush
    Ok(())
}
```

From here, you swap stdout exporter for OTLP/Jaeger/etc.

⸻

## 6. Pros/cons comparison (high level)

Facade vs framework

Layer Examples Pros Cons
Facade log De facto standard API; used by most crates No output by itself
Classic impls env_logger, fern, flexi_logger, log4rs Simple to sophisticated classic logging APIs No spans; async-aware context is bolted on
Structured/tracing tracing + tracing-subscriber Structured fields, spans, async-aware, OTel integration More concepts / setup
Structured logs (alt) slog Mature structured logging alternative Ecosystem momentum shifting to tracing
Observability tracing-opentelemetry, opentelemetry Distributed tracing, metrics, logs into standard backends Extra stack to run (collector, exporters)

⸻

## 7. Suggested “stacks” and when to use them

Given all that, a few combos that tend to work well:

 1. Small CLI / tool
    • log + env_logger.
    • Maybe switch to fern if you want a log file too.
 2. Sync or simple web service
    • Start with tracing + tracing-subscriber::fmt + EnvFilter.
    • Add tracing-log if dependencies use log.
 3. Async microservice / API (Tokio, Axum, etc.)
    • tracing + tracing-subscriber with layers:
    • EnvFilter for filtering
    • fmt::layer() for stdout
    • optional tracing-error, tracing-appender (rolling files)
    • tracing-log to unify log and tracing.
 4. Prod service with distributed tracing
    • Same as (3) +
    • tracing-opentelemetry + opentelemetry exporter (OTLP/Jaeger/Tempo).
 5. Legacy project already on slog
    • Stick with slog unless you’re doing a broader observability refactor.
    • If starting fresh today, I’d only choose slog if you join an ecosystem that standardizes on it.
