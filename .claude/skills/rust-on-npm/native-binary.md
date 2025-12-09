# Native Binary Distribution Guide

Ship pure Rust CLI binaries via npm without requiring Node.js at runtime. This is the approach used by esbuild, swc, biome, and similar tools.

## When to Use This Pattern

**Ideal for:**
- CLI tools (formatters, linters, bundlers, transpilers)
- Rust is the source of truth, npm is delivery mechanism
- No Node.js dependency at runtime
- Maximum performance

**The pattern:** Base package + platform-specific packages via `optionalDependencies`.

## Architecture Overview

```
npm/
├── your-cli/                    # Base package
│   ├── package.json             # optionalDependencies list
│   └── lib/index.js             # Platform-aware launcher
├── your-cli-linux-x64/
│   ├── package.json             # os: ["linux"], cpu: ["x64"]
│   └── bin/your-cli             # Compiled binary
├── your-cli-darwin-arm64/
│   ├── package.json             # os: ["darwin"], cpu: ["arm64"]
│   └── bin/your-cli
└── your-cli-win32-x64/
    ├── package.json             # os: ["win32"], cpu: ["x64"]
    └── bin/your-cli.exe
```

## Base Package Setup

### package.json

```json
{
  "name": "your-cli",
  "version": "1.0.0",
  "bin": "lib/index.js",
  "type": "module",
  "optionalDependencies": {
    "your-cli-linux-x64": "1.0.0",
    "your-cli-linux-arm64": "1.0.0",
    "your-cli-darwin-x64": "1.0.0",
    "your-cli-darwin-arm64": "1.0.0",
    "your-cli-win32-x64": "1.0.0",
    "your-cli-win32-arm64": "1.0.0"
  }
}
```

### lib/index.js (Launcher)

```javascript
#!/usr/bin/env node
import { spawnSync } from "node:child_process";
import { createRequire } from "node:module";
import os from "node:os";

const require = createRequire(import.meta.url);

function getBinaryPackageName() {
  const platform = os.platform();
  const arch = os.arch();
  const osName = platform === "win32" ? "win32" : platform;
  return `your-cli-${osName}-${arch}`;
}

function getBinaryPath() {
  const pkg = getBinaryPackageName();
  const exe = process.platform === "win32" ? ".exe" : "";

  try {
    return require.resolve(`${pkg}/bin/your-cli${exe}`);
  } catch {
    throw new Error(
      `No prebuilt binary for ${process.platform}-${process.arch}. ` +
      `Tried: ${pkg}/bin/your-cli${exe}`
    );
  }
}

function run() {
  const result = spawnSync(getBinaryPath(), process.argv.slice(2), {
    stdio: "inherit",
  });
  process.exit(result.status ?? 1);
}

run();
```

## Platform Package Setup

### package.json Template

```json
{
  "name": "your-cli-darwin-arm64",
  "version": "1.0.0",
  "os": ["darwin"],
  "cpu": ["arm64"],
  "files": ["bin/"]
}
```

### Platform Matrix

| Package Suffix | `os` | `cpu` | Target Triple |
|:---------------|:-----|:------|:--------------|
| `linux-x64` | `["linux"]` | `["x64"]` | `x86_64-unknown-linux-gnu` |
| `linux-arm64` | `["linux"]` | `["arm64"]` | `aarch64-unknown-linux-gnu` |
| `darwin-x64` | `["darwin"]` | `["x64"]` | `x86_64-apple-darwin` |
| `darwin-arm64` | `["darwin"]` | `["arm64"]` | `aarch64-apple-darwin` |
| `win32-x64` | `["win32"]` | `["x64"]` | `x86_64-pc-windows-msvc` |
| `win32-arm64` | `["win32"]` | `["arm64"]` | `aarch64-pc-windows-msvc` |

## GitHub Actions CI/CD

### Build Matrix Workflow

```yaml
name: Release

on:
  push:
    tags: ["v*"]

jobs:
  build:
    strategy:
      matrix:
        include:
          - os: ubuntu-20.04
            target: x86_64-unknown-linux-gnu
            npm_os: linux
            npm_cpu: x64
          - os: ubuntu-20.04
            target: aarch64-unknown-linux-gnu
            npm_os: linux
            npm_cpu: arm64
            cross: true
          - os: macos-14
            target: x86_64-apple-darwin
            npm_os: darwin
            npm_cpu: x64
          - os: macos-14
            target: aarch64-apple-darwin
            npm_os: darwin
            npm_cpu: arm64
          - os: windows-2022
            target: x86_64-pc-windows-msvc
            npm_os: win32
            npm_cpu: x64

    runs-on: ${{ matrix.os }}

    steps:
      - uses: actions/checkout@v4

      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - name: Install cross
        if: matrix.cross
        run: cargo install cross

      - name: Build
        run: |
          if [ "${{ matrix.cross }}" = "true" ]; then
            cross build --release --target ${{ matrix.target }}
          else
            cargo build --release --target ${{ matrix.target }}
          fi

      - name: Create npm package
        run: |
          PKG_NAME="your-cli-${{ matrix.npm_os }}-${{ matrix.npm_cpu }}"
          mkdir -p "npm/${PKG_NAME}/bin"

          if [ "${{ matrix.npm_os }}" = "win32" ]; then
            cp "target/${{ matrix.target }}/release/your-cli.exe" "npm/${PKG_NAME}/bin/"
          else
            cp "target/${{ matrix.target }}/release/your-cli" "npm/${PKG_NAME}/bin/"
          fi

          cat > "npm/${PKG_NAME}/package.json" << EOF
          {
            "name": "${PKG_NAME}",
            "version": "${{ github.ref_name }}",
            "os": ["${{ matrix.npm_os }}"],
            "cpu": ["${{ matrix.npm_cpu }}"],
            "files": ["bin/"]
          }
          EOF

      - uses: actions/upload-artifact@v4
        with:
          name: npm-${{ matrix.npm_os }}-${{ matrix.npm_cpu }}
          path: npm/

  publish:
    needs: build
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4

      - uses: actions/download-artifact@v4
        with:
          path: npm-packages
          pattern: npm-*
          merge-multiple: true

      - uses: actions/setup-node@v4
        with:
          registry-url: https://registry.npmjs.org

      - name: Publish platform packages
        env:
          NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}
        run: |
          for pkg in npm-packages/your-cli-*/; do
            cd "$pkg"
            npm publish --access public
            cd -
          done

      - name: Publish base package
        env:
          NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}
        run: |
          cd npm/your-cli
          npm publish --access public
```

## Alternative: Binary Download Helpers

Instead of multiple npm packages, download binaries at install time from GitHub Releases.

### Using simple-binary-install

```javascript
#!/usr/bin/env node
import { Binary } from "simple-binary-install";
import { fileURLToPath } from "node:url";
import { dirname } from "node:path";

const __dirname = dirname(fileURLToPath(import.meta.url));

const bin = new Binary("your-cli", {
  installDirectory: __dirname,
  url: ({ platform, arch }) =>
    `https://github.com/org/repo/releases/download/v1.0.0/your-cli-${platform}-${arch}.tar.gz`,
});

async function main() {
  await bin.run(process.argv.slice(2));
}

main();
```

### package.json

```json
{
  "name": "your-cli",
  "version": "1.0.0",
  "bin": "index.mjs",
  "type": "module",
  "scripts": {
    "postinstall": "node postinstall.mjs"
  },
  "dependencies": {
    "simple-binary-install": "^1.0.0"
  }
}
```

### postinstall.mjs

```javascript
import { Binary } from "simple-binary-install";

const bin = new Binary("your-cli", {
  url: ({ platform, arch }) =>
    `https://github.com/org/repo/releases/download/v1.0.0/your-cli-${platform}-${arch}.tar.gz`,
});

bin.install();
```

## cargo-dist Automation

cargo-dist automates the entire release process including npm installer generation.

### Setup

```bash
cargo install cargo-dist
cargo dist init
```

### Cargo.toml Configuration

```toml
[workspace.metadata.dist]
# Create npm installer
installers = ["npm"]

# Target platforms
targets = [
  "x86_64-unknown-linux-gnu",
  "aarch64-unknown-linux-gnu",
  "x86_64-apple-darwin",
  "aarch64-apple-darwin",
  "x86_64-pc-windows-msvc",
]

# npm package name
npm-package = "your-cli"
npm-scope = "@yourscope"
```

### Release

```bash
git tag v1.0.0
git push --tags
# cargo-dist CI handles the rest
```

## Comparison: optionalDependencies vs Postinstall Download

| Aspect | optionalDependencies | Postinstall Download |
|:-------|:---------------------|:--------------------|
| **Packages** | Multiple (one per platform) | Single |
| **Install Speed** | Faster (npm selects) | Slower (network at install) |
| **Offline Install** | Works after first | Requires network |
| **CI Complexity** | Higher | Lower |
| **User Experience** | Seamless | May fail on restricted networks |

## Best Practices

1. **Use npm scopes** - Keeps platform packages organized (`@scope/cli-darwin-arm64`)
2. **Version lock** - All packages must have identical versions
3. **Test cross-platform** - Verify on all target platforms before release
4. **Provide fallback** - Clear error messages when platform isn't supported
5. **Strip binaries** - Reduce size with `strip` on release builds

## Linux GLIBC Considerations

For maximum Linux compatibility, build on older systems or use musl:

```yaml
# Use older Ubuntu for wider glibc compatibility
- os: ubuntu-20.04
  target: x86_64-unknown-linux-gnu

# Or use musl for static linking
- os: ubuntu-latest
  target: x86_64-unknown-linux-musl
```

For musl builds:
```bash
rustup target add x86_64-unknown-linux-musl
cargo build --release --target x86_64-unknown-linux-musl
```
