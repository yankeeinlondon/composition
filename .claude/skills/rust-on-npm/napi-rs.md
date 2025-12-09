# napi-rs Guide

napi-rs is a high-level framework for building pre-compiled Node.js addons in Rust. It provides automatic TypeScript generation, cross-platform builds, and seamless npm distribution.

## Key Features

| Feature | Description |
|:--------|:------------|
| **Core Technology** | Node-API (N-API) - stable ABI across Node.js versions |
| **Build Output** | Platform-specific `.node` binary files |
| **Type Support** | Auto-generated TypeScript definitions via `#[napi]` macro |
| **Distribution** | Multiple npm packages per platform (optional dependencies) |
| **Platform Coverage** | Windows, macOS, Linux (glibc/musl), FreeBSD, Android |

## Project Setup

### Install CLI and Scaffold

```bash
npm install -g @napi-rs/cli
napi new
```

The wizard prompts for:
- Package name (use npm scope like `@scope/package`)
- Target platforms
- GitHub Actions generation

### Generated Structure

```
my-project/
├── src/lib.rs          # Rust code with #[napi] attributes
├── build.rs            # Node-API build configuration
├── Cargo.toml          # crate-type = ["cdylib"]
├── index.js            # Auto-generated loader
├── index.d.ts          # Auto-generated TypeScript definitions
└── package.json
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
napi = { version = "3", features = ["napi4"] }
napi-derive = "3"
```

## Writing Rust Code

### Exporting Functions

```rust
use napi_derive::napi;

#[napi]
pub fn fibonacci(n: u32) -> u32 {
    match n {
        1 | 2 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}
```

Generates TypeScript:
```typescript
export function fibonacci(n: number): number;
```

### Exposing Classes

```rust
use napi::bindgen_prelude::*;
use napi_derive::napi;

#[napi]
pub struct ImageProcessor {
    data: Vec<u8>,
}

#[napi]
impl ImageProcessor {
    #[napi(constructor)]
    pub fn new(data: Uint8Array) -> Self {
        Self { data: data.to_vec() }
    }

    #[napi]
    pub fn to_webp(&self) -> Result<Uint8Array> {
        // Process image...
        Ok(Uint8Array::new(processed_data))
    }
}
```

### Handling Callbacks

```rust
use napi::bindgen_prelude::*;

#[napi]
pub fn process_with_callback(
    input: String,
    callback: Function<String, String>
) -> Result<String> {
    let processed = input.to_uppercase();
    callback.call(processed)
}
```

For async callbacks, create a `FunctionRef` to keep the function alive:

```rust
#[napi(ts_return_type = "Promise<void>")]
pub fn delayed_callback<'env>(
    env: &'env Env,
    delay_ms: u32,
    callback: Function<'env, String, ()>
) -> Result<PromiseRaw<'env, ()>> {
    let callback_ref = callback.create_ref()?;
    env.spawn_future_with_callback(
        async move {
            tokio::time::sleep(Duration::from_millis(delay_ms as u64)).await;
            Ok("Done!".to_string())
        },
        move |env, message| {
            let callback = callback_ref.borrow_back(env)?;
            callback.call(message)?;
            Ok(())
        }
    )
}
```

### Error Handling

```rust
use napi::{Error, Status};

#[napi]
pub fn parse_json(input: String) -> Result<JsObject> {
    serde_json::from_str(&input)
        .map_err(|e| Error::new(Status::InvalidArg, e.to_string()))
}
```

## Building

### Development Build

```bash
napi build --platform
```

### Release Build

```bash
napi build --release
```

### Build for All Platforms (CI)

The generated GitHub Actions workflow handles cross-compilation and publishing.

## Publishing

### Setup

1. Create npm scope at npmjs.com
2. Run `napi rename` to set package name
3. Add `NPM_TOKEN` secret to GitHub repository

### Release Flow

```bash
npm version patch  # or minor/major
git push --follow-tags
```

GitHub Actions:
1. Builds binaries for all configured platforms
2. Publishes platform-specific packages
3. Publishes main package with optionalDependencies

## Type Mappings

| Rust | TypeScript |
|:-----|:-----------|
| `i32`, `u32`, `f64` | `number` |
| `i64`, `u64` | `bigint` |
| `String` | `string` |
| `bool` | `boolean` |
| `Vec<T>` | `Array<T>` |
| `Option<T>` | `T \| null` |
| `Result<T>` | `T` (throws on error) |
| `Uint8Array` | `Uint8Array` |
| `Buffer` | `Buffer` |

## Handling External Types (Orphan Rule)

You cannot implement `#[napi]` for types from other crates. Wrap them:

```rust
// Cannot do: #[napi] on external_crate::SomeType
// Solution: Create wrapper

pub struct MyWrapper(external_crate::SomeType);

#[napi]
impl MyWrapper {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self(external_crate::SomeType::new())
    }
}
```

## Dual-Building (Library + Addon)

To use the same Rust code as both a library crate and a Node addon:

```toml
# Cargo.toml
[features]
default = []
napi = ["dep:napi", "dep:napi-derive"]

[dependencies]
napi = { version = "3", optional = true }
napi-derive = { version = "3", optional = true }
```

```rust
// Only compile napi code when feature enabled
#[cfg(feature = "napi")]
mod napi_bindings {
    use napi_derive::napi;
    // ...
}
```

## Performance Tips

- **Minimize boundary crossings**: Batch operations instead of many small calls
- **Use references**: `&str`, `&[u8]` instead of owned types when possible
- **Release builds**: Always use `--release` for production
- **Buffer types**: Use `Uint8Array`/`Buffer` for binary data, not arrays

## Pros and Cons

| Pros | Cons |
|:-----|:-----|
| Excellent TypeScript integration | Learning curve (Rust + Node FFI) |
| Cross-platform CI/CD tooling | Type conversion overhead for complex types |
| ABI stability via N-API | Binary size larger than pure JS |
| First-class async support | Dual-build complexity for lib+addon |
