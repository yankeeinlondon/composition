---
name: gfm
description: Expert knowledge for GitHub Flavored Markdown (GFM) parsing in Rust using pulldown-cmark, covering CommonMark extensions, tables, task lists, strikethrough, autolinks, footnotes, and event stream processing
hash: 3afc059e7a8afed1
---

# GitHub Flavored Markdown with pulldown-cmark

GFM is a strict superset of CommonMark with additional features for tables, task lists, strikethrough, and autolinks. The `pulldown-cmark` Rust crate is the standard parser for GFM in Rust applications.

## Core Principles

- GFM extends CommonMark; all CommonMark documents are valid GFM
- pulldown-cmark is a pull parser producing event streams, not a DOM tree
- Extensions must be explicitly enabled via `Options` flags
- Enable only what you need for better performance
- Use `TextMergeStream` to handle consecutive text events
- Write HTML output to buffered targets (String, Vec<u8>) for performance
- pulldown-cmark may not match GitHub's exact output (post-processing differences)

## Quick Reference

### Enable GFM Extensions

```rust
use pulldown_cmark::{Parser, Options, html};

let mut options = Options::empty();
options.insert(Options::ENABLE_TABLES);
options.insert(Options::ENABLE_TASKLISTS);
options.insert(Options::ENABLE_STRIKETHROUGH);
options.insert(Options::ENABLE_FOOTNOTES);
options.insert(Options::ENABLE_SMART_PUNCTUATION);

let parser = Parser::new_ext(markdown_input, options);
```

### Render to HTML

```rust
let parser = Parser::new_ext(markdown_input, options);
let mut html_output = String::new();
html::push_html(&mut html_output, parser);
```

### Process Event Stream

```rust
use pulldown_cmark::{Parser, Event, Tag};

for event in parser {
    match event {
        Event::Start(tag) => { /* opening tag */ }
        Event::End(tag) => { /* closing tag */ }
        Event::Text(text) => { /* text content */ }
        Event::Code(code) => { /* inline code */ }
        Event::SoftBreak | Event::HardBreak => { /* line breaks */ }
        _ => {}
    }
}
```

## Topics

### GFM Features

- [GFM Extensions](./gfm-extensions.md) - Tables, task lists, strikethrough, autolinks
- [Parsing Strategy](./parsing-strategy.md) - Two-phase block/inline parsing

### pulldown-cmark Usage

- [Event Processing](./event-processing.md) - Working with the event stream
- [Advanced Techniques](./advanced-techniques.md) - Custom transformations, frontmatter, TOC

## Common Patterns

### Merge Consecutive Text Events

```rust
use pulldown_cmark::{Parser, TextMergeStream};

let iterator = TextMergeStream::new(Parser::new(markdown_input));
for event in iterator {
    // Text events are now merged
}
```

### Transform Events During Parsing

```rust
let transformed: Vec<Event> = parser
    .map(|event| match event {
        Event::Start(Tag::Image(link_type, url, title)) => {
            Event::Start(Tag::Image(link_type, "new_url.png".into(), title))
        }
        other => other,
    })
    .collect();
```

## Extension Options Reference

| GFM Feature | Option Flag | Description |
|-------------|-------------|-------------|
| Tables | `ENABLE_TABLES` | Pipe-based table syntax |
| Task Lists | `ENABLE_TASKLISTS` | Checkbox items (`- [x]`, `- [ ]`) |
| Strikethrough | `ENABLE_STRIKETHROUGH` | `~~text~~` syntax |
| Footnotes | `ENABLE_FOOTNOTES` | Numbered footnote references |
| Smart Punctuation | `ENABLE_SMART_PUNCTUATION` | Auto punctuation replacement |
| Definition Lists | `ENABLE_DEFINITION_LISTS` | Definition list syntax (v0.13+) |

## Resources

- [GFM Spec](https://github.github.com/gfm/) - Official GitHub Flavored Markdown specification
- [pulldown-cmark](https://github.com/pulldown-cmark/pulldown-cmark) - GitHub repository
- [pulldown-cmark docs](https://docs.rs/pulldown-cmark) - API documentation
- [CommonMark Spec](https://spec.commonmark.org/) - Base specification
