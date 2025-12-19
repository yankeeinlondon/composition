# Composition Database

We will use the [SurrealDB](../.claude/skills/) for storing all state which this library depends on.

The main areas which will need to be models are:

- **Document Map** - leveraging **SurrealDB**'s graph capabilities we'll map out the various tree-like dependencies for the documents [in scope](./doc-scope.md).
- **Image Cache** - a simple KV store where:
    - the **keys** are _resource hashes_
        - a _resource hash_ is a hash of the unique file or URL identifier
        - this _resource hash_ is also the base filename for the image files from this source
    - the **values** are a *content hash*, where:
        - the content hash represents the image's state
        - and is used to as way to validate/invalidate a cache entry

## Document Map Schema

TBD

## Image Cache Schema

TBD
