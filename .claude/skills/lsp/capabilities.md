# LSP Capabilities

## Core Capabilities Overview

| Capability | Method | Description |
|-----------|--------|-------------|
| Completion | `textDocument/completion` | Code completion suggestions |
| Hover | `textDocument/hover` | Information on hover |
| Definition | `textDocument/definition` | Go to definition |
| References | `textDocument/references` | Find all references |
| Diagnostics | `textDocument/publishDiagnostics` | Errors, warnings, hints |
| Formatting | `textDocument/formatting` | Format document |
| Code Actions | `textDocument/codeAction` | Quick fixes, refactoring |
| Symbols | `textDocument/documentSymbol` | Outline/structure |

## Text Document Synchronization

The foundation of all LSP features. Clients notify servers of document changes.

### Sync Kinds
```rust
// Announce in initialize response
text_document_sync: Some(TextDocumentSyncCapability::Kind(
    TextDocumentSyncKind::INCREMENTAL  // Recommended: only changes sent
    // TextDocumentSyncKind::FULL      // Full text on every change
))
```

### Document Lifecycle
```rust
// Handle these notifications
async fn did_open(&self, params: DidOpenTextDocumentParams) {
    // Document opened - store initial content
}

async fn did_change(&self, params: DidChangeTextDocumentParams) {
    // Document changed - apply incremental changes
}

async fn did_close(&self, params: DidCloseTextDocumentParams) {
    // Document closed - cleanup
}
```

## Completion

```rust
async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
    let uri = params.text_document_position.text_document.uri;
    let position = params.text_document_position.position;

    let items = vec![
        CompletionItem {
            label: "myFunction".into(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some("fn myFunction() -> bool".into()),
            documentation: Some(Documentation::String("Does something useful".into())),
            insert_text: Some("myFunction()".into()),
            insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
            ..Default::default()
        },
        // Snippet completion
        CompletionItem {
            label: "for loop".into(),
            kind: Some(CompletionItemKind::SNIPPET),
            insert_text: Some("for ${1:item} in ${2:items} {\n\t$0\n}".into()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
    ];

    Ok(Some(CompletionResponse::Array(items)))
}
```

### Trigger Characters
```rust
completion_provider: Some(CompletionOptions {
    trigger_characters: Some(vec![".".into(), "::".into(), "/".into()]),
    resolve_provider: Some(true), // Enable completion/resolve for lazy loading
    ..Default::default()
})
```

## Hover

```rust
async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
    let uri = params.text_document_position_params.text_document.uri;
    let position = params.text_document_position_params.position;

    // Find symbol at position, return info
    Ok(Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: "**myFunction**\n\n```rust\nfn myFunction() -> bool\n```\n\nReturns true if successful.".into(),
        }),
        range: Some(Range {
            start: Position { line: 5, character: 0 },
            end: Position { line: 5, character: 10 },
        }),
    }))
}
```

## Diagnostics

Push-based: server sends diagnostics proactively.

```rust
async fn publish_diagnostics(&self, uri: Url, text: &str) {
    let diagnostics = vec![
        Diagnostic {
            range: Range {
                start: Position { line: 10, character: 5 },
                end: Position { line: 10, character: 15 },
            },
            severity: Some(DiagnosticSeverity::ERROR),
            code: Some(NumberOrString::String("E001".into())),
            source: Some("my-lsp".into()),
            message: "Undefined variable 'foo'".into(),
            related_information: Some(vec![
                DiagnosticRelatedInformation {
                    location: Location {
                        uri: uri.clone(),
                        range: Range { /* ... */ },
                    },
                    message: "Did you mean 'bar'?".into(),
                }
            ]),
            ..Default::default()
        },
    ];

    self.client.publish_diagnostics(uri, diagnostics, None).await;
}
```

### Severity Levels
- `ERROR` - Must be fixed
- `WARNING` - Should be addressed
- `INFORMATION` - FYI
- `HINT` - Suggestion

## Go to Definition

```rust
async fn goto_definition(
    &self,
    params: GotoDefinitionParams,
) -> Result<Option<GotoDefinitionResponse>> {
    let uri = params.text_document_position_params.text_document.uri;
    let position = params.text_document_position_params.position;

    // Single location
    Ok(Some(GotoDefinitionResponse::Scalar(Location {
        uri: Url::parse("file:///path/to/definition.rs").unwrap(),
        range: Range {
            start: Position { line: 42, character: 0 },
            end: Position { line: 42, character: 20 },
        },
    })))

    // Or multiple locations
    // Ok(Some(GotoDefinitionResponse::Array(vec![...])))
}
```

## Find References

```rust
async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
    let include_declaration = params.context.include_declaration;

    let locations = vec![
        Location {
            uri: params.text_document_position.text_document.uri.clone(),
            range: Range { /* ... */ },
        },
        // More locations...
    ];

    Ok(Some(locations))
}
```

## Document Symbols (Outline)

```rust
async fn document_symbol(
    &self,
    params: DocumentSymbolParams,
) -> Result<Option<DocumentSymbolResponse>> {
    let symbols = vec![
        DocumentSymbol {
            name: "MyClass".into(),
            kind: SymbolKind::CLASS,
            range: Range { /* full range */ },
            selection_range: Range { /* name range */ },
            children: Some(vec![
                DocumentSymbol {
                    name: "myMethod".into(),
                    kind: SymbolKind::METHOD,
                    range: Range { /* ... */ },
                    selection_range: Range { /* ... */ },
                    children: None,
                    ..Default::default()
                },
            ]),
            ..Default::default()
        },
    ];

    Ok(Some(DocumentSymbolResponse::Nested(symbols)))
}
```

## Code Actions

Quick fixes, refactoring, source actions.

```rust
async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
    let mut actions = vec![];

    // Quick fix for a diagnostic
    if let Some(diag) = params.context.diagnostics.first() {
        actions.push(CodeActionOrCommand::CodeAction(CodeAction {
            title: "Fix: Add missing import".into(),
            kind: Some(CodeActionKind::QUICKFIX),
            diagnostics: Some(vec![diag.clone()]),
            edit: Some(WorkspaceEdit {
                changes: Some(HashMap::from([(
                    params.text_document.uri.clone(),
                    vec![TextEdit {
                        range: Range {
                            start: Position { line: 0, character: 0 },
                            end: Position { line: 0, character: 0 },
                        },
                        new_text: "import { foo } from 'bar';\n".into(),
                    }],
                )])),
                ..Default::default()
            }),
            ..Default::default()
        }));
    }

    Ok(Some(actions))
}
```

## Formatting

```rust
async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
    let uri = params.text_document.uri;
    let options = params.options; // tab_size, insert_spaces, etc.

    // Return edits to format the document
    Ok(Some(vec![
        TextEdit {
            range: Range { /* ... */ },
            new_text: "formatted code".into(),
        },
    ]))
}
```
