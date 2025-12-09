# xxHash for TypeScript

Multiple libraries implement xxHash for TypeScript/JavaScript. This guide covers the options and provides environment-specific recommendations.

## Library Comparison

| Library | Type | Algorithms | Node | Browser | Workers | Deno/Bun |
|---------|------|------------|------|---------|---------|----------|
| **hash-wasm** | WASM | XXH32/64/3/128 + crypto | Yes | Yes | Yes | Yes |
| **xxhash-wasm** | WASM | XXH32/64 | Yes | Yes | Yes | Yes |
| **@node-rs/xxhash** | Native | XXH32/64/3 | Yes | No | No | Bun only |
| **js-xxhash** | Pure JS | XXH32 | Yes | Yes | Yes | Yes |
| **xxhashjs** | Pure JS | XXH32/64 | Yes | Yes | Yes | Yes |

## Primary Recommendation: `hash-wasm`

Best overall choice for cross-runtime TypeScript projects.

### Why hash-wasm?

- **Cross-runtime**: Works in Node, browsers, Deno, Workers
- **Full algorithm coverage**: XXH32, XXH64, XXH3, XXH128
- **Great performance**: Near-native WASM speed, faster than xxhash-wasm
- **Tree-shakable**: Only import what you need
- **TypeScript-first**: Bundled type definitions
- **General hash toolbox**: Also includes SHA, BLAKE3, Argon2, etc.

### Installation

```bash
pnpm add hash-wasm
```

### Basic Usage

```typescript
import { xxhash64, xxhash32, xxhash3, xxhash128 } from 'hash-wasm';

// One-shot hashing (returns hex string)
const hash64 = await xxhash64('Hello, xxHash!');
const hash32 = await xxhash32('Hello, xxHash!');
const hash3 = await xxhash3('Hello, xxHash!');    // XXH3 64-bit
const hash128 = await xxhash128('Hello, xxHash!'); // XXH3 128-bit

// With seed
const seededHash = await xxhash64('data', 42);

// From Uint8Array
const bytes = new TextEncoder().encode('hello');
const hashFromBytes = await xxhash64(bytes);
```

### Streaming API

```typescript
import { createXXHash64 } from 'hash-wasm';

const hasher = await createXXHash64();
hasher.init();  // Optional seed parameter
hasher.update('Hello, ');
hasher.update('xxHash!');
hasher.update(new Uint8Array([1, 2, 3]));
const hash = hasher.digest();  // hex string
```

## Node.js Performance: `@node-rs/xxhash`

For Node.js-only applications where maximum performance is critical.

### Why @node-rs/xxhash?

- **Fastest option** in Node.js (native Rust via N-API)
- **Synchronous API** (no async overhead)
- **Full algorithm support** including XXH3

### Installation

```bash
pnpm add @node-rs/xxhash
```

### Basic Usage

```typescript
import { xxh32, xxh64, xxh3 } from '@node-rs/xxhash';

// One-shot (synchronous!)
const h32: number = xxh32('hello', 0);
const h64: bigint = xxh64('hello', 0n);

// XXH3
const h3_64: bigint = xxh3.xxh64('hello');
const h3_128: bigint = xxh3.xxh128('hello');
```

### Streaming API

```typescript
import { Xxh32, Xxh64, xxh3 } from '@node-rs/xxhash';

// 32-bit streaming
const hasher32 = new Xxh32(0);
hasher32.update('hello ').update('world');
const digest32: number = hasher32.digest();

// XXH3 streaming
const hasher3 = new xxh3.Xxh3();
hasher3.update('hello ').update('world');
const digest3: bigint = hasher3.digest();
```

### Limitations

- **Node.js only** (no browser, Workers, Deno)
- Native module requires compilation or prebuilt binaries
- Can complicate bundling (esbuild, webpack)

## Cross-Platform xxHash-Only: `xxhash-wasm`

When you only need xxHash and want a smaller bundle than hash-wasm.

### Installation

```bash
pnpm add xxhash-wasm
```

### Basic Usage

```typescript
import xxhash from 'xxhash-wasm';

const { h32, h64, h32ToString, h64ToString } = await xxhash();

// One-shot
const hash32: number = h32('input string');
const hash64: bigint = h64('input string');

// As hex strings
const hex32: string = h32ToString('input string');
const hex64: string = h64ToString('input string');
```

### Streaming API

```typescript
import xxhash from 'xxhash-wasm';

const { create32, create64 } = await xxhash();

// With seed
const hasher = create64(42n);
hasher.update('some data');
hasher.update(new Uint8Array([1, 2, 3]));
const digest = hasher.digest();  // BigInt
```

### Cloudflare Workers

xxhash-wasm has explicit Workers support via conditional exports.

## Pure JavaScript: `js-xxhash`

When WASM and native modules are not options.

### Installation

```bash
pnpm add js-xxhash
```

### Usage

```typescript
import { xxHash32 } from 'js-xxhash';

// Synchronous, no initialization needed
const hash = xxHash32('text to hash', 0);
console.log(hash.toString(16));
```

### Limitations

- **XXH32 only** (no 64-bit or XXH3)
- Slower than WASM implementations
- Best for small data only

## Environment Decision Matrix

| Environment | Recommended | Alternative |
|-------------|-------------|-------------|
| Node.js (performance critical) | `@node-rs/xxhash` | `hash-wasm` |
| Node.js (general) | `hash-wasm` | `xxhash-wasm` |
| Browser | `hash-wasm` | `xxhash-wasm` |
| Cloudflare Workers | `xxhash-wasm` | `hash-wasm` |
| Deno/Bun | `hash-wasm` | `xxhash-wasm` |
| No WASM allowed | `js-xxhash` | `xxhashjs` |
| Need multiple hash algorithms | `hash-wasm` | - |

## Bundle Size

| Library | Size (minified) |
|---------|-----------------|
| js-xxhash | ~8.5KB |
| xxhash-wasm | ~25-30KB (includes WASM) |
| hash-wasm | ~20KB per algorithm (tree-shakable) |

## Common Patterns

### Singleton Initialization

```typescript
import xxhash from 'xxhash-wasm';

// Module-level singleton
let hasherPromise: ReturnType<typeof xxhash> | null = null;

async function getHasher() {
  if (!hasherPromise) {
    hasherPromise = xxhash();
  }
  return hasherPromise;
}

// Usage
export async function hash64(data: string): Promise<bigint> {
  const { h64 } = await getHasher();
  return h64(data);
}
```

### File Hashing (Node.js)

```typescript
import { createReadStream } from 'fs';
import { createXXHash64 } from 'hash-wasm';

async function hashFile(path: string): Promise<string> {
  const hasher = await createXXHash64();
  hasher.init();

  return new Promise((resolve, reject) => {
    const stream = createReadStream(path);
    stream.on('data', chunk => hasher.update(chunk));
    stream.on('end', () => resolve(hasher.digest()));
    stream.on('error', reject);
  });
}
```

### Cache Key Generation

```typescript
import { xxhash64 } from 'hash-wasm';

async function getCacheKey(userId: string, params: object): Promise<string> {
  const input = `${userId}:${JSON.stringify(params)}`;
  return xxhash64(input);
}
```

### Content Hash for ETags

```typescript
import { xxhash64 } from 'hash-wasm';

async function generateETag(content: string): Promise<string> {
  const hash = await xxhash64(content);
  return `"${hash}"`;  // ETags are quoted
}
```

## Performance Tips

1. **Initialize once** - cache the WASM module
2. **Use streaming** for large data (>1MB)
3. **Prefer Uint8Array** over strings for binary data
4. **Use @node-rs/xxhash** in Node for max speed
5. **Avoid repeated small hashes** - batch if possible

## Related

- [Variant Comparison](./variants.md) - XXH32 vs XXH64 vs XXH3
- [Rust Implementation](./rust.md) - For backend/CLI tools
- [Usage Examples](./usage-examples.md) - More code examples
