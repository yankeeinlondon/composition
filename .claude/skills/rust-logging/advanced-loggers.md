# Advanced Logging Solutions

Beyond `env_logger`, these crates offer more features for production logging.

## fern

Builder-style logger with multi-output support. Good balance of simplicity and power.

```toml
[dependencies]
log = "0.4"
fern = "0.7"
chrono = "0.4"  # For timestamps
```

```rust
use log::{info, warn, debug};
use std::time::SystemTime;

fn setup_logging() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                humantime::format_rfc3339_seconds(SystemTime::now()),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .level_for("my_crate::verbose_module", log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .chain(fern::log_file("app.log")?)
        .apply()?;
    Ok(())
}

fn main() {
    setup_logging().unwrap();
    info!("Application started");
    debug!("Debug from verbose module");
}
```

**Best for:** Apps needing custom formatting + file output without full `tracing` complexity.

## flexi_logger

Feature-rich logger with file rotation and runtime reconfiguration.

```toml
[dependencies]
log = "0.4"
flexi_logger = "0.29"
```

```rust
use flexi_logger::{Logger, Cleanup, Criterion, Duplicate, Naming, Age};

fn main() {
    Logger::try_with_env_or_str("info, my_crate::db=debug")
        .unwrap()
        .log_to_file()
        .duplicate_to_stderr(Duplicate::Info)  // Also print to stderr
        .rotate(
            Criterion::Size(10_000_000),  // Rotate at 10MB
            Naming::Numbers,
            Cleanup::KeepLogFiles(7),
        )
        .start()
        .unwrap();

    log::info!("flexi_logger configured");
}
```

**Features:**
- File rotation by size or age
- Runtime log level changes
- Multiple output streams
- Asynchronous writing
- `env_logger`-compatible filter syntax

**Best for:** Production apps needing file rotation without `tracing`.

## log4rs

Log4j-style configuration via YAML/TOML files.

```toml
[dependencies]
log = "0.4"
log4rs = "1"
```

**log4rs.yaml:**
```yaml
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

loggers:
  my_crate::noisy_module:
    level: warn
    additive: false
    appenders:
      - stdout
```

```rust
fn main() {
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();
    log::info!("Configured via YAML");
}
```

**Best for:** Enterprise environments where ops needs to tweak logging via config files.

## slog

Structured logging ecosystem with composable drains.

```toml
[dependencies]
slog = "2"
slog-term = "2"
slog-async = "2"
```

```rust
use slog::{info, o, Drain, Logger};

fn main() {
    // Terminal output with colors
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();

    let log = Logger::root(drain, o!(
        "version" => env!("CARGO_PKG_VERSION"),
        "app" => "my_app"
    ));

    info!(log, "Application started"; "port" => 8080);

    // Child logger with additional context
    let db_log = log.new(o!("component" => "database"));
    info!(db_log, "Connected"; "host" => "localhost");
}
```

**Key Concepts:**
- **Logger:** Handle for logging with inherited context
- **Drain:** Processes log records (terminal, file, network)
- **Composability:** Drains can be wrapped/chained

**Best for:** Deep structured logging where tracing's async focus isn't primary.

## Comparison

| Feature | fern | flexi_logger | log4rs | slog |
|---------|------|--------------|--------|------|
| File rotation | Via code | Built-in | Config-driven | Via slog-* crates |
| Config files | No | Optional | Primary | No |
| Runtime reconfig | No | Yes | Yes | Limited |
| Structured K-V | Via log's kv | Via log's kv | Via log's kv | Native |
| Async writing | No | Yes | Yes | Via slog-async |
| Learning curve | Low | Medium | Medium | Higher |

## Choosing an Advanced Logger

**fern**: Simple apps needing more than env_logger
**flexi_logger**: Production apps needing rotation without config files
**log4rs**: Ops-controlled logging via external config
**slog**: When structured logging is paramount and tracing's model doesn't fit
