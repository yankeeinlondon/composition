# Syntax Highlighting with Syntect

Integration guide for adding code syntax highlighting to pulldown-cmark using the `syntect` library.

## Dependencies

```toml
[dependencies]
pulldown-cmark = "0.12"
syntect = "5"
lazy_static = "1"  # For caching SyntaxSet and ThemeSet
```

## Basic Setup

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

## Complete Highlighting Function

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

## Available Themes

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

## Loading Custom Themes

```rust
use syntect::highlighting::ThemeSet;
use std::path::Path;

// Load from .tmTheme file
let theme = ThemeSet::get_theme(Path::new("my-theme.tmTheme"))?;

// Load all themes from directory
let themes = ThemeSet::load_from_folder(Path::new("themes/"))?;
```

## Language Detection

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

## Common Language Mappings

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

## Inline Styles vs CSS Classes

### Inline Styles (Default)

`highlighted_html_for_string` generates inline styles:

```html
<pre style="background-color:#2b303b;">
<code><span style="color:#b48ead;">fn </span><span style="color:#8fa1b3;">main</span>...
```

### CSS Classes

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

## Performance Optimization

### Cache Syntax Lookups

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

### Avoid Repeated Theme Loading

Always use `lazy_static` or `once_cell` to load `SyntaxSet` and `ThemeSet` once:

```rust
use once_cell::sync::Lazy;

static SS: Lazy<SyntaxSet> = Lazy::new(SyntaxSet::load_defaults_newlines);
static TS: Lazy<ThemeSet> = Lazy::new(ThemeSet::load_defaults);
```

## Error Handling

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

## Integration with Static Site Generators

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

## Alternative: highlight-pulldown Crate

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
