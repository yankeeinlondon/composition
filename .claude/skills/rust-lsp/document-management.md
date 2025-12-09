# Document Management

Efficient state management is critical for responsive LSP servers.

## Core Architecture

```rust
use dashmap::DashMap;
use ropey::Rope;
use std::sync::Arc;

pub struct DocumentManager {
    documents: DashMap<Url, DocumentState>,
}

pub struct DocumentState {
    pub content: Rope,           // Efficient text buffer
    pub version: i32,            // LSP document version
    pub ast: Option<ParsedAst>,  // Cached parse result
    pub diagnostics: Vec<Diagnostic>,
}

impl DocumentManager {
    pub fn new() -> Self {
        Self { documents: DashMap::new() }
    }

    pub fn open(&self, uri: Url, text: String, version: i32) {
        let content = Rope::from_str(&text);
        let ast = self.parse(&content);
        self.documents.insert(uri, DocumentState {
            content,
            version,
            ast: Some(ast),
            diagnostics: vec![],
        });
    }

    pub fn update(&self, uri: &Url, changes: Vec<TextDocumentContentChangeEvent>, version: i32) {
        if let Some(mut doc) = self.documents.get_mut(uri) {
            for change in changes {
                self.apply_change(&mut doc.content, change);
            }
            doc.version = version;
            doc.ast = Some(self.parse(&doc.content));
        }
    }

    pub fn close(&self, uri: &Url) {
        self.documents.remove(uri);
    }

    pub fn get(&self, uri: &Url) -> Option<dashmap::mapref::one::Ref<Url, DocumentState>> {
        self.documents.get(uri)
    }
}
```

## Text Rope (ropey)

`Rope` is far more efficient than `String` for large documents with frequent edits:

```rust
use ropey::Rope;

let mut rope = Rope::from_str("Hello, world!");

// Efficient operations
rope.insert(7, "beautiful ");  // Insert at byte offset
rope.remove(0..7);             // Remove range
let line = rope.line(0);       // Get line (O(log n))
let char_count = rope.len_chars();
let line_count = rope.len_lines();

// Position conversions
let byte_idx = rope.char_to_byte(5);      // char index → byte offset
let line_idx = rope.char_to_line(100);    // char index → line number
let line_start = rope.line_to_char(10);   // line number → char index
```

### LSP Position Conversion

LSP uses 0-based line/character positions. Convert between ropey and LSP:

```rust
use lsp_types::Position;

fn lsp_to_char_idx(rope: &Rope, pos: Position) -> usize {
    let line_start = rope.line_to_char(pos.line as usize);
    line_start + pos.character as usize
}

fn char_idx_to_lsp(rope: &Rope, idx: usize) -> Position {
    let line = rope.char_to_line(idx);
    let line_start = rope.line_to_char(line);
    Position::new(line as u32, (idx - line_start) as u32)
}

fn lsp_range_to_char_range(rope: &Rope, range: lsp_types::Range) -> std::ops::Range<usize> {
    let start = lsp_to_char_idx(rope, range.start);
    let end = lsp_to_char_idx(rope, range.end);
    start..end
}
```

## Applying Incremental Changes

```rust
fn apply_change(&self, rope: &mut Rope, change: TextDocumentContentChangeEvent) {
    match change.range {
        Some(range) => {
            // Incremental update
            let start = lsp_to_char_idx(rope, range.start);
            let end = lsp_to_char_idx(rope, range.end);
            rope.remove(start..end);
            rope.insert(start, &change.text);
        }
        None => {
            // Full document sync
            *rope = Rope::from_str(&change.text);
        }
    }
}
```

## Caching Strategy

### Parse Cache

Cache parse results to avoid re-parsing for every LSP request:

```rust
pub struct ParseCache {
    ast: RwLock<Option<ParsedAst>>,
    version: AtomicI32,
}

impl ParseCache {
    pub fn get_or_parse(&self, content: &Rope, current_version: i32) -> Arc<ParsedAst> {
        // Check if cache is valid
        if self.version.load(Ordering::SeqCst) == current_version {
            if let Some(ast) = self.ast.read().unwrap().as_ref() {
                return Arc::clone(ast);
            }
        }

        // Parse and cache
        let new_ast = Arc::new(parse(content));
        *self.ast.write().unwrap() = Some(Arc::clone(&new_ast));
        self.version.store(current_version, Ordering::SeqCst);
        new_ast
    }
}
```

### Symbol Cache

For large workspaces, cache symbol information:

```rust
pub struct WorkspaceCache {
    symbols: DashMap<Url, Vec<DocumentSymbol>>,
    definitions: DashMap<String, Location>,  // name → location
}
```

## Concurrent Access

LSP servers handle concurrent requests. Use appropriate synchronization:

```rust
// DashMap for document storage (concurrent reads/writes)
documents: DashMap<Url, DocumentState>

// RwLock for infrequent writes, frequent reads
config: RwLock<ServerConfig>

// Mutex for exclusive access
workspace_root: Mutex<Option<PathBuf>>
```

## Memory Management

### Cleanup on Close

```rust
async fn did_close(&self, params: DidCloseTextDocumentParams) {
    let uri = params.text_document.uri;
    self.document_manager.close(&uri);

    // Clear diagnostics
    self.client.publish_diagnostics(uri, vec![], None).await;
}
```

### Large File Handling

```rust
const MAX_FILE_SIZE: usize = 10 * 1024 * 1024; // 10MB

async fn did_open(&self, params: DidOpenTextDocumentParams) {
    if params.text_document.text.len() > MAX_FILE_SIZE {
        // Skip analysis for very large files
        return;
    }
    // Normal processing
}
```

## Version Tracking

LSP documents have versions. Track them to avoid stale updates:

```rust
async fn did_change(&self, params: DidChangeTextDocumentParams) {
    let uri = params.text_document.uri;
    let version = params.text_document.version;

    if let Some(doc) = self.documents.get(&uri) {
        if version <= doc.version {
            // Stale update, ignore
            return;
        }
    }

    // Apply changes
}
```

## Debouncing

For expensive operations (full re-parse, diagnostics), debounce rapid changes:

```rust
use tokio::time::{sleep, Duration};
use std::sync::atomic::{AtomicU64, Ordering};

struct DebouncedValidator {
    last_change: AtomicU64,
    delay_ms: u64,
}

impl DebouncedValidator {
    async fn schedule_validation(&self, uri: Url) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        self.last_change.store(now, Ordering::SeqCst);

        sleep(Duration::from_millis(self.delay_ms)).await;

        // Only validate if no newer changes
        if self.last_change.load(Ordering::SeqCst) == now {
            self.validate(uri).await;
        }
    }
}
```
