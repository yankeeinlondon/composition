---
name: anyhow
description: Comprehensive guide to Rust's anyhow crate for application-level error handling
created: 2025-12-08
hash: 1218f9b74d627968
tags:
  - rust
  - error-handling
  - anyhow
  - application-development
---

# The `anyhow` Crate

The `anyhow` crate is a widely-adopted Rust library that provides flexible, application-focused error handling through a single, dynamic error type. It simplifies error handling by using a type-erased wrapper (`anyhow::Error`) that can represent any error implementing `std::error::Error`, eliminating the boilerplate of defining custom error enums while maintaining rich error information for debugging.

The crate's name reflects its philosophy: it allows you to handle errors "anyhow" without needing to precisely categorize every possible error variant upfront. This approach is particularly valuable in applications where errors from multiple subsystems need to be handled uniformly.

## Table of Contents

- [Installation](#installation)
- [Core Concepts](#core-concepts)
- [Key Features](#key-features)
  - [Unified Result Type](#unified-result-type)
  - [Adding Context](#adding-context)
  - [Error Creation Macros](#error-creation-macros)
  - [Downcasting](#downcasting)
  - [Backtrace Support](#backtrace-support)
  - [No-std Support](#no-std-support)
- [Usage Patterns](#usage-patterns)
- [Comparison with Other Crates](#comparison-with-other-crates)
- [Best Practices](#best-practices)
- [Quick Reference](#quick-reference)

## Installation

Add `anyhow` to your `Cargo.toml`:

```toml
[dependencies]
anyhow = "1.0"
```

For no-std environments, disable the default std feature:

```toml
[dependencies]
anyhow = { version = "1.0", default-features = false }
```

Optionally, use `#[macro_use]` to make macros globally visible:

```rust
#[macro_use]
extern crate anyhow; // Makes bail!, ensure!, etc. globally visible
```

## Core Concepts

The key to `anyhow` is its `anyhow::Error` type, a trait object that can wrap any error implementing `std::error::Error`. Rather than requiring explicit error type definitions, `anyhow` provides a generic error type that can be used consistently across your application.

The primary type is `anyhow::Result<T>`, which is a type alias for `Result<T, anyhow::Error>`. This allows all fallible functions in your application to return the same generic error type.

## Key Features

### Unified Result Type

Use `anyhow::Result<T>` as the return type for fallible functions. The `?` operator automatically converts any compatible error into an `anyhow::Error`:

```rust
use anyhow::Result;
use std::fs;

fn read_and_parse_config() -> Result<String> {
    // std::fs::read_to_string returns a std::io::Error on failure.
    // The ? operator automatically converts it to an anyhow::Error.
    let config_content = fs::read_to_string("config.txt")?;

    if config_content.is_empty() {
        // Use the anyhow! macro to create a new, one-off error
        return Err(anyhow::anyhow!("Config file is empty"));
    }

    Ok(config_content)
}

fn main() {
    match read_and_parse_config() {
        Ok(content) => println!("Config content: {}", content),
        Err(e) => eprintln!("Application Error: {:?}", e),
    }
}
```

### Adding Context

The `Context` trait extends `Result` and `Option` types with methods for adding contextual information to errors, creating a semantic trace of what the program was doing when the error occurred:

- `.context()`: Adds static context (evaluated eagerly)
- `.with_context()`: Adds context via a closure (evaluated lazily)

```rust
use anyhow::{Context, Result};
use std::fs;

fn read_user_settings(path: &str) -> Result<String> {
    fs::read_to_string(path)
        // .with_context() takes a closure, useful for expensive context creation
        .with_context(|| format!("Failed to read user settings from {}", path))?
        .parse()
        // .context() takes a static string or any value implementing Display
        .context("Could not parse settings content")
}

fn main() {
    let result = read_user_settings("non_existent_file.json");
    if let Err(e) = result {
        // Output for a missing file:
        // Error: Could not parse settings content
        // Caused by: Failed to read user settings from non_existent_file.json
        // Caused by: No such file or directory (os error 2)
        eprintln!("Error: {:?}", e);
    }
}
```

### Error Creation Macros

`anyhow` provides several macros for creating errors conveniently:

| Macro | Purpose | Example |
|-------|---------|---------|
| `anyhow!` | Creates an error from a formatted string | `anyhow!("value {} is invalid", x)` |
| `bail!` | Returns early with an error | `bail!("operation failed")` |
| `ensure!` | Returns early if condition is false | `ensure!(x > 0, "x must be positive")` |

```rust
use anyhow::{anyhow, bail, ensure, Result};

fn process(value: usize) -> Result<()> {
    // ensure! checks a condition
    ensure!(value % 2 == 0, "value {} is not even", value);

    if value > 1000 {
        // bail! returns early with an error
        bail!("value {} is too large", value);
    }

    // anyhow! creates an error that can be returned or stored
    // return Err(anyhow!("Something went wrong"));

    Ok(())
}
```

### Downcasting

The underlying error can be recovered through downcasting when specific error handling is needed:

```rust
fn handle_error(error: anyhow::Error) {
    // downcast_ref returns Option<&T>
    if let Some(io_err) = error.downcast_ref::<std::io::Error>() {
        eprintln!("Specific IO error occurred: {}", io_err);
    }

    // downcast_mut returns Option<&mut T>
    // downcast returns Result<T, Error> (by value)
}

// Example: matching on specific error variants
match root_cause.downcast_ref::<DataStoreError>() {
    Some(DataStoreError::Censored(_)) => Ok(Poll::Ready(REDACTED_CONTENT)),
    None => Err(error),
}
```

> **Note:** While downcasting is available, it's generally discouraged for application flow control and is typically reserved for logging, reporting, or special-case handling.

### Backtrace Support

With Rust 1.65 or later, `anyhow` automatically captures backtraces when errors are created. Backtraces are controlled via environment variables:

| Variable | Effect |
|----------|--------|
| `RUST_BACKTRACE=1` | Enables backtraces for both panics and errors |
| `RUST_LIB_BACKTRACE=1` | Enables backtraces only for errors |
| `RUST_LIB_BACKTRACE=0` | Disables backtraces for errors (when `RUST_BACKTRACE=1` is set) |

### No-std Support

`anyhow` can be used in no-std environments by disabling the default "std" feature. Almost all of the same API is available, though a global allocator is required.

## Usage Patterns

### Application-Level Error Handling

`anyhow` excels at propagating errors to the top level with helpful context:

```rust
use anyhow::{Context, Result};

fn main() -> Result<()> {
    let config = load_config()
        .context("Failed to load configuration")?;

    run_app(&config)
        .context("Failed to run application")?;

    Ok(())
}

fn load_config() -> Result<Config> {
    let config = std::fs::read_to_string("config.json")?;
    let parsed: Config = serde_json::from_str(&config)?;
    Ok(parsed)
}

fn run_app(config: &Config) -> Result<()> {
    // Application logic...
    Ok(())
}
```

### Error Conversion at Library Boundaries

A common pattern is to use `thiserror` for library error types and convert to `anyhow::Error` at the application boundary:

```rust
// In a library (using thiserror)
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LibraryError {
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
}

// In the application (using anyhow)
use anyhow::Result;

fn use_library() -> Result<()> {
    library_function()?; // Automatically converts LibraryError to anyhow::Error
    Ok(())
}
```

### Combining with Logging

`anyhow` works well with logging libraries like `tracing`:

```rust
use anyhow::{Context, Result};
use tracing::{error, info};

fn process_data() -> Result<()> {
    let data = fetch_data()
        .context("Failed to fetch data")?;

    info!("Successfully fetched {} bytes", data.len());

    let processed = transform(&data)
        .context("Failed to process data")?;

    Ok(())
}

// In error handling code
if let Err(e) = process_data() {
    error!("Error processing data: {:?}", e);
}
```

## Comparison with Other Crates

The choice between error handling crates often depends on whether you are writing an **application** (executable) or a **library** (dependency).

| Crate | Error Type | Primary Use Case | Context Support | Backtraces | Key Features |
|-------|------------|------------------|-----------------|------------|--------------|
| **anyhow** | Dynamic (`anyhow::Error`) | Applications, binaries | `.context()`, `.with_context()` | Automatic (Rust 1.65+) | Easy `?` use, minimal boilerplate |
| **thiserror** | Static (custom `enum`) | Libraries, public APIs | Structured fields in variants | Via underlying error | Derive macro for custom error types |
| **eyre** | Dynamic (`eyre::Report`) | Applications | `.wrap_err()` | Via handlers | Fork of `anyhow` with customization hooks |
| **color-eyre** | Dynamic (`color_eyre::Report`) | CLI tools, user-facing apps | Dynamic strings, span traces | Colored, detailed | Enhanced visual error reports |
| **snafu** | Static (domain-driven) | Complex systems | Structured fields via macros | Via underlying error | Context-specific error types |

### When to Use Each

**Use `anyhow` for:**
- Application code, CLI tools, or services
- Rapid prototyping where development speed is prioritized
- Situations combining errors from multiple libraries with different error types
- Cases where you want to add rich context to errors

**Use `thiserror` for:**
- Library public APIs where consumers need to handle specific error cases
- When downstream users should be able to `match` on specific error conditions

**Use `color-eyre` for:**
- Applications demanding detailed, user-friendly error reports
- CLI tools where visual error presentation matters
- Active development where detailed debugging information is valuable

**Combine them:**
A common best practice is to use `thiserror` to define precise error types within a library, then use `anyhow` or `color-eyre` in the binary that consumes the library.

## Best Practices

### Recommended

1. Use `anyhow::Result<T>` as your return type for fallible functions in applications
2. Add context at error boundaries between subsystems to provide semantic information
3. Use `bail!` and `ensure!` macros for early returns to improve readability
4. Combine with logging to record detailed error information
5. Convert to specific error types at library boundaries when needed

### Avoid

1. Using `anyhow` in public library APIs where precise error types help consumers
2. Downcasting frequently for flow control (use structured error types instead)
3. Ignoring context - always add meaningful context at error boundaries

## Quick Reference

### Type Aliases

```rust
type Result<T> = std::result::Result<T, anyhow::Error>;
```

### Common Imports

```rust
use anyhow::{anyhow, bail, ensure, Context, Result};
```

### Macro Quick Reference

| Macro | Usage |
|-------|-------|
| `anyhow!("message")` | Create an error |
| `anyhow!("fmt {}", val)` | Create a formatted error |
| `bail!("message")` | Return early with error |
| `ensure!(cond, "msg")` | Return error if condition is false |

### Context Methods

| Method | When to Use |
|--------|-------------|
| `.context("static msg")` | Static context message |
| `.with_context(\|\| format!(...))` | Dynamic/expensive context |

### Environment Variables

| Variable | Purpose |
|----------|---------|
| `RUST_BACKTRACE=1` | Enable all backtraces |
| `RUST_LIB_BACKTRACE=1` | Enable error backtraces only |
| `RUST_LIB_BACKTRACE=0` | Disable error backtraces |

## Resources

- [Official Documentation (docs.rs)](https://docs.rs/anyhow)
- [GitHub Repository](https://github.com/dtolnay/anyhow)
- [crates.io Page](https://crates.io/crates/anyhow)
- [thiserror Crate](https://docs.rs/thiserror) - Companion crate for library error types
- [eyre Crate](https://docs.rs/eyre) - Fork with customization hooks
- [color-eyre Crate](https://docs.rs/color-eyre) - Enhanced visual error reports
