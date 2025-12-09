---
_fixed: true
---

# color-eyre Crate Overview

The `color-eyre` crate in Rust is a specialized error report handler that enhances the error reporting capabilities of the foundational `eyre` crate. It's designed to provide colorful, consistent, and well-formatted error reports for both panics and recoverable errors (represented by `eyre::Report`).

It's particularly popular in application development (like CLIs and binaries) where the primary goal of an error is to present clear, debuggable information to the developer or end-user before the program exits.

## Key Features of color-eyre

`color-eyre` builds on the generic error type of `eyre::Report` by customizing how that report is formatted and what diagnostic information it collects.

### Pretty-Printed Reports

The most distinguishing feature is its attractive, colored output. It leverages crates like `color-backtrace` and `color-spantrace` to:

- Colorize backtraces and error messages, making them significantly easier to read and parse quickly in a terminal.
- Present different verbosity levels for the report based on environment variables (`RUST_LIB_BACKTRACE`).
  - **Minimal:** Basic error message.
  - **Short:** Includes a backtrace (`RUST_LIB_BACKTRACE=1`).
  - **Full:** Includes a backtrace and attempts to display source code snippets for the frames where the error originated (`RUST_LIB_BACKTRACE=full`).

### Context and Help Sections

It offers two powerful traits for adding extra information to your error reports:

1. **`WrapErr` (from `eyre`):** Allows you to add a new, descriptive message to an existing error, creating a chain of failures (`Caused by: ...`). This helps track the logical flow of what failed.
2. **`SectionExt`:** Provides methods to attach custom warnings or suggestions (called "Sections") to the error report. These sections are displayed independently from the error chain and are great for providing troubleshooting tips, like "Check your configuration file at /path/to/config.toml."

### Panic and Error Hooks

You install `color-eyre` early in your `main` function using `color_eyre::install()`. This sets up:

- **Error Reporting Hook:** Customizes the output of `eyre::Report` when it's returned from `main`.
- **Panic Hook:** Replaces the default Rust panic handler to ensure that even unrecoverable panics produce the same colorful, detailed reports, including backtraces.

### SpanTrace Integration

When paired with the `tracing` ecosystem, `color-eyre` can capture a `SpanTrace` in addition to a traditional backtrace.

- A backtrace tracks function calls (stack frames).
- A span trace (using the `tracing-error` crate) tracks the user-defined spans (units of work) that were active when the error occurred. This is often more semantic and less noisy than a raw backtrace, especially for asynchronous code.

## Comparison with Other Popular Crates

The choice of error handling crate in Rust generally falls into two categories: Dynamic/Generic (like `anyhow`, `eyre`, and `color-eyre`) and Static/Structured (like `thiserror`).

| Feature | `color-eyre` (via `eyre`) | `anyhow` | `thiserror` |
| :--- | :--- | :--- | :--- |
| **Use Case** | **Application/Binary** (End-user reporting) | **Application/Binary** (Simple, quick error bubbling) | **Library** (Structured, stable API) |
| **Error Type** | **Dynamic** (`eyre::Report`). Type-erased; one type for all errors. | **Dynamic** (`anyhow::Error`). Type-erased; one type for all errors. | **Static** (Custom `enum` or `struct` per library). |
| **Backtraces** | **Built-in** and **colored** with source snippets. Automatically captures a `Backtrace` and `SpanTrace`. | Available via a feature, but less integrated and typically not colored by default. | Does **not** handle backtraces or reporting; focuses only on error definition. |
| **Context/Wrapping** | Excellent via `WrapErr` and additional **`SectionExt`** for user-friendly suggestions. | Excellent via `context` and `wrap_err`. | Excellent via `#[source]` and `#[error]` for defining structured error chains. |
| **Error Matching/Handling** | **Difficult/Requires Downcasting.** The type-erased nature makes `match` cumbersome. | **Difficult/Requires Downcasting.** | **Easy and Idiomatic.** Designed for easy `match` handling by users of the library. |
| **Output** | **Highly Formatted and Colored.** Includes panic hooks and configurable verbosity. | Plain text output by default. No integrated panic/error reporting hooks. | Plain text output, relies on `std::error::Error::Display`. |

### Conclusion on Selection

- **Use `color-eyre` (or `eyre`) in your application's `main` crate:** When your program needs to exit on an error and you want to give the user or developer the most helpful, clear, and debuggable error report possible. It handles the "reporting" part of error handling brilliantly.

- **Use `thiserror` in your library crates:** When you need to define specific, public error types that downstream users can easily match against, handle, and recover from. It handles the "defining" part of error handling.

- **`anyhow`** is a great, simple alternative to `eyre` for applications that don't need the advanced, colorful formatting and diagnostic features that `color-eyre` provides. `eyre` is essentially a fork of `anyhow` that was created to support the highly customizable reporting mechanisms provided by crates like `color-eyre`.
