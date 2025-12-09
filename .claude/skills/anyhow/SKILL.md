---
name: anyhow
description: Expert guidance for Rust's anyhow crate - application-level error handling with type-erased errors, context chaining, bail!/ensure! macros, and when to use anyhow vs thiserror vs eyre vs color-eyre
hash: 51e6f8ef8f7bb995
---

# anyhow - Rust Application Error Handling

`anyhow` provides flexible, application-focused error handling using a single dynamic error type. Best for applications, CLI tools, and services where you need to propagate errors with helpful context.

## Core Principles

- Use `anyhow::Result<T>` as return type for fallible application functions
- Add context at error boundaries with `.context()` or `.with_context()`
- Use `bail!` for early returns, `ensure!` for condition checks
- Reserve `anyhow` for applications; use `thiserror` for library public APIs
- Errors auto-convert via `?` operator - no manual `From` implementations needed
- Enable backtraces with `RUST_BACKTRACE=1` for debugging
- Downcast sparingly - only when you must handle specific error types differently

## Quick Setup

```toml
[dependencies]
anyhow = "1.0"
```

```rust
use anyhow::{anyhow, bail, ensure, Context, Result};
```

## Essential Patterns

### Basic Error Propagation

```rust
use anyhow::Result;

fn load_config() -> Result<Config> {
    let content = std::fs::read_to_string("config.json")?;  // io::Error -> anyhow::Error
    let config: Config = serde_json::from_str(&content)?;   // serde::Error -> anyhow::Error
    Ok(config)
}
```

### Adding Context

```rust
use anyhow::{Context, Result};

fn load_user_data(user_id: u32) -> Result<String> {
    let path = format!("data/{user_id}.json");
    std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to load data for user {user_id}"))
}
// Error: "Failed to load data for user 42" caused by "No such file or directory"
```

### Error Creation Macros

```rust
use anyhow::{anyhow, bail, ensure, Result};

fn process(value: usize) -> Result<()> {
    ensure!(value % 2 == 0, "value {value} is not even");  // Condition check
    if value > 1000 {
        bail!("value {value} is too large");  // Early return
    }
    // Or: return Err(anyhow!("Something went wrong"));
    Ok(())
}
```

### Application Main

```rust
use anyhow::{Context, Result};

fn main() -> Result<()> {
    let config = load_config().context("Failed to load configuration")?;
    run_app(&config).context("Application error")?;
    Ok(())
}
```

## Topics

- [Comparison Guide](./comparison.md) - anyhow vs thiserror vs eyre vs color-eyre
- [Advanced Patterns](./advanced.md) - Downcasting, error chaining, no-std usage

## When to Use What

| Scenario | Recommended |
|----------|-------------|
| Application/CLI main code | `anyhow` |
| Library public API | `thiserror` |
| Colorful CLI error output | `color-eyre` |
| Highly customizable errors | `eyre` |
| Combining library + app | `thiserror` in lib, `anyhow` in binary |

## Environment Variables

- `RUST_BACKTRACE=1` - Enable backtraces for panics and errors
- `RUST_LIB_BACKTRACE=1` - Enable backtraces for errors only
- `RUST_LIB_BACKTRACE=0` - Disable error backtraces when RUST_BACKTRACE=1

## Resources

- [Crates.io](https://crates.io/crates/anyhow)
- [Docs.rs](https://docs.rs/anyhow)
- [GitHub](https://github.com/dtolnay/anyhow)
