# The `markdown` Crate

The `markdown` crate (often referred to as `markdown-rs` in discussions) is a robust, safe, and highly compliant **Markdown parser and compiler written in Rust**. It is a sibling project to the popular JavaScript parser, `micromark`, and aims for 100% compliance with the CommonMark specification.

## Architecture

The `markdown` crate's architecture is built around a **parsing pipeline** that transforms a raw Markdown string into an intermediate Abstract Syntax Tree (AST), which can then be compiled into various formats, primarily **HTML**.

- **State Machine Parser (Tokenization):** The core parsing mechanism uses a **state machine** to process the raw Markdown input. This is a highly robust and byte-accounted approach, ensuring every character is processed and located. It emits concrete tokens with positional information.
- **AST Generation (mdast):** The stream of tokens is used to build the **mdast** (Markdown Abstract Syntax Tree). The AST is a hierarchical, structured representation of the Markdown document's content, making it easier to analyze, manipulate, and compile.
- **Compilation (to HTML):** The AST is then compiled into the final output format, most commonly **HTML**. The crate prioritizes **safe HTML generation** by default, automatically sanitizing potentially dangerous HTML content.

---

## Usage Patterns

The crate exposes simple, high-level functions for quick conversion, as well as more complex functions for granular control over the parsing and compilation process using an `Options` structure.

### Simple Conversion to HTML

The quickest way to use the crate is with the top-level `to_html` function.

```rust
use markdown::to_html;

fn main() {
    let markdown_input = "# Hello, Markdown!\n\nThis is **bold** text.";

    // Convert the markdown string to a safe HTML string
    let html_output = to_html(markdown_input);

    println!("{}", html_output);
    // Output: <h1>Hello, Markdown!</h1>
    // <p>This is <strong>bold</strong> text.</p>
}
```

### Conversion with Custom Options

For fine-grained control, you use `to_html_with_options` and configure the `Options` struct. This is where you toggle CommonMark/GFM extensions and safety features.

```rust
use markdown::{to_html_with_options, Options, ParseOptions, Constructs};

fn main() -> Result<(), String> {
    let gfm_markdown = "| Header 1 | Header 2 |\n| :--- | ---: |\n| Data A | Data B |";

    // Create options to enable GitHub Flavored Markdown (GFM) constructs
    let options = Options {
        parse: ParseOptions {
            // Use the gfm() method to enable GFM features like tables.
            constructs: Constructs::gfm(),
            ..ParseOptions::default()
        },
        ..Options::default()
    };

    let html_output = to_html_with_options(gfm_markdown, &options)?;

    println!("{}", html_output);
    // Output will include <table>, <thead>, and <tbody> tags

    Ok(())
}
```

---

## CommonMark and GFM Support

### CommonMark

The `markdown` crate is **100% compliant** with the **CommonMark** specification. By default, when using `to_html` or the default `Options`, the parser adheres strictly to CommonMark rules.

- **Default Behavior:** Without any explicit configuration, the parser operates in CommonMark mode.
- **Explicit CommonMark:** Using `ParseOptions::default()` explicitly sets the constructs to CommonMark.

### GitHub Flavored Markdown (GFM)

The crate offers **100% support for GFM** extensions, which are a superset of CommonMark. GFM adds features commonly used on platforms like GitHub.

| Feature | CommonMark (Default) | GFM (Enabled with `Constructs::gfm()`) |
| :--- | :--- | :--- |
| **Tables** | Not supported | **Supported** |
| **Strikethrough** | Not supported | **Supported** (using `~~text~~`) |
| **Task Lists** | Not supported | **Supported** (e.g., `- [x] item`) |
| **Autolinks** | Supported for `<URL>` | **Supported** for bare URLs (e.g., `https://example.com`) |

To enable GFM extensions, you use `Constructs::gfm()` in your parsing options.

---

## Use Cases and Examples

### Rendering User-Supplied Content (Web Application)

The most common use case is securely converting Markdown entered by a user (e.g., in a forum post, blog comment, or wiki) into HTML for display. This use case heavily relies on the crate's **default HTML safety features**.

```rust
use markdown::to_html;

fn render_safe_user_content(user_markdown: &str) -> String {
    // The to_html function is safe by default, preventing XSS attacks
    // by removing or sanitizing dangerous HTML tags (e.g., <script>).
    to_html(user_markdown)
}

fn main() {
    let malicious_input = "## My Post\n\n[A link](javascript:alert('xss'))\n\n<script>alert('xss')</script>";

    let safe_html = render_safe_user_content(malicious_input);

    println!("--- Input ---\n{}", malicious_input);
    println!("\n--- Safe HTML Output ---\n{}", safe_html);
    // The <script> tag is removed.
    // The javascript: link is removed or changed to about:blank.
}
```

### Processing Documentation and Technical Content

When processing technical documentation that includes common GFM features like tables, you need to explicitly enable the GFM constructs. This is ideal for static site generators or documentation tools.

```rust
use markdown::{to_html_with_options, Options, ParseOptions, Constructs};

fn render_gfm_doc(doc_markdown: &str) -> Result<String, String> {
    let options = Options {
        parse: ParseOptions {
            // Enable all GFM features
            constructs: Constructs::gfm(),
            ..ParseOptions::default()
        },
        ..Options::default()
    };

    to_html_with_options(doc_markdown, &options)
}

fn main() {
    let doc_input = "# Feature Matrix\n\n| Feature | Supported |\n| :--- | :---: |\n| Tables | ✅ |\n| Strikethrough | ~~Yes~~ |";

    match render_gfm_doc(doc_input) {
        Ok(html) => println!("{}", html),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

### Creating an Abstract Syntax Tree (AST)

Instead of compiling directly to HTML, you can parse the Markdown into an **AST (`mdast`)**. This is useful for advanced tasks like linting, modifying the document structure programmatically, or compiling to non-HTML formats (e.g., LaTeX, plain text).

```rust
use markdown::{to_mdast, to_html_with_options, Options};
use markdown::mdast::{Node, List, ListKind};

fn modify_ast_example(markdown: &str) -> Result<String, String> {
    // Parse into an AST
    let mut tree = to_mdast(markdown, &Options::default())?;

    // Example: Recursively iterate and change a list type from unordered to ordered
    if let Node::Root(root) = &mut tree {
        for node in &mut root.children {
            if let Node::List(list) = node {
                list.kind = ListKind::Ordered; // Mutate the AST
            }
        }
    }

    // Compile the modified AST back to HTML
    to_html_with_options(tree, &Options::default())
}

fn main() {
    let input_markdown = "* Item 1\n* Item 2";

    match modify_ast_example(input_markdown) {
        Ok(html) => println!("Modified HTML:\n{}", html),
        Err(e) => eprintln!("Error: {}", e),
    }
    // Output HTML will be an ordered list:
    // <ol>
    // <li><p>Item 1</p></li>
    // <li><p>Item 2</p></li>
    // </ol>
}
```
