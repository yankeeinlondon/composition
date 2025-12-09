---
_fixed: true
---

# Rust LSP Implementation Guide

## 1. Core Rust crates for building Language Servers

These are the crates you actually reach for when writing an LSP server in Rust today.

### lsp-types

"The de-facto standard Rust types for the Language Server Protocol."

- Provides Rust structs/enums for the full LSP v3.16+ spec (requests, notifications, capabilities, diagnostics, etc.).
- Used by almost every serious Rust LSP project (including tower-lsp, async-lsp, mdbook_ls, etc.).

You almost always depend on this, regardless of which server framework you choose.

### tower-lsp

"Batteries-included server framework"

- High-level LSP server implementation built on the tower ecosystem.
- Gives you:
  - LspService builder and Server main loop abstractions.
  - A LanguageServer trait you implement with async methods (initialize, did_open, completion, etc.).
  - Works with Tokio or other async runtimes.

Rough mental model: tower-lsp + lsp-types is the "happy path" for most new servers.

### lsp-server (from the rust-analyzer team)

"Lower-level scaffold with explicit control over the main loop."

- Synchronous API using crossbeam channels, aimed at servers that want tight control over threading and scheduling.
- Handles:
  - JSON-RPC framing, protocol handshake.
  - Parsing messages and typing them via lsp-types.
- Used in rust-analyzer and some other heavy-duty servers.

If tower-lsp feels too opinionated and you want your own concurrency model, this is the next layer down.

### async-lsp

"Tower-style, middleware-centric LSP core."

- Defines a central LspService trait and a MainLoop driver.
- Uses Tower Layers for pluggable middlewares: concurrency control, tracing, cancellation, etc.
- Integrates with lsp-types and standard async/futures.

It's a more modular / "modern" alternative to lsp-server, and pairs nicely with crates like tower-lsp-server for ready-made servers.

### Legacy / ancillary crates

- languageserver-types - older LSP types crate; largely superseded by lsp-types, but still shows up in some projects.
- lsp-io (in the same ecosystem as some lsp-types forks) - IO helper for reading/writing LSP messages.

For new work, stick to lsp-types + tower-lsp or async-lsp unless you have a specific reason not to.

## 2. Markdown parsing crates you'd typically combine with LSP

These are the actual Markdown engines you'd embed under your language server.

- pulldown-cmark - fast pull parser for CommonMark; iterator over events, low allocation. Great when you want speed and are fine with a streaming model.
- comrak - full CommonMark + GitHub Flavored Markdown implementation that builds an AST and is spec-complete. Better when you need rich structure (headings, sections, links) and extensions.
- markdown-rs (markdown crate) - modern Markdown parser implemented as a state machine (`#![no_std]` + alloc) that yields concrete tokens; maintained under the `wooorm/markdown-rs` umbrella.

If you're building a Markdown LSP, you typically pick one of those, build an internal document model (maybe a custom AST or index), and then expose structure and diagnostics over LSP.

## 3. Markdown-oriented LSPs written in Rust

Crates like lsp-markdown or "vscode-markdown-lsp-server" do not appear as Rust projects. But there are several real Markdown-focused LSPs in Rust:

### 3.1 Markdown-Oxide

"PKM-style Markdown LSP in Rust."

- Project: markdown-oxide - PKM system implemented as a Markdown LSP.
- Exposes an LSP server that works with Neovim, VS Code, Helix, Zed, etc.
- Focus:
  - Personal Knowledge Management: links, backlinks, graph-like features.
  - Strong Obsidian/Logseq inspiration, but editor-agnostic via LSP.

If you want a "notes/PKM-first" Markdown LSP to study or extend, Markdown-Oxide is a prime example.

### 3.2 obsidian-lsp

"Rust LSP for Obsidian-style Markdown vaults."

- Crate & project: obsidian-lsp - LSP implementation for Obsidian markdown files.
- Features:
  - Go-to-definition, references, completion, etc., based on Obsidian-flavored Markdown (links, notes, vault structure).
  - Written in Rust and installable via `cargo install obsidian-lsp`.

This is a good reference if your domain is "vaults of Markdown files with wiki-style links".

### 3.3 Quickmark

"Markdown linter with first-class LSP support."

- Project: quickmark - Markdown/CommonMark linter written in Rust.
- Architecture:
  - quickmark-core (linting engine)
  - quickmark-cli (CLI)
  - quickmark-server (LSP server)
- Focus:
  - Diagnostics (lint errors/warnings) via LSP.
  - Integrations with VS Code and JetBrains via extensions.

If your Markdown LSP is mostly about linting + quick feedback, Quickmark is the closest "reference implementation".

### 3.4 mdBook-LS (mdBook Language Server)

"Live preview / book-oriented Markdown LSP."

- Project: mdbook_ls / mdBook-LS - "Language server to preview mdBook projects live."
- Purpose:
  - Connects your editor to an mdbook instance.
  - On change, patches the edited chapter into the rendered book preview.
  - Uses Rust + LSP to give mdBook-aware features.

It's a nice example of a domain-specific Markdown LSP: not general Markdown, but "Markdown as used by mdBook".

### 3.5 buffer-language-server (Markdown-agnostic but often used with it)

"Generic in-buffer completion server."

- Crate: buffer-language-server
- Idea:
  - Reads the current buffer text and offers LSP completion based on words present there.
  - Often configured as a fallback LSP for simple filetypes like markdown.
  - Not Markdown-aware per se, but relevant as a lightweight completion solution that works across languages.

### 3.6 mdbook-lint (Markdown linter used with LSP tools)

- Crate: mdbook-lint - fast Markdown linter for mdBook, written in Rust.
- Ties into a larger ecosystem where LSP servers (like mdBook-LS) and linting tools get combined for authoring docs.

## 4. Non-Rust Markdown LSPs (for context)

Your draft mentions "vscode-markdown-lsp-server"; in practice, the major generic Markdown LSPs are Node.js-based:

- remark-language-server - LSP for Markdown built on the remark/unified ecosystem.
- unified-language-server - more generic LSP for the unified pipeline, historically used for Markdown and related formats, now largely superseded by remark-language-server.

These get wired into VS Code, Emacs, JupyterLab, etc., but they are not Rust, which is probably what confused the original summary.

## 5. How I'd correct your original write-up

If you want to keep the same structure but make it accurate, I'd:

1. Replace the "cannot identify crates" intro with something like:
   - There are several well-maintained Rust crates specifically for implementing LSP servers: lsp-types for protocol types, tower-lsp and async-lsp for high-level server scaffolding, and lsp-server for a lower-level approach.
2. Keep the "Core Crates" section, but explicitly list:
   - lsp-types, tower-lsp, lsp-server, async-lsp, and optionally languageserver-types.
3. Drop or rename the fictional Markdown LSPs:
   - Replace lsp-markdown, mdbook-lsp, vscode-markdown-lsp-server with:
   - Markdown-Oxide
   - Obsidian-LSP
   - Quickmark
   - mdBook-LS
   - (optionally) buffer-language-server as a generic completion server.
4. Reference real Markdown parsers:
   - pulldown-cmark, comrak, markdown-rs as your parser layer.
