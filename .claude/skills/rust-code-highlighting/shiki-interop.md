# Shiki and syntect Interoperability

Both Shiki (JS/TS) and syntect (Rust) use TextMate grammars. They can share grammar and theme files but don't plug into each other directly.

## What They Share

| Asset | Shiki | syntect |
|-------|-------|---------|
| Grammars | `.tmLanguage.json` | `.tmLanguage`, `.sublime-syntax` |
| Themes | VS Code JSON | `.tmTheme` (plist) |
| Engine | Oniguruma / JS RegExp | Oniguruma via `onig` crate |

## Using Same Grammar in Both

### Shiki (Node/TS)

```typescript
import { getHighlighter } from 'shiki';

const highlighter = await getHighlighter({
  themes: ['vitesse-dark', './themes/my-theme.json'],
  langs: ['ts', './syntaxes/foo.tmLanguage.json'],
});

const html = await highlighter.codeToHtml('let x: Foo = 1;', {
  lang: 'foo',
  theme: 'my-theme',
});
```

### syntect (Rust)

```rust
use syntect::parsing::SyntaxSetBuilder;
use syntect::highlighting::ThemeSet;
use syntect::easy::HighlightLines;

// Load same grammar
let mut builder = SyntaxSetBuilder::new();
builder.add_from_folder("./syntaxes", true)?;
let ps = builder.build();

// syntect prefers .tmTheme - convert VS Code JSON once
let ts = ThemeSet::load_from_folder("./themes")?;
let theme = &ts.themes["my-theme"];

let syntax = ps.find_syntax_by_extension("foo").unwrap();
let mut h = HighlightLines::new(syntax, theme);
```

## Theme Format Notes

- Shiki: VS Code JSON themes directly
- syntect: `.tmTheme` (plist) natively
- Conversion tools exist for VS Code → tmTheme

## Running syntect from JavaScript

The `@syntect/node` package wraps syntect for Node.js/WASM:

```typescript
import { highlight } from '@syntect/node';

const html = await highlight({
  code: 'fn main() {}',
  extension: 'rs',
  theme: 'base16-ocean.dark',
});
```

Use cases:
- CSS-class-based output instead of Shiki's inline styles
- Reuse Rust-side grammar packs
- Experiment with syntect behavior vs Shiki

## What You Cannot Do

- No "Shiki calls syntect" adapter
- No shared token stream protocol between them
- No "Shiki plugin for syntect" or vice versa

## Practical Strategies

### 1. Shared Assets, Separate Engines

Use same TextMate grammars/themes in both:
- Shiki for web doc site
- syntect for CLI/TUI tool
- Ensures visual parity

### 2. JS-Only, Dual Engines

- Shiki as default for web content
- `@syntect/node` when you need:
  - CSS-class output instead of inline styles
  - Different behavior characteristics

### 3. Rust-First, Shiki Optional

- Do all highlighting in Rust (syntect/autumnus)
- Render syntect's HTML in browser/Node
- Shiki becomes redundant unless you need its transformer ecosystem
