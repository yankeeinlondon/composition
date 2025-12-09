---
_fixed: true
---

# Syntax Highlighting in Rust

For syntax highlighting in Rust, two main crate options are **syntect** (a high-quality, regex-based highlighter) and **tree-sitter** (a parser-based system). While other specialized crates exist, syntect is the most widely used and documented, especially in the Rust ecosystem.

Here is an overview of the two main crates.

| **Crate** | **Primary Approach** | **Key Strength** | **Common Use Case** |
| :--- | :--- | :--- | :--- |
| **`syntect`** | Regex-based (Sublime Text grammars) | High-quality, extensive language support, excellent HTML output | Static site generators, documentation tools, blog engines |
| **`tree-sitter`** | Parser-based (incremental parsing) | Semantic accuracy, incremental re-highlighting, structural understanding | Code editors (Helix, Zed), tools needing fast, accurate updates |

### 🛠️ Using syntect

Syntect is known for being easy to use and producing publication-quality output. Here's a basic example that highlights a Rust code string and outputs to the terminal:

```rust
use syntect::easy::HighlightLines;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::{ThemeSet, Style};
use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};

// Load syntaxes and themes once at startup (expensive operation)
let ps = SyntaxSet::load_defaults_newlines();
let ts = ThemeSet::load_defaults();

// Get the Rust syntax and a theme
let syntax = ps.find_syntax_by_extension("rs").unwrap();
let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);

let code = r#"pub struct Wow { hi: u64 }
fn blah() -> u64 {}"#;

// Highlight each line
for line in LinesWithEndings::from(code) {
    let ranges: Vec<(Style, &str)> = h.highlight_line(line, &ps).unwrap();
    let escaped = as_24_bit_terminal_escaped(&ranges[..], true);
    print!("{}", escaped);
}
```

### 🌳 Using tree-sitter

Tree-sitter uses language-specific grammars and queries for highlighting, offering more semantic understanding. The setup is more involved, as shown in this example:

```rust
use tree_sitter_highlight::{HighlightConfiguration, Highlighter, HtmlRenderer};
use tree_sitter_rust::language_rust;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"fn main() {
        let greeting = "Hello, world!";
        println!("{}", greeting);
    }"#;

    // 1. Initialize the highlighter for Rust
    let mut highlighter = Highlighter::new();
    let language = language_rust();

    // 2. Configure highlights (using standard capture names from queries/highlights.scm)
    let highlights = [
        "keyword", "function", "string", "type", "variable",
        "comment", "constant", "property"
    ];
    let mut config = HighlightConfiguration::new(language, "rust", "", "")?;
    config.configure(&highlights);

    // 3. Perform highlighting
    let events = highlighter.highlight(&config, code.as_bytes(), None, |_| None)?;

    // 4. Render to HTML (for example)
    let mut renderer = HtmlRenderer::new();
    renderer.render(events, code.as_bytes(), &|highlight| {
        // Map highlight names to CSS classes
        match highlight {
            "keyword" => "<span class='kw'>",
            "function" => "<span class='fn'>",
            "string" => "<span class='str'>",
            // ... other mappings
            _ => "<span>",
        }
    })?;

    let html_output = renderer.html();
    println!("{}", html_output);

    Ok(())
}
```

### 📊 How They Compare

Your choice between `syntect` and `tree-sitter` depends heavily on your project's specific needs. This table breaks down their key differences:

| **Comparison Criteria** | **`syntect`** | **`tree-sitter`** |
| :--- | :--- | :--- |
| **Underlying Technology** | Regex patterns from **Sublime Text grammars**. | **Incremental parser** that builds a concrete syntax tree. |
| **Language Support & Quality** | **Very broad** via Sublime's ecosystem. Quality is **high and consistent** across languages. | Broad, but **quality varies** per grammar. May have semantic edge for complex languages. |
| **Performance** | **Fast** for highlighting full documents. | **Excellent for editors** due to incremental updates; initial parse can be comparable. |
| **Output & Integration** | **Excellent HTML/ANSI support** out-of-the-box. Easy to integrate into Rust pipelines (e.g., with `pulldown-cmark`). | Provides **structured data** (nodes, scopes). You handle final rendering, offering more flexibility. |
| **Ease of Use** | **Simpler** for basic use cases (load, highlight, output). | **More complex** setup, requiring grammars and query files. |
| **Maintenance & Ecosystem** | **Mature and stable** in Rust. Grammar updates may lag behind upstream Sublime. | **Actively developed**, but Rust bindings/grammar crates can have version sync issues. |

### 💡 How to Choose

* Choose **`syntect`** if you need to generate **static output** (like for a website or documentation), want the **simplest setup**, and prioritize consistent, high-quality highlighting across many languages.
* Choose **`tree-sitter`** if you are building an **interactive tool like a code editor**, need **incremental re-highlighting**, or require access to the **syntax tree structure** for analysis.

If your project involves highlighting code within Markdown to produce HTML, I can provide more specific guidance on integrating `syntect` with a Markdown parser.
