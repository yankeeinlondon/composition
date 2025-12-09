# Event Processing with pulldown-cmark

pulldown-cmark produces an event stream that gives fine-grained control over parsing.

## Basic Event Loop

```rust
use pulldown_cmark::{Parser, Event, Tag};

let markdown = "# Hello\n\nWorld **bold** text";
let parser = Parser::new(markdown);

for event in parser {
    match event {
        Event::Start(tag) => println!("Start: {:?}", tag),
        Event::End(tag) => println!("End: {:?}", tag),
        Event::Text(text) => println!("Text: {}", text),
        Event::Code(code) => println!("Code: {}", code),
        Event::SoftBreak => println!("Soft break"),
        Event::HardBreak => println!("Hard break"),
        Event::Rule => println!("Horizontal rule"),
        Event::TaskListMarker(checked) => println!("Task: {}", checked),
        _ => {}
    }
}
```

## Handling Consecutive Text Events

The parser may yield consecutive `Event::Text` chunks. Use `TextMergeStream` for easier handling:

```rust
use pulldown_cmark::{Parser, Event, TextMergeStream};

let markdown = "Hello world, this is ~~complicated~~ *simple* text.";
let iterator = TextMergeStream::new(Parser::new(markdown));

for event in iterator {
    match event {
        Event::Text(text) => println!("{}", text),
        _ => {}
    }
}
```

**When to use:** Always use `TextMergeStream` when you need complete text content without manual concatenation.

## Common Tag Types

### Block Tags

```rust
Tag::Heading(level, id, classes)  // H1-H6
Tag::Paragraph                     // <p>
Tag::BlockQuote                    // <blockquote>
Tag::CodeBlock(CodeBlockKind)      // <pre><code>
Tag::List(Option<u64>)            // <ul> or <ol>
Tag::Item                          // <li>
Tag::Table(alignments)             // <table>
Tag::TableHead                     // <thead>
Tag::TableRow                      // <tr>
Tag::TableCell                     // <td> or <th>
```

### Inline Tags

```rust
Tag::Emphasis                      // <em>
Tag::Strong                        // <strong>
Tag::Strikethrough                 // <del>
Tag::Link(link_type, url, title)   // <a>
Tag::Image(link_type, url, title)  // <img>
Tag::FootnoteDefinition(name)      // Footnote content
```

## Converting to HTML

```rust
use pulldown_cmark::{Parser, Options, html};

let mut options = Options::empty();
options.insert(Options::ENABLE_TABLES);
options.insert(Options::ENABLE_STRIKETHROUGH);

let parser = Parser::new_ext(markdown, options);
let mut html_output = String::new();
html::push_html(&mut html_output, parser);
```

**Performance tip:** Write to a buffered target (`String`, `Vec<u8>`) as the renderer makes many small writes.

## Collecting Events

```rust
let parser = Parser::new(markdown);
let events: Vec<Event> = parser.collect();

// Process collected events
for event in &events {
    // ...
}

// Render collected events
let mut html = String::new();
html::push_html(&mut html, events.into_iter());
```

## Event Filtering

```rust
let parser = Parser::new(markdown);
let filtered = parser.filter(|event| {
    !matches!(event, Event::SoftBreak)
});

let mut html = String::new();
html::push_html(&mut html, filtered);
```

## Broken Link Handling

```rust
use pulldown_cmark::{Parser, Options, BrokenLink, BrokenLinkCallback};

let callback: BrokenLinkCallback = Some(&mut |broken: BrokenLink| {
    Some(("fallback-url".into(), "fallback title".into()))
});

let parser = Parser::new_with_broken_link_callback(
    markdown,
    Options::empty(),
    callback
);
```

## Related

- [Advanced Techniques](./advanced-techniques.md) - Custom transformations
- [Parsing Strategy](./parsing-strategy.md) - Two-phase parsing overview
