# Using `log` crate in Rust

The `log` crate is a foundational piece of Rust's logging ecosystem. It provides a unified API (a "facade") that libraries can use, while allowing applications to choose a specific logging implementation that fits their needs.

Here is a comparison of key logging and tracing crates to help you understand the ecosystem.

| **Crate Name** | **Primary Role** | **Key Features** | **Best For** |
| :--- | :--- | :--- | :--- |
| **`log`** | Logging Facade | Standardized API, five log level macros, works with many backends. | Libraries and applications needing simple, universal logging. |
| **`env_logger`** | Logger Implementation | Configurable via `RUST_LOG` env var, writes to stderr, simple setup. | Development, CLI tools, and getting started quickly. |
| **`log4rs`** | Logger Implementation | File & console appenders, YAML config, log rotation, runtime config. | Production applications needing fine-grained, file-based logging. |
| **`tracing`** | Structured Diagnostics | **Spans** (timed contexts), structured fields, async-friendly, rich ecosystem. | Complex async applications and deep, contextual diagnostics. |
| **`tracing-subscriber`** | `tracing` Backend | Collects and processes span/event data from `tracing`. | Required to get output from `tracing` instrumentation. |

### 📝 Using the `log` Crate

At its core, `log` provides five macros corresponding to verbosity levels: `error!`, `warn!`, `info!`, `debug!`, and `trace!`. Libraries should instrument their code with these macros.

```rust
use log::{info, warn, error};

pub fn process_data(input: &str) -> Result<(), &'static str> {
    info!("Starting to process data: {}", input);
    
    if input.is_empty() {
        warn!("Received empty input");
        return Err("Input was empty");
    }
    
    // ... processing logic ...
    
    info!("Data processing completed successfully");
    Ok(())
}

fn main() {
    // A logger implementation MUST be initialized here for output to appear.
    // Without it, all log messages are ignored.
    let _ = process_data("test");
}
```

### 🔌 Initializing a Logger

The `log` crate itself does not produce any output. You must initialize a **logger implementation** at the start of your executable. `env_logger` is a popular, simple choice.

First, add dependencies to your `Cargo.toml`:

```toml
[dependencies]
log = "0.4"
env_logger = "0.11"
```

Then, initialize it early in `main` and control the level via the `RUST_LOG` environment variable:

```rust
use log::{debug, info, warn};

fn main() {
    // Initialize the logger
    env_logger::init();
    
    debug!("This is a debug message"); // Hidden by default
    info!("This is an info message");   // Hidden by default
    warn!("This is a warning");         // Shown by default
}
```

Run your program with `RUST_LOG=info cargo run` to see `info` and higher-level messages. You can be more specific, e.g., `RUST_LOG="my_crate=debug,warn"` to set debug level for `my_crate` and warning for others.

### 🏗️ Structured Logging with `kv` (Unstable)

For production systems, structured logging (key-value pairs) is more useful than plain text. `log`'s **unstable** `kv` feature enables this.

```rust
// In Cargo.toml: log = { version = "0.4", features = ["kv_unstable"] }

use log::info;

pub fn handle_request(user_id: u64, action: &str) {
    // Structured logging: adds key-value pairs to the log record
    info!(
        target: "request_handler",
        user_id,
        action,
        status = "started";
        "Handling user request"
    );
}
```

### 🛠️ Popular Configuration & Tracing Crates

Beyond basic loggers, these crates are commonly used for advanced scenarios:

* **For Configuration & Files (`log4rs`)**: Ideal for production. It supports configuration via YAML files, logging to files with rotation, and complex routing rules.

    ```yaml
    # log4rs.yaml example: log warnings to console, info to a file
    refresh_rate: 30 seconds
    appenders:
      stdout:
        kind: console
      log_file:
        kind: file
        path: "logs/app.log"
    root:
      level: warn
      appenders: [stdout]
    loggers:
      my_app:
        level: info
        appenders: [log_file]
        additive: false
    ```

* **For Async & Complex Diagnostics (`tracing`)**: If you're building an asynchronous application (e.g., with Tokio), `tracing` is a more powerful alternative. Its core concepts are **events** (like a log) and **spans** (which capture a period of time, providing context). It works with `tracing-subscriber` as a backend.

    ```rust
    use tracing::{info_span, instrument, Level};
    use tracing_subscriber;

    #[instrument] // Automatically creates a span for this function
    pub fn process_order(order_id: u64) {
        // This event is linked to the "process_order" span
        tracing::event!(Level::INFO, "Order processing started");
        // ... function body ...
    }

    fn main() {
        // Initialize a basic tracing subscriber
        tracing_subscriber::fmt::init();
        process_order(12345);
    }
    ```

### ✅ Best Practices for Production

1. **Use Structured Logging**: Prefer key-value pairs over unstructured text for easier filtering and analysis.
2. **Add Context**: Always include relevant context like `request_id`, `user_id`, and `timestamp`.
3. **Avoid Side Effects**: Don't put code with side effects (like network calls) inside log macro arguments, as they might not execute if the log level is disabled.
4. **Set Appropriate Levels**:
    * `ERROR`: Operation failed (requires immediate attention).
    * `WARN`: Unexpected event, but system continues.
    * `INFO`: Normal system events (e.g., "user logged in").
    * `DEBUG`: Detailed information for debugging.
    * `TRACE`: Very verbose, low-level details.
5. **Never Log Sensitive Data**: Avoid passwords, tokens, or personal information in logs.

To summarize, for a new application, start with **`log` + `env_logger`** for simplicity. As your application grows, consider switching to **`log4rs`** for configurable file logging or **`tracing`** if you need advanced context and async support.

