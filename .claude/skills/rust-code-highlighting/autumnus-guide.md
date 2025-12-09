# autumnus Guide

Modern batteries-included Tree-sitter highlighter with 70+ languages and 100+ Neovim themes.

## When to Use

- New projects wanting good highlighting with minimal setup
- Docs sites, blogs, code-preview backends, TUI tools
- Want modern Tree-sitter accuracy with Neovim theme ecosystem
- Need both HTML and terminal output

## Key Features

- 70+ languages via Tree-sitter
- 100+ Neovim themes built-in
- HTML output (inline styles or CSS classes)
- Terminal ANSI output
- Language auto-detection
- Line highlighting support

## Basic Usage - HTML Inline Styles

```rust
use autumnus::{highlight, Options, FormatterOption};

fn main() {
    let code = r#"
    fn main() {
        println!("Hello, world!");
    }
    "#;

    let html = highlight(
        code,
        Options {
            lang_or_file: Some("rust"),
            formatter: FormatterOption::HtmlInline {
                pre_class: Some("code-block"),
                italic: false,
                include_highlights: false,
                theme: Some("tokyonight"),
                highlight_lines: None,
                header: None,
            },
        },
    );

    println!("{html}");
}
```

## HTML with CSS Classes

```rust
use autumnus::{highlight, Options, FormatterOption};

let html = highlight(
    code,
    Options {
        lang_or_file: Some("rust"),
        formatter: FormatterOption::HtmlLinked {
            pre_class: Some("code-block"),
            italic: false,
            include_highlights: false,
            highlight_lines: None,
            header: None,
        },
    },
);
```

## Terminal Output

```rust
use autumnus::{highlight, Options, FormatterOption};

let ansi = highlight(
    code,
    Options {
        lang_or_file: Some("rust"),
        formatter: FormatterOption::Terminal {
            italic: true,
            theme: Some("tokyonight"),
            highlight_lines: None,
        },
    },
);

println!("{ansi}");
```

## Builder API

```rust
use autumnus::HtmlInlineBuilder;

let html = HtmlInlineBuilder::new()
    .language("rust")
    .theme("tokyonight")
    .pre_class("code-block")
    .highlight(code);
```

## Language Auto-Detection

```rust
// Let autumnus detect from filename or shebang
let html = highlight(
    code,
    Options {
        lang_or_file: None, // auto-detect
        formatter: FormatterOption::HtmlInline { /* ... */ },
    },
);
```

## Line Highlighting

```rust
FormatterOption::HtmlInline {
    highlight_lines: Some(vec![1, 3, 5]), // highlight lines 1, 3, 5
    // ...
}
```

## Pros

- Very simple API: one function, one options struct
- Batteries included: grammars + queries + themes all bundled
- Handles HTML and terminal both
- Line-highlighting, headers, and other conveniences

## Cons

- Larger binary size (all Tree-sitter C parsers bundled)
- Higher-level API means less control over token semantics
- If you need weird custom token handling, may need tree-sitter-highlight directly

## Migration from inkjet

The `inkjet` crate is archived and its author recommends migrating to `autumnus`. The API is similar but autumnus has more features and active maintenance.
