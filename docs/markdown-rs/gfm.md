# GitHub Flavored Markdown with `markdown-rs`

Practical examples showing how to configure and use GitHub Flavored Markdown (GFM) features with the `markdown-rs` crate in Rust.

## Basic Setup and API

The primary way to use GFM features is through the `to_html_with_options` function. You configure the desired features by setting boolean flags in the `Constructs` struct, which is part of the `CompileOptions`.

Here's a basic example that enables all GFM features:

```rust
use markdown::{to_html_with_options, CompileOptions, Constructs, Options};

fn main() {
    let markdown_input = "Hello, **world**!";

    // Configure options to enable all GFM features
    let options = Options {
        compile: CompileOptions {
            constructs: Constructs::gfm(), // Enables all GFM constructs
            ..CompileOptions::default()
        },
        ..Options::default()
    };

    let html_output = to_html_with_options(markdown_input, &options)
        .expect("Failed to compile markdown");

    println!("{}", html_output);
}
```

The `Constructs::gfm()` method is a convenient shortcut that enables the standard set of GFM features like tables, task lists, footnotes, strikethrough, and autolink literals.

## Selective Feature Configuration

You can enable GFM features individually for finer control. This is done by setting specific fields in the `Constructs` struct to `true`.

```rust
use markdown::{to_html_with_options, CompileOptions, Constructs, Options};

fn main() {
    // Example markdown using several GFM features
    let markdown_input = r#"
## GFM Features Example

### Task List
- [x] Completed task
- [ ] Incomplete task

### Strikethrough
~~This text is crossed out~~

### Table
| Header 1 | Header 2 |
|----------|----------|
| Cell A   | Cell B   |
| Cell C   | Cell D   |

### Autolink Literal
Visit https://example.com or email contact@example.com
"#;

    // Configure specific GFM features
    let options = Options {
        compile: CompileOptions {
            constructs: Constructs {
                // Enable specific GFM features
                gfm_table: true,          // For tables using | and -
                gfm_task_list_item: true, // For task lists [x] or [ ]
                gfm_strikethrough: true,  // For ~~strikethrough~~
                gfm_autolink_literal: true, // For autolinking URLs/emails
                // Disable GFM footnotes if not needed
                gfm_footnote_definition: false,
                gfm_footnote_label: false,
                gfm_footnote_reference: false,
                // Keep CommonMark features enabled
                ..Constructs::default()
            },
            ..CompileOptions::default()
        },
        ..Options::default()
    };

    let html_output = to_html_with_options(markdown_input, &options)
        .expect("Failed to compile markdown");

    println!("{}", html_output);
}
```

## Parsing to an AST (Abstract Syntax Tree)

For advanced use cases, you can parse markdown into a structured AST using `to_mdast()`, manipulate it, and then compile it to HTML.

```rust
use markdown::{to_mdast, to_html_with_options, CompileOptions, Constructs, Options, ParseOptions};
use markdown::mdast::Node;

fn main() {
    let markdown_input = r#"
## Project Tasks
- [x] Design database schema
- [ ] Implement API endpoints
- [ ] Write documentation

~~Old feature~~ → **New feature**
"#;

    // First, parse to AST with GFM enabled
    let parse_options = ParseOptions {
        constructs: Constructs::gfm(),
        ..ParseOptions::default()
    };

    let ast = to_mdast(markdown_input, &parse_options)
        .expect("Failed to parse markdown");

    // You can now programmatically inspect or manipulate the AST
    if let Node::Root(root) = ast {
        println!("Parsed document with {} direct children", root.children.len());
        // Process nodes here...
    }

    // Then compile to HTML
    let compile_options = CompileOptions {
        constructs: Constructs::gfm(),
        ..CompileOptions::default()
    };

    let options = Options {
        parse: parse_options,
        compile: compile_options,
    };

    let html_output = to_html_with_options(markdown_input, &options)
        .expect("Failed to compile markdown");

    println!("\nGenerated HTML:\n{}", html_output);
}
```

## Summary of Key GFM Constructs

| **Feature** | **Construct Field** | **Markdown Syntax Example** |
|-------------|-------------------|----------------------------|
| Tables | `gfm_table: true` | `\| Header \|` |
| Task Lists | `gfm_task_list_item: true` | `- [x] Task` |
| Strikethrough | `gfm_strikethrough: true` | `~~text~~` |
| Autolink Literals | `gfm_autolink_literal: true` | `https://example.com` |
| Footnotes | `gfm_footnote_*: true` | `[^1]` and `[^1]: note` |

## Important Notes

1. **Safety**: By default, `markdown-rs` sanitizes output for security. If you need to allow raw HTML or custom protocols, use `allow_dangerous_html` and `allow_dangerous_protocol` options.
2. **Performance**: For very large documents, consider processing in a separate thread to prevent blocking.
3. **AST Manipulation**: The AST nodes are defined in `mdast.rs` and follow the unist specification, allowing for detailed programmatic analysis and transformation.
