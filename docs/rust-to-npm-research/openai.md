# Shipping Rust programs on `npm`

You’ve basically got three big shapes for “Rust CLI via npm”:

 1. Ship native Rust binaries and wrap them with npm (what esbuild, swc, biome-style tools do).
 2. Expose Rust as a Node native addon (napi-rs / neon) and build a JS CLI around that.
 3. Compile to WebAssembly (wasm-pack) and ship a JS wrapper CLI.

For a CLI specifically, (1) is usually the sweet spot. I’ll start there and then cover the others.

⸻

## 1. Pattern: native Rust binaries wrapped in npm

This is “pure Rust CLI, npm is just a delivery mechanism”. Users run:

```sh
npm install -g your-cli
# or
npx your-cli@latest …
```

1.1 Classic pattern: base package + per-platform packages

Orhun’s “Packaging Rust Applications for the NPM Registry” is the cleanest write-up of this pattern. ￼

Core idea:
 • Base package: your-cli
 • Has "bin": "lib/index.js" so npm installs a JS launcher script on $PATH.
 • Declares platform-specific packages in optionalDependencies.
 • Platform packages: your-cli-linux-x64, your-cli-darwin-arm64, etc.
 • Each one:
 • Has "os": ["linux"] / "darwin" / "win32" etc.
 • Has "cpu": ["x64"] / "arm64" etc.
 • Contains the compiled Rust binary under bin/your-cli[.exe].

At install time npm decides which optional deps match the current os and cpu, and only installs that package. ￼

Example npm/your-cli/package.json:

```json
{
  "name": "your-cli",
  "version": "0.1.0",
  "bin": "lib/index.js",
  "type": "module",
  "optionalDependencies": {
    "your-cli-linux-x64": "0.1.0",
    "your-cli-linux-arm64": "0.1.0",
    "your-cli-darwin-x64": "0.1.0",
    "your-cli-darwin-arm64": "0.1.0",
    "your-cli-windows-x64": "0.1.0",
    "your-cli-windows-arm64": "0.1.0"
  }
}
```

Base package launcher (npm/your-cli/src/index.ts):

This is a typesafe version of the launcher Orhun describes. ￼

```js
# !/usr/bin/env node
import { spawnSync } from "node:child_process";
import os from "node:os";

function binaryPackageName(): string {
  const platform = os.platform(); // 'darwin' | 'linux' | 'win32' | 'aix' | ...
  const arch = os.arch();        // 'x64' | 'arm64' | 'arm' | ...

  const osName = platform === "win32" || platform === "cygwin" ? "windows" : platform;
  return `your-cli-${osName}-${arch}`;
}

function binaryPath(): string {
  const pkg = binaryPackageName();
  const exeSuffix =
    process.platform === "win32" || process.platform === "cygwin" ? ".exe" : "";
  try {
    // Resolve into that package's bin folder
    return require.resolve(`${pkg}/bin/your-cli${exeSuffix}`);
  } catch {
    throw new Error(
      `No prebuilt binary found for ${process.platform}-${process.arch}.` +
        `Tried to resolve ${pkg}/bin/your-cli${exeSuffix}`
    );
  }
}

function run() {
  const args = process.argv.slice(2);
  const result = spawnSync(binaryPath(), args, { stdio: "inherit" });
  process.exit(result.status ?? 0);
}

run();
```

Each platform package is tiny; its package.json can be generated from a template in CI:

```json
{
  "name": "your-cli-darwin-arm64",
  "version": "0.1.0",
  "os": ["darwin"],
  "cpu": ["arm64"]
  // plus "files": ["bin/"], etc.
}
```

…and the directory structure ends up like:

```txt
npm/
  your-cli/
    package.json     # base
    lib/index.js     # launcher
  your-cli-linux-x64/
    bin/your-cli     # compiled binary
    package.json
  your-cli-darwin-arm64/
    bin/your-cli
    package.json
```

Orhun’s article also shows how to generate those per-platform packages and publish them from a GitHub Actions build matrix. ￼

### 1.2 Building the binaries (CI setup)

Typical CI setup (again, basically what Orhun shows):
 • GitHub Actions matrix over {OS, arch, target triple}, e.g.:
 • ubuntu-20.04 / x86_64-unknown-linux-gnu
 • ubuntu-20.04 / aarch64-unknown-linux-gnu
 • windows-2022 / x86_64-pc-windows-msvc
 • macos-14 / x86_64-apple-darwin
 • macos-14 / aarch64-apple-darwin ￼
 • Use actions-rs/toolchain to install Rust for each target and cross for Linux cross-compiles if you want. ￼
 • For each job:
 • cargo build --release --locked --target $TARGET
 • Create npm/your-cli-${os}-${arch}/bin/your-cli and copy the built binary into it.
 • Generate its package.json from a template using envsubst or a small script.
 • npm publish that platform package.
 • Finally, in a dependent job, build and publish the base your-cli package (after all optionalDependencies exist on npm). ￼

### 1.3 cargo-dist: automate a lot of this

cargo-dist￼ (“dist”) is increasingly the go-to “make my Rust app shippable everywhere” tool. It:
 • Plans builds and builds binaries/tarballs/installers for multiple targets. ￼
 • Generates CI workflows that:
 • Build for a target matrix.
 • Upload artifacts to GitHub Releases / package managers.
 • Supports “installers” such as shell/PowerShell scripts and npm packages (used by real projects like Pomsky to provide npm install -g @pomsky-lang/cli). ￼

In practice:
 • You add a [workspace.metadata.dist] section in Cargo.toml to declare your binaries and which installers you want.
 • Run dist init to generate CI (e.g. .github/workflows/release.yml). ￼
 • Push a vX.Y.Z tag; dist’s workflow builds binaries and publishes installers, including an npm installer package if configured. ￼

This is a very “batteries included” way to get to npm install -g your-cli without hand-rolling all the npm plumbing.

### 1.4 Binary download helpers: binary-install / simple-binary-install

An alternative to the “base + per-platform npm pkg” pattern is:
 • Publish binaries to GitHub Releases (or another URL).
 • Make a single npm package whose postinstall script downloads the correct binary for the user’s platform.

Cloudflare’s binary-install library is built exactly for this use case: install binary applications via npm using GitHub Releases. ￼

There’s also simple-binary-install, a leaner successor that refactors binary-install to use fewer dependencies and streaming APIs. ￼

Rough shape using binary-install / simple-binary-install:

index.mjs (the CLI entrypoint):

```js
#!/usr/bin/env node
import { Binary } from "simple-binary-install"; // or "binary-install"

const repo = "your-org/your-cli"; // GitHub org/repo
const name = "your-cli";
const version = "v0.1.0";

const bin = new Binary(name, {
  installDirectory: __dirname, // or node_modules/.bin, etc.
  // This URL pattern must match your release artifacts:
  // e.g. <https://github.com/your-org/your-cli/releases/download/v0.1.0/your-cli-darwin-arm64.tar.gz>
  url: ({ platform, arch }) =>
    `https://github.com/${repo}/releases/download/${version}/${name}-${platform}-${arch}.tar.gz`
});

async function main() {
  await bin.run(process.argv.slice(2)); // ensures installed, then execs
}

main().catch(err => {
  console.error(err);
  process.exit(1);
});
```

package.json scripting:

```json
{
  "name": "your-cli",
  "version": "0.1.0",
  "bin": "index.mjs",
  "type": "module",
  "scripts": {
    "postinstall": "node postinstall.mjs"
  }
}
```

Where postinstall.mjs basically calls bin.install() to download/extract once. ￼

This pattern trades npm’s os/cpu filtering for a simpler single package, but depends on network availability at install time and careful hosting of artifacts.

⸻

## 2. Pattern: Rust as a Node native addon (napi-rs / neon) + CLI wrapper

Here your CLI is actually a Node process whose heavy lifting is inside a Rust Node-API addon.

### 2.1 Tools

 • napi-rs (napi crate + @napi-rs/cli):
 • Framework for building Node-API addons in Rust; no node-gyp required. ￼
 • Ships a CLI that scaffolds projects and generates GitHub Actions to build prebuilt .node binaries for multiple platforms and publish to npm. ￼
 • neon and node-bindgen:
 • Alternative Rust wrappers for Node-API.
 • Often paired with node-pre-gyp, prebuild, or prebuildify to ship prebuilt binaries. ￼

This approach is great if:
 • You expect consumers to be in the Node ecosystem anyway.
 • You want a programmatic JS API plus a CLI, reusing the same Rust core.

### 2.2 Minimal napi-rs CLI example

Rust side (Cargo.toml):

```toml
[package]
name = "your_cli_core"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
napi = { version = "3", features = ["napi4"] }
napi-derive = "3"
```

Rust addon (src/lib.rs):

```rs
use napi::bindgen_prelude::*;

# [napi]
pub fn greet(name: String) -> String {
    format!("Hello, {name} from Rust!")
}
```

JS CLI (cli.mjs):

```js
# !/usr/bin/env node
import { greet } from "./index.node"; // name produced by napi-rs build

const name = process.argv[2] ?? "world";
console.log(greet(name));
```

package.json:

```json
{
  "name": "@your-scope/your-cli",
  "version": "0.1.0",
  "main": "index.node",
  "bin": {
    "your-cli": "cli.mjs"
  },
  "scripts": {
    "build": "napi build --release",
    "prepublishOnly": "npm run build"
  }
}
```

Using @napi-rs/cli you can generate a template and a CI workflow that builds .node binaries for a matrix of Node versions/platforms and publishes them as a single npm package with prebuilds (no local compilation on install). ￼

### 2.3 Pros/cons vs “pure binary” approach

Pros:
 • Direct interop with Node:
 • You get both CLI and programmable API.
 • Can share in-process state with other JS tooling.
 • Prebuild tools (napi-rs, prebuildify) mean users usually don’t need Rust or a compiler.

Cons:
 • Requires Node runtime to execute the CLI.
 • You’re in prebuilt-addon land: Node ABI compatibility, Node version drift, etc.
 • Slightly more moving parts than “raw binary + wrapper”.

For a CLI-only tool where Node isn’t otherwise required, I’d still lean to pattern (1).

⸻

## 3. Pattern: Rust → WebAssembly (wasm-pack) + npm CLI wrapper

If your CLI is mostly pure computation (little to no filesystem, processes, or sockets), you can compile your Rust core to WASM and invoke it from Node.

Key tools:
 • wasm-bindgen + wasm-pack to compile Rust to WASM and generate JS glue. ￼

High-level flow:

 1. Write your core logic in Rust as a lib crate annotated with wasm-bindgen.
 2. Run wasm-pack build --target nodejs to generate pkg/ containing:
 • your_crate_bg.wasm
 • JS shim (ESM or CJS)
 • package.json metadata suitable for npm. ￼
 3. Publish that package to npm directly or wrap it in a top-level CLI package.

Example CLI wrapper (cli.mjs):

```js
# !/usr/bin/env node
import { run_cli } from "your-crate-wasm"; // the wasm-pack pkg

async function main() {
  const args = process.argv.slice(2);
  const exitCode = await run_cli(args); // implemented in Rust+WASM
  process.exit(exitCode ?? 0);
}

main().catch(err => {
  console.error(err);
  process.exit(1);
});
```

Pros:
 • Single artifact works in Node, browsers, and possibly Deno. ￼
 • No per-platform compiler issues; everything is WASM.

Cons:
 • Limited access to system APIs; CLIs that touch FS/process/env heavily don’t map cleanly.
 • Slight runtime overhead and streaming/IO quirks vs native.

It’s more attractive for “algorithm engine as a library that also happens to have a CLI”.

⸻

## 4. npm-side techniques and knobs that matter

Regardless of which Rust side you choose, there’s a standard set of npm mechanics you’ll touch:

### 4.1 bin

"bin" in package.json is what makes your CLI appear on $PATH. The value can be:
 • A single string (default command name equals package name).
 • A map of command names to paths.

```json
{
  "bin": {
    "your-cli": "lib/index.js",
    "yc": "lib/index.js"
  }
}
```

For the binary-wrapper pattern, lib/index.js runs Node and then spawns your Rust binary. In the binary-installer pattern, bin might point directly to a script that calls Binary.run() from binary-install/simple-binary-install. ￼

### 4.2 optionalDependencies + os/cpu

The platform-splitting trick relies on:
 • optionalDependencies in the base package listing per-platform packages.
 • os / cpu fields in the per-platform packages so npm only installs the matching one. ￼

This avoids any runtime “pick and download” logic; npm’s resolver handles it.

### 4.3 scripts.postinstall

If you’re using binary downloaders:
 • Add "postinstall": "node postinstall.mjs" to kick off a one-time download of the correct binary (usually using Binary.install() in those helper libs). ￼

### 4.4 Versioning & distribution tags

All flows play nicely with standard npm versioning. You can also:
 • Use distribution tags (npm dist-tag add your-cli@0.2.0 beta) to let users npm i your-cli@beta. ￼
 • Keep Rust crate version == npm package version for sanity, or decouple if you really must.


## 5. Putting it together: which combo when?

Given you’re targeting “Rust based CLI program in the npm ecosystem”, I’d think in terms of combinations:

A. “Tooling-style CLI, Node optional”

e.g. formatter, linter, transpiler that you want available via npx, but Rust is the source of truth.

 • Rust: just a normal bin crate.
 • Packaging stack:
 • Either:
 • cargo-dist with npm installer enabled (minimal npm glue). ￼
 • Or:
 • Orhun-style base+platform npm packages, with GitHub Actions building binaries. ￼
 • Pros: dead-simple runtime requirements, predictable performance, no Node ABI pain.

B. “CLI + rich JS API”

You want require('@your-scope/your-cli') to be a first-class Node library as well.

 • Rust: library exposed via napi-rs (#[napi]), optionally with a small bin crate for a non-Node distribution.
 • Packaging stack:
 • napi-rs CLI + CI templates to build .node prebuilds and publish to npm. ￼
 • Pros: single npm artifact gives you both CLI and programmatic API; nice for “tooling in the JS ecosystem”.

C. “Isomorphic library with a thin CLI”

Core logic is a library you also want in the browser; CLI is a bonus.

 • Rust: library compiled to WASM with wasm-pack.
 • Packaging stack:
 • wasm-pack builds the npm package.
 • Tiny JS CLI wrapper that imports the wasm package and delegates. ￼
