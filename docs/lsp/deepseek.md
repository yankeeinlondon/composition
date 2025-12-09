---
_fixed: true
---

# Building a Language Server in Rust

A language server acts as a backend for IDE features (like autocomplete or go-to-definition), communicating with the editor frontend via the JSON-RPC-based Language Server Protocol (LSP). In Rust, two crates are central to this architecture.

## Core Crates for Building a Language Server in Rust

| Crate Name | Description & Role | Key Features & Notes |
| :--- | :--- | :--- |
| `tower-lsp` | A higher-level, "batteries-included" framework built on `tower` for writing LSP servers. It abstracts away much of the protocol and networking boilerplate. | Provides a simple `LanguageServer` trait to implement. Handles JSON-RPC message dispatch, lifecycle, and concurrency. Includes built-in support for runtime-agnostic async (works with Tokio, async-std). **Easier to start with** for a new project. |
| `lsp-server` / `lsp-types` | Lower-level foundational crates maintained by the Rust Analyzer team. `lsp-types` provides the complete type definitions for the LSP specification, while `lsp-server` handles the connection and message flow. | `lsp-types` is the **de facto standard** for LSP data structures in Rust, used by nearly all Rust LSP projects. `lsp-server` gives you more fine-grained control over the server loop and message handling. Requires you to manage more protocol details yourself. |

### How to Choose

For most new projects, `tower-lsp` is the recommended starting point due to its ease of use. However, if you need maximum control or are integrating with a complex existing system, the combination of `lsp-server` and `lsp-types` offers more flexibility. It's also common to use `lsp-types` even with `tower-lsp`.

## Known Markdown LSPs Written in Rust

Building a full-featured Markdown LSP is complex, involving parsing, understanding document structure, and providing intelligent editing features. Here are the known implementations in the Rust ecosystem:

| Project Name | Description & Status | Key Capabilities |
| :--- | :--- | :--- |
| `lsp-markdown` | A **library** for creating Markdown language servers, not a ready-to-run server itself. It's designed to be integrated into other tools. | Built on `tower-lsp`. Provides a `MarkdownServer` struct that handles standard LSP requests (document symbols, folding ranges, workspace diagnostics). Uses `markdown-rs` or `comrak` for parsing, offering a pluggable architecture. |
| `mdbook-lsp` | An **integrated LSP server** for the `mdbook` documentation tool. It is tailored for working with `mdbook` projects. | Provides diagnostics, previews, and navigation specific to `mdbook`'s structure (e.g., `SUMMARY.md`). Tightly coupled with the `mdbook` ecosystem. |
| `vscode-markdown-lsp-server` | A **standalone LSP server** specifically built for the VS Code `vscode-markdown` extension. This is likely the most full-featured and actively used Rust-based Markdown LSP. | Offers a wide range of features: outline/document symbols, find all references, folding, smart selection, and more. Aims for high compatibility with CommonMark and GitHub Flavored Markdown (GFM). |

## Implementation Considerations

When starting a language server project in Rust, keep these points in mind:

- **Foundation First**: Any language server needs a **robust parser** for the target language. For a custom language, you would need to develop or integrate one using crates like `nom`, `pest`, or `lalrpop`.
- **State Management**: You must efficiently manage the state of opened documents, including incremental updates sent by the client (`textDocument/didChange`).
- **Async Processing**: Language servers must handle multiple simultaneous requests without blocking. Using an async runtime like **Tokio** is standard practice.
