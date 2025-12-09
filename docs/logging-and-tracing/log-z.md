# Using `log` crate in Rust

### 1. Introduction: Why Logging and Why `log`?

In any non-trivial application, understanding what the program is doing is crucial. `println!()` is great for quick debugging, but it's unstructured, unfiltered, and can't be turned off in production. This is where logging comes in. A good logging framework provides:

* **Severity Levels:** Categorize messages (e.g., `error`, `warn`, `info`, `debug`).
* **Filtering:** Choose which messages to see based on their severity or the module they came from.
* **Destination:** Send logs to the console, a file, or a centralized logging service.
* **Structure:** Add context (like timestamps or request IDs) to your logs.

The `log` crate is the de-facto standard for this in Rust. However, it has a very specific and important design philosophy: **it is a logging facade, not a logger itself.**

Think of it like a standard electrical outlet. Your application (the appliance) plugs into the `log` crate's outlet. Then, you can plug any logger implementation (the power source) into that outlet. This decouples your application's logging *calls* from the actual logging *implementation*.

**Key Benefit:** As a library author, you can depend only on the lightweight `log` crate. As an application author, you can choose the logger that best fits your needs (simple console logging, complex file rotation, etc.) without your dependencies caring.

---

### 2. The Core: The `log` Crate Facade

The `log` crate provides a set of macros that you use throughout your code.

#### Log Levels (in order of severity)

1. `error!`: For errors that the program may or may not be able to recover from.
2. `warn!`: For potentially harmful situations or for when using deprecated APIs.
3. `info!`: For informational messages that highlight the progress of the application at a coarse-grained level.
4. `debug!`: For detailed diagnostic information that can be useful for developers.
5. `trace!`: For very detailed, "trace" level information, often logging every step of a function.

#### Basic Usage Example

Let's create a new project and add the `log` crate.

**`Cargo.toml`**

```toml
[dependencies]
log = "0.4"
```

**`src/main.rs`**

```rust
use log::{debug, error, info, trace, warn};

fn main() {
    // This call is necessary to initialize the logger.
    // We'll see what this does in the next section.
    // For now, just know it's required.
    // log::set_logger(&...).map(|_| log::set_max_level(...)).unwrap();
    
    println!("About to start logging...");
    
    trace!("This is a trace message, the most verbose level.");
    debug!("This is a debug message.");
    info!("This is an info message.");
    warn!("This is a warning message.");
    error!("This is an error message.");
    
    println!("Finished logging.");
}
```

If you run this code right now (`cargo run`), you will see:

```
About to start logging...
Finished logging.
```

**Notice anything?** Your log messages are nowhere to be found! This is because we've only used the facade. We haven't provided an *implementation* to actually capture and display the logs.

---

### 3. Choosing an Implementation: `env_logger`

The most common and simplest logger implementation is `env_logger`. It configures the logger based on environment variables.

**`Cargo.toml`**

```toml
[dependencies]
log = "0.4"
env_logger = "0.10" # Use a recent version
```

Now, let's modify our `main.rs` to initialize `env_logger`.

**`src/main.rs`**

```rust
use log::{debug, error, info, trace, warn};

fn main() {
    // Initialize the logger. This should be done early in main().
    // It reads the RUST_LOG environment variable.
    env_logger::init();

    println!("About to start logging...");

    trace!("This is a trace message, the most verbose level.");
    debug!("This is a debug message.");
    info!("This is an info message.");
    warn!("This is a warning message.");
    error!("This is an error message.");

    println!("Finished logging.");
}
```

Now, run your program from the terminal. By default, `env_logger` only shows `error`, `warn`, and `info` level logs.

```bash
$ cargo run
   Compiling my_log_app v0.1.0 (...)
    Finished dev [unoptimized + debuginfo] target(s) in ...
     Running `target/debug/my_log_app`
About to start logging...
INFO  my_log_app] This is an info message.
WARN  my_log_app] This is a warning message.
ERROR my_log_app] This is an error message.
Finished logging.
```

The power of `env_logger` comes from the `RUST_LOG` environment variable.

* **Set the log level to `debug`:**

    ```bash
    $ RUST_LOG=debug cargo run
    INFO  my_log_app] This is an info message.
    DEBUG my_log_app] This is a debug message.
    WARN  my_log_app] This is a warning message.
    ERROR my_log_app] This is an error message.
    ```

* **Set the log level to `trace` to see everything:**

    ```bash
    $ RUST_LOG=trace cargo run
    TRACE my_log_app] This is a trace message, the most verbose level.
    DEBUG my_log_app] This is a debug message.
    INFO  my_log_app] This is an info message.
    WARN  my_log_app] This is a warning message.
    ERROR my_log_app] This is an error message.
    ```

* **Filter by crate/module:** This is incredibly useful in large projects.

    ```bash
    # Only show debug logs for the `my_crate::networking` module
    # and info logs for everything else.
    $ RUST_LOG="my_crate::networking=debug,info" cargo run
    ```

---

### 4. Advanced `log` Features

#### Structured Logging

While `log` is primarily a text-based facade, it does support structured data in a key-value format. This is a stepping stone to more powerful structured loggers like `tracing`.

```rust
use log::info;

fn process_user(user_id: u32, email: &str) {
    info!(
        user_id = user_id,       // Key-value pair
        email = email,           // Key-value pair
        "Processing new user"    // The main message
    );
}
```

When used with a compatible logger, this can be rendered as JSON or another structured format, making it easy to parse and search.

#### Targets

Every log macro has an optional `target` argument. By default, the target is the current module path (e.g., `my_crate::my_module`). Libraries often use this to namespace their logs, allowing application developers to filter them easily.

```rust
// In a library called `cool_lib`
pub fn do_something() {
    log::info!(target: "cool_lib", "Did something cool!");
}
```

You could then filter this specifically: `RUST_LOG="cool_lib=debug"`.

---

### 5. The Ecosystem: Beyond `env_logger`

The `log` ecosystem is rich. Here are some popular choices for implementations.

* **`env_logger`**: The standard, simple choice. Configurable via environment variables.
* **`fern`**: A highly configurable logger that allows for fine-grained control over formatting, output destinations (console, multiple files), and filtering via code. It's a great choice when `env_logger` isn't flexible enough.

    **Example `fern` setup:**

    ```rust
    use log::LevelFilter;
    use fern::Dispatch;

    fn setup_logger() -> Result<(), Box<dyn std::error::Error>> {
        fern::Dispatch::new()
            .format(|out, message, record| {
                out.finish(format_args!(
                    "{}[{}][{}] {}",
                    chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                    record.level(),
                    record.target(),
                    message
                ))
            })
            .level(LevelFilter::Debug)
            .chain(std::io::stdout())
            .chain(fern::log_file("output.log")?)
            .apply()?;
        Ok(())
    }
    ```

* **`flexi_logger`**: Another powerful logger known for its features like writing to files with rotation, re-configuring the log spec at runtime, and more.

---

### 6. The Modern Successor: The `tracing` Crate

While `log` is the established standard, the **`tracing`** crate is its modern, more powerful successor. It's designed for understanding the *flow* of a system, not just recording discrete events.

**Key Concepts of `tracing`:**

* **Spans:** Represent a period of time during which your program is executing. A span has a beginning and an end. Think of a web request handling process as a span.
* **Events:** Are things that happen *within* a span. They are analogous to the log messages from the `log` crate.
* **Context:** Spans can be nested, creating a hierarchical context. A database query span might be inside a web request span.

The `tracing` crate also follows the facade/implementation pattern. You use `tracing` for your instrumentation and a subscriber like `tracing-subscriber` to actually process the data.

**`tracing` Example:**

**`Cargo.toml`**

```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

**`src/main.rs`**

```rust
use tracing::{info, instrument};
use tracing_subscriber;

// The #[instrument] attribute automatically creates a span for the function.
#[instrument]
fn authenticate_user(user_id: u64) {
    // This event is recorded within the 'authenticate_user' span.
    info!("Starting authentication for user");
    // ... logic ...
    info!("Authentication successful");
}

#[instrument] // The main function can also be a span!
fn main() {
    // Initialize the tracing subscriber.
    tracing_subscriber::fmt::init();

    info!("Application starting up");
    authenticate_user(123);
    info!("Application shutting down");
}
```

**Running with `RUST_LOG=info`:**

```bash
$ RUST_LOG=info cargo run
INFO  my_app: Application starting up
INFO  my_app:authenticate_user{user_id=123}: authenticate_user{user_id=123}: Starting authentication for user
INFO  my_app:authenticate_user{user_id=123}: authenticate_user{user_id=123}: Authentication successful
INFO  my_app: Application shutting down
```

Notice how the output shows the context: the `info` messages from `authenticate_user` are clearly nested within the function's span, and the `user_id` is automatically included as context.

**Recommendation:** For new applications, especially complex ones like web services, **start with `tracing`**. For libraries, `log` is still perfectly fine, and `tracing` can forward its events to the `log` crate for compatibility.

---

### 7. Configuration Crates

Hardcoding log settings is inflexible. These crates help you manage configuration.

* **`config`**: The go-to crate for general application configuration. It can read settings from TOML, JSON, YAML, or environment variables. You can define a `log_level` in your `config.toml` and use it to programmatically set the level for `env_logger` or `fern`.
* **`clap`**: The standard for command-line argument parsing. You can add a `--log-level` or `--verbose` flag to your application to control logging at runtime.
* **`dotenv`**: For loading environment variables from a `.env` file during development, which is perfect for setting `RUST_LOG` without typing it every time.

**Example using `clap` and `env_logger`:**

```rust
// Cargo.toml dependencies: log, env_logger, clap (with derive feature)
use clap::Parser as _;
use log::LevelFilter;

#[derive(clap::Parser)]
struct Cli {
    /// The log level to use (trace, debug, info, warn, error)
    #[arg(short, long, default_value = "info")]
    log_level: LevelFilter,
}

fn main() {
    let cli = Cli::parse();

    // Initialize env_logger with the level from the CLI argument
    env_logger::Builder::from_default_env()
        .filter_level(cli.log_level)
        .init();

    log::info!("This will be shown if log_level is info or higher.");
    log::debug!("This will only be shown if --log-level debug is passed.");
}
```

### Summary and Best Practices

1. **For Libraries:** Use the `log` crate. It's lightweight, standard, and lets the end-user choose their logger.
2. **For New Applications:** Strongly consider using the `tracing` and `tracing-subscriber` ecosystem. It provides superior context and is the direction the community is moving.
3. **Choose the Right Logger:**
    * `env_logger`: For simple apps and quick setups.
    * `fern` or `flexi_logger`: For complex needs like custom formatting, file rotation, or programmatic configuration.
4. **Use Appropriate Log Levels:** Don't log everything at `info!`. Use `debug!` and `trace!` for detailed information that you don't need to see in production by default.
5. **Log Structured Data:** When possible, use key-value syntax (`user_id = 123`) to make your logs machine-readable.
6. **Externalize Configuration:** Use crates like `clap` or `config` to avoid hardcoding log levels and other settings.
