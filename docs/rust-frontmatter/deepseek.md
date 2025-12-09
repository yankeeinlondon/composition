# Parsing Frontmatter in a Rust Program

Here are the best Rust crates for parsing frontmatter in markdown files, with their key features and trade-offs.

## Frontmatter Parser Crate Comparison

| **Crate Name** | **Primary Use Case** | **Supported Formats** | **Key Feature / Approach** | **License** |
| :--- | :--- | :--- | :--- | :--- |
| **`markdown-frontmatter`** | Dedicated frontmatter parsing | YAML, TOML, JSON (via features) | Type-safe, splits frontmatter and body. Optional frontmatter support. | MIT |
| **`fronma`** | Dedicated frontmatter parsing | YAML (default), TOML, JSON (via features) | Uses parsing "engines". Returns a struct with `headers` and `body`. | Not specified |
| **`markdown-rs`** | Full CommonMark/GFM parsing | Frontmatter (as an extension) | Comprehensive, spec-compliant markdown parser. Frontmatter is one of many extensions. | Likely MIT (common) |

## Detailed Crate Overviews

Here is a more detailed look at each crate to help you choose:

### 1. `markdown-frontmatter`

This crate is designed specifically for parsing frontmatter. It cleanly splits a document into metadata and body and deserializes the frontmatter into your specified Rust type using `serde`.

- **Pros**: Simple, focused API. Explicitly handles documents with no frontmatter by treating them as having an empty one.
- **Cons**: Only parses frontmatter; does not process the markdown body itself.
- **Features**: Format support is gated behind features (`yaml`, `toml`, `json`), with none enabled by default.

### 2. `fronma`

Similar to `markdown-frontmatter`, `fronma` is a dedicated frontmatter parser. Its API returns a data structure containing both the parsed headers and the body.

- **Pros**: Straightforward output structure. Also uses feature-gated format support.
- **Cons**: Less documentation available compared to `markdown-frontmatter`.
- **Features**: YAML is the default format. The `toml` and `json` features enable support for those formats.

### 3. `markdown-rs`

This is a full-featured, CommonMark-compliant markdown parser. Frontmatter parsing is one of its available extensions.

- **Pros**: Ideal if you need to parse the entire markdown document (e.g., to HTML or an AST), not just extract frontmatter. Highly robust and security-conscious.
- **Cons**: More complex API if you only need frontmatter.
- **Features**: Parses frontmatter as part of a larger markdown processing pipeline.

## How to Choose the Right Crate

To decide which crate is best for your project, consider your primary goal:

- **Choose `markdown-frontmatter` or `fronma` if**: You only need to **extract metadata** from the top of markdown files and separate it from the body. These are lightweight, purpose-built tools. The choice between them may come down to a preference for their slightly different APIs.

- **Choose `markdown-rs` if**: You need to **fully process the markdown content** (the body) after the frontmatter. It's the best choice for building static site generators, documentation tools, or renderers where you need to convert the entire document.

Most dedicated frontmatter crates use Cargo **features** to manage support for different formats (YAML, TOML, JSON). This keeps the core crate small and lets you only compile the parsers you need.
