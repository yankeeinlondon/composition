# Rust Frontmatter Crate Comparison

## Detailed Analysis

### 1. `markdown-frontmatter`

**Overview**: A dedicated, type-safe frontmatter parser designed specifically for extracting metadata from Markdown files.

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

**Best For**: General-purpose frontmatter extraction with type safety.

---

### 2. `gray_matter`

**Overview**: A Rust implementation of the popular JavaScript gray-matter library, offering maximum flexibility.

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

**Best For**: Projects needing custom delimiters, excerpt extraction, or non-standard frontmatter formats.

---

### 3. `yaml_front_matter`

**Overview**: A minimal crate focused solely on YAML frontmatter parsing.

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

**Best For**: Projects exclusively using YAML frontmatter that want minimal dependencies.

---

### 4. `markdown-rs` (markdown crate)

**Overview**: A full-featured, CommonMark-compliant Markdown parser with frontmatter as an extension.

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

**Best For**: Static site generators, documentation tools, or renderers where you need to process both frontmatter and Markdown content.

---

### 5. `fronma`

**Overview**: A minimal frontmatter parser supporting multiple formats via feature flags.

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
let result = fronma::parse::<T>(doc)?;
let headers = result.headers;
let body = result.body;
```

**Best For**: Lightweight projects that need basic multi-format support.

---

### 6. `markdown-it-front-matter`

**Overview**: A plugin for the markdown-it parser ecosystem.

**Supported Formats**: YAML, TOML, JSON

**Pros**:
- Seamless integration with markdown-it
- Extensible with custom parsers
- Returns frontmatter as part of AST

**Cons**:
- Requires markdown-it as dependency
- Less straightforward for standalone frontmatter extraction
- Plugin architecture adds complexity

**Best For**: Projects already using markdown-it for Markdown processing.

---

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
