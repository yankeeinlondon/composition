---
_fixed: true
---

# GitHub Flavored Markdown and pulldown-cmark

GitHub Flavored Markdown (GFM) is the formal Markdown dialect used on GitHub, defined as a **strict superset of CommonMark** with added features like tables, task lists, and strikethrough text. The Rust crate `pulldown-cmark` is a CommonMark parser that can handle many GFM features, but you must enable them individually through its `Options` struct.

## GFM Features and Corresponding pulldown-cmark Options

To work with GFM in `pulldown-cmark`, you enable extensions via bitflags in the `Options` struct. Below is a summary of key features:

| GFM Feature | Description | `pulldown-cmark` Option (from `Options`) | Notes |
| :--- | :--- | :--- | :--- |
| **Tables** | Create tables with pipes `\|` and hyphens `---`. | `ENABLE_TABLES` | Basic table structure is supported. |
| **Task Lists** | Create clickable checkboxes (`- [ ]`, `- [x]`). | `ENABLE_TASK_LISTS` | Parses into appropriate HTML. |
| **Strikethrough** | Text enclosed by two tildes (`~~text~~`). | `ENABLE_STRIKETHROUGH` | |
| **Autolinks** | URLs and email addresses auto-linked. | `ENABLE_AUTOLINK` | Part of CommonMark spec. |
| **Footnotes** | Add numbered footnote references and definitions. | `ENABLE_FOOTNOTES` | |
| **Heading IDs** | Automatically generate IDs for anchor links. | (Implicit) | Enabled when using the HTML renderer. |

> **Note**: `pulldown-cmark` does not have a single flag to enable "all GFM features." You must enable each extension you need. Also, some GFM features (like **emoji shortcodes** or certain **table formatting** nuances) are not natively supported and may require post-processing.

## How to Use GFM Extensions in Code

You enable features by setting flags in the `Options` struct before creating a parser. Here's a practical example:

```rust
use pulldown_cmark::{Parser, Options, html};

let markdown_input = "
## Task List
- [x] Write code
- [ ] Test features

## Table
| Header | Description |
|--------|-------------|
| Cell   | Content     |
";

// Enable specific GFM extensions
let mut options = Options::empty();
options.insert(Options::ENABLE_TABLES);
options.insert(Options::ENABLE_TASK_LISTS);
options.insert(Options::ENABLE_STRIKETHROUGH);

// Create parser with extended options
let parser = Parser::new_ext(markdown_input, options);

// Render to HTML
let mut html_output = String::new();
html::push_html(&mut html_output, parser);

println!("{}", html_output);
```

## Best Practices for Working with pulldown-cmark

- **Enable Only What You Need**: For performance, only insert the `Options` flags required by your Markdown content.
- **Handle Consecutive Text Events**: The parser may yield consecutive `Event::Text` chunks. For smoother processing, you can wrap the parser in a `TextMergeStream`.
- **Check Your Version**: New GFM-related features are added over time. For example, definition lists (`ENABLE_DEFINITION_LISTS`) were added in v0.13.0. Consult the [release notes](https://github.com/pulldown-cmark/pulldown-cmark/releases) for the latest features.
- **Performance**: When using the built-in HTML renderer, write to a buffered target like a `String` or `Vec<u8>` for better performance, as it performs many small writes.

## Important Considerations

1. **Not a Complete 1:1 Match**: `pulldown-cmark` aims for high compliance, but it may not reproduce every detail of GitHub's exact output, which involves additional post-processing.
2. **Historical Context**: Rust's documentation tool, `rustdoc`, switched to `pulldown-cmark` from an older parser. This caused minor breaking changes for some existing Markdown documents, highlighting that differences between Markdown parsers do exist. Your documents should follow the CommonMark standard for the most predictable results.

## Looking for a Specific Feature?

The development of `pulldown-cmark` is active. If you are looking for a specific GFM feature not mentioned here, such as support for **mathematical equations** or **wiki-style links**, you can check the [pulldown-cmark GitHub repository](https://github.com/pulldown-cmark/pulldown-cmark) for the latest discussions, open issues, or feature requests.
