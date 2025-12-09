# xxHash Variant Comparison

XXHash comes in four main variants: **XXH32**, **XXH64**, **XXH3_64bits**, and **XXH3_128bits**. Each offers different trade-offs between speed, hash size, and collision resistance.

## Variant Overview

### XXH32

- **Bit Width**: 32-bit
- **Speed**: ~9.7 GB/s
- **Best For**: Legacy 32-bit systems, memory-constrained environments, embedded systems
- **Characteristics**:
  - Good balance of speed and collision resistance
  - Passes all SMHasher tests
  - Outperformed by XXH64/XXH3 on 64-bit systems

### XXH64

- **Bit Width**: 64-bit
- **Speed**: ~19.4 GB/s
- **Best For**: General-purpose hashing on 64-bit systems
- **Characteristics**:
  - Significantly faster than XXH32 on 64-bit architectures
  - Larger hash space reduces collision probability
  - Widely adopted in databases and caching systems (Redis)

### XXH3 (64-bit and 128-bit)

- **Bit Width**: 64-bit or 128-bit
- **Speed**: ~31.5 GB/s (64-bit), ~29.6 GB/s (128-bit)
- **Best For**: All new applications, performance-critical systems
- **Characteristics**:
  - Optimized for modern CPUs with SIMD (SSE2, AVX2, NEON)
  - Excellent small data performance (ideal for hash tables, bloom filters)
  - XXH3_128bits offers strongest collision resistance with minimal speed penalty

## Performance Benchmarks

Based on Intel i7-9700K (Ubuntu x64 20.04, clang -O3):

| Hash Name | Bit Width | Bandwidth (GB/s) | Small Data Velocity | Quality |
|-----------|-----------|------------------|---------------------|---------|
| **XXH3 (SSE2)** | 64 | 31.5 | 133.1 | 10 |
| **XXH128 (SSE2)** | 128 | 29.6 | 118.1 | 10 |
| **XXH64** | 64 | 19.4 | 71.0 | 10 |
| **XXH32** | 32 | 9.7 | 71.9 | 10 |
| City64 | 64 | 22.0 | 76.6 | 10 |
| Murmur3 | 32 | 3.9 | 56.1 | 10 |
| SipHash | 64 | 3.0 | 43.2 | 10 |

**Note**: XXH3 can achieve faster-than-RAM speeds when data is in CPU cache (L3 or better).

## Use Case Recommendations

| Variant | Ideal Use Cases | Example Applications |
|---------|-----------------|----------------------|
| **XXH32** | Embedded systems, 32-bit architectures, memory-constrained | Firmware hashing, lightweight checksums |
| **XXH64** | General-purpose on 64-bit, databases, network protocols | Redis, file integrity verification |
| **XXH3_64bits** | High-performance hash tables, real-time processing, caching | Bloom filters, in-memory databases |
| **XXH3_128bits** | Security-sensitive non-crypto, large-scale deduplication | Content-addressable storage, distributed file systems |

## Decision Flowchart

```
Start
  |
  v
Is this a 32-bit embedded/legacy system?
  |-- Yes --> Use XXH32
  |
  No
  |
  v
Do you need maximum collision resistance?
  |-- Yes --> Use XXH3_128bits
  |
  No
  |
  v
Use XXH3_64bits (default recommendation)
```

## Migration from XXH32/XXH64 to XXH3

If upgrading:

1. XXH3 produces **different hashes** - not backward compatible
2. Stored hashes will need regeneration
3. Consider versioning your hash format
4. XXH3 requires modern CPU for full SIMD performance

## Seed Values

All variants support seeding for hash randomization:

```rust
// Rust with seed
xxh3_64_with_seed(data, 42);
xxh64(data, 42);
```

```typescript
// TypeScript with seed
h64(input, 42n);  // BigInt seed
```

Use seeds to:

- Prevent hash collision attacks
- Create multiple independent hash spaces
- Ensure different hash distributions per use case

## Key Strengths

1. **Speed**: All variants operate at near-RAM speeds
2. **Quality**: Passes SMHasher and extended collision tests
3. **Versatility**: Supports 32-bit and 64-bit systems
4. **Adoption**: Used in Redis, Linux kernel, Facebook's folly

## Related

- [Rust Implementation](./rust.md) - Using xxHash in Rust
- [TypeScript Libraries](./typescript.md) - Using xxHash in TypeScript/JavaScript
- [Usage Examples](./usage-examples.md) - Code examples across languages
