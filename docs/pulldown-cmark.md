# pulldown-cmark Deep Dive

A comprehensive guide to working with `pulldown-cmark`, a high-performance CommonMark parser for Rust using a pull-parser architecture. Supports GitHub Flavored Markdown (GFM) extensions including tables, task lists, strikethrough, and autolinks.

## Table of Contents

- [Core Concepts](#core-concepts)
- [Quick Start](#quick-start)
- [Event Stream Reference](#event-stream-reference)
- [GitHub Flavored Markdown (GFM)](#github-flavored-markdown-gfm)
- [Extensions Guide](#extensions-guide)
- [Syntax Highlighting with Syntect](#syntax-highlighting-with-syntect)
- [LSP Integration](#lsp-integration)
- [Code Examples](#code-examples)
- [Performance Tips](#performance-tips)

## Core Concepts

### Pull Parser Architecture

Unlike push parsers (SAX) or DOM parsers, pulldown-cmark returns control to the application. The `Parser` implements `Iterator<Item = Event>`, allowing you to process events on-demand with `for` loops, `map`, `filter`, etc.

### Key Benefits

- **Memory efficient**: Streaming, no full AST in memory
- **Zero-copy strings**: Via `CowStr<'a>`
- **Composable**: Works with Rust's iterator ecosystem
- **Source mapping**: Support via `into_offset_iter()`

## Quick Start

### Basic Usage

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

### Cargo.toml

```toml
[dependencies]
pulldown-cmark = "0.12"  # Check crates.io for latest

# Optional: for syntax highlighting
syntect = "5"
lazy_static = "1"
```

## Event Stream Reference

### Event Enum

The `Event<'a>` enum represents all parsing events:

```rust
pub enum Event<'a> {
    // Element boundaries
    Start(Tag<'a>),        // Opening tag with metadata
    End(TagEnd),           // Closing tag (no metadata)

    // Content events
    Text(CowStr<'a>),      // Text content
    Code(CowStr<'a>),      // Inline code (`code`)
    Html(CowStr<'a>),      // HTML block
    InlineHtml(CowStr<'a>),// Inline HTML (<span>, etc.)

    // Math (requires ENABLE_MATH)
    InlineMath(CowStr<'a>),  // $...$
    DisplayMath(CowStr<'a>), // $$...$$

    // Breaks and rules
    SoftBreak,             // Line break (no trailing spaces)
    HardBreak,             // Line break (2+ spaces or \)
    Rule,                  // Horizontal rule (---, ***, ___)

    // Special elements
    FootnoteReference(CowStr<'a>), // [^label]
    TaskListMarker(bool),          // - [x] or - [ ]
}
```

### Event Types Summary

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

### Tag Enum (Element Types)

```rust
pub enum Tag<'a> {
    // Block elements
    Paragraph,
    Heading { level: HeadingLevel, id: Option<CowStr<'a>>, classes: Vec<CowStr<'a>>, attrs: Vec<(CowStr<'a>, Option<CowStr<'a>>)> },
    BlockQuote(Option<BlockQuoteKind>),
    CodeBlock(CodeBlockKind<'a>),
    HtmlBlock,

    // Lists
    List(Option<u64>),     // Some(n) = ordered starting at n, None = unordered
    Item,                  // List item

    // Tables (requires ENABLE_TABLES)
    Table(Vec<Alignment>),
    TableHead,
    TableRow,
    TableCell,

    // Inline elements
    Emphasis,              // *text* or _text_
    Strong,                // **text** or __text__
    Strikethrough,         // ~~text~~ (requires ENABLE_STRIKETHROUGH)
    Link { link_type: LinkType, dest_url: CowStr<'a>, title: CowStr<'a>, id: CowStr<'a> },
    Image { link_type: LinkType, dest_url: CowStr<'a>, title: CowStr<'a>, id: CowStr<'a> },

    // Special
    FootnoteDefinition(CowStr<'a>),
    MetadataBlock(MetadataBlockKind),
}
```

### TagEnd Enum

`TagEnd` mirrors `Tag` but without data (used only to mark element end):

```rust
pub enum TagEnd {
    Paragraph,
    Heading(HeadingLevel),
    BlockQuote,
    CodeBlock,
    HtmlBlock,
    List(bool),  // true = ordered
    Item,
    Table,
    TableHead,
    TableRow,
    TableCell,
    Emphasis,
    Strong,
    Strikethrough,
    Link,
    Image,
    FootnoteDefinition,
    MetadataBlock(MetadataBlockKind),
}
```

### Supporting Types

#### HeadingLevel

```rust
pub enum HeadingLevel { H1, H2, H3, H4, H5, H6 }
```

#### CodeBlockKind

```rust
pub enum CodeBlockKind<'a> {
    Indented,              // 4-space indented code
    Fenced(CowStr<'a>),    // ```lang - info string (language)
}
```

#### LinkType

```rust
pub enum LinkType {
    Inline,                // [text](url)
    Reference,             // [text][label]
    ReferenceUnknown,      // [text][label] where label undefined
    Collapsed,             // [text][]
    CollapsedUnknown,      // [text][] where text undefined
    Shortcut,              // [text]
    ShortcutUnknown,       // [text] where text undefined
    Autolink,              // <url>
    Email,                 // <email@example.com>
}
```

#### Alignment (Tables)

```rust
pub enum Alignment {
    None,
    Left,
    Center,
    Right,
}
```

#### BlockQuoteKind

```rust
pub enum BlockQuoteKind {
    Note,
    Tip,
    Important,
    Warning,
    Caution,
}
```

### CowStr (Copy-on-Write String)

`CowStr<'a>` is optimized for zero-copy when possible:

```rust
// Usually borrows from input (no allocation)
let text: CowStr = cow_str;

// Convert to owned String when needed
let owned: String = text.into_string();

// Create from String
let cow: CowStr = "modified".to_string().into();
```

### Event Flow Pattern

Events follow depth-first traversal with balanced Start/End pairs:

```md
# Heading          → Start(Heading{H1}) → Text("Heading") → End(Heading(H1))
Paragraph text     → Start(Paragraph) → Text("Paragraph text") → End(Paragraph)
- List item        → Start(List(None)) → Start(Item) → Text("List item") → End(Item) → End(List(false))
```

### Handling Consecutive Text Events

Text may be split across multiple `Event::Text` events. Use `TextMergeStream` to merge them:

```rust
use pulldown_cmark::TextMergeStream;

let merged = TextMergeStream::new(parser);
for event in merged {
    if let Event::Text(text) = event {
        // `text` is now guaranteed to be a single merged string
    }
}
```

### Source Position Tracking

Get byte ranges for each event:

```rust
for (event, range) in parser.into_offset_iter() {
    let source_text = &markdown[range.clone()];
    println!("{:?} at {}..{}: {:?}", event, range.start, range.end, source_text);
}
```

## GitHub Flavored Markdown (GFM)

### Overview

**GitHub Flavored Markdown (GFM)** is a **strict superset** of CommonMark - all CommonMark documents are valid GFM, but GFM adds extensions not in CommonMark. The formal specification (version 0.29-gfm, 2019-04-06) provides precise, unambiguous syntax definitions.

Markdown's formatting syntax prioritizes **readability** - a Markdown document should be publishable as plain text without appearing marked up. GFM maintains this principle while adding practical extensions for GitHub's use cases.

### GFM Extensions vs CommonMark

| Feature | CommonMark | GFM | Description |
|:--------|:----------:|:---:|:------------|
| Tables | - | Yes | Pipe-based table syntax with alignment |
| Task Lists | - | Yes | Checkbox functionality in lists |
| Strikethrough | - | Yes | `~~text~~` for deleted content |
| Autolinks | Limited | Extended | Broader URL detection without angle brackets |
| Disallowed HTML | - | Yes | Security-focused HTML filtering |

### GFM Parsing Strategy

GFM uses a **two-phase parsing approach**:

#### Phase 1: Block Structure

Identifies block-level elements (headings, paragraphs, lists, code blocks, tables) by scanning line-by-line. Establishes the document skeleton without processing inline formatting.

#### Phase 2: Inline Structure

Processes inline elements within each block (emphasis, links, images, code spans, autolinks). Handles text formatting and link reference resolution.

```
Markdown Input → Block Structure → Inline Structure → Output (HTML/AST)
```

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

### Important Considerations

#### Not a Perfect 1:1 Match

`pulldown-cmark` aims for high compliance but **may not reproduce GitHub's exact output**. GitHub applies additional post-processing. Your documents should follow CommonMark standards for predictable results.

#### Feature Availability by Version

GFM-related features are added over time:

- Definition lists (`ENABLE_DEFINITION_LISTS`) added in v0.13.0
- Check [release notes](https://github.com/pulldown-cmark/pulldown-cmark/releases) for latest features

### GFM Syntax Quick Reference

#### Tables

```markdown
| Left | Center | Right |
|:-----|:------:|------:|
| L    |   C    |     R |
```

Column alignment via colons in separator row.

#### Task Lists

```markdown
- [x] Completed task
- [ ] Incomplete task
```

#### Strikethrough

```markdown
~~deleted text~~
```

#### Autolinks

```markdown
https://example.com
user@example.com
```

Automatically converted to links without `<>` wrappers.

### Best Practices

#### Performance

- **Enable only needed extensions** - each adds parsing overhead
- Write to buffered targets (`String`, `Vec<u8>`) not directly to stdout/files
- Use `TextMergeStream` only when needed (adds overhead)

#### Security

- GFM's disallowed raw HTML provides some protection
- Consider additional sanitization for user-generated content
- Be cautious with raw HTML passthrough

#### Compatibility

- Test edge cases against GitHub's actual renderer
- For maximum compatibility, ensure content works with both CommonMark and GFM parsers
- Document which GFM extensions your application requires

### Extended Ecosystem

#### Frontmatter Support

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

#### Table of Contents Generation

Use `pulldown-cmark-toc`:

```rust
use pulldown_cmark_toc::Toc;

let toc = Toc::new(markdown);
println!("{}", toc.to_html());
```

## Extensions Guide

### Enabling Extensions

Use `Options` flags with `Parser::new_ext()`:

```rust
use pulldown_cmark::{Parser, Options};

let mut options = Options::empty();
options.insert(Options::ENABLE_TABLES);
options.insert(Options::ENABLE_FOOTNOTES);
options.insert(Options::ENABLE_STRIKETHROUGH);
options.insert(Options::ENABLE_TASKLISTS);

let parser = Parser::new_ext(markdown, options);
```

Or enable all at once:

```rust
let options = Options::all();
let parser = Parser::new_ext(markdown, options);
```

### Available Extensions

#### Tables (`ENABLE_TABLES`)

GFM-style tables with alignment:

```markdown
| Left | Center | Right |
|:-----|:------:|------:|
| L    |   C    |     R |
```

**Events generated:**

```
Start(Table([Left, Center, Right]))
  Start(TableHead)
    Start(TableCell) → Text("Left") → End(TableCell)
    Start(TableCell) → Text("Center") → End(TableCell)
    Start(TableCell) → Text("Right") → End(TableCell)
  End(TableHead)
  Start(TableRow)
    Start(TableCell) → Text("L") → End(TableCell)
    ...
  End(TableRow)
End(Table)
```

#### Footnotes (`ENABLE_FOOTNOTES`)

```markdown
Here is a footnote[^1].

[^1]: This is the footnote content.
```

**Events:**

- `FootnoteReference("1")` at reference site
- `Start(FootnoteDefinition("1"))` ... `End(FootnoteDefinition)` for definition

#### Strikethrough (`ENABLE_STRIKETHROUGH`)

```markdown
This is ~~deleted~~ text.
```

**Events:** `Start(Strikethrough)` → `Text("deleted")` → `End(Strikethrough)`

#### Task Lists (`ENABLE_TASKLISTS`)

```markdown
- [x] Completed task
- [ ] Incomplete task
```

**Events:**

```
Start(List(None))
  Start(Item)
    TaskListMarker(true)   // checked
    Text("Completed task")
  End(Item)
  Start(Item)
    TaskListMarker(false)  // unchecked
    Text("Incomplete task")
  End(Item)
End(List(false))
```

#### Smart Punctuation (`ENABLE_SMART_PUNCTUATION`)

Converts ASCII punctuation to typographic equivalents:

- `"text"` → "text" (curly quotes)
- `'text'` → 'text' (curly quotes)
- `--` → – (en dash)
- `---` → — (em dash)
- `...` → … (ellipsis)

#### Heading Attributes (`ENABLE_HEADING_ATTRIBUTES`)

Add IDs and classes to headings:

```markdown
# My Heading {#custom-id .class1 .class2 attr="value"}
```

**Events:**

```rust
Start(Tag::Heading {
    level: HeadingLevel::H1,
    id: Some("custom-id"),
    classes: vec!["class1", "class2"],
    attrs: vec![("attr", Some("value"))],
})
```

#### Math (`ENABLE_MATH`)

Inline and display math:

```markdown
Inline math: $E = mc^2$

Display math:
$$
\int_0^\infty e^{-x^2} dx = \frac{\sqrt{\pi}}{2}
$$
```

**Events:**

- `InlineMath("E = mc^2")`
- `DisplayMath("\\int_0^\\infty e^{-x^2} dx = \\frac{\\sqrt{\\pi}}{2}")`

#### Metadata Blocks (`ENABLE_YAML_STYLE_METADATA_BLOCKS`, `ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS`)

YAML front matter:

```markdown
---
title: My Document
author: Jane Doe
---

Content here...
```

TOML-style (pluses):

```markdown
+++
title = "My Document"
author = "Jane Doe"
+++

Content here...
```

**Events:** `Start(MetadataBlock(kind))` → `Text(content)` → `End(MetadataBlock(kind))`

Use `pulldown-cmark-frontmatter` crate for parsing metadata content.

#### GFM Autolinks (`ENABLE_GFM_AUTOLINKS`)

Auto-link plain URLs and email addresses without angle brackets:

```markdown
Visit https://example.com for more info.
Contact user@example.com for support.
```

**Events:** These generate `Event::Start(Tag::Link { link_type: LinkType::Autolink, .. })` events.

Unlike basic CommonMark autolinks which require angle brackets (`<https://example.com>`), GFM autolinks detect URLs automatically in plain text.

#### Old Heading Attributes (`ENABLE_OLD_HEADING_ATTRIBUTES`)

Compatibility with older `{#id}` syntax without full attribute support.

### Combining Extensions

```rust
// Common GFM-like configuration
let options = Options::ENABLE_TABLES
    | Options::ENABLE_FOOTNOTES
    | Options::ENABLE_STRIKETHROUGH
    | Options::ENABLE_TASKLISTS
    | Options::ENABLE_SMART_PUNCTUATION;

// Full feature set
let options = Options::all();

// Selective parsing
let options = Options::ENABLE_TABLES | Options::ENABLE_MATH;
```

### Extension-Specific Rendering

The built-in HTML renderer handles all extensions automatically:

```rust
use pulldown_cmark::{Parser, html, Options};

let options = Options::all();
let parser = Parser::new_ext(markdown, options);
let mut html_output = String::new();
html::push_html(&mut html_output, parser);
```

For custom rendering, handle extension-specific events in your transformation:

```rust
for event in parser {
    match event {
        Event::TaskListMarker(checked) => {
            let checkbox = if checked {
                r#"<input type="checkbox" checked disabled>"#
            } else {
                r#"<input type="checkbox" disabled>"#
            };
            // Emit custom HTML
        }
        Event::InlineMath(math) => {
            // Render with KaTeX or MathJax
        }
        Event::FootnoteReference(label) => {
            // Create footnote link
        }
        _ => { /* default handling */ }
    }
}
```

### Feature Detection at Runtime

Check which options are enabled:

```rust
let options = Options::ENABLE_TABLES | Options::ENABLE_MATH;

if options.contains(Options::ENABLE_TABLES) {
    println!("Tables are enabled");
}
```

## Syntax Highlighting with Syntect

Integration guide for adding code syntax highlighting to pulldown-cmark using the `syntect` library.

### Dependencies

```toml
[dependencies]
pulldown-cmark = "0.12"
syntect = "5"
lazy_static = "1"  # For caching SyntaxSet and ThemeSet
```

### Basic Setup

```rust
use pulldown_cmark::{Parser, Event, Tag, TagEnd, CodeBlockKind, CowStr, html};
use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;

lazy_static::lazy_static! {
    static ref SYNTAX_SET: SyntaxSet = SyntaxSet::load_defaults_newlines();
    static ref THEME_SET: ThemeSet = ThemeSet::load_defaults();
}
```

### Complete Highlighting Function

```rust
use pulldown_cmark::{Parser, Event, Tag, TagEnd, CodeBlockKind, CowStr, html};
use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;

lazy_static::lazy_static! {
    static ref SS: SyntaxSet = SyntaxSet::load_defaults_newlines();
    static ref TS: ThemeSet = ThemeSet::load_defaults();
}

pub fn markdown_to_html_with_highlighting(markdown: &str) -> String {
    let parser = Parser::new(markdown);
    let highlighted = add_syntax_highlighting(parser);
    let mut output = String::new();
    html::push_html(&mut output, highlighted);
    output
}

fn add_syntax_highlighting<'a>(
    parser: impl Iterator<Item = Event<'a>>,
) -> impl Iterator<Item = Event<'a>> {
    let mut in_code_block = false;
    let mut code_buffer = String::new();
    let mut lang_info = String::new();

    parser.filter_map(move |event| match event {
        Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang))) => {
            in_code_block = true;
            code_buffer.clear();
            lang_info = lang.into_string();
            None
        }
        Event::Start(Tag::CodeBlock(CodeBlockKind::Indented)) => {
            in_code_block = true;
            code_buffer.clear();
            lang_info.clear();
            None
        }
        Event::Text(text) if in_code_block => {
            code_buffer.push_str(&text);
            None
        }
        Event::End(TagEnd::CodeBlock) if in_code_block => {
            in_code_block = false;
            let html = highlight_code(&code_buffer, &lang_info);
            Some(Event::Html(CowStr::from(html)))
        }
        other => Some(other),
    })
}

fn highlight_code(code: &str, lang: &str) -> String {
    let theme = &TS.themes["base16-ocean.dark"];

    // Try to find syntax by extension, then by name
    let syntax = SS
        .find_syntax_by_extension(lang)
        .or_else(|| SS.find_syntax_by_name(lang))
        .or_else(|| SS.find_syntax_by_token(lang))
        .unwrap_or_else(|| SS.find_syntax_plain_text());

    match highlighted_html_for_string(code, &SS, syntax, theme) {
        Ok(html) => html,
        Err(_) => {
            // Fallback: escape and wrap in pre/code
            format!(
                "<pre><code class=\"language-{}\">{}</code></pre>",
                html_escape(lang),
                html_escape(code)
            )
        }
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
```

### Available Themes

Built-in themes from `ThemeSet::load_defaults()`:

- `base16-ocean.dark` (popular dark theme)
- `base16-eighties.dark`
- `base16-mocha.dark`
- `base16-ocean.light`
- `InspiredGitHub` (light theme)
- `Solarized (dark)`
- `Solarized (light)`

```rust
// List all available themes
for theme_name in TS.themes.keys() {
    println!("{}", theme_name);
}
```

### Loading Custom Themes

```rust
use syntect::highlighting::ThemeSet;
use std::path::Path;

// Load from .tmTheme file
let theme = ThemeSet::get_theme(Path::new("my-theme.tmTheme"))?;

// Load all themes from directory
let themes = ThemeSet::load_from_folder(Path::new("themes/"))?;
```

### Language Detection

```rust
// By file extension
let syntax = SS.find_syntax_by_extension("rs");  // Rust
let syntax = SS.find_syntax_by_extension("py");  // Python
let syntax = SS.find_syntax_by_extension("js");  // JavaScript

// By language name
let syntax = SS.find_syntax_by_name("Rust");
let syntax = SS.find_syntax_by_name("Python");

// By token (info string from code fence)
let syntax = SS.find_syntax_by_token("rust");
let syntax = SS.find_syntax_by_token("python3");

// Fallback to plain text
let syntax = SS.find_syntax_plain_text();
```

### Common Language Mappings

| Info String | Extension | Language |
|:------------|:----------|:---------|
| `rust`, `rs` | `rs` | Rust |
| `python`, `py` | `py` | Python |
| `javascript`, `js` | `js` | JavaScript |
| `typescript`, `ts` | `ts` | TypeScript |
| `go` | `go` | Go |
| `java` | `java` | Java |
| `c`, `h` | `c` | C |
| `cpp`, `c++`, `cxx` | `cpp` | C++ |
| `csharp`, `cs` | `cs` | C# |
| `ruby`, `rb` | `rb` | Ruby |
| `php` | `php` | PHP |
| `html` | `html` | HTML |
| `css` | `css` | CSS |
| `json` | `json` | JSON |
| `yaml`, `yml` | `yaml` | YAML |
| `toml` | `toml` | TOML |
| `bash`, `sh` | `sh` | Shell |
| `sql` | `sql` | SQL |
| `markdown`, `md` | `md` | Markdown |

### Inline Styles vs CSS Classes

#### Inline Styles (Default)

`highlighted_html_for_string` generates inline styles:

```html
<pre style="background-color:#2b303b;">
<code><span style="color:#b48ead;">fn </span><span style="color:#8fa1b3;">main</span>...
```

#### CSS Classes

For class-based styling, use `ClassedHTMLGenerator`:

```rust
use syntect::html::{ClassedHTMLGenerator, ClassStyle};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

fn highlight_with_classes(code: &str, lang: &str) -> String {
    let syntax = SS
        .find_syntax_by_token(lang)
        .unwrap_or_else(|| SS.find_syntax_plain_text());

    let mut html_generator = ClassedHTMLGenerator::new_with_class_style(
        syntax,
        &SS,
        ClassStyle::Spaced,
    );

    for line in LinesWithEndings::from(code) {
        html_generator.parse_html_for_line_which_includes_newline(line)?;
    }

    format!(
        "<pre class=\"code\"><code class=\"language-{}\">{}</code></pre>",
        lang,
        html_generator.finalize()
    )
}
```

Then generate CSS:

```rust
use syntect::html::css_for_theme_with_class_style;

let css = css_for_theme_with_class_style(theme, ClassStyle::Spaced)?;
// Write to stylesheet
```

### Performance Optimization

#### Cache Syntax Lookups

```rust
use std::collections::HashMap;
use syntect::parsing::SyntaxReference;

lazy_static::lazy_static! {
    static ref SYNTAX_CACHE: HashMap<&'static str, &'static SyntaxReference> = {
        let mut cache = HashMap::new();
        for lang in ["rust", "python", "javascript", "typescript", "go", "java", "html", "css", "json"] {
            if let Some(syntax) = SS.find_syntax_by_extension(lang) {
                cache.insert(lang, syntax);
            }
        }
        cache
    };
}

fn get_syntax(lang: &str) -> &'static SyntaxReference {
    SYNTAX_CACHE
        .get(lang)
        .copied()
        .or_else(|| SS.find_syntax_by_token(lang))
        .unwrap_or_else(|| SS.find_syntax_plain_text())
}
```

#### Avoid Repeated Theme Loading

Always use `lazy_static` or `once_cell` to load `SyntaxSet` and `ThemeSet` once:

```rust
use once_cell::sync::Lazy;

static SS: Lazy<SyntaxSet> = Lazy::new(SyntaxSet::load_defaults_newlines);
static TS: Lazy<ThemeSet> = Lazy::new(ThemeSet::load_defaults);
```

### Error Handling

```rust
fn highlight_code_safe(code: &str, lang: &str) -> String {
    let theme = &TS.themes["base16-ocean.dark"];
    let syntax = SS
        .find_syntax_by_token(lang)
        .unwrap_or_else(|| SS.find_syntax_plain_text());

    highlighted_html_for_string(code, &SS, syntax, theme)
        .unwrap_or_else(|e| {
            eprintln!("Highlighting error for {}: {}", lang, e);
            format!(
                "<pre><code class=\"language-{}\">{}</code></pre>",
                html_escape(lang),
                html_escape(code)
            )
        })
}
```

### Integration with Static Site Generators

```rust
use pulldown_cmark::{Parser, Options, html};

pub fn render_markdown_page(markdown: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);

    let parser = Parser::new_ext(markdown, options);
    let highlighted = add_syntax_highlighting(parser);

    let mut html_content = String::new();
    html::push_html(&mut html_content, highlighted);

    // Wrap in HTML template
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <style>{}</style>
</head>
<body>
    <article>{}</article>
</body>
</html>"#,
        include_str!("syntax-theme.css"),
        html_content
    )
}
```

### Alternative: highlight-pulldown Crate

For simpler integration, consider the `highlight-pulldown` crate:

```toml
[dependencies]
highlight-pulldown = "0.1"
pulldown-cmark = "0.12"
```

```rust
use highlight_pulldown::PulldownHighlighter;
use pulldown_cmark::{Parser, html};

let parser = Parser::new(markdown);
let highlighter = PulldownHighlighter::new(parser, Some("base16-ocean.dark"));

let mut output = String::new();
html::push_html(&mut output, highlighter);
```

## LSP Integration

Guide for using pulldown-cmark as the parsing layer for a Language Server Protocol implementation.

### Why pulldown-cmark for LSP?

- **Source mapping**: `into_offset_iter()` provides byte ranges for every event
- **Streaming**: Process documents incrementally without building full AST
- **Zero-copy**: `CowStr` minimizes allocations
- **GFM support**: Tables, task lists, strikethrough, autolinks
- **Fast**: Suitable for real-time parsing on every keystroke

### Core Pattern: Source Position Tracking

The key to LSP integration is mapping parse events back to source positions:

```rust
use pulldown_cmark::{Parser, Event, Tag, Options};
use lsp_types::{Position, Range};
use ropey::Rope;

fn parse_with_positions(text: &str) -> Vec<(Event, std::ops::Range<usize>)> {
    let options = Options::all();
    Parser::new_ext(text, options)
        .into_offset_iter()
        .collect()
}

// Convert byte offset to LSP Position
fn byte_to_position(rope: &Rope, byte_offset: usize) -> Position {
    let char_idx = rope.byte_to_char(byte_offset);
    let line = rope.char_to_line(char_idx);
    let line_start = rope.line_to_char(line);
    Position::new(line as u32, (char_idx - line_start) as u32)
}

// Convert byte range to LSP Range
fn byte_range_to_lsp_range(rope: &Rope, range: std::ops::Range<usize>) -> Range {
    Range::new(
        byte_to_position(rope, range.start),
        byte_to_position(rope, range.end),
    )
}
```

### Extracting Document Structure

#### Headings (for Document Symbols)

```rust
use pulldown_cmark::{Parser, Event, Tag, HeadingLevel, Options};
use lsp_types::{DocumentSymbol, SymbolKind};

pub struct Heading {
    pub level: u8,
    pub text: String,
    pub range: lsp_types::Range,
    pub selection_range: lsp_types::Range,
}

pub fn extract_headings(text: &str, rope: &Rope) -> Vec<Heading> {
    let mut headings = Vec::new();
    let mut current: Option<(HeadingLevel, std::ops::Range<usize>)> = None;
    let mut text_content = String::new();

    for (event, range) in Parser::new_ext(text, Options::all()).into_offset_iter() {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                current = Some((level, range));
                text_content.clear();
            }
            Event::Text(t) if current.is_some() => {
                text_content.push_str(&t);
            }
            Event::End(pulldown_cmark::TagEnd::Heading(_)) if current.is_some() => {
                let (level, start_range) = current.take().unwrap();
                headings.push(Heading {
                    level: level as u8,
                    text: text_content.clone(),
                    range: byte_range_to_lsp_range(rope, start_range.start..range.end),
                    selection_range: byte_range_to_lsp_range(rope, start_range.clone()),
                });
            }
            _ => {}
        }
    }
    headings
}

// Convert to LSP DocumentSymbol
pub fn headings_to_symbols(headings: Vec<Heading>) -> Vec<DocumentSymbol> {
    headings.into_iter().map(|h| DocumentSymbol {
        name: h.text,
        kind: SymbolKind::STRING,
        range: h.range,
        selection_range: h.selection_range,
        children: None,
        detail: Some(format!("H{}", h.level)),
        tags: None,
        deprecated: None,
    }).collect()
}
```

#### Links (for Go-to-Definition, References)

```rust
use pulldown_cmark::{Event, Tag, LinkType};

pub struct Link {
    pub text: String,
    pub url: String,
    pub link_type: LinkType,
    pub range: lsp_types::Range,
}

pub fn extract_links(text: &str, rope: &Rope) -> Vec<Link> {
    let mut links = Vec::new();
    let mut current_link: Option<(String, LinkType, std::ops::Range<usize>)> = None;
    let mut link_text = String::new();

    for (event, range) in Parser::new_ext(text, Options::all()).into_offset_iter() {
        match event {
            Event::Start(Tag::Link { link_type, dest_url, .. }) => {
                current_link = Some((dest_url.to_string(), link_type, range));
                link_text.clear();
            }
            Event::Text(t) if current_link.is_some() => {
                link_text.push_str(&t);
            }
            Event::End(pulldown_cmark::TagEnd::Link) if current_link.is_some() => {
                let (url, link_type, start_range) = current_link.take().unwrap();
                links.push(Link {
                    text: link_text.clone(),
                    url,
                    link_type,
                    range: byte_range_to_lsp_range(rope, start_range.start..range.end),
                });
            }
            _ => {}
        }
    }
    links
}
```

#### Code Blocks (for Syntax Highlighting, Diagnostics)

```rust
use pulldown_cmark::{CodeBlockKind, Event, Tag};

pub struct CodeBlock {
    pub language: Option<String>,
    pub content: String,
    pub range: lsp_types::Range,
}

pub fn extract_code_blocks(text: &str, rope: &Rope) -> Vec<CodeBlock> {
    let mut blocks = Vec::new();
    let mut current: Option<(Option<String>, std::ops::Range<usize>)> = None;
    let mut content = String::new();

    for (event, range) in Parser::new_ext(text, Options::all()).into_offset_iter() {
        match event {
            Event::Start(Tag::CodeBlock(kind)) => {
                let lang = match kind {
                    CodeBlockKind::Fenced(info) => {
                        let info = info.split_whitespace().next().unwrap_or("");
                        if info.is_empty() { None } else { Some(info.to_string()) }
                    }
                    CodeBlockKind::Indented => None,
                };
                current = Some((lang, range));
                content.clear();
            }
            Event::Text(t) if current.is_some() => {
                content.push_str(&t);
            }
            Event::End(pulldown_cmark::TagEnd::CodeBlock) if current.is_some() => {
                let (language, start_range) = current.take().unwrap();
                blocks.push(CodeBlock {
                    language,
                    content: content.clone(),
                    range: byte_range_to_lsp_range(rope, start_range.start..range.end),
                });
            }
            _ => {}
        }
    }
    blocks
}
```

### LSP Feature Implementations

#### Document Symbols

```rust
async fn document_symbol(&self, params: DocumentSymbolParams) -> Result<Option<DocumentSymbolResponse>> {
    let uri = params.text_document.uri;
    let doc = self.documents.get(&uri)?;

    let headings = extract_headings(&doc.text, &doc.rope);
    let symbols = headings_to_symbols(headings);

    Ok(Some(DocumentSymbolResponse::Nested(symbols)))
}
```

#### Hover (Link Preview)

```rust
async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
    let uri = params.text_document_position_params.text_document.uri;
    let pos = params.text_document_position_params.position;
    let doc = self.documents.get(&uri)?;

    let links = extract_links(&doc.text, &doc.rope);

    // Find link at cursor position
    for link in links {
        if contains_position(&link.range, &pos) {
            return Ok(Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!("**Link:** [{}]({})", link.text, link.url),
                }),
                range: Some(link.range),
            }));
        }
    }
    Ok(None)
}

fn contains_position(range: &Range, pos: &Position) -> bool {
    (range.start.line < pos.line || (range.start.line == pos.line && range.start.character <= pos.character))
        && (range.end.line > pos.line || (range.end.line == pos.line && range.end.character >= pos.character))
}
```

#### Diagnostics (Broken Links)

```rust
async fn validate_links(&self, uri: Url, doc: &DocumentState) {
    let links = extract_links(&doc.text, &doc.rope);
    let mut diagnostics = Vec::new();

    for link in links {
        if link.url.starts_with("./") || link.url.starts_with("../") {
            // Check if relative link target exists
            let target = resolve_relative_path(&uri, &link.url);
            if !target.exists() {
                diagnostics.push(Diagnostic {
                    range: link.range,
                    severity: Some(DiagnosticSeverity::WARNING),
                    message: format!("Broken link: {} not found", link.url),
                    source: Some("markdown-lsp".into()),
                    ..Default::default()
                });
            }
        }
    }

    self.client.publish_diagnostics(uri, diagnostics, None).await;
}
```

#### Completion (Link Targets)

```rust
async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
    let uri = params.text_document_position.text_document.uri;
    let pos = params.text_document_position.position;
    let doc = self.documents.get(&uri)?;

    // Check if we're in a link context
    let line = doc.rope.line(pos.line as usize);
    let line_str: String = line.chars().collect();
    let char_pos = pos.character as usize;

    // Simple check: inside [...]( or [[
    if line_str[..char_pos].contains("](") || line_str[..char_pos].contains("[[") {
        let files = self.workspace.list_markdown_files();
        let items: Vec<CompletionItem> = files.iter().map(|f| {
            CompletionItem {
                label: f.file_name().unwrap().to_string_lossy().into(),
                kind: Some(CompletionItemKind::FILE),
                ..Default::default()
            }
        }).collect();
        return Ok(Some(CompletionResponse::Array(items)));
    }

    Ok(None)
}
```

### Caching Parsed Results

Re-parsing on every LSP request is expensive. Cache the parsed structure:

```rust
pub struct CachedDocument {
    pub text: String,
    pub rope: Rope,
    pub version: i32,
    // Cached parse results
    pub headings: Vec<Heading>,
    pub links: Vec<Link>,
    pub code_blocks: Vec<CodeBlock>,
}

impl CachedDocument {
    pub fn new(text: String, version: i32) -> Self {
        let rope = Rope::from_str(&text);
        let headings = extract_headings(&text, &rope);
        let links = extract_links(&text, &rope);
        let code_blocks = extract_code_blocks(&text, &rope);

        Self { text, rope, version, headings, links, code_blocks }
    }

    pub fn update(&mut self, text: String, version: i32) {
        self.text = text;
        self.rope = Rope::from_str(&self.text);
        self.version = version;
        // Re-extract
        self.headings = extract_headings(&self.text, &self.rope);
        self.links = extract_links(&self.text, &self.rope);
        self.code_blocks = extract_code_blocks(&self.text, &self.rope);
    }
}
```

## Code Examples

### Basic Markdown to HTML

```rust
use pulldown_cmark::{Parser, html};

fn markdown_to_html(markdown: &str) -> String {
    let parser = Parser::new(markdown);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

fn main() {
    let md = "# Hello\n\nThis is **bold** and *italic*.";
    println!("{}", markdown_to_html(md));
}
```

### With Extensions

```rust
use pulldown_cmark::{Parser, html, Options};

fn markdown_to_html_extended(markdown: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(markdown, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}
```

### Extract All Headings

```rust
use pulldown_cmark::{Parser, Event, Tag, HeadingLevel};

fn extract_headings(markdown: &str) -> Vec<(HeadingLevel, String)> {
    let parser = Parser::new(markdown);
    let mut headings = Vec::new();
    let mut current_level = None;
    let mut current_text = String::new();

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                current_level = Some(level);
                current_text.clear();
            }
            Event::Text(text) if current_level.is_some() => {
                current_text.push_str(&text);
            }
            Event::End(pulldown_cmark::TagEnd::Heading(_)) => {
                if let Some(level) = current_level.take() {
                    headings.push((level, std::mem::take(&mut current_text)));
                }
            }
            _ => {}
        }
    }
    headings
}
```

### Build Table of Contents

```rust
use pulldown_cmark::{Parser, Event, Tag, TagEnd, HeadingLevel};

#[derive(Debug)]
struct TocEntry {
    level: u8,
    id: String,
    title: String,
}

fn build_toc(markdown: &str) -> Vec<TocEntry> {
    let mut options = pulldown_cmark::Options::empty();
    options.insert(pulldown_cmark::Options::ENABLE_HEADING_ATTRIBUTES);

    let parser = Parser::new_ext(markdown, options);
    let mut toc = Vec::new();
    let mut current: Option<(HeadingLevel, Option<String>)> = None;
    let mut title = String::new();

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, id, .. }) => {
                current = Some((level, id.map(|s| s.into_string())));
                title.clear();
            }
            Event::Text(text) if current.is_some() => {
                title.push_str(&text);
            }
            Event::End(TagEnd::Heading(_)) => {
                if let Some((level, id)) = current.take() {
                    let slug = id.unwrap_or_else(|| slugify(&title));
                    toc.push(TocEntry {
                        level: level as u8,
                        id: slug,
                        title: std::mem::take(&mut title),
                    });
                }
            }
            _ => {}
        }
    }
    toc
}

fn slugify(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}
```

### Extract All Links

```rust
use pulldown_cmark::{Parser, Event, Tag};

#[derive(Debug)]
struct Link {
    url: String,
    title: String,
    text: String,
}

fn extract_links(markdown: &str) -> Vec<Link> {
    let parser = Parser::new(markdown);
    let mut links = Vec::new();
    let mut current_link: Option<(String, String)> = None;
    let mut link_text = String::new();

    for event in parser {
        match event {
            Event::Start(Tag::Link { dest_url, title, .. }) => {
                current_link = Some((dest_url.into_string(), title.into_string()));
                link_text.clear();
            }
            Event::Text(text) if current_link.is_some() => {
                link_text.push_str(&text);
            }
            Event::End(pulldown_cmark::TagEnd::Link) => {
                if let Some((url, title)) = current_link.take() {
                    links.push(Link {
                        url,
                        title,
                        text: std::mem::take(&mut link_text),
                    });
                }
            }
            _ => {}
        }
    }
    links
}
```

### Add target="_blank" to External Links

```rust
use pulldown_cmark::{Parser, Event, Tag, TagEnd, html, CowStr};

fn add_external_link_attrs(markdown: &str) -> String {
    let parser = Parser::new(markdown);
    let mut in_link = false;
    let mut link_url = String::new();
    let mut link_title = String::new();
    let mut link_content = Vec::new();

    let transformed = parser.flat_map(move |event| {
        match event {
            Event::Start(Tag::Link { dest_url, title, .. }) => {
                in_link = true;
                link_url = dest_url.into_string();
                link_title = title.into_string();
                link_content.clear();
                vec![]
            }
            Event::End(TagEnd::Link) if in_link => {
                in_link = false;
                let is_external = link_url.starts_with("http://") || link_url.starts_with("https://");

                // Render link content to HTML
                let mut inner_html = String::new();
                html::push_html(&mut inner_html, link_content.drain(..).into_iter());

                let html = if is_external {
                    format!(
                        r#"<a href="{}" title="{}" target="_blank" rel="noopener noreferrer">{}</a>"#,
                        html_escape(&link_url),
                        html_escape(&link_title),
                        inner_html
                    )
                } else {
                    format!(
                        r#"<a href="{}" title="{}">{}</a>"#,
                        html_escape(&link_url),
                        html_escape(&link_title),
                        inner_html
                    )
                };
                vec![Event::Html(html.into())]
            }
            other if in_link => {
                link_content.push(other);
                vec![]
            }
            other => vec![other],
        }
    });

    let mut output = String::new();
    html::push_html(&mut output, transformed);
    output
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
```

### Calculate Document Statistics

```rust
use pulldown_cmark::{Parser, Event, Tag, TagEnd};

#[derive(Debug, Default)]
struct DocStats {
    word_count: usize,
    heading_count: usize,
    paragraph_count: usize,
    link_count: usize,
    code_block_count: usize,
    max_nesting: usize,
}

fn analyze_document(markdown: &str) -> DocStats {
    let parser = Parser::new(markdown);
    let mut stats = DocStats::default();
    let mut current_depth = 0;

    for event in parser {
        match event {
            Event::Start(tag) => {
                current_depth += 1;
                stats.max_nesting = stats.max_nesting.max(current_depth);
                match tag {
                    Tag::Heading { .. } => stats.heading_count += 1,
                    Tag::Paragraph => stats.paragraph_count += 1,
                    Tag::Link { .. } => stats.link_count += 1,
                    Tag::CodeBlock(_) => stats.code_block_count += 1,
                    _ => {}
                }
            }
            Event::End(_) => {
                current_depth -= 1;
            }
            Event::Text(text) => {
                stats.word_count += text.split_whitespace().count();
            }
            _ => {}
        }
    }
    stats
}
```

### Render with Source Positions

```rust
use pulldown_cmark::{Parser, Event};

fn render_with_positions(markdown: &str) {
    let parser = Parser::new(markdown);

    for (event, range) in parser.into_offset_iter() {
        let source = &markdown[range.clone()];
        match event {
            Event::Start(tag) => {
                println!("[{}..{}] START {:?}", range.start, range.end, tag);
            }
            Event::End(tag) => {
                println!("[{}..{}] END {:?}", range.start, range.end, tag);
            }
            Event::Text(_) => {
                println!("[{}..{}] TEXT: {:?}", range.start, range.end, source);
            }
            other => {
                println!("[{}..{}] {:?}", range.start, range.end, other);
            }
        }
    }
}
```

### Wrap Headings in Sections

```rust
use pulldown_cmark::{Parser, Event, Tag, TagEnd, HeadingLevel, html, CowStr};

fn wrap_headings_in_sections(markdown: &str) -> String {
    let mut options = pulldown_cmark::Options::empty();
    options.insert(pulldown_cmark::Options::ENABLE_HEADING_ATTRIBUTES);

    let parser = Parser::new_ext(markdown, options);
    let mut events: Vec<Event> = Vec::new();
    let mut section_stack: Vec<HeadingLevel> = Vec::new();

    for event in parser {
        match &event {
            Event::Start(Tag::Heading { level, id, .. }) => {
                // Close sections at same or lower level
                while let Some(open_level) = section_stack.last() {
                    if *level <= *open_level {
                        events.push(Event::Html("</section>\n".into()));
                        section_stack.pop();
                    } else {
                        break;
                    }
                }
                // Open new section
                let id_attr = id.as_ref()
                    .map(|i| format!(r#" id="section-{}""#, i))
                    .unwrap_or_default();
                events.push(Event::Html(format!("<section{}>\n", id_attr).into()));
                section_stack.push(*level);
                events.push(event);
            }
            _ => events.push(event),
        }
    }

    // Close remaining sections
    for _ in section_stack {
        events.push(Event::Html("</section>\n".into()));
    }

    let mut output = String::new();
    html::push_html(&mut output, events.into_iter());
    output
}
```

### Strip Markdown to Plain Text

```rust
use pulldown_cmark::{Parser, Event};

fn strip_markdown(markdown: &str) -> String {
    let parser = Parser::new(markdown);
    let mut plain_text = String::new();

    for event in parser {
        match event {
            Event::Text(text) | Event::Code(text) => {
                plain_text.push_str(&text);
            }
            Event::SoftBreak | Event::HardBreak => {
                plain_text.push('\n');
            }
            Event::Start(_) | Event::End(_) => {
                // Add space between block elements
                if !plain_text.ends_with(' ') && !plain_text.ends_with('\n') {
                    plain_text.push(' ');
                }
            }
            _ => {}
        }
    }
    plain_text.trim().to_string()
}
```

### Transformation Patterns

#### Basic Map Transform

```rust
let transformed = parser.map(|event| match event {
    Event::SoftBreak => Event::HardBreak,
    other => other,
});
```

#### Filter Events

```rust
// Remove all images
let no_images = parser.filter(|event| {
    !matches!(event, Event::Start(Tag::Image { .. }) | Event::End(TagEnd::Image))
});
```

#### Stateful Transform (Custom Iterator)

```rust
struct MyTransform<I> {
    inner: I,
    in_heading: bool,
}

impl<'a, I: Iterator<Item = Event<'a>>> Iterator for MyTransform<I> {
    type Item = Event<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let event = self.inner.next()?;
        match &event {
            Event::Start(Tag::Heading { .. }) => self.in_heading = true,
            Event::End(TagEnd::Heading(_)) => self.in_heading = false,
            Event::Text(text) if self.in_heading => {
                return Some(Event::Text(text.to_uppercase().into()));
            }
            _ => {}
        }
        Some(event)
    }
}
```

#### Accumulating Transform (Code Blocks)

```rust
let mut in_code = false;
let mut code_buf = String::new();
let mut lang = String::new();

let events: Vec<_> = parser.filter_map(|event| match event {
    Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(l))) => {
        in_code = true;
        lang = l.into_string();
        code_buf.clear();
        None
    }
    Event::Text(text) if in_code => {
        code_buf.push_str(&text);
        None
    }
    Event::End(TagEnd::CodeBlock) if in_code => {
        in_code = false;
        // Process code_buf and lang, return transformed event
        Some(Event::Html(format!("<pre data-lang=\"{lang}\">{code_buf}</pre>").into()))
    }
    other => Some(other),
}).collect();
```

## Performance Tips

- Write to buffered targets (`String`, `Vec<u8>`) not directly to stdout/files
- Use `TextMergeStream` only when needed (adds overhead)
- For release builds: `lto = true`, `codegen-units = 1`, `panic = "abort"`
- Enable `simd` feature on x64 for SIMD-accelerated scanning
- **Enable only needed extensions** - each adds parsing overhead

## Related Skills

- [rust-lsp](../rust-lsp/SKILL.md) - Full LSP development guide
- [rust-lsp/markdown-lsps](../rust-lsp/markdown-lsps.md) - Existing Rust Markdown LSPs
- [rust-lsp/document-management](../rust-lsp/document-management.md) - Text rope and state management
