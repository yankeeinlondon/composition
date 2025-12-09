# pulldown-cmark for LSP Development

Guide for using pulldown-cmark as the parsing layer for a Language Server Protocol implementation.

## Why pulldown-cmark for LSP?

- **Source mapping**: `into_offset_iter()` provides byte ranges for every event
- **Streaming**: Process documents incrementally without building full AST
- **Zero-copy**: `CowStr` minimizes allocations
- **GFM support**: Tables, task lists, strikethrough, autolinks
- **Fast**: Suitable for real-time parsing on every keystroke

## Core Pattern: Source Position Tracking

The key to LSP integration is mapping parse events back to source positions:

```rust
use pulldown_cmark::{Parser, Event, Tag, Options};
use lsp_types::{Position, Range};
use ropey::Rope;

fn parse_with_positions(text: &str) -> Vec<(Event, std::ops::Range<usize>)> {
    let options = Options::all();
    Parser::new_ext(text, options)
        .into_offset_iter()
        .collect()
}

// Convert byte offset to LSP Position
fn byte_to_position(rope: &Rope, byte_offset: usize) -> Position {
    let char_idx = rope.byte_to_char(byte_offset);
    let line = rope.char_to_line(char_idx);
    let line_start = rope.line_to_char(line);
    Position::new(line as u32, (char_idx - line_start) as u32)
}

// Convert byte range to LSP Range
fn byte_range_to_lsp_range(rope: &Rope, range: std::ops::Range<usize>) -> Range {
    Range::new(
        byte_to_position(rope, range.start),
        byte_to_position(rope, range.end),
    )
}
```

## Extracting Document Structure

### Headings (for Document Symbols)

```rust
use pulldown_cmark::{Parser, Event, Tag, HeadingLevel, Options};
use lsp_types::{DocumentSymbol, SymbolKind};

pub struct Heading {
    pub level: u8,
    pub text: String,
    pub range: lsp_types::Range,
    pub selection_range: lsp_types::Range,
}

pub fn extract_headings(text: &str, rope: &Rope) -> Vec<Heading> {
    let mut headings = Vec::new();
    let mut current: Option<(HeadingLevel, std::ops::Range<usize>)> = None;
    let mut text_content = String::new();

    for (event, range) in Parser::new_ext(text, Options::all()).into_offset_iter() {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                current = Some((level, range));
                text_content.clear();
            }
            Event::Text(t) if current.is_some() => {
                text_content.push_str(&t);
            }
            Event::End(pulldown_cmark::TagEnd::Heading(_)) if current.is_some() => {
                let (level, start_range) = current.take().unwrap();
                headings.push(Heading {
                    level: level as u8,
                    text: text_content.clone(),
                    range: byte_range_to_lsp_range(rope, start_range.start..range.end),
                    selection_range: byte_range_to_lsp_range(rope, start_range.clone()),
                });
            }
            _ => {}
        }
    }
    headings
}

// Convert to LSP DocumentSymbol
pub fn headings_to_symbols(headings: Vec<Heading>) -> Vec<DocumentSymbol> {
    headings.into_iter().map(|h| DocumentSymbol {
        name: h.text,
        kind: SymbolKind::STRING,
        range: h.range,
        selection_range: h.selection_range,
        children: None,
        detail: Some(format!("H{}", h.level)),
        tags: None,
        deprecated: None,
    }).collect()
}
```

### Links (for Go-to-Definition, References)

```rust
use pulldown_cmark::{Event, Tag, LinkType};

pub struct Link {
    pub text: String,
    pub url: String,
    pub link_type: LinkType,
    pub range: lsp_types::Range,
}

pub fn extract_links(text: &str, rope: &Rope) -> Vec<Link> {
    let mut links = Vec::new();
    let mut current_link: Option<(String, LinkType, std::ops::Range<usize>)> = None;
    let mut link_text = String::new();

    for (event, range) in Parser::new_ext(text, Options::all()).into_offset_iter() {
        match event {
            Event::Start(Tag::Link { link_type, dest_url, .. }) => {
                current_link = Some((dest_url.to_string(), link_type, range));
                link_text.clear();
            }
            Event::Text(t) if current_link.is_some() => {
                link_text.push_str(&t);
            }
            Event::End(pulldown_cmark::TagEnd::Link) if current_link.is_some() => {
                let (url, link_type, start_range) = current_link.take().unwrap();
                links.push(Link {
                    text: link_text.clone(),
                    url,
                    link_type,
                    range: byte_range_to_lsp_range(rope, start_range.start..range.end),
                });
            }
            _ => {}
        }
    }
    links
}
```

### Code Blocks (for Syntax Highlighting, Diagnostics)

```rust
use pulldown_cmark::{CodeBlockKind, Event, Tag};

pub struct CodeBlock {
    pub language: Option<String>,
    pub content: String,
    pub range: lsp_types::Range,
}

pub fn extract_code_blocks(text: &str, rope: &Rope) -> Vec<CodeBlock> {
    let mut blocks = Vec::new();
    let mut current: Option<(Option<String>, std::ops::Range<usize>)> = None;
    let mut content = String::new();

    for (event, range) in Parser::new_ext(text, Options::all()).into_offset_iter() {
        match event {
            Event::Start(Tag::CodeBlock(kind)) => {
                let lang = match kind {
                    CodeBlockKind::Fenced(info) => {
                        let info = info.split_whitespace().next().unwrap_or("");
                        if info.is_empty() { None } else { Some(info.to_string()) }
                    }
                    CodeBlockKind::Indented => None,
                };
                current = Some((lang, range));
                content.clear();
            }
            Event::Text(t) if current.is_some() => {
                content.push_str(&t);
            }
            Event::End(pulldown_cmark::TagEnd::CodeBlock) if current.is_some() => {
                let (language, start_range) = current.take().unwrap();
                blocks.push(CodeBlock {
                    language,
                    content: content.clone(),
                    range: byte_range_to_lsp_range(rope, start_range.start..range.end),
                });
            }
            _ => {}
        }
    }
    blocks
}
```

## LSP Feature Implementations

### Document Symbols

```rust
async fn document_symbol(&self, params: DocumentSymbolParams) -> Result<Option<DocumentSymbolResponse>> {
    let uri = params.text_document.uri;
    let doc = self.documents.get(&uri)?;

    let headings = extract_headings(&doc.text, &doc.rope);
    let symbols = headings_to_symbols(headings);

    Ok(Some(DocumentSymbolResponse::Nested(symbols)))
}
```

### Hover (Link Preview)

```rust
async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
    let uri = params.text_document_position_params.text_document.uri;
    let pos = params.text_document_position_params.position;
    let doc = self.documents.get(&uri)?;

    let links = extract_links(&doc.text, &doc.rope);

    // Find link at cursor position
    for link in links {
        if contains_position(&link.range, &pos) {
            return Ok(Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!("**Link:** [{}]({})", link.text, link.url),
                }),
                range: Some(link.range),
            }));
        }
    }
    Ok(None)
}

fn contains_position(range: &Range, pos: &Position) -> bool {
    (range.start.line < pos.line || (range.start.line == pos.line && range.start.character <= pos.character))
        && (range.end.line > pos.line || (range.end.line == pos.line && range.end.character >= pos.character))
}
```

### Diagnostics (Broken Links)

```rust
async fn validate_links(&self, uri: Url, doc: &DocumentState) {
    let links = extract_links(&doc.text, &doc.rope);
    let mut diagnostics = Vec::new();

    for link in links {
        if link.url.starts_with("./") || link.url.starts_with("../") {
            // Check if relative link target exists
            let target = resolve_relative_path(&uri, &link.url);
            if !target.exists() {
                diagnostics.push(Diagnostic {
                    range: link.range,
                    severity: Some(DiagnosticSeverity::WARNING),
                    message: format!("Broken link: {} not found", link.url),
                    source: Some("markdown-lsp".into()),
                    ..Default::default()
                });
            }
        }
    }

    self.client.publish_diagnostics(uri, diagnostics, None).await;
}
```

### Completion (Link Targets)

```rust
async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
    let uri = params.text_document_position.text_document.uri;
    let pos = params.text_document_position.position;
    let doc = self.documents.get(&uri)?;

    // Check if we're in a link context
    let line = doc.rope.line(pos.line as usize);
    let line_str: String = line.chars().collect();
    let char_pos = pos.character as usize;

    // Simple check: inside [...]( or [[
    if line_str[..char_pos].contains("](") || line_str[..char_pos].contains("[[") {
        let files = self.workspace.list_markdown_files();
        let items: Vec<CompletionItem> = files.iter().map(|f| {
            CompletionItem {
                label: f.file_name().unwrap().to_string_lossy().into(),
                kind: Some(CompletionItemKind::FILE),
                ..Default::default()
            }
        }).collect();
        return Ok(Some(CompletionResponse::Array(items)));
    }

    Ok(None)
}
```

## Caching Parsed Results

Re-parsing on every LSP request is expensive. Cache the parsed structure:

```rust
pub struct CachedDocument {
    pub text: String,
    pub rope: Rope,
    pub version: i32,
    // Cached parse results
    pub headings: Vec<Heading>,
    pub links: Vec<Link>,
    pub code_blocks: Vec<CodeBlock>,
}

impl CachedDocument {
    pub fn new(text: String, version: i32) -> Self {
        let rope = Rope::from_str(&text);
        let headings = extract_headings(&text, &rope);
        let links = extract_links(&text, &rope);
        let code_blocks = extract_code_blocks(&text, &rope);

        Self { text, rope, version, headings, links, code_blocks }
    }

    pub fn update(&mut self, text: String, version: i32) {
        self.text = text;
        self.rope = Rope::from_str(&self.text);
        self.version = version;
        // Re-extract
        self.headings = extract_headings(&self.text, &self.rope);
        self.links = extract_links(&self.text, &self.rope);
        self.code_blocks = extract_code_blocks(&self.text, &self.rope);
    }
}
```

## Related Skills

- [rust-lsp](../rust-lsp/SKILL.md) - Full LSP development guide
- [rust-lsp/markdown-lsps](../rust-lsp/markdown-lsps.md) - Existing Rust Markdown LSPs
- [rust-lsp/document-management](../rust-lsp/document-management.md) - Text rope and state management
