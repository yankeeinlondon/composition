# markdown-rs Deep Dive

A comprehensive guide to parsing and rendering Markdown with markdown-rs (the `markdown` crate), a spec-compliant CommonMark/GFM parser with full AST support for Rust applications.

## Overview

`markdown-rs` is a **spec-compliant**, **safe-by-default** Markdown parser for Rust. It provides 100% CommonMark and GFM compliance with full AST access through the mdast format.

**Key insight**: Unlike streaming parsers, markdown-rs builds a complete AST, making it ideal for document analysis, manipulation, and rich transformations - at the cost of some memory overhead.

**Sibling projects**: Part of the unified ecosystem with JavaScript's `micromark` and `mdxjs-rs` for MDX compilation.

### Core Dependencies

```toml
[dependencies]
markdown = "1.0.0-alpha.21"  # The crate name is "markdown", commonly called markdown-rs
```

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

### Key Options Reference

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

## GitHub Flavored Markdown (GFM)

GFM extends CommonMark with features commonly used on GitHub: tables, task lists, strikethrough, autolinks, and footnotes.

### Enabling GFM

#### All GFM Features

```rust
use markdown::{to_html_with_options, Options};

let html = to_html_with_options(input, &Options::gfm())?;
```

#### Selective Features

```rust
use markdown::{to_html_with_options, Options, ParseOptions, Constructs};

let options = Options {
    parse: ParseOptions {
        constructs: Constructs {
            gfm_table: true,
            gfm_task_list_item: true,
            gfm_strikethrough: true,
            gfm_autolink_literal: true,
            // Disable footnotes if not needed
            gfm_footnote_definition: false,
            gfm_footnote_label: false,
            gfm_footnote_reference: false,
            ..Constructs::default()
        },
        ..ParseOptions::default()
    },
    ..Options::default()
};

let html = to_html_with_options(input, &options)?;
```

### GFM Feature Reference

#### Tables

```markdown
| Left | Center | Right |
|:-----|:------:|------:|
| L    | C      | R     |
```

**Construct flag**: `gfm_table: true`

**Generated HTML**:

```html
<table>
<thead><tr><th align="left">Left</th><th align="center">Center</th><th align="right">Right</th></tr></thead>
<tbody><tr><td align="left">L</td><td align="center">C</td><td align="right">R</td></tr></tbody>
</table>
```

**AST Node**: `Node::Table` with `align` vector and `Node::TableRow`/`Node::TableCell` children.

#### Task Lists

```markdown
- [x] Completed task
- [ ] Incomplete task
- Regular list item
```

**Construct flag**: `gfm_task_list_item: true`

**Generated HTML**:

```html
<ul>
<li><input type="checkbox" disabled checked /> Completed task</li>
<li><input type="checkbox" disabled /> Incomplete task</li>
<li>Regular list item</li>
</ul>
```

**AST Node**: `Node::ListItem` with `checked: Option<bool>` field.

#### Strikethrough

```markdown
~~deleted text~~
```

**Construct flag**: `gfm_strikethrough: true`

**Generated HTML**: `<del>deleted text</del>`

**AST Node**: `Node::Delete` with children.

#### Autolink Literals

```markdown
Visit https://example.com or email user@example.com
```

**Construct flag**: `gfm_autolink_literal: true`

URLs and emails are automatically linked without requiring `<angle brackets>`.

#### Footnotes

```markdown
Here's a statement with a footnote[^1].

[^1]: This is the footnote content.
```

**Construct flags**:

- `gfm_footnote_reference: true` - for `[^1]` references
- `gfm_footnote_definition: true` - for `[^1]: content` definitions
- `gfm_footnote_label: true` - for label parsing

### Working with GFM AST

#### Extracting Task List Status

```rust
use markdown::{to_mdast, ParseOptions, Constructs, mdast::{Node, ListItem}};

fn extract_tasks(markdown: &str) -> Vec<(String, bool)> {
    let options = ParseOptions {
        constructs: Constructs::gfm(),
        ..ParseOptions::default()
    };
    let ast = to_mdast(markdown, &options).unwrap();
    let mut tasks = Vec::new();

    fn walk(node: &Node, tasks: &mut Vec<(String, bool)>) {
        if let Node::ListItem(ListItem { checked: Some(done), children, .. }) = node {
            let text = extract_text(children);
            tasks.push((text, *done));
        }
        if let Some(children) = node.children() {
            for child in children {
                walk(child, tasks);
            }
        }
    }

    fn extract_text(nodes: &[Node]) -> String {
        nodes.iter()
            .map(|n| match n {
                Node::Text(t) => t.value.clone(),
                Node::Paragraph(p) => extract_text(&p.children),
                _ => String::new(),
            })
            .collect()
    }

    walk(&ast, &mut tasks);
    tasks
}
```

#### Extracting Table Data

```rust
use markdown::{to_mdast, ParseOptions, Constructs};
use markdown::mdast::{Node, Table, TableRow, TableCell, AlignKind};

struct TableData {
    headers: Vec<String>,
    alignments: Vec<AlignKind>,
    rows: Vec<Vec<String>>,
}

fn extract_table(markdown: &str) -> Option<TableData> {
    let options = ParseOptions {
        constructs: Constructs::gfm(),
        ..ParseOptions::default()
    };
    let ast = to_mdast(markdown, &options).ok()?;

    fn find_table(node: &Node) -> Option<&Table> {
        match node {
            Node::Table(t) => Some(t),
            _ => node.children()?.iter().find_map(find_table),
        }
    }

    let table = find_table(&ast)?;
    let alignments = table.align.clone();

    let mut rows_iter = table.children.iter();
    let header_row = rows_iter.next()?;
    let headers = extract_row_text(header_row);

    let rows: Vec<Vec<String>> = rows_iter.map(|row| extract_row_text(row)).collect();

    Some(TableData { headers, alignments, rows })
}

fn extract_row_text(row: &Node) -> Vec<String> {
    if let Node::TableRow(TableRow { children, .. }) = row {
        children.iter().map(|cell| {
            if let Node::TableCell(TableCell { children, .. }) = cell {
                cell_to_text(children)
            } else {
                String::new()
            }
        }).collect()
    } else {
        vec![]
    }
}

fn cell_to_text(nodes: &[Node]) -> String {
    nodes.iter()
        .filter_map(|n| if let Node::Text(t) = n { Some(t.value.as_str()) } else { None })
        .collect()
}
```

### GFM Constructs Reference

| Construct | Flag | Markdown Syntax |
|-----------|------|-----------------|
| Tables | `gfm_table` | `\| cell \|` with `---` separators |
| Task Lists | `gfm_task_list_item` | `- [x]` or `- [ ]` |
| Strikethrough | `gfm_strikethrough` | `~~text~~` |
| Autolink Literals | `gfm_autolink_literal` | bare `https://...` or `email@...` |
| Footnote References | `gfm_footnote_reference` | `[^label]` |
| Footnote Definitions | `gfm_footnote_definition` | `[^label]: content` |
| Footnote Labels | `gfm_footnote_label` | Label parsing for footnotes |
| Tag Filter | `gfm_tagfilter` | Filters dangerous HTML tags |

## MDX Support

MDX extends Markdown with JSX components, JavaScript expressions, and ESM imports. markdown-rs provides full MDX support through configurable constructs.

### Enabling MDX

```rust
use markdown::{to_html_with_options, Options, ParseOptions, Constructs};

let options = Options {
    parse: ParseOptions {
        constructs: Constructs {
            mdx_esm: true,           // import/export statements
            mdx_expression_flow: true,  // {expressions} in flow
            mdx_expression_text: true,  // {expressions} in text
            mdx_jsx_flow: true,      // <Component /> in flow
            mdx_jsx_text: true,      // <Component /> in text
            ..Constructs::gfm()
        },
        ..ParseOptions::default()
    },
    ..Options::default()
};

let html = to_html_with_options(mdx_input, &options)?;
```

### MDX Features

#### JSX Components (Flow)

```mdx
# My Document

<Alert type="warning">
  This is a warning message.
</Alert>

<Card>
  ## Card Title
  Some content inside the card.
</Card>
```

**Construct**: `mdx_jsx_flow: true`

#### JSX Components (Text/Inline)

```mdx
Read the <Link href="/docs">documentation</Link> for more info.
```

**Construct**: `mdx_jsx_text: true`

#### Expressions (Flow)

```mdx
{/* This is a comment */}

{items.map(item => (
  <li key={item.id}>{item.name}</li>
))}
```

**Construct**: `mdx_expression_flow: true`

#### Expressions (Text/Inline)

```mdx
The answer is {2 + 2}.

Hello, {user.name}!
```

**Construct**: `mdx_expression_text: true`

#### ESM Imports/Exports

```mdx
import { Button } from './components'
import data from './data.json'

export const metadata = {
  title: 'My Page'
}

# Page Content

<Button onClick={() => console.log('clicked')}>
  Click me
</Button>
```

**Construct**: `mdx_esm: true`

### Full MDX Configuration

```rust
use markdown::{Options, ParseOptions, CompileOptions, Constructs};

fn mdx_options() -> Options {
    Options {
        parse: ParseOptions {
            constructs: Constructs {
                // MDX features
                mdx_esm: true,
                mdx_expression_flow: true,
                mdx_expression_text: true,
                mdx_jsx_flow: true,
                mdx_jsx_text: true,

                // GFM features (often used with MDX)
                gfm_table: true,
                gfm_task_list_item: true,
                gfm_strikethrough: true,
                gfm_autolink_literal: true,

                // Math support
                math_flow: true,
                math_text: true,

                ..Constructs::default()
            },
            mdx_esm_parse: None,          // Custom ESM parser
            mdx_expression_parse: None,   // Custom expression parser
            ..ParseOptions::default()
        },
        compile: CompileOptions {
            // MDX often needs dangerous HTML for components
            allow_dangerous_html: true,
            ..CompileOptions::default()
        },
    }
}
```

### MDX AST Node Types

| Node Type | Description | Example |
|-----------|-------------|---------|
| `MdxJsxFlowElement` | Block-level JSX | `<Card>...</Card>` |
| `MdxJsxTextElement` | Inline JSX | `<Link>text</Link>` |
| `MdxFlowExpression` | Block-level expression | `{items.map(...)}` |
| `MdxTextExpression` | Inline expression | `{user.name}` |
| `MdxjsEsm` | Import/export | `import X from 'y'` |

### Integration with mdxjs-rs

For full MDX compilation to JavaScript, use the sibling `mdxjs-rs` crate:

```toml
[dependencies]
markdown = "1.0.0-alpha.21"
mdxjs = "0.2"
```

```rust
use mdxjs::{compile, Options};

fn compile_mdx(input: &str) -> Result<String, String> {
    let options = Options {
        development: false,
        ..Options::default()
    };

    compile(input, &options)
        .map_err(|e| e.to_string())
}
```

### Extracting Component Usage

```rust
use markdown::mdast::{Node, MdxJsxFlowElement, MdxJsxTextElement};
use std::collections::HashSet;

fn extract_component_names(ast: &Node) -> HashSet<String> {
    let mut components = HashSet::new();

    fn walk(node: &Node, components: &mut HashSet<String>) {
        match node {
            Node::MdxJsxFlowElement(MdxJsxFlowElement { name: Some(n), .. }) |
            Node::MdxJsxTextElement(MdxJsxTextElement { name: Some(n), .. }) => {
                // Only track PascalCase names (components, not HTML elements)
                if n.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                    components.insert(n.clone());
                }
            }
            _ => {}
        }

        if let Some(children) = node.children() {
            for child in children {
                walk(child, components);
            }
        }
    }

    walk(ast, &mut components);
    components
}
```

## Frontmatter

Frontmatter allows embedding metadata (YAML, TOML, or JSON) at the beginning of Markdown documents. markdown-rs supports parsing frontmatter as part of the AST.

### Enabling Frontmatter

```rust
use markdown::{to_mdast, ParseOptions, Constructs};

let options = ParseOptions {
    constructs: Constructs {
        frontmatter: true,  // Enable frontmatter parsing
        ..Constructs::default()
    },
    ..ParseOptions::default()
};

let ast = to_mdast(input, &options)?;
```

### Frontmatter Formats

#### YAML Frontmatter

```markdown
---
title: My Document
author: John Doe
date: 2024-01-15
tags:
  - rust
  - markdown
---

# Document Content
```

Delimited by `---` on both sides.

#### TOML Frontmatter

```markdown
+++
title = "My Document"
author = "John Doe"
date = 2024-01-15
tags = ["rust", "markdown"]
+++

# Document Content
```

Delimited by `+++` on both sides.

### Extracting Frontmatter from AST

```rust
use markdown::{to_mdast, ParseOptions, Constructs};
use markdown::mdast::{Node, Yaml, Toml};

fn extract_frontmatter(input: &str) -> Option<String> {
    let options = ParseOptions {
        constructs: Constructs {
            frontmatter: true,
            ..Constructs::default()
        },
        ..ParseOptions::default()
    };

    let ast = to_mdast(input, &options).ok()?;

    // Frontmatter is typically the first child of Root
    if let Node::Root(root) = &ast {
        if let Some(first) = root.children.first() {
            match first {
                Node::Yaml(Yaml { value, .. }) => return Some(value.clone()),
                Node::Toml(Toml { value, .. }) => return Some(value.clone()),
                _ => {}
            }
        }
    }

    None
}
```

### Parsing YAML Frontmatter

```rust
use markdown::{to_mdast, ParseOptions, Constructs};
use markdown::mdast::{Node, Yaml};
use serde::Deserialize;

#[derive(Deserialize)]
struct Metadata {
    title: String,
    author: Option<String>,
    date: Option<String>,
    tags: Option<Vec<String>>,
}

fn parse_yaml_frontmatter(input: &str) -> Option<Metadata> {
    let options = ParseOptions {
        constructs: Constructs {
            frontmatter: true,
            ..Constructs::default()
        },
        ..ParseOptions::default()
    };

    let ast = to_mdast(input, &options).ok()?;

    if let Node::Root(root) = &ast {
        if let Some(Node::Yaml(Yaml { value, .. })) = root.children.first() {
            return serde_yaml::from_str(value).ok();
        }
    }

    None
}
```

### Generic Frontmatter Handler

```rust
use markdown::{to_mdast, ParseOptions, Constructs};
use markdown::mdast::Node;
use serde::de::DeserializeOwned;

enum FrontmatterFormat {
    Yaml(String),
    Toml(String),
    None,
}

fn detect_frontmatter(input: &str) -> FrontmatterFormat {
    let options = ParseOptions {
        constructs: Constructs {
            frontmatter: true,
            ..Constructs::default()
        },
        ..ParseOptions::default()
    };

    let ast = match to_mdast(input, &options) {
        Ok(ast) => ast,
        Err(_) => return FrontmatterFormat::None,
    };

    if let Node::Root(root) = &ast {
        if let Some(first) = root.children.first() {
            match first {
                Node::Yaml(y) => return FrontmatterFormat::Yaml(y.value.clone()),
                Node::Toml(t) => return FrontmatterFormat::Toml(t.value.clone()),
                _ => {}
            }
        }
    }

    FrontmatterFormat::None
}

fn parse_frontmatter<T: DeserializeOwned>(input: &str) -> Option<T> {
    match detect_frontmatter(input) {
        FrontmatterFormat::Yaml(yaml) => serde_yaml::from_str(&yaml).ok(),
        FrontmatterFormat::Toml(toml) => toml::from_str(&toml).ok(),
        FrontmatterFormat::None => None,
    }
}
```

### Static Site Generator Pattern

```rust
use markdown::{to_mdast, to_html_with_options, ParseOptions, Options, Constructs};
use markdown::mdast::{Node, Yaml};
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize, Default)]
struct PageMeta {
    title: Option<String>,
    description: Option<String>,
    date: Option<String>,
    draft: Option<bool>,
    layout: Option<String>,
    tags: Option<Vec<String>>,
}

struct Page {
    meta: PageMeta,
    content_html: String,
    source_path: String,
}

fn load_page(path: &Path) -> Result<Page, Box<dyn std::error::Error>> {
    let source = std::fs::read_to_string(path)?;

    let parse_options = ParseOptions {
        constructs: Constructs {
            frontmatter: true,
            ..Constructs::gfm()
        },
        ..ParseOptions::default()
    };

    let options = Options {
        parse: parse_options.clone(),
        ..Options::gfm()
    };

    // Parse metadata
    let meta = match to_mdast(&source, &parse_options)? {
        Node::Root(root) => {
            match root.children.first() {
                Some(Node::Yaml(Yaml { value, .. })) => {
                    serde_yaml::from_str(value).unwrap_or_default()
                }
                _ => PageMeta::default(),
            }
        }
        _ => PageMeta::default(),
    };

    let content_html = to_html_with_options(&source, &options)?;

    Ok(Page {
        meta,
        content_html,
        source_path: path.display().to_string(),
    })
}
```

### Frontmatter Dependencies

```toml
[dependencies]
markdown = "1.0.0-alpha.21"
serde = { version = "1.0", features = ["derive"] }
serde_yaml = "0.9"  # For YAML frontmatter
toml = "0.8"        # For TOML frontmatter
serde_json = "1.0"  # For JSON frontmatter
```

## Syntax Highlighting

Integrate syntax highlighting into Markdown-rendered code blocks using syntect, autumnus, or tree-sitter.

### Integration Approaches

Since markdown-rs builds an AST rather than exposing event streams, there are two main approaches:

1. **AST Traversal**: Parse to AST, find code blocks, apply highlighting
2. **HTML Post-Processing**: Generate HTML, parse to find `<code>` elements, replace with highlighted versions

### Solution Comparison

| Solution | Approach | Pros | Cons |
|----------|----------|------|------|
| **syntect** | Regex-based (Sublime) | Mature, extensive languages, fast | Not semantic |
| **autumnus** | Tree-sitter | Modern, Neovim themes, accurate | Growing language support |
| **tree-sitter** | Direct parsing | Most accurate, semantic | Complex setup |

### syntect Integration

#### Dependencies

```toml
[dependencies]
markdown = "1.0.0-alpha.21"
syntect = "5.2"
```

#### AST-Based Approach

```rust
use markdown::{to_mdast, to_html_with_options, ParseOptions, Options};
use markdown::mdast::{Node, Code};
use syntect::html::{ClassedHTMLGenerator, ClassStyle};
use syntect::parsing::SyntaxSet;

struct Highlighter {
    syntax_set: SyntaxSet,
}

impl Highlighter {
    fn new() -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
        }
    }

    fn highlight(&self, code: &str, lang: &str) -> String {
        let syntax = self.syntax_set
            .find_syntax_by_token(lang)
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());

        let mut generator = ClassedHTMLGenerator::new_with_class_style(
            syntax,
            &self.syntax_set,
            ClassStyle::Spaced,
        );

        for line in code.lines() {
            generator.parse_html_for_line_which_includes_newline(line)
                .expect("failed to parse line");
        }

        format!("<pre><code class=\"language-{}\">{}</code></pre>",
            lang,
            generator.finalize()
        )
    }

    fn process_markdown(&self, input: &str) -> String {
        let ast = to_mdast(input, &ParseOptions::gfm()).unwrap();
        let mut html = to_html_with_options(input, &Options::gfm()).unwrap();

        self.walk_and_replace(&ast, &mut html);
        html
    }

    fn walk_and_replace(&self, node: &Node, html: &mut String) {
        if let Node::Code(Code { value, lang, .. }) = node {
            let lang = lang.as_deref().unwrap_or("text");
            let original = format!("<pre><code class=\"language-{}\">{}</code></pre>",
                lang,
                html_escape(value)
            );
            let highlighted = self.highlight(value, lang);
            *html = html.replace(&original, &highlighted);
        }

        if let Some(children) = node.children() {
            for child in children {
                self.walk_and_replace(child, html);
            }
        }
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
```

#### Inline Theme (No CSS Required)

```rust
use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;

fn highlight_with_theme(code: &str, lang: &str) -> String {
    let syntax_set = SyntaxSet::load_defaults_newlines();
    let theme_set = ThemeSet::load_defaults();
    let theme = &theme_set.themes["base16-ocean.dark"];

    let syntax = syntax_set
        .find_syntax_by_token(lang)
        .unwrap_or_else(|| syntax_set.find_syntax_plain_text());

    highlighted_html_for_string(code, &syntax_set, syntax, theme)
        .unwrap_or_else(|_| format!("<pre><code>{}</code></pre>", html_escape(code)))
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
```

### autumnus Integration

#### Dependencies

```toml
[dependencies]
markdown = "1.0.0-alpha.21"
autumnus = "0.2"
```

#### Basic Usage

```rust
use markdown::{to_mdast, ParseOptions};
use markdown::mdast::{Node, Code};
use autumnus::{highlight, Options as AutumnusOptions, FormatterOption};

fn highlight_code(code: &str, lang: &str) -> String {
    highlight(
        code,
        AutumnusOptions {
            lang_or_file: Some(lang),
            formatter: FormatterOption::HtmlInline {
                pre_class: Some("code-block"),
                italic: false,
                include_highlights: false,
                theme: None,
                highlight_lines: None,
                header: None,
            },
        },
    )
}
```

### tree-sitter Integration

#### Dependencies

```toml
[dependencies]
markdown = "1.0.0-alpha.21"
tree-sitter = "0.24"
tree-sitter-highlight = "0.24"
tree-sitter-rust = "0.23"  # Add grammars as needed
```

#### Direct Tree-sitter Highlighting

```rust
use tree_sitter_highlight::{HighlightConfiguration, Highlighter, HtmlRenderer};

fn highlight_rust_code(code: &str) -> Result<String, Box<dyn std::error::Error>> {
    let language = tree_sitter_rust::LANGUAGE.into();

    let mut config = HighlightConfiguration::new(
        language,
        "rust",
        tree_sitter_rust::HIGHLIGHTS_QUERY,
        "",  // injections query
        "",  // locals query
    )?;

    let highlight_names = &[
        "keyword", "function", "type", "variable",
        "string", "comment", "operator", "number", "property",
    ];

    config.configure(highlight_names);

    let mut highlighter = Highlighter::new();
    let highlights = highlighter.highlight(&config, code.as_bytes(), None, |_| None)?;

    let mut renderer = HtmlRenderer::new();
    renderer.render(highlights, code.as_bytes(), &|h| {
        let class = match h.0 {
            0 => "keyword", 1 => "function", 2 => "type",
            3 => "variable", 4 => "string", 5 => "comment",
            6 => "operator", 7 => "number", 8 => "property",
            _ => "unknown",
        };
        format!("<span class=\"{}\">", class).into_bytes()
    })?;

    Ok(format!("<pre><code class=\"language-rust\">{}</code></pre>",
        String::from_utf8(renderer.html)?))
}
```

### CSS for Highlighted Code

```css
/* Base code block styles */
pre.highlight {
    background: #282c34;
    padding: 1rem;
    border-radius: 4px;
    overflow-x: auto;
}

pre.highlight code {
    color: #abb2bf;
    font-family: 'Fira Code', monospace;
}

/* Syntect class-based highlighting */
.keyword { color: #c678dd; }
.function { color: #61afef; }
.type { color: #e5c07b; }
.string { color: #98c379; }
.comment { color: #5c6370; font-style: italic; }
.number { color: #d19a66; }
.operator { color: #56b6c2; }
```

### Highlighting Recommendations

| Use Case | Recommended |
|----------|-------------|
| Quick integration, many languages | **syntect** |
| Modern theming (Neovim themes) | **autumnus** |
| Maximum accuracy, semantic highlighting | **tree-sitter** |
| Static site generation | **syntect** (best performance) |
| IDE-like experience | **tree-sitter** |

## Axum Integration

Integrate markdown-rs into Axum web applications for rendering Markdown content, building APIs, and creating documentation systems.

### Dependencies

```toml
[dependencies]
axum = "0.8"
tokio = { version = "1", features = ["full"] }
markdown = "1.0.0-alpha.21"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

### Simple Handler

```rust
use axum::{extract::Json, response::Html};
use markdown::to_html;
use serde::Deserialize;

#[derive(Deserialize)]
struct MarkdownRequest {
    content: String,
}

async fn render_markdown(Json(payload): Json<MarkdownRequest>) -> Html<String> {
    Html(to_html(&payload.content))
}

let app = axum::Router::new()
    .route("/api/render", axum::routing::post(render_markdown));
```

### With GFM Options

```rust
use axum::{extract::Json, response::Html, http::StatusCode};
use markdown::{to_html_with_options, Options};
use serde::Deserialize;

#[derive(Deserialize)]
struct MarkdownRequest {
    content: String,
    gfm: Option<bool>,
}

async fn render_markdown(
    Json(payload): Json<MarkdownRequest>,
) -> Result<Html<String>, (StatusCode, String)> {
    let options = if payload.gfm.unwrap_or(false) {
        Options::gfm()
    } else {
        Options::default()
    };

    to_html_with_options(&payload.content, &options)
        .map(Html)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.reason.to_string()))
}
```

### Stateful Service Pattern

Share pre-configured options across handlers:

```rust
use axum::{extract::{Json, State}, response::Html, Router};
use markdown::{to_html_with_options, Options};
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    md_options: Arc<Options>,
}

async fn render_with_state(
    State(state): State<AppState>,
    Json(payload): Json<MarkdownRequest>,
) -> Html<String> {
    let html = to_html_with_options(&payload.content, &state.md_options)
        .unwrap_or_else(|e| format!("<p>Error: {}</p>", e.reason));
    Html(html)
}

#[tokio::main]
async fn main() {
    let state = AppState {
        md_options: Arc::new(Options::gfm()),
    };

    let app = Router::new()
        .route("/api/render", axum::routing::post(render_with_state))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

### Document Caching

```rust
use axum::{extract::{Path, State}, response::Html, Router};
use markdown::{to_html_with_options, Options};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
struct CachedDoc {
    source: String,
    html: String,
}

#[derive(Clone)]
struct AppState {
    cache: Arc<RwLock<HashMap<String, CachedDoc>>>,
    options: Arc<Options>,
}

async fn get_document(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Html<String>, axum::http::StatusCode> {
    // Check cache first
    {
        let cache = state.cache.read().await;
        if let Some(doc) = cache.get(&id) {
            return Ok(Html(doc.html.clone()));
        }
    }

    // Load from storage
    let source = load_document(&id).await
        .map_err(|_| axum::http::StatusCode::NOT_FOUND)?;

    // Parse and cache
    let html = to_html_with_options(&source, &state.options)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    {
        let mut cache = state.cache.write().await;
        cache.insert(id, CachedDoc { source, html: html.clone() });
    }

    Ok(Html(html))
}

async fn load_document(id: &str) -> Result<String, std::io::Error> {
    tokio::fs::read_to_string(format!("docs/{}.md", id)).await
}
```

### JSON Response with Metadata

```rust
use axum::{extract::Json, response::IntoResponse};
use markdown::{to_mdast, to_html_with_options, Options, ParseOptions, mdast::{Node, Heading, Text}};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct RenderRequest {
    content: String,
}

#[derive(Serialize)]
struct RenderResponse {
    html: String,
    title: Option<String>,
    headings: Vec<HeadingInfo>,
    word_count: usize,
}

#[derive(Serialize)]
struct HeadingInfo {
    depth: u8,
    text: String,
}

async fn render_with_metadata(
    Json(payload): Json<RenderRequest>,
) -> Json<RenderResponse> {
    let options = Options::gfm();
    let html = to_html_with_options(&payload.content, &options)
        .unwrap_or_default();

    let ast = to_mdast(&payload.content, &ParseOptions::gfm()).ok();

    let (title, headings) = ast.as_ref()
        .map(|a| extract_headings(a))
        .unwrap_or_default();

    let word_count = payload.content.split_whitespace().count();

    Json(RenderResponse { html, title, headings, word_count })
}

fn extract_headings(ast: &Node) -> (Option<String>, Vec<HeadingInfo>) {
    let mut title = None;
    let mut headings = Vec::new();

    fn walk(node: &Node, title: &mut Option<String>, headings: &mut Vec<HeadingInfo>) {
        if let Node::Heading(Heading { depth, children, .. }) = node {
            let text: String = children.iter()
                .filter_map(|c| if let Node::Text(Text { value, .. }) = c { Some(value.as_str()) } else { None })
                .collect();

            if *depth == 1 && title.is_none() {
                *title = Some(text.clone());
            }
            headings.push(HeadingInfo { depth: *depth, text });
        }
        if let Some(children) = node.children() {
            for child in children {
                walk(child, title, headings);
            }
        }
    }

    walk(ast, &mut title, &mut headings);
    (title, headings)
}
```

### Error Handling

```rust
use axum::{http::StatusCode, response::{IntoResponse, Response}};
use markdown::message::Message;

enum MarkdownError {
    ParseError(Message),
    NotFound,
    Internal(String),
}

impl IntoResponse for MarkdownError {
    fn into_response(self) -> Response {
        match self {
            MarkdownError::ParseError(msg) => {
                let body = format!(
                    "Markdown parse error at line {}, column {}: {}",
                    msg.point.as_ref().map(|p| p.line).unwrap_or(0),
                    msg.point.as_ref().map(|p| p.column).unwrap_or(0),
                    msg.reason
                );
                (StatusCode::BAD_REQUEST, body).into_response()
            }
            MarkdownError::NotFound => StatusCode::NOT_FOUND.into_response(),
            MarkdownError::Internal(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response()
            }
        }
    }
}
```

## LSP Integration

Build Language Server Protocol implementations for Markdown editing using markdown-rs for parsing and analysis.

### Architecture Overview

```txt
┌─────────────────────────────────────────────────────────┐
│  Editor/IDE                                             │
│  (VS Code, Neovim, Emacs, etc.)                        │
└─────────────────────┬───────────────────────────────────┘
                      │ LSP Messages
┌─────────────────────▼───────────────────────────────────┐
│  LSP Server                                             │
│  ┌────────────────┐  ┌────────────────┐                │
│  │ Text Documents │  │ markdown-rs    │                │
│  │ Synchronization│  │ Parser         │                │
│  └────────┬───────┘  └────────┬───────┘                │
│           │                   │                         │
│  ┌────────▼───────────────────▼───────┐                │
│  │       Document Analysis            │                │
│  │  - Diagnostics                     │                │
│  │  - Completions                     │                │
│  │  - Symbols                         │                │
│  │  - Hover                           │                │
│  └────────────────────────────────────┘                │
└─────────────────────────────────────────────────────────┘
```

### Dependencies

```toml
[dependencies]
tower-lsp = "0.20"
tokio = { version = "1", features = ["full"] }
markdown = "1.0.0-alpha.21"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

### Basic LSP Server Structure

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use markdown::{to_mdast, ParseOptions, mdast::Node};

#[derive(Debug)]
struct MarkdownDocument {
    content: String,
    ast: Node,
    version: i32,
}

struct MarkdownLanguageServer {
    client: Client,
    documents: Arc<RwLock<HashMap<Url, MarkdownDocument>>>,
}

impl MarkdownLanguageServer {
    fn new(client: Client) -> Self {
        Self {
            client,
            documents: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn parse_document(&self, uri: &Url, content: &str, version: i32) {
        let options = ParseOptions::gfm();
        if let Ok(ast) = to_mdast(content, &options) {
            let doc = MarkdownDocument {
                content: content.to_string(),
                ast,
                version,
            };
            self.documents.write().await.insert(uri.clone(), doc);
        }
    }
}
```

### Text Document Synchronization

```rust
#[tower_lsp::async_trait]
impl LanguageServer for MarkdownLanguageServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions::default()),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let content = params.text_document.text;
        let version = params.text_document.version;

        self.parse_document(&uri, &content, version).await;
        self.publish_diagnostics(&uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let version = params.text_document.version;

        if let Some(change) = params.content_changes.into_iter().last() {
            self.parse_document(&uri, &change.text, version).await;
            self.publish_diagnostics(&uri).await;
        }
    }
}
```

### Document Symbols (Outline)

```rust
use markdown::mdast::{Heading, Text};

impl MarkdownLanguageServer {
    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let docs = self.documents.read().await;
        let doc = match docs.get(&params.text_document.uri) {
            Some(d) => d,
            None => return Ok(None),
        };

        let symbols = self.extract_symbols(&doc.ast, &doc.content);
        Ok(Some(DocumentSymbolResponse::Nested(symbols)))
    }

    fn walk_for_symbols(&self, node: &Node, content: &str, symbols: &mut Vec<DocumentSymbol>) {
        match node {
            Node::Heading(Heading { depth, children, position, .. }) => {
                let text = self.extract_text(children);
                let range = self.position_to_range(position.as_ref(), content);

                #[allow(deprecated)]
                symbols.push(DocumentSymbol {
                    name: text,
                    detail: Some(format!("H{}", depth)),
                    kind: SymbolKind::STRING,
                    tags: None,
                    deprecated: None,
                    range,
                    selection_range: range,
                    children: None,
                });
            }
            _ => {}
        }

        if let Some(children) = node.children() {
            for child in children {
                self.walk_for_symbols(child, content, symbols);
            }
        }
    }

    fn position_to_range(&self, pos: Option<&markdown::unist::Position>, _content: &str) -> Range {
        match pos {
            Some(p) => Range {
                start: Position {
                    line: (p.start.line - 1) as u32,
                    character: (p.start.column - 1) as u32,
                },
                end: Position {
                    line: (p.end.line - 1) as u32,
                    character: (p.end.column - 1) as u32,
                },
            },
            None => Range::default(),
        }
    }
}
```

### Diagnostics

```rust
use markdown::mdast::{Link, Image};

impl MarkdownLanguageServer {
    async fn publish_diagnostics(&self, uri: &Url) {
        let docs = self.documents.read().await;
        let doc = match docs.get(uri) {
            Some(d) => d,
            None => return,
        };

        let diagnostics = self.generate_diagnostics(&doc.ast, &doc.content);

        self.client
            .publish_diagnostics(uri.clone(), diagnostics, Some(doc.version))
            .await;
    }

    fn walk_for_diagnostics(&self, node: &Node, content: &str, diagnostics: &mut Vec<Diagnostic>) {
        match node {
            Node::Link(Link { url, position, .. }) => {
                if url.is_empty() {
                    diagnostics.push(Diagnostic {
                        range: self.position_to_range(position.as_ref(), content),
                        severity: Some(DiagnosticSeverity::ERROR),
                        message: "Empty link URL".to_string(),
                        source: Some("markdown-lsp".to_string()),
                        ..Default::default()
                    });
                } else if !self.is_valid_url(url) {
                    diagnostics.push(Diagnostic {
                        range: self.position_to_range(position.as_ref(), content),
                        severity: Some(DiagnosticSeverity::WARNING),
                        message: format!("Possibly invalid URL: {}", url),
                        source: Some("markdown-lsp".to_string()),
                        ..Default::default()
                    });
                }
            }
            Node::Image(Image { alt, position, .. }) => {
                if alt.is_empty() {
                    diagnostics.push(Diagnostic {
                        range: self.position_to_range(position.as_ref(), content),
                        severity: Some(DiagnosticSeverity::INFORMATION),
                        message: "Image missing alt text".to_string(),
                        source: Some("markdown-lsp".to_string()),
                        ..Default::default()
                    });
                }
            }
            _ => {}
        }

        if let Some(children) = node.children() {
            for child in children {
                self.walk_for_diagnostics(child, content, diagnostics);
            }
        }
    }

    fn is_valid_url(&self, url: &str) -> bool {
        url.starts_with("http://")
            || url.starts_with("https://")
            || url.starts_with("mailto:")
            || url.starts_with("#")
            || url.starts_with("/")
            || url.starts_with("./")
            || url.starts_with("../")
    }
}
```

### Running the Server

```rust
#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(MarkdownLanguageServer::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}
```

### Performance Considerations

1. **Cache parsed ASTs** - avoid reparsing on every keystroke
2. **Debounce diagnostics** - don't publish on every change
3. **Use incremental parsing** if document changes are small
4. **Limit diagnostic scope** - only re-analyze changed sections

```rust
use std::time::Duration;
use tokio::time::sleep;

impl MarkdownLanguageServer {
    async fn debounced_publish_diagnostics(&self, uri: &Url) {
        sleep(Duration::from_millis(300)).await;
        self.publish_diagnostics(uri).await;
    }
}
```

## markdown-rs vs pulldown-cmark

A comparison of Rust's two main Markdown parsers.

### Philosophy

| Aspect | markdown-rs | pulldown-cmark |
|--------|-------------|----------------|
| **Design** | Spec-obsessed, AST-centric | Fast, streaming, minimal |
| **Model** | State machine -> tokens -> AST/HTML | Pull parser (iterator of events) |
| **Primary goal** | 100% CommonMark + GFM + MDX compliance | Speed with minimal allocations |
| **Safety default** | Safe HTML by default | Raw HTML passed through |

### Architecture Comparison

#### markdown-rs Flow

```txt
Markdown Input
     ↓
State Machine Tokenizer
     ↓
Token Stream (with positions)
     ↓
Event Stream
     ↓
┌────────────┬─────────────┐
│ to_html()  │ to_mdast()  │
│ HTML out   │ Full AST    │
└────────────┴─────────────┘
```

#### pulldown-cmark Flow

```txt
Markdown Input
     ↓
Pull Parser
     ↓
Iterator<Item = Event>
     ↓
┌─────────────────┬────────────────┐
│ html::push_html │ Custom handler │
│ HTML output     │ (you decide)   │
└─────────────────┴────────────────┘
```

### Feature Comparison

| Feature | markdown-rs | pulldown-cmark |
|---------|-------------|----------------|
| CommonMark compliance | 100% | High |
| GFM tables | Yes | Yes |
| GFM strikethrough | Yes | Yes |
| GFM task lists | Yes | Yes |
| GFM autolinks | Yes | No |
| Footnotes | Yes | Yes |
| MDX/JSX | Yes | No |
| Math notation | Yes | Yes |
| Frontmatter | Yes | Via extension |
| Wikilinks | No | Yes |
| Heading attributes | No | Yes |
| `#![no_std]` | Yes | No |
| Built-in AST | Yes (mdast) | No (roll your own) |
| Safe HTML by default | Yes | No |

### Performance

#### Memory

- **pulldown-cmark**: Lower memory - streaming events, no full tree materialization
- **markdown-rs**: Higher memory - builds complete AST with position info

#### Throughput

- **pulldown-cmark**: Generally faster for pure markdown->HTML (designed as "bare minimum of allocation and copying")
- **markdown-rs**: More overhead but provides richer output

#### Benchmark Context

pulldown-cmark is optimized for throughput with:

- Minimal allocations
- Optional SIMD acceleration
- Zero-copy string handling where possible
- Used in rustdoc, docs.rs - proven at scale

markdown-rs prioritizes:

- Correctness (650+ CommonMark tests, 1000+ additional, fuzzing)
- Rich positional information
- Complete AST for manipulation
- 100% code coverage

**Rule of thumb**: For pure "bytes -> events -> HTML" throughput, pulldown-cmark wins. For feature-rich correctness and AST manipulation, markdown-rs is worth the extra cycles.

### Decision Matrix

| Use Case | Recommendation | Rationale |
|----------|----------------|-----------|
| Static site generator | pulldown-cmark | Speed for processing many docs |
| Documentation system | pulldown-cmark | Rustdoc compatibility, battle-tested |
| CMS with user content | markdown-rs | Safe HTML by default |
| MDX processing | markdown-rs | Only Rust option with full MDX |
| Document analysis/linting | markdown-rs | First-class AST with positions |
| Live preview editor | pulldown-cmark | Streaming events for incremental updates |
| Academic writing (math) | Either | Both support LaTeX-style math |
| WASM/no_std | markdown-rs | `#![no_std]` + alloc support |
| JS ecosystem integration | markdown-rs | Matches micromark/mdast semantics |
| High-throughput API | pulldown-cmark | Minimal allocations, proven at scale |
| Cross-language tooling | markdown-rs | Same AST format as unified ecosystem |

### Migration Examples

#### pulldown-cmark -> markdown-rs

```rust
// Before: pulldown-cmark
use pulldown_cmark::{Parser, Options, html};

let mut opts = Options::empty();
opts.insert(Options::ENABLE_TABLES);
opts.insert(Options::ENABLE_STRIKETHROUGH);

let parser = Parser::new_ext(input, opts);
let mut html = String::new();
html::push_html(&mut html, parser);

// After: markdown-rs
use markdown::{to_html_with_options, Options};

let html = to_html_with_options(input, &Options::gfm())?;
```

#### markdown-rs -> pulldown-cmark

```rust
// Before: markdown-rs
use markdown::to_html;

let html = to_html(input);

// After: pulldown-cmark
use pulldown_cmark::{Parser, html};

let parser = Parser::new(input);
let mut html = String::new();
html::push_html(&mut html, parser);

// Note: You may need to add HTML sanitization
// if processing untrusted input
```

### Combining Both

For some projects, using both makes sense:

```rust
// Use pulldown-cmark for fast rendering
fn render_trusted_docs(input: &str) -> String {
    let parser = pulldown_cmark::Parser::new(input);
    let mut html = String::new();
    pulldown_cmark::html::push_html(&mut html, parser);
    html
}

// Use markdown-rs for analysis or untrusted content
fn analyze_document(input: &str) -> DocumentStats {
    let ast = markdown::to_mdast(input, &markdown::ParseOptions::gfm()).unwrap();
    extract_stats(&ast)
}

fn render_user_content(input: &str) -> String {
    // Safe by default
    markdown::to_html(input)
}
```

## Best Practices Summary

1. **Use default options for untrusted input** - safe HTML by default
2. **Use `to_mdast` for document analysis** - full AST access
3. **Use `to_html` for simple rendering** - fastest path
4. **Enable only needed constructs** - better performance
5. **Pair with a sanitizer if allowing raw HTML** - defense in depth
6. **Cache parsed ASTs** for repeated access to same document
7. **Use `Options::gfm()` for full GitHub compatibility**
8. **Use strongly-typed structs with serde** for frontmatter metadata
9. **Cache parsed HTML** for documents that don't change frequently
10. **Use `Arc<Options>`** to share configuration without cloning

## References

- [markdown-rs Documentation](https://docs.rs/markdown)
- [markdown-rs GitHub](https://github.com/wooorm/markdown-rs)
- [CommonMark Specification](https://spec.commonmark.org/)
- [GFM Specification](https://github.github.com/gfm/)
- [mdast Specification](https://github.com/syntax-tree/mdast)
