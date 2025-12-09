---
name: xxHash
description: Comprehensive guide to the xxHash non-cryptographic hash algorithm family, implementations, and usage across programming languages
created: 2025-12-08
hash: 5b81fe7e0a2c0b79
tags:
  - hashing
  - xxhash
  - performance
  - algorithms
  - rust
  - typescript
  - python
  - go
  - java
---

# xxHash: A Comprehensive Deep Dive

xxHash is an extremely fast non-cryptographic hash algorithm designed to operate at RAM speed limits. Originally developed by Yann Collet, it has become the go-to choice for applications requiring high-performance hashing such as hash tables, checksums, bloom filters, and data integrity verification. This guide covers the algorithm variants, implementations across programming languages, and practical recommendations for choosing the right library.

## Table of Contents

- [What is xxHash](#what-is-xxhash)
- [Algorithm Variants](#algorithm-variants)
  - [XXH32](#xxh32)
  - [XXH64](#xxh64)
  - [XXH3](#xxh3)
- [Performance Comparison](#performance-comparison)
- [TypeScript and JavaScript Implementations](#typescript-and-javascript-implementations)
- [Rust Implementations](#rust-implementations)
- [Other Language Implementations](#other-language-implementations)
- [Usage Examples](#usage-examples)
- [Best Practices](#best-practices)
- [Quick Reference](#quick-reference)
- [Resources](#resources)

## What is xxHash

xxHash is a family of non-cryptographic hash functions renowned for exceptional performance while maintaining excellent collision resistance. The algorithm comes in several variants: the older XXH32/XXH64 and the newer XXH3 family, which offers superior performance on modern hardware through SIMD optimizations.

**Key Characteristics:**

- Operates at near-RAM speed limits (up to 31.5 GB/s for XXH3)
- Excellent collision resistance (passes all SMHasher tests)
- Stable output across languages and platforms
- Not suitable for cryptographic purposes

**Common Use Cases:**

- Hash tables and hash maps
- Data integrity verification and checksums
- Bloom filters
- Content deduplication
- Cache key generation
- File comparison and synchronization

## Algorithm Variants

### XXH32

- **Bit Width:** 32-bit
- **Throughput:** ~9.7 GB/s (Intel i7-9700K)
- **Small Data Velocity:** 71.9

The original 32-bit version of xxHash, designed for 32-bit systems and applications where smaller hash values are sufficient. It offers excellent performance and hash quality but is outperformed by newer variants on 64-bit systems.

**Best For:**
- Embedded systems with 32-bit architectures
- Memory-constrained environments
- Legacy systems requiring 32-bit hashes
- Lightweight checksums

### XXH64

- **Bit Width:** 64-bit
- **Throughput:** ~19.4 GB/s (Intel i7-9700K)
- **Small Data Velocity:** 71.0

A 64-bit version that significantly improves performance on modern 64-bit architectures. It provides a larger hash space, reducing collision probability compared to XXH32.

**Best For:**
- General-purpose hashing on 64-bit systems
- Databases and file systems
- Network protocols
- Applications requiring higher collision resistance

### XXH3

- **Bit Width:** 64-bit (XXH3_64bits) or 128-bit (XXH3_128bits)
- **Throughput:** ~31.5 GB/s (64-bit) / ~29.6 GB/s (128-bit)
- **Small Data Velocity:** 133.1 (64-bit) / 118.1 (128-bit)

The latest and most advanced variant introduces revolutionary improvements in both speed and hash quality. Optimized for modern CPUs with SIMD instructions (SSE2, AVX2, AVX512, NEON).

**Best For:**
- High-performance hash tables
- Real-time data processing
- Caching systems
- Large-scale data deduplication
- Content-addressable storage

**Recommendation:** For most modern applications, XXH3_64bits provides the best combination of speed and hash quality, while XXH3_128bits is ideal for scenarios requiring maximal collision resistance.

## Performance Comparison

Based on benchmarks from an Intel i7-9700K system (Ubuntu x64 20.04):

| Hash Name | Bit Width | Bandwidth (GB/s) | Small Data Velocity | Quality |
|-----------|-----------|------------------|---------------------|---------|
| XXH3 (SSE2) | 64 | 31.5 | 133.1 | 10 |
| XXH128 (SSE2) | 128 | 29.6 | 118.1 | 10 |
| XXH64 | 64 | 19.4 | 71.0 | 10 |
| XXH32 | 32 | 9.7 | 71.9 | 10 |
| City64 | 64 | 22.0 | 76.6 | 10 |
| Murmur3 | 32 | 3.9 | 56.1 | 10 |
| SipHash | 64 | 3.0 | 43.2 | 10 |

> **Note:** Some algorithms achieve faster-than-RAM speeds when data is in CPU cache (L3 or better). Otherwise, they are limited by RAM speed.

## TypeScript and JavaScript Implementations

### Library Comparison Matrix

| Library | Implementation | Algorithms | Runtimes | TS Support | Streaming |
|---------|---------------|------------|----------|------------|-----------|
| `@node-rs/xxhash` | Native (Rust) | XXH32/64/XXH3 | Node only | Bundled .d.ts | Yes |
| `hash-wasm` | WASM | XXH32/64/3/128 + many | Node, browser, Deno, Workers | Bundled | Yes |
| `xxhash-wasm` | WASM | XXH32/64 | Node, browser, CF Workers | Types included | Yes |
| `xxhash-addon` | Native (C) | XXH32/64/XXH3 | Node only | Types included | Yes |
| `js-xxhash` | Pure JS/TS | XXH32 only | Any JS env | Native TS | No |
| `xxhashjs` | Pure JS | XXH32/64 | Node, browser | @types | Yes |
| `xxh3-ts` | Pure TS | XXH64, XXH3-128 | Node, browsers | Native TS | One-shot |
| `@jabr/xxhash64` | TS + WASM | XXH64, XXH3 | Node, Deno, Bun, browser, Workers | Native TS | Varies |

### Recommended: hash-wasm (Cross-Platform Default)

For new TypeScript codebases that need xxHash, `hash-wasm` offers the best balance of performance, compatibility, and features.

**Why hash-wasm:**
- Cross-runtime: Works in Node, browsers, Deno, and Workers without native addons
- Full xxHash coverage: XXH32, XXH64, XXH3, and XXH128
- Great performance: Benchmarks show xxHash64 throughput slightly ahead of `xxhash-wasm`
- First-class TypeScript: Bundled `.d.ts`, modern ESM, Promise-based API
- Bonus algorithms: Also includes SHA, BLAKE3, Argon2, etc.

```typescript
import { xxhash64, createXXHash64 } from 'hash-wasm';

// One-shot (returns hex string)
const digest = await xxhash64('hello', 0);

// Streaming
const hasher = await createXXHash64();
hasher.init(0);
hasher.update('chunk1');
hasher.update('chunk2');
const hex = hasher.digest();
```

### Alternative: @node-rs/xxhash (Node-Only Maximum Performance)

For Node-only services where maximum throughput is critical, `@node-rs/xxhash` provides the fastest implementation via native Rust bindings.

```typescript
import { xxh32, xxh64, Xxh32, Xxh64, xxh3 } from '@node-rs/xxhash';

// One-shot
const h32: number = xxh32('hello', 0);
const h64: bigint = xxh64('hello', 0n);

// Streaming
const hasher = new Xxh64(0n);
hasher.update('chunk1').update('chunk2');
const digest: bigint = hasher.digest();

// XXH3 64/128
const h3_64: bigint = xxh3.xxh64('hello', 0n);
const h128: bigint = xxh3.xxh128('hello', 0n);
```

**Trade-offs:**
- Node-only (no browser, Deno, or Workers)
- Requires native addon compilation
- Platform-specific binaries complicate deployment

### Alternative: xxhash-wasm (WASM xxHash-Only)

When you only need xxHash and want the smallest dependency footprint with cross-platform support:

```typescript
import xxhash from 'xxhash-wasm';

const { h32, h64, create32, create64 } = await xxhash();

// One-shot
const hash32 = h32('input string'); // number
const hash64 = h64('input string'); // bigint

// Streaming
const streamer = create64();
streamer.update('chunk1').update('chunk2');
const digest = streamer.digest();
```

### Alternative: js-xxhash (Pure JS, No Dependencies)

When you need zero dependencies, no WASM, and XXH32 is sufficient:

```typescript
import { xxHash32 } from 'js-xxhash';

const hash = xxHash32('My text to hash', 0);
console.log(hash.toString(16));
```

### Scenario-Based Recommendations

| Scenario | Recommended Library |
|----------|---------------------|
| Node-only, max performance | `@node-rs/xxhash` |
| Cross-platform (Node + browser + Workers) | `hash-wasm` |
| xxHash-only, smallest footprint | `xxhash-wasm` |
| No WASM/native allowed, XXH32 enough | `js-xxhash` |
| No WASM/native allowed, need XXH64/XXH3 | `xxh3-ts` |
| Deno/Bun/JSR-centric stack | `@jabr/xxhash64` |

## Rust Implementations

### Crate Comparison Matrix

| Feature | twox-hash | xxhash-rust | xxhash-c | xxh3 |
|---------|-----------|-------------|----------|------|
| Algorithms | XXH32/64/XXH3 | XXH32/64/XXH3 | XXH32/64/XXH3 | XXH3 only |
| Implementation | Pure Rust | Pure Rust | C bindings | Pure Rust |
| SIMD Support | Limited | Extensive (AVX2/512, NEON) | Native C | Runtime detection |
| Const fn Support | No | Yes (all variants) | No | No |
| No-std Support | Yes | Yes | No | Yes |
| License | MIT | BSL-1.0 | BSD-2 | MIT/Apache-2.0 |

### Recommended: xxhash-rust

For most Rust applications, `xxhash-rust` provides the optimal combination of performance, feature flexibility, and active maintenance.

**Why xxhash-rust:**
- Superior performance via extensive SIMD optimizations (SSE2, AVX2, AVX512, NEON)
- Const fn support enables compile-time hash computation
- Feature-gated algorithms for precise control over binary size
- No-std support for embedded environments
- Actively maintained with regular updates

```toml
# Cargo.toml
[dependencies.xxhash-rust]
version = "0.8"
features = ["xxh3", "const_xxh3"]
```

```rust
use xxhash_rust::xxh3::xxh3_64;
use xxhash_rust::const_xxh3::xxh3_64 as const_xxh3_64;

// Runtime hashing
let hash = xxh3_64(b"Hello, xxHash!");

// Compile-time hashing (const fn)
const STATIC_HASH: u64 = const_xxh3_64(b"static key");

// Streaming
use xxhash_rust::xxh3::Xxh3;
let mut hasher = Xxh3::new();
hasher.update(b"chunk1");
hasher.update(b"chunk2");
let hash = hasher.digest();
```

### Alternative: twox-hash (Simplicity and HashMap Integration)

For simpler use cases or when seamless `HashMap` integration is prioritized:

```rust
use std::collections::HashMap;
use twox_hash::xxh64::RandomState;

let mut map = HashMap::with_hasher(RandomState::default());
map.insert(42, "the answer");
```

### Alternative: xxhash-c (C Reference Compatibility)

When exact byte-for-byte compatibility with the C reference implementation is required:

```rust
use xxhash_c::xxh3::xxh3_64;

let hash = xxh3_64(b"data");
```

**Trade-off:** Requires C toolchain for compilation.

### Scenario-Based Recommendations

| Scenario | Recommended Crate |
|----------|-------------------|
| General-purpose, max performance | `xxhash-rust` |
| Simple HashMap replacement | `twox-hash` |
| Embedded/no-std | `xxhash-rust` (default-features = false) |
| C implementation compatibility | `xxhash-c` |
| Compile-time hashing | `xxhash-rust` with `const_xxh3` |

## Other Language Implementations

### Python

```bash
pip install xxhash
```

```python
import xxhash

# One-shot hashing
data = b"Hello, xxHash!"
hash_64 = xxhash.xxh64(data).hexdigest()
hash_32 = xxhash.xxh32(data).hexdigest()

# With seed
hash_seeded = xxhash.xxh64(data, seed=42).hexdigest()

# Streaming
hasher = xxhash.xxh64()
hasher.update(b"chunk1")
hasher.update(b"chunk2")
digest = hasher.hexdigest()

# File hashing
def hash_file(filepath):
    hasher = xxhash.xxh3_64()
    with open(filepath, 'rb') as f:
        while chunk := f.read(8192):
            hasher.update(chunk)
    return hasher.hexdigest()
```

### Go

```bash
go get github.com/cespare/xxhash/v2
```

```go
package main

import (
    "fmt"
    "github.com/cespare/xxhash/v2"
)

func main() {
    // Simple string hashing
    hash := xxhash.Sum64String("Hello, xxHash!")
    fmt.Printf("Hash: %x\n", hash)

    // Streaming
    hasher := xxhash.New()
    hasher.Write([]byte("chunk1"))
    hasher.Write([]byte("chunk2"))
    hash = hasher.Sum64()

    // With seed
    seededHasher := xxhash.NewWithSeed(42)
    seededHasher.Write([]byte("data"))
    hash = seededHasher.Sum64()
}
```

### Java

```xml
<dependency>
    <groupId>org.lz4</groupId>
    <artifactId>lz4-java</artifactId>
    <version>1.8.0</version>
</dependency>
```

```java
import net.jpountz.xxhash.XXHashFactory;
import net.jpountz.xxhash.XXHash64;
import net.jpountz.xxhash.StreamingXXHash64;

// One-shot
XXHashFactory factory = XXHashFactory.fastestInstance();
XXHash64 hasher = factory.hash64();
byte[] data = "Hello, xxHash!".getBytes();
long hash = hasher.hash(data, 0, data.length, 0L);

// Streaming
StreamingXXHash64 streaming = factory.newStreamingHash64(0L);
streaming.update("chunk1".getBytes());
streaming.update("chunk2".getBytes());
long streamHash = streaming.getValue();
```

## Usage Examples

### TypeScript: File Hashing

```typescript
import { createXXHash3_64 } from 'hash-wasm';
import { createReadStream } from 'fs';

async function hashFile(filepath: string): Promise<string> {
  const hasher = await createXXHash3_64();

  return new Promise((resolve, reject) => {
    const stream = createReadStream(filepath);
    stream.on('data', (chunk) => hasher.update(chunk));
    stream.on('end', () => resolve(hasher.digest()));
    stream.on('error', reject);
  });
}
```

### Rust: Using with HashMap

```rust
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use xxhash_rust::xxh64::Xxh64;

type FastHashMap<K, V> = HashMap<K, V, BuildHasherDefault<Xxh64>>;

fn main() {
    let mut map: FastHashMap<&str, i32> = FastHashMap::default();
    map.insert("key", 42);
}
```

### Python: Parallel File Hashing

```python
import xxhash
from concurrent.futures import ProcessPoolExecutor
from pathlib import Path

def hash_file(filepath: Path) -> tuple[str, str]:
    hasher = xxhash.xxh3_64()
    with open(filepath, 'rb') as f:
        while chunk := f.read(65536):
            hasher.update(chunk)
    return str(filepath), hasher.hexdigest()

def hash_directory(directory: Path) -> dict[str, str]:
    files = list(directory.rglob('*'))
    files = [f for f in files if f.is_file()]

    with ProcessPoolExecutor() as executor:
        results = executor.map(hash_file, files)

    return dict(results)
```

## Best Practices

### Choose the Right Algorithm Variant

- Use **XXH3** for best performance on modern CPUs
- Use **XXH64** for general-purpose 64-bit hashing
- Use **XXH32** only when 32-bit hashes are specifically required or on 32-bit systems

### Use Seeds for Better Distribution

Seeds can help with hash partitioning and reduce collision probability in specific use cases:

```typescript
// TypeScript
const hash1 = await xxhash64(data, 0);  // seed 0
const hash2 = await xxhash64(data, 42); // seed 42 - different hash
```

### Stream Large Data

For files or large buffers, always use streaming APIs to avoid memory pressure:

```rust
// Rust
let mut hasher = Xxh3::new();
for chunk in data.chunks(8192) {
    hasher.update(chunk);
}
let result = hasher.digest();
```

### Security Considerations

- xxHash is NOT cryptographic - never use for password hashing or security-sensitive applications
- Use for checksums, hash tables, and data integrity checks only
- For security-sensitive applications, use SHA-256, BLAKE3, or Argon2

### Cross-Language Consistency

When hashing data across different languages:
- Use the same seed value
- Verify data encoding (especially UTF-8 for strings)
- Use the same algorithm variant (XXH64 vs XXH3)
- Be aware of endianness in hash output format

## Quick Reference

### Algorithm Selection

| Need | Algorithm |
|------|-----------|
| Maximum speed on modern hardware | XXH3_64bits |
| Maximum collision resistance | XXH3_128bits |
| 64-bit system, general purpose | XXH64 |
| 32-bit system or legacy | XXH32 |

### TypeScript Library Selection

| Need | Library |
|------|---------|
| Cross-platform default | `hash-wasm` |
| Node-only max speed | `@node-rs/xxhash` |
| xxHash-only, small bundle | `xxhash-wasm` |
| No WASM, XXH32 only | `js-xxhash` |

### Rust Crate Selection

| Need | Crate |
|------|-------|
| Performance-critical | `xxhash-rust` |
| Simple HashMap swap | `twox-hash` |
| C compatibility | `xxhash-c` |
| Compile-time hashing | `xxhash-rust` + `const_xxh3` |

## Resources

### Official

- [xxHash Official Repository](https://github.com/Cyan4973/xxHash)
- [xxHash Performance Wiki](https://github.com/Cyan4973/xxHash/wiki/Performance-comparison)
- [xxHash Official Site](https://xxhash.com)

### TypeScript/JavaScript

- [hash-wasm](https://www.npmjs.com/package/hash-wasm)
- [@node-rs/xxhash](https://www.npmjs.com/package/@node-rs/xxhash)
- [xxhash-wasm](https://github.com/jungomi/xxhash-wasm)
- [js-xxhash](https://github.com/Jason3S/xxhash)

### Rust

- [xxhash-rust](https://docs.rs/xxhash-rust)
- [twox-hash](https://docs.rs/twox-hash)
- [xxhash-c](https://docs.rs/xxhash-c)

### Other Languages

- [Python: python-xxhash](https://github.com/ifduyue/python-xxhash)
- [Go: cespare/xxhash](https://github.com/cespare/xxhash)
- [Java: LZ4-Java](https://github.com/lz4/lz4-java)
