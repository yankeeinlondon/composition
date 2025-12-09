# The log Crate and Ecosystem

The `log` crate is Rust's de-facto standard logging facade. Libraries depend on it to emit diagnostics without dictating the logging backend.

## Core Concepts

**Facade Pattern:** `log` provides macros; applications choose an implementation.

**Log Levels (highest to lowest priority):**
- `error!` - Operation failed, may require intervention
- `warn!` - Unexpected but recoverable
- `info!` - Normal application flow
- `debug!` - Development details
- `trace!` - Very verbose low-level info

**Targets:** Each log has a target (default: module path). Use for filtering:
```rust
info!(target: "database", "Connected to {}", host);
// Filter with RUST_LOG=database=debug
```

## Basic Setup with env_logger

```toml
[dependencies]
log = "0.4"
env_logger = "0.11"
```

```rust
use log::{info, warn, error, debug, trace};

fn main() {
    // Initialize early - logs before this are ignored
    env_logger::init();

    trace!("Very detailed info");
    debug!("Development details");
    info!("Normal operation");
    warn!("Something unusual");
    error!("Something failed");
}
```

**Controlling output:**
```bash
RUST_LOG=debug cargo run           # All debug and above
RUST_LOG=my_crate=trace cargo run  # trace for my_crate only
RUST_LOG=warn,my_crate=debug       # warn globally, debug for my_crate
```

## Custom Initialization

```rust
use log::LevelFilter;

fn main() {
    // Default to "info" if RUST_LOG not set
    env_logger::Builder::from_env(
        env_logger::Env::default()
            .default_filter_or("info")
    ).init();

    log::info!("Application started");
}
```

### With Timestamps and Format

```rust
env_logger::Builder::from_default_env()
    .filter_level(log::LevelFilter::Info)
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

## Structured Logging (kv feature - unstable)

```toml
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

## For Library Authors

Libraries should **only depend on `log`**, not on any implementation:

```toml
[dependencies]
log = "0.4"
```

```rust
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

This lets consumers choose their logger without your library imposing one.

## Compile-Time Filtering

Completely remove log calls from release builds:

```toml
[dependencies]
log = { version = "0.4", features = ["release_max_level_info"] }
```

Available features:
- `max_level_off`, `max_level_error`, ..., `max_level_trace`
- `release_max_level_*` - Only for release builds

## Alternative Implementations

| Crate | Best For |
|-------|----------|
| `env_logger` | Simple, env-var controlled |
| `pretty_env_logger` | Colored output |
| `simplelog` | Multiple outputs |
| `fern` | Custom formatting, multiple outputs |
| `flexi_logger` | File rotation, runtime reconfiguration |
| `log4rs` | Config-file driven (Log4j style) |

## When to Use log vs tracing

**Choose log when:**
- Building a library (maximum compatibility)
- Simple CLI tool or script
- Synchronous application
- Dependencies use `log`

**Choose tracing when:**
- Building async applications (Tokio, Axum)
- Need span-based context propagation
- Want structured, machine-parseable output
- Need distributed tracing
