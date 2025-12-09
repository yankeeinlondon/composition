# TypeScript LSP Implementation

## TypeScript Language Servers

### typescript-language-server (Community)
Community-maintained LSP wrapper around TypeScript's tsserver.

```bash
npm install -g typescript-language-server typescript
```

**Features:**
- Code actions on save (organize imports, fix all)
- Workspace commands (go to source definition, refactoring)
- Code lenses and inlay hints
- TypeScript version notification

### vtsls (Recommended for VS Code Parity)
LSP wrapper around VS Code's TypeScript extension with minimal patches.

```bash
npm install -g @vtsls/language-server
```

**Features:**
- Nearly identical to VS Code's TypeScript experience
- TypeScript version switching
- Advanced refactoring (move to file)
- Update paths on rename

### Comparison

| Feature | typescript-language-server | vtsls |
|---------|---------------------------|-------|
| VS Code Feature Parity | Good | Excellent |
| Performance | Excellent | Very Good |
| TypeScript Version Mgmt | Manual | Automatic |
| Advanced Refactoring | Basic | Advanced |

## Building a TypeScript LSP Server

Use the `vscode-languageserver` npm package:

```bash
npm install vscode-languageserver vscode-languageserver-textdocument
```

### Basic Server Structure

```typescript
import {
  createConnection,
  TextDocuments,
  ProposedFeatures,
  InitializeParams,
  InitializeResult,
  TextDocumentSyncKind,
  CompletionItem,
  CompletionItemKind,
} from 'vscode-languageserver/node';
import { TextDocument } from 'vscode-languageserver-textdocument';

// Create connection using Node's IPC or stdio
const connection = createConnection(ProposedFeatures.all);

// Document manager
const documents: TextDocuments<TextDocument> = new TextDocuments(TextDocument);

connection.onInitialize((params: InitializeParams): InitializeResult => {
  return {
    capabilities: {
      textDocumentSync: TextDocumentSyncKind.Incremental,
      completionProvider: {
        resolveProvider: true,
        triggerCharacters: ['.', '"', "'", '/'],
      },
      hoverProvider: true,
      definitionProvider: true,
    },
  };
});

// Completion handler
connection.onCompletion((_textDocumentPosition) => {
  return [
    {
      label: 'TypeScript',
      kind: CompletionItemKind.Text,
      data: 1,
    },
    {
      label: 'JavaScript',
      kind: CompletionItemKind.Text,
      data: 2,
    },
  ];
});

// Hover handler
connection.onHover(({ textDocument, position }) => {
  const doc = documents.get(textDocument.uri);
  if (!doc) return null;

  return {
    contents: {
      kind: 'markdown',
      value: '**Hover information**\n\nDetails about the symbol.',
    },
  };
});

// Listen for document changes
documents.onDidChangeContent((change) => {
  validateTextDocument(change.document);
});

async function validateTextDocument(textDocument: TextDocument): Promise<void> {
  const diagnostics: Diagnostic[] = [];

  // Your validation logic here

  connection.sendDiagnostics({ uri: textDocument.uri, diagnostics });
}

// Start listening
documents.listen(connection);
connection.listen();
```

### VS Code Extension Client

```typescript
import * as path from 'path';
import { workspace, ExtensionContext } from 'vscode';
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from 'vscode-languageclient/node';

let client: LanguageClient;

export function activate(context: ExtensionContext) {
  const serverModule = context.asAbsolutePath(
    path.join('server', 'out', 'server.js')
  );

  const serverOptions: ServerOptions = {
    run: { module: serverModule, transport: TransportKind.ipc },
    debug: { module: serverModule, transport: TransportKind.ipc },
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: 'file', language: 'myLanguage' }],
    synchronize: {
      fileEvents: workspace.createFileSystemWatcher('**/*.mylang'),
    },
  };

  client = new LanguageClient(
    'myLanguageServer',
    'My Language Server',
    serverOptions,
    clientOptions
  );

  client.start();
}

export function deactivate(): Thenable<void> | undefined {
  return client?.stop();
}
```

## Web Language Servers

### HTML: vscode-html-language-server
```bash
npm install -g vscode-langservers-extracted
```

### CSS: vscode-css-language-server
Included in `vscode-langservers-extracted`. Supports CSS, LESS, SCSS.

### Markdown: marksman
```bash
# Via package managers or binary releases
brew install marksman  # macOS
```

**Features:**
- Wiki-style link completion
- Go to definition for links
- Document symbols/outline

## Neovim Configuration (vtsls)

```lua
vim.lsp.enable('vtsls')
vim.lsp.config('vtsls', {
  cmd = {'vtsls', '--stdio'},
  filetypes = {'javascript', 'typescript', 'typescriptreact'},
  root_dir = vim.fs.root(0, {'package.json', '.git'}),
  settings = {
    vtsls = {
      enableMoveToFileCodeAction = true,
      autoUseWorkspaceTsdk = true,
    },
    typescript = {
      updateImportsOnFileMove = { enabled = "always" },
      inlayHints = {
        parameterNames = { enabled = "literals" },
        functionLikeReturnTypes = { enabled = true },
      },
    },
  },
})
```

## Helix Configuration

```toml
# ~/.config/helix/languages.toml
[language-server.vtsls]
command = "vtsls"
args = ["--stdio"]

[[language]]
name = "typescript"
language-servers = ["vtsls"]

[[language]]
name = "tsx"
language-servers = ["vtsls"]
```
