---
_fixed: true
---

# Clap: Building Command-Line Interfaces in Rust

The **clap** crate is the standard for building command-line interfaces in Rust, offering two main API styles: the **Derive API** and the **Builder API**. Each has distinct advantages, and the best choice depends on your project's needs and your personal preference as a developer.

Here is a comparison of the two primary modes of operation:

| **Aspect** | **Derive API** | **Builder API** |
| :--- | :--- | :--- |
| **Approach & Syntax** | **Declarative**. Uses Rust structs and attributes (`#[derive(Parser)]`). | **Imperative**. Uses a fluent, method-chaining interface. |
| **Ease of Use** | **Very high**. Minimal boilerplate; argument definition and storage are unified. | **Moderate**. More verbose, requires manually connecting argument definitions and storage. |
| **Flexibility & Control** | **High for most cases**. Covers 90% of use cases. Dynamic logic is harder to express. | **Very high**. Allows dynamic argument construction and advanced, conditional logic. |
| **Compile Time** | **Slightly higher** due to procedural macro use. | **Lower**, unless you are already using other procedural macros. |
| **Best For** | • Most applications<br>• Quick prototyping<br>• When you prefer a clean, declarative style | • Highly dynamic interfaces<br>• When compile time is a major concern<br>• Integration into complex, conditional logic |

## Code Examples

Here are concrete examples showing how to build the same simple CLI tool with both APIs.

### Using the Derive API

This approach is concise and integrates the CLI definition with the data structure holding the values.

```rust
use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    name: String,

    /// Number of times to greet
    #[arg(short, long, default_value_t = 1)]
    count: u8,
}

fn main() {
    let args = Args::parse();
    for _ in 0..args.count {
        println!("Hello, {}!", args.name);
    }
}
```

### Using the Builder API

This approach offers detailed, step-by-step control over the parser's construction.

```rust
use clap::{Arg, Command};

fn main() {
    let matches = Command::new("demo")
        .author("Author Name")
        .version("1.0")
        .about("Simple program to greet a person")
        .arg(
            Arg::new("name")
                .short('n')
                .long("name")
                .value_name("NAME")
                .help("Name of the person to greet")
                .required(true),
        )
        .arg(
            Arg::new("count")
                .short('c')
                .long("count")
                .value_name("COUNT")
                .help("Number of times to greet")
                .default_value("1"),
        )
        .get_matches();

    let name = matches.get_one::<String>("name").expect("required");
    let count: u8 = matches
        .get_one::<String>("count")
        .unwrap()
        .parse()
        .expect("count should be a number");

    for _ in 0..count {
        println!("Hello, {}!", name);
    }
}
```

## Advanced Features and Ecosystem

Beyond basic parsing, `clap` provides features for production-grade tools:

- **Shell Completion**: The `clap_complete` crate can generate completion scripts for shells like bash, zsh, and fish at runtime or compile time. This makes your tool feel more polished and user-friendly.
- **Layered Configuration**: For complex applications, you can layer command-line arguments on top of settings from config files and environment variables. A common pattern is to use `serde` to deserialize a config file, then let `clap` parse the command line, merging the two sources with clear precedence rules.

## How to Choose and Next Steps

Here are some practical guidelines for selecting the right API and where to go from here:

- **Start with the Derive API** if you are new to `clap` or building a typical CLI application. Its simplicity and clarity help you get results quickly.
- **Switch to or use the Builder API** if you find yourself needing to generate arguments dynamically based on runtime conditions (e.g., loading a list of valid choices from a file).
- **You can mix both APIs** within the same project if necessary. For instance, you can use the Derive API for the main application structure and the Builder API to construct a particularly complex subcommand dynamically.

To effectively use `clap`:

1. **Add it to your `Cargo.toml`**. For the Derive API, include the `derive` feature: `clap = { version = "4", features = ["derive"] }`.
2. **Explore the official documentation** for details on validation, custom types, subcommands, and styling.
3. **Use the `--help` output**. One of `clap`'s strengths is its automatically generated, professional-looking help messages; use them as a design tool for your interface.
