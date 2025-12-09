---
name: cross-platform-builds
description: Expert knowledge for Rust cross-platform builds and deployments, covering cross-compilation tools (cross, cargo-zigbuild), target triples, CI/CD workflows, and frameworks like Tauri and Dioxus. Use when building Rust for multiple platforms, setting up cross-compilation, or creating multi-architecture Docker images.
---

# Cross-Platform Rust Development

Expert guidance for building and deploying Rust applications across multiple platforms.

## Approach Overview

| Approach | Use Case | Key Tools | Considerations |
|:---------|:---------|:----------|:---------------|
| **Native Cross-Compilation** | CLI tools, servers, libraries | rustup, cargo, target linker | Manual toolchain/linker setup per platform |
| **Containerized Cross-Compilation** | CI/CD pipelines, complex setups | `cross` (Docker/Podman) | Abstracts host dependencies, reproducible |
| **cargo-zigbuild** | Simple cross-linking, glibc control | Zig toolchain as linker | No GCC toolchains needed per target |
| **GUI Frameworks** | Desktop, mobile, web apps | Tauri, Dioxus, Slint | Manages windowing, rendering, packaging |

## Quick Decision Guide

**Choose `cross` when:**
- Multiple targets with different requirements
- Complex native dependencies
- Reproducible builds across team/CI

**Choose `cargo-zigbuild` when:**
- Want simplicity without Docker
- Need glibc version control
- Creating musl static binaries

**Choose Native Toolchain when:**
- Single target builds
- Full control over configuration
- Limited Docker availability

## Detailed Documentation

- [Tooling Guide](./tooling.md) - cross, cargo-zigbuild, rustup setup
- [CI/CD Workflows](./cicd.md) - GitHub Actions, build matrices
- [GUI Frameworks](./gui-frameworks.md) - Tauri, Dioxus for applications
- [Advanced Topics](./advanced.md) - WebAssembly, musl, embedded

## Essential Commands

```bash
# Add a target
rustup target add aarch64-unknown-linux-gnu

# Build with cargo-zigbuild
cargo zigbuild --target aarch64-unknown-linux-gnu --release

# Build with cross (uses Docker)
cross build --target aarch64-unknown-linux-gnu --release
```

## Common Target Triples

| Target | Platform | Notes |
|:-------|:---------|:------|
| `x86_64-unknown-linux-gnu` | Linux x64 (glibc) | Tier 1, most common |
| `x86_64-unknown-linux-musl` | Linux x64 (static) | Portable, no libc deps |
| `aarch64-unknown-linux-gnu` | Linux ARM64 | Tier 2, Raspberry Pi 4, servers |
| `x86_64-pc-windows-msvc` | Windows x64 | Tier 1, native MSVC |
| `x86_64-pc-windows-gnu` | Windows x64 | MinGW, easier from Linux |
| `x86_64-apple-darwin` | macOS Intel | Tier 1 |
| `aarch64-apple-darwin` | macOS ARM64 | Tier 2, M1/M2/M3 Macs |
| `wasm32-unknown-unknown` | WebAssembly | Browser/Node.js |

## Configuration (.cargo/config.toml)

```toml
[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"

[target.x86_64-unknown-linux-musl]
linker = "rust-lld"
```

## Platform-Specific Notes

**Linux Static Binaries:**
Use musl target for portable binaries that run on any Linux distro.

**macOS Cross-Compilation:**
Building for macOS from non-Apple hardware has legal/SDK constraints. Use macOS CI runners.

**Windows from Linux:**
Prefer `-gnu` targets (MinGW) over `-msvc` for simpler cross-compilation.

## External Resources

- [Rust Platform Support](https://doc.rust-lang.org/nightly/rustc/platform-support.html)
- [cross GitHub](https://github.com/cross-rs/cross)
- [cargo-zigbuild](https://github.com/rust-cross/cargo-zigbuild)
- [Tauri](https://tauri.app)
- [Dioxus](https://dioxuslabs.com)
