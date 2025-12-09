# Cross-Compilation Tooling Guide

## Core Toolchain: rustup and Cargo

The foundation of Rust cross-compilation is **rustup**, the official toolchain manager.

### Adding Targets

```bash
# Add support for 64-bit ARM Linux
rustup target add aarch64-unknown-linux-gnu

# Add support for WebAssembly
rustup target add wasm32-unknown-unknown

# List all installed targets
rustup target list --installed

# List all available targets
rustup target list
```

### Cargo Cross-Compilation

Cargo has built-in support via the `--target` flag:

```bash
cargo build --target aarch64-unknown-linux-gnu --release
```

Output goes to `target/<triple>/release/` or `target/<triple>/debug/`.

**Limitation:** Cargo alone doesn't handle linking or native dependencies—you need linkers configured for each target.

---

## cargo-zigbuild

Uses the **Zig toolchain as a cross-linker**, dramatically simplifying setup.

### Installation

```bash
# Via cargo
cargo install --locked cargo-zigbuild

# Or via pip (includes Zig)
pip install cargo-zigbuild
```

### Usage

```bash
# Ensure target is installed
rustup target add aarch64-unknown-linux-gnu

# Build with zigbuild
cargo zigbuild --target aarch64-unknown-linux-gnu --release
```

### Key Features

- **No GCC toolchains needed** per target
- **glibc version control** - specify minimum glibc:
  ```bash
  cargo zigbuild --target x86_64-unknown-linux-gnu.2.17 --release
  ```
- **musl static binaries** supported
- **Multi-target builds** in single command

### Multi-Architecture Docker Builds

```dockerfile
FROM rust:latest AS builder
RUN cargo install cargo-zigbuild
WORKDIR /app
COPY . .

# Build for both architectures
RUN cargo zigbuild -r \
  --target x86_64-unknown-linux-musl \
  --target aarch64-unknown-linux-musl && \
  mkdir -p /app/linux && \
  cp target/aarch64-unknown-linux-musl/release/myapp /app/linux/arm64 && \
  cp target/x86_64-unknown-linux-musl/release/myapp /app/linux/amd64

FROM alpine:latest AS runtime
WORKDIR /app
ARG TARGETPLATFORM
COPY --from=builder /app/linux/${TARGETPLATFORM} /app/myapp
CMD ["/app/myapp"]
```

Build with Docker buildx:
```bash
docker buildx build --platform linux/amd64,linux/arm64 -t myapp:latest .
```

---

## Cross (Docker-based)

Uses **pre-configured Docker containers** with all toolchains ready.

### Installation

```bash
cargo install cross --git https://github.com/cross-rs/cross
```

Requires Docker or Podman running.

### Usage

Simply replace `cargo` with `cross`:

```bash
cross build --target aarch64-unknown-linux-gnu --release
cross test --target aarch64-unknown-linux-gnu
```

### Key Features

- **Zero configuration** for most targets
- **Pre-installed toolchains** in containers
- **Native testing** via QEMU emulation
- **Custom images** via `Cross.toml`

### Custom Configuration (Cross.toml)

```toml
[target.aarch64-unknown-linux-gnu]
image = "my-custom-image:latest"

[build.env]
passthrough = ["MY_ENV_VAR"]
```

### When Cross Excels

- Complex native dependencies (OpenSSL, libpq)
- Team reproducibility
- Testing cross-compiled binaries
- CI/CD pipelines

---

## Native Toolchain Setup

For scenarios where Docker isn't available or you want full control.

### Linux Host → Linux ARM64

```bash
# Install toolchain (Ubuntu/Debian)
sudo apt install gcc-aarch64-linux-gnu

# Configure cargo
cat >> ~/.cargo/config.toml << 'EOF'
[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"
EOF

# Build
cargo build --target aarch64-unknown-linux-gnu --release
```

### Linux Host → Windows (MinGW)

```bash
# Install toolchain
sudo apt install mingw-w64

# Configure cargo
cat >> ~/.cargo/config.toml << 'EOF'
[target.x86_64-pc-windows-gnu]
linker = "x86_64-w64-mingw32-gcc"
EOF

# Build
cargo build --target x86_64-pc-windows-gnu --release
```

### macOS Host → Linux

Generally easier to use `cross` or `cargo-zigbuild`. Native setup requires:
- musl-cross-make or similar toolchain
- Complex path configuration

---

## Tool Comparison

| Feature | cargo-zigbuild | cross | Native |
|:--------|:---------------|:------|:-------|
| Setup complexity | Low | Medium (needs Docker) | High |
| No Docker needed | ✓ | ✗ | ✓ |
| Testing cross binaries | ✗ | ✓ (QEMU) | ✗ |
| glibc version control | ✓ | Limited | ✗ |
| Native deps handling | Basic | Excellent | Manual |
| CI/CD integration | Easy | Easy | Complex |
| Build reproducibility | Good | Excellent | Varies |

## Recommendations

1. **Start with cargo-zigbuild** for most projects—simplest setup
2. **Switch to cross** when you need to test cross-compiled binaries or have complex native deps
3. **Use native toolchains** only when Docker unavailable and full control needed
