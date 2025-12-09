# xxHash for Rust

Multiple Rust crates implement xxHash. This guide covers the major options and provides recommendations.

## Crate Comparison

| Feature | xxhash-rust | twox-hash | xxhash-c |
|---------|-------------|-----------|----------|
| **Algorithms** | XXH32, XXH64, XXH3 64/128 | XXH32, XXH64, XXH3 | XXH32, XXH64, XXH3 |
| **Implementation** | Pure Rust | Pure Rust | C bindings |
| **SIMD Support** | Extensive (AVX2, AVX512, NEON) | Limited | Native from C |
| **Const Fn** | Yes (all variants) | No | No |
| **No-std** | Yes | Yes | No |
| **License** | BSL-1.0 | MIT | BSD-2 |

## Recommended: `xxhash-rust`

Best choice for most Rust applications due to performance, features, and active maintenance.

### Installation

```toml
[dependencies.xxhash-rust]
version = "0.8"
features = ["xxh3", "const_xxh3"]
```

### Feature Flags

| Feature | Description |
|---------|-------------|
| `xxh32` | Enable XXH32 algorithm |
| `xxh64` | Enable XXH64 algorithm |
| `xxh3` | Enable XXH3 algorithm (64 and 128-bit) |
| `const_xxh3` | Enable compile-time XXH3 |
| `const_xxh32` | Enable compile-time XXH32 |
| `const_xxh64` | Enable compile-time XXH64 |

### Basic Usage

```rust
use xxhash_rust::xxh3::xxh3_64;
use xxhash_rust::const_xxh3::const_xxh3_64;

const DATA: &[u8] = b"Hello, xxHash!";

// Runtime hashing
let hash = xxh3_64(DATA);
println!("64-bit hash: {:x}", hash);

// Compile-time hashing (const fn) - zero runtime cost!
const CONST_HASH: u64 = const_xxh3_64(DATA);
println!("Const hash: {:x}", CONST_HASH);
```

### All Variants

```rust
use xxhash_rust::xxh32::xxh32;
use xxhash_rust::xxh64::xxh64;
use xxhash_rust::xxh3::{xxh3_64, xxh3_128};

let data = b"Hello, xxHash!";

let h32 = xxh32(data, 0);           // u32
let h64 = xxh64(data, 0);           // u64
let h3_64 = xxh3_64(data);          // u64
let h3_128 = xxh3_128(data);        // u128

println!("XXH32:    {:08x}", h32);
println!("XXH64:    {:016x}", h64);
println!("XXH3-64:  {:016x}", h3_64);
println!("XXH3-128: {:032x}", h3_128);
```

### With Seeds

```rust
use xxhash_rust::xxh3::xxh3_64_with_seed;
use xxhash_rust::xxh64::xxh64;

let data = b"Hello, xxHash!";
let seed = 42u64;

let hash_seeded = xxh3_64_with_seed(data, seed);
let hash_64 = xxh64(data, seed);
```

### Streaming API

For large data or incremental hashing:

```rust
use xxhash_rust::xxh3::Xxh3;

let mut hasher = Xxh3::new();
hasher.update(b"Hello, ");
hasher.update(b"xxHash!");
hasher.update(b" More data...");
let hash = hasher.digest();
println!("Streaming hash: {:x}", hash);
```

### File Hashing

```rust
use xxhash_rust::xxh3::Xxh3;
use std::fs::File;
use std::io::{BufReader, Read};

fn hash_file(path: &str) -> std::io::Result<u64> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = Xxh3::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(hasher.digest())
}
```

### With HashMap

```rust
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use xxhash_rust::xxh64::Xxh64;

// Type alias for convenience
type Xxh64HashMap<K, V> = HashMap<K, V, BuildHasherDefault<Xxh64>>;

fn main() {
    let mut map: Xxh64HashMap<&str, i32> = HashMap::default();
    map.insert("key", 42);

    // Or explicitly
    let mut map2 = HashMap::<_, _, BuildHasherDefault<Xxh64>>::default();
    map2.insert(123, "value");
}
```

## Alternative: `twox-hash`

Good for simplicity and ease of use, especially with `HashMap`.

### Installation

```toml
[dependencies]
twox-hash = "2"
```

### Usage

```rust
use std::collections::HashMap;
use twox_hash::xxh3::RandomHashBuilder64;

let mut map = HashMap::with_hasher(RandomHashBuilder64::default());
map.insert("key", 42);
```

## SIMD Support

xxhash-rust automatically enables SIMD based on target architecture:

| Architecture | SIMD |
|--------------|------|
| x86_64 | SSE2 (default), AVX2 (flag), AVX512 (flag) |
| aarch64 | NEON (default) |
| wasm32 | SIMD128 (flag required) |

Enable AVX2 for maximum performance:

```bash
RUSTFLAGS="-C target-cpu=native" cargo build --release
```

Or in `.cargo/config.toml`:

```toml
[build]
rustflags = ["-C", "target-cpu=native"]
```

## No-std Support

For embedded environments:

```toml
[dependencies]
xxhash-rust = { version = "0.8", default-features = false, features = ["xxh3"] }
```

```rust
#![no_std]
use xxhash_rust::xxh3::xxh3_64;

fn hash(data: &[u8]) -> u64 {
    xxh3_64(data)
}
```

## Common Patterns

### Content-Addressable Key

```rust
use xxhash_rust::xxh3::xxh3_64;

fn content_key(data: &[u8]) -> String {
    let hash = xxh3_64(data);
    format!("{:016x}", hash)
}
```

### Cache Key Generation

```rust
use xxhash_rust::xxh3::Xxh3;

fn cache_key(params: &[&str]) -> u64 {
    let mut hasher = Xxh3::new();
    for param in params {
        hasher.update(param.as_bytes());
        hasher.update(b"\0");  // Separator
    }
    hasher.digest()
}
```

### Bloom Filter Hashes

```rust
use xxhash_rust::xxh3::xxh3_64_with_seed;

fn bloom_hashes(data: &[u8], num_hashes: usize) -> Vec<u64> {
    (0..num_hashes as u64)
        .map(|seed| xxh3_64_with_seed(data, seed))
        .collect()
}
```

## Performance Tips

1. **Use `xxh3_64`** for best single-threaded performance
2. **Enable native CPU features** via RUSTFLAGS
3. **Use streaming API** for data >64KB to avoid memory allocation
4. **Use `const_xxh3`** for static strings/keys
5. **Prefer `&[u8]` over String** - avoid UTF-8 validation overhead

## Recommendation Summary

| Scenario | Recommended Crate |
|----------|-------------------|
| Performance-critical | `xxhash-rust` with SIMD |
| General purpose | `xxhash-rust` or `twox-hash` |
| HashMap integration | `twox-hash` (simpler) |
| Compile-time hashing | `xxhash-rust` with `const_xxh3` |
| C compatibility | `xxhash-c` |
| Embedded/no-std | `xxhash-rust` |

## Related

- [Variant Comparison](./variants.md) - Choosing between XXH32, XXH64, XXH3
- [TypeScript Libraries](./typescript.md) - Cross-language considerations
- [Usage Examples](./usage-examples.md) - More code examples
