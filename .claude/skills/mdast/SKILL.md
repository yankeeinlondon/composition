---
name: mdast
description: Expert knowledge for working with mdast (Markdown Abstract Syntax Tree) - the unified/remark ecosystem standard for programmatic markdown parsing, transformation, and serialization. Covers node types, TypeScript types, plugins, unified pipelines, GFM/MDX extensions, and Rust markdown-rs comparisons.
hash: 2f717dfe3b6e5d5c
---

# mdast (Markdown Abstract Syntax Tree)

mdast is a formal specification for representing Markdown documents as syntax trees. Part of the unified collective, it enables programmatic parsing, analysis, transformation, and serialization of Markdown content.

## Core Principles

- mdast extends **unist** (Universal Syntax Tree) - all nodes have `type`, optional `position`, and optional `data`
- Use **remark** for high-level processing; use **mdast-util-from-markdown** for low-level parsing
- All nodes carry source position info (`start`/`end` with `line`/`column`/`offset`)
- The AST is **serializable to JSON** for cross-language interop and debugging
- TypeScript types available via `@types/mdast` package
- GFM and MDX are extensions - enable via plugins (`remark-gfm`, `remark-mdx`)
- Transform ASTs using `unist-util-visit` for traversal
- Convert mdast to HTML via `remark-rehype` then `rehype-stringify`
- Round-trip supported: markdown -> mdast -> markdown via `remark-stringify`

## Quick Reference

```typescript
import { remark } from 'remark';
import { visit } from 'unist-util-visit';

// Parse markdown to mdast
const tree = remark().parse('# Hello **world**!');

// Transform the AST
visit(tree, 'heading', (node) => {
  if (node.depth === 1) node.depth = 2;
});

// Serialize back to markdown
const output = remark().stringify(tree);
```

### Full Pipeline (Markdown to HTML)

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
  .process('# Hello **world**!');
```

## Topics

### Specification & Node Types

- [Node Types Reference](./node-types.md) - All mdast node types with interfaces
- [Content Model](./node-types.md#content-model) - Which nodes can contain which children

### Ecosystem & Tools

- [Ecosystem Overview](./ecosystem.md) - Key libraries and their purposes
- [Plugin Development](./ecosystem.md#plugin-development) - Creating custom remark plugins

### Comparisons

- [TypeScript vs Rust](./ts-vs-rust.md) - Feature comparison of remark vs markdown-rs
- [Performance](./performance.md) - Benchmarks and optimization guidance

## Common Patterns

### Custom Plugin

```typescript
import { visit } from 'unist-util-visit';
import type { Root, Heading } from 'mdast';

function addHeadingIds() {
  return (tree: Root) => {
    visit(tree, 'heading', (node: Heading) => {
      const text = node.children
        .filter(c => c.type === 'text')
        .map(c => c.value)
        .join(' ');
      node.data = node.data || {};
      node.data.hProperties = { id: text.toLowerCase().replace(/\W+/g, '-') };
    });
  };
}
```

### Type-Safe Node Creation

```typescript
import type { Heading, Text } from 'mdast';

const heading: Heading = {
  type: 'heading',
  depth: 2,
  children: [{ type: 'text', value: 'My Heading' } as Text]
};
```

## Resources

- [mdast Specification](https://github.com/syntax-tree/mdast)
- [unified Handbook](https://unifiedjs.com/learn/)
- [remark Plugins](https://github.com/remarkjs/awesome-remark)
- [markdown-rs (Rust)](https://github.com/wooorm/markdown-rs)
- [@types/mdast](https://www.npmjs.com/package/@types/mdast)
