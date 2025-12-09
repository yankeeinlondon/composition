# markdown-rs vs pulldown-cmark

Both crates are solid, but they have very different philosophies. Roughly:

- **pulldown-cmark**: low-level, streaming, battle-tested workhorse (used by rustdoc, docs.rs, etc.).
- **markdown-rs**: spec-obsessed, AST-centric, CommonMark+GFM+MDX+math with "safe HTML by default" and an mdast tree (used on crates.io).

I'll walk through differences, then call out which one I'd pick for specific use-cases.

---

## 1. High-level comparison

### pulldown-cmark

- **Model**: pull parser – Parser is an `Iterator<Item = Event>`. You transform the event stream and optionally feed it into `html::push_html`.
- **Goals**: fast, minimal allocations, safe Rust (plus an optional SIMD feature), and 100% CommonMark compliant.
- **Extensions**: tables, task lists, strikethrough, footnotes, admonitions, LaTeX-style math, heading IDs/classes, metadata blocks, wikilinks.
- **Output**: iterator of events + optional HTML renderer; you build your own AST if you want one.

**Basic usage:**

```rust
use pulldown_cmark::{Parser, Options, html};

let mut options = Options::empty();
options.insert(Options::ENABLE_STRIKETHROUGH);

let markdown_input = "Hello, ~~world~~!";
let parser = Parser::new_ext(markdown_input, options);

let mut html_output = String::new();
html::push_html(&mut html_output, parser);
```

### markdown-rs (markdown crate)

- **Model**: state machine (`#![no_std]` + alloc) that emits tokens and then either:
  - compiles directly to HTML (`to_html` / `to_html_with_options`), or
  - builds an mdast syntax tree (`to_mdast`).
- **Goals**:
  - 100% CommonMark compliance, plus 100% GFM and MDX support, plus frontmatter & math.
  - "100% safe Rust, also 100% safe HTML by default" – raw HTML is escaped unless you opt in.
  - Heavy emphasis on test coverage and spec-faithfulness (650 CM tests + >1k extra + fuzzing).
- **Output**: HTML string, or mdast AST (compatible with the JS unified/micromark ecosystem).

**Basic usage:**

```rust
fn main() {
    println!("{}", markdown::to_html("## Hi, *Saturn*! "));
}
```

**With GFM and options:**

```rust
fn main() -> Result<(), markdown::message::Message> {
    let html = markdown::to_html_with_options(
        "* [x] contact ~Mercury~Venus at <hi@venus.com>!",
        &markdown::Options::gfm(),
    )?;
    println!("{html}");
    Ok(())
}
```

---

## 2. Feature & API comparison

### Parsing model & API surface

| Aspect | pulldown-cmark | markdown-rs |
|--------|----------------|-------------|
| Core API | `Parser: Iterator<Item=Event>` plus `html::push_html` | `to_html`, `to_html_with_options`, `to_mdast` functions + Options/ParseOptions/CompileOptions |
| Architecture | Pull parser, streaming, no AST by default | State machine → events → HTML or mdast AST |
| Source positions | `into_offset_iter()` gives `(Event, Range)` pairs for source mapping | mdast nodes carry position data with line/column/offset |
| AST | You roll your own (or collect events into a tree) | First-class mdast tree with a dedicated module |
| no_std | Not advertised as no_std | Implemented as `#![no_std]` + alloc |

If you like "iterator of events, map/filter/fold" style, pulldown-cmark feels idiomatic. If you want "parse to a well-known AST and then manipulate that", markdown-rs is a nicer fit.

### Extensions & dialects

**pulldown-cmark:**

- Base CommonMark + optional extensions via Options:
  - Tables, task lists, strikethrough, footnotes, admonitions, LaTeX-style math, heading attributes, metadata blocks, wikilinks.

**markdown-rs:**

- Base CommonMark (100% compliance) + extensions:
  - GFM: autolink literals, footnotes, strikethrough, tables, tagfilter, task lists
  - MDX: ESM, expressions, JSX
  - Frontmatter
  - Math

If you need MDX or want parity with the JS micromark/mdast/unified ecosystem, markdown-rs is clearly ahead; it is explicitly tied to that ecosystem.

### HTML safety

**markdown-rs:**

- `to_html()` is explicitly documented as "safe way to transform (untrusted?) markdown into HTML"; raw HTML is not allowed unless you configure it.
- `to_html_with_options()` lets you enable "dangerous" HTML and other constructs if you really want them.

**pulldown-cmark:**

- Focus is on correct parsing and efficient rendering; not an HTML sanitizer.
- The community guidance is to pair it with something like ammonia if you're handling untrusted input.
- There's even an issue asking for "disallowed raw HTML" as an extension which is "not planned", reinforcing that "HTML safety" is out of scope.

So if you want something that behaves like "Markdown + built-in sanitization defaults", markdown-rs is more opinionated in that direction.

### Performance & memory

Both are designed to be fast and safe, but with a different emphasis:

**pulldown-cmark:**

- Design goal: "Fast; a bare minimum of allocation and copying", written in pure Rust with optional SIMD acceleration.
- Text events are copy-on-write slices of the original input in most cases – allocation is minimized.
- Widely regarded as one of the fastest Rust markdown parsers; external write-ups often call it out specifically when memory use is critical.

**markdown-rs:**

- Also optimized, with a full set of benches in the repo, but its focus is on spec correctness + rich AST + extensions rather than being the minimal-possible parser.
- The state machine tokenization + mdast building is more work than streaming events only; in practice it's still plenty fast for typical web / CLI use, but pulldown-cmark will generally win on "bytes → events → HTML" micro-benchmarks when you don't need ASTs.

**Short version:** for pure throughput and low memory, pulldown-cmark still has the edge; for feature-rich correctness and AST manipulation, markdown-rs spends more cycles but gives you more structure.

### Ecosystem & maturity

**pulldown-cmark:**

- Around since 2015, used in rustdoc and other core tools, so it's extremely battle-hardened.
- Has its own guide site, with detailed docs, spec links, and extension specs.

**markdown-rs:**

- Newer, but built by the wooorm / unified ecosystem author (micromark, mdast, mdxjs, etc.), which gives it an unusually strong cross-language story.
- 1.0.0 shipped with ~100% code coverage and extensive fuzzing.

**Ecosystem trade-off in practice:**

- Need compatibility with existing Rust libraries & prior art → pulldown-cmark.
- Need compatibility with JS tooling, or plan to share ASTs/specs across languages → markdown-rs.

---

## 3. Code examples in context

### pulldown-cmark: transforming events

Say you want to:

- Turn soft breaks into `<br>`
- Drop inline HTML entirely (quick-n-dirty "safer" mode)

```rust
use pulldown_cmark::{Parser, Event, Options, html};

fn render_safeish(markdown_input: &str) -> String {
    let parser = Parser::new_ext(markdown_input, Options::all());

    let filtered = parser.map(|event| match event {
        Event::SoftBreak => Event::HardBreak,
        Event::Html(_) => Event::Text("".into()),
        other => other,
    });

    let mut html_output = String::new();
    html::push_html(&mut html_output, filtered);
    html_output
}
```

You can also get source ranges:

```rust
use pulldown_cmark::Parser;

let parser = Parser::new("## Title");
for (event, range) in parser.into_offset_iter() {
    println!("{event:?} from {} to {}", range.start, range.end);
}
```

### markdown-rs: working with mdast

Parse to AST and inspect:

```rust
use markdown::{to_mdast, ParseOptions};
use markdown::mdast::{Node, Heading, Text};

fn find_headings(doc: &str) -> Result<Vec<String>, markdown::message::Message> {
    let tree = to_mdast(doc, &ParseOptions::default())?;
    let mut headings = Vec::new();

    fn walk(node: &Node, out: &mut Vec<String>) {
        match node {
            Node::Heading(Heading { children, .. }) => {
                let mut text = String::new();
                for child in children {
                    if let Node::Text(Text { value, .. }) = child {
                        text.push_str(value);
                    }
                }
                out.push(text);
            }
            _ => node.children().for_each(|c| walk(c, out)),
        }
    }

    walk(&tree, &mut headings);
    Ok(headings)
}
```

Opting into MDX+GFM:

```rust
use markdown::{Options, to_html_with_options};

fn render_mdx(input: &str) -> Result<String, markdown::message::Message> {
    let mut options = Options::gfm(); // enables GFM bundle
    options.constructs().mdx_expression = true;
    options.constructs().mdx_jsx = true;

    to_html_with_options(input, &options)
}
```

---

## 4. "Which should I use for X?" – concrete use-cases

### A. High-throughput markdown → HTML (e.g. docs site, blog engine)

- **Primary goal**: speed, low memory, stable behavior.
- **Recommendation**: pulldown-cmark.
  - It's tuned for throughput, has SIMD, and is already proven at scale (e.g., rustdoc).
  - Stream processing means you can build a "filter → HTML" pipeline with little overhead.

If you don't care about ASTs and just want HTML quickly and predictably, pulldown-cmark is the easy win.

### B. User-generated content where security is a concern (comments, forum posts)

- **Goal**: correctness + avoid XSS.
- **Options**:
  - **markdown-rs**:
    - `to_html()` gives you safe HTML by default – raw HTML is treated as text.
    - You still may want a sanitizer in front of your templating if you're extremely paranoid, but it reduces attack surface.
  - **pulldown-cmark + Ammonia**:
    - Use pulldown-cmark for parsing, then run ammonia on the output, as typical Rust forum/blog examples do.

If you want an out-of-the-box "Markdown, no raw HTML" behavior with fewer moving parts, markdown-rs is more ergonomic. If you're already comfortable with a sanitizer pipeline (markdown → ammonia → template), pulldown-cmark works great.

### C. Rich Markdown/MDX content for a React-style frontend

- **Goal**: MDX (JSX in markdown), frontmatter, math; ideally, shared AST semantics with JS.
- **Recommendation**: markdown-rs, no contest.
  - It fully supports MDX (ESM, expressions, JSX) and frontmatter, and exposes an mdast AST that matches the JS ecosystem.
  - There's even a sibling mdxjs-rs project for compiling MDX.

Pulldown-cmark doesn't aim at MDX; it's "Markdown + GFM bits", not a JSX host language.

### D. Static analysis / linting / refactoring based on markdown structure

- **Goal**: walk an AST, rewrite nodes, compute structure-aware metrics (heading hierarchy, link graph, etc.).
- **Recommendation**: markdown-rs.
  - You get mdast nodes with positions and types out of the box.
  - Easier to express "transform headings to IDs", "extract link targets", "build ToC" as tree transforms instead of event filters.

You can build an AST atop pulldown-cmark events, but you'd be re-doing work markdown-rs already does.

### E. WASM / memory-constrained environments / no_std

- **Goal**: small footprint, maybe no std, but still need full CommonMark/GFM/MDX features.
- **Recommendation**: markdown-rs is attractive because it's explicitly `#![no_std]` + alloc.
  - In practice, both are used in WASM, but if you're chasing no_std correctness (embedded, constrained runtime, etc.), markdown-rs is built with that in mind.

### F. IDE/editor-like live previews where latency and streaming matter

- **Goal**: partial re-parsing, incremental-ish behavior, streaming to UI.
- **Recommendation**:
  - **pulldown-cmark** if you want to stream events as the user types and do minimal allocations.
  - **markdown-rs** if the editor needs an AST for structural operations (folding, outline, semantic tools) and you can afford the extra work per document.

For something like "live preview on each keystroke" or server-side streaming of HTML, pulldown's iterator story is very natural.

### G. Tooling that wants cross-language consistency

For example, you have a JS pipeline using unified/remark and a Rust backend that should interpret markdown exactly the same way.

- **Goal**: avoid "this renders differently between JS and Rust".
- **Recommendation**: markdown-rs.
  - It intentionally mirrors micromark/mdast semantics.

---

## 5. Strengths & weaknesses summary

### pulldown-cmark

**Strengths:**

- Extremely fast and memory-efficient; optional SIMD.
- Simple, idiomatic iterator API; great for streaming transforms.
- Widely deployed (rustdoc, etc.), well-understood behavior.
- Good set of GFM-adjacent extensions including math, admonitions, wikilinks, and heading attributes.

**Weaknesses:**

- No built-in AST; you have to build your own if you need one.
- Not an HTML sanitizer; requires pairing with something like ammonia for untrusted input.
- No MDX/JSX support; its scope is Markdown+GFM, not "markdown as a host language".

### markdown-rs (markdown)

**Strengths:**

- 100% CommonMark + 100% GFM + full MDX + math + frontmatter.
- Safe HTML by default; explicit opt-in for dangerous HTML.
- First-class mdast AST with precise positional info; great for analysis and transforms.
- `no_std` + alloc design, heavy test coverage + fuzzing.

**Weaknesses:**

- Newer; less "institutional history" than pulldown-cmark in the Rust ecosystem.
- For pure "markdown → HTML" throughput with no bells/whistles, it's generally not going to beat a purpose-built streaming parser like pulldown-cmark.
- API is more opinionated (`to_html`/`to_mdast`); if you only want a low-level event stream, pulldown-cmark's "just an iterator" is simpler.

---

## 6. My rule-of-thumb recommendation

If I had to boil it down:

**Pick pulldown-cmark when:**

- You want the fastest/leanest markdown → HTML path.
- You're comfortable pairing it with a sanitizer for untrusted input.
- You prefer working with iterators/events and don't need a shared AST with JS tools.

**Pick markdown-rs when:**

- You care about MDX, or want mdast ASTs for non-trivial transforms.
- You like having safe HTML by default and opt-in dangerous HTML.
- You want better alignment with the unified/micromark world, or you're building cross-language tooling.

If you tell me your concrete project (e.g., "SSG for docs", "multi-language MDX+Rust pipeline", "WASM live preview", etc.), I can give you a more opinionated "X + Y crate combo" recommendation on top of these two.
