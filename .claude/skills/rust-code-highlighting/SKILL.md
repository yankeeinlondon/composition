---
name: rust-code-highlighting
description: Expert knowledge for implementing syntax highlighting in Rust using syntect, tree-sitter, or autumnus. Use when building code highlighters, documentation tools, static site generators, or CLI tools that need syntax coloring.
---

# Rust Code Highlighting

## Quick Decision Guide

| Need | Recommended Crate |
|------|-------------------|
| Simple, fast setup for HTML/terminal output | `autumnus` |
| Maximum grammar compatibility (VS Code/Sublime) | `syntect` |
| Building an editor with incremental updates | `tree-sitter-highlight` |
| Existing project using inkjet | Migrate to `autumnus` |

## Crate Overview

**syntect** - TextMate/Sublime grammars, mature ecosystem, powers `bat`

- Best for: static sites, docs, CLI tools wanting VS Code parity
- See [syntect-guide.md](syntect-guide.md)

**autumnus** - Modern Tree-sitter with 70+ languages, 100+ Neovim themes

- Best for: new projects wanting good highlighting fast
- See [autumnus-guide.md](autumnus-guide.md)

**tree-sitter-highlight** - Low-level Tree-sitter, maximum control

- Best for: editors, IDEs, custom visualizers
- See [tree-sitter-guide.md](tree-sitter-guide.md)

## Quick Examples

### autumnus (recommended for most cases)

```rust
use autumnus::{highlight, Options, FormatterOption};

let html = highlight(code, Options {
    lang_or_file: Some("rust"),
    formatter: FormatterOption::HtmlInline {
        pre_class: Some("code-block"),
        italic: false,
        include_highlights: false,
        theme: Some("tokyonight"),
        highlight_lines: None,
        header: None,
    },
});
```

### syntect (TextMate grammars)

```rust
use syntect::easy::HighlightLines;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::ThemeSet;

let ps = SyntaxSet::load_defaults_newlines();
let ts = ThemeSet::load_defaults();
let syntax = ps.find_syntax_by_extension("rs").unwrap();
let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);
```

## Shiki/syntect Interoperability

Both Shiki (JS) and syntect (Rust) use TextMate grammars - they can share grammar/theme files but don't plug into each other directly. See [shiki-interop.md](shiki-interop.md).
