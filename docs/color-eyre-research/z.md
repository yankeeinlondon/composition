---
_fixed: true
---

# What is `color_eyre`?

`color_eyre` is a crate for application-level error handling. Its primary goal is to make errors easy for developers to diagnose and debug. It achieves this by providing a `Result` type alias and a suite of tools for attaching rich, contextual information to errors, which are then printed in a highly readable, colored, and detailed format.

It's a fork of the `eyre` crate, which itself was a fork of the popular `anyhow` crate. The key differentiator for `color_eyre` is its focus on beautiful, customizable, and informative error reports, right out of the box.

## A Detailed Look at `color_eyre`'s Features

Let's break down the components that make `color_eyre` so effective.

### 1. The `eyre::Result` Type Alias

Just like `anyhow`, `color_eyre` provides a convenient type alias:

```rust
pub type Result<T, E = Report> = std::result::Result<T, E>;
```

This means you can replace `std::result::Result<T, MyErrorType>` with the much simpler `eyre::Result<T>` in your function signatures, especially for `main` and other top-level application functions. The `E` defaults to `color_eyre::Report`.

### 2. The `WrapErr` Trait and `.wrap_err()`

This is the core mechanism for adding context. Instead of manually formatting error messages, you use the `.wrap_err()` method, which is added to all `Result` types via the `WrapErr` trait.

```rust
use color_eyre::eyre::{WrapErr, Result};
use std::fs;
use std::path::Path;

fn read_file_contents(path: &Path) -> Result<String> {
    fs::read_to_string(path)
        .wrap_err_with(|| format!("Failed to read file from path: {}", path.display()))
}
```

**Why is this better than `map_err` or `format!`?**

- **Preserves the Original Error:** The original `std::io::Error` is not lost. It's wrapped inside the new `Report`. This is crucial for debugging.
- **Creates a Chain:** You can call `.wrap_err()` at multiple levels of your application, building a "breadcrumb trail" of what went wrong and where.
- **Ergonomic:** The syntax is clean and readable.

### 3. Beautiful, Multi-line Error Reports

This is `color_eyre`'s main selling point. When an error is printed (e.g., using `println!("{:?}", error)`), it doesn't just dump a single line. It produces a structured report:

- **The Error Chain:** It prints each layer of context added by `.wrap_err()`, from the most recent to the original "source" error.
- **Backtraces:** It automatically captures and displays a backtrace, pointing you to the exact source code location where the error originated. This is enabled by the `install()` function.
- **Snippets:** It can even show snippets of your source code around the error location.
- **Color and Symbols:** It uses ANSI color codes and symbols (like `→`, `╰▸`) to make the output easy to parse visually.

### 4. The `Help` Trait

This is a unique and incredibly useful feature. You can attach help text directly to your errors. This is perfect for suggesting solutions to common problems.

You implement the `Help` trait for your custom error types or use `eyre::Help::with_note()` or `with_help()` on a `Report`.

```rust
use color_eyre::eyre::{WrapErr, Result, Help};
use std::env;

fn get_api_key() -> Result<String> {
    env::var("API_KEY")
        .map_err(|e| eyre::Report::new(e))
        .with_help(|| "Set the API_KEY environment variable to access the service. \
                        For example: `export API_KEY=your_secret_key`")
}
```

When this error occurs, the help text will be displayed prominently at the bottom of the error report.

### 5. Installation and Panic Hook Setup

To get the full experience (especially backtraces), you need to install `color_eyre`'s panic and error report handlers at the start of your `main` function.

```rust
use color_eyre::eyre::Result;

fn main() -> Result<()> {
    // Install the global panic and error report handlers.
    // This should be the first thing in main.
    color_eyre::install()?;

    // ... your application logic here ...
    // my_app_logic()?;

    Ok(())
}
```

- `color_eyre::install()` sets up a global hook that captures panics and prints them using the same beautiful formatting as `Report`.
- It returns a `Result`, so you can use the `?` operator to handle any setup failures gracefully.

### 6. Customization

`color_eyre` is highly customizable. You can change the color theme, the symbols used in the output, and more. This is done by creating a `Config` object and passing it to `install()`.

```rust
use color_eyre::{config::HookBuilder, Result};

fn main() -> Result<()> {
    let (panic_hook, eyre_hook) = HookBuilder::default()
        .panic_section("Consider reporting this bug at: https://github.com/my/project/issues")
        .into_hooks();

    // Use the hooks to override the default handlers
    eyre_hook.install()?;
    std::panic::set_hook(Box::new(panic_hook));

    // ...
    Ok(())
}
```

## Putting It All Together: A Practical Example

Imagine a simple CLI tool that reads a configuration file and a data file.

```rust
use color_eyre::eyre::{Context, Result, WrapErr};
use std::fs;
use std::path::Path;

// This function simulates reading a config file
fn read_config(path: &Path) -> Result<serde_json::Value> {
    let content = fs::read_to_string(path)
        .wrap_err_with(|| format!("Failed to read config file at {}", path.display()))?;

    serde_json::from_str(&content)
        .wrap_err("The config file is not valid JSON")
        .with_note(|| format!("File content was:\n---\n{}\n---", content))
}

// This function simulates reading a data file, which requires a value from the config
fn read_data(config_path: &Path, data_path: &Path) -> Result<String> {
    let config = read_config(config_path)
        .wrap_err("Could not load application configuration")?;

    let required_key = config["database_url"]
        .as_str()
        .ok_or_else(|| color_eyre::eyre::eyre!("Missing 'database_url' in config"))?;

    // Let's pretend we use the key to read the data
    if required_key.is_empty() {
        return Err(color_eyre::eyre::eyre!("'database_url' cannot be empty"))
            .with_help("Ensure your config.toml has a non-empty 'database_url' field.");
    }

    fs::read_to_string(data_path)
        .wrap_err_with(|| format!("Failed to read data file from {}", data_path.display()))
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let config_path = Path::new("config.toml");
    let data_path = Path::new("data.csv");

    // Let's assume config.toml is malformed and data.csv doesn't exist
    read_data(config_path, data_path)?;

    println!("Program ran successfully!");
    Ok(())
}
```

If `config.toml` contains invalid JSON, the output would look something like this (colors and symbols included):

```text
Error:
   0: Failed to read data file from data.csv
   1: Could not load application configuration
   2: The config file is not valid JSON
   3: Failed to read config file at config.toml

  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Caused by:
    EOF while parsing a value at line 1 column 0

Note:
    File content was:
    ---
    this is not json
    ---

Backtrace omitted. Run with RUST_BACKTRACE=1 to display it.
Run with RUST_BACKTRACE=full to include source snippets.
```

This output is immediately more useful than a simple `Error: "Failed to read config file at config.toml: EOF while parsing..."`. It shows the chain of events, the root cause, and even the problematic file content.

## Comparison with Other Popular Choices

| Feature                 | `color_eyre` / `eyre`                               | `anyhow`                                          | `thiserror`                                       | `Box<dyn std::error::Error>`                 |
| ----------------------- | --------------------------------------------------- | ------------------------------------------------- | ------------------------------------------------- | -------------------------------------------- |
| **Primary Use Case**    | **Application** error handling (CLIs, servers, GUIs) | **Application** error handling                    | **Library** error type definition                 | Basic, generic error handling                |
| **Error Type**          | `eyre::Report`                                      | `anyhow::Error`                                   | Custom `enum` or `struct` (via derive macro)      | Trait object                                 |
| **Context Addition**    | `.wrap_err()` (preserves source, adds context)      | `.context()` (preserves source, adds context)      | N/A (you define the context in the type)          | Manual `map_err` with `format!`              |
| **Error Reporting**     | **Excellent** (colored, multi-line, backtraces, help) | Good (single-line by default, can be verbose)     | Depends on `Display` impl (usually single-line)   | Poor (just the `Debug` or `Display` output) |
| **Help Text / Notes**   | **Yes**, built-in via `Help` trait                  | No                                                | No                                                | No                                           |
| **Downcasting**         | Yes (`.downcast_ref()`)                             | Yes (`.downcast_ref()`)                           | N/A (type is known)                               | Yes (`.downcast_ref()`)                      |
| **Performance**         | Slightly more overhead due to rich reporting        | Very low overhead                                  | Zero-cost (it's just your type)                   | Low overhead                                  |
| **Philosophy**          | Diagnosable errors for developers                   | Ergonomic error propagation for applications      | Structured, public error APIs for libraries       | Simple, generic error aggregation            |

### `color_eyre` vs. `anyhow`

This is the most direct comparison. They are very similar in purpose and API.

- **Similarity:** Both provide `Result<T, E>` type aliases, a `.context()`/`.wrap_err()` method for chaining errors, and are designed for applications, not libraries.
- **Difference:** The key difference is the **presentation**. `color_eyre`'s `Report` is designed from the ground up to be printed to a terminal. It has the beautiful multi-line format, integrated backtraces, and the unique `Help` text feature. `anyhow`'s `Error` is more of a container; its `Debug` output is useful but not as visually rich or user-friendly.

**Choose `anyhow` if:** You want a lightweight, minimal-dependency way to handle errors in an application and don't need the fancy printing. It's the de-facto standard for simple application error handling.

**Choose `color_eyre` if:** You are building a CLI, a server, or any tool where a developer or user might see the error output directly. The enhanced readability and help text can dramatically improve the debugging experience.

### `color_eyre` vs. `thiserror`

This is a comparison of different paradigms, not direct competitors.

- **`thiserror`** is for **library authors**. When you create a library, you want to expose a stable, well-defined error API so that users of your library can programmatically handle different kinds of errors. `thiserror`'s derive macro makes it trivial to create a structured `enum` for your errors, automatically implementing `Display`, `Error`, and `From`.
- **`color_eyre`** is for **application authors** who *consume* libraries. You use `color_eyre` to handle the errors that come from your own code *and* from the libraries you use (including those that use `thiserror`).

**The common workflow:** A library uses `thiserror` to define its errors. An application uses that library and `color_eyre` to handle any errors it propagates.

```rust
// In a library `my-lib`
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LibError {
    #[error("Invalid configuration: {0}")]
    BadConfig(String),
    #[error("Network failure: {source}")]
    NetworkError { #[from] source: reqwest::Error },
}

// In your application `my-app`
use color_eyre::eyre::Result;
use my_lib::do_something; // This function returns a Result<T, LibError>

fn main() -> Result<()> {
    color_eyre::install()?;
    do_something().wrap_err("The library failed to perform its task")?;
    Ok(())
}
```

Here, `thiserror::Error` automatically implements `Into<color_eyre::Report>`, so the `?` operator works seamlessly.

### `color_eyre` vs. `Box<dyn std::error::Error>`

This is the modern approach vs. the "classic" approach.

- `Box<dyn Error>` is the standard library's way of handling heterogeneous error types. It's flexible but clunky. Adding context requires `map_err`, which often leads to lost source errors and boilerplate `format!` calls. Backtraces are not automatically included.
- `color_eyre` (and `anyhow`) solve all these problems. They provide a concrete type (`Report`) that can hold any error, offer an ergonomic way to add context, and integrate deeply with the Rust debugging ecosystem.

## Conclusion: When to Use `color_eyre`

You should reach for `color_eyre` when:

1. **You are building an application, not a library.** This includes command-line tools, web servers, GUI applications, etc.
2. **Developer experience is a priority.** If you, your team, or your users will be debugging errors from a terminal, the rich output is a massive quality-of-life improvement.
3. **You want to provide helpful hints.** The `Help` trait is perfect for guiding users toward fixing common configuration or runtime issues.
4. **You want a "batteries-included" solution.** With one `install()` call, you get better error reports and better panic reports.

You should **avoid** `color_eyre` (and `anyhow`) in the public API of a library, as it hides the specific error types behind an opaque `Report`. For libraries, `thiserror` remains the gold standard.
