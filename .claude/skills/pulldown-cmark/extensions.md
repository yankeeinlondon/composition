# Extensions Guide

## Enabling Extensions

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

## Available Extensions

### Tables (`ENABLE_TABLES`)

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

### Footnotes (`ENABLE_FOOTNOTES`)

```markdown
Here is a footnote[^1].

[^1]: This is the footnote content.
```

**Events:**
- `FootnoteReference("1")` at reference site
- `Start(FootnoteDefinition("1"))` ... `End(FootnoteDefinition)` for definition

### Strikethrough (`ENABLE_STRIKETHROUGH`)

```markdown
This is ~~deleted~~ text.
```

**Events:** `Start(Strikethrough)` → `Text("deleted")` → `End(Strikethrough)`

### Task Lists (`ENABLE_TASKLISTS`)

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

### Smart Punctuation (`ENABLE_SMART_PUNCTUATION`)

Converts ASCII punctuation to typographic equivalents:
- `"text"` → "text" (curly quotes)
- `'text'` → 'text' (curly quotes)
- `--` → – (en dash)
- `---` → — (em dash)
- `...` → … (ellipsis)

### Heading Attributes (`ENABLE_HEADING_ATTRIBUTES`)

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

### Math (`ENABLE_MATH`)

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

### Metadata Blocks (`ENABLE_YAML_STYLE_METADATA_BLOCKS`, `ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS`)

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

### GFM Autolinks (`ENABLE_GFM_AUTOLINKS`)

Auto-link plain URLs and email addresses without angle brackets:

```markdown
Visit https://example.com for more info.
Contact user@example.com for support.
```

**Events:** These generate `Event::Start(Tag::Link { link_type: LinkType::Autolink, .. })` events.

Unlike basic CommonMark autolinks which require angle brackets (`<https://example.com>`), GFM autolinks detect URLs automatically in plain text.

### Old Heading Attributes (`ENABLE_OLD_HEADING_ATTRIBUTES`)

Compatibility with older `{#id}` syntax without full attribute support.

## Combining Extensions

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

## Extension-Specific Rendering

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

## Feature Detection at Runtime

Check which options are enabled:

```rust
let options = Options::ENABLE_TABLES | Options::ENABLE_MATH;

if options.contains(Options::ENABLE_TABLES) {
    println!("Tables are enabled");
}
```
