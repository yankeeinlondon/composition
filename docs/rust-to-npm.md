---
name: rust-to-npm
description: Comprehensive guide to shipping Rust programs and libraries to npm using native binaries, napi-rs, Neon, and WebAssembly
created: 2025-12-08
hash: a52fe2a32c06aa55
tags:
  - rust
  - npm
  - node
  - napi-rs
  - neon
  - wasm
  - native-modules
  - cli
---

# Shipping Rust to npm

This guide covers the strategies, tools, and patterns for bringing Rust code to the npm ecosystem. Whether you are building a performance-critical CLI tool, a native Node.js addon with a JavaScript API, or a cross-platform library compiled to WebAssembly, this document provides the technical details you need.

## Table of Contents

- [Overview: Three Patterns](#overview-three-patterns)
- [Pattern 1: Native Rust Binaries](#pattern-1-native-rust-binaries)
- [Pattern 2: Native Node Addons with napi-rs](#pattern-2-native-node-addons-with-napi-rs)
- [Pattern 3: Native Node Addons with Neon](#pattern-3-native-node-addons-with-neon)
- [Pattern 4: WebAssembly with wasm-pack](#pattern-4-webassembly-with-wasm-pack)
- [Choosing the Right Approach](#choosing-the-right-approach)
- [npm Mechanics and Best Practices](#npm-mechanics-and-best-practices)
- [Quick Reference](#quick-reference)
- [Resources](#resources)

## Overview: Three Patterns

There are three primary approaches to shipping Rust code via npm:

| Pattern | Description | Best For |
|---------|-------------|----------|
| **Native Binaries** | Pure Rust CLI, npm is just a delivery mechanism | CLI tools, build tools, standalone utilities |
| **Native Addons** (napi-rs/Neon) | Rust as a Node-API addon with JS wrapper | Libraries with JS API, performance-critical modules |
| **WebAssembly** | Rust compiled to WASM with JS glue | Cross-platform libraries, browser + Node |

Each pattern has distinct tradeoffs in performance, distribution complexity, and integration depth.

## Pattern 1: Native Rust Binaries

This approach treats npm purely as a distribution channel. Users install via `npm install -g your-cli` or `npx your-cli@latest` and run a native Rust binary.

### Architecture: Base Package + Platform Packages

The standard pattern uses multiple npm packages:

- **Base package** (`your-cli`): Contains a JavaScript launcher and declares platform-specific packages as `optionalDependencies`
- **Platform packages** (`your-cli-linux-x64`, `your-cli-darwin-arm64`, etc.): Each contains a compiled binary and specifies `os`/`cpu` constraints

npm's resolver automatically installs only the matching platform package.

**Base package structure:**

```
your-cli/
  package.json     # Main package with optionalDependencies
  lib/index.js     # JavaScript launcher script
```

**Base package.json:**

```json
{
  "name": "your-cli",
  "version": "0.1.0",
  "bin": "lib/index.js",
  "type": "module",
  "optionalDependencies": {
    "your-cli-linux-x64": "0.1.0",
    "your-cli-linux-arm64": "0.1.0",
    "your-cli-darwin-x64": "0.1.0",
    "your-cli-darwin-arm64": "0.1.0",
    "your-cli-windows-x64": "0.1.0",
    "your-cli-windows-arm64": "0.1.0"
  }
}
```

**Platform package.json:**

```json
{
  "name": "your-cli-darwin-arm64",
  "version": "0.1.0",
  "os": ["darwin"],
  "cpu": ["arm64"],
  "files": ["bin/"]
}
```

### JavaScript Launcher Script

The launcher detects the platform and spawns the correct binary:

```javascript
#!/usr/bin/env node
import { spawnSync } from "node:child_process";
import os from "node:os";

function binaryPackageName() {
  const platform = os.platform();
  const arch = os.arch();
  const osName = platform === "win32" || platform === "cygwin" ? "windows" : platform;
  return `your-cli-${osName}-${arch}`;
}

function binaryPath() {
  const pkg = binaryPackageName();
  const exeSuffix = process.platform === "win32" || process.platform === "cygwin" ? ".exe" : "";
  try {
    return require.resolve(`${pkg}/bin/your-cli${exeSuffix}`);
  } catch {
    throw new Error(
      `No prebuilt binary found for ${process.platform}-${process.arch}. ` +
      `Tried to resolve ${pkg}/bin/your-cli${exeSuffix}`
    );
  }
}

function run() {
  const args = process.argv.slice(2);
  const result = spawnSync(binaryPath(), args, { stdio: "inherit" });
  process.exit(result.status ?? 0);
}

run();
```

### Building Binaries with CI

A typical GitHub Actions workflow builds for multiple targets:

```yaml
jobs:
  build:
    strategy:
      matrix:
        include:
          - os: ubuntu-20.04
            target: x86_64-unknown-linux-gnu
            npm_os: linux
            npm_arch: x64
          - os: ubuntu-20.04
            target: aarch64-unknown-linux-gnu
            npm_os: linux
            npm_arch: arm64
          - os: macos-14
            target: x86_64-apple-darwin
            npm_os: darwin
            npm_arch: x64
          - os: macos-14
            target: aarch64-apple-darwin
            npm_os: darwin
            npm_arch: arm64
          - os: windows-2022
            target: x86_64-pc-windows-msvc
            npm_os: windows
            npm_arch: x64
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - run: cargo build --release --locked --target ${{ matrix.target }}
      - name: Package npm binary
        run: |
          mkdir -p npm/your-cli-${{ matrix.npm_os }}-${{ matrix.npm_arch }}/bin
          cp target/${{ matrix.target }}/release/your-cli* npm/your-cli-${{ matrix.npm_os }}-${{ matrix.npm_arch }}/bin/
      - run: npm publish
        working-directory: npm/your-cli-${{ matrix.npm_os }}-${{ matrix.npm_arch }}
```

### cargo-dist: Automated Distribution

`cargo-dist` automates much of this workflow. It generates CI workflows, builds binaries, and publishes to npm:

```toml
# Cargo.toml
[workspace.metadata.dist]
installers = ["npm"]
npm-scope = "@your-scope"
```

Run `cargo dist init` to generate the release workflow, then push a version tag to trigger builds and publishing.

### Alternative: Binary Download at Install Time

Instead of multiple packages, publish binaries to GitHub Releases and download during `postinstall`:

```javascript
#!/usr/bin/env node
import { Binary } from "simple-binary-install";

const bin = new Binary("your-cli", {
  installDirectory: __dirname,
  url: ({ platform, arch }) =>
    `https://github.com/your-org/your-cli/releases/download/v0.1.0/your-cli-${platform}-${arch}.tar.gz`
});

async function main() {
  await bin.run(process.argv.slice(2));
}

main().catch(err => {
  console.error(err);
  process.exit(1);
});
```

Libraries like `binary-install` and `simple-binary-install` handle the download and caching logic.

## Pattern 2: Native Node Addons with napi-rs

napi-rs is a framework for building pre-compiled Node.js addons in Rust. It leverages Node-API (N-API) for ABI stability across Node versions.

### Key Features

| Feature | Description |
|---------|-------------|
| **Core Technology** | Builds on Node-API, Node.js's stable C API for native addons |
| **Build Process** | Compiles Rust to `.node` binary with platform-specific builds |
| **Type Support** | Auto-generates TypeScript definitions via `#[napi]` macro |
| **Distribution** | Publishes multiple npm packages per platform |
| **Platform Support** | Windows, macOS, Linux (glibc/musl), FreeBSD, Android |

napi-rs powers projects like SWC, Prisma, Polars, and Logseq.

### Project Setup

Install the CLI and create a new project:

```bash
npm install -g @napi-rs/cli
napi new
```

The CLI prompts for package name and target platforms. Using an npm scope (`@yourscope/core`) is recommended since napi-rs creates platform-specific packages.

**Generated structure:**

```
my-napi-project/
  src/lib.rs           # Rust source with #[napi] attributes
  index.js             # JavaScript entry point (auto-generated)
  index.d.ts           # TypeScript definitions (auto-generated)
  package.json         # Package configuration
  Cargo.toml           # crate-type = ["cdylib"]
  build.rs             # Build script
  .github/workflows/   # CI workflow for builds
```

### Writing Functions

Use the `#[napi]` macro to export functions:

```rust
use napi_derive::napi;

#[napi]
pub fn sum(a: i32, b: i32) -> i32 {
    a + b
}
```

This generates TypeScript:

```typescript
export function sum(a: number, b: number): number;
```

### Exposing Classes

Rust structs become JavaScript classes:

```rust
use napi_derive::napi;
use napi::bindgen_prelude::*;

#[napi]
pub struct Transformer {
    inner: Uint8Array,
}

#[napi]
impl Transformer {
    #[napi(constructor)]
    pub fn new(inner: Uint8Array) -> Self {
        Self { inner }
    }

    #[napi]
    pub fn to_webp(&self) -> Result<Uint8Array> {
        // Image processing logic
        Ok(processed_data)
    }
}
```

### Handling Callbacks

napi-rs supports type-safe JavaScript callbacks:

```rust
use napi::bindgen_prelude::*;

#[napi]
pub fn process_data(
    data: String,
    callback: Function<String, String>
) -> Result<String> {
    let processed = data.to_uppercase();
    callback.call(processed)
}
```

For callbacks with multiple arguments:

```rust
#[napi]
pub fn calculate(
    base: f64,
    callback: Function<FnArgs<(f64, f64, String)>, f64>
) -> Result<f64> {
    let tax = base * 0.2;
    let bonus = 1000.0;
    let dept = "Engineering".to_string();
    callback.call((base, tax, dept).into())
}
```

### Async Operations

Return promises for async work:

```rust
use napi::{Env, PromiseRaw};

#[napi(ts_return_type = "Promise<void>")]
pub fn schedule_task<'env>(
    env: &'env Env,
    delay_ms: u32,
    callback: Function<'env, String, ()>
) -> Result<PromiseRaw<'env, ()>> {
    let callback_ref = callback.create_ref()?;
    env.spawn_future_with_callback(
        async move {
            tokio::time::sleep(std::time::Duration::from_millis(delay_ms as u64)).await;
            Ok("Task complete!".to_string())
        },
        move |env, message| {
            let callback = callback_ref.borrow_back(env)?;
            callback.call(message)?;
            Ok(())
        }
    )
}
```

> **Note:** JavaScript functions passed to Rust only live within the current function scope. For async operations, create a `FunctionRef` to keep the function alive.

### Building and Testing

```bash
# Development build
napi build --platform

# Release build
napi build --release

# Test in Node
node -e "const { sum } = require('./'); console.log(sum(40, 2))"
```

### Publishing

1. Create an npm scope at npmjs.com
2. Rename your package: `napi rename`
3. Add `NPM_TOKEN` to GitHub repository secrets
4. Push a version tag:

```bash
npm version patch
git push --follow-tags
```

The generated GitHub Actions workflow builds for all platforms and publishes automatically.

### The Orphan Rule Challenge

Exposing types from third-party crates requires wrapper structs due to Rust's orphan rule:

```rust
// Cannot implement ToNapiValue for external_crate::SomeType directly
// Solution: create a wrapper
pub struct MyWrapper(external_crate::SomeType);

#[napi]
impl MyWrapper {
    // Implement conversion methods
}
```

## Pattern 3: Native Node Addons with Neon

Neon is an alternative framework for Rust-based Node.js addons, established since 2017.

### Key Differences from napi-rs

| Feature | napi-rs | Neon |
|---------|---------|------|
| TypeScript Generation | Automatic | Manual |
| Performance | Excellent | Excellent (slightly lower in some benchmarks) |
| API Style | Macro-based (`#[napi]`) | Context-based (`FunctionContext`) |
| Ecosystem | Growing rapidly | More mature, comprehensive docs |

### Project Setup

```bash
npm init neon my-project
```

This creates a hybrid directory that is both a Rust crate and an npm package:

```
my-project/
  src/lib.rs      # Rust source
  package.json    # npm configuration
  Cargo.toml      # Rust configuration
  index.js        # JavaScript entry point
```

### Writing Functions

Neon functions take a `FunctionContext` and return a `JsResult`:

```rust
use neon::prelude::*;

fn hello(mut cx: FunctionContext) -> JsResult<JsString> {
    Ok(cx.string("hello from Rust"))
}

fn add(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let a = cx.argument::<JsNumber>(0)?;
    let b = cx.argument::<JsNumber>(1)?;
    let sum = a.value(&mut cx) + b.value(&mut cx);
    Ok(cx.number(sum))
}
```

### Module Entry Point

Export functions via the `#[neon::main]` attribute:

```rust
#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("hello", hello)?;
    cx.export_function("add", add)?;
    Ok(())
}
```

### Using Rust Crates

Add dependencies to `Cargo.toml` and use them directly:

```toml
[dependencies]
neon = { version = "1.0.0", features = ["napi-6"] }
num_cpus = "1"
```

```rust
use neon::prelude::*;

fn get_cpu_count(mut cx: FunctionContext) -> JsResult<JsNumber> {
    Ok(cx.number(num_cpus::get() as f64))
}
```

### Safe Parallelism

Neon provides `vm::lock()` for safe multi-threading with Node.js buffers:

```rust
use neon::prelude::*;
use rayon::prelude::*;

fn parallel_process(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let buffer = cx.argument::<JsBuffer>(0)?;

    // Lock prevents JS from running while we access the buffer
    let result = cx.borrow(&buffer, |data| {
        data.as_slice()
            .par_chunks(1024)
            .map(|chunk| process_chunk(chunk))
            .sum()
    });

    Ok(cx.number(result))
}
```

### Performance Example

Word counting across Shakespeare's plays:

- Pure JavaScript: ~280-290ms
- Rust (single-threaded): ~80-85ms
- Rust with Rayon (parallel): ~50ms

### Building and Publishing

```bash
# Build
npm run build

# Release build
neon build --release
```

```json
{
  "files": ["index.js", "native/index.node", "package.json"],
  "scripts": {
    "prepublishOnly": "neon build --release"
  }
}
```

### Error Handling

Convert Rust errors to JavaScript exceptions:

```rust
fn risky_operation(mut cx: FunctionContext) -> JsResult<JsString> {
    match some_operation() {
        Ok(value) => Ok(cx.string(value)),
        Err(err) => cx.throw_error(&format!("Operation failed: {}", err))
    }
}
```

## Pattern 4: WebAssembly with wasm-pack

For pure computation that needs to run in browsers and Node.js, compile to WebAssembly.

### Setup

```bash
cargo install wasm-pack
wasm-pack new my-wasm-lib
```

### Writing WASM-Compatible Code

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn process(input: &str) -> String {
    input.to_uppercase()
}

#[wasm_bindgen]
pub fn run_cli(args: Vec<String>) -> i32 {
    // CLI logic
    0
}
```

### Building

```bash
wasm-pack build --target nodejs
```

This produces:

```
pkg/
  your_crate_bg.wasm    # WebAssembly binary
  your_crate.js         # JavaScript glue
  package.json          # npm metadata
```

### CLI Wrapper

```javascript
#!/usr/bin/env node
import { run_cli } from "your-crate-wasm";

async function main() {
  const args = process.argv.slice(2);
  const exitCode = await run_cli(args);
  process.exit(exitCode ?? 0);
}

main().catch(err => {
  console.error(err);
  process.exit(1);
});
```

### Tradeoffs

| Aspect | WebAssembly | Native Addons |
|--------|-------------|---------------|
| Portability | Single artifact works everywhere | Platform-specific binaries |
| Performance | Good, with sandboxing overhead | Excellent, direct system access |
| System Access | Limited (no FS, processes) | Full access |
| Threading | Emerging support | Full support |
| Distribution | Simple | Complex (multi-platform) |

## Choosing the Right Approach

### Decision Matrix

| Use Case | Recommended Approach |
|----------|---------------------|
| CLI tool, Node optional | Native binaries + cargo-dist |
| CLI tool + JS API | napi-rs with CLI wrapper |
| Performance-critical library | napi-rs or Neon |
| Browser + Node library | WebAssembly |
| Heavy parallelism | Neon (mature threading) |
| Best TypeScript DX | napi-rs (auto-generated types) |

### Combination A: Tooling-Style CLI

For formatters, linters, transpilers where Rust is the source of truth:

- **Rust**: Standard bin crate
- **Distribution**: cargo-dist with npm installer or Orhun-style platform packages
- **Pros**: Simple runtime, predictable performance, no Node ABI concerns

### Combination B: CLI + Rich JS API

For tools that also expose a programmatic API:

- **Rust**: Library with `#[napi]` exports
- **Distribution**: napi-rs CLI + GitHub Actions
- **Pros**: Single npm artifact provides both CLI and API

### Combination C: Isomorphic Library

For libraries that run in browsers and Node:

- **Rust**: Library compiled with wasm-pack
- **Distribution**: wasm-pack builds npm package
- **Pros**: Single WASM file, maximum portability

## npm Mechanics and Best Practices

### The `bin` Field

```json
{
  "bin": {
    "your-cli": "lib/index.js",
    "yc": "lib/index.js"
  }
}
```

### Platform Filtering

```json
{
  "optionalDependencies": {
    "your-cli-darwin-arm64": "0.1.0"
  }
}
```

Each platform package declares:

```json
{
  "os": ["darwin"],
  "cpu": ["arm64"]
}
```

### Post-Install Scripts

For binary downloaders:

```json
{
  "scripts": {
    "postinstall": "node postinstall.mjs"
  }
}
```

### Version Synchronization

Keep Rust crate version and npm package version in sync for maintainability. Use distribution tags for pre-releases:

```bash
npm dist-tag add your-cli@0.2.0 beta
npm install your-cli@beta
```

### Performance Tips

- **Minimize boundary crossings**: Batch operations to reduce JS-Rust calls
- **Use release builds**: Always `--release` for production
- **Efficient data structures**: Leverage zero-copy where possible
- **Pre-allocate memory**: Reuse buffers in hot paths

## Quick Reference

### napi-rs Commands

```bash
npm install -g @napi-rs/cli  # Install CLI
napi new                      # Create project
napi build --platform         # Development build
napi build --release          # Production build
napi rename                   # Rename package
```

### Neon Commands

```bash
npm init neon my-project      # Create project
npm run build                 # Development build
neon build --release          # Production build
```

### cargo-dist Commands

```bash
cargo install cargo-dist      # Install
cargo dist init               # Initialize project
cargo dist build              # Build locally
git tag v0.1.0 && git push --tags  # Trigger release
```

### wasm-pack Commands

```bash
cargo install wasm-pack       # Install
wasm-pack new my-lib          # Create project
wasm-pack build --target nodejs  # Build for Node
wasm-pack build --target web     # Build for browser
wasm-pack publish             # Publish to npm
```

### Type Mappings (napi-rs)

| Rust Type | JavaScript Type |
|-----------|-----------------|
| `i32`, `u32`, `f64` | `number` |
| `String` | `string` |
| `bool` | `boolean` |
| `Vec<T>` | `Array<T>` |
| `Option<T>` | `T \| null` |
| `Result<T, E>` | `T` (throws on error) |
| `Uint8Array` | `Uint8Array` |
| `Buffer` | `Buffer` |

## Resources

### Official Documentation

- [napi-rs](https://napi.rs) - napi-rs documentation and examples
- [Neon](https://neon-bindings.com) - Neon bindings documentation
- [wasm-pack](https://rustwasm.github.io/wasm-pack/) - wasm-pack documentation
- [cargo-dist](https://opensource.axo.dev/cargo-dist/) - cargo-dist documentation

### Tools

- [@napi-rs/cli](https://www.npmjs.com/package/@napi-rs/cli) - napi-rs CLI tool
- [binary-install](https://www.npmjs.com/package/binary-install) - Binary download helper
- [simple-binary-install](https://www.npmjs.com/package/simple-binary-install) - Lighter binary helper

### Example Projects

- [SWC](https://github.com/swc-project/swc) - TypeScript/JavaScript compiler using napi-rs
- [Prisma](https://github.com/prisma/prisma) - Database ORM using napi-rs
- [@napi-rs/tar](https://github.com/nicholasknudson/napi-tar) - Tar archive handling
- [Pomsky](https://github.com/pomsky-lang/pomsky) - Example using cargo-dist npm installer
