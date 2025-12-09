---
name: rust-on-npm
description: Expert knowledge for publishing Rust code to npm, covering napi-rs native addons, Neon bindings, WebAssembly via wasm-pack, and pure binary distribution patterns. Use when building npm packages with Rust, creating Node.js native addons, or distributing Rust CLIs through npm.
hash: b54c788763965b26
---

# Publishing Rust to the npm Ecosystem

Expert guidance for bringing Rust performance and safety to JavaScript/Node.js through npm packages.

## Distribution Patterns Overview

| Pattern | Use Case | Runtime | Portability |
|:--------|:---------|:--------|:------------|
| **Native Addon (napi-rs)** | Library with JS API, max performance | Node.js required | Per-platform binaries |
| **Native Addon (Neon)** | Library with JS API, mature ecosystem | Node.js required | Per-platform binaries |
| **WebAssembly** | Isomorphic (browser + Node), pure computation | Any JS runtime | Single portable artifact |
| **Binary Wrapper** | CLI tools, npm as delivery mechanism | None (native) | Per-platform binaries |

## Quick Decision Guide

**Choose napi-rs when:**
- You need a programmatic JS/TS API alongside CLI
- TypeScript-first development (auto-generated `.d.ts`)
- Maximum performance for Node.js native modules

**Choose Neon when:**
- You prefer a mature, well-documented ecosystem
- Need class macros and advanced async patterns
- Want comprehensive community support

**Choose WebAssembly when:**
- Code must run in browsers and Node.js
- Computation-heavy, minimal I/O (no filesystem/network)
- Single artifact simplicity is priority

**Choose Binary Wrapper when:**
- Building a CLI tool (formatter, linter, bundler)
- Rust is the source of truth, npm is just distribution
- No Node.js dependency at runtime

## Detailed Documentation

- [napi-rs Guide](./napi-rs.md) - Native addons with auto-generated TypeScript
- [Neon Guide](./neon.md) - Mature Rust-Node bindings framework
- [WebAssembly Guide](./wasm.md) - wasm-pack and browser/Node portability
- [Binary Distribution](./native-binary.md) - CLI tools via optionalDependencies
- [Code Examples](./examples.md) - Working examples for each pattern

## Common Patterns

### Platform-Specific npm Packages

All native approaches use npm's `optionalDependencies` + `os`/`cpu` fields:

```json
{
  "name": "@scope/package",
  "optionalDependencies": {
    "@scope/package-darwin-arm64": "1.0.0",
    "@scope/package-darwin-x64": "1.0.0",
    "@scope/package-linux-x64-gnu": "1.0.0",
    "@scope/package-win32-x64": "1.0.0"
  }
}
```

Each platform package declares constraints:
```json
{
  "name": "@scope/package-darwin-arm64",
  "os": ["darwin"],
  "cpu": ["arm64"]
}
```

### CI/CD Build Matrix

GitHub Actions matrix for cross-platform builds:
- `ubuntu-20.04` / `x86_64-unknown-linux-gnu`
- `ubuntu-20.04` / `aarch64-unknown-linux-gnu` (via cross)
- `macos-14` / `x86_64-apple-darwin`
- `macos-14` / `aarch64-apple-darwin`
- `windows-2022` / `x86_64-pc-windows-msvc`

## Key npm Mechanics

| Field | Purpose |
|:------|:--------|
| `bin` | Makes CLI available on `$PATH` after install |
| `optionalDependencies` | Platform packages - npm installs matching only |
| `os` / `cpu` | Platform constraints for conditional install |
| `postinstall` | Run script after install (for binary downloaders) |

## Tooling Shortcuts

**cargo-dist** - Automates binary releases with npm installer support:
```bash
cargo dist init  # Generates CI workflows
git push --tags  # Triggers release to npm
```

**@napi-rs/cli** - Full napi-rs scaffolding:
```bash
npm install -g @napi-rs/cli
napi new
```

**Neon** - Project generator:
```bash
npm init neon my-project
```

**wasm-pack** - WebAssembly toolchain:
```bash
wasm-pack build --target nodejs
```

## External Resources

- [napi-rs Documentation](https://napi.rs)
- [Neon Documentation](https://neon-bindings.com)
- [wasm-pack Documentation](https://rustwasm.github.io/wasm-pack/)
- [cargo-dist](https://opensource.axo.dev/cargo-dist/)
- [Orhun's npm Packaging Guide](https://blog.orhun.dev/packaging-rust-for-npm/)
