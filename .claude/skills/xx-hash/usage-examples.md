# xxHash Usage Examples

Practical code examples for using xxHash across multiple programming languages.

## Rust

### Installation

```toml
[dependencies]
xxhash-rust = { version = "0.8", features = ["xxh3", "const_xxh3"] }
```

### Basic Hashing

```rust
use xxhash_rust::xxh3::xxh3_64;
use xxhash_rust::const_xxh3::const_xxh3_64;

const DATA: &[u8] = b"Hello, xxHash!";

// Runtime hashing
let hash = xxh3_64(DATA);
println!("64-bit hash: {:x}", hash);

// Compile-time hashing (zero runtime cost)
const CONST_HASH: u64 = const_xxh3_64(DATA);
println!("Const hash: {:x}", CONST_HASH);
```

### Streaming

```rust
use xxhash_rust::xxh3::Xxh3;

let mut hasher = Xxh3::new();
hasher.update(b"Hello, ");
hasher.update(b"xxHash!");
let hash = hasher.digest();
println!("Streaming hash: {:x}", hash);
```

### With HashMap

```rust
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use xxhash_rust::xxh64::Xxh64;

let mut map = HashMap::<_, _, BuildHasherDefault<Xxh64>>::default();
map.insert(42, "the answer");
assert_eq!(map.get(&42), Some(&"the answer"));
```

## TypeScript

### Using hash-wasm (Recommended)

```typescript
import { xxhash64, createXXHash64 } from 'hash-wasm';

// One-shot
const hash = await xxhash64('data');

// Streaming
const hasher = await createXXHash64();
hasher.init();
hasher.update('chunk1');
hasher.update('chunk2');
const streamHash = hasher.digest();
```

### Using xxhash-wasm

```typescript
import xxhash from 'xxhash-wasm';

const { h32, h64, create32, create64 } = await xxhash();

// One-shot
const hash32 = h32('data');
const hash64 = h64('data');

// Streaming
const hasher = create64();
hasher.update('some data');
hasher.update(new Uint8Array([1, 2, 3]));
const digest = hasher.digest();
```

### Using @node-rs/xxhash (Node.js only)

```typescript
import { xxh64, Xxh64, xxh3 } from '@node-rs/xxhash';

// One-shot (synchronous)
const hash: bigint = xxh64('data', 0n);

// Streaming
const hasher = new Xxh64(0n);
hasher.update('hello ').update('world');
const digest: bigint = hasher.digest();

// XXH3
const h3_64: bigint = xxh3.xxh64('data');
const h3_128: bigint = xxh3.xxh128('data');
```

## Python

### Installation

```bash
pip install xxhash
```

### Basic Hashing

```python
import xxhash

data = b"Hello, xxHash!"

# One-shot
hash_64 = xxhash.xxh64(data).hexdigest()
hash_32 = xxhash.xxh32(data).hexdigest()
hash_3 = xxhash.xxh3_64(data).hexdigest()

# With seed
hash_seeded = xxhash.xxh64(data, seed=42).hexdigest()

print(f"XXH64: {hash_64}")
print(f"XXH32: {hash_32}")
print(f"XXH3: {hash_3}")
```

### Streaming

```python
import xxhash

hasher = xxhash.xxh3_64()
hasher.update(b"Hello, ")
hasher.update(b"xxHash!")

digest = hasher.hexdigest()
int_digest = hasher.intdigest()

print(f"Hex: {digest}")
print(f"Int: {int_digest}")
```

### File Hashing

```python
import xxhash

def hash_file(filepath: str) -> str:
    hasher = xxhash.xxh3_64()
    with open(filepath, 'rb') as f:
        while chunk := f.read(8192):
            hasher.update(chunk)
    return hasher.hexdigest()

file_hash = hash_file('large_file.dat')
print(f"File hash: {file_hash}")
```

## Go

### Installation

```bash
go get github.com/cespare/xxhash/v2
```

### Basic Hashing

```go
package main

import (
    "fmt"
    "github.com/cespare/xxhash/v2"
)

func main() {
    // String hashing
    hash := xxhash.Sum64String("Hello, xxHash!")
    fmt.Printf("Hash: %x\n", hash)

    // Byte slice hashing
    hashBytes := xxhash.Sum64([]byte("Hello, xxHash!"))
    fmt.Printf("Hash from bytes: %x\n", hashBytes)
}
```

### Streaming

```go
package main

import (
    "fmt"
    "github.com/cespare/xxhash/v2"
)

func main() {
    hasher := xxhash.New()
    hasher.Write([]byte("Hello, "))
    hasher.Write([]byte("xxHash!"))
    hash := hasher.Sum64()
    fmt.Printf("Streaming hash: %x\n", hash)
}
```

### File Hashing

```go
package main

import (
    "fmt"
    "io"
    "os"
    "github.com/cespare/xxhash/v2"
)

func hashFile(filepath string) (uint64, error) {
    file, err := os.Open(filepath)
    if err != nil {
        return 0, err
    }
    defer file.Close()

    hasher := xxhash.New()
    if _, err := io.Copy(hasher, file); err != nil {
        return 0, err
    }

    return hasher.Sum64(), nil
}

func main() {
    hash, _ := hashFile("large_file.dat")
    fmt.Printf("File hash: %x\n", hash)
}
```

### Custom Seed

```go
package main

import (
    "fmt"
    "github.com/cespare/xxhash/v2"
)

func main() {
    seed := uint64(42)
    hasher := xxhash.NewWithSeed(seed)
    hasher.Write([]byte("data"))
    hash := hasher.Sum64()
    fmt.Printf("Seeded hash: %x\n", hash)
}
```

## Java

### Installation (Maven)

```xml
<dependency>
    <groupId>org.lz4</groupId>
    <artifactId>lz4-java</artifactId>
    <version>1.8.0</version>
</dependency>
```

### Basic Hashing

```java
import net.jpountz.xxhash.XXHashFactory;
import net.jpountz.xxhash.XXHash64;

public class XXHashExample {
    public static void main(String[] args) {
        XXHashFactory factory = XXHashFactory.fastestInstance();
        XXHash64 hasher = factory.hash64();

        byte[] data = "Hello, xxHash!".getBytes();
        long hash = hasher.hash(data, 0, data.length, 0L);

        System.out.println("Hash: " + Long.toHexString(hash));
    }
}
```

### Streaming

```java
import net.jpountz.xxhash.XXHashFactory;
import net.jpountz.xxhash.StreamingXXHash64;

public class StreamingExample {
    public static void main(String[] args) {
        XXHashFactory factory = XXHashFactory.fastestInstance();
        StreamingXXHash64 hasher = factory.newStreamingHash64(0L);

        hasher.update("Hello, ".getBytes());
        hasher.update("xxHash!".getBytes());

        long hash = hasher.getValue();
        System.out.println("Streaming hash: " + Long.toHexString(hash));
    }
}
```

## Cross-Language Consistency

When using xxHash across languages, ensure:

1. **Same algorithm variant** (XXH32, XXH64, or XXH3)
2. **Same seed value** (default is usually 0)
3. **Consistent string encoding** (UTF-8)
4. **Same byte order** for binary data

### Verification Example

All these should produce the same hash for "test" with seed 0:

```python
# Python
import xxhash
print(xxhash.xxh64(b"test", seed=0).intdigest())
# Output: 5754696928334414137
```

```typescript
// TypeScript
import { xxh64 } from '@node-rs/xxhash';
console.log(xxh64('test', 0n).toString());
// Output: 5754696928334414137n
```

```rust
// Rust
use xxhash_rust::xxh64::xxh64;
println!("{}", xxh64(b"test", 0));
// Output: 5754696928334414137
```

```go
// Go
hash := xxhash.Sum64String("test")
fmt.Println(hash)
// Output: 5754696928334414137
```

## Performance Comparison

Approximate throughput by language (XXH3/XXH64):

| Language | Implementation | Throughput |
|----------|----------------|------------|
| Rust | xxhash-rust | ~31.5 GB/s |
| Go | cespare/xxhash | ~19.4 GB/s |
| Python | python-xxhash | ~10-15 GB/s |
| Java | LZ4-Java | ~15-20 GB/s |
| TypeScript | xxhash-wasm | ~5-10 GB/s |
| TypeScript | @node-rs/xxhash | ~15-20 GB/s |

*Benchmarks vary by hardware and input size.*

## Best Practices (All Languages)

1. **Choose XXH3** for new code - fastest and best quality
2. **Use streaming** for large data (>1MB)
3. **Use consistent seeds** across services
4. **Verify encoding** - UTF-8 is standard
5. **Cache hasher instances** where possible
6. **Never use for security** - not cryptographic

## Related

- [Variant Comparison](./variants.md) - Choosing the right algorithm
- [Rust Implementation](./rust.md) - Rust-specific details
- [TypeScript Libraries](./typescript.md) - TypeScript-specific details
