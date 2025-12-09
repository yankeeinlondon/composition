# mdast Performance: Benchmarks and Optimization

Performance characteristics and benchmarks for mdast parsing in TypeScript (remark) vs Rust (markdown-rs).

## Key Findings

| Aspect | TypeScript (remark) | Rust (markdown-rs) |
|--------|--------------------|--------------------|
| **Constant Factor** | Baseline | ~2-4x faster |
| **Asymptotic Complexity** | O(n) typical, O(n^2) edge cases | Same |
| **Memory Usage** | Higher (GC overhead) | Lower, predictable |
| **Build Tool Impact** | 50-70% of MDX build time | Significant reduction |

## Benchmark Data

### markdown-rs Issue #113 (MDX Stress Test)

Direct comparison using identical synthetic MDX documents:

**Setup:**
```javascript
// JS: mdast-util-from-markdown + micromark-extension-mdxjs
import { fromMarkdown } from "mdast-util-from-markdown";
import { mdxjs } from "micromark-extension-mdxjs";
fromMarkdown(file, { extensions: [mdxjs()] });
```

```rust
// Rust: markdown::to_mdast with MDX options
let mdast = markdown::to_mdast(&content, &markdown::ParseOptions::mdx())?;
```

**Results:**
- Rust consistently below JS curve (better constant factors)
- Both exhibit similar asymptotic behavior on pathological inputs
- At large sizes (10MB+ MDX), both can hit multi-second parse times

**Caveat:** This benchmark uses adversarial MDX (giant code blocks, random UUIDs). Normal blog/docs content performs significantly better.

### Micromark vs Old remark-parse (JS vs JS)

The current `micromark`-based parser is ~1.5x slower than the previous hand-rolled parser, prioritizing correctness over speed.

**Mental model:**
```
old remark-parse < micromark/mdast-util-from-markdown (1.5x) < markdown-rs (faster by constant factor)
```

## Real-World Impact

### Docusaurus/MDX Build Profiling

From MDX compilation profiles:
- 50-70% of total build time spent in parsing phase
- `mdast-util-from-markdown` identified as "expensive task" in CPU profiles
- Teams actively evaluating markdown-rs/mdxjs-rs for performance gains

### Inkdrop Editor

After switching to mdast-based parser:
- Users reported keyboard lag on long notes
- Developer considering Rust implementation for performance
- Trade-off: performance vs JS plugin extensibility

## Optimization Strategies

### For TypeScript/JavaScript

1. **Cache parsed ASTs** - Parse once, transform multiple times
2. **Lazy parsing** - Only parse visible/needed content
3. **Limit plugin chains** - Each plugin adds overhead
4. **Use remark-stringify** - Avoid re-parsing modified markdown

```typescript
// Cache pattern
const astCache = new Map<string, Root>();

function getAst(content: string, hash: string): Root {
  if (!astCache.has(hash)) {
    astCache.set(hash, remark().parse(content));
  }
  return astCache.get(hash)!;
}
```

### For Rust

1. **Stream processing** - Use event-based parsing for huge files
2. **Parallel parsing** - Process multiple files concurrently
3. **Avoid unnecessary AST** - Use `to_html()` directly if no transformation needed

```rust
// Direct HTML when AST not needed
let html = markdown::to_html_with_options(&content, &Options::gfm())?;

// Parallel file processing
use rayon::prelude::*;
let asts: Vec<_> = files.par_iter()
    .map(|f| markdown::to_mdast(f, &ParseOptions::gfm()))
    .collect();
```

### Hybrid Approach

For best of both worlds:

```typescript
// Use Rust for parsing (via WASM or native addon)
import { parseToMdast } from 'markdown-rs-wasm';

// Use JS for transformation (rich plugin ecosystem)
import { visit } from 'unist-util-visit';

const ast = parseToMdast(markdown);  // Fast Rust parsing
visit(ast, 'link', transformLink);    // Flexible JS transform
```

## When Performance Matters

### High Priority

- Static site generators with 1000s of pages
- Real-time collaborative editors
- Server-side rendering under load
- CLI tools processing large codebases

### Lower Priority

- Single-page documentation
- Client-side blog rendering
- Build tools with caching
- Low-traffic applications

## Recommended Benchmarking

For your specific workload:

```typescript
// JS benchmark
import { fromMarkdown } from 'mdast-util-from-markdown';
import { gfm } from 'micromark-extension-gfm';

const start = performance.now();
for (const file of files) {
  fromMarkdown(file, { extensions: [gfm()] });
}
console.log(`JS: ${performance.now() - start}ms`);
```

```rust
// Rust benchmark
use std::time::Instant;

let start = Instant::now();
for file in &files {
    markdown::to_mdast(file, &ParseOptions::gfm())?;
}
println!("Rust: {:?}", start.elapsed());
```

**Metrics to capture:**
- Time to parse (ms/MB)
- Peak memory usage (RSS)
- Throughput (files/second)

## Summary

- **Rust is faster** by a constant factor (typically 2-4x)
- **Both share similar algorithmic complexity** - Rust won't solve O(n^2) edge cases
- **JS ecosystem is richer** - trade performance for extensibility
- **Cache aggressively** in JS to mitigate parsing cost
- **Profile your actual workload** - benchmarks vary by content type

## Related

- [TypeScript vs Rust](./ts-vs-rust.md) - Feature comparison
- [Ecosystem Overview](./ecosystem.md) - Available tools and plugins
