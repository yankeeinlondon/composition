# mdast Node Types Reference

Complete reference for all mdast node types and their TypeScript interfaces.

## Abstract Interfaces

mdast defines two primary abstract interfaces that other nodes inherit from:

```typescript
import type { Literal as UnistLiteral, Parent as UnistParent } from 'unist';

// Literal interface for nodes containing a value
interface Literal extends UnistLiteral {
  value: string;
}

// Parent interface for nodes containing other nodes
interface Parent extends UnistParent {
  children: MdastContent[];
}
```

## All Node Types

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
| | `ThematicBreak` | Horizontal rule (`---`) | Node |
| **Inlines** | `Emphasis` | Emphasized text (*italic*) | Parent |
| | `Strong` | Strong emphasis (**bold**) | Parent |
| | `InlineCode` | Inline code (\`code\`) | Literal |
| | `Break` | Hard line break | Node |
| | `Link` | Hyperlink with URL and title | Parent |
| | `Image` | Image with source and alt text | Node |
| | `Text` | Plain text content | Literal |
| **References** | `Definition` | Link definition target | Node |
| | `ImageReference` | Reference to image definition | Node |
| | `LinkReference` | Reference to link definition | Parent |

## Key Node Interfaces

### Root

```typescript
interface Root extends Parent {
  type: 'root';
  children: FlowContent[];
}
```

### Heading

```typescript
interface Heading extends Parent {
  type: 'heading';
  depth: 1 | 2 | 3 | 4 | 5 | 6;
  children: PhrasingContent[];
}
```

### Paragraph

```typescript
interface Paragraph extends Parent {
  type: 'paragraph';
  children: PhrasingContent[];
}
```

### List

```typescript
interface List extends Parent {
  type: 'list';
  ordered?: boolean;
  start?: number;
  spread?: boolean;
  children: ListItem[];
}
```

### Link

```typescript
interface Link extends Parent {
  type: 'link';
  url: string;
  title?: string;
  children: PhrasingContent[];
}
```

### Image

```typescript
interface Image extends Node {
  type: 'image';
  url: string;
  title?: string;
  alt?: string;
}
```

### Code (Fenced Block)

```typescript
interface Code extends Literal {
  type: 'code';
  lang?: string;
  meta?: string;
  value: string;
}
```

### Text

```typescript
interface Text extends Literal {
  type: 'text';
  value: string;
}
```

## Content Model

mdast defines a strict content model specifying valid parent-child relationships:

| Content Type | Description | Includes |
|--------------|-------------|----------|
| **FlowContent** | Block-level elements | `Paragraph`, `Heading`, `List`, `Blockquote`, `Code`, `Html`, `ThematicBreak` |
| **PhrasingContent** | Inline elements | `Text`, `Emphasis`, `Strong`, `InlineCode`, `Link`, `Image`, `Break` |
| **ListContent** | List-specific | `ListItem` |
| **Content** | Union of all | All content types |

### Nesting Rules

- `Root` contains `FlowContent`
- `Paragraph`, `Heading`, `Emphasis`, `Strong`, `Link` contain `PhrasingContent`
- `List` contains `ListItem`
- `Blockquote` contains `FlowContent`
- `ListItem` contains `FlowContent`

## Extension Node Types

### GFM Extensions (via remark-gfm)

```typescript
// Table
interface Table extends Parent {
  type: 'table';
  align?: ('left' | 'right' | 'center' | null)[];
  children: TableRow[];
}

// Footnote Definition
interface FootnoteDefinition extends Parent {
  type: 'footnoteDefinition';
  identifier: string;
  label?: string;
  children: FlowContent[];
}

// Strikethrough
interface Delete extends Parent {
  type: 'delete';
  children: PhrasingContent[];
}
```

### MDX Extensions (via remark-mdx)

```typescript
// JSX Element
interface MdxJsxFlowElement extends Parent {
  type: 'mdxJsxFlowElement';
  name: string | null;
  attributes: MdxJsxAttribute[];
  children: FlowContent[];
}

// JSX Expression
interface MdxFlowExpression extends Literal {
  type: 'mdxFlowExpression';
  value: string;
}
```

## Position Information

All nodes can have position data:

```typescript
interface Position {
  start: Point;
  end: Point;
}

interface Point {
  line: number;    // 1-indexed
  column: number;  // 1-indexed
  offset?: number; // 0-indexed byte offset
}
```

## Related

- [Ecosystem Overview](./ecosystem.md) - Tools for working with these nodes
- [TypeScript vs Rust](./ts-vs-rust.md) - Node type differences across implementations
