---
name: gfm
description: Comprehensive guide to GitHub Flavored Markdown (GFM) and parsing with pulldown-cmark in Rust
created: 2025-12-08
hash: 38471b6f29eebed7
tags:
  - markdown
  - gfm
  - rust
  - pulldown-cmark
  - parsing
  - commonmark
---

# GitHub Flavored Markdown (GFM) and pulldown-cmark

GitHub Flavored Markdown (GFM) is the formal Markdown dialect used on GitHub.com and GitHub Enterprise. Defined as a **strict superset of CommonMark**, GFM extends the base specification with practical features like tables, task lists, and strikethrough text that have become standard across many platforms. In the Rust ecosystem, `pulldown-cmark` serves as the primary parser for working with GFM content, offering a performant, event-based approach to Markdown processing.

## Table of Contents

1. [Understanding GFM](#understanding-gfm)
2. [GFM Extensions Beyond CommonMark](#gfm-extensions-beyond-commonmark)
3. [GFM Parsing Strategy](#gfm-parsing-strategy)
4. [Introduction to pulldown-cmark](#introduction-to-pulldown-cmark)
5. [Enabling GFM Extensions in pulldown-cmark](#enabling-gfm-extensions-in-pulldown-cmark)
6. [Working with the Event Stream](#working-with-the-event-stream)
7. [Advanced Techniques](#advanced-techniques)
8. [Best Practices](#best-practices)
9. [Quick Reference](#quick-reference)
10. [Resources](#resources)

## Understanding GFM

### Relationship to CommonMark

GFM is built upon CommonMark, which was created to address the ambiguities and inconsistencies in the original Markdown description by John Gruber. While CommonMark provides a strongly defined, highly compatible specification of Markdown, GFM adds several popular extensions that were not part of the original standard but were needed for GitHub's use cases.

**Key principle**: All CommonMark documents are valid GFM documents, but not all GFM documents are valid CommonMark due to these extensions.

### Design Philosophy

The overriding design goal for Markdown's formatting syntax is to make it as **readable as possible**. A Markdown-formatted document should be publishable as-is, as plain text, without looking like it has been marked up with tags or formatting instructions. GFM maintains this principle even with its added extensions.

### Specification Version

The formal GFM specification (currently version 0.29-gfm, released 2019-04-06) is based on the CommonMark Spec by John MacFarlane and provides a precise, unambiguous definition of the syntax and semantics of this dialect.

## GFM Extensions Beyond CommonMark

GFM introduces several key extensions that enhance the base CommonMark functionality. These extensions address common needs while maintaining readability and compatibility.

### Tables

Tables allow for structured data presentation using a simple pipe-based syntax:

```markdown
| Header 1 | Header 2 | Header 3 |
|----------|----------|----------|
| Cell 1   | Cell 2   | Cell 3   |
| Cell 4   | Cell 5   | Cell 6   |
```

Tables are part of the GFM specification but not included in the base CommonMark spec.

### Task List Items

Task lists extend regular lists with checkbox functionality:

```markdown
- [x] Completed task
- [ ] Incomplete task
```

This feature is particularly useful for project management and issue tracking.

### Strikethrough Text

Strikethrough allows for text to be marked as deleted or no longer relevant:

```markdown
~~This text is struck through~~
```

This uses one or two tildes around the text to be struck through.

### Extended Autolinks

GFM extends autolinking to automatically detect and convert URLs without explicit markup:

```markdown
https://github.com
```

This will be automatically converted to a link without needing surrounding angle brackets.

### Disallowed Raw HTML

For security reasons, GFM restricts certain HTML tags that could be used for malicious purposes. This extension defines which HTML tags are allowed and which are disallowed in user content.

### Feature Comparison

| Feature | CommonMark | GFM | Description |
|---------|------------|-----|-------------|
| Tables | No | Yes | Pipe-based table syntax |
| Task Lists | No | Yes | Checkbox functionality in lists |
| Strikethrough | No | Yes | Tilde-wrapped text for deletion |
| Autolinks | Limited | Extended | Broader URL detection |
| HTML Restrictions | No | Yes | Security-focused HTML filtering |

## GFM Parsing Strategy

The GFM specification outlines a **two-phase parsing strategy** that efficiently converts Markdown to the desired output format.

### Phase 1: Block Structure

This phase identifies the block-level elements in the document, such as headings, paragraphs, lists, code blocks, and tables. The parser scans the document line by line, determining the type of each block and how blocks are nested within each other. This phase establishes the document skeleton without processing inline formatting.

### Phase 2: Inline Structure

Once the block structure is established, this phase processes the inline elements within each block, such as emphasis, links, images, code spans, and autolinks. This phase handles the more granular aspects of Markdown parsing, resolving text formatting and link references.

```
Markdown Input --> Block Structure Identification --> Inline Structure Processing --> Output (HTML/AST)
```

## Introduction to pulldown-cmark

**pulldown-cmark** is a pull parser for CommonMark written in Rust, designed with performance and flexibility in mind. It serves as the foundation for GFM parsing in Rust applications and provides a low-level event-based API that gives developers fine-grained control over the parsing process.

### Pull Parser Design

Unlike push parsers that generate a complete document tree immediately, pulldown-cmark produces a **stream of events** that can be processed incrementally. This approach offers several advantages:

- **Memory efficiency**: No need to hold the entire document tree in memory
- **Performance**: Parsing can begin before the entire document is available
- **Flexibility**: Developers can transform or filter the event stream as needed
- **Streaming support**: Well-suited for processing large documents or network streams

### Core Architecture

The crate is structured around several key components:

| Component | Purpose |
|-----------|---------|
| `Parser` | The main struct that implements parsing logic and generates events |
| `Event` | Enum representing different types of markdown elements encountered |
| `Tag` | Enum representing different types of block and inline elements |
| `Options` | Configuration for enabling extensions beyond CommonMark |

### Historical Context

Rust's documentation tool, `rustdoc`, switched to pulldown-cmark from an older parser. This caused minor breaking changes for some existing Markdown documents, highlighting that differences between Markdown parsers do exist. Documents should follow the CommonMark standard for the most predictable results.

## Enabling GFM Extensions in pulldown-cmark

By default, pulldown-cmark only supports the CommonMark standard. To use GFM features, you must explicitly enable them through the `Options` struct.

### Available Extension Flags

| GFM Feature | Option Flag | Notes |
|-------------|-------------|-------|
| Tables | `ENABLE_TABLES` | Basic table structure supported |
| Task Lists | `ENABLE_TASKLISTS` | Parses into appropriate HTML |
| Strikethrough | `ENABLE_STRIKETHROUGH` | Two tildes syntax |
| Autolinks | `ENABLE_AUTOLINK` | Part of CommonMark spec |
| Footnotes | `ENABLE_FOOTNOTES` | Numbered references and definitions |
| Smart Punctuation | `ENABLE_SMART_PUNCTUATION` | Automatic punctuation replacement |
| Definition Lists | `ENABLE_DEFINITION_LISTS` | Added in v0.13.0 |

> **Note:** pulldown-cmark does not have a single flag to enable "all GFM features." You must enable each extension you need. Some GFM features (like emoji shortcodes or certain table formatting nuances) are not natively supported and may require post-processing.

### Basic Usage Example

```rust
use pulldown_cmark::{Parser, Options, html};

let markdown_input = "
## Task List
- [x] Write code
- [ ] Test features

## Table
| Header | Description |
|--------|-------------|
| Cell   | Content     |
";

// Enable specific GFM extensions
let mut options = Options::empty();
options.insert(Options::ENABLE_TABLES);
options.insert(Options::ENABLE_TASKLISTS);
options.insert(Options::ENABLE_STRIKETHROUGH);

// Create parser with extended options
let parser = Parser::new_ext(markdown_input, options);

// Render to HTML
let mut html_output = String::new();
html::push_html(&mut html_output, parser);

println!("{}", html_output);
```

## Working with the Event Stream

The parser produces a stream of events that you can process directly or convert to HTML.

### Processing Events Directly

```rust
use pulldown_cmark::{Parser, Event};

let markdown_input = "Hello **world**!";
let parser = Parser::new(markdown_input);

for event in parser {
    match event {
        Event::Text(text) => println!("Text: {}", text),
        Event::Start(tag) => println!("Start: {:?}", tag),
        Event::End(tag) => println!("End: {:?}", tag),
        _ => ()
    }
}
```

### Converting to HTML

```rust
use pulldown_cmark::{Parser, html};

let markdown_input = "Hello **world**!";
let parser = Parser::new(markdown_input);

let mut html_output = String::new();
html::push_html(&mut html_output, parser);
```

### Handling Consecutive Text Events

The parser may yield consecutive `Event::Text` chunks due to how it evaluates the source. pulldown-cmark provides a `TextMergeStream` utility for smoother processing:

```rust
use pulldown_cmark::{Event, Parser, TextMergeStream};

let markdown_input = "Hello world, this is a ~~complicated~~ *very simple* example.";
let iterator = TextMergeStream::new(Parser::new(markdown_input));

for event in iterator {
    match event {
        Event::Text(text) => println!("{}", text),
        _ => {}
    }
}
```

## Advanced Techniques

### Custom Parsing and Transformation

Because pulldown-cmark provides an event stream, you can easily transform content during parsing:

```rust
use pulldown_cmark::{Parser, Event, Tag, Options};

let markdown_input = "![alt text](image.png \"title\")";
let mut options = Options::empty();
options.insert(Options::ENABLE_FOOTNOTES);
let parser = Parser::new_ext(markdown_input, options);

let transformed_events: Vec<Event> = parser
    .map(|event| match event {
        Event::Start(Tag::Image(..)) => Event::Start(Tag::Image(
            pulldown_cmark::LinkType::Inline,
            "modified.png".into(),
            "modified title".into(),
        )),
        other => other,
    })
    .collect();
```

### Working with Frontmatter

While not part of the GFM spec, frontmatter is commonly used in Markdown documents. The `pulldown-cmark-frontmatter` crate extends pulldown-cmark to handle frontmatter:

```rust
use pulldown_cmark_frontmatter::FrontmatterExtractor;

let markdown_input = include_str!("document-with-frontmatter.md");
let mut extractor = FrontmatterExtractor::new(pulldown_cmark::Parser::new(markdown_input));

let mut html_output = String::new();
pulldown_cmark::html::push_html(&mut html_output, &mut extractor);

let frontmatter = extractor.frontmatter.expect("frontmatter not detected");
println!("Title: {:?}", frontmatter.title);
```

### Generating Table of Contents

The `pulldown-cmark-toc` crate can generate a table of contents from a Markdown document:

```rust
use pulldown_cmark_toc::Toc;

let markdown_input = include_str!("document.md");
let toc = Toc::new(markdown_input);
println!("{}", toc.to_html());
```

## Best Practices

### Performance

- **Enable only what you need**: For performance, only insert the `Options` flags required by your Markdown content. Each enabled extension adds parsing overhead.
- **Use streaming for large documents**: Process the event stream incrementally rather than collecting all events first.
- **Use buffered targets**: When using the built-in HTML renderer, write to a buffered target like a `String` or `Vec<u8>` for better performance, as it performs many small writes.
- **Reuse parser configurations**: When parsing multiple documents, consider reusing parser configurations.

### Security

- **Sanitize HTML output**: Even with GFM's disallowed raw HTML, consider additional sanitization for user-generated content.
- **Handle broken links**: Implement proper broken link callbacks to avoid unexpected behavior.

### Compatibility

- **Test against GitHub's renderer**: Since GFM is defined by GitHub's implementation, test edge cases against actual GitHub rendering.
- **Consider CommonMark compliance**: For maximum compatibility, ensure your content works with both CommonMark and GFM parsers.
- **Follow CommonMark standard**: Your documents should follow the CommonMark standard for the most predictable results across different parsers.

### Version Awareness

New GFM-related features are added over time. For example, definition lists (`ENABLE_DEFINITION_LISTS`) were added in v0.13.0. Consult the release notes for the latest features.

## Quick Reference

### Enabling All Common GFM Features

```rust
use pulldown_cmark::{Parser, Options, html};

let mut options = Options::empty();
options.insert(Options::ENABLE_TABLES);
options.insert(Options::ENABLE_TASKLISTS);
options.insert(Options::ENABLE_STRIKETHROUGH);
options.insert(Options::ENABLE_FOOTNOTES);

let parser = Parser::new_ext(markdown_input, options);
let mut html_output = String::new();
html::push_html(&mut html_output, parser);
```

### GFM Syntax Quick Reference

| Element | Syntax |
|---------|--------|
| Table | `\| Header \| Header \|` with `\|---\|---\|` separator |
| Task (complete) | `- [x] Task text` |
| Task (incomplete) | `- [ ] Task text` |
| Strikethrough | `~~struck text~~` |
| Autolink | Just paste the URL: `https://example.com` |
| Footnote | `Text[^1]` with `[^1]: Footnote content` |

### Important Considerations

1. **Not a complete 1:1 match**: pulldown-cmark aims for high compliance, but it may not reproduce every detail of GitHub's exact output, which involves additional post-processing.
2. **Some features require post-processing**: Features like emoji shortcodes or certain table formatting nuances are not natively supported.

## Resources

- [pulldown-cmark GitHub Repository](https://github.com/pulldown-cmark/pulldown-cmark)
- [pulldown-cmark Release Notes](https://github.com/pulldown-cmark/pulldown-cmark/releases)
- [CommonMark Specification](https://spec.commonmark.org/)
- [GitHub Flavored Markdown Specification](https://github.github.com/gfm/)
- [pulldown-cmark-frontmatter crate](https://crates.io/crates/pulldown-cmark-frontmatter)
- [pulldown-cmark-toc crate](https://crates.io/crates/pulldown-cmark-toc)
