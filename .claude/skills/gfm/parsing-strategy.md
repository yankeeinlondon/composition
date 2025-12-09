# GFM Parsing Strategy

GFM uses a two-phase parsing strategy that efficiently converts Markdown to output.

## Two-Phase Approach

### Phase 1: Block Structure

Identifies block-level elements:
- Headings
- Paragraphs
- Lists
- Code blocks
- Tables
- Block quotes

**Process:**
1. Scan document line by line
2. Determine type of each block
3. Resolve block nesting
4. Establish document skeleton

**No inline processing happens in this phase.**

### Phase 2: Inline Structure

Processes inline elements within each block:
- Emphasis (`*text*`, `**text**`)
- Links and images
- Code spans
- Autolinks
- Strikethrough

**Process:**
1. Parse within established blocks
2. Resolve text formatting
3. Process link references
4. Handle escapes and entities

## Visual Flow

```
Markdown Input
    |
    v
Block Structure Identification
    |
    v
Inline Structure Processing
    |
    v
Output Generation (HTML/AST/etc.)
```

## pulldown-cmark Implementation

pulldown-cmark implements this as a **pull parser**:

```rust
use pulldown_cmark::{Parser, Event, Tag};

let parser = Parser::new(markdown);

for event in parser {
    match event {
        // Block events
        Event::Start(Tag::Heading(level, id, classes)) => {}
        Event::Start(Tag::Paragraph) => {}
        Event::Start(Tag::List(first_item_number)) => {}
        Event::Start(Tag::CodeBlock(kind)) => {}
        Event::Start(Tag::Table(alignments)) => {}

        // Inline events (occur within blocks)
        Event::Text(text) => {}
        Event::Code(code) => {}
        Event::Start(Tag::Emphasis) => {}
        Event::Start(Tag::Strong) => {}
        Event::Start(Tag::Link(..)) => {}

        _ => {}
    }
}
```

## Pull Parser Advantages

| Advantage | Description |
|-----------|-------------|
| Memory Efficiency | No full document tree in memory |
| Performance | Parsing begins before full document available |
| Flexibility | Transform/filter event stream as needed |
| Streaming | Handle large documents or network streams |

## Event Pairing

Events come in pairs for block/inline elements:

```rust
Event::Start(Tag::Paragraph)  // Opening
Event::Text("content")        // Content
Event::End(Tag::Paragraph)    // Closing
```

**Exception:** Some events are standalone:
- `Event::SoftBreak`
- `Event::HardBreak`
- `Event::Rule`
- `Event::TaskListMarker(checked)`

## Related

- [Event Processing](./event-processing.md) - Detailed event handling
- [GFM Extensions](./gfm-extensions.md) - Extension-specific events
