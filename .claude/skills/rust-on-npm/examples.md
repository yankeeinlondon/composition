# Code Examples

Working examples for each Rust-to-npm pattern.

## napi-rs: Image Processing Library

A complete example exposing image processing via napi-rs.

### Cargo.toml

```toml
[package]
name = "image-processor"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
napi = { version = "3", features = ["napi4"] }
napi-derive = "3"
image = "0.25"
```

### src/lib.rs

```rust
use napi::bindgen_prelude::*;
use napi_derive::napi;

#[napi]
pub struct ImageProcessor {
    data: Vec<u8>,
    width: u32,
    height: u32,
}

#[napi]
impl ImageProcessor {
    #[napi(constructor)]
    pub fn new(data: Uint8Array) -> Result<Self> {
        let img = image::load_from_memory(&data)
            .map_err(|e| Error::from_reason(e.to_string()))?;

        Ok(Self {
            data: data.to_vec(),
            width: img.width(),
            height: img.height(),
        })
    }

    #[napi(getter)]
    pub fn width(&self) -> u32 {
        self.width
    }

    #[napi(getter)]
    pub fn height(&self) -> u32 {
        self.height
    }

    #[napi]
    pub fn resize(&self, width: u32, height: u32) -> Result<Uint8Array> {
        let img = image::load_from_memory(&self.data)
            .map_err(|e| Error::from_reason(e.to_string()))?;

        let resized = img.resize(width, height, image::imageops::FilterType::Lanczos3);

        let mut buffer = Vec::new();
        resized
            .write_to(&mut std::io::Cursor::new(&mut buffer), image::ImageFormat::Png)
            .map_err(|e| Error::from_reason(e.to_string()))?;

        Ok(buffer.into())
    }

    #[napi]
    pub fn to_webp(&self, quality: Option<u8>) -> Result<Uint8Array> {
        let img = image::load_from_memory(&self.data)
            .map_err(|e| Error::from_reason(e.to_string()))?;

        let encoder = webp::Encoder::from_image(&img)
            .map_err(|e| Error::from_reason(e.to_string()))?;

        let quality = quality.unwrap_or(80) as f32;
        let webp_data = encoder.encode(quality);

        Ok(webp_data.to_vec().into())
    }
}

#[napi]
pub fn get_image_dimensions(data: Uint8Array) -> Result<ImageDimensions> {
    let img = image::load_from_memory(&data)
        .map_err(|e| Error::from_reason(e.to_string()))?;

    Ok(ImageDimensions {
        width: img.width(),
        height: img.height(),
    })
}

#[napi(object)]
pub struct ImageDimensions {
    pub width: u32,
    pub height: u32,
}
```

### Usage (TypeScript)

```typescript
import { ImageProcessor, getImageDimensions } from '@scope/image-processor';
import { readFileSync, writeFileSync } from 'fs';

const imageData = readFileSync('input.jpg');
const processor = new ImageProcessor(imageData);

console.log(`Image size: ${processor.width}x${processor.height}`);

// Resize
const resized = processor.resize(800, 600);
writeFileSync('resized.png', resized);

// Convert to WebP
const webp = processor.toWebp(85);
writeFileSync('output.webp', webp);

// Quick dimensions check
const dims = getImageDimensions(imageData);
console.log(dims); // { width: 1920, height: 1080 }
```

---

## Neon: CPU-Bound Computation

Parallel word counting with Rayon.

### Cargo.toml

```toml
[package]
name = "word-counter"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
neon = { version = "1.0.0", features = ["napi-6"] }
rayon = "1.10"
```

### src/lib.rs

```rust
use neon::prelude::*;
use rayon::prelude::*;
use std::collections::HashMap;

fn count_words(mut cx: FunctionContext) -> JsResult<JsObject> {
    let text = cx.argument::<JsString>(0)?.value(&mut cx);

    // Parallel word counting with Rayon
    let counts: HashMap<String, usize> = text
        .par_lines()
        .flat_map(|line| line.split_whitespace())
        .map(|word| word.to_lowercase())
        .fold(
            || HashMap::new(),
            |mut acc, word| {
                *acc.entry(word).or_insert(0) += 1;
                acc
            },
        )
        .reduce(
            || HashMap::new(),
            |mut a, b| {
                for (word, count) in b {
                    *a.entry(word).or_insert(0) += count;
                }
                a
            },
        );

    // Convert to JS object
    let result = cx.empty_object();
    for (word, count) in counts {
        let key = cx.string(&word);
        let value = cx.number(count as f64);
        result.set(&mut cx, key, value)?;
    }

    Ok(result)
}

fn count_total(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let text = cx.argument::<JsString>(0)?.value(&mut cx);

    let count: usize = text
        .par_lines()
        .map(|line| line.split_whitespace().count())
        .sum();

    Ok(cx.number(count as f64))
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("countWords", count_words)?;
    cx.export_function("countTotal", count_total)?;
    Ok(())
}
```

### Usage (JavaScript)

```javascript
const { countWords, countTotal } = require('./');
const fs = require('fs');

const text = fs.readFileSync('large-text.txt', 'utf-8');

console.time('countTotal');
const total = countTotal(text);
console.timeEnd('countTotal');
console.log(`Total words: ${total}`);

console.time('countWords');
const frequencies = countWords(text);
console.timeEnd('countWords');

// Get top 10 words
const sorted = Object.entries(frequencies)
  .sort((a, b) => b[1] - a[1])
  .slice(0, 10);

console.log('Top 10 words:', sorted);
```

---

## WebAssembly: JSON Schema Validator

Isomorphic validation for browser and Node.js.

### Cargo.toml

```toml
[package]
name = "schema-validator"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
wasm-bindgen = "0.2"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
jsonschema = "0.18"

[profile.release]
opt-level = "s"
lto = true
```

### src/lib.rs

```rust
use wasm_bindgen::prelude::*;
use jsonschema::JSONSchema;
use serde_json::Value;

#[wasm_bindgen]
pub struct Validator {
    schema: JSONSchema,
}

#[wasm_bindgen]
impl Validator {
    #[wasm_bindgen(constructor)]
    pub fn new(schema_json: &str) -> Result<Validator, JsValue> {
        let schema: Value = serde_json::from_str(schema_json)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let compiled = JSONSchema::compile(&schema)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        Ok(Validator { schema: compiled })
    }

    #[wasm_bindgen]
    pub fn validate(&self, data_json: &str) -> Result<ValidationResult, JsValue> {
        let data: Value = serde_json::from_str(data_json)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let result = self.schema.validate(&data);

        match result {
            Ok(_) => Ok(ValidationResult {
                valid: true,
                errors: Vec::new(),
            }),
            Err(errors) => {
                let error_messages: Vec<String> = errors
                    .map(|e| format!("{}: {}", e.instance_path, e))
                    .collect();

                Ok(ValidationResult {
                    valid: false,
                    errors: error_messages,
                })
            }
        }
    }

    #[wasm_bindgen]
    pub fn is_valid(&self, data_json: &str) -> bool {
        serde_json::from_str::<Value>(data_json)
            .map(|data| self.schema.is_valid(&data))
            .unwrap_or(false)
    }
}

#[wasm_bindgen]
pub struct ValidationResult {
    valid: bool,
    errors: Vec<String>,
}

#[wasm_bindgen]
impl ValidationResult {
    #[wasm_bindgen(getter)]
    pub fn valid(&self) -> bool {
        self.valid
    }

    #[wasm_bindgen(getter)]
    pub fn errors(&self) -> Vec<String> {
        self.errors.clone()
    }
}

// Quick validation without creating Validator instance
#[wasm_bindgen]
pub fn validate_once(schema_json: &str, data_json: &str) -> Result<bool, JsValue> {
    let schema: Value = serde_json::from_str(schema_json)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let data: Value = serde_json::from_str(data_json)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let compiled = JSONSchema::compile(&schema)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    Ok(compiled.is_valid(&data))
}
```

### Build

```bash
wasm-pack build --target bundler
```

### Usage (Browser/Node)

```typescript
import { Validator, validate_once } from './pkg/schema_validator';

const schema = JSON.stringify({
  type: 'object',
  properties: {
    name: { type: 'string', minLength: 1 },
    age: { type: 'integer', minimum: 0 },
    email: { type: 'string', format: 'email' },
  },
  required: ['name', 'email'],
});

// Reusable validator
const validator = new Validator(schema);

const data1 = JSON.stringify({ name: 'Alice', email: 'alice@example.com', age: 30 });
const data2 = JSON.stringify({ name: '', email: 'invalid' });

console.log(validator.isValid(data1)); // true
console.log(validator.isValid(data2)); // false

const result = validator.validate(data2);
console.log(result.valid);   // false
console.log(result.errors);  // ["": "..." validation errors]

// One-off validation
const isValid = validate_once(schema, data1);
console.log(isValid); // true
```

---

## Native Binary: CLI Formatter

A code formatter CLI distributed via npm.

### Project Structure

```
formatter/
├── Cargo.toml
├── src/main.rs
└── npm/
    ├── formatter/
    │   ├── package.json
    │   └── lib/index.js
    └── formatter-darwin-arm64/
        ├── package.json
        └── bin/formatter
```

### Cargo.toml

```toml
[package]
name = "formatter"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "formatter"
path = "src/main.rs"

[dependencies]
clap = { version = "4", features = ["derive"] }
```

### src/main.rs

```rust
use clap::Parser;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "formatter")]
#[command(about = "A fast code formatter")]
struct Cli {
    /// Files to format
    #[arg(required = true)]
    files: Vec<PathBuf>,

    /// Check mode (don't write changes)
    #[arg(long)]
    check: bool,

    /// Write changes to files
    #[arg(long, short)]
    write: bool,
}

fn main() {
    let cli = Cli::parse();

    let mut has_changes = false;

    for file in &cli.files {
        let content = match fs::read_to_string(file) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error reading {}: {}", file.display(), e);
                continue;
            }
        };

        let formatted = format_code(&content);

        if formatted != content {
            has_changes = true;

            if cli.check {
                println!("{} needs formatting", file.display());
            } else if cli.write {
                if let Err(e) = fs::write(file, &formatted) {
                    eprintln!("Error writing {}: {}", file.display(), e);
                } else {
                    println!("Formatted {}", file.display());
                }
            } else {
                print!("{}", formatted);
            }
        }
    }

    if cli.check && has_changes {
        std::process::exit(1);
    }
}

fn format_code(content: &str) -> String {
    // Actual formatting logic here
    content.to_string()
}
```

### npm/formatter/package.json

```json
{
  "name": "formatter",
  "version": "0.1.0",
  "description": "A fast code formatter",
  "bin": "lib/index.js",
  "type": "module",
  "optionalDependencies": {
    "formatter-darwin-arm64": "0.1.0",
    "formatter-darwin-x64": "0.1.0",
    "formatter-linux-x64": "0.1.0",
    "formatter-linux-arm64": "0.1.0",
    "formatter-win32-x64": "0.1.0"
  },
  "keywords": ["formatter", "cli"],
  "license": "MIT"
}
```

### npm/formatter/lib/index.js

```javascript
#!/usr/bin/env node
import { spawnSync } from "node:child_process";
import { createRequire } from "node:module";
import os from "node:os";

const require = createRequire(import.meta.url);

const PLATFORMS = {
  darwin: { x64: "darwin-x64", arm64: "darwin-arm64" },
  linux: { x64: "linux-x64", arm64: "linux-arm64" },
  win32: { x64: "win32-x64" },
};

function getBinaryPath() {
  const platform = os.platform();
  const arch = os.arch();
  const platformArch = PLATFORMS[platform]?.[arch];

  if (!platformArch) {
    throw new Error(`Unsupported platform: ${platform}-${arch}`);
  }

  const pkgName = `formatter-${platformArch}`;
  const exe = platform === "win32" ? ".exe" : "";

  try {
    return require.resolve(`${pkgName}/bin/formatter${exe}`);
  } catch {
    throw new Error(`Could not find binary package: ${pkgName}`);
  }
}

const result = spawnSync(getBinaryPath(), process.argv.slice(2), {
  stdio: "inherit",
});

process.exit(result.status ?? 1);
```

### npm/formatter-darwin-arm64/package.json

```json
{
  "name": "formatter-darwin-arm64",
  "version": "0.1.0",
  "os": ["darwin"],
  "cpu": ["arm64"],
  "files": ["bin/"]
}
```

### Usage

```bash
# Install globally
npm install -g formatter

# Format files
formatter src/**/*.ts --write

# Check mode (CI)
formatter src/**/*.ts --check
```

---

## Project Selection Summary

| Project Type | Recommended Pattern | Example |
|:-------------|:-------------------|:--------|
| Image/media processing | napi-rs | `@napi-rs/image` |
| Parallel computation | Neon | Word counter, data processing |
| Validation/parsing | WebAssembly | Schema validator, markdown parser |
| CLI tool | Native binary | Formatter, linter, bundler |
| Database driver | napi-rs | `@prisma/client` |
| Cryptography | WebAssembly or napi-rs | Hashing, encryption |
