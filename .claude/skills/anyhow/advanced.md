# Advanced anyhow Patterns

Advanced usage patterns for `anyhow` including downcasting, error chaining, and special configurations.

## Downcasting Errors

Recover the original error type when you need specific handling.

### By Reference

```rust
fn handle_error(error: &anyhow::Error) {
    if let Some(io_err) = error.downcast_ref::<std::io::Error>() {
        match io_err.kind() {
            std::io::ErrorKind::NotFound => {
                eprintln!("File not found - creating default");
            }
            std::io::ErrorKind::PermissionDenied => {
                eprintln!("Permission denied - check file permissions");
            }
            _ => eprintln!("IO error: {io_err}"),
        }
    }
}
```

### By Value (Consuming)

```rust
fn recover_specific_error(error: anyhow::Error) -> Result<(), anyhow::Error> {
    match error.downcast::<MyRecoverableError>() {
        Ok(recoverable) => {
            // Handle the specific error
            recoverable.attempt_recovery()?;
            Ok(())
        }
        Err(original_error) => {
            // Not the expected type, propagate
            Err(original_error)
        }
    }
}
```

### By Mutable Reference

```rust
fn modify_error(error: &mut anyhow::Error) {
    if let Some(custom_err) = error.downcast_mut::<CustomError>() {
        custom_err.add_retry_count();
    }
}
```

**Warning:** Frequent downcasting often indicates you should use `thiserror` with explicit variants instead.

## Error Chain Traversal

Iterate through the chain of errors to find root causes.

```rust
use anyhow::Error;

fn log_error_chain(error: &Error) {
    eprintln!("Error: {error}");

    for (i, cause) in error.chain().skip(1).enumerate() {
        eprintln!("  Caused by [{i}]: {cause}");
    }
}

fn find_root_cause(error: &Error) -> &(dyn std::error::Error + 'static) {
    error.chain().last().unwrap()
}

fn contains_io_error(error: &Error) -> bool {
    error.chain()
        .any(|cause| cause.downcast_ref::<std::io::Error>().is_some())
}
```

## Context Patterns

### Lazy Context (Expensive to Compute)

```rust
use anyhow::{Context, Result};

fn process_file(path: &Path) -> Result<Data> {
    std::fs::read(path)
        // Closure only called on error
        .with_context(|| format!("Failed to read {}", path.display()))
}
```

### Static Context (Cheap)

```rust
fn quick_operation() -> Result<()> {
    something_fallible()
        .context("Quick operation failed")  // No allocation on success
}
```

### Chaining Multiple Contexts

```rust
fn complex_operation(id: u32) -> Result<Output> {
    let data = fetch_data(id)
        .context("Fetch failed")?;

    let processed = transform_data(&data)
        .context("Transform failed")?;

    save_output(&processed)
        .with_context(|| format!("Failed to save output for id {id}"))
}
// Error: "Failed to save output for id 42"
// Caused by: "Transform failed"
// Caused by: "Fetch failed"
// Caused by: [original error]
```

## No-std Support

Use `anyhow` in embedded or no-std environments.

### Cargo.toml

```toml
[dependencies]
anyhow = { version = "1.0", default-features = false }
```

### Requirements

- Global allocator required (for `Box<dyn Error>`)
- Most API identical to std version
- No backtrace support in no-std mode

### Example

```rust
#![no_std]
extern crate alloc;

use anyhow::{anyhow, Result};
use alloc::string::String;

fn no_std_function() -> Result<String> {
    if some_condition {
        return Err(anyhow!("Something went wrong"));
    }
    Ok(String::from("success"))
}
```

## Integration with Logging

### With tracing

```rust
use anyhow::{Context, Result};
use tracing::{error, info, instrument};

#[instrument(skip(config))]
fn process_request(config: &Config) -> Result<Response> {
    let data = fetch_data()
        .context("Data fetch failed")?;

    info!(bytes = data.len(), "Data fetched successfully");

    process(&data)
        .context("Processing failed")
}

fn handle_request(config: &Config) {
    if let Err(e) = process_request(config) {
        // Log full error chain
        error!(error = ?e, "Request processing failed");
    }
}
```

### With log crate

```rust
use anyhow::{Context, Result};
use log::{error, info};

fn main() -> Result<()> {
    env_logger::init();

    if let Err(e) = run_app() {
        error!("Application error: {:#}", e);  // Pretty-print with causes
        std::process::exit(1);
    }
    Ok(())
}
```

## Display Formats

Different ways to format anyhow errors:

```rust
let error: anyhow::Error = /* ... */;

// Just the top-level message
println!("{}", error);
// Output: "Failed to process data"

// Full chain (alternate format)
println!("{:#}", error);
// Output: "Failed to process data: Invalid format: unexpected EOF"

// Debug format (with backtrace if available)
println!("{:?}", error);
// Output: Full debug representation including backtrace

// Debug alternate format
println!("{:#?}", error);
// Output: Pretty-printed debug with indentation
```

## Creating Errors from Non-Error Types

### From strings

```rust
use anyhow::anyhow;

let error = anyhow!("Something went wrong");
let error = anyhow!("Failed to process item {}", item_id);
```

### Wrapping arbitrary types

```rust
use anyhow::Error;

// Any type implementing Display + Debug + Send + Sync + 'static
struct CustomData {
    code: i32,
    message: String,
}

impl std::fmt::Display for CustomData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::fmt::Debug for CustomData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CustomData {{ code: {}, message: {:?} }}", self.code, self.message)
    }
}

impl std::error::Error for CustomData {}

let error: Error = CustomData {
    code: 404,
    message: "Not found".into()
}.into();
```

## Testing with anyhow

### Asserting error messages

```rust
#[test]
fn test_error_message() {
    let result = fallible_function();
    let err = result.unwrap_err();

    assert!(err.to_string().contains("expected substring"));
}
```

### Asserting error types

```rust
#[test]
fn test_specific_error() {
    let result = fallible_function();
    let err = result.unwrap_err();

    assert!(err.downcast_ref::<std::io::Error>().is_some());
}
```

### Testing error chains

```rust
#[test]
fn test_error_chain() {
    let result = complex_function();
    let err = result.unwrap_err();

    let messages: Vec<_> = err.chain()
        .map(|e| e.to_string())
        .collect();

    assert_eq!(messages, vec![
        "High-level context",
        "Mid-level context",
        "Root cause"
    ]);
}
```
