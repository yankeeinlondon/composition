---
name: markdown-rs
description: Expert knowledge for parsing and rendering Markdown with markdown-rs (the `markdown` crate), a spec-compliant CommonMark/GFM parser with AST support. Use when building documentation systems, static site generators, content management, or any Rust application processing Markdown.
---

# markdown-rs (Rust Markdown Parser)

`markdown-rs` is a **spec-compliant**, **safe-by-default** Markdown parser for Rust. It provides 100% CommonMark and GFM compliance with full AST access through the mdast format.

**Key insight**: Unlike streaming parsers, markdown-rs builds a complete AST, making it ideal for document analysis, manipulation, and rich transformations - at the cost of some memory overhead.

## Modular Knowledge Base

| Module | When to Use |
|--------|-------------|
| [GFM & Extensions](./gfm.md) | Tables, task lists, strikethrough, footnotes |
| [MDX Support](./mdx.md) | JSX components, expressions, ESM imports |
| [Frontmatter](./frontmatter.md) | YAML/TOML metadata at document start |
| [Syntax Highlighting](./syntax-highlighting.md) | Integrating syntect, autumnus, or tree-sitter |
| [Axum Integration](./axum-integration.md) | Web APIs, middleware, stateful Markdown services |
| [LSP Integration](./lsp-integration.md) | Editor tooling, real-time diagnostics, completions |
| [vs pulldown-cmark](./comparison.md) | Choosing between the two main Rust parsers |

## Core Dependencies

```toml
[dependencies]
markdown = "1.0.0-alpha.21"  # The crate name is "markdown", commonly called markdown-rs
```

**Sibling projects**: Part of the unified ecosystem with JavaScript's `micromark` and `mdxjs-rs` for MDX compilation.

## Architecture

```txt
┌─────────────────────────────────────────────┐
│  Markdown Input                             │
├─────────────────────────────────────────────┤
│  State Machine Tokenizer                    │
│  Byte-by-byte parsing with position info    │
├─────────────────────────────────────────────┤
│  Token Stream → Event Stream                │
├───────────────────┬─────────────────────────┤
│  to_html()        │  to_mdast()             │
│  Direct HTML      │  Full AST (mdast)       │
│  output           │  for manipulation       │
└───────────────────┴─────────────────────────┘
```

**Design philosophy**: Safe HTML by default. Raw HTML is escaped unless explicitly allowed. The parser is `#![no_std]` + `alloc`, suitable for embedded/WASM environments.

**Compliance**: 100% CommonMark compliant (650+ spec tests), 100% GFM compliant (1000+ additional tests), with fuzz testing for robustness.

## Quick Start

### Basic HTML Conversion

```rust
use markdown::to_html;

let html = to_html("# Hello, *world*!");
assert_eq!(html, "<h1>Hello, <em>world</em>!</h1>\n");
```

### With GFM Extensions

```rust
use markdown::{to_html_with_options, Options};

let input = r#"
| Feature | Status |
|---------|--------|
| Tables  | Yes    |

- [x] Task complete
- [ ] Task pending

~~Strikethrough~~
"#;

let html = to_html_with_options(input, &Options::gfm()).unwrap();
```

### AST Access

```rust
use markdown::{to_mdast, ParseOptions};
use markdown::mdast::{Node, Heading};

let ast = to_mdast("# Title\n\nParagraph.", &ParseOptions::default()).unwrap();

// Walk the tree
fn walk(node: &Node) {
    match node {
        Node::Heading(Heading { depth, children, .. }) => {
            println!("H{}: {:?}", depth, children);
        }
        _ => {
            if let Some(children) = node.children() {
                for child in children {
                    walk(child);
                }
            }
        }
    }
}
walk(&ast);
```

## Options Structure

```rust
use markdown::{Options, ParseOptions, CompileOptions, Constructs};

let options = Options {
    parse: ParseOptions {
        constructs: Constructs::gfm(),  // Enable GFM features
        ..ParseOptions::default()
    },
    compile: CompileOptions {
        allow_dangerous_html: false,      // Default: safe
        allow_dangerous_protocol: false,  // Default: safe
        ..CompileOptions::default()
    },
};
```

### Key Options

| Option | Purpose | Default |
|--------|---------|---------|
| `Constructs::gfm()` | Enable all GFM features | CommonMark only |
| `allow_dangerous_html` | Pass through raw HTML | `false` (escaped) |
| `allow_dangerous_protocol` | Allow `javascript:` etc. | `false` (blocked) |
| `gfm_table` | Table support | `false` |
| `gfm_strikethrough` | `~~text~~` support | `false` |
| `gfm_task_list_item` | `- [x]` support | `false` |
| `gfm_autolink_literal` | Auto-link URLs | `false` |

## Common Patterns

### Safe User Content Rendering

```rust
use markdown::to_html;

fn render_user_content(input: &str) -> String {
    // Default is safe: no raw HTML, no javascript: links
    to_html(input)
}
```

### Extracting All Links

```rust
use markdown::{to_mdast, ParseOptions, mdast::{Node, Link}};
use std::collections::HashSet;

fn extract_links(markdown: &str) -> HashSet<String> {
    let ast = to_mdast(markdown, &ParseOptions::default()).unwrap();
    let mut links = HashSet::new();

    fn walk(node: &Node, links: &mut HashSet<String>) {
        if let Node::Link(Link { url, .. }) = node {
            links.insert(url.clone());
        }
        if let Some(children) = node.children() {
            for child in children {
                walk(child, links);
            }
        }
    }

    walk(&ast, &mut links);
    links
}
```

### Document Outline (Table of Contents)

```rust
use markdown::{to_mdast, ParseOptions, mdast::{Node, Heading, Text}};

struct TocEntry {
    depth: u8,
    text: String,
}

fn extract_toc(markdown: &str) -> Vec<TocEntry> {
    let ast = to_mdast(markdown, &ParseOptions::default()).unwrap();
    let mut entries = Vec::new();

    fn walk(node: &Node, entries: &mut Vec<TocEntry>) {
        if let Node::Heading(Heading { depth, children, .. }) = node {
            let text: String = children.iter()
                .filter_map(|c| if let Node::Text(Text { value, .. }) = c { Some(value.as_str()) } else { None })
                .collect();
            entries.push(TocEntry { depth: *depth, text });
        }
        if let Some(children) = node.children() {
            for child in children {
                walk(child, entries);
            }
        }
    }

    walk(&ast, &mut entries);
    entries
}
```

## Decision Guide

| Scenario | Recommendation |
|----------|----------------|
| Maximum spec compliance needed | markdown-rs |
| Need AST for manipulation/analysis | markdown-rs |
| MDX/JSX support | markdown-rs (with mdxjs-rs) |
| Maximum throughput | Consider pulldown-cmark |
| Streaming/low memory | Consider pulldown-cmark |
| Cross-language consistency with JS | markdown-rs (matches micromark) |

## Best Practices

1. **Use default options for untrusted input** - safe HTML by default
2. **Use `to_mdast` for document analysis** - full AST access
3. **Use `to_html` for simple rendering** - fastest path
4. **Enable only needed constructs** - better performance
5. **Pair with a sanitizer if allowing raw HTML** - defense in depth
6. **Cache parsed ASTs** for repeated access to same document

## Error Handling

```rust
use markdown::{to_html_with_options, Options};

fn render(input: &str) -> Result<String, String> {
    to_html_with_options(input, &Options::gfm())
        .map_err(|e| format!("Parse error: {}", e.reason))
}
```

The `Message` type includes:
- `reason`: Human-readable error description
- `point`: Position (line, column, offset) where error occurred
- `source`: Optional source error

## Detailed Guides

| Topic | Description |
|-------|-------------|
| [GFM & Extensions](./gfm.md) | Enabling and using GitHub Flavored Markdown features |
| [MDX Support](./mdx.md) | JSX in Markdown, expressions, ESM imports |
| [Frontmatter](./frontmatter.md) | YAML/TOML/JSON metadata parsing |
| [Syntax Highlighting](./syntax-highlighting.md) | Integration with syntect, autumnus, tree-sitter |
| [Axum Integration](./axum-integration.md) | Building web APIs with Markdown processing |
| [LSP Integration](./lsp-integration.md) | Real-time editor features, diagnostics |
| [vs pulldown-cmark](./comparison.md) | Detailed comparison, when to use each |

## References

- [markdown-rs Documentation](https://docs.rs/markdown)
- [markdown-rs GitHub](https://github.com/wooorm/markdown-rs)
- [CommonMark Specification](https://spec.commonmark.org/)
- [GFM Specification](https://github.github.com/gfm/)
- [mdast Specification](https://github.com/syntax-tree/mdast)
