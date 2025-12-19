# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
# Build and test (run from module directory)
cargo build                    # Build the module
cargo test                     # Run unit tests
cargo test test_name           # Run a specific test

# From lib/
cargo build
cargo test

# From cli/
cargo build
cargo test
```

## Project Overview

Composition is a Rust monorepo for document composition with three modules:

- **Library** (`/lib`) - Core library with `init`, `graph`, `render`, `transclude`, `toHTML` functions
- **CLI** (`/cli`) - Command-line interface: `compose <file|glob> [--output <dir>] [--inline]`
- **LSP** (`/lsp`) - Language server for editor integration (planned)

The project implements the **DarkMatter DSL**, a superset of CommonMark/GFM markdown for content composition.

## Architecture

### Document Processing Pipeline

1. `init(dir?, frontmatter?)` - Initialize with project scope, locate/create `.composition.db`
2. `graph(resource)` - Build dependency graph for a file
3. `generateWorkplan(resources[])` - Plan concurrent execution, filter cached items
4. `render(resources[], state?)` - Orchestrate parsing with concurrency via rayon
5. `toHTML(glob[])` - Convert to self-contained HTML with inline assets

### Key DSL Features

- Transclusion: `::file ./path.md` - Include external content
- Summarization: `::summarize ./doc.md` - LLM-powered summary
- Consolidation: `::consolidate ./a.md ./b.md` - LLM-merged content
- Tables/Charts: `::table`, `::bar-chart` - Data visualization
- Frontmatter interpolation: `{{variable}}`
- Resource suffixes: `!` = required (error if missing), `?` = optional (silent if missing)

### Caching Strategy

- Local markdown: real-time resolution
- Remote HTTP content: cached 1 day (configurable per-reference)
- LLM operations: cached by content hash
- Database location: `{repo_root}/.composition.db` (git repos) or `$HOME/.composition.db`

## Tech Stack

- **Markdown**: pulldown-cmark for CommonMark/GFM parsing
- **Frontmatter**: markdown-frontmatter crate
- **Error handling**: thiserror
- **CLI**: Clap
- **Database**: SurrealDB
- **Concurrency**: rayon for parallel processing
- **Image processing**: image/zune-image crates
- **Hashing**: xxhash-rust

## Key Documentation

- [DarkMatter DSL](./docs/features/darkmatter-dsl.md) - Complete DSL specification
- [Library Features](./docs/features/library-features.md) - Library API surface
- [CLI Features](./docs/features/cli-features.md) - CLI usage and flags
- [LSP Features](./docs/features/lsp-features.md) - Editor integration features
- [Tech Stack](./docs/reference/tech-stack.md) - Detailed technology choices
- [Project Scope](./docs/reference/project-scope.md) - File scope rules

## Project-Local Skills

Skills in `.claude/skills/` provide expert knowledge for key dependencies:
- `pulldown-cmark` - Markdown parsing with event streams
- `rust-lsp` - Language server implementation
- `surrealdb` - Database operations
- `rayon` - Parallel processing
- `rust-image` - Image optimization
- `thiserror`, `color-eyre`, `anyhow` - Error handling patterns
- `clap` - CLI argument parsing
- `ropey` - Rope data structure for text editing
