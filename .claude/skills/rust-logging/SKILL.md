---
name: rust-logging
description: Expert knowledge for implementing logging and tracing in Rust. Use when building applications that need structured logging, async-aware diagnostics, console/file output, or distributed tracing with OpenTelemetry. Covers log, tracing, env_logger, fern, slog, and their ecosystems.
---

# Rust Logging and Tracing

## Quick Decision Guide

| Need | Recommended Crate(s) |
|------|----------------------|
| Simple CLI or library logging | `log` + `env_logger` |
| Async application (Tokio, Axum) | `tracing` + `tracing-subscriber` |
| Multi-output or custom formatting | `fern` or `flexi_logger` |
| Deep structured logging | `slog` or `tracing` |
| Distributed tracing (Jaeger, OTLP) | `tracing` + `tracing-opentelemetry` |
| File rotation in production | `tracing-appender` or `flexi_logger` |
| Config-file-driven logging | `log4rs` |

## Ecosystem Overview

| Layer | Examples | Purpose |
|:------|:---------|:--------|
| **Facade** | `log` | Standard API for log records - use in libraries |
| **Structured Diagnostics** | `tracing` | Spans + events for async-aware, context-rich diagnostics |
| **Simple Implementations** | `env_logger`, `pretty_env_logger` | Env-var controlled console logging |
| **Configurable Loggers** | `fern`, `flexi_logger`, `log4rs` | Multi-output, rotation, custom format |
| **Structured Ecosystem** | `slog` | Composable drains, contextual K-V logging |
| **Observability** | `tracing-opentelemetry` | Export to Jaeger, Tempo, Honeycomb |

## Key Concepts

**`log` vs `tracing`:**
- `log` = facade for discrete log records (error!, warn!, info!, debug!, trace!)
- `tracing` = spans (time periods) + events (moments), designed for async

**When to use each:**
- **Libraries**: Use `log` facade for maximum compatibility
- **CLI tools, simple sync apps**: `log` + `env_logger`
- **Async services (Tokio/Axum)**: `tracing` + `tracing-subscriber`
- **Production services**: Add `tracing-appender` for files, `EnvFilter` for runtime control

## Detailed Documentation

- [Deep Dive Guide](./deep-dive.md) - Comprehensive reference covering the entire logging/tracing ecosystem
- [log Crate Guide](./log-guide.md) - The logging facade and env_logger
- [tracing Framework](./tracing-guide.md) - Spans, events, subscribers, and async instrumentation
- [Advanced Loggers](./advanced-loggers.md) - fern, flexi_logger, log4rs, slog
- [OpenTelemetry Integration](./opentelemetry.md) - Distributed tracing setup
- [Code Examples](./examples.md) - Working examples for each library

## Quick Start Examples

### Simple Logging (log + env_logger)

```rust
use log::{info, warn, error};

fn main() {
    env_logger::init(); // Controlled by RUST_LOG env var

    info!("Application started");
    warn!("Low memory: {}MB free", 128);
    error!("Connection failed: {}", "timeout");
}
```

Run with: `RUST_LOG=info cargo run`

### Async Application (tracing)

```rust
use tracing::{info, instrument};
use tracing_subscriber::{fmt, EnvFilter};

#[instrument]
async fn handle_request(user_id: u64) {
    info!("Processing request");
    // spans propagate across await points
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    handle_request(123).await;
}
```

### Bridging log to tracing

```rust
use tracing_log::LogTracer;

// Forward log records to tracing subscriber
LogTracer::init()?;
```

## Cargo.toml Patterns

```toml
# Simple logging
log = "0.4"
env_logger = "0.11"

# Async tracing
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# File output with rotation
tracing-appender = "0.2"

# Bridge log to tracing
tracing-log = "0.2"

# OpenTelemetry
tracing-opentelemetry = "0.28"
opentelemetry = { version = "0.24", features = ["trace"] }
```

## Configuration via RUST_LOG

```bash
# Global level
RUST_LOG=info

# Per-crate filtering
RUST_LOG=warn,my_crate=debug,hyper=info

# Span filtering (tracing)
RUST_LOG=my_crate[handle_request]=trace
```

## Best Practices

1. **Libraries**: Depend only on `log` facade
2. **Avoid side effects** in log macro arguments (may not evaluate)
3. **Use structured fields**: `info!(user_id = %id, "Processing")`
4. **Never log secrets**: Passwords, tokens, PII
5. **Choose appropriate levels**:
   - ERROR: Failures requiring attention
   - WARN: Unexpected but recoverable
   - INFO: Normal milestones
   - DEBUG: Development diagnostics
   - TRACE: Verbose low-level details
