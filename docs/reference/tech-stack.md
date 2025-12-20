# Tech Stack

## All Modules

- will use [thiserror](../../.claude/skills/thiserror/SKILL.md) for error definition and handling
- will use [tracing](../../.claude/skills/rust-logging/opentelemetry.md) for logging (OpenTelemetry integration planned but not yet implemented)
- will use **cargo test** for unit testing
- will use [criterion](../../.claude/skills/rust-testing/SKILL.md) for performance testing
- will use [xxhash-rust](../../.claude/skills/xx-hash/rust.md) for hashing (see [hashing](./hashing.md))


## Library Module

- we will use [pulldown-cmark](../../.claude/skills/pulldown-cmark/SKILL.md) for parsing markdown files and replacing _references_ with resolved content.
- we will use [yaml-rust2](https://crates.io/crates/yaml-rust2) for parsing out the frontmatter properties
- Smart Image
    - we will use the [image](https://crates.io/crates/image) crate for image optimization (resizing, converting formats, metadata)
    - we wll combine that with [rayon](../../.claude/skills/rayon/SKILL.md) create to concurrently process multiple images at a time.

> **Note:** the library will a [SurrealDB](./database.md) for all state management.

## CLI

- we will use [Clap](../../.claude/skills/clap/SKILL.md) as the primary framework for building the CLI
- we will use either [napi-rs](../../.claude/skills/rust-on-npm/napi-rs.md) or [Neon](../../.claude/skills/rust-on-npm/neon.md) to package the CLI up and publish to **npm**

## LSP

TODO
