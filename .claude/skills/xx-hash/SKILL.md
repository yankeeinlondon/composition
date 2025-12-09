---
name: xx-hash
description: Expert knowledge for xxHash, an extremely fast non-cryptographic hash algorithm. Use when implementing checksums, hash tables, bloom filters, content deduplication, cache keys, or data integrity verification. Covers XXH32, XXH64, XXH3 variants with language-specific guidance for Rust (xxhash-rust, twox-hash), TypeScript (hash-wasm, xxhash-wasm, @node-rs/xxhash), Python, Go, and Java.
hash: 885e8fcd13586d50
---

# xxHash

xxHash is an **extremely fast non-cryptographic hash algorithm** designed to operate at RAM speed limits. It provides exceptional performance while maintaining excellent hash quality and collision resistance.

## Core Principles

1. **Use XXH3 for new applications** - Best performance on modern CPUs with SIMD optimization
2. **Use 64-bit by default** - Best balance of speed and collision resistance
3. **Use 128-bit when collision resistance is critical** - Large-scale deduplication, content-addressable storage
4. **Use streaming API for large data** - Avoid loading entire files into memory
5. **Use consistent seeds across languages** - Required for cross-language interoperability
6. **Verify string encoding** - UTF-8 encoding affects hash output consistency
7. **Never use for security** - Not cryptographic; use SHA-256, BLAKE3, Argon2 for security
8. **Match algorithm to architecture** - XXH32 only for 32-bit systems, XXH64/XXH3 for 64-bit

## Algorithm Variants

| Variant | Bits | Speed (GB/s) | Best For |
|---------|------|--------------|----------|
| **XXH3** | 64/128 | ~31.5 | Default choice, modern applications |
| **XXH64** | 64 | ~19.4 | General-purpose, 64-bit systems |
| **XXH32** | 32 | ~9.7 | Legacy/embedded, 32-bit systems |

See [Variant Comparison](./variants.md) for detailed analysis.

## Quick Reference by Language

### Rust (Recommended: `xxhash-rust`)

```rust
use xxhash_rust::xxh3::xxh3_64;
let hash = xxh3_64(b"data");
```

See [Rust Implementation](./rust.md) for crate comparison and advanced usage.

### TypeScript (Recommended: `hash-wasm`)

```typescript
import { xxhash64 } from 'hash-wasm';
const hash = await xxhash64('data');
```

See [TypeScript Libraries](./typescript.md) for library comparison and environment-specific guidance.

### Python

```bash
pip install xxhash
```

```python
import xxhash
hash = xxhash.xxh3_64(b"data").hexdigest()
```

### Go

```bash
go get github.com/cespare/xxhash/v2
```

```go
hash := xxhash.Sum64String("data")
```

## Topics

### Core Concepts

- [Variant Comparison](./variants.md) - XXH32 vs XXH64 vs XXH3, performance data, decision guide

### Language Implementations

- [Rust Implementation](./rust.md) - xxhash-rust, twox-hash, xxhash-c comparison
- [TypeScript Libraries](./typescript.md) - hash-wasm, xxhash-wasm, @node-rs/xxhash, environment matrix

### Practical Usage

- [Usage Examples](./usage-examples.md) - Multi-language code examples, streaming, HashMap integration

## Decision Guide

| Scenario | Recommendation |
|----------|----------------|
| Hash tables / bloom filters | XXH3_64 |
| File checksums | XXH3_64 or XXH3_128 |
| Content deduplication | XXH3_128 |
| Legacy 32-bit systems | XXH32 |
| Cross-language consistency | Verify seed and encoding |
| Node.js max performance | `@node-rs/xxhash` |
| Browser + Node.js | `hash-wasm` or `xxhash-wasm` |
| Rust performance critical | `xxhash-rust` with SIMD features |
| Rust simplicity | `twox-hash` |

## Important Caveats

**xxHash is NOT cryptographic**. Do NOT use for:

- Password hashing (use Argon2, bcrypt)
- Security signatures (use SHA-256, BLAKE3)
- Tamper detection (use HMAC)

**DO use for**:

- Hash tables and maps
- Checksums and integrity checks
- Content-addressable storage
- Cache key generation
- Bloom filters
- Data deduplication

## Resources

- [Official xxHash](https://github.com/Cyan4973/xxHash)
- [xxHash Performance Wiki](https://github.com/Cyan4973/xxHash/wiki/Performance-comparison)
- [xxhash-rust (Rust)](https://docs.rs/xxhash-rust)
- [hash-wasm (TypeScript)](https://github.com/Daninet/hash-wasm)
- [xxhash-wasm (TypeScript)](https://github.com/jungomi/xxhash-wasm)
- [@node-rs/xxhash](https://github.com/napi-rs/node-rs/tree/main/packages/xxhash)
