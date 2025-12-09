# LSP Integration with markdown-rs

Build Language Server Protocol implementations for Markdown editing using markdown-rs for parsing and analysis.

## Architecture Overview

```txt
┌─────────────────────────────────────────────────────────┐
│  Editor/IDE                                             │
│  (VS Code, Neovim, Emacs, etc.)                        │
└─────────────────────┬───────────────────────────────────┘
                      │ LSP Messages
┌─────────────────────▼───────────────────────────────────┐
│  LSP Server                                             │
│  ┌────────────────┐  ┌────────────────┐                │
│  │ Text Documents │  │ markdown-rs    │                │
│  │ Synchronization│  │ Parser         │                │
│  └────────┬───────┘  └────────┬───────┘                │
│           │                   │                         │
│  ┌────────▼───────────────────▼───────┐                │
│  │       Document Analysis            │                │
│  │  - Diagnostics                     │                │
│  │  - Completions                     │                │
│  │  - Symbols                         │                │
│  │  - Hover                           │                │
│  └────────────────────────────────────┘                │
└─────────────────────────────────────────────────────────┘
```

## Dependencies

```toml
[dependencies]
tower-lsp = "0.20"
tokio = { version = "1", features = ["full"] }
markdown = "1.0.0-alpha.21"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

## Basic LSP Server Structure

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use markdown::{to_mdast, ParseOptions, mdast::Node};

#[derive(Debug)]
struct MarkdownDocument {
    content: String,
    ast: Node,
    version: i32,
}

struct MarkdownLanguageServer {
    client: Client,
    documents: Arc<RwLock<HashMap<Url, MarkdownDocument>>>,
}

impl MarkdownLanguageServer {
    fn new(client: Client) -> Self {
        Self {
            client,
            documents: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn parse_document(&self, uri: &Url, content: &str, version: i32) {
        let options = ParseOptions::gfm();
        if let Ok(ast) = to_mdast(content, &options) {
            let doc = MarkdownDocument {
                content: content.to_string(),
                ast,
                version,
            };
            self.documents.write().await.insert(uri.clone(), doc);
        }
    }
}
```

## LSP Feature Implementations

### Text Document Synchronization

```rust
#[tower_lsp::async_trait]
impl LanguageServer for MarkdownLanguageServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions::default()),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let content = params.text_document.text;
        let version = params.text_document.version;

        self.parse_document(&uri, &content, version).await;
        self.publish_diagnostics(&uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let version = params.text_document.version;

        if let Some(change) = params.content_changes.into_iter().last() {
            self.parse_document(&uri, &change.text, version).await;
            self.publish_diagnostics(&uri).await;
        }
    }
}
```

### Document Symbols (Outline)

```rust
use markdown::mdast::{Heading, Text};

impl MarkdownLanguageServer {
    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let docs = self.documents.read().await;
        let doc = match docs.get(&params.text_document.uri) {
            Some(d) => d,
            None => return Ok(None),
        };

        let symbols = self.extract_symbols(&doc.ast, &doc.content);
        Ok(Some(DocumentSymbolResponse::Nested(symbols)))
    }

    fn extract_symbols(&self, node: &Node, content: &str) -> Vec<DocumentSymbol> {
        let mut symbols = Vec::new();

        self.walk_for_symbols(node, content, &mut symbols);
        symbols
    }

    fn walk_for_symbols(&self, node: &Node, content: &str, symbols: &mut Vec<DocumentSymbol>) {
        match node {
            Node::Heading(Heading { depth, children, position, .. }) => {
                let text = self.extract_text(children);
                let range = self.position_to_range(position.as_ref(), content);

                #[allow(deprecated)]
                symbols.push(DocumentSymbol {
                    name: text,
                    detail: Some(format!("H{}", depth)),
                    kind: SymbolKind::STRING,
                    tags: None,
                    deprecated: None,
                    range,
                    selection_range: range,
                    children: None,
                });
            }
            _ => {}
        }

        if let Some(children) = node.children() {
            for child in children {
                self.walk_for_symbols(child, content, symbols);
            }
        }
    }

    fn extract_text(&self, nodes: &[Node]) -> String {
        nodes.iter()
            .filter_map(|n| {
                if let Node::Text(Text { value, .. }) = n {
                    Some(value.as_str())
                } else {
                    None
                }
            })
            .collect()
    }

    fn position_to_range(&self, pos: Option<&markdown::unist::Position>, _content: &str) -> Range {
        match pos {
            Some(p) => Range {
                start: Position {
                    line: (p.start.line - 1) as u32,
                    character: (p.start.column - 1) as u32,
                },
                end: Position {
                    line: (p.end.line - 1) as u32,
                    character: (p.end.column - 1) as u32,
                },
            },
            None => Range::default(),
        }
    }
}
```

### Diagnostics

```rust
use markdown::mdast::{Link, Image};

impl MarkdownLanguageServer {
    async fn publish_diagnostics(&self, uri: &Url) {
        let docs = self.documents.read().await;
        let doc = match docs.get(uri) {
            Some(d) => d,
            None => return,
        };

        let diagnostics = self.generate_diagnostics(&doc.ast, &doc.content);

        self.client
            .publish_diagnostics(uri.clone(), diagnostics, Some(doc.version))
            .await;
    }

    fn generate_diagnostics(&self, node: &Node, content: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        self.walk_for_diagnostics(node, content, &mut diagnostics);
        diagnostics
    }

    fn walk_for_diagnostics(&self, node: &Node, content: &str, diagnostics: &mut Vec<Diagnostic>) {
        match node {
            Node::Link(Link { url, position, .. }) => {
                if url.is_empty() {
                    diagnostics.push(Diagnostic {
                        range: self.position_to_range(position.as_ref(), content),
                        severity: Some(DiagnosticSeverity::ERROR),
                        message: "Empty link URL".to_string(),
                        source: Some("markdown-lsp".to_string()),
                        ..Default::default()
                    });
                } else if !self.is_valid_url(url) {
                    diagnostics.push(Diagnostic {
                        range: self.position_to_range(position.as_ref(), content),
                        severity: Some(DiagnosticSeverity::WARNING),
                        message: format!("Possibly invalid URL: {}", url),
                        source: Some("markdown-lsp".to_string()),
                        ..Default::default()
                    });
                }
            }
            Node::Image(Image { url, alt, position, .. }) => {
                if alt.is_empty() {
                    diagnostics.push(Diagnostic {
                        range: self.position_to_range(position.as_ref(), content),
                        severity: Some(DiagnosticSeverity::INFORMATION),
                        message: "Image missing alt text".to_string(),
                        source: Some("markdown-lsp".to_string()),
                        ..Default::default()
                    });
                }
            }
            _ => {}
        }

        if let Some(children) = node.children() {
            for child in children {
                self.walk_for_diagnostics(child, content, diagnostics);
            }
        }
    }

    fn is_valid_url(&self, url: &str) -> bool {
        url.starts_with("http://")
            || url.starts_with("https://")
            || url.starts_with("mailto:")
            || url.starts_with("#")
            || url.starts_with("/")
            || url.starts_with("./")
            || url.starts_with("../")
    }
}
```

### Hover Information

```rust
impl MarkdownLanguageServer {
    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let docs = self.documents.read().await;
        let doc = match docs.get(&uri) {
            Some(d) => d,
            None => return Ok(None),
        };

        // Find node at position
        if let Some(hover_info) = self.find_hover_at_position(&doc.ast, &doc.content, &position) {
            return Ok(Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: hover_info,
                }),
                range: None,
            }));
        }

        Ok(None)
    }

    fn find_hover_at_position(&self, node: &Node, content: &str, pos: &Position) -> Option<String> {
        match node {
            Node::Link(Link { url, title, position, .. }) => {
                if self.position_contains(position.as_ref(), pos) {
                    let mut info = format!("**Link**: `{}`", url);
                    if let Some(t) = title {
                        info.push_str(&format!("\n\n*Title*: {}", t));
                    }
                    return Some(info);
                }
            }
            Node::Image(Image { url, alt, position, .. }) => {
                if self.position_contains(position.as_ref(), pos) {
                    return Some(format!("**Image**: `{}`\n\n*Alt*: {}", url, alt));
                }
            }
            Node::Code(markdown::mdast::Code { lang, position, .. }) => {
                if self.position_contains(position.as_ref(), pos) {
                    let lang = lang.as_deref().unwrap_or("plain text");
                    return Some(format!("**Code Block**: {}", lang));
                }
            }
            _ => {}
        }

        if let Some(children) = node.children() {
            for child in children {
                if let Some(info) = self.find_hover_at_position(child, content, pos) {
                    return Some(info);
                }
            }
        }

        None
    }

    fn position_contains(&self, node_pos: Option<&markdown::unist::Position>, cursor: &Position) -> bool {
        let Some(p) = node_pos else { return false };

        let start_line = (p.start.line - 1) as u32;
        let end_line = (p.end.line - 1) as u32;
        let start_col = (p.start.column - 1) as u32;
        let end_col = (p.end.column - 1) as u32;

        if cursor.line < start_line || cursor.line > end_line {
            return false;
        }
        if cursor.line == start_line && cursor.character < start_col {
            return false;
        }
        if cursor.line == end_line && cursor.character > end_col {
            return false;
        }
        true
    }
}
```

### Completions

```rust
impl MarkdownLanguageServer {
    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        let docs = self.documents.read().await;
        let doc = match docs.get(&uri) {
            Some(d) => d,
            None => return Ok(None),
        };

        // Get current line content
        let lines: Vec<&str> = doc.content.lines().collect();
        let current_line = lines.get(position.line as usize).unwrap_or(&"");
        let prefix = &current_line[..position.character as usize];

        let mut items = Vec::new();

        // Link completion after [text](
        if prefix.ends_with("](") {
            items.extend(self.get_heading_links(&doc.ast));
        }

        // Snippet completions
        if prefix.trim().is_empty() || prefix.ends_with(' ') {
            items.extend(self.get_snippet_completions());
        }

        if items.is_empty() {
            Ok(None)
        } else {
            Ok(Some(CompletionResponse::Array(items)))
        }
    }

    fn get_heading_links(&self, node: &Node) -> Vec<CompletionItem> {
        let mut items = Vec::new();

        fn walk(node: &Node, items: &mut Vec<CompletionItem>) {
            if let Node::Heading(Heading { children, .. }) = node {
                let text: String = children.iter()
                    .filter_map(|c| if let Node::Text(Text { value, .. }) = c { Some(value.as_str()) } else { None })
                    .collect();

                let slug = text.to_lowercase().replace(' ', "-");
                items.push(CompletionItem {
                    label: format!("#{}", slug),
                    kind: Some(CompletionItemKind::REFERENCE),
                    detail: Some(text),
                    ..Default::default()
                });
            }

            if let Some(children) = node.children() {
                for child in children {
                    walk(child, items);
                }
            }
        }

        walk(node, &mut items);
        items
    }

    fn get_snippet_completions(&self) -> Vec<CompletionItem> {
        vec![
            CompletionItem {
                label: "table".to_string(),
                kind: Some(CompletionItemKind::SNIPPET),
                insert_text: Some("| ${1:Header} | ${2:Header} |\n|----------|----------|\n| ${3:Cell} | ${4:Cell} |".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                detail: Some("Insert table".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "code".to_string(),
                kind: Some(CompletionItemKind::SNIPPET),
                insert_text: Some("```${1:language}\n${2:code}\n```".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                detail: Some("Insert code block".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "link".to_string(),
                kind: Some(CompletionItemKind::SNIPPET),
                insert_text: Some("[${1:text}](${2:url})".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                detail: Some("Insert link".to_string()),
                ..Default::default()
            },
        ]
    }
}
```

## Running the Server

```rust
#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(MarkdownLanguageServer::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}
```

## Performance Considerations

1. **Cache parsed ASTs** - avoid reparsing on every keystroke
2. **Debounce diagnostics** - don't publish on every change
3. **Use incremental parsing** if document changes are small
4. **Limit diagnostic scope** - only re-analyze changed sections

```rust
use std::time::Duration;
use tokio::time::sleep;

impl MarkdownLanguageServer {
    async fn debounced_publish_diagnostics(&self, uri: &Url) {
        // Wait for typing to settle
        sleep(Duration::from_millis(300)).await;
        self.publish_diagnostics(uri).await;
    }
}
```

## Best Practices

1. **Use position info from AST** - markdown-rs provides accurate positions
2. **Validate links and images** - common diagnostic checks
3. **Provide heading completions** - for internal links
4. **Support GFM features** - tables, task lists in completions
5. **Cache frequently accessed data** - document symbols, headings
