# Rust LSP Architecture

## Recommended Architecture Pattern

```
LSP Server (tower-lsp)
    |
    +-- Document Manager (state)
    |       |-- documents: DashMap<Url, DocumentState>
    |       +-- DocumentState { content: Rope, ast: Option<T>, version: i32 }
    |
    +-- Parser Layer (cached results)
    |       |-- Incremental parsing on didChange
    |       +-- Source position tracking
    |
    +-- Feature Handlers
            |-- CompletionHandler
            |-- HoverHandler
            |-- DiagnosticsHandler
            +-- NavigationHandler (go-to-def, references)
```

## Document Manager Pattern

```rust
use dashmap::DashMap;
use ropey::Rope;
use std::sync::Arc;

pub struct DocumentManager {
    documents: DashMap<lsp_types::Url, DocumentState>,
}

pub struct DocumentState {
    pub content: Rope,
    pub version: i32,
    pub ast: Option<SyntaxTree>, // Your parsed representation
    pub diagnostics: Vec<lsp_types::Diagnostic>,
}

impl DocumentManager {
    pub fn new() -> Self {
        Self { documents: DashMap::new() }
    }

    pub fn open(&self, uri: lsp_types::Url, text: String, version: i32) {
        let content = Rope::from_str(&text);
        let ast = self.parse(&content);
        self.documents.insert(uri, DocumentState {
            content,
            version,
            ast: Some(ast),
            diagnostics: vec![],
        });
    }

    pub fn update(&self, uri: &lsp_types::Url, changes: Vec<lsp_types::TextDocumentContentChangeEvent>) {
        if let Some(mut doc) = self.documents.get_mut(uri) {
            for change in changes {
                // Apply incremental changes to rope
                if let Some(range) = change.range {
                    let start = self.position_to_offset(&doc.content, range.start);
                    let end = self.position_to_offset(&doc.content, range.end);
                    doc.content.remove(start..end);
                    doc.content.insert(start, &change.text);
                } else {
                    // Full document sync
                    doc.content = Rope::from_str(&change.text);
                }
            }
            // Re-parse after changes
            doc.ast = Some(self.parse(&doc.content));
        }
    }

    fn position_to_offset(&self, rope: &Rope, pos: lsp_types::Position) -> usize {
        let line_start = rope.line_to_char(pos.line as usize);
        line_start + pos.character as usize
    }

    fn parse(&self, content: &Rope) -> SyntaxTree {
        // Your parsing logic here
        todo!()
    }
}
```

## Incremental Parsing Strategy

For large documents, avoid re-parsing everything on each keystroke:

```rust
pub struct IncrementalParser<T> {
    cache: HashMap<Url, CachedParse<T>>,
}

struct CachedParse<T> {
    tree: T,
    checksum: u64, // Hash of source text
}

impl<T> IncrementalParser<T> {
    pub fn parse_with_cache(
        &mut self,
        uri: &Url,
        content: &str,
        full_parse: impl Fn(&str) -> T,
    ) -> &T {
        let checksum = hash(content);

        let needs_reparse = self.cache.get(uri)
            .map(|c| c.checksum != checksum)
            .unwrap_or(true);

        if needs_reparse {
            let tree = full_parse(content);
            self.cache.insert(uri.clone(), CachedParse { tree, checksum });
        }

        &self.cache.get(uri).unwrap().tree
    }
}
```

## Feature Handler Trait

```rust
use tower_lsp::lsp_types::*;

pub trait FeatureHandler: Send + Sync {
    fn provide_completions(
        &self,
        doc: &DocumentState,
        position: Position,
    ) -> Option<Vec<CompletionItem>>;

    fn provide_hover(
        &self,
        doc: &DocumentState,
        position: Position,
    ) -> Option<Hover>;

    fn provide_diagnostics(
        &self,
        doc: &DocumentState,
    ) -> Vec<Diagnostic>;
}
```

## Concurrency Considerations

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

struct Backend {
    client: tower_lsp::Client,
    // Use Arc for shared ownership across async tasks
    document_manager: Arc<DocumentManager>,
    // Use RwLock for state that needs mutable access
    config: Arc<RwLock<ServerConfig>>,
}

// Implement Clone for backend if needed for multiple handlers
impl Backend {
    async fn update_config(&self, new_config: ServerConfig) {
        let mut config = self.config.write().await;
        *config = new_config;
    }
}
```

## Publishing Diagnostics

```rust
impl Backend {
    async fn validate_document(&self, uri: Url) {
        if let Some(doc) = self.document_manager.get(&uri) {
            let diagnostics = self.analyze(&doc);
            self.client.publish_diagnostics(uri, diagnostics, Some(doc.version)).await;
        }
    }

    fn analyze(&self, doc: &DocumentState) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];

        // Example: Check for issues in AST
        if let Some(ref ast) = doc.ast {
            for error in ast.errors() {
                diagnostics.push(Diagnostic {
                    range: error.range,
                    severity: Some(DiagnosticSeverity::ERROR),
                    message: error.message.clone(),
                    source: Some("my-lsp".into()),
                    ..Default::default()
                });
            }
        }

        diagnostics
    }
}
```

## Performance Tips

1. **Debounce validation** - Don't run diagnostics on every keystroke
2. **Background processing** - Use `tokio::spawn` for heavy analysis
3. **Lazy computation** - Only compute expensive features when requested
4. **Smart invalidation** - Track which documents need re-analysis
5. **Use ropey** - Much faster than String for incremental text updates
