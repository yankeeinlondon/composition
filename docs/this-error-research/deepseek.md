---
_fixed: true
---

# Rust's `thiserror` Crate

The `thiserror` crate in Rust is a powerful tool that simplifies the creation of custom, structured error types for libraries and applications. It uses Rust's procedural macros to automatically generate boilerplate code for the `std::error::Error`, `Display`, and `From` traits.

## Key Features of `thiserror`

`thiserror` streamlines error definition through a declarative macro system. Here are its core capabilities:

- **Automatic Trait Implementation**: By adding `#[derive(Error)]` to an enum or struct, `thiserror` automatically implements the `std::error::Error` trait for your type. Combined with Rust's standard `#[derive(Debug)]`, it fulfills the two mandatory traits for errors.
- **Concise Error Messages**: The `#[error("...")]` attribute lets you define the `Display` format for each error variant directly above its definition. This format string can interpolate fields from the variant.

    ```rust
    #[derive(Error, Debug)]
    pub enum MyError {
        #[error("Invalid header. Expected: {expected}, found: {found}")]
        InvalidHeader { expected: String, found: String },
    }
    ```

- **Easy Error Chaining**: The `#[from]` attribute can be placed on a field (e.g., `io::Error`). This automatically generates a `From` conversion, allowing you to use the `?` operator to propagate underlying errors and have `thiserror` correctly implement the `source()` method.

    ```rust
    #[derive(Error, Debug)]
    pub enum MyError {
        #[error("I/O error occurred")]
        IoError(#[from] std::io::Error), // Allows `?` to auto-convert io::Error
    }
    ```

- **Transparent Wrapping**: The `#[error(transparent)]` attribute delegates the `Display` and `source` methods entirely to the underlying error type, useful for wrapping errors without adding new context.

## Comparison with Other Popular Crates

`thiserror` is one of several prominent error-handling solutions in Rust. The choice between them depends primarily on your project's needs: whether you are building a **library** (which exposes error types for callers to handle) or an **application** (which typically logs or displays errors).

The table below summarizes how `thiserror` compares to `anyhow` and `snafu`:

| Dimension | **`thiserror`** | **`anyhow`** | **`snafu`** |
| :--- | :--- | :--- | :--- |
| **Core Philosophy** | Define **static, custom error types** for precise handling. | Use a **unified, dynamic error type** for simplicity. | Add **structured context** to errors for complex systems. |
| **Error Type** | Your own `enum` or `struct`. | Single `anyhow::Error` type (a trait object). | Your own types, generated with context. |
| **Context Support** | Via fields in your error variants. | Dynamic strings via `.context()` method. | Structured context attached via *report* or *whatever* attributes. |
| **Primary Use Case** | **Library development**, where callers need to match on specific error variants. | **Application development**, prototypes, or places where errors are primarily logged or reported. | **Complex systems** where detailed, chainable error context is critical. |
| **Learning Curve** | Medium (need to design error types). | Low (immediate use). | High (more complex concepts). |

## How to Choose the Right Tool

Selecting a crate depends on your project's stage and the caller's need to handle errors programmatically.

1. **Use `thiserror` when building a library or API.** Its main strength is defining clear, matchable error types that are part of your public API, giving users maximum control.
2. **Use `anyhow` for applications, binaries, or quick prototyping.** It excels where the main goal is to propagate errors to a top-level handler with helpful messages, not to expose types for matching.
3. **Consider `snafu` for large, complex applications.** If you need extremely detailed, structured error context that forms a clear "error stack" without the performance cost of backtraces, `snafu` is a powerful choice.

## Code Examples: Contrasting Approaches

The difference in philosophy is clearest in code. For a function that might fail, here are two approaches:

- **With `thiserror` (for libraries):**
    The caller can precisely handle different failures.

    ```rust
    // Library exposes a clear, matchable enum.
    #[derive(Error, Debug)]
    pub enum ParserError {
        #[error("I/O error: {0}")]
        Io(#[from] std::io::Error),
        #[error("Failed to parse integer from '{input}'")]
        ParseInt { input: String, source: std::num::ParseIntError },
    }
    // Caller can handle specific cases.
    match parse_file("data.txt") {
        Err(ParserError::Io { .. }) => { /* retry logic */ },
        Err(ParserError::ParseInt { input, .. }) => { /* notify about bad data */ },
        _ => {},
    }
    ```

- **With `anyhow` (for applications):**
    The error is easily propagated and formatted for the user, but its type is opaque.

    ```rust
    // Application uses a simple, unified type.
    fn process() -> Result<(), anyhow::Error> {
        let data = std::fs::read_to_string("data.txt")
            .context("Failed to read config file")?; // Add context
        let _num: i32 = data.trim().parse()
            .context("Config file did not contain a valid number")?;
        Ok(())
    }
    // Error is typically just logged or displayed.
    if let Err(e) = process() {
        eprintln!("Error: {}", e); // Shows: "Failed to read config file"
    }
    ```

## Best Practices and Trends

A modern trend, especially in large projects like databases, involves creating a **"virtual error stack"** or **"stacked error"**. This approach attaches lightweight, structured context (like query IDs or operation stages) at multiple points as an error propagates. It provides much more debuggable information than a root cause alone, and is more performant and targeted than capturing full system backtraces. While `snafu` is explicitly designed for this, you can achieve similar patterns with `thiserror` by carefully designing your error enums to carry contextual information.
