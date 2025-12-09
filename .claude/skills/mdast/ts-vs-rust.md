# TypeScript vs Rust: mdast Implementations

Comparison of the TypeScript (remark/unified) and Rust (markdown-rs) ecosystems for mdast processing.

## Feature Comparison

| Feature | TypeScript (Remark/Unified) | Rust (markdown-rs) |
|---------|----------------------------|-------------------|
| **CommonMark** | 100% compliant | 100% compliant |
| **GFM** | Via `remark-gfm` plugin | Built-in (enable via options) |
| **MDX** | Via `remark-mdx` plugin | Built-in MDX parsing |
| **AST Format** | mdast (unist nodes) | mdast (same spec) |
| **Positions** | Full source positions | Full source positions |
| **Parsing Speed** | Moderate | High (native code) |
| **Plugin System** | 150+ plugins | No plugin system |
| **Custom Syntax** | Extensible via micromark | Not supported |
| **Linting** | ~70 rules via remark-lint | None built-in |
| **HTML Output** | Via rehype | Built-in, sanitized by default |
| **JSON Serialization** | Native JS objects | Via serde feature |

## When to Choose TypeScript

**Use remark/unified when:**

- You need the rich plugin ecosystem
- Complex transformations requiring multiple plugins
- Custom markdown syntax extensions
- Integration with existing JS/TS toolchains (Gatsby, Next.js, Docusaurus)
- Markdown linting requirements
- MDX with React component rendering

**Key advantages:**

- 150+ existing plugins
- Easy custom plugin development
- Direct React/JSX integration
- Mature, battle-tested ecosystem

## When to Choose Rust

**Use markdown-rs when:**

- Performance is critical (high-throughput servers, large docs)
- Building Rust-native applications
- Need WASM for browser with better perf than JS
- Untrusted input (sanitization by default)
- Simpler use case without custom extensions

**Key advantages:**

- Significantly faster parsing
- Lower memory usage
- Safe HTML output by default
- No runtime dependencies

## API Comparison

### TypeScript: Parse to mdast

```typescript
import { fromMarkdown } from 'mdast-util-from-markdown';
import { gfm } from 'micromark-extension-gfm';
import { gfmFromMarkdown } from 'mdast-util-gfm';

const tree = fromMarkdown(markdown, {
  extensions: [gfm()],
  mdastExtensions: [gfmFromMarkdown()]
});
```

### Rust: Parse to mdast

```rust
use markdown::{to_mdast, ParseOptions};

let tree = to_mdast(
    &markdown,
    &ParseOptions::gfm()
)?;
```

### TypeScript: Full Pipeline

```typescript
import { unified } from 'unified';
import remarkParse from 'remark-parse';
import remarkGfm from 'remark-gfm';
import remarkRehype from 'remark-rehype';
import rehypeStringify from 'rehype-stringify';

const html = await unified()
  .use(remarkParse)
  .use(remarkGfm)
  .use(remarkRehype)
  .use(rehypeStringify)
  .process(markdown);
```

### Rust: Direct HTML

```rust
use markdown::{to_html_with_options, Options};

let html = to_html_with_options(
    &markdown,
    &Options::gfm()
)?;
```

## Extension Handling

### TypeScript: Custom Syntax

```typescript
// Can define entirely new syntax via micromark extensions
import { directive } from 'micromark-extension-directive';
import { directiveFromMarkdown } from 'mdast-util-directive';

const tree = fromMarkdown(markdown, {
  extensions: [directive()],
  mdastExtensions: [directiveFromMarkdown()]
});
```

### Rust: Limited to Built-ins

```rust
// Only built-in extensions available
let options = ParseOptions {
    constructs: Constructs {
        gfm_autolink_literal: true,
        gfm_table: true,
        gfm_task_list_item: true,
        gfm_strikethrough: true,
        gfm_footnote_definition: true,
        ..Default::default()
    },
    ..Default::default()
};
```

## AST Node Compatibility

Both produce the same mdast structure. A heading in both:

```json
{
  "type": "heading",
  "depth": 2,
  "children": [
    { "type": "text", "value": "Hello World" }
  ],
  "position": {
    "start": { "line": 1, "column": 1, "offset": 0 },
    "end": { "line": 1, "column": 16, "offset": 15 }
  }
}
```

## Transformation Patterns

### TypeScript: Plugin-Based

```typescript
import { visit } from 'unist-util-visit';

function myTransform() {
  return (tree) => {
    visit(tree, 'link', (node) => {
      // Add target="_blank" to external links
      if (node.url.startsWith('http')) {
        node.data = node.data || {};
        node.data.hProperties = { target: '_blank' };
      }
    });
  };
}
```

### Rust: Manual Traversal

```rust
use markdown::mdast::{Node, Root};

fn transform_links(node: &mut Node) {
    match node {
        Node::Link(link) => {
            if link.url.starts_with("http") {
                // Modify link properties
            }
        }
        Node::Root(Root { children, .. }) |
        Node::Paragraph(Paragraph { children, .. }) => {
            for child in children {
                transform_links(child);
            }
        }
        _ => {}
    }
}
```

## Hybrid Approaches

### Rust Parser + JS Plugins

For performance-critical parsing with rich transformations:

1. Parse with markdown-rs (native or WASM)
2. Serialize AST to JSON
3. Transform in JS with unist utilities
4. Render with rehype

### Shared AST Format

Both ecosystems can exchange mdast via JSON:

```typescript
// Export from Rust (with serde)
let json = serde_json::to_string(&ast)?;

// Import in TypeScript
const tree = JSON.parse(rustJson);
visit(tree, 'heading', transformFn);
```

## Related

- [Ecosystem Overview](./ecosystem.md) - TypeScript ecosystem details
- [Performance](./performance.md) - Benchmark data
- [Node Types Reference](./node-types.md) - Shared node type definitions
