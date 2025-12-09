# LSP Capabilities Reference

LSP capabilities are features your server advertises during initialization. Enable only what you implement.

## Server Capabilities

Announced in `InitializeResult::capabilities`:

```rust
ServerCapabilities {
    // Text synchronization
    text_document_sync: Some(TextDocumentSyncCapability::Kind(
        TextDocumentSyncKind::INCREMENTAL  // or FULL
    )),

    // Completion
    completion_provider: Some(CompletionOptions {
        trigger_characters: Some(vec!["[".into(), ".".into()]),
        resolve_provider: Some(true),
        ..Default::default()
    }),

    // Hover
    hover_provider: Some(HoverProviderCapability::Simple(true)),

    // Navigation
    definition_provider: Some(OneOf::Left(true)),
    references_provider: Some(OneOf::Left(true)),
    document_symbol_provider: Some(OneOf::Left(true)),

    // Diagnostics (push-based, no capability needed)

    ..Default::default()
}
```

## Text Document Sync

### Sync Kinds

| Kind | Behavior | Use When |
|:-----|:---------|:---------|
| `NONE` | No sync | Rare |
| `FULL` | Send entire document on change | Simple parsers |
| `INCREMENTAL` | Send only changes | Large documents, performance |

### Related Notifications

| Notification | Handler | Purpose |
|:-------------|:--------|:--------|
| `textDocument/didOpen` | `did_open()` | Document opened |
| `textDocument/didChange` | `did_change()` | Content changed |
| `textDocument/didClose` | `did_close()` | Document closed |
| `textDocument/didSave` | `did_save()` | Document saved |

### Incremental Updates

```rust
async fn did_change(&self, params: DidChangeTextDocumentParams) {
    for change in params.content_changes {
        match change.range {
            Some(range) => {
                // Incremental: apply change at range
                apply_edit(&mut doc, range, &change.text);
            }
            None => {
                // Full sync: replace entire content
                doc.set_text(&change.text);
            }
        }
    }
    // Re-parse and publish diagnostics
}
```

## Completion

### Implementation

```rust
async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
    let uri = params.text_document_position.text_document.uri;
    let pos = params.text_document_position.position;

    let items = vec![
        CompletionItem {
            label: "function".into(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some("Insert function template".into()),
            insert_text: Some("fn ${1:name}() {\n\t$0\n}".into()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
    ];

    Ok(Some(CompletionResponse::Array(items)))
}
```

### CompletionItemKind

Common kinds: `TEXT`, `METHOD`, `FUNCTION`, `CONSTRUCTOR`, `FIELD`, `VARIABLE`, `CLASS`, `INTERFACE`, `MODULE`, `PROPERTY`, `KEYWORD`, `SNIPPET`, `FILE`, `REFERENCE`

### Resolve Provider

For expensive completion details, use two-phase completion:

```rust
// In capabilities
completion_provider: Some(CompletionOptions {
    resolve_provider: Some(true),
    ..Default::default()
}),

// Initial completion (fast)
async fn completion(&self, _: CompletionParams) -> Result<Option<CompletionResponse>> {
    // Return items without full details
}

// Resolve details on demand
async fn completion_resolve(&self, item: CompletionItem) -> Result<CompletionItem> {
    // Add documentation, etc.
    Ok(CompletionItem {
        documentation: Some(Documentation::String("Full docs here".into())),
        ..item
    })
}
```

## Hover

```rust
async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
    let pos = params.text_document_position_params.position;

    // Find element at position, return info
    Ok(Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: "**Bold** and `code`".into(),
        }),
        range: Some(Range::new(pos, pos)), // Optional highlight range
    }))
}
```

## Diagnostics

Push-based via client notification (no capability needed):

```rust
async fn publish_errors(&self, uri: Url, errors: Vec<ParseError>) {
    let diagnostics: Vec<Diagnostic> = errors.iter().map(|e| {
        Diagnostic {
            range: e.range,
            severity: Some(DiagnosticSeverity::ERROR),
            code: Some(NumberOrString::String(e.code.clone())),
            source: Some("my-lsp".into()),
            message: e.message.clone(),
            related_information: None,
            tags: None,
            ..Default::default()
        }
    }).collect();

    self.client.publish_diagnostics(uri, diagnostics, None).await;
}
```

### Diagnostic Severity

`ERROR`, `WARNING`, `INFORMATION`, `HINT`

### Diagnostic Tags

- `UNNECESSARY` - Faded out (unused code)
- `DEPRECATED` - Strikethrough

## Go to Definition

```rust
async fn goto_definition(&self, params: GotoDefinitionParams) -> Result<Option<GotoDefinitionResponse>> {
    let target = Location {
        uri: Url::parse("file:///path/to/file.md")?,
        range: Range::new(Position::new(10, 0), Position::new(10, 20)),
    };
    Ok(Some(GotoDefinitionResponse::Scalar(target)))
}
```

## Document Symbols

For outline/breadcrumb navigation:

```rust
async fn document_symbol(&self, params: DocumentSymbolParams) -> Result<Option<DocumentSymbolResponse>> {
    let symbols = vec![
        DocumentSymbol {
            name: "Introduction".into(),
            kind: SymbolKind::STRING, // or MODULE, FUNCTION, etc.
            range: Range::new(Position::new(0, 0), Position::new(10, 0)),
            selection_range: Range::new(Position::new(0, 2), Position::new(0, 14)),
            children: Some(vec![/* nested symbols */]),
            ..Default::default()
        },
    ];
    Ok(Some(DocumentSymbolResponse::Nested(symbols)))
}
```

## Code Actions

Quick fixes, refactorings:

```rust
async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
    let actions = vec![
        CodeActionOrCommand::CodeAction(CodeAction {
            title: "Fix typo".into(),
            kind: Some(CodeActionKind::QUICKFIX),
            edit: Some(WorkspaceEdit {
                changes: Some(HashMap::from([(
                    params.text_document.uri.clone(),
                    vec![TextEdit {
                        range: Range::new(Position::new(5, 0), Position::new(5, 4)),
                        new_text: "fixed".into(),
                    }],
                )])),
                ..Default::default()
            }),
            ..Default::default()
        }),
    ];
    Ok(Some(actions))
}
```

## Full Capability List

| Capability | Method | Purpose |
|:-----------|:-------|:--------|
| `completionProvider` | `completion` | Autocomplete |
| `hoverProvider` | `hover` | Tooltips |
| `signatureHelpProvider` | `signature_help` | Function signatures |
| `definitionProvider` | `goto_definition` | Jump to definition |
| `typeDefinitionProvider` | `goto_type_definition` | Jump to type |
| `implementationProvider` | `goto_implementation` | Jump to impl |
| `referencesProvider` | `references` | Find all references |
| `documentSymbolProvider` | `document_symbol` | Document outline |
| `workspaceSymbolProvider` | `symbol` | Workspace search |
| `codeActionProvider` | `code_action` | Quick fixes |
| `codeLensProvider` | `code_lens` | Inline annotations |
| `documentFormattingProvider` | `formatting` | Format document |
| `renameProvider` | `rename` | Rename symbol |
| `foldingRangeProvider` | `folding_range` | Code folding |
| `selectionRangeProvider` | `selection_range` | Smart selection |
| `semanticTokensProvider` | `semantic_tokens_*` | Syntax highlighting |
| `inlayHintProvider` | `inlay_hint` | Inline hints |
