# Document Scope

Based on where the cursor is placed in the document you will end up in one of the following document scopes:

- `unscoped` - this is the most common scope and it indicates you are not in a special command scope. When you're in this scope, things like `@` file autocomplete will not be active.
- `audio` - this becomes active when the cursor resides in a command area such as:
    - `::audio ` at this point using the `@` character for autocomplete will suggest audio files which are in the [project scope](./project-scope.md) as well as any URLs which were found in the `audio_sources` frontmatter property.
- `video` - this becomes active when the cursor resides in a command area such as:
    - `::video ` at this point there are no local files which will show up but you will have URL's for YouTube and other popular video embedding services.
- `image` - this becomes active when you're in command areas such as:
    - `::image ` - a block level smart image
    - `![alt text](` - an inline Markdown image reference
- `file`
    - this becomes active when the cursor resides in a command area such as `::file `
    - when in this document scope, both document extensions and code-block extensions are deemed valid
        - If the file extension is a code-block file extension then the contents of the file will be brought in but placed inside a code block with the appropriate language setting set automatically.
        - If the file is a document extension then it is just brought in
- `document`
    - this becomes active when the cursor resides in a command area such as:
        - `::summarize `
- `document-list`
    - quite similar to the `document` scope but in this case there are one or more files which can be expressed as a comma-delimited list.
    - commands which invoke this scope include:
        - `::consolidate `
        - `::topic `
