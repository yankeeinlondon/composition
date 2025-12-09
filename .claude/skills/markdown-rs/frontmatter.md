# Frontmatter with markdown-rs

Frontmatter allows embedding metadata (YAML, TOML, or JSON) at the beginning of Markdown documents. markdown-rs supports parsing frontmatter as part of the AST.

## Enabling Frontmatter

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

## Frontmatter Formats

### YAML Frontmatter

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

### TOML Frontmatter

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

### JSON Frontmatter (as code fence)

```markdown
```json
{
  "title": "My Document",
  "author": "John Doe"
}
```

# Document Content
```

Using standard fenced code block with `json` language.

## Extracting Frontmatter

### From AST

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

// Usage
let input = r#"---
title: My Post
author: Jane Doe
tags:
  - rust
  - web
---

# Content here
"#;

if let Some(meta) = parse_yaml_frontmatter(input) {
    println!("Title: {}", meta.title);
    println!("Author: {:?}", meta.author);
}
```

### Parsing TOML Frontmatter

```rust
use markdown::{to_mdast, ParseOptions, Constructs};
use markdown::mdast::{Node, Toml as TomlNode};
use serde::Deserialize;

#[derive(Deserialize)]
struct Config {
    title: String,
    draft: Option<bool>,
    weight: Option<i32>,
}

fn parse_toml_frontmatter(input: &str) -> Option<Config> {
    let options = ParseOptions {
        constructs: Constructs {
            frontmatter: true,
            ..Constructs::default()
        },
        ..ParseOptions::default()
    };

    let ast = to_mdast(input, &options).ok()?;

    if let Node::Root(root) = &ast {
        if let Some(Node::Toml(TomlNode { value, .. })) = root.children.first() {
            return toml::from_str(value).ok();
        }
    }

    None
}
```

## Generic Frontmatter Handler

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

## Separating Frontmatter from Content

```rust
use markdown::{to_mdast, to_html_with_options, ParseOptions, Options, Constructs};
use markdown::mdast::{Node, Yaml, Toml};

struct Document {
    frontmatter: Option<String>,
    frontmatter_format: Option<&'static str>,
    html: String,
}

fn parse_document(input: &str) -> Document {
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

    // Extract frontmatter from AST
    let (frontmatter, format) = match to_mdast(input, &parse_options) {
        Ok(ast) => {
            if let Node::Root(root) = &ast {
                match root.children.first() {
                    Some(Node::Yaml(Yaml { value, .. })) => (Some(value.clone()), Some("yaml")),
                    Some(Node::Toml(Toml { value, .. })) => (Some(value.clone()), Some("toml")),
                    _ => (None, None),
                }
            } else {
                (None, None)
            }
        }
        Err(_) => (None, None),
    };

    // Generate HTML (frontmatter is not rendered)
    let html = to_html_with_options(input, &options)
        .unwrap_or_default();

    Document {
        frontmatter,
        frontmatter_format: format,
        html,
    }
}
```

## Static Site Generator Pattern

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

    // Skip draft pages in production
    if meta.draft.unwrap_or(false) {
        // Handle draft logic...
    }

    let content_html = to_html_with_options(&source, &options)?;

    Ok(Page {
        meta,
        content_html,
        source_path: path.display().to_string(),
    })
}
```

## AST Node Reference

| Node | Delimiters | Description |
|------|------------|-------------|
| `Node::Yaml` | `---` | YAML frontmatter |
| `Node::Toml` | `+++` | TOML frontmatter |

Both nodes have:
- `value: String` - raw frontmatter content (without delimiters)
- `position: Option<Position>` - source location

## Dependencies for Parsing

```toml
[dependencies]
markdown = "1.0.0-alpha.21"
serde = { version = "1.0", features = ["derive"] }
serde_yaml = "0.9"  # For YAML frontmatter
toml = "0.8"        # For TOML frontmatter
serde_json = "1.0"  # For JSON frontmatter
```

## Best Practices

1. **Always enable `frontmatter: true`** when expecting metadata
2. **Use strongly-typed structs** with serde for metadata
3. **Provide defaults** for optional fields with `#[serde(default)]`
4. **Validate required fields** after parsing
5. **Handle parse errors gracefully** - invalid YAML/TOML shouldn't crash
6. **Cache parsed metadata** for repeated access
