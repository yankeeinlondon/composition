# Composition

This monorepo has the following packages:

1. **Library** (`/lib`)
2. **CLI** (`/cli`)
3. **LSP** (`/lsp`)

## Functional Goal

To provide a Rust library, CLI, and LSP that can compose content together using a myriad of strategies which are enabled by a [custom DSL](./docs/features/darkmatter-dsl.md) which sits on top of standard [CommonMark](https://commonmark.org/) and [GFM](https://github.github.com/gfm/) based Markdown content.

## Modules

### Library Module

- the functional specs can be found in the [Darkmatter DSL](./docs/features/darkmatter-dsl.md) document.
- the expected library API surface is described in the [Library API Surface](./docs/library-api.md) document

### CLI

- the [Clap](./claude/skills/clap) crate will be used to structure the CLI application
- the [CLI User's Guide](./docs/cli-users-guide.md) provides an overview of the expected features for the CLI.


### LSP

- The [LSP Technical Strategy](./docs/design/lsp-technical-strategy.md) document lays out current thinking on the LSP technical strategy
- The [LSP Features](./docs/features/lsp-features.md) covers the desired features that this LSP will provide beyond the base Markdown features.

## Tech Stack

- For detailed tech stack specifics refer to the [Composition Tech Stack](./docs/reference/tech-stack.md) document.
