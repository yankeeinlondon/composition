# Markdown Language Servers

## Rust Implementations

### Markdown-Oxide
PKM-focused LSP for interconnected Markdown notes.

**Repository:** [Feel-ix-343/markdown-oxide](https://github.com/Feel-ix-343/markdown-oxide)

**Features:**
- Wiki-style link completion (`[[Page Name]]`)
- Cross-reference validation and diagnostics
- Go-to-definition for links
- Backlink discovery
- Works with Obsidian, Logseq-style vaults

**Best For:** Personal knowledge management, note-taking systems

**Installation:**
```bash
cargo install markdown-oxide
```

### Marksman
General-purpose Markdown LSP with wiki-link support.

**Repository:** [artempyanykh/marksman](https://github.com/artempyanykh/marksman)

**Features:**
- Document symbols/outline
- Wiki-style and standard link completion
- Go-to-definition for links
- Find references
- Table of contents

**Best For:** Documentation, general Markdown editing

**Installation:**
```bash
brew install marksman  # macOS
# Or download binary from releases
```

### Quickmark
Markdown linter with LSP support.

**Features:**
- Diagnostics (lint errors/warnings)
- CommonMark validation
- VS Code and JetBrains integrations

**Architecture:**
- `quickmark-core` - Linting engine
- `quickmark-cli` - Command line
- `quickmark-server` - LSP server

### mdBook-LS
Specialized for mdBook documentation projects.

**Features:**
- Live preview integration
- mdBook-aware navigation
- SUMMARY.md understanding
- Chapter-specific features

### Obsidian-LSP
Specifically for Obsidian vault workflows.

**Features:**
- Obsidian-flavored Markdown support
- Vault structure awareness
- Wiki-link handling

**Installation:**
```bash
cargo install obsidian-lsp
```

## Node.js Implementations

### remark-language-server
Built on the unified/remark ecosystem.

**Installation:**
```bash
npm install -g remark-language-server
```

### VS Code Markdown Language Service
Powers VS Code's built-in Markdown support.

**Features:**
- Document outlines
- Workspace symbols
- Document links
- Smart folding
- Preview capabilities

## Comparison

| Server | Focus | Links | Diagnostics | PKM Features |
|--------|-------|-------|-------------|--------------|
| markdown-oxide | PKM | Wiki + Standard | Yes | Backlinks, Graph |
| marksman | General | Wiki + Standard | Basic | Some |
| quickmark | Linting | N/A | Extensive | No |
| mdBook-LS | mdBook | Standard | Book-specific | No |
| remark-ls | General | Standard | Plugin-based | No |

## Editor Setup

### Neovim with Marksman
```lua
vim.lsp.enable('marksman')
vim.lsp.config('marksman', {
  cmd = {'marksman', 'server'},
  filetypes = {'markdown'},
  root_dir = vim.fs.root(0, {'.git', '.marksman.toml'}),
})
```

### Helix with Markdown-Oxide
```toml
# ~/.config/helix/languages.toml
[language-server.markdown-oxide]
command = "markdown-oxide"

[[language]]
name = "markdown"
language-servers = ["markdown-oxide"]
```

### VS Code
Most Markdown LSPs have dedicated VS Code extensions. Search the marketplace for:
- "Markdown Oxide"
- "Marksman"

## Building a Custom Markdown LSP

For custom Markdown dialects, consider these parsing crates:

### pulldown-cmark
Fast CommonMark parser with source positions.

```rust
use pulldown_cmark::{Parser, Event, Options};

let options = Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH;
let parser = Parser::new_ext(text, options);

// Get events with positions
for (event, range) in parser.into_offset_iter() {
    match event {
        Event::Start(tag) => { /* ... */ }
        Event::End(tag) => { /* ... */ }
        Event::Text(text) => { /* ... */ }
        _ => {}
    }
}
```

### markdown-rs
AST-based parser with extension points.

```rust
use markdown::{to_mdast, ParseOptions};

let ast = to_mdast(text, &ParseOptions::gfm())?;

// Walk the AST
fn visit(node: &markdown::mdast::Node) {
    match node {
        Node::Heading(h) => { /* depth, children */ }
        Node::Link(l) => { /* url, title */ }
        _ => {}
    }
    for child in node.children().unwrap_or(&vec![]) {
        visit(child);
    }
}
```

### comrak
Full CommonMark + GFM with rich AST.

```rust
use comrak::{parse_document, Arena, Options};

let arena = Arena::new();
let root = parse_document(&arena, text, &Options::default());

// Iterate AST nodes
for node in root.descendants() {
    match &node.data.borrow().value {
        NodeValue::Heading(h) => { /* level */ }
        NodeValue::Link(l) => { /* url, title */ }
        _ => {}
    }
}
```
