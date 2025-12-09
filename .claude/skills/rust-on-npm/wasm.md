# WebAssembly Guide

Compile Rust to WebAssembly for isomorphic code that runs in browsers, Node.js, Deno, and edge runtimes from a single artifact.

## When to Use WebAssembly

**Ideal for:**
- Pure computation (parsers, compressors, cryptography)
- Browser + Node.js from same code
- Edge runtimes (Cloudflare Workers, Vercel Edge)
- Avoiding platform-specific compilation

**Avoid when:**
- Heavy filesystem or network I/O
- Maximum native performance required
- Deep Node.js integration needed

## Key Tools

| Tool | Purpose |
|:-----|:--------|
| `wasm-pack` | Build toolchain for Rust → WASM |
| `wasm-bindgen` | Generate JS/TS bindings |
| `wasm-opt` | Optimize WASM binary size |
| `web-sys` | Browser API bindings |
| `js-sys` | JavaScript standard library bindings |

## Project Setup

### Install wasm-pack

```bash
cargo install wasm-pack
```

### Create Project

```bash
cargo new --lib my-wasm-lib
cd my-wasm-lib
```

### Cargo.toml Configuration

```toml
[package]
name = "my-wasm-lib"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
wasm-bindgen = "0.2"
js-sys = "0.3"      # Optional: JS standard library
web-sys = "0.3"     # Optional: Browser APIs

[profile.release]
opt-level = "s"      # Optimize for size
lto = true           # Link-time optimization
```

## Writing WASM Code

### Basic Function Export

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

#[wasm_bindgen]
pub fn fibonacci(n: u32) -> u32 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}
```

### Exposing Structs

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Parser {
    content: String,
}

#[wasm_bindgen]
impl Parser {
    #[wasm_bindgen(constructor)]
    pub fn new(content: String) -> Parser {
        Parser { content }
    }

    pub fn parse(&self) -> JsValue {
        // Return JavaScript object
        serde_wasm_bindgen::to_value(&self.parse_internal()).unwrap()
    }
}
```

### Working with JavaScript Types

```rust
use wasm_bindgen::prelude::*;
use js_sys::{Array, Object, Reflect};

#[wasm_bindgen]
pub fn process_array(arr: Array) -> Array {
    let result = Array::new();
    for i in 0..arr.length() {
        let val = arr.get(i);
        // Process and push to result
        result.push(&val);
    }
    result
}

#[wasm_bindgen]
pub fn create_object() -> Object {
    let obj = Object::new();
    Reflect::set(&obj, &"key".into(), &"value".into()).unwrap();
    obj
}
```

### Calling JavaScript from Rust

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    #[wasm_bindgen(js_namespace = Math)]
    fn random() -> f64;
}

#[wasm_bindgen]
pub fn do_something() {
    log("Called from Rust!");
    let r = random();
    log(&format!("Random: {}", r));
}
```

### Async Functions

```rust
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, Response};

#[wasm_bindgen]
pub async fn fetch_data(url: String) -> Result<JsValue, JsValue> {
    let opts = RequestInit::new();
    opts.set_method("GET");

    let request = Request::new_with_str_and_init(&url, &opts)?;
    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: Response = resp_value.dyn_into()?;
    let json = JsFuture::from(resp.json()?).await?;

    Ok(json)
}
```

## Building

### Build Targets

```bash
# For Node.js (CommonJS)
wasm-pack build --target nodejs

# For bundlers (Webpack, Vite, etc.)
wasm-pack build --target bundler

# For browsers (ES modules)
wasm-pack build --target web

# For Deno
wasm-pack build --target deno
```

### Output Structure

```
pkg/
├── my_wasm_lib_bg.wasm      # Compiled WASM binary
├── my_wasm_lib_bg.wasm.d.ts # WASM TypeScript types
├── my_wasm_lib.js           # JavaScript glue code
├── my_wasm_lib.d.ts         # TypeScript definitions
├── package.json             # npm package metadata
└── README.md
```

## Using in JavaScript

### Node.js

```javascript
const { greet, fibonacci } = require('./pkg/my_wasm_lib');

console.log(greet("World"));     // "Hello, World!"
console.log(fibonacci(10));       // 55
```

### Browser (ES Modules)

```javascript
import init, { greet, fibonacci } from './pkg/my_wasm_lib.js';

async function main() {
    await init();  // Load WASM
    console.log(greet("World"));
}

main();
```

### With Bundlers (Vite/Webpack)

```javascript
// Bundlers handle init automatically
import { greet } from './pkg/my_wasm_lib';

console.log(greet("World"));
```

## CLI Wrapper Pattern

Wrap WASM library for CLI distribution:

```javascript
#!/usr/bin/env node
import { run_cli } from "./pkg/my_wasm_lib.js";

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

```json
{
  "name": "my-cli",
  "bin": "cli.mjs",
  "type": "module",
  "files": ["cli.mjs", "pkg/"]
}
```

## Publishing to npm

### Direct pkg/ Publishing

```bash
cd pkg
npm publish
```

### Wrapper Package

```json
{
  "name": "@scope/my-lib",
  "version": "1.0.0",
  "main": "./pkg/my_wasm_lib.js",
  "types": "./pkg/my_wasm_lib.d.ts",
  "files": ["pkg/"],
  "scripts": {
    "build": "wasm-pack build --target bundler",
    "prepublishOnly": "npm run build"
  }
}
```

## Optimization

### Binary Size

```toml
[profile.release]
opt-level = "z"      # Optimize for size (smaller than "s")
lto = true
codegen-units = 1
panic = "abort"      # No unwinding, smaller binary
```

### wasm-opt Post-Processing

```bash
# Install binaryen
brew install binaryen

# Optimize
wasm-opt -O3 pkg/my_wasm_lib_bg.wasm -o pkg/my_wasm_lib_bg.wasm
```

### Stripping Debug Info

```bash
wasm-pack build --release
```

## Limitations

| Capability | Status |
|:-----------|:-------|
| Filesystem access | Limited (browser sandbox) |
| Network requests | Via fetch API only |
| Threading | Experimental (SharedArrayBuffer) |
| System calls | Not available |
| Direct memory | Available but sandboxed |

## WASM vs Native Comparison

| Aspect | WASM | Native (napi-rs/Neon) |
|:-------|:-----|:---------------------|
| **Portability** | Single artifact everywhere | Per-platform binaries |
| **Performance** | ~70-90% of native | 100% native speed |
| **I/O Access** | Sandboxed, limited | Full system access |
| **Browser Support** | Yes | No |
| **Threading** | Limited | Full support |
| **Binary Size** | Smaller | Larger |

## Common Use Cases

1. **Parsers** - JSON, YAML, Markdown, custom DSLs
2. **Compression** - zstd, brotli, gzip
3. **Cryptography** - Hashing, encryption
4. **Image Processing** - Resize, format conversion
5. **Computation** - Scientific, financial calculations
6. **Validation** - Schema validation, linting logic
