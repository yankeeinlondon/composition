# Extending Markdown with Custom Syntax

## Strategy Overview

Two main approaches for adding custom syntax to a Markdown LSP:

1. **Plugin/Extension Point** (Recommended) - Use markdown-rs's built-in extension system
2. **Pre-processor** - Transform custom syntax before parsing

## Strategy 1: Plugin Architecture with markdown-rs

The cleanest method using `markdown-rs`'s ParseOptions for custom parsers.

### Why markdown-rs?

- Public, first-class AST (`markdown::mdast::Node`)
- Built-in extension points via `ParseOptions`
- High performance Rust implementation
- Designed for extensibility

### Architecture

```
Document Text
    |
    v
markdown-rs Parser
    |-- Standard Markdown parsers
    |-- Custom parser for [[wikilink]]
    |-- Custom parser for >!spoiler!<
    |-- Custom parser for @mention
    |
    v
Unified AST (standard + custom nodes)
    |
    v
LSP Feature Handlers
    |-- Completion (context-aware)
    |-- Diagnostics (validate custom syntax)
    |-- Hover (show custom info)
    +-- Navigation (go-to-definition)
```

### Example: Adding Wiki-Links

```rust
use markdown::{mdast::Node, ParseOptions};
use tower_lsp::lsp_types::*;

struct Backend {
    client: tower_lsp::Client,
    parse_options: ParseOptions<'static>,
}

impl Backend {
    fn new(client: tower_lsp::Client) -> Self {
        let mut parse_options = ParseOptions::gfm();

        // Register custom parsers here
        // parse_options.constructs.text = Some(Box::new(parse_wikilink));

        Self { client, parse_options }
    }

    fn parse_document(&self, text: &str) -> Option<Node> {
        markdown::to_mdast(text, &self.parse_options).ok()
    }
}
```

### Custom Parser Function Structure

```rust
// Conceptual structure for a custom [[wikilink]] parser
fn parse_wikilink(
    text: &str,
    position: usize,
) -> Option<(Node, usize)> {
    // 1. Check if we're at the start of a wikilink: `[[`
    if !text[position..].starts_with("[[") {
        return None;
    }

    // 2. Find closing `]]`
    let start = position + 2;
    if let Some(end) = text[start..].find("]]") {
        let link_text = &text[start..start + end];

        // 3. Create a Link node
        let node = Node::Link(markdown::mdast::Link {
            children: vec![Node::Text(markdown::mdast::Text {
                value: link_text.to_string(),
                position: None,
            })],
            url: format!("/wiki/{}", link_text.replace(' ', "_")),
            title: None,
            position: None, // Fill in actual positions
        });

        // Return node and consumed bytes
        return Some((node, start + end + 2));
    }

    None
}
```

### LSP Features for Custom Syntax

```rust
// Completion for wiki-links
async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
    let doc = self.get_document(&params.text_document_position.text_document.uri);
    let position = params.text_document_position.position;

    // Check if we're inside a wiki-link context
    if self.is_in_wikilink_context(&doc, position) {
        // Provide completions for known pages
        let pages = self.get_all_pages();
        let items: Vec<CompletionItem> = pages.iter()
            .map(|page| CompletionItem {
                label: page.name.clone(),
                kind: Some(CompletionItemKind::REFERENCE),
                detail: Some(page.path.clone()),
                ..Default::default()
            })
            .collect();

        return Ok(Some(CompletionResponse::Array(items)));
    }

    Ok(None)
}

// Go-to-definition for wiki-links
async fn goto_definition(&self, params: GotoDefinitionParams) -> Result<Option<GotoDefinitionResponse>> {
    let doc = self.get_document(&params.text_document_position_params.text_document.uri);
    let position = params.text_document_position_params.position;

    // Find wiki-link at position
    if let Some(link) = self.find_wikilink_at_position(&doc, position) {
        // Resolve to file
        if let Some(target_uri) = self.resolve_wikilink(&link) {
            return Ok(Some(GotoDefinitionResponse::Scalar(Location {
                uri: target_uri,
                range: Range::default(), // Start of file
            })));
        }
    }

    Ok(None)
}

// Diagnostics for broken links
fn validate_wikilinks(&self, doc: &Document) -> Vec<Diagnostic> {
    let mut diagnostics = vec![];

    for link in self.find_all_wikilinks(&doc.ast) {
        if !self.page_exists(&link.target) {
            diagnostics.push(Diagnostic {
                range: link.range,
                severity: Some(DiagnosticSeverity::WARNING),
                message: format!("Page '{}' does not exist", link.target),
                source: Some("markdown-lsp".into()),
                ..Default::default()
            });
        }
    }

    diagnostics
}
```

## Strategy 2: Pre-processor Approach

Works with any Markdown parser but requires source mapping.

### How It Works

1. Transform custom syntax to standard Markdown/HTML
2. Parse transformed text
3. Maintain source map for position translation

### Example: Spoiler Syntax

```rust
struct SourceMap {
    original_range: Range,
    transformed_range: Range,
}

fn preprocess_spoilers(text: &str) -> (String, Vec<SourceMap>) {
    let mut output = String::new();
    let mut source_maps = vec![];

    // Find >!spoiler text!< and convert to <details>
    // Track position mappings

    // Regex or manual parsing to find >!...!<
    // Replace with: <details><summary>Spoiler</summary>...</details>
    // Record original and new positions in source_maps

    (output, source_maps)
}

// In LSP handlers, translate positions
fn translate_position(&self, pos: Position, maps: &[SourceMap]) -> Position {
    for map in maps {
        if self.position_in_range(pos, &map.transformed_range) {
            // Translate back to original position
            return self.map_to_original(pos, map);
        }
    }
    pos
}
```

### Pros/Cons

**Pros:**
- Works with any parser (pulldown-cmark, comrak, etc.)
- No parser modifications needed

**Cons:**
- Source map management is error-prone
- Custom syntax becomes "invisible" to parser
- Multi-byte character handling is tricky
- Less semantic understanding of custom features

## Recommended Approach

For building a Markdown LSP with custom syntax:

1. **Use markdown-rs** as the core parser for its extension capabilities
2. **Define custom AST node types** for each syntax extension
3. **Implement custom parsers** that integrate with markdown-rs
4. **Keep source positions accurate** for all LSP features
5. **Test extensively** with real documents containing mixed syntax

### Full Architecture

```
tower-lsp Server
    |
    +-- Document Manager
    |       +-- Rope text buffer (ropey)
    |       +-- Cached AST per document
    |
    +-- Extended markdown-rs Parser
    |       +-- Standard CommonMark/GFM
    |       +-- Wiki-link parser
    |       +-- Custom block parser
    |       +-- Frontmatter parser
    |
    +-- Feature Handlers
            +-- Completion (context-aware for custom syntax)
            +-- Hover (custom documentation)
            +-- Diagnostics (validation rules)
            +-- Navigation (cross-file linking)
            +-- Code Actions (quick fixes)
```
