# CLI Features

## Basics

- The CLI will be structured simply and not include any "sub commands" as we often find in more complicated CLI's.
- The executable will be called `compose`
- The general calling structure is `compose <file | glob> [...flags]`
- No flags are required

### Example: Markdown Document

A very simple example would be:

```sh
compose README.md
```

This would:

- take the `README.md` in the current directory as input
- parse the dependency tree of references using the library module
- output diagnostics to STDERR:

    ```txt
    - The 'README.md' document was processed (including X dependency nodes, Y were from cache)
        - 6 markdown documents (1 cached)
        - 2 images (all cached)
        - 1 summarization (cached)
    ```

- and output to STDOUT the completed/processed file

> **Note:** the output to STDOUT is **only** done when a single file is given as input. When more than one are given as input then it will save the results to the output directory. The output directory can be set with the `OUTPUT_DIR` environment variable -- the CLI will check for a `.env` -- or by using the `--output/-o` flag in the call.

### Example: Image File or URL

If a user were to type:

```sh
compose design-assets/logo.png
```

Then the following would take place:

- the source image would be passed to **smart image** for optimization
- STDERR would provide diagnostics:

    ```txt
    - The 'design-assets/logo.png' image was optimized (no transparency, blur image included, XX total images, YY were from cache) -> `jpg`, `avif`, `webp`
    ```

### Example: Process all Docs

If a user were to type:

```sh
compose content/**/*.md
```

Then the CLI would process all markdown files in the content directory and subdirectories. Diagnostics would be reported to STDERR while processing.

### Example: Ensure all Images are Cached

If a user were to type:

```sh
compose design-assets/**/*.{jpg,png,gif,avif,webp}
# or alternatively
compose design-assets/**/*.+(jpg|png|gif|avif|webp)
```

This will process all images under the `design-assets` and make sure the smart image cache is completely fresh.


## CLI Flags

1. Help (`--help`, `-h`) - displays a help menu for the CLI
2. Output (`--output`, `-o`) - override/specify the output directory
3. Inline Replacement (`--inline`)

    This will process the input file(s) but instead of writing the output to the output directory, it will instead _update_ the various input files with resolved references.

