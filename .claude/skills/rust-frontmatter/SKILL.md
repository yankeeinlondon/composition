---
name: rust-frontmatter
description: Expert knowledge for parsing frontmatter metadata in Rust Markdown files. Use when extracting YAML, TOML, or JSON frontmatter from Markdown, building static site generators, documentation tools, or blog engines. Covers markdown-frontmatter, gray_matter, yaml_front_matter, and fronma crates.
---

# Rust Frontmatter Parsing

## Quick Decision Guide

| Need | Recommended Crate |
|------|-------------------|
| Type-safe multi-format parsing | `markdown-frontmatter` |
| Custom delimiters or excerpts | `gray_matter` |
| YAML-only, minimal deps | `yaml_front_matter` |
| Full Markdown + frontmatter | `markdown-rs` |
| Lightweight, feature-gated | `fronma` |
| Integration with markdown-it | `markdown-it-front-matter` |

## Crate Overview

| Crate | Formats | Key Feature | Best For |
|:------|:--------|:------------|:---------|
| **`markdown-frontmatter`** | YAML, TOML, JSON | Type-safe serde integration | General use |
| **`gray_matter`** | YAML, TOML, JSON | Custom delimiters, excerpts | Advanced needs |
| **`yaml_front_matter`** | YAML only | Simple, minimal | YAML workflows |
| **`markdown-rs`** | YAML, TOML | Full parser with extension | Complete Markdown parsing |
| **`fronma`** | YAML, TOML, JSON | Feature-gated, minimal | Lightweight projects |

## Key Concepts

**Frontmatter Delimiters:**
- YAML: `---` (three dashes)
- TOML: `+++` (three plus signs)
- JSON: `{` and `}` (curly braces)

**All crates use Serde** for deserialization into custom Rust structs.

## Detailed Documentation

- [Crate Comparison](./crate-comparison.md) - Detailed pros/cons and feature analysis
- [Code Examples](./examples.md) - Working examples for each library

## Quick Start

### markdown-frontmatter (Recommended)

```toml
[dependencies]
markdown-frontmatter = { version = "0.2", features = ["yaml"] }
serde = { version = "1", features = ["derive"] }
```

```rust
use serde::Deserialize;

#[derive(Deserialize)]
struct Frontmatter {
    title: String,
    date: Option<String>,
    tags: Option<Vec<String>>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let doc = r#"---
title: My Post
date: 2025-01-15
tags: [rust, markdown]
---
# Content starts here

The body of your document.
"#;

    let (frontmatter, body) = markdown_frontmatter::parse::<Frontmatter>(doc)?;
    println!("Title: {}", frontmatter.title);
    println!("Body: {}", body);
    Ok(())
}
```

### gray_matter (Custom Delimiters)

```toml
[dependencies]
gray_matter = "0.2"
serde = { version = "1", features = ["derive"] }
```

```rust
use gray_matter::{Matter, engine::YAML};
use serde::Deserialize;

#[derive(Deserialize)]
struct Frontmatter {
    title: String,
}

fn main() {
    let matter = Matter::<YAML>::new();
    let result = matter.parse(doc);
    let fm: Frontmatter = result.data.unwrap().deserialize().unwrap();
}
```

## Cargo.toml Patterns

```toml
# Multi-format support
markdown-frontmatter = { version = "0.2", features = ["yaml", "toml", "json"] }

# Custom delimiters and excerpts
gray_matter = "0.2"

# YAML only (minimal)
yaml_front_matter = "0.1"

# Full Markdown parsing with frontmatter
markdown = { version = "1", features = ["frontmatter"] }
```

## Best Practices

1. **Enable only needed formats** via Cargo features to minimize dependencies
2. **Use `Option<T>`** for optional frontmatter fields
3. **Handle missing frontmatter** - most crates treat it as empty or return an error
4. **Validate early** - parse frontmatter at load time, not render time
5. **Keep frontmatter minimal** - metadata only, not content
