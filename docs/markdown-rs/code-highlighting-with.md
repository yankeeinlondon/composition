# Integrating `markdown-rs` with Syntax Highlighting Crates in Rust

## Overview of `markdown-rs`

`markdown-rs` is a CommonMark-compliant markdown parser for Rust that provides AST (Abstract Syntax Tree) functionality and extensions. Unlike some parsers that operate on event streams, `markdown-rs` builds a full AST, which makes it particularly well-suited for complex transformations and plugin development.

## Integration Approaches with Syntax Highlighting Crates

### Integration with `syntect`

`syntect` is a popular syntax highlighting library that uses Sublime Text syntax definitions for high-quality highlighting.

#### Code Example

```rust
use markdown::{to_html_with_options, CompileOptions, Options};
use syntect::html::{highlighted_html_for_string, ClassStyle};
use syntect::parsing::SyntaxSet;
use syntect::highlighting::ThemeSet;

fn markdown_with_syntect_highlighting(input: &str) -> String {
    let syntax_set = SyntaxSet::load_defaults_newlines();
    let theme_set = ThemeSet::load_defaults();

    let options = Options {
        compile: CompileOptions {
            allow_dangerous_html: true,
            ..Default::default()
        },
        ..Default::default()
    };

    // First convert markdown to HTML
    let html = to_html_with_options(input, &options).unwrap();

    // Then process code blocks with syntect
    // This is a simplified example - in practice you'd need to parse HTML
    // and replace code blocks with highlighted versions
    html
}
```

#### Pros

- **Mature and well-tested** with extensive language support
- **High performance** with efficient highlighting algorithms
- **Rich theme ecosystem** with many pre-built themes
- **Good documentation** and community adoption

#### Cons

- **Requires HTML post-processing** since `markdown-rs` doesn't expose event streams
- **Additional dependency** on Sublime Text syntax definitions
- **Limited to static highlighting** without semantic analysis

### Integration with `autumnus`

`autumnus` is a modern syntax highlighter powered by Tree-sitter and Neovim themes.

#### Code Example

```rust
use markdown::{to_html_with_options, CompileOptions, Options};
use autumnus::{highlight, Options as AutumnusOptions, FormatterOption};

fn markdown_with_autumnus_highlighting(input: &str) -> String {
    let options = Options {
        compile: CompileOptions {
            allow_dangerous_html: true,
            ..Default::default()
        },
        ..Default::default()
    };

    // Convert markdown to HTML first
    let html = to_html_with_options(input, &options).unwrap();

    // Process code blocks with autumnus
    // This would require parsing the HTML and extracting code blocks
    // Here's how you would highlight a code string:
    let code = r#"
fn main() {
    println!("Hello, world!");
}
"#;

    let highlighted = highlight(
        code,
        AutumnusOptions {
            lang_or_file: Some("rust"),
            formatter: FormatterOption::HtmlInline {
                pre_class: Some("code-block"),
                italic: false,
                include_highlights: false,
                theme: None,
                highlight_lines: None,
                header: None,
            },
        },
    );

    html // In practice, replace code blocks with highlighted versions
}
```

#### Pros

- **Tree-sitter powered** for more accurate parsing
- **Modern theming** with Neovim theme support
- **Multiple output formats** (HTML, terminal)
- **Active development** and modern Rust practices

#### Cons

- **Relatively new** with less community adoption
- **Fewer language grammars** than syntect (though growing)
- **Similar post-processing requirements** as syntect
- **Additional Tree-sitter grammar compilation** step

### Integration with `tree-sitter`

Direct integration with Tree-sitter provides the most flexible and powerful approach.

#### Code Example

```rust
use markdown::{to_html_with_options, CompileOptions, Options};
use tree_sitter::{Language, Parser};
use tree_sitter_highlight::{HighlightConfiguration, Highlighter, HtmlRenderer};

extern "C" { fn tree_sitter_rust() -> Language; }

fn markdown_with_treesitter_highlighting(input: &str) -> String {
    let options = Options {
        compile: CompileOptions {
            allow_dangerous_html: true,
            ..Default::default()
        },
        ..Default::default()
    };

    // Convert markdown to HTML
    let html = to_html_with_options(input, &options).unwrap();

    // Set up Tree-sitter highlighter
    let language = unsafe { tree_sitter_rust() };
    let mut highlight_config = HighlightConfiguration::new(
        language,
        "rust",
        tree_sitter_rust::highlight_query(),
        tree_sitter_rust::injections_query(),
        tree_sitter_rust::locals_query(),
    ).unwrap();

    // This would require parsing HTML and processing each code block
    // Here's a conceptual example of highlighting a code string:
    let source_code = "fn main() { println!(\"Hello\"); }";
    let mut highlighter = Highlighter::new();
    let mut renderer = HtmlRenderer::new();

    let events = highlighter.highlight(
        &highlight_config,
        source_code.as_bytes(),
        None,
        |_| None
    ).unwrap();

    renderer.render(events, source_code.as_bytes(), &|highlight| {
        format!(" class=\"{}\"", highlight.attribute())
    }).unwrap();

    html // In practice, replace code blocks with highlighted versions
}
```

#### Pros

- **Most accurate parsing** with full language grammars
- **Semantic highlighting** capabilities
- **Incremental parsing** for performance
- **Language-agnostic framework** for many languages

#### Cons

- **Most complex integration** requiring significant setup
- **Requires language-specific grammar compilation**
- **Higher memory footprint** due to parsing tables
- **Steeper learning curve** for query configuration

## Comparison of Highlighting Solutions

| Feature | `syntect` | `autumnus` | `tree-sitter` |
|---------|-----------|------------|---------------|
| **Parsing approach** | Regex-based | Tree-sitter | Tree-sitter |
| **Setup complexity** | Low | Medium | High |
| **Language support** | Extensive | Growing | Extensive |
| **Semantic accuracy** | Basic | Good | Excellent |
| **Performance** | Excellent | Good | Variable |
| **Theme support** | Sublime Text | Neovim | Custom |
| **Output formats** | HTML | HTML, Terminal | HTML, Custom |

## Implementation Considerations

### AST Traversal Approach

Since `markdown-rs` builds an AST rather than exposing event streams, you'll need to:

1. Parse the markdown to an AST
2. Traverse the AST to find code block nodes
3. Extract code content and language info
4. Apply syntax highlighting
5. Replace code blocks with highlighted versions

### HTML Post-Processing Approach

Alternatively, you can:

1. Generate HTML from markdown
2. Parse the HTML to find code elements
3. Replace code content with highlighted versions
4. Serialize back to HTML

### Plugin Architecture

`markdown-rs` supports a plugin architecture that could potentially be used for syntax highlighting, though this would require developing a custom plugin.

## Recommendations

### For Quick Integration

Choose **`syntect`** for its ease of use and extensive language support.

### For Modern Applications

Choose **`autumnus`** for its Tree-sitter foundation and modern theming.

### For Maximum Accuracy

Choose **`tree-sitter`** directly for semantic highlighting and language precision.

## Conclusion

Integrating syntax highlighting with `markdown-rs` requires additional work compared to event-stream parsers like `pulldown-cmark`. However, the AST-based approach provides more flexibility for complex transformations. Each highlighting solution has distinct advantages:

- **`syntect`** offers the most straightforward integration with good performance
- **`autumnus`** provides modern Tree-sitter-based highlighting with great theming
- **`tree-sitter`** delivers the most accurate highlighting at the cost of complexity

The choice depends on your specific requirements for accuracy, performance, and implementation complexity. For most applications, `autumnus` strikes a good balance between modern features and implementation simplicity.
