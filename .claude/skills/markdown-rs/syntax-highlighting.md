# Syntax Highlighting with markdown-rs

Integrate syntax highlighting into Markdown-rendered code blocks using syntect, autumnus, or tree-sitter.

## Integration Approaches

Since markdown-rs builds an AST rather than exposing event streams, there are two main approaches:

1. **AST Traversal**: Parse to AST, find code blocks, apply highlighting
2. **HTML Post-Processing**: Generate HTML, parse to find `<code>` elements, replace with highlighted versions

## Solution Comparison

| Solution | Approach | Pros | Cons |
|----------|----------|------|------|
| **syntect** | Regex-based (Sublime) | Mature, extensive languages, fast | Not semantic |
| **autumnus** | Tree-sitter | Modern, Neovim themes, accurate | Growing language support |
| **tree-sitter** | Direct parsing | Most accurate, semantic | Complex setup |

## syntect Integration

### Dependencies

```toml
[dependencies]
markdown = "1.0.0-alpha.21"
syntect = "5.2"
```

### AST-Based Approach

```rust
use markdown::{to_mdast, to_html_with_options, ParseOptions, Options};
use markdown::mdast::{Node, Code};
use syntect::html::{ClassedHTMLGenerator, ClassStyle};
use syntect::parsing::SyntaxSet;

struct Highlighter {
    syntax_set: SyntaxSet,
}

impl Highlighter {
    fn new() -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
        }
    }

    fn highlight(&self, code: &str, lang: &str) -> String {
        let syntax = self.syntax_set
            .find_syntax_by_token(lang)
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());

        let mut generator = ClassedHTMLGenerator::new_with_class_style(
            syntax,
            &self.syntax_set,
            ClassStyle::Spaced,
        );

        for line in code.lines() {
            generator.parse_html_for_line_which_includes_newline(line)
                .expect("failed to parse line");
        }

        format!("<pre><code class=\"language-{}\">{}</code></pre>",
            lang,
            generator.finalize()
        )
    }

    fn process_markdown(&self, input: &str) -> String {
        let ast = to_mdast(input, &ParseOptions::gfm()).unwrap();
        let mut html = to_html_with_options(input, &Options::gfm()).unwrap();

        // Find code blocks and replace with highlighted versions
        self.walk_and_replace(&ast, &mut html);
        html
    }

    fn walk_and_replace(&self, node: &Node, html: &mut String) {
        if let Node::Code(Code { value, lang, .. }) = node {
            let lang = lang.as_deref().unwrap_or("text");
            let original = format!("<pre><code class=\"language-{}\">{}</code></pre>",
                lang,
                html_escape(value)
            );
            let highlighted = self.highlight(value, lang);
            *html = html.replace(&original, &highlighted);
        }

        if let Some(children) = node.children() {
            for child in children {
                self.walk_and_replace(child, html);
            }
        }
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
```

### Inline Theme (No CSS Required)

```rust
use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_html_for_string;

fn highlight_with_theme(code: &str, lang: &str) -> String {
    let syntax_set = SyntaxSet::load_defaults_newlines();
    let theme_set = ThemeSet::load_defaults();
    let theme = &theme_set.themes["base16-ocean.dark"];

    let syntax = syntax_set
        .find_syntax_by_token(lang)
        .unwrap_or_else(|| syntax_set.find_syntax_plain_text());

    highlighted_html_for_string(code, &syntax_set, syntax, theme)
        .unwrap_or_else(|_| format!("<pre><code>{}</code></pre>", html_escape(code)))
}
```

## autumnus Integration

### Dependencies

```toml
[dependencies]
markdown = "1.0.0-alpha.21"
autumnus = "0.2"
```

### Basic Usage

```rust
use markdown::{to_mdast, ParseOptions};
use markdown::mdast::{Node, Code};
use autumnus::{highlight, Options as AutumnusOptions, FormatterOption};

fn highlight_code(code: &str, lang: &str) -> String {
    highlight(
        code,
        AutumnusOptions {
            lang_or_file: Some(lang),
            formatter: FormatterOption::HtmlInline {
                pre_class: Some("code-block"),
                italic: false,
                include_highlights: false,
                theme: None,
                highlight_lines: None,
                header: None,
            },
        },
    )
}

fn process_markdown_with_autumnus(input: &str) -> String {
    let ast = to_mdast(input, &ParseOptions::gfm()).unwrap();
    let mut output = String::new();

    fn walk(node: &Node, output: &mut String) {
        match node {
            Node::Code(Code { value, lang, .. }) => {
                let lang = lang.as_deref().unwrap_or("text");
                output.push_str(&highlight_code(value, lang));
            }
            Node::Paragraph(p) => {
                output.push_str("<p>");
                for child in &p.children {
                    walk(child, output);
                }
                output.push_str("</p>\n");
            }
            Node::Text(t) => {
                output.push_str(&html_escape(&t.value));
            }
            // Handle other node types...
            _ => {
                if let Some(children) = node.children() {
                    for child in children {
                        walk(child, output);
                    }
                }
            }
        }
    }

    walk(&ast, &mut output);
    output
}
```

## tree-sitter Integration

### Dependencies

```toml
[dependencies]
markdown = "1.0.0-alpha.21"
tree-sitter = "0.24"
tree-sitter-highlight = "0.24"
tree-sitter-rust = "0.23"  # Add grammars as needed
```

### Direct Tree-sitter Highlighting

```rust
use tree_sitter_highlight::{HighlightConfiguration, Highlighter, HtmlRenderer};

fn highlight_rust_code(code: &str) -> Result<String, Box<dyn std::error::Error>> {
    let language = tree_sitter_rust::LANGUAGE.into();

    let mut config = HighlightConfiguration::new(
        language,
        "rust",
        tree_sitter_rust::HIGHLIGHTS_QUERY,
        "",  // injections query
        "",  // locals query
    )?;

    let highlight_names = &[
        "keyword",
        "function",
        "type",
        "variable",
        "string",
        "comment",
        "operator",
        "number",
        "property",
    ];

    config.configure(highlight_names);

    let mut highlighter = Highlighter::new();
    let highlights = highlighter.highlight(
        &config,
        code.as_bytes(),
        None,
        |_| None,
    )?;

    let mut renderer = HtmlRenderer::new();
    renderer.render(highlights, code.as_bytes(), &|h| {
        let class = match h.0 {
            0 => "keyword",
            1 => "function",
            2 => "type",
            3 => "variable",
            4 => "string",
            5 => "comment",
            6 => "operator",
            7 => "number",
            8 => "property",
            _ => "unknown",
        };
        format!("<span class=\"{}\">", class).into_bytes()
    })?;

    Ok(format!("<pre><code class=\"language-rust\">{}</code></pre>",
        String::from_utf8(renderer.html)?))
}
```

## Complete Example: syntect + markdown-rs

```rust
use markdown::{to_mdast, to_html_with_options, Options, ParseOptions};
use markdown::mdast::{Node, Code};
use syntect::html::ClassedHTMLGenerator;
use syntect::parsing::SyntaxSet;

pub struct MarkdownRenderer {
    syntax_set: SyntaxSet,
}

impl MarkdownRenderer {
    pub fn new() -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
        }
    }

    pub fn render(&self, input: &str) -> String {
        let options = Options::gfm();
        let mut html = to_html_with_options(input, &options)
            .unwrap_or_else(|e| format!("<p>Error: {}</p>", e.reason));

        // Parse AST to find code blocks
        if let Ok(ast) = to_mdast(input, &ParseOptions::gfm()) {
            self.highlight_code_blocks(&ast, &mut html);
        }

        html
    }

    fn highlight_code_blocks(&self, node: &Node, html: &mut String) {
        if let Node::Code(Code { value, lang, .. }) = node {
            let lang = lang.as_deref().unwrap_or("text");

            // Original HTML that markdown-rs generated
            let original_code = self.escape_html(value);
            let original = format!("<pre><code class=\"language-{}\">{}</code></pre>",
                lang, original_code);

            // Highlighted version
            let highlighted = self.highlight(value, lang);

            *html = html.replace(&original, &highlighted);
        }

        if let Some(children) = node.children() {
            for child in children {
                self.highlight_code_blocks(child, html);
            }
        }
    }

    fn highlight(&self, code: &str, lang: &str) -> String {
        let syntax = self.syntax_set
            .find_syntax_by_token(lang)
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());

        let mut generator = ClassedHTMLGenerator::new_with_class_style(
            syntax,
            &self.syntax_set,
            syntect::html::ClassStyle::Spaced,
        );

        for line in code.lines() {
            let _ = generator.parse_html_for_line_which_includes_newline(
                &format!("{}\n", line)
            );
        }

        format!("<pre class=\"highlight\"><code class=\"language-{}\">{}</code></pre>",
            lang,
            generator.finalize()
        )
    }

    fn escape_html(&self, s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
    }
}

fn main() {
    let renderer = MarkdownRenderer::new();

    let markdown = r#"
# Code Example

Here's some Rust code:

```rust
fn main() {
    println!("Hello, world!");
}
```

And some Python:

```python
def hello():
    print("Hello, world!")
```
"#;

    let html = renderer.render(markdown);
    println!("{}", html);
}
```

## CSS for Highlighted Code

```css
/* Base code block styles */
pre.highlight {
    background: #282c34;
    padding: 1rem;
    border-radius: 4px;
    overflow-x: auto;
}

pre.highlight code {
    color: #abb2bf;
    font-family: 'Fira Code', monospace;
}

/* Syntect class-based highlighting */
.keyword { color: #c678dd; }
.function { color: #61afef; }
.type { color: #e5c07b; }
.string { color: #98c379; }
.comment { color: #5c6370; font-style: italic; }
.number { color: #d19a66; }
.operator { color: #56b6c2; }
```

## Recommendations

| Use Case | Recommended |
|----------|-------------|
| Quick integration, many languages | **syntect** |
| Modern theming (Neovim themes) | **autumnus** |
| Maximum accuracy, semantic highlighting | **tree-sitter** |
| Static site generation | **syntect** (best performance) |
| IDE-like experience | **tree-sitter** |
