---
name: rust-code-highlighting
description: Comprehensive guide to syntax highlighting in Rust using syntect, tree-sitter, and related crates
created: 2025-12-08
hash: dd29dca61e1ac397
tags:
  - rust
  - syntax-highlighting
  - syntect
  - tree-sitter
  - autumnus
  - code-formatting
---

# Rust Code Highlighting

Syntax highlighting is essential for code documentation, editors, terminal applications, and web-based code displays. The Rust ecosystem offers several mature options, each with distinct trade-offs between accuracy, ease of use, and integration flexibility.

## Table of Contents

- [Overview](#overview)
- [Choosing a Highlighter](#choosing-a-highlighter)
- [syntect: TextMate Grammars in Rust](#syntect-textmate-grammars-in-rust)
- [tree-sitter-highlight: Low-Level Tree-sitter](#tree-sitter-highlight-low-level-tree-sitter)
- [autumnus: Batteries-Included Tree-sitter](#autumnus-batteries-included-tree-sitter)
- [Shiki and Syntect Interoperability](#shiki-and-syntect-interoperability)
- [Advanced Topics](#advanced-topics)
- [Quick Reference](#quick-reference)
- [Resources](#resources)

## Overview

Rust has three main families of syntax highlighters:

| Family | Approach | Key Crate(s) | Best For |
|--------|----------|--------------|----------|
| **TextMate-based** | Regex patterns from Sublime Text grammars | `syntect` | Static output, VS Code parity |
| **Tree-sitter Low-Level** | Incremental parsing with full AST | `tree-sitter-highlight`, `pepegsitter` | Editors, custom visualizers |
| **Tree-sitter Batteries-Included** | One-call API with bundled grammars and themes | `autumnus` | Quick integration, docs/blogs |

### TextMate vs Tree-sitter

The fundamental distinction is between regex-based tokenization (TextMate) and full parsing (Tree-sitter):

- **TextMate grammars** use regular expressions to identify tokens. They are battle-tested, widely available (VS Code, Sublime Text), and produce high-quality output for most languages. However, they can struggle with complex or context-sensitive syntax.

- **Tree-sitter** builds a complete syntax tree through incremental parsing. This provides semantic accuracy and enables features like incremental re-highlighting. The trade-off is more complex setup and larger binary sizes when bundling parsers.

## Choosing a Highlighter

| Use Case | Recommended Crate | Rationale |
|----------|-------------------|-----------|
| Static site generator or blog | `autumnus` | Simple API, 70+ languages, 100+ themes |
| Documentation tools | `syntect` or `autumnus` | Both produce high-quality HTML output |
| Code editor or IDE | `tree-sitter-highlight` | Incremental updates, semantic accuracy |
| CLI/TUI application | `syntect` or `autumnus` | Both support ANSI terminal output |
| VS Code theme compatibility | `syntect` | Native TextMate grammar support |
| Neovim theme compatibility | `autumnus` | Native Neovim theme support |
| Minimal dependencies | `syntect` | Data files vs compiled C parsers |

## syntect: TextMate Grammars in Rust

`syntect` is the classic Rust syntax highlighter, using Sublime Text / TextMate grammars (`.sublime-syntax`, `.tmLanguage`) and themes (`.tmTheme`). It powers tools like `bat` and many documentation generators.

### Core Components

- **SyntaxSet** - Collection of language grammars
- **ThemeSet** - Collection of color themes
- **HighlightLines** - Incremental highlighter for a syntax and theme
- **Utilities** - ANSI terminal and HTML output helpers

### Basic Usage: Terminal Output

```rust
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};

fn main() {
    // Load built-in syntaxes and themes (expensive, do once at startup)
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();

    // Pick a syntax and theme
    let syntax = ps.find_syntax_by_extension("rs").unwrap();
    let theme = &ts.themes["base16-ocean.dark"];

    let mut h = HighlightLines::new(syntax, theme);

    let code = r#"fn main() { println!("Hello, world!"); }"#;

    for line in LinesWithEndings::from(code) {
        let ranges: Vec<(Style, &str)> = h.highlight_line(line, &ps).unwrap();
        let escaped = as_24_bit_terminal_escaped(&ranges, false);
        print!("{escaped}");
    }
}
```

### Loading Custom Grammars and Themes

```rust
use syntect::parsing::SyntaxSetBuilder;
use syntect::highlighting::{ThemeSet, Theme};
use syntect::easy::HighlightLines;

fn main() -> anyhow::Result<()> {
    // Build SyntaxSet from custom TextMate grammars
    let mut builder = SyntaxSetBuilder::new();
    builder.add_from_folder("./syntaxes", true)?;
    let ps = builder.build();

    // Load themes from folder
    let ts = ThemeSet::load_from_folder("./themes")?;
    let theme: &Theme = &ts.themes["my-theme"];

    let syntax = ps.find_syntax_by_extension("foo").unwrap();
    let mut h = HighlightLines::new(syntax, theme);

    let code = "let x: Foo = 1;";

    for line in syntect::util::LinesWithEndings::from(code) {
        let ranges = h.highlight_line(line, &ps)?;
        let escaped = syntect::util::as_24_bit_terminal_escaped(&ranges, false);
        print!("{escaped}");
    }

    Ok(())
}
```

### Strengths and Limitations

**Strengths:**
- Huge grammar ecosystem (anything TextMate-compatible)
- Mature and widely used with extensive examples
- Built-in HTML and ANSI output helpers
- Stable API, good for offline/static site use

**Limitations:**
- Regex-based grammars can be slower and more brittle than Tree-sitter
- Less semantic understanding (no parse tree)
- Adding new languages requires TextMate grammar availability
- Themes are TextMate-style only

## tree-sitter-highlight: Low-Level Tree-sitter

For maximum control and semantic accuracy, `tree-sitter-highlight` provides direct access to Tree-sitter's parsing and highlighting capabilities. You provide the language parser and highlight queries, receiving a stream of highlight events to render as you choose.

### Typical Workflow

1. Get a `Language` from a language-specific crate (e.g., `tree_sitter_rust::language()`)
2. Build a `HighlightConfiguration` with highlight queries
3. Feed source to `Highlighter::highlight`
4. Iterate over `HighlightEvent`s and render

### Example: Custom HTML Output

```rust
use tree_sitter_highlight::{HighlightConfiguration, Highlighter, HighlightEvent};
use tree_sitter_rust::LANGUAGE;

fn main() -> anyhow::Result<()> {
    // Highlight queries usually come with the grammar or from nvim-treesitter
    let highlight_query = include_str!("queries/rust/highlights.scm");
    let injections_query = "";
    let locals_query = "";

    let mut config = HighlightConfiguration::new(
        LANGUAGE,
        highlight_query,
        injections_query,
        locals_query,
    )?;

    // Define scope names to map to CSS classes or colors
    config.configure(&[
        "keyword",
        "function",
        "type",
        "string",
        "comment",
        "number",
        "operator",
        "variable",
    ]);

    let mut highlighter = Highlighter::new();
    let source = r#"fn main() { println!("Hello, world!"); }"#;

    let events = highlighter.highlight(&config, source.as_bytes(), None, |_| None)?;

    let mut styled = String::new();
    let mut stack: Vec<&str> = Vec::new();

    for event in events {
        match event? {
            HighlightEvent::HighlightStart(hl) => {
                let scope_name = config.names()[hl.0];
                stack.push(scope_name);
                styled.push_str(&format!("<span class=\"{}\">", scope_name));
            }
            HighlightEvent::Source { start, end } => {
                styled.push_str(&html_escape::encode_text(&source[start..end]));
            }
            HighlightEvent::HighlightEnd => {
                stack.pop();
                styled.push_str("</span>");
            }
        }
    }

    println!("{styled}");
    Ok(())
}
```

### Using pepegsitter for Bundled Parsers

`pepegsitter` bundles many Tree-sitter parsers and highlight queries behind a unified API:

```rust
// Get a Tree-sitter language
let rust_lang = pepegsitter::rust::language();

// Get a ready-made HighlightConfiguration
let rust_config = pepegsitter::rust::highlight();

// Queries are also exposed as constants for full control
let rust_query = pepegsitter::rust::HIGHLIGHT_QUERY;
```

### Strengths and Limitations

**Strengths:**
- Full parse tree for semantic accuracy
- Excellent for editors with incremental re-highlighting
- Access to the huge Tree-sitter grammar ecosystem
- Maximum control over theming and output

**Limitations:**
- Low-level: you must bring grammars, queries, and implement rendering
- Working with highlight queries can be complex
- Involves unsafe/C code build for parsers

## autumnus: Batteries-Included Tree-sitter

`autumnus` (also known as "Autumn") is the modern batteries-included Tree-sitter highlighter. It provides a simple one-call API with 70+ bundled languages and 100+ Neovim themes.

### Core API

```rust
use autumnus::{highlight, Options, FormatterOption};
```

- `highlight(source: &str, options: Options<'_>) -> String` - Returns highlighted string
- `write_highlight(source, options, &mut Write)` - Writes to an output sink
- `Options` - Configuration including language, formatter, and theme

### Example: HTML with Inline Styles

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
                theme: None,  // or Some("tokyonight")
                highlight_lines: None,
                header: None,
            },
        },
    );

    println!("{html}");
}
```

### Example: Terminal ANSI Output

```rust
use autumnus::{highlight, Options, FormatterOption};

fn main() {
    let code = r#"let x = 42;"#;

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
}
```

### Features

- 70+ languages via Tree-sitter
- 100+ Neovim themes
- HTML output (inline styles or CSS classes)
- Terminal ANSI output
- Language auto-detection
- Line highlighting support
- Builder APIs for ergonomic configuration

### Strengths and Limitations

**Strengths:**
- Simple integration: one function, one options struct
- Batteries included: grammars, queries, and themes bundled
- Handles both HTML and terminal output
- Supports line highlighting, headers, and more

**Limitations:**
- Large build size due to bundled C parsers
- Higher-level API may not suit custom token semantics
- Less control compared to raw `tree-sitter-highlight`

### Note on inkjet

`inkjet` was a similar batteries-included crate that is now archived. The author recommends migrating to `autumnus`. If you have existing code using `inkjet`, plan to migrate but it will continue to work for now.

## Shiki and Syntect Interoperability

Shiki (a JavaScript/TypeScript highlighter) and syntect share common foundations but do not directly integrate.

### Common Ground

Both use:
- TextMate grammars for tokenization
- VS Code/TextMate-style themes for coloring

The key difference is the runtime:
- **Shiki** = TS/JS TextMate engine (Oniguruma / JS RegExp) producing HTML with inline styles or HAST
- **syntect** = Rust TextMate engine (Oniguruma via `onig` crate) producing HTML/ANSI/custom output

### Sharing Assets Between Stacks

The integration point is **using the same grammar and theme files** in both stacks, not embedding one engine in the other.

**In Shiki (Node/TS):**

```ts
import { getHighlighter } from 'shiki';

const highlighter = await getHighlighter({
  themes: ['vitesse-dark', './themes/my-theme.json'],
  langs: ['ts', './syntaxes/foo.tmLanguage.json'],
});

const html = await highlighter.codeToHtml('let x: Foo = 1;', {
  lang: 'foo',
  theme: 'my-theme',
});
```

**In syntect (Rust):**

```rust
let mut builder = SyntaxSetBuilder::new();
builder.add_from_folder("./syntaxes", true)?;
let ps = builder.build();

let ts = ThemeSet::load_from_folder("./themes")?;
// Note: syntect prefers .tmTheme format; convert VS Code JSON themes
```

### Theme Format Considerations

- Shiki uses VS Code JSON themes directly
- syntect's native format is `.tmTheme` (plist)
- Convert VS Code JSON to `.tmTheme` for syntect compatibility

### Running syntect from JavaScript

The `@syntect/node` package exposes syntect to Node.js via WebAssembly:

```javascript
import { highlight } from '@syntect/node';

const html = await highlight({
  code: 'fn main() {}',
  extension: 'rs',
  theme: 'base16-ocean.dark',
});

console.log(html); // HTML with CSS classes
```

This allows using syntect alongside Shiki in JavaScript projects, though they remain independent engines.

### Current Limitations

There is no:
- Official adapter making Shiki use syntect as a backend
- Shared token stream protocol between the two
- Plugin system in either direction

## Advanced Topics

### Performance Considerations

**Build Time and Binary Size:**

Tree-sitter crates bundling multiple parsers (`autumnus`, the archived `inkjet`) include compiled C code for each grammar, significantly increasing:
- Compile time
- Binary size (multiple megabytes)

`syntect` also ships data files (grammars and themes) but these are more compact than compiled parsers.

**Runtime Performance:**

- For static/batch highlighting, both approaches perform well
- Tree-sitter excels at incremental re-highlighting (editor use cases)
- `syntect` is typically faster for one-shot highlighting of documents

### Incremental Highlighting

Tree-sitter's primary advantage for editors is incremental parsing. When source code changes, only the affected portions of the syntax tree are re-parsed. This enables efficient real-time highlighting in editors like Helix and Zed.

For batch processing (static sites, documentation), this advantage is less relevant.

### Language Quality Variations

- **syntect/TextMate**: High and consistent quality across languages due to the mature Sublime Text ecosystem
- **Tree-sitter**: Quality varies per grammar. Well-maintained grammars (Rust, JavaScript, Python) are excellent; less common languages may have gaps

### Theme Ecosystems

| Highlighter | Native Theme Format | Ecosystem |
|-------------|---------------------|-----------|
| syntect | `.tmTheme` (TextMate) | VS Code, Sublime Text |
| autumnus | Neovim themes | Neovim, Helix |
| tree-sitter-highlight | Custom (you define) | Flexible |

## Quick Reference

### Crate Comparison Table

| Crate | Grammar Tech | Batteries Included | Output Modes | Status |
|-------|--------------|-------------------|--------------|--------|
| `syntect` | TextMate | Syntaxes + themes bundled | HTML, ANSI, custom | Mature, stable |
| `tree-sitter-highlight` | Tree-sitter | None (bring your own) | Custom only | Active |
| `pepegsitter` | Tree-sitter | Parsers + queries only | Custom only | Helper crate |
| `autumnus` | Tree-sitter | 70+ languages, 100+ themes | HTML, ANSI | Active, recommended |
| `inkjet` | Tree-sitter | 70+ languages, themes | HTML, ANSI | Archived |

### Decision Flowchart

1. **Need quick integration with minimal code?** Use `autumnus`
2. **Building an editor or need incremental updates?** Use `tree-sitter-highlight`
3. **Need VS Code/TextMate theme compatibility?** Use `syntect`
4. **Need Neovim theme compatibility?** Use `autumnus`
5. **Need maximum control over output?** Use `tree-sitter-highlight` + `pepegsitter`

### Cargo Dependencies

```toml
# For syntect
[dependencies]
syntect = "5"

# For autumnus
[dependencies]
autumnus = "0.1"

# For tree-sitter-highlight (manual setup)
[dependencies]
tree-sitter-highlight = "0.20"
tree-sitter-rust = "0.20"  # or other language crates

# For pepegsitter (bundled parsers)
[dependencies]
pepegsitter = "0.1"
```

## Resources

### Official Documentation

- [syntect on crates.io](https://crates.io/crates/syntect)
- [syntect documentation](https://docs.rs/syntect)
- [autumnus on crates.io](https://crates.io/crates/autumnus)
- [autumnus documentation](https://docs.rs/autumnus)
- [tree-sitter-highlight on crates.io](https://crates.io/crates/tree-sitter-highlight)
- [Tree-sitter official site](https://tree-sitter.github.io/tree-sitter/)

### Related Projects

- [bat](https://github.com/sharkdp/bat) - A cat clone with syntax highlighting (uses syntect)
- [Helix](https://helix-editor.com/) - A post-modern text editor (uses tree-sitter)
- [Shiki](https://shiki.style/) - JavaScript syntax highlighter
- [@syntect/node](https://www.npmjs.com/package/@syntect/node) - syntect bindings for Node.js

### Grammar and Theme Resources

- [TextMate Language Grammars](https://macromates.com/manual/en/language_grammars)
- [VS Code Language Extensions](https://code.visualstudio.com/api/language-extensions/syntax-highlight-guide)
- [nvim-treesitter](https://github.com/nvim-treesitter/nvim-treesitter) - Tree-sitter queries for Neovim
