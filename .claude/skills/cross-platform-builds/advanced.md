# Advanced Cross-Platform Topics

## WebAssembly Compilation

Rust to WebAssembly (`wasm32-unknown-unknown`) has special considerations.

### Basic Setup

```bash
# Add target
rustup target add wasm32-unknown-unknown

# Install wasm-pack for npm integration
cargo install wasm-pack
```

### wasm-pack Usage

```bash
# Build for Node.js
wasm-pack build --target nodejs

# Build for web (ES modules)
wasm-pack build --target web

# Build for bundlers (webpack, etc.)
wasm-pack build --target bundler
```

### Project Configuration

```toml
# Cargo.toml
[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
wasm-bindgen = "0.2"

[profile.release]
opt-level = "s"      # Optimize for size
lto = true           # Link-time optimization
```

### Maximum Compatibility

WebAssembly is evolving; for maximum browser compatibility:

```bash
export RUSTFLAGS="-Ctarget-cpu=mvp"
cargo +nightly build -Zbuild-std=panic_abort,std --target wasm32-unknown-unknown
```

### Testing WASM

```bash
# With wasm-pack
wasm-pack test --node

# With wasm-bindgen-test
cargo test --target wasm32-unknown-unknown
```

---

## Static Linking with musl

musl libc enables fully static Linux binaries with no runtime dependencies.

### When to Use musl

- **Containerized apps** - minimal images (FROM scratch)
- **Distribution** - single binary runs on any Linux
- **Embedded Linux** - reduced dependencies

### Building musl Binaries

```bash
# Add target
rustup target add x86_64-unknown-linux-musl

# Build with cargo-zigbuild (easiest)
cargo zigbuild --target x86_64-unknown-linux-musl --release

# Or with cross
cross build --target x86_64-unknown-linux-musl --release
```

### Native musl Setup (Ubuntu)

```bash
# Install musl toolchain
sudo apt install musl-tools

# Build
cargo build --target x86_64-unknown-linux-musl --release
```

### Verifying Static Linking

```bash
# Should show "statically linked" or "not a dynamic executable"
file target/x86_64-unknown-linux-musl/release/myapp
ldd target/x86_64-unknown-linux-musl/release/myapp
```

### Trade-offs

| Aspect | musl | glibc |
|:-------|:-----|:------|
| Portability | Excellent | Varies by distro |
| Binary size | Larger (all linked in) | Smaller |
| DNS resolution | Custom (may differ) | System resolver |
| Async I/O perf | Can be slower | Optimized |
| Debugging | Harder | Easier |

### Handling Native Dependencies with musl

Some crates with C dependencies need configuration:

```toml
# .cargo/config.toml
[target.x86_64-unknown-linux-musl]
rustflags = ["-C", "target-feature=-crt-static"]

# For OpenSSL - use vendored feature
[dependencies]
openssl = { version = "0.10", features = ["vendored"] }
```

---

## Embedded Systems

For embedded targets (`thumbv6m-none-eabi`, `thumbv7m-none-eabi`, etc.).

### no_std Development

```rust
#![no_std]
#![no_main]

use panic_halt as _;  // Panic handler
use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    loop {
        // Embedded logic
    }
}
```

### Cargo Configuration

```toml
# .cargo/config.toml
[build]
target = "thumbv7m-none-eabi"

[target.thumbv7m-none-eabi]
runner = "probe-run --chip STM32F103C8"
rustflags = [
  "-C", "link-arg=-Tlink.x",
]
```

### Building

```bash
rustup target add thumbv7m-none-eabi
cargo build --release
```

### Using cross for Embedded

```bash
cross build --target thumbv7m-none-eabi --release
```

---

## Rust Target Tier System

Rust organizes targets into tiers:

### Tier 1 (Guaranteed)

- Upstream testing, guaranteed to build and work
- Examples: `x86_64-unknown-linux-gnu`, `x86_64-pc-windows-msvc`, `x86_64-apple-darwin`

### Tier 2 (Guaranteed to Build)

- Builds are tested, but not the resulting binaries
- Examples: `aarch64-unknown-linux-gnu`, `aarch64-apple-darwin`, most musl targets

### Tier 3 (No Guarantees)

- May require building your own standard library
- Nightly compiler often needed

```bash
# Build std for tier 3 target
cargo +nightly build -Zbuild-std --target <tier3-target>
```

---

## Platform-Specific Compilation

### Conditional Code

```rust
#[cfg(target_os = "windows")]
fn platform_path() -> &'static str {
    "C:\\Program Files\\MyApp"
}

#[cfg(target_os = "linux")]
fn platform_path() -> &'static str {
    "/opt/myapp"
}

#[cfg(target_os = "macos")]
fn platform_path() -> &'static str {
    "/Applications/MyApp.app"
}
```

### Conditional Dependencies

```toml
# Cargo.toml
[target.'cfg(windows)'.dependencies]
winapi = "0.3"

[target.'cfg(unix)'.dependencies]
nix = "0.27"

[target.'cfg(target_os = "macos")'.dependencies]
cocoa = "0.25"
```

### Feature Flags for Platforms

```toml
[features]
default = []
linux-only = []
macos-extras = []
```

```rust
#[cfg(feature = "linux-only")]
mod linux_specific;
```

---

## Native Dependencies Handling

### Common Patterns

**1. Vendored dependencies** (compile from source):
```toml
[dependencies]
openssl = { version = "0.10", features = ["vendored"] }
sqlite = { version = "0.30", features = ["bundled"] }
```

**2. System libraries** (must be installed):
```toml
[dependencies]
openssl = "0.10"  # Requires libssl-dev
```

**3. Pre-built binaries**:
Use `cross` which has pre-compiled libraries in containers.

### build.rs for Cross-Compilation

```rust
// build.rs
fn main() {
    let target = std::env::var("TARGET").unwrap();

    if target.contains("linux") {
        println!("cargo:rustc-link-lib=static=mylib");
        println!("cargo:rustc-link-search=native=libs/{}", target);
    }
}
```

### pkg-config for Cross

```bash
# Set pkg-config for cross target
export PKG_CONFIG_PATH=/usr/lib/aarch64-linux-gnu/pkgconfig
export PKG_CONFIG_ALLOW_CROSS=1
```

---

## Binary Size Optimization

For embedded or distribution where size matters:

```toml
# Cargo.toml
[profile.release]
opt-level = "z"      # Optimize for size
lto = true           # Link-time optimization
codegen-units = 1    # Single codegen unit
panic = "abort"      # Remove unwinding code
strip = true         # Strip symbols
```

Additional tools:
```bash
# Strip after build
strip target/release/myapp

# Analyze binary size
cargo install cargo-bloat
cargo bloat --release
```

---

## Signing and Notarization

### macOS

```bash
# Code sign
codesign --force --sign "Developer ID Application: ..." myapp

# Notarize
xcrun notarytool submit myapp.zip --apple-id ... --team-id ...
xcrun stapler staple myapp
```

### Windows

```powershell
# Sign with signtool
signtool sign /f cert.pfx /p password /tr http://timestamp.digicert.com myapp.exe
```

For CI/CD, use dedicated actions:
- `apple-actions/import-codesign-certs`
- `softprops/action-gh-release` with signing steps
