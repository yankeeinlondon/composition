# Rust Frontmatter Code Examples

Working code examples for each frontmatter parsing library.

## markdown-frontmatter Examples

### Basic YAML Parsing

```toml
# Cargo.toml
[dependencies]
markdown-frontmatter = { version = "0.2", features = ["yaml"] }
serde = { version = "1", features = ["derive"] }
```

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct BlogPost {
    title: String,
    date: String,
    author: Option<String>,
    tags: Option<Vec<String>>,
    draft: Option<bool>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let markdown = r#"---
title: Getting Started with Rust
date: 2025-01-15
author: Ken Snyder
tags: [rust, tutorial, beginners]
draft: false
---

# Introduction

Welcome to this Rust tutorial...
"#;

    let (frontmatter, body) = markdown_frontmatter::parse::<BlogPost>(markdown)?;

    println!("Title: {}", frontmatter.title);
    println!("Date: {}", frontmatter.date);
    println!("Author: {:?}", frontmatter.author);
    println!("Tags: {:?}", frontmatter.tags);
    println!("Body length: {} chars", body.len());

    Ok(())
}
```

### TOML Frontmatter

```toml
# Cargo.toml
[dependencies]
markdown-frontmatter = { version = "0.2", features = ["toml"] }
serde = { version = "1", features = ["derive"] }
```

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Config {
    title: String,
    weight: i32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let doc = r#"+++
title = "Page Title"
weight = 10
+++

Content here...
"#;

    let (config, body) = markdown_frontmatter::parse::<Config>(doc)?;
    println!("Title: {}, Weight: {}", config.title, config.weight);
    Ok(())
}
```

### Handling Optional Frontmatter

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
struct Metadata {
    #[serde(default)]
    title: String,
    #[serde(default)]
    description: Option<String>,
}

fn parse_with_fallback(content: &str) -> (Metadata, &str) {
    match markdown_frontmatter::parse::<Metadata>(content) {
        Ok((fm, body)) => (fm, body),
        Err(_) => (Metadata::default(), content),
    }
}
```

---

## gray_matter Examples

### Basic Usage with Custom Delimiters

```toml
# Cargo.toml
[dependencies]
gray_matter = "0.2"
serde = { version = "1", features = ["derive"] }
```

```rust
use gray_matter::{Matter, engine::YAML};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Frontmatter {
    title: String,
    category: Option<String>,
}

fn main() {
    let mut matter = Matter::<YAML>::new();

    // Optional: set custom delimiters
    matter.delimiter = "~~~".to_string();

    let doc = r#"~~~
title: Custom Delimiter Example
category: tutorials
~~~

Content with custom delimiters.
"#;

    let result = matter.parse(doc);

    if let Some(data) = result.data {
        let fm: Frontmatter = data.deserialize().unwrap();
        println!("Title: {}", fm.title);
    }

    println!("Content: {}", result.content);
}
```

### Excerpt Extraction

```rust
use gray_matter::{Matter, engine::YAML};

fn main() {
    let mut matter = Matter::<YAML>::new();
    matter.excerpt_delimiter = Some("<!-- more -->".to_string());

    let doc = r#"---
title: Blog Post with Excerpt
---

This is the excerpt that appears in listings.

<!-- more -->

This is the full content that only appears on the detail page.
"#;

    let result = matter.parse(doc);

    println!("Excerpt: {:?}", result.excerpt);
    println!("Full content: {}", result.content);
}
```

### Using TOML Engine

```rust
use gray_matter::{Matter, engine::TOML};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Config {
    title: String,
    weight: i32,
}

fn main() {
    let matter = Matter::<TOML>::new();

    let doc = r#"+++
title = "TOML Example"
weight = 5
+++

Content here.
"#;

    let result = matter.parse(doc);
    let config: Config = result.data.unwrap().deserialize().unwrap();
    println!("Config: {:?}", config);
}
```

---

## yaml_front_matter Examples

### Simple YAML Parsing

```toml
# Cargo.toml
[dependencies]
yaml_front_matter = "0.1"
serde = { version = "1", features = ["derive"] }
```

```rust
use yaml_front_matter::YamlFrontMatter;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Metadata {
    title: String,
    slug: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let doc = r#"---
title: Simple Example
slug: simple-example
---

# Heading

Paragraph content.
"#;

    let document = YamlFrontMatter::parse::<Metadata>(doc)?;

    println!("Title: {}", document.metadata.title);
    println!("Slug: {:?}", document.metadata.slug);
    println!("Content: {}", document.content);

    Ok(())
}
```

---

## fronma Examples

### Multi-Format Support

```toml
# Cargo.toml
[dependencies]
fronma = { version = "0.2", features = ["yaml", "toml"] }
serde = { version = "1", features = ["derive"] }
```

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct PageMeta {
    title: String,
    order: Option<i32>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // YAML frontmatter
    let yaml_doc = r#"---
title: YAML Page
order: 1
---
Content"#;

    let result = fronma::parser::parse::<PageMeta>(yaml_doc)?;
    println!("YAML: {:?}", result.headers);

    // TOML frontmatter
    let toml_doc = r#"+++
title = "TOML Page"
order = 2
+++
Content"#;

    let result = fronma::parser::parse::<PageMeta>(toml_doc)?;
    println!("TOML: {:?}", result.headers);

    Ok(())
}
```

---

## Complete Static Site Generator Example

```rust
use std::fs;
use std::path::Path;
use serde::Deserialize;
use walkdir::WalkDir;

#[derive(Debug, Deserialize)]
struct PageFrontmatter {
    title: String,
    date: Option<String>,
    template: Option<String>,
    draft: Option<bool>,
    #[serde(default)]
    tags: Vec<String>,
}

struct Page {
    frontmatter: PageFrontmatter,
    content: String,
    path: String,
}

fn load_pages(content_dir: &Path) -> Vec<Page> {
    let mut pages = Vec::new();

    for entry in WalkDir::new(content_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "md"))
    {
        let content = fs::read_to_string(entry.path()).unwrap();

        match markdown_frontmatter::parse::<PageFrontmatter>(&content) {
            Ok((frontmatter, body)) => {
                // Skip drafts in production
                if frontmatter.draft.unwrap_or(false) {
                    continue;
                }

                pages.push(Page {
                    frontmatter,
                    content: body.to_string(),
                    path: entry.path().display().to_string(),
                });
            }
            Err(e) => {
                eprintln!("Failed to parse {}: {}", entry.path().display(), e);
            }
        }
    }

    // Sort by date (newest first)
    pages.sort_by(|a, b| {
        b.frontmatter.date.cmp(&a.frontmatter.date)
    });

    pages
}

fn main() {
    let pages = load_pages(Path::new("content"));

    for page in &pages {
        println!("- {} ({})",
            page.frontmatter.title,
            page.frontmatter.date.as_deref().unwrap_or("no date")
        );
    }
}
```

---

## Error Handling Patterns

### Graceful Fallback

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
struct OptionalMeta {
    #[serde(default)]
    title: Option<String>,
}

fn parse_markdown(content: &str) -> (Option<String>, String) {
    match markdown_frontmatter::parse::<OptionalMeta>(content) {
        Ok((meta, body)) => (meta.title, body.to_string()),
        Err(_) => (None, content.to_string()),
    }
}
```

### Strict Validation

```rust
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Deserialize)]
struct RequiredMeta {
    title: String,
    date: String,
}

#[derive(Error, Debug)]
enum ParseError {
    #[error("Invalid frontmatter: {0}")]
    Frontmatter(String),
    #[error("Missing required field: {0}")]
    MissingField(&'static str),
}

fn parse_strict(content: &str) -> Result<(RequiredMeta, String), ParseError> {
    let (meta, body) = markdown_frontmatter::parse::<RequiredMeta>(content)
        .map_err(|e| ParseError::Frontmatter(e.to_string()))?;

    if meta.title.is_empty() {
        return Err(ParseError::MissingField("title"));
    }

    Ok((meta, body.to_string()))
}
```
