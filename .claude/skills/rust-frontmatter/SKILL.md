---
name: rust-frontmatter
description: Expert knowledge for parsing frontmatter metadata in Rust Markdown files. Use when extracting YAML, TOML, or JSON frontmatter from Markdown, building static site generators, documentation tools, or blog engines. Covers markdown-frontmatter, gray_matter, yaml_front_matter, fronma, and markdown-rs crates.
hash: ca115e2861ab1d8d
---

# Rust Frontmatter Parsing

Parse YAML, TOML, or JSON metadata from the top of Markdown files using type-safe Rust crates with serde integration.

## Quick Decision Guide

| Need | Recommended Crate |
|------|-------------------|
| Type-safe multi-format parsing | `markdown-frontmatter` |
| Custom delimiters or excerpts | `gray_matter` |
| YAML-only, minimal deps | `yaml_front_matter` |
| Full Markdown + frontmatter | `markdown-rs` |
| Lightweight, feature-gated | `fronma` |

## Core Principles

- Enable only needed format features to minimize dependencies
- Use `Option<T>` for optional frontmatter fields
- Validate frontmatter early at load time, not render time
- Keep frontmatter minimal - metadata only, not content
- All crates integrate with serde for type-safe deserialization
- Handle missing frontmatter gracefully (most crates treat as empty)

## Frontmatter Delimiters

- **YAML**: `---` (three dashes)
- **TOML**: `+++` (three plus signs)
- **JSON**: `{` and `}` (curly braces)

## Quick Start - markdown-frontmatter

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

let doc = r#"---
title: My Post
date: 2025-01-15
tags: [rust, markdown]
---
# Content starts here
"#;

let (frontmatter, body) = markdown_frontmatter::parse::<Frontmatter>(doc)?;
```

## Quick Start - gray_matter (Custom Delimiters)

```rust
use gray_matter::{Matter, engine::YAML};
use serde::Deserialize;

#[derive(Deserialize)]
struct Frontmatter { title: String }

let mut matter = Matter::<YAML>::new();
matter.delimiter = "~~~".to_string();  // Custom delimiter
matter.excerpt_delimiter = Some("<!-- more -->".to_string());  // Excerpt support

let result = matter.parse(doc);
let fm: Frontmatter = result.data.unwrap().deserialize()?;
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

# Lightweight alternative
fronma = { version = "0.2", features = ["yaml"] }
```

## Topics

- [Crate Comparison](./crate-comparison.md) - Detailed pros/cons and feature analysis
- [Code Examples](./examples.md) - Working examples for each library

## Resources

- [markdown-frontmatter on crates.io](https://crates.io/crates/markdown-frontmatter)
- [gray_matter on crates.io](https://crates.io/crates/gray_matter)
- [yaml_front_matter on crates.io](https://crates.io/crates/yaml_front_matter)
- [fronma on crates.io](https://crates.io/crates/fronma)
- [markdown-rs on crates.io](https://crates.io/crates/markdown)
