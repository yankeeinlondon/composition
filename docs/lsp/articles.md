---
_fixed: true
---

# LSP Implementation Articles and Tutorials

High-quality written guides for understanding the concepts behind building an LSP server in both Rust and TypeScript, complete with direct links.

## Rust LSP Server Articles

These guides often cover using the `tower-lsp` and `lsp-types` crates, which are standard for building production-ready Rust language servers.

| Title | Description | Link |
| :--- | :--- | :--- |
| **LSP outside the editor. Using the language server protocol in...** (Medium) | This article uses Rust to demonstrate implementing an LSP client to perform static analysis, providing a great, deep dive into the LSP message structure (JSON-RPC) from a client perspective. | [https://medium.com/@selfint/lsp-outside-the-editor-431f77a9a4be](https://medium.com/@selfint/lsp-outside-the-editor-431f77a9a4be) |
| **Rust tutorial on writing a lsp server?** (The Rust Programming Language Forum) | Although a forum thread, it is highly valued for linking to crucial, simpler examples of LSP servers in Rust, such as those that use the `tower-lsp` crate, and provides context for newcomers. | [https://users.rust-lang.org/t/rust-tutorial-on-writing-a-lsp-server/75570](https://users.rust-lang.org/t/rust-tutorial-on-writing-a-lsp-server/75570) |
| **TypeScript Language Server (Rust)** (Lib.rs) | This is the official page for a high-performance, Rust-based TypeScript language server implementation. The page itself serves as a detailed breakdown of the features and libraries (`tree-sitter`, `tower-lsp`) used in a complex LSP project. | [https://lib.rs/crates/typescript-language-server](https://lib.rs/crates/typescript-language-server) |

## TypeScript LSP Server Articles

These resources focus on the official Microsoft approach, typically involving the `vscode-languageserver` npm package for easy integration with VS Code.

| Title | Description | Link |
| :--- | :--- | :--- |
| **Language Server Extension Guide** (Visual Studio Code Docs) | This is the official Microsoft guide on building an LSP server and client using the Node SDK (`vscode-languageserver-node`), which is the canonical way to start a TypeScript/JavaScript LSP project. | [https://code.visualstudio.com/api/language-extensions/language-server-extension-guide](https://code.visualstudio.com/api/language-extensions/language-server-extension-guide) |
| **A Practical Guide for Language Server Protocol** (Medium) | A detailed, multi-part series that walks through implementing a language server (focused on the Ballerina language) using a transport-agnostic approach, which is excellent for understanding the architecture. | [https://medium.com/ballerina-techblog/practical-guide-for-the-language-server-protocol-3091a122b750](https://medium.com/ballerina-techblog/practical-guide-for-the-language-server-protocol-3091a122b750) |
| **Getting Started with the Language Server Protocol** (Nabeel Valley) | A beginner-friendly tutorial that shows how to manually handle the JSON-RPC communication (headers, content length) over stdin/stdout, demonstrating the protocol from the ground up without relying on high-level libraries. | [https://nabeelvalley.co.za/blog/2025/26-03/the-language-server-protocol/](https://nabeelvalley.co.za/blog/2025/26-03/the-language-server-protocol/) |

## General LSP Overviews

| Title | Description | Link |
| :--- | :--- | :--- |
| **Understanding the Language Server Protocol – Easier Code Editing Across Languages and Tools** (freeCodeCamp) | A fantastic general-purpose introduction that explains why LSP was created, its core components (client vs. server), and how the JSON-RPC communication works. | [https://www.freecodecamp.org/news/what-is-the-language-server-protocol-easier-code-editing-across-languages/](https://www.freecodecamp.org/news/what-is-the-language-server-protocol-easier-code-editing-across-languages/) |
