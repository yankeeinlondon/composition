# Markdown LSPs Written in Rust

Reference implementations for building Markdown-focused language servers.

## Production-Ready Projects

### markdown-oxide

**"PKM-style Markdown LSP"**

- **Repository**: [Feel-ix-343/markdown-oxide](https://github.com/Feel-ix-343/markdown-oxide)
- **Focus**: Personal Knowledge Management (Obsidian/Logseq-style workflows)
- **Features**:
  - Wiki-style link completion (`[[page]]`)
  - Backlink tracking and navigation
  - Cross-file reference validation
  - Go-to-definition for links
  - Workspace-wide symbol search
- **Editor Support**: VS Code, Neovim, Helix, Zed

**Good reference for**: Link-based navigation, workspace indexing, PKM features

### obsidian-lsp

**"LSP for Obsidian-style vaults"**

- **Crate**: `obsidian-lsp` on crates.io
- **Focus**: Obsidian Markdown flavor
- **Features**:
  - Obsidian link syntax support
  - Vault structure awareness
  - Tag completion
  - Property/frontmatter handling
- **Install**: `cargo install obsidian-lsp`

**Good reference for**: Vault/workspace patterns, frontmatter handling

### Quickmark

**"Markdown linter with LSP support"**

- **Repository**: [blaesink/quickmark](https://github.com/blaesink/quickmark)
- **Architecture**:
  - `quickmark-core` - Linting engine
  - `quickmark-cli` - CLI tool
  - `quickmark-server` - LSP server
- **Focus**: Diagnostics and linting
- **Features**:
  - Style/formatting diagnostics
  - CommonMark compliance checking
  - VS Code and JetBrains integrations

**Good reference for**: Diagnostic generation, modular architecture

### mdBook-LS

**"mdBook live preview server"**

- **Repository**: [mdbook_ls](https://github.com/mdbook-rs/mdbook_ls)
- **Focus**: mdBook documentation projects
- **Features**:
  - SUMMARY.md navigation
  - Chapter preview
  - Book structure awareness
  - Live rebuild on change

**Good reference for**: Domain-specific Markdown (book/docs structure)

### zeta-note

**"Rust Markdown LSP inspired by rust-analyzer"**

- **Focus**: Note-taking and documentation
- **Features**:
  - Diagnostics for broken links
  - Duplicate header detection
  - Cross-reference navigation
  - File/header completion
- **Architecture**: Incremental analysis inspired by rust-analyzer

**Good reference for**: rust-analyzer-style architecture, incremental computation

## Non-Rust Alternatives (For Context)

- **remark-language-server** (Node.js) - LSP built on unified/remark ecosystem
- **unified-language-server** (Node.js) - Generic unified pipeline LSP

These are more mature for general Markdown editing but lack Rust's performance.

## Common Patterns Across Projects

### Document Structure Analysis

All Markdown LSPs need to extract structure:

```rust
struct MarkdownDocument {
    headings: Vec<Heading>,
    links: Vec<Link>,
    code_blocks: Vec<CodeBlock>,
    frontmatter: Option<Frontmatter>,
}

struct Heading {
    level: u8,
    text: String,
    range: Range,
    id: Option<String>,
}

struct Link {
    text: String,
    target: String,
    link_type: LinkType, // WikiLink, Reference, Inline, etc.
    range: Range,
}
```

### Workspace Indexing

For cross-file features (backlinks, go-to-definition):

```rust
struct WorkspaceIndex {
    // File path → document structure
    documents: HashMap<PathBuf, MarkdownDocument>,

    // Link target → source locations (for backlinks)
    backlinks: HashMap<String, Vec<Location>>,

    // Heading ID → location (for anchor links)
    anchors: HashMap<String, Location>,
}
```

### Link Resolution

```rust
fn resolve_link(&self, link: &Link, current_file: &Path) -> Option<Location> {
    match &link.link_type {
        LinkType::WikiLink => {
            // [[Page Name]] → find file named "Page Name.md"
            self.find_file_by_name(&link.target)
        }
        LinkType::Relative => {
            // ./path/to/file.md
            let resolved = current_file.parent()?.join(&link.target);
            self.get_location(&resolved)
        }
        LinkType::Anchor => {
            // #heading-id
            self.anchors.get(&link.target).cloned()
        }
        LinkType::External => None, // https://...
    }
}
```

## Building a Custom Markdown LSP

### Recommended Stack

```toml
[dependencies]
# LSP framework
tower-lsp = "0.20"
lsp-types = "0.95"
tokio = { version = "1", features = ["full"] }

# Document management
ropey = "1"
dashmap = "5"

# Markdown parsing (choose one or combine)
pulldown-cmark = "0.12"    # Fast CommonMark
comrak = "0.28"             # Full GFM with AST
markdown = "1.0.0-alpha"    # markdown-rs, extensible

# Utilities
url = "2"
walkdir = "2"
tracing = "0.1"
```

### Minimal Architecture

```
MarkdownLspServer
├── DocumentManager
│   └── DashMap<Url, DocumentState>
├── WorkspaceIndex (for cross-file features)
├── Parser (pulldown-cmark or comrak)
└── FeatureHandlers
    ├── CompletionHandler (links, headings)
    ├── HoverHandler (link previews)
    ├── DefinitionHandler (go-to-definition)
    └── DiagnosticsHandler (broken links, style)
```

### pulldown-cmark Integration

See the [pulldown-cmark skill](../pulldown-cmark/SKILL.md) for detailed parsing guidance.

Key for LSP: Use `into_offset_iter()` for source positions:

```rust
use pulldown_cmark::{Parser, Event, Tag, Options};

fn extract_structure(text: &str) -> Vec<(Event, std::ops::Range<usize>)> {
    let options = Options::all();
    Parser::new_ext(text, options)
        .into_offset_iter()
        .collect()
}

fn find_headings(text: &str) -> Vec<Heading> {
    let mut headings = vec![];
    let mut current_heading: Option<(HeadingLevel, Range<usize>)> = None;

    for (event, range) in Parser::new(text).into_offset_iter() {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                current_heading = Some((level, range));
            }
            Event::Text(text) if current_heading.is_some() => {
                let (level, range) = current_heading.take().unwrap();
                headings.push(Heading {
                    level: level as u8,
                    text: text.to_string(),
                    range: byte_range_to_lsp_range(&rope, range),
                    id: None,
                });
            }
            _ => {}
        }
    }
    headings
}
```
