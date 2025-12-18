# Composition Project

This monorepo has the following packages:

1. **Library** (`/lib`)
2. **CLI** (`/cli`)
3. **LSP** (`/lsp`)

## Library Module

Exposes the following functions to any calling application code:

1. `init(dir)`

   - determines where the SurrealDB database file should be located for files under the passed in `dir`
     - if the `dir` passed in is part of a **git** repo then the database will be stored in the repo's root as `_composition-cache.db`
     - if not then we look for the database in `${HOME}/_composition-cache.db`
   - if SurrealDB file already exists then exit on success, if not then create the database before exiting

2. `graph(filepath)`

   - determines the dependency graph for the passed in file

3. `validate(filepath)`

   - evaluates the dependency graph for a given Markdown/Darkmatter file.

4. `resolve(filepath, [state])`

   - This utility takes in a Markdown-like document (e.g., [DarkMatter DSL](./docs/darkmatter-dsl.md) ) and resolves it's tree (DAG) of reference dependencies to produce an output Markdown document.
   - It uses the [pulldown-cmark](https://github.com/pulldown-cmark/pulldown-cmark) crate to parse the document identified by the `filepath`:
     - calls `validate()` to ensure the dependency tree is not circular
     - resolves the DSL nodes in the input content to the appropriate content (see [Walking the Document Tree](./walking-document-tree.md))
     - resolving references will use cached results whenever the cache has a 

5. `parse(glob[], [state])`

   - This function acts as an orchestrator for all of the files which are in scope
   - It takes one or more glob patterns and returns a list of all files with the `.md` or `.dm` file extension
   - This list of files represents the "master documents" and this function then iterates over the documents, using `resolve()` to produce the finalized document structure

6. `toHTML(glob)`

   - converts a markdown file (or set of files) to HTML (with inline CSS and inline images)
   - there is a 1:1 relationship between Markdown file and HTML file
   - the HTML file is intended to be fully self-contained; meaning that the HTML file can be shared and all resources to run the page will be included.
   - TODO: To address links to other local files, we will need to investigate whether we use some sort of web-archive format, remove the links, or do something else to address this.

## CLI

Provides the command `compose` to the terminal with very simple interface:

SYNTAX: `compose <input-file | glob> --out <dir>`
WHERE:

- the `--out` switch is optional and if not included the output file or files will be saved to a `./temp` directory off of the current working directory
- the `--html` switch can be used to convert the output to HTML instead of Markdown
