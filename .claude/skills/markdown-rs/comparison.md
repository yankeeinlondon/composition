# markdown-rs vs pulldown-cmark

A detailed comparison of Rust's two main Markdown parsers to help you choose the right one for your project.

## Philosophy

| Aspect | markdown-rs | pulldown-cmark |
|--------|-------------|----------------|
| **Design** | Spec-obsessed, AST-centric | Fast, streaming, minimal |
| **Model** | State machine → tokens → AST/HTML | Pull parser (iterator of events) |
| **Primary goal** | 100% CommonMark + GFM + MDX compliance | Speed with minimal allocations |
| **Safety default** | Safe HTML by default | Raw HTML passed through |

## Architecture Comparison

### markdown-rs Flow

```txt
Markdown Input
     ↓
State Machine Tokenizer
     ↓
Token Stream (with positions)
     ↓
Event Stream
     ↓
┌────────────┬─────────────┐
│ to_html()  │ to_mdast()  │
│ HTML out   │ Full AST    │
└────────────┴─────────────┘
```

### pulldown-cmark Flow

```txt
Markdown Input
     ↓
Pull Parser
     ↓
Iterator<Item = Event>
     ↓
┌─────────────────┬────────────────┐
│ html::push_html │ Custom handler │
│ HTML output     │ (you decide)   │
└─────────────────┴────────────────┘
```

## API Comparison

### markdown-rs

```rust
// Simple
let html = markdown::to_html("# Hello");

// With options
let html = markdown::to_html_with_options(
    "* [x] task",
    &markdown::Options::gfm(),
)?;

// Get AST
let ast = markdown::to_mdast(
    "# Title",
    &markdown::ParseOptions::default(),
)?;
```

### pulldown-cmark

```rust
use pulldown_cmark::{Parser, Options, html};

// Simple
let parser = Parser::new("# Hello");
let mut html = String::new();
html::push_html(&mut html, parser);

// With options
let mut opts = Options::empty();
opts.insert(Options::ENABLE_STRIKETHROUGH);
opts.insert(Options::ENABLE_TABLES);

let parser = Parser::new_ext("| a | b |", opts);
let mut html = String::new();
html::push_html(&mut html, parser);
```

## Feature Comparison

| Feature | markdown-rs | pulldown-cmark |
|---------|-------------|----------------|
| CommonMark compliance | 100% | High |
| GFM tables | Yes | Yes |
| GFM strikethrough | Yes | Yes |
| GFM task lists | Yes | Yes |
| GFM autolinks | Yes | No |
| Footnotes | Yes | Yes |
| MDX/JSX | Yes | No |
| Math notation | Yes | Yes |
| Frontmatter | Yes | Via extension |
| Wikilinks | No | Yes |
| Heading attributes | No | Yes |
| `#![no_std]` | Yes | No |
| Built-in AST | Yes (mdast) | No (roll your own) |
| Safe HTML by default | Yes | No |

## Performance

### Memory

- **pulldown-cmark**: Lower memory - streaming events, no full tree materialization
- **markdown-rs**: Higher memory - builds complete AST with position info

### Throughput

- **pulldown-cmark**: Generally faster for pure markdown→HTML (designed as "bare minimum of allocation and copying")
- **markdown-rs**: More overhead but provides richer output

### Benchmark Context

pulldown-cmark is optimized for throughput with:
- Minimal allocations
- Optional SIMD acceleration
- Zero-copy string handling where possible
- Used in rustdoc, docs.rs - proven at scale

markdown-rs prioritizes:
- Correctness (650+ CommonMark tests, 1000+ additional, fuzzing)
- Rich positional information
- Complete AST for manipulation
- 100% code coverage

**Rule of thumb**: For pure "bytes → events → HTML" throughput, pulldown-cmark wins. For feature-rich correctness and AST manipulation, markdown-rs is worth the extra cycles.

## When to Use markdown-rs

1. **You need AST access**
   ```rust
   // Easy document analysis
   let ast = to_mdast(content, &ParseOptions::gfm())?;
   let links = extract_all_links(&ast);
   ```

2. **You need MDX support**
   ```rust
   // With mdxjs-rs for JSX compilation
   let options = Options {
       parse: ParseOptions {
           constructs: Constructs {
               mdx_expression: true,
               mdx_jsx: true,
               ..Constructs::gfm()
           },
           ..ParseOptions::default()
       },
       ..Options::default()
   };
   ```

3. **Security is a priority**
   ```rust
   // Safe by default - raw HTML escaped
   let safe_html = to_html(untrusted_input);
   ```

4. **Cross-language consistency**
   - Matches micromark/mdast behavior in JavaScript
   - Same AST format as unified ecosystem

5. **`no_std` environments**
   - Works with just `alloc`
   - Suitable for embedded/WASM

## When to Use pulldown-cmark

1. **Maximum throughput**
   ```rust
   // Streaming is efficient
   let parser = Parser::new_ext(large_doc, Options::all());
   html::push_html(&mut output, parser);
   ```

2. **Stream transformations**
   ```rust
   // Filter/map events elegantly
   let parser = Parser::new(input);
   let filtered = parser.map(|event| match event {
       Event::SoftBreak => Event::HardBreak,
       e => e,
   });
   html::push_html(&mut output, filtered);
   ```

3. **Memory-constrained environments**
   - No full AST materialization
   - Events processed and discarded

4. **Rustdoc compatibility**
   - Used in rustdoc itself
   - Battle-tested at scale

5. **Streaming/live preview**
   - Natural fit for incremental rendering
   - Events available as parsed

## Decision Matrix

| Use Case | Recommendation | Rationale |
|----------|----------------|-----------|
| Static site generator | pulldown-cmark | Speed for processing many docs |
| Documentation system | pulldown-cmark | Rustdoc compatibility, battle-tested |
| CMS with user content | markdown-rs | Safe HTML by default |
| MDX processing | markdown-rs | Only Rust option with full MDX |
| Document analysis/linting | markdown-rs | First-class AST with positions |
| Live preview editor | pulldown-cmark | Streaming events for incremental updates |
| Academic writing (math) | Either | Both support LaTeX-style math |
| WASM/no_std | markdown-rs | `#![no_std]` + alloc support |
| JS ecosystem integration | markdown-rs | Matches micromark/mdast semantics |
| High-throughput API | pulldown-cmark | Minimal allocations, proven at scale |
| Cross-language tooling | markdown-rs | Same AST format as unified ecosystem |

## Migration Between Libraries

### pulldown-cmark → markdown-rs

```rust
// Before: pulldown-cmark
use pulldown_cmark::{Parser, Options, html};

let mut opts = Options::empty();
opts.insert(Options::ENABLE_TABLES);
opts.insert(Options::ENABLE_STRIKETHROUGH);

let parser = Parser::new_ext(input, opts);
let mut html = String::new();
html::push_html(&mut html, parser);

// After: markdown-rs
use markdown::{to_html_with_options, Options};

let html = to_html_with_options(input, &Options::gfm())?;
```

### markdown-rs → pulldown-cmark

```rust
// Before: markdown-rs
use markdown::to_html;

let html = to_html(input);

// After: pulldown-cmark
use pulldown_cmark::{Parser, html};

let parser = Parser::new(input);
let mut html = String::new();
html::push_html(&mut html, parser);

// Note: You may need to add HTML sanitization
// if processing untrusted input
```

## Combining Both

For some projects, using both makes sense:

```rust
// Use pulldown-cmark for fast rendering
fn render_trusted_docs(input: &str) -> String {
    let parser = pulldown_cmark::Parser::new(input);
    let mut html = String::new();
    pulldown_cmark::html::push_html(&mut html, parser);
    html
}

// Use markdown-rs for analysis or untrusted content
fn analyze_document(input: &str) -> DocumentStats {
    let ast = markdown::to_mdast(input, &markdown::ParseOptions::gfm()).unwrap();
    extract_stats(&ast)
}

fn render_user_content(input: &str) -> String {
    // Safe by default
    markdown::to_html(input)
}
```

## Summary

**Choose markdown-rs when:**
- You need the AST for manipulation/analysis
- Safety matters (untrusted input)
- MDX support is required
- Cross-language consistency with JS tooling
- Working in `no_std` environments

**Choose pulldown-cmark when:**
- Performance is the top priority
- You prefer streaming/iterator patterns
- Memory efficiency is critical
- Rustdoc compatibility matters
- You're building event-based transformations
