---
_fixed: true
---

# Rust Code Highlighter Options

## 1. Conceptual landscape

Rust has three main "families" of syntax highlighters:

1. **TextMate grammar-based**
   - Classic, battle-tested; same ecosystem as VS Code/Sublime.
   - Crate: `syntect`

2. **Tree-sitter-based building blocks**
   - Incremental parsing, great accuracy, but you wire more yourself.
   - Crates: `tree-sitter-highlight`, helper crates like `pepegsitter`

3. **Tree-sitter "batteries included"**
   - One call in, HTML / ANSI out, with built-in grammars & themes.
   - Older: `inkjet` (archived now, recommends moving)
   - Modern successor: `autumnus` (crate autumnus, branded "Autumnus/Autumn")

I'll deep-dive those, then compare.

## 2. syntect: TextMate grammars in Rust

What it is
 • Classic Rust syntax highlighter using Sublime Text / TextMate grammars (.sublime-syntax, .tmLanguage) and themes (.tmTheme).  ￼
 • Powers tools like bat and various TUI/CLI tools.
 • Great if:
 • You want maximum grammar compatibility with existing TextMate grammars.
 • You care about stable ecosystem and don’t mind regex-based grammars.

Key pieces
 • SyntaxSet – collection of syntaxes (grammars).
 • ThemeSet – collection of color themes.
 • HighlightLines – incremental highlighter for a syntax & theme.
 • Utilities for both ANSI / terminal and HTML output.

Basic example: highlight Rust to ANSI

use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};

fn main() {
    // Load built-in syntaxes and themes.
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();

    // Pick a syntax and theme.
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

Pros
 • Huge grammar ecosystem (anything TextMate-compatible).
 • Mature & widely used, lots of examples in the wild.
 • Built-in HTML and ANSI output helpers.
 • Fine for offline/static site use (pre-rendered HTML).

Cons
 • Regex-based grammars can be:
 • Slower and more brittle than Tree-sitter for weird languages.
 • Less “semantic” (no incremental parse tree).
 • Getting new languages often means porting or writing a TextMate grammar.
 • Themes are TextMate-style; if you want Neovim/Helix themes, that’s not native.

When I’d use it
 • You already have TextMate grammars you like (e.g. you want VS Code parity).
 • Static site / docs generator, or CLI where you’re OK with regex grammars.
 • You want something stable and not tied to Tree-sitter’s C codegen/binary bloat.

⸻

## 3. tree-sitter-highlight (+ pepegsitter): low-level Tree-sitter

What they are
 • tree-sitter-highlight – core highlighting engine built on Tree-sitter’s parsed syntax trees; you provide:
 • a Language (parser),
 • a HighlightConfiguration (queries describing how to color nodes).  ￼
 • pepegsitter – helper crate bundling a bunch of Tree-sitter parsers + highlight queries behind a unified API.  ￼

You get a stream of HighlightEvents and decide how to render (HTML, ANSI, etc.).

Typical flow

 1. Get a Language from a language-specific crate (tree_sitter_rust::language()).
 2. Build a HighlightConfiguration with highlight queries.
 3. Feed source to Highlighter::highlight.
 4. Iterate over HighlightEvents and render.

### Example: hand-rolled highlighter for Rust

NB: this is intentionally schematic; you’d adjust to the exact highlight signature for the version you use.

```rust
use tree_sitter_highlight::{HighlightConfiguration, Highlighter, HighlightEvent};
use tree_sitter_rust::LANGUAGE;

fn main() -> anyhow::Result<()> {
    // Highlight queries usually come with the grammar or from nvim-treesitter.
    let highlight_query = include_str!("queries/rust/highlights.scm");
    let injections_query = "";
    let locals_query = "";

    let mut config = HighlightConfiguration::new(
        LANGUAGE,
        highlight_query,
        injections_query,
        locals_query,
    )?;

    // Define the “names” for scopes (string IDs you’ll map to CSS classes, colors, etc.)
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

### Using pepegsitter to get languages

```rust
// Get a Tree-sitter language:
let rust_lang = pepegsitter::rust::language();
// Get a ready-made HighlightConfiguration:
let rust_config = pepegsitter::rust::highlight();
// Queries are also exposed as constants if you want full control:
let rust_query = pepegsitter::rust::HIGHLIGHT_QUERY;
```

Pros
 • Very accurate and resilient: full parse tree, not regex.
 • Excellent for editors and anything incremental.
 • You can tap into the huge Tree-sitter grammar ecosystem.
 • pepegsitter saves you from wiring a bunch of parsers manually.

Cons
 • Low-level: you must:
 • bring grammars,
 • bring highlight queries,
 • implement formatting (HTML/ANSI) yourself.
 • Working directly with queries can be fiddly.
 • You deal with unsafe / C code build for parsers, or vendored code.

When I’d use it
 • Building an editor, IDE-like UI, or custom code viewer.
 • You want fine-grained control over token categories, theming, and output.
 • You’re willing to own plumbing and build tooling.

⸻

## 4. inkjet: batteries-included Tree-sitter (archived)

What it is
 • A “batteries-included syntax highlighting library for Rust, based on Tree-sitter” with 70+ languages built in, queries, and an HTML formatter.  ￼
 • Supports:
 • multiple languages via a Language enum,
 • HTML formatter, optional terminal formatter, and a basic theme API,
 • Helix editor themes as input.  ￼
 • As of 2025 the repo is archived and the author explicitly recommends moving to autumnus instead.  ￼

High-level API
 • Highlighter::new() – create a highlighter.
 • Methods:
 • highlight_to_string(lang, &Formatter, source) → String
 • highlight_to_writer(lang, &Formatter, source, &mut Write)
 • highlight_raw(...) → iterator of low-level highlight events.  ￼

Example: Rust → HTML string

```rust
use inkjet::{Highlighter, Language};
use inkjet::formatter::Html;

fn main() -> inkjet::Result<()> {
    let mut highlighter = Highlighter::new();
    let code = r#"
        fn main() {
            println!("Hello, world!");
        }
    "#;

    let html = highlighter.highlight_to_string(Language::Rust, &Html, code)?;
    println!("{html}");

    Ok(())
}
```

That explodes directly to <span class=...> HTML you theme with CSS.  ￼

Pros
 • Simple API: language enum + formatter + source.
 • Doesn’t require you to manage Tree-sitter grammars or queries.
 • Bundles many languages and Helix themes.

Cons
 • Archived / no longer maintained. The author points you at autumnus as the preferred successor.  ￼
 • Binary size and build time can be large (all those C parsers).  ￼
 • Dependency list is non-trivial.

When I’d use it
 • Only if you’re working in an existing codebase using it and you’re not ready to migrate yet.
 • For new work, I’d go straight to autumnus (next section).

⸻

## 5. autumnus (“Autumn”): modern batteries-included Tree-sitter highlighter

What it is
 • A syntax highlighter powered by Tree-sitter and Neovim themes, implemented in Rust (crate autumnus) and also exposed to Elixir as autumn.  ￼
 • Features:
 • 70+ languages via Tree-sitter.  ￼
 • 100+ Neovim themes.  ￼
 • HTML output (inline styles or class-based) and terminal ANSI output.  ￼
 • Language auto-detection and line-highlighting support.  ￼

Core API

From docs.rs:  ￼
 • highlight(source: &str, options: Options<'_>) -> String – returns highlighted string.
 • write_highlight(source, options, &mut Write) – writes to an output sink.
 • Options – includes:
 • lang_or_file: Option<&str> – force a language or let it inspect filename / shebang.
 • formatter: FormatterOption – choose HTML inline, HTML with CSS classes, or terminal.

Re-exports builder APIs like HtmlInlineBuilder, HtmlLinkedBuilder, TerminalBuilder for a more ergonomic builder-style configuration.  ￼

Basic example: Rust → HTML inline

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
            // You can pass "rust" or a filename, or None and rely on auto-detection.
            lang_or_file: Some("rust"),
            formatter: FormatterOption::HtmlInline {
                pre_class: Some("code-block"),
                italic: false,
                include_highlights: false,
                theme: None,          // or Some("tokyonight"), etc.
                highlight_lines: None,
                header: None,
            },
        },
    );

    println!("{html}");
}
```

Terminal example

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

Pros
 • Very simple to integrate: one function, one options struct.
 • Batteries included: Tree-sitter grammars + queries + Neovim themes.  ￼
 • Handles HTML and terminal both, plus line-highlighting, headers, etc.
 • Nice for static site generators, CLIs, and web apps that pre-render code.

Cons
 • Similar build/binary concerns as Inkjet: lots of C parsers imply larger builds (but that’s inherent to the approach).
 • The API is higher-level; if you need weird custom token semantics you may still want to drop to tree-sitter-highlight yourself.

When I’d use it
 • New project where you want:
 • modern Tree-sitter accuracy,
 • Neovim theme ecosystem,
 • minimal setup.
 • Docs sites, blogs, code-preview backends, or TUI tools that just want “good highlighting, right now”.

⸻

## 6. Other crates worth mentioning (briefly)

Just to round out the ecosystem:
 • pepegsitter – already covered above, but note it’s strictly about bundling parsers and highlight configs, not about rendering. Great companion to tree-sitter-highlight if you want to roll your own UI.  ￼
 • There are smaller/younger projects (e.g. thin wrappers around syntect or Tree-sitter; regex-based highlighters) but their ecosystems and APIs are less stable than the four “pillars” above.

⸻

## 7. How the top crates compare

### 7.1 Feature comparison table (high-level)

Crate Grammar tech Batteries included? Output modes Maintenance status (Rust)
syntect TextMate grammars Syntaxes + themes, but you manage HTML, ANSI, custom Mature, stable
tree-sitter-highlight Tree-sitter No (you bring language & queries) Whatever you implement Active as part of Tree-sitter
pepegsitter Tree-sitter Parsers + queries only Whatever you implement Small helper crate
inkjet Tree-sitter 70+ languages, HTML/terminal, themes HTML, ANSI Archived, superseded by autumnus
autumnus Tree-sitter 70+ languages, 100+ themes, formatter HTML (inline/linked), ANSI Actively maintained, current “batteries included” pick  ￼

### 7.2 Thematic trade-offs

TextMate (syntect) vs Tree-sitter (autumnus / tree-sitter-highlight)
 • Accuracy & structure
 • Tree-sitter crates have a full AST and can do more semantic classification.
 • TextMate is regex-based; usually “good enough”, but can break on tricky syntax.
 • Ecosystem
 • TextMate grammars are insanely widespread (VS Code, Sublime, etc.); if you want to reuse those exactly, syntect is great.
 • Tree-sitter grammars are now the default in many editors (Neovim, Helix); autumnus and tree-sitter-highlight ride that wave.
 • Performance / incremental
 • Tree-sitter shines when you need incremental re-highlighting (editors).
 • For static or batch highlighting, performance is usually fine for both.

DIY vs batteries-included Tree-sitter
 • DIY (tree-sitter-highlight + pepegsitter)
 • Maximum control: you control queries, language sets, and rendering.
 • You pay in complexity: you own HTML/ANSI generation, theming, etc.
 • Batteries-included (autumnus, historically inkjet)
 • Minimal code: call highlight() or builder, get HTML/ANSI.
 • You accept the crate’s theme model and language roster.
 • For 99% of use-cases (blog, docs, TUI viewer) this is a win.

Binary size & build time
 • Anything bundling a lot of Tree-sitter grammars (inkjet, autumnus) will:
 • Increase compile time,
 • Increase binary size (multi-MB grammars).  ￼
 • syntect is also non-tiny (many grammars + themes), but those are data files more than C code.

## 8. Recommendations

If I were you, I’d roughly choose like this:
 • New project, want good highlighting fast
→ Use autumnus
 • One call, flexible HTML / terminal, Neovim themes, language auto-detection.
 • Ideal for web backends, doc generators, or CLIs.
 • Building an editor or deeply custom visualizer
→ Use tree-sitter-highlight + language crates, optionally pepegsitter
 • Gives you raw highlight events and full control over theming / output.
 • Need to reuse TextMate grammars or mimic VS Code exactly
→ Use syntect
 • Especially if you’re already living in that ecosystem.
 • Existing codebase on inkjet
→ Keep it for now, but plan to migrate to autumnus; the original author recommends that path and inkjet is archived.  ￼

