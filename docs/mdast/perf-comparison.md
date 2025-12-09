---
_fixed: true
---

# MDAST Performance: Rust vs JavaScript Comparison

There are a couple of real "Rust vs JS mdast" comparisons out there, but nothing like a big, polished benchmark suite. What exists is:

- One explicit micro-benchmark that directly compares `markdown::to_mdast` (Rust) with `mdast-util-from-markdown` + `micromark-extension-mdxjs` (JS)
- Several profiles/issues showing where remark/mdast-util-from-markdown dominate CPU time
- Anecdotal statements that a Rust port (markdown-rs) would "greatly improve performance"

## Direct Rust vs JS mdast Benchmark (markdown-rs issue #113)

The only truly apples-to-apples comparison I could find is in [markdown-rs issue #113](https://github.com/wooorm/markdown-rs/issues/113): "Performance: larger MDX files are unmanageably slow to parse".

They benchmark exactly what you asked about:

**JS side**: `fromMarkdown` from `mdast-util-from-markdown` with the MDX extension:

```javascript
import { fromMarkdown } from "mdast-util-from-markdown";
import { mdxjs } from "micromark-extension-mdxjs";

const file = generateTestData(size); // synthetic giant MDX document
const start = performance.now();
fromMarkdown(file, { extensions: [mdxjs()] });
const end = performance.now();
```

With dependencies:

```json
{
  "dependencies": {
    "mdast-util-from-markdown": "^2.0.0",
    "micromark-extension-mdxjs": "^3.0.0"
  }
}
```

**Rust side**: `markdown::to_mdast` from markdown-rs with MDX parse options:

```rust
let mdast = markdown::to_mdast(
    &result,
    &markdown::ParseOptions::mdx()
)?;
```

In a loop they generate the same synthetic MDX structure (tons of `<DummyComponent code={...} />` with huge randomly generated code segments) and measure elapsed time with `Instant::now()`.

### Benchmark Summary

> "The constant factors are better, but the asymptotic complexity means that we're back to 60-second parse times on files that are just an order of magnitude bigger than the ones that caused 60-second parses in micromark-js."

And:

> "For comparison, the 'JS' lines on these graphs show micromark's performance using subtokenize 2.0.1..."

### Takeaways

- Rust (`markdown::to_mdast`) is a constant-factor faster than JS (`mdast-util-from-markdown` + `micromark-extension-mdxjs`) for this workload - the graphs (which compare both) show the Rust curve consistently below the JS curve, and the author explicitly calls out better constant factors.
- Both implementations exhibit roughly the same asymptotic behavior on these pathological MDX documents (close to quadratic). At large sizes they both hit tens of seconds and become unusable.
- This benchmark is very MDX-specific and adversarial (giant embedded code blocks, random UUIDs, etc.), so it's a worst-case-ish stress test, not "normal blog markdown".

**Conclusion**: If you're just comparing mdast generation on identical MDX with identical semantics, the best current data says: `markdown-rs` is faster by some constant factor, but not magically O(n) where JS is O(n²).

---

## Evidence from MDX/Docusaurus Profiling

On the JS/TS side, there's an MDX issue about compiling MDAST to JSX for Docusaurus that includes concrete profiling notes:

- In a Docusaurus/MDX static site, 50-70% of total build time was spent in the parsing phase (remark-parse / mdast-util-from-markdown + MDX utilities).
- The reporter calls out `mdast-util-from-markdown` as "an expensive task" in the CPU profile.
- They explicitly point to `markdown-rs` and `mdxjs-rs` as Rust ports that could improve performance:

> "...has been reimplemented in Rust and could help us improve performances:
> - [mdxjs-rs](https://github.com/wooorm/mdxjs-rs)
> - [markdown-rs](https://github.com/wooorm/markdown-rs)"

This isn't a benchmark yet (no "X ms vs Y ms"), but it does show:

- For large MDX sites, mdast parsing in JS is the dominant cost.
- Folks are actively eyeing Rust mdast implementations as a way to bring that down.

---

## Micromark vs Old Remark-Parse Core (JS vs JS)

This one is intra-JS, but it's useful context for expectations.

In a micromark discussion about performance, the maintainer notes:

> "micromark is currently slower than what used to be in remark-parse. About 50% slower."

The key points from that thread:

- `micromark` (which now underpins `remark-parse` and `mdast-util-from-markdown`) is ~1.5x slower than the previous hand-rolled remark parser, in raw parsing time.
- They emphasize that in most real apps, plugin work on the AST dominates, not the raw parse, so that 50% regression usually isn't what users feel.
- The priority for `micromark` is correctness and spec compliance over absolute speed.

Put together with the Rust data:

- JS moved from "fast but looser" parsing to "slightly slower but more correct" (`micromark` + `mdast-util-from-markdown`).
- Rust `markdown-rs` implements essentially the same algorithmic model (sibling project; mdast output) but gets a constant-factor win from being native code.

So your rough mental model of mdast generation cost could be:

```
old remark-parse < micromark/mdast-util-from-markdown (≈1.5x) < same semantics in Rust (markdown-rs) but with better constant factors
```

Exact multipliers will depend heavily on your CPU, input, and extensions (GFM vs MDX vs math).

---

## Anecdotal JS vs Rust Comparisons in the Wild

A couple of places talk about performance in less formal ways:

### Inkdrop Forum: Keyboard Lag After Switching to mdast-Based Parser

- Inkdrop moved its markdown engine to the newer `mdast-util-from-markdown`-based stack, and users reported slower keystrokes / editor lag on long notes.
- The author there links to `markdown-rs` and notes that a Rust implementation could "greatly improve performance," with the trade-off of losing JS-side extensibility.

This again reinforces the idea: mdast generation is real work, and moving that part to Rust is attractive if you can live without the JS plugin ecosystem.

### General Markdown Parser Surveys

Blog posts that catalogue parsers (JS: micromark/remark; Rust: markdown-rs, comrak, pulldown-cmark, etc.) generally call out markdown-rs as the Rust sibling to micromark that targets CommonMark/GFM/MDX with an mdast AST. They don't publish numbers, but they implicitly group markdown-rs with other "high-performance" Rust parsers.

---

## What We Don't Have (and How to Benchmark It)

I did not find:

- A published, neutral "markdown-rs vs remark/mdast" benchmark suite covering:
  - Normal markdown (docs/blog posts)
  - GFM quirks
  - MDX with realistic JSX, not just adversarial enormous code fences
- Any cross-language benchmark with concrete multipliers like "Rust is 4.3× faster on 10 MB of CommonMark".

The only direct numbers are buried in the graphs of issue #113, which aren't text-parseable, and in the micromark intra-JS comparison (~50% slower than old remark core).

### Recommended Benchmarking Approach

If you want hard data tailored to your workload, crib directly from that issue's harness:

- **JS side (Node)**: use `fromMarkdown` with whatever extensions you actually need (gfm, MDX, math, directives, etc.).
- **Rust side**: use `markdown::to_mdast` with matching options (`ParseOptions::gfm`, `::mdx`, etc.).
- **Measure all of**:
  - Time to parse (ms/MB)
  - Peak RSS / heap
  - Maybe run with realistic batch sizes (number of files in a build)

You'll likely see something like:

- **Rust mdast (markdown-rs)**: noticeably lower wall-clock time and lower CPU per document, especially for large batches.
- **JS mdast (remark/mdast-util-from-markdown)**: still "fast enough" for many use cases, but can dominate build/SSR time when you have lots of MDX + heavy plugin chains.

---

## Practical Takeaways

Given the limited—but non-zero—data:

### If You're Already in Rust and Just Need mdast

`markdown-rs::to_mdast` is the obvious choice. You get spec-correct parsing, mdast output, and better constant factors than the JS tools.

### If You're in a JS/TS Stack and Care More About Perf Than Plugins

Embedding markdown-rs via a small Rust service or via WASM is a reasonable route if mdast generation is the hot path (Docusaurus/MDX build style workloads). Expect some constant-factor speedup; how big depends on your input.

### If You Lean Heavily on the Remark/Rehype Plugin Ecosystem

Today, JS remark + mdast-util-from-markdown is still the practical choice; the Rust side doesn't yet give you the same dynamic plugin story, and you'll end up paying IPC/WASM overhead to get back into JS to run plugins.

---

## Summary

There is one real benchmark and several strong signals that Rust's markdown-rs wins on constant factor for mdast generation, but you're still looking at the same broad algorithmic profile as the JS mdast/remark ecosystem rather than some order-of-magnitude asymptotic improvement.
