# Composition

A monorepo which contains the following modules:

- **Composition Library**
- **Darkmatter LSP Server**
- **CLI**

Each of these modules will be discussed in the following sections. In addition to these modules directories, we also have a `./docs` directory intended to store valuable reusable knowledge bases on related technologies that will be used in this project.

## Module Details

### Composition Library

The composition library provides primitives to _compose_ documents together. The root document is always expected to be a Markdown document but compositional elements of that root document can be sourced from:

- images
- code files
- other markdown documents
- website summaries
- PDF's
- results of a web search
- code diagnostics report

In addition to the various inputs allowed for composition:

- the composition of various things can be made _conditional_.
- The final output of this composed content can be Markdown _or_ HTML.

#### Tech Stack

- we use the [markdown-rs](https://github.com/wooorm/markdown-rs) crate to do Markdown parsing and convert to HTML when requested
- we use XXX for _frontmatter_ metadata parsing
- we use YYY for code block highlighting

### Darkmatter LSP Server

The root document is always a Markdown-_ish_ file and in case of writing prose this root document is perfectly served by any existing Markdown LSP. However, this library accepts and encourages use a set of _language primitives_ for composing content that a normal Markdown LSP would not be able to help with.

To address this we're coining a new file format called **Darkmatter** with a file extension of `.dm` (e.g., `.md` inverted). The LSP for **Darkmatter** is a _super set_ of a Markdown parser which is aware of the extensions and will 

#### Tech Stack

- we use YYY as a LSP server framework
- we use XXX as a baseline Markdown LSP
