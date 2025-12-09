# LSP Architecture Patterns

## Event-Based Architecture

Leverage pull-parser output (like pulldown-cmark) with a syntax tree layer:

```rust
pub struct LspServer {
    document_manager: Arc<DocumentManager>,
    feature_handlers: Vec<Box<dyn FeatureHandler>>,
}

pub struct DocumentState {
    content: Rope,
    syntax_tree: SyntaxNode,
    version: i32,
}

pub trait FeatureHandler: Send + Sync {
    fn handle_event(&self, event: &Event, context: &mut Context);
    fn provide_completions(&self, params: &CompletionParams, doc: &DocumentState) -> Option<Vec<CompletionItem>>;
    fn provide_hover(&self, params: &HoverParams, doc: &DocumentState) -> Option<Hover>;
}
```

**Benefits:**
- Efficient event-based parsing
- Syntax tree enables navigation for LSP features
- Clear separation with feature handlers

## Plugin Architecture

For extensible language features:

```rust
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;

    /// Custom parsing for plugin-specific syntax
    fn parse_extension(&self, input: &str) -> Vec<CustomEvent>;

    /// LSP feature implementations
    fn completions(&self, context: &Context) -> Vec<CompletionItem>;
    fn hover(&self, context: &Context) -> Option<Hover>;
    fn diagnostics(&self, context: &Context) -> Vec<Diagnostic>;
}

pub struct PluginManager {
    plugins: HashMap<String, Box<dyn Plugin>>,
}

impl PluginManager {
    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.insert(plugin.name().to_string(), plugin);
    }

    pub fn collect_completions(&self, ctx: &Context) -> Vec<CompletionItem> {
        self.plugins.values()
            .flat_map(|p| p.completions(ctx))
            .collect()
    }
}
```

**Example plugins:**
- `MermaidPlugin` - Diagram syntax
- `WikiLinkPlugin` - `[[wiki-style]]` links
- `FrontmatterPlugin` - YAML/TOML metadata
- `MathPlugin` - LaTeX equations

## Processing Pipeline

Multi-stage document processing:

```rust
pub trait ProcessingStage: Send + Sync {
    fn process(&self, input: &[Event], context: &mut Context) -> Vec<Event>;
}

pub struct Pipeline {
    stages: Vec<Box<dyn ProcessingStage>>,
}

impl Pipeline {
    pub fn run(&self, events: Vec<Event>) -> Vec<Event> {
        let mut context = Context::new();
        let mut current = events;

        for stage in &self.stages {
            current = stage.process(&current, &mut context);
        }
        current
    }
}

// Example stages
struct MarkdownParsingStage;      // pulldown-cmark → events
struct CustomSyntaxStage;          // Transform custom syntax
struct SymbolExtractionStage;      // Extract symbols for LSP
struct DiagnosticStage;            // Generate diagnostics
```

## Incremental Parsing

For large documents, only reparse changed regions:

```rust
pub struct IncrementalParser {
    base_tree: SyntaxNode,
    change_log: Vec<TextChange>,
}

impl IncrementalParser {
    pub fn apply_change(&mut self, change: TextChange) {
        // Find affected subtree
        let affected = self.find_affected_nodes(&change);

        // Reparse only affected region
        let new_nodes = self.reparse_region(affected, &change);

        // Splice into tree
        self.splice_tree(affected, new_nodes);
    }

    fn find_affected_nodes(&self, change: &TextChange) -> Range<usize> {
        // Find parent node containing change
        // Expand to nearest recoverable boundary
    }
}
```

**Benefits:**
- Efficient for large files
- Responsive on every keystroke
- Reduced memory allocation

## Workspace Architecture

For multi-file features (references, go-to-definition):

```rust
pub struct Workspace {
    root: PathBuf,
    documents: DashMap<PathBuf, DocumentState>,
    index: WorkspaceIndex,
}

pub struct WorkspaceIndex {
    // Symbol name → defining location
    definitions: DashMap<String, Location>,

    // Symbol name → all reference locations
    references: DashMap<String, Vec<Location>>,

    // File → symbols defined in it
    file_symbols: DashMap<PathBuf, Vec<Symbol>>,

    // For link resolution
    file_by_name: DashMap<String, PathBuf>,
}

impl Workspace {
    pub fn index_file(&self, path: &Path, content: &str) {
        let symbols = extract_symbols(content);

        for symbol in &symbols {
            self.index.definitions.insert(
                symbol.name.clone(),
                symbol.location.clone(),
            );
        }

        self.index.file_symbols.insert(path.to_path_buf(), symbols);
    }

    pub fn find_definition(&self, name: &str) -> Option<Location> {
        self.index.definitions.get(name).map(|r| r.clone())
    }

    pub fn find_references(&self, name: &str) -> Vec<Location> {
        self.index.references
            .get(name)
            .map(|r| r.clone())
            .unwrap_or_default()
    }
}
```

## Async Request Handling

Handle concurrent LSP requests efficiently:

```rust
#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        // Get document (concurrent read)
        let doc = self.documents.get(&params.text_document_position.text_document.uri);

        // Spawn analysis task if expensive
        let result = tokio::task::spawn_blocking(move || {
            compute_completions(&doc, params.text_document_position.position)
        }).await?;

        Ok(Some(CompletionResponse::Array(result)))
    }
}
```

## Cancellation Support

LSP supports request cancellation. Handle it gracefully:

```rust
use tokio::select;
use tokio_util::sync::CancellationToken;

async fn handle_with_cancellation<F, T>(
    token: CancellationToken,
    operation: F,
) -> Option<T>
where
    F: Future<Output = T>,
{
    select! {
        result = operation => Some(result),
        _ = token.cancelled() => None,
    }
}
```

## Diagnostic Batching

Avoid flooding the client with diagnostic updates:

```rust
pub struct DiagnosticPublisher {
    pending: DashMap<Url, Vec<Diagnostic>>,
    debounce_ms: u64,
}

impl DiagnosticPublisher {
    pub async fn queue(&self, uri: Url, diagnostics: Vec<Diagnostic>) {
        self.pending.insert(uri.clone(), diagnostics);

        // Debounce
        tokio::time::sleep(Duration::from_millis(self.debounce_ms)).await;

        // Publish if still latest
        if let Some((_, diags)) = self.pending.remove(&uri) {
            self.client.publish_diagnostics(uri, diags, None).await;
        }
    }
}
```

## Configuration

Handle LSP configuration requests:

```rust
#[derive(Deserialize, Default)]
pub struct ServerConfig {
    pub enable_diagnostics: bool,
    pub lint_on_save: bool,
    pub max_completions: usize,
}

impl LanguageServer for Backend {
    async fn did_change_configuration(&self, params: DidChangeConfigurationParams) {
        if let Ok(config) = serde_json::from_value::<ServerConfig>(params.settings) {
            *self.config.write().await = config;
        }
    }
}
```

## Testing Strategy

### Unit Tests
```rust
#[test]
fn test_symbol_extraction() {
    let markdown = "# Heading\n\nSome text.";
    let symbols = extract_symbols(markdown);
    assert_eq!(symbols.len(), 1);
    assert_eq!(symbols[0].name, "Heading");
}
```

### Integration Tests
```rust
#[tokio::test]
async fn test_completion() {
    let (service, _) = LspService::new(|client| Backend::new(client));

    // Initialize
    let init_result = service.initialize(InitializeParams::default()).await;
    assert!(init_result.is_ok());

    // Open document
    service.did_open(DidOpenTextDocumentParams { /* ... */ }).await;

    // Request completion
    let completions = service.completion(CompletionParams { /* ... */ }).await;
    assert!(completions.is_ok());
}
```

### LSP Protocol Tests
Use `lsp-test` crate or VS Code's test client for end-to-end testing.
