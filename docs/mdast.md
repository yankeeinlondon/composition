---
name: mdast
description: Comprehensive guide to mdast (Markdown Abstract Syntax Tree) specification, ecosystem, and implementations
created: 2025-12-08
hash: 471700165b1797fa
_fixed: true
tags:
  - markdown
  - ast
  - unified
  - remark
  - rust
  - typescript
  - parsing
---

# mdast: Markdown Abstract Syntax Tree

mdast (Markdown Abstract Syntax Tree) is a formal specification for representing Markdown documents as abstract syntax trees. It provides a structured, programmatic way to parse, analyze, transform, and serialize Markdown content. As the backbone of the unified ecosystem, mdast enables sophisticated content processing pipelines used by major organizations including GitHub, Gatsby, Mozilla, and Adobe.

## Table of Contents

- [Core Concepts](#core-concepts)
- [AST Node Types](#ast-node-types)
- [The Unified Ecosystem](#the-unified-ecosystem)
- [TypeScript Implementation](#typescript-implementation)
- [Rust Implementation](#rust-implementation)
- [Cross-Language Compatibility](#cross-language-compatibility)
- [Performance Considerations](#performance-considerations)
- [Practical Usage](#practical-usage)
- [Extension Ecosystem](#extension-ecosystem)
- [When to Use mdast](#when-to-use-mdast)
- [Quick Reference](#quick-reference)
- [Resources](#resources)

## Core Concepts

### What is mdast?

mdast is a specification that defines how Markdown documents should be represented as syntax trees. It extends **unist** (Universal Syntax Tree), which provides the foundational interface for all syntax trees in the unified ecosystem. The specification creates a common language for processing Markdown across different tools, libraries, and programming languages.

Key characteristics of mdast:

- **Formal specification** with versioned releases (currently at version 5.0.0)
- **Language-agnostic** design enabling implementations in JavaScript, Rust, and other languages
- **Extensible architecture** supporting CommonMark, GFM, MDX, and custom extensions
- **Serializable to JSON or YAML** for cross-platform interoperability

### Standardization and Governance

mdast is maintained by the **syntax-tree** organization on GitHub, part of the broader **unified collective**. The project is led by **Titus Wormer** (wooorm), who also created remark and markdown-rs.

The governance model follows open-source principles:

- Specification-first approach with careful versioning
- Reference implementations providing de facto standards
- Community contribution through GitHub issues and pull requests
- Backward compatibility prioritized for specification changes

### Relationship to Other Standards

mdast exists within a family of related specifications:

| Standard | Purpose |
|----------|---------|
| **unist** | Foundational syntax tree specification |
| **mdast** | Markdown Abstract Syntax Tree |
| **hast** | HTML Abstract Syntax Tree |
| **xast** | XML Abstract Syntax Tree |
| **nlcst** | Natural Language Syntax Tree |

This relationship enables interoperability between different content formats through transformation utilities.

## AST Node Types

### Abstract Interfaces

mdast defines two primary abstract interfaces that other nodes inherit from:

```typescript
// Literal interface for nodes containing a value
interface Literal extends UnistLiteral {
  value: string
}

// Parent interface for nodes containing other nodes
interface Parent extends UnistParent {
  children: MdastContent[]
}
```

### Concrete Node Types

| Category | Node Type | Description | Interface |
|----------|-----------|-------------|-----------|
| **Document** | `Root` | Root node of the document | Parent |
| **Headings** | `Heading` | Heading content (levels 1-6) | Parent |
| **Paragraph** | `Paragraph` | Paragraph content | Parent |
| **Lists** | `List` | List container (ordered/unordered) | Parent |
| | `ListItem` | List item content | Parent |
| **Blocks** | `Blockquote` | Quoted text section | Parent |
| | `Code` | Fenced code block | Literal |
| | `Html` | Raw HTML content | Literal |
| | `ThematicBreak` | Horizontal rule | Void |
| **Inlines** | `Emphasis` | Emphasized text (*italic*) | Parent |
| | `Strong` | Strong emphasis (**bold**) | Parent |
| | `InlineCode` | Inline code | Literal |
| | `Break` | Hard line break | Void |
| | `Link` | Hyperlink with URL and title | Parent |
| | `Image` | Image with source and alt text | Void |
| | `Text` | Plain text content | Literal |
| **References** | `Definition` | Link definition target | Void |
| | `ImageReference` | Reference to image definition | Void |
| | `LinkReference` | Reference to link definition | Parent |

### Content Model

mdast defines a strict content model specifying which nodes can appear as children of other nodes:

- **FlowContent**: Block-level elements (paragraphs, headings, lists, etc.)
- **PhrasingContent**: Inline-level elements (text, emphasis, links, etc.)
- **ListContent**: Specifically for list items
- **Content**: Union of all possible content types

### Extension Node Types

Beyond standard nodes, mdast can be extended with:

- **GFM nodes**: Tables, footnote definitions, strikethrough, task list items
- **MDX nodes**: JSX elements and expressions embedded in Markdown
- **Directive nodes**: Generic directives like `:cite[smith04]`
- **Frontmatter nodes**: YAML, TOML, or other metadata formats

## The Unified Ecosystem

### Architecture Overview

The unified ecosystem enables processing content through plugins that transform syntax trees:

```
Markdown Text -> Parser -> mdast -> Transformers -> mdast -> Compiler -> Output
```

The typical workflow involves:

1. **Parsing**: Markdown text to mdast (using remark-parse)
2. **Transformation**: mdast to mdast (using various plugins)
3. **Serialization**: mdast to other formats (HTML, Markdown, etc.)

### Key Tools

| Tool | Purpose | Description |
|------|---------|-------------|
| **unified** | Orchestrator | Processes content through plugins |
| **remark** | Markdown processor | Plugin-based processor for Markdown |
| **remark-parse** | Parser | Parses Markdown to mdast |
| **remark-stringify** | Serializer | Serializes mdast to Markdown |
| **micromark** | Tokenizer | Low-level CommonMark tokenizer |
| **mdast-util-from-markdown** | Utility | Low-level Markdown to mdast conversion |
| **mdast-util-to-markdown** | Utility | Low-level mdast to Markdown conversion |
| **mdast-util-to-hast** | Transformer | Converts mdast to hast (HTML) |
| **rehype** | HTML processor | Plugin-based processor for HTML |

### Plugin Pipeline

```javascript
import { unified } from 'unified';
import remarkParse from 'remark-parse';
import remarkGfm from 'remark-gfm';
import remarkRehype from 'remark-rehype';
import rehypeStringify from 'rehype-stringify';

const result = await unified()
  .use(remarkParse)        // Parse Markdown to mdast
  .use(remarkGfm)          // Enable GFM extensions
  .use(remarkRehype)       // Convert mdast to hast
  .use(rehypeStringify)    // Serialize hast to HTML
  .process('# Hello **world**!');
```

## TypeScript Implementation

### Core Libraries

The TypeScript/JavaScript mdast ecosystem centers around the unified framework:

- **100% CommonMark compliant** via remark-parse
- **GFM support** via remark-gfm plugin
- **MDX support** via remark-mdx plugin
- **Rich plugin ecosystem** with 150+ plugins available

### Type Definitions

TypeScript users get comprehensive type definitions through `@types/mdast`:

```typescript
import type { Root, Heading, Paragraph, Text } from 'mdast';

const tree: Root = {
  type: 'root',
  children: [
    {
      type: 'heading',
      depth: 1,
      children: [{ type: 'text', value: 'Hello World' }]
    }
  ]
};
```

### Linting and Diagnostics

remark-lint provides approximately 70 rule plugins for style and consistency checks:

```javascript
import remarkLint from 'remark-lint';
import remarkLintHeadingIncrement from 'remark-lint-heading-increment';

unified()
  .use(remarkParse)
  .use(remarkLint)
  .use(remarkLintHeadingIncrement)
  .process(markdown);
```

Lint warnings include detailed position information (line, column) for IDE integration.

### Extension Development

Creating custom syntax extensions involves:

1. **Micromark extension**: Define tokenization rules
2. **mdast utility**: Transform tokens to AST nodes
3. **Remark plugin**: Package for use in unified pipelines

```javascript
import { visit } from 'unist-util-visit';

// Plugin to add IDs to headings
function remarkHeadingIds() {
  return (tree) => {
    visit(tree, 'heading', (node) => {
      const text = node.children
        .filter(child => child.type === 'text')
        .map(child => child.value)
        .join(' ');

      node.data = node.data || {};
      node.data.hProperties = {
        id: text.toLowerCase().replace(/[^\w]+/g, '-')
      };
    });
  };
}
```

## Rust Implementation

### markdown-rs

The Rust ecosystem centers around **markdown-rs**, created by Titus Wormer and funded initially by Vercel:

- **100% CommonMark compliant**
- **Built-in GFM support** (tables, strikethrough, task lists, footnotes)
- **Built-in MDX support** (JSX, expressions, import/export)
- **Zero unsafe Rust** implementation
- **No-std compatible** for WebAssembly targets

### Basic Usage

```rust
use markdown::{to_mdast, to_html, ParseOptions};

// Parse to mdast
let tree = to_mdast("# Hello **world**!", &ParseOptions::default())?;

// Convert to HTML
let html = to_html("# Hello **world**!")?;

// With GFM extensions
let html_gfm = to_html_with_options(
    "| A | B |\n|---|---|\n| 1 | 2 |",
    &Options::gfm()
)?;
```

### AST Structure

Rust mdast nodes mirror the JavaScript specification:

```rust
use markdown::mdast::{Node, Root, Heading, Text};

match &tree {
    Node::Root(root) => {
        for child in &root.children {
            match child {
                Node::Heading(h) => println!("Heading level {}", h.depth),
                Node::Paragraph(_) => println!("Paragraph"),
                _ => {}
            }
        }
    }
    _ => {}
}
```

### Safety Features

markdown-rs follows secure-by-default principles:

- Raw HTML is escaped by default (unlike remark)
- Dangerous protocols (e.g., `javascript:`) are blocked
- Options available to allow raw HTML when input is trusted

```rust
let options = Options {
    compile: CompileOptions {
        allow_dangerous_html: true,
        allow_dangerous_protocol: false,
        ..Default::default()
    },
    ..Default::default()
};
```

### Companion Crate: mdxjs-rs

For MDX compilation to JavaScript/JSX output, use `mdxjs-rs`:

```rust
use mdxjs::{compile, Options};

let jsx = compile("# Hello <Component />", &Options::default())?;
```

## Cross-Language Compatibility

### Language Support Matrix

| Language | mdast Support | Key Libraries |
|----------|---------------|---------------|
| **JavaScript/TypeScript** | Native & Extensive | mdast-util-from-markdown, remark, unified |
| **Rust** | Good Native Support | markdown-rs, mdxjs-rs |
| **Go** | Not Found | No dedicated mdast libraries |
| **Python** | Not Found | No dedicated mdast libraries |

### JSON Interoperability

Since mdast is a data specification, trees can be serialized to JSON for cross-language use:

```javascript
// JavaScript: Generate mdast JSON
import { fromMarkdown } from 'mdast-util-from-markdown';
const tree = fromMarkdown('# Hello');
const json = JSON.stringify(tree);
```

```rust
// Rust: Generate mdast JSON (with serde feature)
use markdown::to_mdast;
let tree = to_mdast("# Hello", &ParseOptions::default())?;
let json = serde_json::to_string(&tree)?;
```

### Compatibility Considerations

While both ecosystems produce similar ASTs:

- Node type names match the mdast specification
- Position information is available in both
- Some edge cases may produce different results
- Extensions may not be identically supported

> **Note:** When cross-language compatibility is critical, test with representative documents to verify consistent output.

## Performance Considerations

### Benchmark Data

The most direct comparison comes from [markdown-rs issue #113](https://github.com/wooorm/markdown-rs/issues/113), which benchmarks mdast generation:

| Implementation | Relative Speed | Notes |
|----------------|----------------|-------|
| markdown-rs (Rust) | Faster | Better constant factors |
| mdast-util-from-markdown (JS) | Baseline | Reference implementation |
| Old remark-parse (JS) | ~1.5x faster than micromark | Less spec-compliant |

### Key Findings

1. **Rust wins on constant factors**: markdown-rs consistently outperforms JavaScript for raw parsing speed
2. **Same algorithmic complexity**: Both exhibit similar asymptotic behavior; neither is O(n) where the other is O(n^2)
3. **MDX is expensive**: Large MDX files with complex embedded code can cause performance issues in both ecosystems
4. **Plugin overhead**: In real applications, plugin processing often dominates over raw parse time

### Real-World Implications

For **Docusaurus/MDX builds**:

- 50-70% of build time can be spent in parsing (mdast-util-from-markdown)
- Rust mdast via WASM or native service is an attractive optimization

For **Editor/Real-time use**:

- JavaScript mdast may cause lag on very long documents
- Consider incremental parsing or caching strategies

### When Performance Matters

```
Small documents (<10KB): Either implementation is fine
Medium documents (10KB-1MB): JS is adequate, Rust offers headroom
Large documents (>1MB): Consider Rust, especially for batch processing
Real-time editing: Profile your specific use case
```

## Practical Usage

### Basic Parsing and Transformation

```javascript
import { remark } from 'remark';
import { visit } from 'unist-util-visit';

// Parse markdown to mdast
const tree = remark().parse('# Hello **world**!');

// Transform the AST
visit(tree, 'heading', (node) => {
  if (node.depth === 1) {
    node.depth = 2; // Demote h1 to h2
  }
});

// Serialize back to markdown
const output = remark().stringify(tree);
// Output: ## Hello **world**!
```

### Converting to HTML

```javascript
import { unified } from 'unified';
import remarkParse from 'remark-parse';
import remarkRehype from 'remark-rehype';
import rehypeStringify from 'rehype-stringify';
import rehypeHighlight from 'rehype-highlight';

const html = await unified()
  .use(remarkParse)
  .use(remarkRehype)
  .use(rehypeHighlight) // Syntax highlighting
  .use(rehypeStringify)
  .process(markdown);
```

### Working with Frontmatter

```javascript
import { unified } from 'unified';
import remarkParse from 'remark-parse';
import remarkFrontmatter from 'remark-frontmatter';

const tree = unified()
  .use(remarkParse)
  .use(remarkFrontmatter, ['yaml'])
  .parse(`---
title: My Post
---

# Content here`);
```

### Custom Node Traversal

```javascript
import { visit, SKIP, EXIT } from 'unist-util-visit';

// Find all links
const links = [];
visit(tree, 'link', (node) => {
  links.push({ url: node.url, title: node.title });
});

// Modify images
visit(tree, 'image', (node) => {
  node.url = `/assets/${node.url}`;
});

// Early exit
visit(tree, 'heading', (node) => {
  if (node.depth === 1) {
    return EXIT; // Stop traversal
  }
});
```

## Extension Ecosystem

### GFM (GitHub Flavored Markdown)

**JavaScript:**

```javascript
import remarkGfm from 'remark-gfm';

unified()
  .use(remarkParse)
  .use(remarkGfm) // Enables tables, strikethrough, task lists, etc.
```

**Rust:**

```rust
let html = to_html_with_options(markdown, &Options::gfm())?;
```

### MDX

MDX allows embedding JSX components in Markdown:

**JavaScript:**

```javascript
import { compile } from '@mdx-js/mdx';

const jsxCode = await compile(`
# Hello

<MyComponent prop="value" />
`);
```

**Rust:**

```rust
use mdxjs::compile;

let jsx = compile("# Hello\n\n<MyComponent />", &Options::default())?;
```

### Popular Plugins

| Plugin | Purpose |
|--------|---------|
| remark-gfm | GitHub Flavored Markdown |
| remark-frontmatter | YAML/TOML frontmatter |
| remark-math | LaTeX math blocks |
| remark-directive | Generic directive syntax |
| remark-toc | Table of contents generation |
| remark-slug | Add IDs to headings |
| rehype-highlight | Syntax highlighting |
| rehype-sanitize | HTML sanitization |

### Custom Extensions in JavaScript

The JavaScript ecosystem allows defining custom syntax:

1. Create a micromark extension (tokenizer)
2. Create an mdast utility (token to node conversion)
3. Package as a remark plugin

### Extension Limitations in Rust

markdown-rs does not currently support user-defined syntax extensions:

- Built-in extensions only (GFM, MDX, frontmatter, math)
- Custom syntax requires preprocessing or AST post-processing
- Plugin system not a current project goal

## When to Use mdast

### mdast is Ideal When

- You need **complex transformations** requiring document structure understanding
- You want to **leverage the unified ecosystem** of plugins
- **Interoperability** between Markdown tools is important
- You're in a **JavaScript/TypeScript environment**
- You require **extensibility** for custom Markdown features

### Consider Alternatives When

- You have **simple transformation needs** (regex may suffice)
- **Performance is critical** and AST overhead is prohibitive
- You're in a **non-JavaScript language** without good mdast support
- You have **highly specialized requirements** that don't fit the model

### Ecosystem Comparison

| Aspect | TypeScript (Remark/Unified) | Rust (markdown-rs) |
|--------|----------------------------|---------------------|
| **Plugin Ecosystem** | 150+ plugins | Built-in only |
| **Customization** | Highly extensible | Limited to built-ins |
| **Performance** | Moderate | High |
| **MDX Support** | Full (remark-mdx) | Full (built-in) |
| **Linting** | remark-lint (~70 rules) | None built-in |
| **Safety** | User controls HTML | Safe by default |
| **TypeScript** | Full type definitions | N/A |
| **WASM** | Via bundlers | Native support |

### Choosing Your Implementation

**Choose JavaScript/TypeScript when:**

- You need the plugin ecosystem
- Your stack is already JS-based
- You want remark-lint for style checking
- MDX with React integration is required

**Choose Rust when:**

- Performance is critical
- You're already in a Rust environment
- You need safe-by-default HTML output
- WASM integration is planned

**Consider Hybrid approaches when:**

- Build-time performance matters (Rust for parsing)
- Runtime flexibility is needed (JS for plugins)
- Cross-platform deployment is required

## Quick Reference

### Node Type Cheat Sheet

| Markdown | Node Type | Example |
|----------|-----------|---------|
| `# Heading` | `heading` (depth: 1) | H1 heading |
| `**bold**` | `strong` | Bold text |
| `*italic*` | `emphasis` | Italic text |
| `` `code` `` | `inlineCode` | Inline code |
| `[text](url)` | `link` | Hyperlink |
| `![alt](url)` | `image` | Image |
| `> quote` | `blockquote` | Block quote |
| `- item` | `list` + `listItem` | Unordered list |
| `1. item` | `list` + `listItem` | Ordered list |
| ` ```code``` ` | `code` | Code block |
| `---` | `thematicBreak` | Horizontal rule |

### Common Operations

```javascript
// Parse
const tree = remark().parse(markdown);

// Transform
visit(tree, 'type', callback);

// Stringify to Markdown
const md = remark().stringify(tree);

// Convert to HTML
const html = await unified()
  .use(remarkParse)
  .use(remarkRehype)
  .use(rehypeStringify)
  .process(markdown);
```

### Position Information

Every mdast node includes position data:

```typescript
interface Position {
  start: { line: number; column: number; offset: number };
  end: { line: number; column: number; offset: number };
}
```

## Resources

### Official Documentation

- [mdast Specification](https://github.com/syntax-tree/mdast)
- [unist Specification](https://github.com/syntax-tree/unist)
- [unified Handbook](https://unifiedjs.com/learn/)
- [remark Documentation](https://github.com/remarkjs/remark)

### Rust Implementation

- [markdown-rs Repository](https://github.com/wooorm/markdown-rs)
- [markdown-rs Docs.rs](https://docs.rs/markdown/)
- [mdxjs-rs Repository](https://github.com/wooorm/mdxjs-rs)

### Learning Resources

- [Intro to unified](https://unifiedjs.com/learn/guide/introduction-to-unified/)
- [Create a remark plugin](https://unifiedjs.com/learn/guide/create-a-remark-plugin/)
- [Processing Markdown with remark](https://brunoscheufler.com/blog/2021-04-18-processing-markdown-with-remark-and-unified-plugins)

### Community

- [unified Collective](https://github.com/unifiedjs)
- [syntax-tree Organization](https://github.com/syntax-tree)
- [awesome-remark](https://github.com/remarkjs/awesome-remark)

### Package Statistics (as of 2025)

- **mdast-util-to-markdown**: 333+ million downloads
- **unified ecosystem**: ~10 million monthly npm downloads
- **markdown-rs**: Stable at 1.0.x, actively maintained
