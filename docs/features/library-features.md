# Library API

The library module in this monorepo is meant to support all of the features required by the [Darkmatter DSL](./darkmatter-dsl.md). This document describe the API surface that is exposed.

## Initialization

Using the library module starts by calling the `init(dir?, frontmatter?)` function:

- if no directory is passed in then the _current working directory_ will be used instead
- calling `init()` uses the [project scope](../reference/project-scope.md) to determine where the database should be located
- if there is no database yet defined then it is responsible for creating it
- the return type of the `init()` function is the **Composition API** along with the initial `frontmatter`
- the frontmatter will be composed of:
    - any recognized ENV variables
    - all [utility frontmatter](../reference/utility-frontmatter.md) key/values
    - merged with any key/values passed into `init()`

## Composition API


### Core API

Exposes the following core functions:

> **Note:** the term `resources` is meant to be any valid resource identifier. This includes local files but also references to external URL's.


1. `graph(resource)`

   - determines the dependency graph for the passed in file
       - only Darkmatter documents (with references) have dependencies, however any asset can be _depended upon_
       - Note: it is possible that a Darkmatter resource will be returned by a URL reference instead of a local file reference (this is not common but it is possible)
   - saves this dependency graph to the database if any changes in dependencies were encountered in the file versus the DB

2. `generateWorkplan(resources[])`

    - takes in the set of resources which has been requested for parsing and by identifying the dependency graphs of these files is able to return grouped parsing plan
    - this plan immediately filters out the fresh cached resources
    - it then groups the resources that are not cached into layers
    - the layers are organized to allow each layer to be run concurrently and with an attempt to put nodes which are highly depended upon higher in the stack so they are  higher in the stack

3. `render(resources[], [state])`

    - This function acts as an orchestrator for all of the resources which were passed in
    - It will generate a workplan and then execute that plan with as much concurrency as possible (leveraging the [rayon](../../.claude/skills/rayon/SKILL.md) crate when possible)
    - The `state` passed in is a dictionary which will be the starting frontmatter for markdown documents.
    - How rendering is accomplished will depend on the type of asset which is being processed. This function will rely on utility functions described in the next section.

4. `toHTML(glob[])`

    - converts a markdown file (or set of files) to HTML (with inline CSS and inline images)
    - there is a 1:1 relationship between Markdown file and HTML file
    - the HTML file is intended to be fully self-contained; meaning that the HTML file can be shared and all resources to run the page will be included.
        - TODO: To address links to other local files, we will need to investigate whether we use some sort of web-archive format, remove the links, or do something else to address this.

### Supplemental API

The following functions will also be provided back when calling `init()`. They are separated from the **Core API** because they are somewhat lower level and many callers of the library may only interact with this **core** layer.

1. `transclude(file | url)`

    - expects to receive a _resource_ which points directly or indirectly to Markdown or Darkmatter content
    - if the content has DarkMatter references in it then it will recursively render these using the `render()` function
    - once the content is fully resolved it will make sure a file is saved to the output directory
        - for all images, local or external, the images will be saved to the image cache at `${output_dir}/images/${hash}.${ext}`
        - for url based inputs all files will be saved as `${output_dir}/external/${hash}.${ext}`
        - for file based document inputs (`.md`,`.pdf`,`.txt`) this file will reside in the output directory with a filepath offset which mimics the input file (e.g., input file `content/one/info.md` will be saved to `${output_dir}/content/one/info.md`)
            - this is a bit different then the other flatter file patterns but it allows markdown files to maintain relative links and the source directory structure is often providing an contextual structure to developers and consumers of the output files.

    **Note:** this is where we expect the primary use of the [pulldown-cmark](../../.claude/skills/pulldown-cmark/SKILL.md) crate to be.

2. `optimizeImage(file|url)`

    - expects a valid image file or URL reference
    - ensures that the optimized images for this resource are fresh in `${output_dir}/images`

3. `summarize(resource, &frontmatter)` ✅ IMPLEMENTED (Phase 6)

    - Leverages AI to summarize content using async LLM providers
    - Results are cached in SurrealDB with 30-day expiration
    - The frontmatter reference gives access to the `summarize_model` property
    - Uses custom `CompletionModel` trait for provider flexibility
    - **Implementation:** `lib/src/ai/summarize.rs`

4. `consolidate(resource[], &frontmatter)` ✅ IMPLEMENTED (Phase 6)

    - Leverages AI to consolidate multiple document-based resources into a comprehensive whole
    - Uses the frontmatter's `consolidate_model` to choose an [AI model](../reference/models.md)
    - Results cached by combined hash of all input documents
    - Order-sensitive caching (different order = different result)
    - **Implementation:** `lib/src/ai/consolidate.rs`

5. `topicExtraction(topic, resources[], review)` ✅ IMPLEMENTED (Phase 6)

    - Extracts content related to a specific topic from multiple documents
    - Optional review mode provides analysis of extracted content
    - Results cached by topic + review flag + input documents
    - **Implementation:** `lib/src/ai/topic.rs`

6. `generateEmbedding(resource, text)` ✅ IMPLEMENTED (Phase 6)

    - Generates vector embeddings for text content using embedding models
    - Stores embeddings in SurrealDB with HNSW index support
    - Supports similarity search via `findSimilar()`
    - **Implementation:** `lib/src/ai/embedding.rs`
