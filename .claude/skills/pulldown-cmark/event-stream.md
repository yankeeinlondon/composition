# Event Stream Reference

## Event Enum

The `Event<'a>` enum represents all parsing events:

```rust
pub enum Event<'a> {
    // Element boundaries
    Start(Tag<'a>),        // Opening tag with metadata
    End(TagEnd),           // Closing tag (no metadata)

    // Content events
    Text(CowStr<'a>),      // Text content
    Code(CowStr<'a>),      // Inline code (`code`)
    Html(CowStr<'a>),      // HTML block
    InlineHtml(CowStr<'a>),// Inline HTML (<span>, etc.)

    // Math (requires ENABLE_MATH)
    InlineMath(CowStr<'a>),  // $...$
    DisplayMath(CowStr<'a>), // $$...$$

    // Breaks and rules
    SoftBreak,             // Line break (no trailing spaces)
    HardBreak,             // Line break (2+ spaces or \)
    Rule,                  // Horizontal rule (---, ***, ___)

    // Special elements
    FootnoteReference(CowStr<'a>), // [^label]
    TaskListMarker(bool),          // - [x] or - [ ]
}
```

## Tag Enum (Element Types)

```rust
pub enum Tag<'a> {
    // Block elements
    Paragraph,
    Heading { level: HeadingLevel, id: Option<CowStr<'a>>, classes: Vec<CowStr<'a>>, attrs: Vec<(CowStr<'a>, Option<CowStr<'a>>)> },
    BlockQuote(Option<BlockQuoteKind>),
    CodeBlock(CodeBlockKind<'a>),
    HtmlBlock,

    // Lists
    List(Option<u64>),     // Some(n) = ordered starting at n, None = unordered
    Item,                  // List item

    // Tables (requires ENABLE_TABLES)
    Table(Vec<Alignment>),
    TableHead,
    TableRow,
    TableCell,

    // Inline elements
    Emphasis,              // *text* or _text_
    Strong,                // **text** or __text__
    Strikethrough,         // ~~text~~ (requires ENABLE_STRIKETHROUGH)
    Link { link_type: LinkType, dest_url: CowStr<'a>, title: CowStr<'a>, id: CowStr<'a> },
    Image { link_type: LinkType, dest_url: CowStr<'a>, title: CowStr<'a>, id: CowStr<'a> },

    // Special
    FootnoteDefinition(CowStr<'a>),
    MetadataBlock(MetadataBlockKind),
}
```

## TagEnd Enum

`TagEnd` mirrors `Tag` but without data (used only to mark element end):

```rust
pub enum TagEnd {
    Paragraph,
    Heading(HeadingLevel),
    BlockQuote,
    CodeBlock,
    HtmlBlock,
    List(bool),  // true = ordered
    Item,
    Table,
    TableHead,
    TableRow,
    TableCell,
    Emphasis,
    Strong,
    Strikethrough,
    Link,
    Image,
    FootnoteDefinition,
    MetadataBlock(MetadataBlockKind),
}
```

## Supporting Types

### HeadingLevel
```rust
pub enum HeadingLevel { H1, H2, H3, H4, H5, H6 }
```

### CodeBlockKind
```rust
pub enum CodeBlockKind<'a> {
    Indented,              // 4-space indented code
    Fenced(CowStr<'a>),    // ```lang - info string (language)
}
```

### LinkType
```rust
pub enum LinkType {
    Inline,                // [text](url)
    Reference,             // [text][label]
    ReferenceUnknown,      // [text][label] where label undefined
    Collapsed,             // [text][]
    CollapsedUnknown,      // [text][] where text undefined
    Shortcut,              // [text]
    ShortcutUnknown,       // [text] where text undefined
    Autolink,              // <url>
    Email,                 // <email@example.com>
}
```

### Alignment (Tables)
```rust
pub enum Alignment {
    None,
    Left,
    Center,
    Right,
}
```

### BlockQuoteKind
```rust
pub enum BlockQuoteKind {
    Note,
    Tip,
    Important,
    Warning,
    Caution,
}
```

## CowStr (Copy-on-Write String)

`CowStr<'a>` is optimized for zero-copy when possible:

```rust
// Usually borrows from input (no allocation)
let text: CowStr = cow_str;

// Convert to owned String when needed
let owned: String = text.into_string();

// Create from String
let cow: CowStr = "modified".to_string().into();
```

## Event Flow Pattern

Events follow depth-first traversal with balanced Start/End pairs:

```
# Heading          → Start(Heading{H1}) → Text("Heading") → End(Heading(H1))
Paragraph text     → Start(Paragraph) → Text("Paragraph text") → End(Paragraph)
- List item        → Start(List(None)) → Start(Item) → Text("List item") → End(Item) → End(List(false))
```

## Transformation Patterns

### Basic Map Transform
```rust
let transformed = parser.map(|event| match event {
    Event::SoftBreak => Event::HardBreak,
    other => other,
});
```

### Filter Events
```rust
// Remove all images
let no_images = parser.filter(|event| {
    !matches!(event, Event::Start(Tag::Image { .. }) | Event::End(TagEnd::Image))
});
```

### Stateful Transform (Custom Iterator)
```rust
struct MyTransform<I> {
    inner: I,
    in_heading: bool,
}

impl<'a, I: Iterator<Item = Event<'a>>> Iterator for MyTransform<I> {
    type Item = Event<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let event = self.inner.next()?;
        match &event {
            Event::Start(Tag::Heading { .. }) => self.in_heading = true,
            Event::End(TagEnd::Heading(_)) => self.in_heading = false,
            Event::Text(text) if self.in_heading => {
                return Some(Event::Text(text.to_uppercase().into()));
            }
            _ => {}
        }
        Some(event)
    }
}
```

### Accumulating Transform (Code Blocks)
```rust
let mut in_code = false;
let mut code_buf = String::new();
let mut lang = String::new();

let events: Vec<_> = parser.filter_map(|event| match event {
    Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(l))) => {
        in_code = true;
        lang = l.into_string();
        code_buf.clear();
        None
    }
    Event::Text(text) if in_code => {
        code_buf.push_str(&text);
        None
    }
    Event::End(TagEnd::CodeBlock) if in_code => {
        in_code = false;
        // Process code_buf and lang, return transformed event
        Some(Event::Html(format!("<pre data-lang=\"{lang}\">{code_buf}</pre>").into()))
    }
    other => Some(other),
}).collect();
```

## Handling Consecutive Text Events

Text may be split across multiple `Event::Text` events. Use `TextMergeStream` to merge them:

```rust
use pulldown_cmark::TextMergeStream;

let merged = TextMergeStream::new(parser);
for event in merged {
    if let Event::Text(text) = event {
        // `text` is now guaranteed to be a single merged string
    }
}
```

## Source Position Tracking

Get byte ranges for each event:

```rust
for (event, range) in parser.into_offset_iter() {
    let source_text = &markdown[range.clone()];
    println!("{:?} at {}..{}: {:?}", event, range.start, range.end, source_text);
}
```
