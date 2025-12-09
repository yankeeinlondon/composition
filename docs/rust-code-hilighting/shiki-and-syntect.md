Short answer: there’s no "drop in syntect into Shiki” adapter, but they are interoperable at the asset level (grammars & themes), and you can run syntect from the JS world alongside Shiki.

Let me break down the practical options.

⸻

## 1. What they have in common

Both Shiki and syntect:
 • Use TextMate grammars for tokenization.  ￼
 • Use VS Code/TextMate-style themes (.tmTheme, VS Code JSON) for coloring.  ￼

So while Shiki and syntect don’t plug into each other directly, they can share the same grammar and theme files.

Conceptually:
 • Shiki = TS/JS TextMate engine (Oniguruma / JS RegExp) → HTML (inline styles, HAST).  ￼
 • syntect = Rust TextMate engine (Oniguruma via onig crate) → your choice of HTML/ANSI/etc.  ￼

So the “integration point” is:

Use the same grammar/theme definitions in both stacks, not one engine inside the other.

⸻

## 2. Sharing grammars/themes between syntect and Shiki

### 2.1. Using a VS Code / TextMate grammar in both

Say you’ve got a foo language with:
 • Grammar: foo.tmLanguage.json
 • Theme: my-theme.json (VS Code theme)

In Shiki (Node/TS):

```ts
import { getHighlighter } from 'shiki';

const highlighter = await getHighlighter({
  themes: ['vitesse-dark', './themes/my-theme.json'],
  langs: ['ts', './syntaxes/foo.tmLanguage.json'],
});

const html = await highlighter.codeToHtml('let x: Foo = 1;', {
  lang: 'foo',
  theme: 'my-theme',
});

console.log(html);
```
￼

In syntect (Rust):

```rust
use syntect::parsing::SyntaxSetBuilder;
use syntect::highlighting::{ThemeSet, Theme};
use syntect::easy::HighlightLines;

fn main() -> anyhow::Result<()> {
    // Build SyntaxSet from your own TextMate grammar.
    let mut builder = SyntaxSetBuilder::new();
    builder.add_from_folder("./syntaxes", true)?; // contains foo.tmLanguage(.json)
    let ps = builder.build();

    // Load theme. syntect likes .tmTheme, but you can convert VSCode JSON
    // to tmTheme (a one-time conversion step).
    let ts = ThemeSet::load_from_folder("./themes")?;
    let theme: &Theme = &ts.themes["my-theme"];

    let syntax = ps.find_syntax_by_extension("foo").unwrap();
    let mut h = HighlightLines::new(syntax, theme);

    let code = "let x: Foo = 1;";

    for line in syntect::util::LinesWithEndings::from(code) {
        let ranges = h.highlight_line(line, &ps)?;
        let escaped = syntect::util::as_24_bit_terminal_escaped(&ranges, false);
        print!("{escaped}");
    }

    Ok(())
}
```

Key point: both are reading the same TextMate grammar, so tokens/scopes should mostly line up. The differences will mainly be:
 • What regex engine is used (Oniguruma vs transpiled JS regex).  ￼
 • How themes map scopes to colors.

### 2.2. Theme-sharing caveats
 • Shiki typically uses VS Code JSON themes directly.  ￼
 • syntect’s official format is .tmTheme (plist), but people regularly:
 • Convert VS Code themes → .tmTheme via tooling.
 • Or just use bundles that already exist (e.g. load_defaults()).

So in practice, “using Shiki themes in syntect” = “convert VS Code JSON to .tmTheme once, then load in syntect”.

## 3. Running syntect in a JS/TS environment (next to Shiki)

If what you really want is “I’m in Shiki land (Node/TS) but want to call syntect”, there is a way:
 • There’s a Node/WASM wrapper for syntect:
 • syntect-js / @syntect/node exposes syntect to Node.js and WebAssembly.  ￼

From its README (simplified):

import { highlight } from '@syntect/node';

const html = await highlight({
  code: 'fn main() {}',
  extension: 'rs',
  theme: 'base16-ocean.dark',
});

console.log(html); // HTML with CSS classes

So you can absolutely:
 • Use Shiki for some paths (e.g. docs site build).
 • Use syntect via @syntect/node for others (e.g. when you want class-based HTML instead of inline styles, or reuse Rust-side grammar packs).

But that’s still two independent engines, not “Shiki calling out to syntect”.

⸻

## 4. What you cannot (currently) do

As of now there is:
 • No official adapter that makes Shiki treat syntect as a drop-in backend.
 • No shared “token stream” protocol where Shiki produces tokens and syntect just renders them, or vice versa.
 • No “Shiki plugin for syntect” or “syntect plugin for Shiki” in either ecosystem.

If you wanted something like:

“Use Shiki’s grammar loaders, but run the actual highlighting in Rust”

you’d basically be re-implementing Shiki’s TextMate engine on the Rust side (which is more or less what syntect already is) and then writing glue around it. At that point you’re no longer using Shiki so much as reproducing its capabilities with the same assets.

⸻

## 5. Practical strategies

Given all that, here are realistic patterns:

 1. Shared assets, separate engines
 • Use the same TextMate grammars and themes with Shiki (in JS) and syntect (in Rust).
 • Good when you want visual parity between, say, a web doc site (Shiki) and a CLI/TUI (syntect).
 2. JS-only, dual engines
 • Keep Shiki as your “default” highlighter for web content.
 • Use @syntect/node when you want:
 • CSS-class–based output instead of inline styles,
 • or to experiment with syntect’s behavior vs Shiki’s.
 3. Rust-first, Shiki as an optional frontend
 • Do all highlighting in Rust (syntect).
 • In a browser/Node context, just render syntect’s HTML output; Shiki is then redundant, unless you need its transformers ecosystem.

