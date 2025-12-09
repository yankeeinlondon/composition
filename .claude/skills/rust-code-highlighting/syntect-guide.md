# syntect Guide

TextMate/Sublime grammar-based syntax highlighter. Mature, stable, powers tools like `bat`.

## When to Use

- Need maximum grammar compatibility with VS Code/Sublime
- Static site generators, documentation tools, blog engines
- Already have TextMate grammars you want to reuse
- Want stable ecosystem not tied to Tree-sitter's C codegen

## Key Components

| Component | Purpose |
|-----------|---------|
| `SyntaxSet` | Collection of loaded grammars |
| `ThemeSet` | Collection of color themes |
| `HighlightLines` | Incremental highlighter for syntax + theme |

## Basic Usage - Terminal Output

```rust
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};

fn main() {
    // Load built-in syntaxes and themes (expensive - do once at startup)
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();

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

## HTML Output

```rust
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::ThemeSet;

let ps = SyntaxSet::load_defaults_newlines();
let ts = ThemeSet::load_defaults();
let syntax = ps.find_syntax_by_extension("rs").unwrap();
let theme = &ts.themes["base16-ocean.dark"];

let html = highlighted_html_for_string(code, &ps, syntax, theme)?;
```

## Custom Grammars and Themes

```rust
use syntect::parsing::SyntaxSetBuilder;
use syntect::highlighting::ThemeSet;

// Build SyntaxSet from custom TextMate grammars
let mut builder = SyntaxSetBuilder::new();
builder.add_from_folder("./syntaxes", true)?; // .tmLanguage files
let ps = builder.build();

// Load custom themes (.tmTheme format)
let ts = ThemeSet::load_from_folder("./themes")?;
let theme = &ts.themes["my-theme"];
```

## Theme Format Notes

- syntect uses `.tmTheme` (plist) format natively
- VS Code JSON themes need conversion to .tmTheme
- Many tools exist for VS Code → tmTheme conversion

## Pros

- Huge grammar ecosystem (anything TextMate-compatible)
- Mature, widely used, lots of examples
- Built-in HTML and ANSI output helpers
- Fine for offline/static site use

## Cons

- Regex-based grammars can be slower/more brittle than Tree-sitter
- Less semantic (no incremental parse tree)
- Adding new languages requires porting/writing TextMate grammars
- TextMate-style themes only (not Neovim/Helix native)
