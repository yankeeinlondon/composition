# mdast Ecosystem Overview

The mdast ecosystem centers around the unified collective, providing tools for parsing, transforming, and serializing markdown.

## Core Libraries

| Package | Purpose | Description |
|---------|---------|-------------|
| **unified** | Orchestrator | Process content through plugin pipelines |
| **remark** | Processor | Markdown processor (unified + remark-parse + remark-stringify) |
| **remark-parse** | Parser | Parse markdown to mdast |
| **remark-stringify** | Serializer | Serialize mdast to markdown |
| **mdast-util-from-markdown** | Low-level parser | Direct markdown to mdast conversion |
| **mdast-util-to-markdown** | Low-level serializer | Direct mdast to markdown conversion |
| **mdast-util-to-hast** | Transformer | Convert mdast to hast (HTML AST) |

## Typical Workflow

```
Markdown Text
     |
     v (remark-parse)
   mdast
     |
     v (plugins transform AST)
   mdast
     |
     +---> (remark-stringify) --> Markdown
     |
     +---> (remark-rehype) --> hast --> (rehype-stringify) --> HTML
```

## Essential Plugins

### Extensions

| Plugin | Purpose |
|--------|---------|
| `remark-gfm` | GitHub Flavored Markdown (tables, task lists, strikethrough, autolinks) |
| `remark-mdx` | MDX support (JSX in markdown) |
| `remark-frontmatter` | YAML/TOML frontmatter |
| `remark-math` | LaTeX math blocks |
| `remark-directive` | Generic directive syntax (`:name[label]{attributes}`) |

### Transformation

| Plugin | Purpose |
|--------|---------|
| `remark-rehype` | Convert mdast to hast for HTML output |
| `remark-toc` | Generate table of contents |
| `remark-slug` | Add IDs to headings |
| `remark-unwrap-images` | Remove paragraph wrappers from images |

### Linting

| Plugin | Purpose |
|--------|---------|
| `remark-lint` | Markdown style linting (~70 rules) |
| `remark-preset-lint-recommended` | Recommended lint preset |
| `remark-validate-links` | Check for broken links |

## Utility Libraries

| Package | Purpose |
|---------|---------|
| `unist-util-visit` | Traverse and transform AST nodes |
| `unist-util-select` | CSS-like selectors for AST nodes |
| `mdast-util-find-and-replace` | Find and replace text in AST |
| `mdast-util-heading-range` | Extract sections by heading |

## Plugin Development

### Basic Plugin Structure

```typescript
import { visit } from 'unist-util-visit';
import type { Root } from 'mdast';
import type { Plugin } from 'unified';

const myPlugin: Plugin<[], Root> = () => {
  return (tree: Root) => {
    visit(tree, 'heading', (node) => {
      // Transform heading nodes
    });
  };
};

export default myPlugin;
```

### Plugin with Options

```typescript
import type { Root } from 'mdast';
import type { Plugin } from 'unified';

interface Options {
  prefix?: string;
  maxDepth?: number;
}

const myPlugin: Plugin<[Options?], Root> = (options = {}) => {
  const { prefix = '', maxDepth = 6 } = options;

  return (tree: Root) => {
    // Use options in transformation
  };
};
```

### Attacher vs Transformer

```typescript
// Attacher: called once when plugin is added (returns transformer)
function attacher(options) {
  // Setup code runs once

  // Transformer: called for each file processed
  return function transformer(tree, file) {
    // Transform code runs per file
  };
}
```

## Working with Positions

```typescript
import { visit } from 'unist-util-visit';
import type { Root } from 'mdast';
import type { VFile } from 'vfile';

function warnOnShortHeadings() {
  return (tree: Root, file: VFile) => {
    visit(tree, 'heading', (node) => {
      const text = node.children
        .filter(c => c.type === 'text')
        .map(c => c.value)
        .join('');

      if (text.length < 3) {
        file.message('Heading too short', node.position);
      }
    });
  };
}
```

## Cross-Format Pipelines

### Markdown to HTML

```typescript
import { unified } from 'unified';
import remarkParse from 'remark-parse';
import remarkGfm from 'remark-gfm';
import remarkRehype from 'remark-rehype';
import rehypeSanitize from 'rehype-sanitize';
import rehypeStringify from 'rehype-stringify';

const processor = unified()
  .use(remarkParse)
  .use(remarkGfm)
  .use(remarkRehype)
  .use(rehypeSanitize)
  .use(rehypeStringify);

const html = await processor.process(markdown);
```

### Markdown to React (MDX)

```typescript
import { compile } from '@mdx-js/mdx';
import remarkGfm from 'remark-gfm';

const code = await compile(mdxContent, {
  remarkPlugins: [remarkGfm],
  // Produces JSX/React component
});
```

## Notable Adopters

- **Gatsby** - Static site generation
- **Docusaurus** - Documentation sites
- **Next.js** - MDX integration
- **Prettier** - Markdown formatting
- **VS Code** - Markdown features
- **GitHub** - Markdown processing

## Related

- [Node Types Reference](./node-types.md) - All mdast node types
- [TypeScript vs Rust](./ts-vs-rust.md) - Ecosystem comparison
- [Performance](./performance.md) - Optimization considerations
