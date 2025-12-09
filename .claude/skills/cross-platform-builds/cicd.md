# CI/CD Workflows for Cross-Platform Rust

## GitHub Actions Build Matrix

### Basic Multi-Target Build

```yaml
name: Build
on:
  push:
    branches: [main]
  pull_request:

jobs:
  build:
    name: Build ${{ matrix.target }}
    runs-on: ${{ matrix.os }}
    strategy:
      fail-fast: false
      matrix:
        include:
          - target: x86_64-unknown-linux-gnu
            os: ubuntu-latest
          - target: x86_64-unknown-linux-musl
            os: ubuntu-latest
          - target: aarch64-unknown-linux-gnu
            os: ubuntu-latest
          - target: x86_64-apple-darwin
            os: macos-latest
          - target: aarch64-apple-darwin
            os: macos-latest
          - target: x86_64-pc-windows-msvc
            os: windows-latest

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-action@stable
        with:
          targets: ${{ matrix.target }}

      - name: Install cargo-zigbuild
        if: matrix.os == 'ubuntu-latest'
        run: pip install cargo-zigbuild

      - name: Build (Linux with zigbuild)
        if: matrix.os == 'ubuntu-latest'
        run: cargo zigbuild --target ${{ matrix.target }} --release

      - name: Build (macOS/Windows native)
        if: matrix.os != 'ubuntu-latest'
        run: cargo build --target ${{ matrix.target }} --release

      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: binary-${{ matrix.target }}
          path: target/${{ matrix.target }}/release/myapp*
```

### Release Workflow with Assets

```yaml
name: Release
on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    name: Build ${{ matrix.target }}
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        include:
          - target: x86_64-unknown-linux-musl
            os: ubuntu-latest
            artifact: myapp-linux-x64
          - target: aarch64-unknown-linux-musl
            os: ubuntu-latest
            artifact: myapp-linux-arm64
          - target: x86_64-apple-darwin
            os: macos-latest
            artifact: myapp-macos-x64
          - target: aarch64-apple-darwin
            os: macos-latest
            artifact: myapp-macos-arm64
          - target: x86_64-pc-windows-msvc
            os: windows-latest
            artifact: myapp-windows-x64.exe

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-action@stable
        with:
          targets: ${{ matrix.target }}

      - name: Install cargo-zigbuild
        if: matrix.os == 'ubuntu-latest'
        run: pip install cargo-zigbuild

      - name: Build
        run: |
          if [ "${{ matrix.os }}" = "ubuntu-latest" ]; then
            cargo zigbuild --target ${{ matrix.target }} --release
          else
            cargo build --target ${{ matrix.target }} --release
          fi
        shell: bash

      - name: Prepare artifact (Unix)
        if: matrix.os != 'windows-latest'
        run: |
          cp target/${{ matrix.target }}/release/myapp ${{ matrix.artifact }}
          chmod +x ${{ matrix.artifact }}

      - name: Prepare artifact (Windows)
        if: matrix.os == 'windows-latest'
        run: |
          cp target/${{ matrix.target }}/release/myapp.exe ${{ matrix.artifact }}

      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: ${{ matrix.artifact }}
          path: ${{ matrix.artifact }}

  release:
    needs: build
    runs-on: ubuntu-latest
    steps:
      - name: Download all artifacts
        uses: actions/download-artifact@v4

      - name: Create Release
        uses: softprops/action-gh-release@v1
        with:
          files: |
            myapp-linux-x64/myapp-linux-x64
            myapp-linux-arm64/myapp-linux-arm64
            myapp-macos-x64/myapp-macos-x64
            myapp-macos-arm64/myapp-macos-arm64
            myapp-windows-x64.exe/myapp-windows-x64.exe
          generate_release_notes: true
```

### Using `cross` in CI

```yaml
jobs:
  build:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        target:
          - x86_64-unknown-linux-gnu
          - aarch64-unknown-linux-gnu
          - armv7-unknown-linux-gnueabihf

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-action@stable

      - name: Install cross
        run: cargo install cross --git https://github.com/cross-rs/cross

      - name: Build
        run: cross build --target ${{ matrix.target }} --release

      - name: Test (via QEMU)
        run: cross test --target ${{ matrix.target }}
```

---

## Docker Multi-Architecture Images

### Build Script

```yaml
name: Docker Multi-Arch
on:
  push:
    tags: ['v*']

jobs:
  docker:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Set up QEMU
        uses: docker/setup-qemu-action@v3

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3

      - name: Login to GHCR
        uses: docker/login-action@v3
        with:
          registry: ghcr.io
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      - name: Build and push
        uses: docker/build-push-action@v5
        with:
          context: .
          platforms: linux/amd64,linux/arm64
          push: true
          tags: ghcr.io/${{ github.repository }}:${{ github.ref_name }}
```

### Optimized Dockerfile

```dockerfile
# Build stage - uses cargo-zigbuild for fast multi-arch
FROM rust:1.75-bookworm AS builder

RUN pip install cargo-zigbuild
RUN rustup target add x86_64-unknown-linux-musl aarch64-unknown-linux-musl

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build both architectures
RUN cargo zigbuild --release \
    --target x86_64-unknown-linux-musl \
    --target aarch64-unknown-linux-musl

# Organize binaries by platform
RUN mkdir -p /out/linux && \
    cp target/x86_64-unknown-linux-musl/release/myapp /out/linux/amd64 && \
    cp target/aarch64-unknown-linux-musl/release/myapp /out/linux/arm64

# Runtime stage - minimal
FROM alpine:3.19 AS runtime
RUN apk add --no-cache ca-certificates

ARG TARGETARCH
COPY --from=builder /out/linux/${TARGETARCH} /usr/local/bin/myapp

ENTRYPOINT ["/usr/local/bin/myapp"]
```

---

## cargo-dist Automation

For comprehensive release automation including npm/Homebrew:

```bash
# Initialize cargo-dist
cargo install cargo-dist
cargo dist init

# Answer prompts for:
# - Target platforms
# - Installer types (shell, PowerShell, npm, Homebrew)
# - CI provider (GitHub Actions)

# Generates .github/workflows/release.yml
# Commit and push to enable
```

### Generated Capabilities

- Multi-platform binary builds
- Installers (shell script, PowerShell)
- Homebrew tap updates
- npm package publishing
- Checksums and signatures

---

## Best Practices

### 1. Fail-Fast Strategy

```yaml
strategy:
  fail-fast: false  # Continue other builds if one fails
```

### 2. Caching Dependencies

```yaml
- uses: Swatinem/rust-cache@v2
  with:
    key: ${{ matrix.target }}
```

### 3. Conditional Platform Steps

```yaml
- name: Install Linux deps
  if: runner.os == 'Linux'
  run: sudo apt-get install -y libssl-dev

- name: Install macOS deps
  if: runner.os == 'macOS'
  run: brew install openssl
```

### 4. Matrix Exclusions

```yaml
strategy:
  matrix:
    os: [ubuntu-latest, macos-latest, windows-latest]
    target: [x86_64, aarch64]
    exclude:
      # Windows ARM64 not commonly needed
      - os: windows-latest
        target: aarch64
```

### 5. Reusable Workflows

```yaml
# .github/workflows/build-rust.yml
name: Build Rust
on:
  workflow_call:
    inputs:
      target:
        required: true
        type: string

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      # ... build steps
```

Use in other workflows:
```yaml
jobs:
  linux:
    uses: ./.github/workflows/build-rust.yml
    with:
      target: x86_64-unknown-linux-musl
```
