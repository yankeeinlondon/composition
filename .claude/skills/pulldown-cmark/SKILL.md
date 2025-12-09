# pulldown-cmark: Rust Markdown Parser

Expert knowledge for working with `pulldown-cmark`, a high-performance CommonMark parser for Rust using a pull-parser architecture. Supports GitHub Flavored Markdown (GFM) extensions including tables, task lists, strikethrough, and autolinks.

## Core Concepts

**Pull Parser Architecture**: Unlike push parsers (SAX) or DOM parsers, pulldown-cmark returns control to the application. The `Parser` implements `Iterator<Item = Event>`, allowing you to process events on-demand with `for` loops, `map`, `filter`, etc.

**Key Benefits**:
- Memory efficient (streaming, no full AST in memory)
- Zero-copy strings via `CowStr<'a>`
- Composable with Rust's iterator ecosystem
- Source mapping support via `into_offset_iter()`

## Quick Start

```rust
use pulldown_cmark::{Parser, html, Options};

// Basic CommonMark conversion
let parser = Parser::new("# Hello\n\nSome **bold** text.");
let mut html_output = String::new();
html::push_html(&mut html_output, parser);

// With GFM extensions (no single "enable all GFM" flag - enable each)
let options = Options::ENABLE_TABLES
    | Options::ENABLE_STRIKETHROUGH
    | Options::ENABLE_TASKLISTS
    | Options::ENABLE_GFM_AUTOLINKS;
let parser = Parser::new_ext(markdown, options);
```

**Note**: GFM is a strict superset of CommonMark. pulldown-cmark aims for high compliance but may not reproduce GitHub's exact output due to post-processing differences.

## Event Types Summary

| Event | Description |
|:------|:------------|
| `Start(Tag)` / `End(TagEnd)` | Element boundaries (paragraph, heading, list, etc.) |
| `Text(CowStr)` | Text content |
| `Code(CowStr)` | Inline code |
| `Html(CowStr)` / `InlineHtml(CowStr)` | Raw HTML |
| `SoftBreak` / `HardBreak` | Line breaks |
| `Rule` | Horizontal rule |
| `FootnoteReference(CowStr)` | Footnote reference |
| `TaskListMarker(bool)` | Checkbox state |

## Extension Flags

Enable via `Options::ENABLE_*`:

**GFM Extensions:**
- `TABLES` - GFM tables with pipe syntax
- `STRIKETHROUGH` - `~~text~~` syntax
- `TASKLISTS` - `- [x]` checkboxes
- `GFM_AUTOLINKS` - Plain URL auto-linking

**Additional Extensions:**
- `FOOTNOTES` - Footnote definitions and references
- `SMART_PUNCTUATION` - Curly quotes, dashes
- `HEADING_ATTRIBUTES` - `{#id .class}` on headings
- `MATH` - `$inline$` and `$$block$$` math
- `YAML_STYLE_METADATA_BLOCKS` - YAML frontmatter support
- `DEFINITION_LISTS` - Definition list syntax (v0.13.0+)

## Detailed Documentation

- [GFM Guide](./gfm.md) - GitHub Flavored Markdown overview, parsing strategy, and compatibility notes
- [Event Stream Reference](./event-stream.md) - Complete event types, Tag variants, transformation patterns
- [Extensions Guide](./extensions.md) - Enabling and using all extension features
- [Code Examples](./examples.md) - Working examples for common tasks
- [Syntax Highlighting](./syntax-highlighting.md) - Integrating syntect for code highlighting
- [LSP Integration](./lsp-integration.md) - Building Language Server Protocol servers with pulldown-cmark

## Related Skills

- [rust-lsp](../rust-lsp/SKILL.md) - Full guide to building LSP servers in Rust

## Common Patterns

### Event Stream Transformation
```rust
// Convert soft breaks to hard breaks
let transformed = parser.map(|event| match event {
    Event::SoftBreak => Event::HardBreak,
    _ => event,
});
```

### Text Replacement
```rust
let transformed = parser.map(|event| match event {
    Event::Text(text) => Event::Text(text.replace("abbr", "abbreviation").into()),
    _ => event,
});
```

### Merge Consecutive Text Events
```rust
use pulldown_cmark::TextMergeStream;
let merged = TextMergeStream::new(parser);
```

### Source Position Tracking
```rust
for (event, range) in parser.into_offset_iter() {
    println!("Event {:?} at bytes {}..{}", event, range.start, range.end);
}
```

## Performance Tips

- Write to buffered targets (`String`, `Vec<u8>`) not directly to stdout/files
- Use `TextMergeStream` only when needed (adds overhead)
- For release builds: `lto = true`, `codegen-units = 1`, `panic = "abort"`
- Enable `simd` feature on x64 for SIMD-accelerated scanning

## Cargo.toml

```toml
[dependencies]
pulldown-cmark = "0.12"  # Check crates.io for latest

# Optional: for syntax highlighting
syntect = "5"
lazy_static = "1"
```
