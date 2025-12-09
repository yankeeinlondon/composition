# Neon Guide

Neon is a mature framework for writing native Node.js modules in Rust. It provides safe abstractions for V8/Node-API interop with comprehensive documentation and community support.

## Key Features

| Feature | Description |
|:--------|:------------|
| **Architecture** | Rust bindings to Node-API with Handle-based memory management |
| **Build Output** | Platform-specific `.node` binary files |
| **Memory Safety** | Rust ownership system prevents common memory errors |
| **Maturity** | Established since 2017, production-proven |
| **Async Support** | Full async/await integration with Node.js event loop |

## Project Setup

### Create New Project

```bash
npm init neon my-project
```

This creates a hybrid directory that's both a Rust crate and npm package.

### Generated Structure

```
my-project/
├── src/lib.rs          # Rust source code
├── Cargo.toml          # Rust configuration
├── package.json        # npm configuration
└── index.node          # Built native module (after build)
```

### Cargo.toml Configuration

```toml
[package]
name = "my-project"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
neon = { version = "1.0.0", features = ["napi-6"] }
```

### package.json Scripts

```json
{
  "scripts": {
    "build": "neon build --release",
    "install": "neon build",
    "prepublishOnly": "neon build --release"
  },
  "dependencies": {
    "@neon-rs/cli": "^0.10.0"
  }
}
```

## Core Concepts

### Module Entry Point

Every Neon module needs a `#[neon::main]` entry point:

```rust
use neon::prelude::*;

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("hello", hello)?;
    cx.export_function("add", add)?;
    Ok(())
}
```

### Function Context

Functions receive a `FunctionContext` for interacting with JavaScript:

```rust
fn hello(mut cx: FunctionContext) -> JsResult<JsString> {
    Ok(cx.string("Hello from Rust!"))
}
```

### Argument Handling

```rust
fn add(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let a = cx.argument::<JsNumber>(0)?.value(&mut cx);
    let b = cx.argument::<JsNumber>(1)?.value(&mut cx);
    Ok(cx.number(a + b))
}
```

### Optional Arguments

```rust
fn greet(mut cx: FunctionContext) -> JsResult<JsString> {
    let name = cx.argument_opt(0)
        .and_then(|v| v.downcast::<JsString, _>(&mut cx).ok())
        .map(|s| s.value(&mut cx))
        .unwrap_or_else(|| "World".to_string());

    Ok(cx.string(format!("Hello, {}!", name)))
}
```

## Type Conversion

### JavaScript Types

| Neon Type | JavaScript Equivalent |
|:----------|:---------------------|
| `JsNumber` | `number` |
| `JsString` | `string` |
| `JsBoolean` | `boolean` |
| `JsArray` | `Array` |
| `JsObject` | `Object` |
| `JsBuffer` | `Buffer` |
| `JsFunction` | `Function` |
| `JsNull` | `null` |
| `JsUndefined` | `undefined` |

### Creating Values

```rust
fn create_values(mut cx: FunctionContext) -> JsResult<JsObject> {
    let obj = cx.empty_object();

    let num = cx.number(42);
    let str = cx.string("hello");
    let bool_val = cx.boolean(true);
    let arr = cx.empty_array();

    obj.set(&mut cx, "num", num)?;
    obj.set(&mut cx, "str", str)?;
    obj.set(&mut cx, "active", bool_val)?;
    obj.set(&mut cx, "items", arr)?;

    Ok(obj)
}
```

### Working with Arrays

```rust
fn process_array(mut cx: FunctionContext) -> JsResult<JsArray> {
    let input = cx.argument::<JsArray>(0)?;
    let len = input.len(&mut cx);
    let output = cx.empty_array();

    for i in 0..len {
        let value: Handle<JsNumber> = input.get(&mut cx, i)?;
        let doubled = value.value(&mut cx) * 2.0;
        let result = cx.number(doubled);
        output.set(&mut cx, i, result)?;
    }

    Ok(output)
}
```

## Classes and Objects

### Exporting a Class

```rust
use neon::types::Finalize;

struct Counter {
    value: i32,
}

impl Finalize for Counter {}

impl Counter {
    fn new() -> Self {
        Counter { value: 0 }
    }

    fn increment(&mut self) {
        self.value += 1;
    }

    fn get(&self) -> i32 {
        self.value
    }
}

fn counter_new(mut cx: FunctionContext) -> JsResult<JsBox<Counter>> {
    Ok(cx.boxed(Counter::new()))
}

fn counter_increment(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let counter = cx.argument::<JsBox<Counter>>(0)?;
    // Need RefCell for mutable access through JsBox
    Ok(cx.undefined())
}

fn counter_get(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let counter = cx.argument::<JsBox<Counter>>(0)?;
    Ok(cx.number(counter.get()))
}
```

## Async Operations

### Returning Promises

```rust
fn async_task(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let input = cx.argument::<JsString>(0)?.value(&mut cx);

    let promise = cx.task(move || {
        // Runs on separate thread
        std::thread::sleep(std::time::Duration::from_millis(100));
        input.to_uppercase()
    })
    .promise(|mut cx, result| {
        // Runs on main thread with result
        Ok(cx.string(result))
    });

    Ok(promise)
}
```

### Using Tokio

```rust
use neon::prelude::*;
use tokio::runtime::Runtime;

fn fetch_url(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let url = cx.argument::<JsString>(0)?.value(&mut cx);
    let channel = cx.channel();

    let (deferred, promise) = cx.promise();

    std::thread::spawn(move || {
        let rt = Runtime::new().unwrap();
        let result = rt.block_on(async {
            reqwest::get(&url).await?.text().await
        });

        deferred.settle_with(&channel, move |mut cx| {
            match result {
                Ok(body) => Ok(cx.string(body)),
                Err(e) => cx.throw_error(e.to_string()),
            }
        });
    });

    Ok(promise)
}
```

## Error Handling

### Throwing Errors

```rust
fn risky_operation(mut cx: FunctionContext) -> JsResult<JsString> {
    let input = cx.argument::<JsString>(0)?.value(&mut cx);

    if input.is_empty() {
        return cx.throw_error("Input cannot be empty");
    }

    Ok(cx.string(input.to_uppercase()))
}
```

### Converting Rust Errors

```rust
fn parse_json(mut cx: FunctionContext) -> JsResult<JsValue> {
    let input = cx.argument::<JsString>(0)?.value(&mut cx);

    serde_json::from_str::<serde_json::Value>(&input)
        .or_else(|e| cx.throw_error(e.to_string()))
        .map(|_| cx.undefined().upcast())
}
```

## Safe Parallelism

### Using vm::lock for Thread Safety

Neon provides `vm::lock()` to safely share Node.js `Buffer` data with multiple Rust threads:

```rust
use neon::prelude::*;
use rayon::prelude::*;

fn parallel_process(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let buffer = cx.argument::<JsBuffer>(0)?;

    // Lock prevents JavaScript execution while threads access buffer
    let result = cx.borrow(&buffer, |data| {
        data.as_slice()
            .par_chunks(1024)
            .map(|chunk| chunk.iter().map(|&b| b as u64).sum::<u64>())
            .sum::<u64>()
    });

    Ok(cx.number(result as f64))
}
```

## Building and Publishing

### Development Build

```bash
npm run build
```

### Release Build

```bash
neon build --release
```

### Publishing to npm

```json
{
  "files": [
    "index.js",
    "index.node",
    "package.json"
  ],
  "scripts": {
    "prepublishOnly": "neon build --release"
  }
}
```

### Cross-Platform CI

```yaml
jobs:
  build:
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: npm ci
      - run: npm run build
```

## Neon vs napi-rs

| Aspect | Neon | napi-rs |
|:-------|:-----|:--------|
| **TypeScript** | Manual `.d.ts` files | Auto-generated |
| **Maturity** | Older, more documentation | Newer, faster development |
| **Performance** | Excellent | Slightly faster in benchmarks |
| **API Style** | Context-based (`cx.string()`) | Attribute macros (`#[napi]`) |
| **Class Support** | Via `JsBox` | Via `#[napi]` on impl |

## When to Choose Neon

- Need comprehensive documentation and examples
- Prefer explicit context-based API style
- Want established community support
- Building Electron apps (good Electron integration)
- Complex async patterns with explicit control

## Pros and Cons

| Pros | Cons |
|:-----|:-----|
| Mature, well-documented | No auto TypeScript generation |
| Safe memory management | More verbose than napi-rs |
| Good async support | Requires manual type definitions |
| Active maintenance | Slightly more boilerplate |
