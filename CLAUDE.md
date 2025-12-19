# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Composition is a Rust monorepo for document composition with three modules:

- **Library** (`/lib`) - Core library exposing `init`, `graph`, `validate`, `resolve`, `parse`, and `toHTML` functions
- **CLI** (`/cli`) - Command-line interface using Clap: `compose <input> --out <dir> [--html]`
- **LSP** (`/lsp`) - Language server for editor integration with file autocomplete and interpolation support

The project implements the **DarkMatter DSL**, a custom superset of CommonMark/GFM markdown for content composition. Key DSL features include transclusion (`::file`), LLM-powered summarization (`::summarize`), consolidation (`::consolidate`), tables/charts, popovers, and frontmatter interpolation (`{{variable}}`).

## Tech Stack

- **Language**: Rust
- **Markdown parsing**: [pulldown-cmark](https://github.com/pulldown-cmark/pulldown-cmark) for CommonMark/GFM
- **Error handling**: thiserror
- **CLI**: Clap
- **Caching**: SurrealDB (stored as `_composition-cache.db` in git root or `$HOME`)

## Architecture

### Document Processing Pipeline

1. `init()` - Initialize cache database location
2. `validate()` - Check dependency graph is acyclic (DAG)
3. `resolve()` - Walk document tree, resolve DSL nodes, apply caching
4. `parse()` - Orchestrate multiple files via glob patterns
5. `toHTML()` - Convert to self-contained HTML with inline assets

### Caching Strategy

- Local markdown: real-time resolution
- Remote HTTP content: cached 1 day by default (configurable per-reference)
- LLM operations (summarize, consolidate): cached to avoid recomputation
- Resource suffixes: `!` = required (error if missing), `?` = optional (silent if missing)

## Key Documentation

- [DarkMatter DSL](./docs/darkmatter-dsl.md) - Complete DSL specification
- [Library API](./docs/library-api.md) - Public API surface
- [LSP Features](./docs/lsp-features.md) - Editor integration features
- [Document Scope](./docs/doc-scope.md) - Context-aware autocomplete scopes

## Project-Local Skills

Skills in `.claude/skills/` provide expert knowledge for key dependencies:
- `pulldown-cmark` - Markdown parsing with event streams
- `rust-lsp` - Language server implementation
- `thiserror`, `color-eyre`, `anyhow` - Error handling patterns
- `clap` - CLI argument parsing
- `ropey` - Rope data structure for text editing
