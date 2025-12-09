# Rust Code Highlighting - Crate Comparison

## Feature Comparison

| Crate | Grammar Tech | Batteries Included | Output Modes | Maintenance |
|-------|--------------|-------------------|--------------|-------------|
| `syntect` | TextMate | Syntaxes + themes | HTML, ANSI, custom | Mature, stable |
| `tree-sitter-highlight` | Tree-sitter | No (bring your own) | Custom only | Active |
| `pepegsitter` | Tree-sitter | Parsers + queries | Custom only | Helper crate |
| `inkjet` | Tree-sitter | 70+ langs, themes | HTML, ANSI | **Archived** |
| `autumnus` | Tree-sitter | 70+ langs, 100+ themes | HTML, ANSI | Active, recommended |

## TextMate (syntect) vs Tree-sitter

### Accuracy & Structure

- **Tree-sitter**: Full AST, semantic classification
- **TextMate**: Regex-based, usually "good enough" but can break on tricky syntax

### Ecosystem

- **TextMate grammars**: Widespread (VS Code, Sublime)
- **Tree-sitter grammars**: Default in modern editors (Neovim, Helix)

### Performance

- **Tree-sitter**: Shines with incremental re-highlighting (editors)
- **Both**: Fine for static/batch highlighting

## DIY vs Batteries-Included Tree-sitter

### DIY (tree-sitter-highlight + pepegsitter)

- Maximum control over queries, languages, rendering
- You own HTML/ANSI generation and theming
- More complexity

### Batteries-Included (autumnus)

- Minimal code: call `highlight()`, get output
- Accept crate's theme model and language roster
- Win for 99% of use cases

## Binary Size & Build Time

Tree-sitter bundling crates (inkjet, autumnus):
- Increase compile time
- Increase binary size (multi-MB grammars)
- These are C parsers, not just data files

syntect:
- Also non-tiny (many grammars + themes)
- Data files more than C code

## Decision Matrix

| Scenario | Recommended |
|----------|-------------|
| New project, want highlighting fast | `autumnus` |
| Building an editor/IDE | `tree-sitter-highlight` |
| Need VS Code grammar compatibility | `syntect` |
| Existing inkjet codebase | Migrate to `autumnus` |
| Need smallest binary | `syntect` or roll-your-own |
| Maximum theme options | `autumnus` (Neovim themes) |
