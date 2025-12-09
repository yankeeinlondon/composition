# Rust Frontmatter Crate Comparison

Detailed analysis of Rust crates for parsing frontmatter metadata from Markdown files.

## Feature Comparison Matrix

| Feature | markdown-frontmatter | gray_matter | yaml_front_matter | markdown-rs | fronma |
|:--------|:-------------------:|:-----------:|:-----------------:|:-----------:|:------:|
| YAML support | Feature | Feature | Built-in | Feature | Default |
| TOML support | Feature | Feature | No | Feature | Feature |
| JSON support | Feature | Feature | No | No | Feature |
| Custom delimiters | No | Yes | No | No | No |
| Excerpt parsing | No | Yes | No | No | No |
| Full Markdown parsing | No | No | No | Yes | No |
| Serde integration | Yes | Yes | Yes | Partial | Yes |
| Optional frontmatter | Yes | Yes | No | Yes | Yes |

## Detailed Analysis

### markdown-frontmatter

**Best for**: General-purpose frontmatter extraction with type safety.

**Supported Formats**: YAML, TOML, JSON (via Cargo features)

**Pros**:
- Type-safe parsing with full Serde integration
- Explicit handling of documents with no frontmatter (treats as empty)
- Feature-gated formats keep dependencies minimal
- Simple, focused API - does one thing well
- Clear error messages for malformed frontmatter

**Cons**:
- No features enabled by default - must specify format(s)
- Limited to basic frontmatter parsing (no excerpts)
- Only parses frontmatter; does not process Markdown body

**API Pattern**:
```rust
let (frontmatter, body) = markdown_frontmatter::parse::<T>(doc)?;
```

---

### gray_matter

**Best for**: Projects needing custom delimiters, excerpt extraction, or non-standard frontmatter formats.

**Supported Formats**: YAML, TOML, JSON

**Pros**:
- Custom delimiters (e.g., `~~~` instead of `---`)
- Excerpt parsing with configurable delimiters (e.g., `<!-- endexcerpt -->`)
- Extensible engine trait for custom parsers
- Flexible output: generic `Pod` type or custom structs
- Mature, battle-tested approach from JS ecosystem

**Cons**:
- More complex API due to flexibility
- Requires feature flags for each format
- Slightly higher learning curve

**API Pattern**:
```rust
let matter = Matter::<YAML>::new();
let result = matter.parse(doc);
let data: T = result.data.unwrap().deserialize()?;
let content = result.content;
let excerpt = result.excerpt;
```

---

### yaml_front_matter

**Best for**: Projects exclusively using YAML frontmatter that want minimal dependencies.

**Supported Formats**: YAML only

**Pros**:
- Simple and lightweight for YAML-only use cases
- Full Serde integration for type-safe parsing
- Clear error handling
- Minimal dependencies

**Cons**:
- Only supports YAML (no TOML or JSON)
- Less flexible than other options
- Fewer features overall

**API Pattern**:
```rust
let document = YamlFrontMatter::parse::<T>(doc)?;
let metadata = document.metadata;
let content = document.content;
```

---

### markdown-rs (markdown crate)

**Best for**: Static site generators, documentation tools, or renderers where you need to process both frontmatter and Markdown content.

**Supported Formats**: YAML, TOML (via feature flags)

**Pros**:
- Complete Markdown parsing and rendering in one crate
- Highly robust and security-conscious
- CommonMark and GFM compliant
- Frontmatter integrates with full AST

**Cons**:
- Overkill if you only need frontmatter extraction
- More complex API for simple metadata extraction
- Larger dependency footprint

---

### fronma

**Best for**: Lightweight projects that need basic multi-format support.

**Supported Formats**: YAML (default), TOML, JSON

**Pros**:
- Feature-gated formats reduce bloat
- Simple API similar to markdown-frontmatter
- Minimal dependencies
- Returns structured output with `headers` and `body`

**Cons**:
- Less actively maintained
- Limited documentation
- Smaller community

**API Pattern**:
```rust
let result = fronma::parser::parse::<T>(doc)?;
let headers = result.headers;
let body = result.body;
```

---

### markdown-it-front-matter

**Best for**: Projects already using markdown-it for Markdown processing.

**Supported Formats**: YAML, TOML, JSON

**Pros**:
- Seamless integration with markdown-it
- Extensible with custom parsers
- Returns frontmatter as part of AST

**Cons**:
- Requires markdown-it as dependency
- Less straightforward for standalone frontmatter extraction
- Plugin architecture adds complexity

## Choosing the Right Crate

**Start with `markdown-frontmatter` if:**
- You need multi-format support
- You want a simple, type-safe API
- You're building a static site generator or blog engine

**Choose `gray_matter` if:**
- You need custom delimiters
- You want excerpt extraction
- You're porting a project from JavaScript gray-matter

**Choose `yaml_front_matter` if:**
- You only use YAML frontmatter
- Minimal dependencies are critical
- You want the simplest possible API

**Choose `markdown-rs` if:**
- You need full Markdown parsing
- You're building a complete rendering pipeline
- Frontmatter is just one part of larger Markdown processing

**Choose `fronma` if:**
- You want a lightweight alternative
- You need basic multi-format support
- You prefer feature-gated format selection
