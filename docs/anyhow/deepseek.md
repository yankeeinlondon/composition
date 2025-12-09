---
_fixed: true
---

# The `anyhow` Crate

The `anyhow` crate provides a flexible, application-focused error handling system by using a single, dynamic error type. It's designed to minimize boilerplate while providing rich context for debugging.

## Core Features and Usage

The key to `anyhow` is its `anyhow::Error` type, a trait object that can wrap any error implementing the standard `std::error::Error` trait. Its main components and typical setup are as follows:

**Installation and Setup**
Add `anyhow` to your `Cargo.toml`. For macros to be available throughout your project, you can use `#[macro_use]` in your main file.

```toml
[dependencies]
anyhow = "1.0"
```

```rust
#[macro_use]
extern crate anyhow; // Makes bail!, ensure!, etc. globally visible
```

**Key Features in Practice**

- **Unified Result Type**: Use `anyhow::Result<T>` as the return type for fallible functions. This allows the `?` operator to automatically convert any compatible error into an `anyhow::Error`.

    ```rust
    use anyhow::Result;
    fn get_config() -> Result<String> {
        let config = std::fs::read_to_string("config.json")?; // io::Error converted to anyhow::Error
        Ok(config)
    }
    ```

- **Attaching Context**: Use the `.context()` or `.with_context()` methods to add descriptive messages to errors, creating a clear chain of what went wrong.

    ```rust
    use anyhow::{Context, Result};
    fn load_user_data(user_id: u32) -> Result<String> {
        let path = format!("data/{user_id}.json");
        let data = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read data for user {user_id}"))?;
        Ok(data)
    }
    // Error output: "Failed to read data for user 42" caused by "No such file or directory"
    ```

- **Creating Errors**: Use the `anyhow!` macro for one-off errors or `bail!` to return early. The `ensure!` macro checks a condition.

    ```rust
    use anyhow::{bail, ensure, Result};
    fn process(value: usize) -> Result<()> {
        ensure!(value % 2 == 0, "value {value} is not even"); // Checks condition
        if value > 1000 {
            bail!("value {value} is too large"); // Returns early with error
        }
        // Alternatively: return Err(anyhow!("Something went wrong"));
        Ok(())
    }
    ```

- **Handling Specific Errors**: You can "downcast" an `anyhow::Error` back to its original type to handle specific cases.

    ```rust
    fn handle_error(error: anyhow::Error) {
        if let Some(e) = error.downcast_ref::<std::io::Error>() {
            eprintln!("Specific IO error occurred: {e}");
        }
    }
    ```

- **Backtrace Support**: With Rust 1.65+, `anyhow` automatically captures backtraces on error if enabled via the `RUST_BACKTRACE=1` environment variable.

## Comparison with Other Error Handling Crates

`anyhow` is best for application development. For libraries or different needs, other crates are more suitable. Here is a comparison:

| Tool | Error Type | Primary Use Case | Context Support | Backtraces | Key Features / Notes |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **`anyhow`** | Dynamic (`anyhow::Error`) | **Applications**, binaries, scripts | Dynamic strings (`.context()`) | Automatic (Rust ≥1.65) | Easy `?` use, minimal boilerplate. |
| **`thiserror`** | Static (your custom `enum`) | **Libraries**, public APIs | Structured fields (in enum variants) | Via underlying error | Derive macro for custom, library-friendly error types. |
| **`eyre`** | Dynamic (`eyre::Report`) | Applications (like `anyhow`) | Dynamic strings (`.wrap_err()`) | Via `color-eyre` or `stable-eyre` | Fork of `anyhow` with different hooks for error reporting. |
| **`color-eyre`** | Dynamic (`color_eyre::Report`) | Applications demanding detailed, user-friendly reports | Dynamic strings & **automatic span traces** | **Colored, detailed backtraces** | Fork of `eyre` with colorful output, panic and error hooks, section tracing. |
| **`snafu`** | Static (domain-driven types) | Complex systems requiring granular error contexts | **Structured fields** attached via macros | Via underlying error | Creates context-specific error types, more boilerplate but precise control. |

## How to Choose

- **Use `anyhow`** for **applications, CLI tools, or services** where your main goal is to propagate errors with helpful messages and you don't need callers to handle specific error variants.
- **Use `thiserror`** when building a **library** where you want to expose a stable, typed API so users can match on specific, documented error conditions.
- **Use `color-eyre`** for application development where you want **extremely detailed, visually enhanced error and panic reports**, especially during active development or for user-facing binaries.
- **Combine them**: A common pattern is to use `thiserror` to define precise error types within a library and then use `anyhow` or `color-eyre` in the binary that consumes the library for easy error handling.
