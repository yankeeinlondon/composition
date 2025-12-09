# Error Handling Crate Comparison

Detailed comparison of Rust's main error handling crates to help you choose the right one.

## Quick Decision Matrix

| Crate | Error Type | Best For | Context Support |
|-------|-----------|----------|-----------------|
| `anyhow` | Dynamic (`anyhow::Error`) | Applications, CLI tools | `.context()`, `.with_context()` |
| `thiserror` | Static (custom enum) | Library public APIs | Structured in enum variants |
| `eyre` | Dynamic (`eyre::Report`) | Customizable applications | `.wrap_err()` |
| `color-eyre` | Dynamic (`color_eyre::Report`) | User-facing CLI tools | Colored, detailed output |
| `snafu` | Static (domain types) | Complex domain systems | Structured via macros |

## anyhow vs thiserror

The most common comparison - they complement each other.

### anyhow

```rust
use anyhow::{Context, Result};

fn fetch_data(url: &str) -> Result<String> {
    reqwest::blocking::get(url)
        .context("HTTP request failed")?
        .text()
        .context("Failed to read response body")
}
```

**Pros:**
- Minimal boilerplate
- Easy `?` propagation from any error type
- Excellent context chaining
- Great for rapid development

**Cons:**
- Not suitable for library APIs (consumers can't match on variants)
- Dynamic dispatch (minor performance cost)

### thiserror

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Invalid data format: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("Resource not found: {resource}")]
    NotFound { resource: String },
}
```

**Pros:**
- Type-safe, matchable error variants
- Clear public API contract
- Consumers can handle specific errors
- Zero runtime overhead

**Cons:**
- More boilerplate
- Must anticipate all error variants upfront

### Combined Pattern

The recommended approach for production code:

```rust
// In your library (my_lib/src/lib.rs)
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LibError {
    #[error("Configuration invalid: {0}")]
    Config(String),
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
}

pub fn library_function() -> Result<Data, LibError> {
    // ...
}

// In your binary (src/main.rs)
use anyhow::{Context, Result};
use my_lib::library_function;

fn main() -> Result<()> {
    let data = library_function()
        .context("Failed to fetch data from library")?;
    // ...
    Ok(())
}
```

## anyhow vs eyre

`eyre` is a fork of `anyhow` focused on customization.

### Key Differences

| Feature | anyhow | eyre |
|---------|--------|------|
| Error macro | `anyhow!` | `eyre!` |
| Option context | `.context()` | `.ok_or_eyre()` |
| Customization | Limited | High (custom handlers) |
| Philosophy | Simple, opinionated | Extensible |

### eyre Example

```rust
use eyre::{eyre, OptionExt, Result, WrapErr};

fn get_user(id: u32) -> Result<User> {
    let user = users.get(&id)
        .ok_or_eyre("User not found")?;  // Different from anyhow

    validate_user(user)
        .wrap_err("User validation failed")?;  // wrap_err instead of context

    Ok(user.clone())
}
```

**Choose eyre when:**
- You need custom error report handlers
- You want fine-grained control over error formatting
- Your project already uses eyre-ecosystem crates

## anyhow vs color-eyre

`color-eyre` extends `eyre` with beautiful terminal output.

### Setup

```rust
use color_eyre::eyre::Result;

fn main() -> Result<()> {
    color_eyre::install()?;  // Required initialization

    run_app()
}
```

### Output Comparison

**anyhow output:**
```
Error: Failed to load config

Caused by:
    0: Could not read file
    1: No such file or directory (os error 2)
```

**color-eyre output:**
```
Error:
   0: Failed to load config
   1: Could not read file
   2: No such file or directory (os error 2)

Location:
   src/main.rs:42

Backtrace:
   [colored, formatted backtrace]

Suggestion: Check that the config file exists at the expected path
```

**Choose color-eyre when:**
- Building user-facing CLI tools
- You want rich debugging information
- Visual error presentation matters
- You're integrating with `tracing` for span capture

## Feature Comparison Table

| Feature | anyhow | thiserror | eyre | color-eyre |
|---------|--------|-----------|------|------------|
| Dynamic errors | Yes | No | Yes | Yes |
| Static errors | No | Yes | No | No |
| Context chaining | Yes | Limited | Yes | Yes |
| Automatic backtraces | Yes (1.65+) | Via source | Via handler | Yes |
| Colorized output | No | No | No | Yes |
| Custom handlers | No | N/A | Yes | Yes |
| no-std support | Yes | Yes | Yes | No |
| Setup complexity | Minimal | Minimal | Minimal | Moderate |

## Migration Paths

### anyhow to eyre

```rust
// Change imports
- use anyhow::{anyhow, Context, Result};
+ use eyre::{eyre, Result, WrapErr};

// Change Option handling
- opt.context("missing value")
+ opt.ok_or_eyre("missing value")

// context() becomes wrap_err()
- .context("failed")
+ .wrap_err("failed")
```

### anyhow to color-eyre

Same as eyre migration, plus:

```rust
fn main() -> color_eyre::eyre::Result<()> {
    color_eyre::install()?;  // Add this line
    // ... rest of your code
}
```
