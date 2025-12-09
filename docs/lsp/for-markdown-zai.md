
---
_fixed: true
---

# Building an LSP Server for Extended Markdown with `pulldown-cmark`

To build a Language Server Protocol (LSP) server for Markdown with additional features using `pulldown-cmark`, you'll want to consider both the right crates and an appropriate architecture. Here's a comprehensive guide:

## Recommended Crates

### Core LSP Components

- **`tower-lsp`**: A robust LSP server implementation that handles the protocol details, connection management, and request/response handling
- **`lsp-types`**: Provides type definitions for all LSP protocol structures
- **`tokio`**: Async runtime needed for handling concurrent LSP requests

### Markdown Processing

- **`pulldown-cmark`**: The core Markdown parser - fast, memory-efficient, and event-based
- **`pulldown-cmark-to-cmark`**: Useful for converting back to Markdown after modifications
- **`rowan`**: For building a syntax tree from the parsed Markdown events, enabling efficient navigation and manipulation

### Text Manipulation and Positioning

- **`text-size`**: For handling text positions and ranges in a UTF-8 aware way
- **`ropey`**: Efficient text buffer implementation for large files with frequent modifications

### Serialization and Error Handling

- **`serde`**: For serializing/deserializing LSP messages
- **`serde_json`**: JSON support for the LSP protocol
- **`anyhow`**: For error handling throughout the application
- **`tracing`**: For structured logging

### Additional Utilities

- **`dashmap`**: Concurrent hashmap for caching document states
- **`once_cell`**: For thread-safe initialization of shared resources

## Architectural Approaches

### 1. Event-Based Architecture with Syntax Tree Layer

This architecture leverages `pulldown-cmark`'s event-based parsing while building a more structured representation:

```rust
// Core components
pub struct MarkdownLspServer {
    document_manager: Arc<DocumentManager>,
    feature_handlers: Vec<Box<dyn FeatureHandler>>,
}

pub struct DocumentManager {
    documents: Dashmap<Url, DocumentState>,
}

pub struct DocumentState {
    content: Rope,
    syntax_tree: SyntaxNode,
    version: i32,
    // Additional metadata
}

// Feature handling
pub trait FeatureHandler {
    fn handle_event(&mut self, event: &Event, context: &mut Context);
    fn provide_completions(&self, params: &CompletionParams) -> Option<Vec<CompletionItem>>;
    // Other LSP feature methods
}
```

**Benefits:**

- Leverages `pulldown-cmark`'s efficient event-based parsing
- Syntax tree enables efficient navigation for LSP features
- Clear separation of concerns with feature handlers

### 2. Incremental Parsing Architecture

For better performance with large documents:

```rust
pub struct IncrementalParser {
    base_tree: SyntaxNode,
    change_set: Vec<TextChange>,
}

impl IncrementalParser {
    pub fn reparse(&mut self, changes: &[TextChange]) -> Result<SyntaxNode, ParseError> {
        // Apply changes to the tree incrementally
        // Only reparse affected regions
    }
}
```

**Benefits:**

- Efficient handling of document changes
- Better performance for large files
- Reduced memory allocation

### 3. Plugin Architecture for Extensible Features

To easily add new Markdown features:

```rust
pub struct PluginManager {
    plugins: HashMap<String, Box<dyn Plugin>>,
}

pub trait Plugin {
    fn name(&self) -> &str;
    fn parse_extension(&self, input: &str) -> Result<Vec<Event>, ParseError>;
    fn provide_lsp_features(&self) -> Box<dyn LspFeatureProvider>;
}

// Example plugin for a custom syntax
pub struct MermaidPlugin;

impl Plugin for MermaidPlugin {
    fn name(&self) -> &str {
        "mermaid"
    }
    
    fn parse_extension(&self, input: &str) -> Result<Vec<Event>, ParseError> {
        // Parse Mermaid diagrams and convert to events
    }
    
    fn provide_lsp_features(&self) -> Box<dyn LspFeatureProvider> {
        Box::new(MermaidLspFeatures)
    }
}
```

**Benefits:**

- Easy to add new features without modifying core code
- Modular and maintainable
- Supports third-party extensions

### 4. Multi-Layered Processing Pipeline

A pipeline approach for processing Markdown:

```rust
pub struct ProcessingPipeline {
    stages: Vec<Box<dyn ProcessingStage>>,
}

pub trait ProcessingStage {
    fn process(&self, input: &[Event], context: &mut Context) -> Result<Vec<Event>, ProcessingError>;
}

// Example stages
pub struct MarkdownParsingStage;
pub struct CustomSyntaxStage;
pub struct LspFeatureExtractionStage;
```

**Benefits:**

- Clear separation of processing steps
- Easy to modify or add processing stages
- Each stage can focus on a specific concern

## Implementation Example

Here's a simplified example of how you might structure the core of your LSP server:

```rust
use tower_lsp::{LspService, Server};
use pulldown_cmark::{Parser, Event, Tag};
use rowan::{GreenNode, SyntaxNode};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    
    let service = LspService::new(|client| Backend {
        client,
        document_manager: Arc::new(DocumentManager::new()),
        feature_handlers: vec![
            Box::new(CompletionHandler::new()),
            Box::new(HoverHandler::new()),
            // Add your custom feature handlers here
        ],
    });
    
    Server::new(stdin, stdout, service).serve().await;
}

struct Backend {
    client: tower_lsp::Client,
    document_manager: Arc<DocumentManager>,
    feature_handlers: Vec<Box<dyn FeatureHandler>>,
}

#[tower_lsp::async_trait]
impl tower_lsp::LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult, InitializeError> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::Full)),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec!["[".to_string(), "`".to_string()]),
                    work_done_progress_options: Default::default(),
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                // Add other capabilities
                ..ServerCapabilities::default()
            },
            ..InitializeResult::default()
        })
    }
    
    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let text = params.text_document.text;
        
        // Parse the document
        let parser = Parser::new(&text);
        let events: Vec<Event> = parser.collect();
        
        // Build syntax tree
        let syntax_tree = build_syntax_tree(events);
        
        // Store in document manager
        self.document_manager.store_document(uri, text, syntax_tree);
    }
    
    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>, ResponseError> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        
        if let Some(document) = self.document_manager.get_document(&uri) {
            for handler in &self.feature_handlers {
                if let Some(completions) = handler.provide_completions(&params, &document) {
                    return Ok(Some(CompletionResponse::Array(completions)));
                }
            }
        }
        
        Ok(None)
    }
    
    // Implement other LSP methods...
}

fn build_syntax_tree(events: Vec<Event>) -> SyntaxNode {
    // Convert pulldown-cmark events to a rowan syntax tree
    // This is a simplified example - you'd need to implement the actual conversion
    let builder = rowan::GreenNodeBuilder::new();
    
    for event in events {
        match event {
            Event::Start(tag) => {
                // Start a new node based on the tag
                builder.start_node(SyntaxKind::from_tag(&tag));
            }
            Event::End(tag) => {
                // End the current node
                builder.finish_node();
            }
            Event::Text(text) => {
                // Add text token
                builder.token(SyntaxKind::Text, text.as_str());
            }
            // Handle other events...
        }
    }
    
    let green_node = builder.finish();
    SyntaxNode::new_root(green_node)
}
```

## Handling Custom Markdown Features

To extend Markdown with new features, you can:

1. **Pre-process the input** to convert custom syntax to standard Markdown
2. **Create custom event types** that extend `pulldown-cmark`'s events
3. **Build a custom parser** that combines `pulldown-cmark` with additional parsing logic

Here's an example of adding a custom "highlight" syntax:

```rust
use pulldown_cmark::{Event, Tag, Parser};

pub struct CustomParser<'a> {
    inner: Parser<'a>,
    buffer: Vec<Event<'a>>,
}

impl<'a> CustomParser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            inner: Parser::new(input),
            buffer: Vec::new(),
        }
    }
}

impl<'a> Iterator for CustomParser<'a> {
    type Item = Event<'a>;
    
    fn next(&mut self) -> Option<Self::Item> {
        // First check if we have buffered events
        if !self.buffer.is_empty() {
            return self.buffer.pop();
        }
        
        // Get next event from pulldown-cmark
        match self.inner.next()? {
            Event::Text(text) => {
                // Check for custom syntax in text
                if let Some((prefix, highlighted, suffix)) = parse_highlight(&text) {
                    // Emit events for custom syntax
                    self.buffer.push(Event::Text(suffix));
                    self.buffer.push(Event::End(Tag::Emphasis));
                    self.buffer.push(Event::Text(highlighted));
                    self.buffer.push(Event::Start(Tag::Emphasis));
                    self.buffer.push(Event::Text(prefix));
                    
                    // Return the first buffered event
                    self.buffer.pop()
                } else {
                    Some(Event::Text(text))
                }
            }
            event => Some(event),
        }
    }
}

fn parse_highlight(text: &str) -> Option<(&str, &str, &str)> {
    // Parse custom highlight syntax, e.g., ==highlighted text==
    // Returns (before, highlighted, after)
    // Implementation depends on your custom syntax
}
```

## Performance Considerations

1. **Incremental Parsing**: Only reparse changed parts of the document
2. **Caching**: Cache parsed results and LSP feature computations
3. **Lazy Computation**: Compute expensive features only when requested
4. **Parallel Processing**: Use Rayon for parallel processing of independent document parts

## Testing Strategy

1. **Unit Tests**: Test individual parsing components and feature handlers
2. **Integration Tests**: Test the complete LSP server with real editors
3. **Performance Tests**: Ensure the server remains responsive with large documents
4. **Specification Tests**: Test against the CommonMark specification and your custom extensions

This architecture provides a solid foundation for building a performant LSP server for extended Markdown using `pulldown-cmark`. The modular approach allows for easy extension and maintenance while leveraging the efficiency of `pulldown-cmark`'s parsing capabilities.
