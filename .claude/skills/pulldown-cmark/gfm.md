# GitHub Flavored Markdown (GFM) Guide

## Overview

**GitHub Flavored Markdown (GFM)** is a **strict superset** of CommonMark - all CommonMark documents are valid GFM, but GFM adds extensions not in CommonMark. The formal specification (version 0.29-gfm, 2019-04-06) provides precise, unambiguous syntax definitions.

### Design Philosophy

Markdown's formatting syntax prioritizes **readability** - a Markdown document should be publishable as plain text without appearing marked up. GFM maintains this principle while adding practical extensions for GitHub's use cases.

## GFM Extensions vs CommonMark

| Feature | CommonMark | GFM | Description |
|:--------|:----------:|:---:|:------------|
| Tables | - | Yes | Pipe-based table syntax with alignment |
| Task Lists | - | Yes | Checkbox functionality in lists |
| Strikethrough | - | Yes | `~~text~~` for deleted content |
| Autolinks | Limited | Extended | Broader URL detection without angle brackets |
| Disallowed HTML | - | Yes | Security-focused HTML filtering |

## GFM Parsing Strategy

GFM uses a **two-phase parsing approach**:

### Phase 1: Block Structure
Identifies block-level elements (headings, paragraphs, lists, code blocks, tables) by scanning line-by-line. Establishes the document skeleton without processing inline formatting.

### Phase 2: Inline Structure
Processes inline elements within each block (emphasis, links, images, code spans, autolinks). Handles text formatting and link reference resolution.

```
Markdown Input → Block Structure → Inline Structure → Output (HTML/AST)
```

## Working with GFM in pulldown-cmark

### Enabling GFM Extensions

By default, pulldown-cmark only supports CommonMark. Enable GFM features explicitly:

```rust
use pulldown_cmark::{Parser, Options};

let mut options = Options::empty();
options.insert(Options::ENABLE_TABLES);
options.insert(Options::ENABLE_STRIKETHROUGH);
options.insert(Options::ENABLE_TASKLISTS);
options.insert(Options::ENABLE_GFM_AUTOLINKS);

let parser = Parser::new_ext(markdown, options);
```

**Important**: There is no single "enable all GFM" flag. Enable each extension individually.

### GFM Feature Mapping

| GFM Feature | `pulldown-cmark` Option | Notes |
|:------------|:------------------------|:------|
| Tables | `ENABLE_TABLES` | Basic table structure supported |
| Task Lists | `ENABLE_TASKLISTS` | Parses `- [ ]` and `- [x]` |
| Strikethrough | `ENABLE_STRIKETHROUGH` | `~~text~~` syntax |
| Autolinks | `ENABLE_GFM_AUTOLINKS` | Plain URL auto-linking |
| Footnotes | `ENABLE_FOOTNOTES` | Not strictly GFM, but commonly used |

### Common GFM Configuration

```rust
use pulldown_cmark::{Parser, Options, html};

// Typical GFM-like setup
let options = Options::ENABLE_TABLES
    | Options::ENABLE_STRIKETHROUGH
    | Options::ENABLE_TASKLISTS
    | Options::ENABLE_GFM_AUTOLINKS;

let markdown = r#"
## Task List
- [x] Write code
- [ ] Test features

## Table
| Header | Description |
|--------|-------------|
| Cell   | Content     |

This is ~~deleted~~ text.

Visit https://example.com for info.
"#;

let parser = Parser::new_ext(markdown, options);
let mut html_output = String::new();
html::push_html(&mut html_output, parser);
```

## Important Considerations

### Not a Perfect 1:1 Match

`pulldown-cmark` aims for high compliance but **may not reproduce GitHub's exact output**. GitHub applies additional post-processing. Your documents should follow CommonMark standards for predictable results.

### Feature Availability by Version

GFM-related features are added over time:
- Definition lists (`ENABLE_DEFINITION_LISTS`) added in v0.13.0
- Check [release notes](https://github.com/pulldown-cmark/pulldown-cmark/releases) for latest features

### Handling Consecutive Text Events

The parser may yield consecutive `Event::Text` chunks. For complete text processing:

```rust
use pulldown_cmark::{Parser, TextMergeStream};

let parser = Parser::new_ext(markdown, options);
let merged = TextMergeStream::new(parser);

for event in merged {
    // Text events are now merged
}
```

## Best Practices

### Performance
- **Enable only needed extensions** - each adds parsing overhead
- Write to buffered targets (`String`, `Vec<u8>`) not directly to stdout/files
- Use `TextMergeStream` only when needed (adds overhead)

### Security
- GFM's disallowed raw HTML provides some protection
- Consider additional sanitization for user-generated content
- Be cautious with raw HTML passthrough

### Compatibility
- Test edge cases against GitHub's actual renderer
- For maximum compatibility, ensure content works with both CommonMark and GFM parsers
- Document which GFM extensions your application requires

## Extended Ecosystem

### Frontmatter Support

While not part of GFM spec, YAML frontmatter is commonly used. Use `pulldown-cmark-frontmatter`:

```rust
use pulldown_cmark_frontmatter::FrontmatterExtractor;

let mut extractor = FrontmatterExtractor::new(Parser::new(markdown));
let mut html_output = String::new();
pulldown_cmark::html::push_html(&mut html_output, &mut extractor);

if let Some(frontmatter) = extractor.frontmatter {
    println!("Title: {:?}", frontmatter.title);
}
```

Or enable native metadata blocks:

```rust
let options = Options::ENABLE_YAML_STYLE_METADATA_BLOCKS;
let parser = Parser::new_ext(markdown, options);
```

### Table of Contents Generation

Use `pulldown-cmark-toc`:

```rust
use pulldown_cmark_toc::Toc;

let toc = Toc::new(markdown);
println!("{}", toc.to_html());
```

## GFM Syntax Quick Reference

### Tables

```markdown
| Left | Center | Right |
|:-----|:------:|------:|
| L    |   C    |     R |
```

Column alignment via colons in separator row.

### Task Lists

```markdown
- [x] Completed task
- [ ] Incomplete task
```

### Strikethrough

```markdown
~~deleted text~~
```

### Autolinks

```markdown
https://example.com
user@example.com
```

Automatically converted to links without `<>` wrappers.
