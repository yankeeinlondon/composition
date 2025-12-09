# tree-sitter-highlight Guide

Low-level Tree-sitter highlighting with maximum control. Use when you need semantic accuracy and full customization.

## When to Use

- Building an editor or IDE-like UI
- Need incremental re-highlighting on code changes
- Want fine-grained control over token categories and theming
- Willing to own the plumbing and build tooling

## Key Concepts

Tree-sitter uses:
- **Language parsers** - generate full AST from source code
- **Highlight queries** - `.scm` files that map AST nodes to highlight scopes
- **Highlighter** - processes code and emits highlight events

## Basic Setup

```rust
use tree_sitter_highlight::{HighlightConfiguration, Highlighter, HighlightEvent};

fn main() -> anyhow::Result<()> {
    // Queries usually come with the grammar or from nvim-treesitter
    let highlight_query = include_str!("queries/rust/highlights.scm");
    let injections_query = "";
    let locals_query = "";

    let mut config = HighlightConfiguration::new(
        tree_sitter_rust::LANGUAGE.into(),
        "rust",
        highlight_query,
        injections_query,
        locals_query,
    )?;

    // Define scope names (map to CSS classes, colors, etc.)
    config.configure(&[
        "keyword",
        "function",
        "type",
        "string",
        "comment",
        "number",
        "operator",
        "variable",
    ]);

    let mut highlighter = Highlighter::new();
    let source = r#"fn main() { println!("Hello!"); }"#;

    let events = highlighter.highlight(&config, source.as_bytes(), None, |_| None)?;

    // Process events...
    Ok(())
}
```

## Rendering to HTML

```rust
let mut styled = String::new();
let mut stack: Vec<&str> = Vec::new();

for event in events {
    match event? {
        HighlightEvent::HighlightStart(hl) => {
            let scope_name = config.names()[hl.0];
            stack.push(scope_name);
            styled.push_str(&format!("<span class=\"{}\">", scope_name));
        }
        HighlightEvent::Source { start, end } => {
            styled.push_str(&html_escape::encode_text(&source[start..end]));
        }
        HighlightEvent::HighlightEnd => {
            stack.pop();
            styled.push_str("</span>");
        }
    }
}
```

## Using pepegsitter

The `pepegsitter` crate bundles many Tree-sitter parsers and highlight queries:

```rust
// Get a Tree-sitter language
let rust_lang = pepegsitter::rust::language();

// Get a ready-made HighlightConfiguration
let rust_config = pepegsitter::rust::highlight();

// Access queries directly for full control
let rust_query = pepegsitter::rust::HIGHLIGHT_QUERY;
```

## Tree-sitter vs TextMate (syntect)

| Aspect | Tree-sitter | TextMate/syntect |
|--------|-------------|------------------|
| Parsing | Full AST, incremental | Regex-based, line-oriented |
| Accuracy | Semantic, structure-aware | Pattern matching |
| Performance | Excellent for editors (incremental) | Fast for batch processing |
| Setup | More complex, need grammars + queries | Simpler, load and go |

## Pros

- Very accurate: full parse tree, not regex
- Excellent for editors with incremental updates
- Access to huge Tree-sitter grammar ecosystem
- pepegsitter saves manual parser wiring

## Cons

- Low-level: you must bring grammars, queries, and implement rendering
- Working with queries can be fiddly
- Deals with unsafe/C code for parsers
- More setup than batteries-included options

## Getting Highlight Queries

Highlight queries (`.scm` files) are typically found:
- In the grammar repository under `queries/highlights.scm`
- From nvim-treesitter's query collection
- Bundled in pepegsitter

## Language Injection

Tree-sitter supports language injection (e.g., CSS in HTML, SQL in strings):

```rust
let events = highlighter.highlight(
    &config,
    source.as_bytes(),
    None,
    |lang_name| {
        // Return HighlightConfiguration for injected language
        match lang_name {
            "css" => Some(&css_config),
            "javascript" => Some(&js_config),
            _ => None,
        }
    },
)?;
```
