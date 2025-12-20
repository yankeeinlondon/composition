# Composition Database

We use [SurrealDB](../../.claude/skills/surrealdb/SKILL.md) with a RocksDB backend for storing all state which this library depends on. The database provides graph capabilities for dependency tracking and efficient caching for images, LLM operations, audio, and embeddings.

## Backend Technology

**Database:** SurrealDB
**Backend:** RocksDB (persisted to disk)
**Namespace:** `composition`
**Database:** `composition`

### Why RocksDB?

- **Persistent storage** - Survives process restarts, ensuring cache durability
- **High performance** - Optimized for read-heavy workloads typical in document composition
- **Efficient key-value lookups** - Fast cache hit/miss operations
- **Transaction support** - Maintains graph consistency during updates
- **Embedded operation** - No separate server process required

## Database Location Resolution

The database file location is determined by `locate_database_path()` using the following logic:

1. **Git repository detected:** `{repo_root}/.composition.db`
   - Searches upward from the start directory for a `.git` folder
   - Example: `/path/to/my-project/.composition.db`

2. **Explicit start_dir provided (no git repo):** `{start_dir}/.composition.db`
   - Useful for tests with temporary directories
   - Example: `/tmp/test-123/.composition.db`

3. **Fallback (no start_dir, no git):** `$HOME/.composition.db`
   - Global cache for non-git projects
   - Example: `/Users/ken/.composition.db`

This strategy ensures project-scoped caching for git repositories while providing a sensible fallback for standalone documents.

## Schema Overview

The database consists of 6 tables organized into two functional areas:

### Document Graph (2 tables)
- `document` - Graph nodes representing markdown documents
- `depends_on` - Graph edges representing document dependencies

### Cache Tables (4 tables)
- `image_cache` - Cached image metadata and source tracking
- `llm_cache` - Cached LLM operation results (summarize, consolidate)
- `embedding` - Vector embeddings for semantic search
- `audio_cache` - Audio file metadata and format information

## Entity-Relationship Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                      Document Graph                              │
└─────────────────────────────────────────────────────────────────┘

    ┌──────────────────┐
    │    document      │
    │──────────────────│
    │ resource_hash PK │◄────┐
    │ content_hash     │     │
    │ file_path        │     │ in
    │ url              │     │
    │ last_validated   │     │
    └──────────────────┘     │
            ▲                │
            │ out            │
            │                │
    ┌───────┴──────────┐     │
    │   depends_on     │─────┘
    │──────────────────│
    │ in → document    │ (EDGE TABLE)
    │ out → document   │
    │ reference_type   │ (transclusion/summarize/consolidate)
    │ required         │ (bool: error if missing vs silent skip)
    └──────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                      Cache Tables                                │
└─────────────────────────────────────────────────────────────────┘

┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐
│  image_cache     │  │   llm_cache      │  │  audio_cache     │
│──────────────────│  │──────────────────│  │──────────────────│
│ resource_hash PK │  │ operation        │  │ resource_hash PK │
│ content_hash     │  │ input_hash       │  │ content_hash     │
│ created_at       │  │ model            │  │ created_at       │
│ expires_at       │  │ response         │  │ source_type      │
│ source_type      │  │ created_at       │  │ source           │
│ source           │  │ expires_at       │  │ format           │
│ has_transparency │  │ tokens_used      │  │ duration_secs    │
│ original_width   │  │                  │  │ bitrate          │
│ original_height  │  │ Composite Index: │  │ sample_rate      │
│                  │  │ (operation,      │  │ channels         │
│ Indexes:         │  │  input_hash,     │  │                  │
│ - resource_hash  │  │  model)          │  │ Indexes:         │
│ - (resource_hash,│  │ - expires_at     │  │ - resource_hash  │
│    content_hash) │  │                  │  │ - (resource_hash,│
└──────────────────┘  └──────────────────┘  │    content_hash) │
                                            └──────────────────┘
┌──────────────────┐
│   embedding      │
│──────────────────│
│ resource_hash PK │
│ content_hash     │
│ model            │
│ vector           │ (array<float>)
│ created_at       │
│                  │
│ Indexes:         │
│ - resource_hash  │
│ (HNSW planned)   │
└──────────────────┘
```

## Table Schemas

### `document` Table

**Purpose:** Graph nodes representing markdown documents being composed. Each document is identified by a resource hash (derived from file path or URL) and tracks its content hash for cache invalidation.

| Field | Type | Description |
|-------|------|-------------|
| `resource_hash` | `string` | xxHash of resource identifier (file path or URL). Primary key. |
| `content_hash` | `string` | xxHash of actual document content. Changes trigger cache invalidation. |
| `file_path` | `option<string>` | Absolute path for local files. Mutually exclusive with `url`. |
| `url` | `option<string>` | HTTP/HTTPS URL for remote resources. Mutually exclusive with `file_path`. |
| `last_validated` | `datetime` | Last time cache entry was validated. Used for expiration policies. |

**Indexes:**
- `idx_resource_hash` - UNIQUE on `resource_hash` for fast lookups by file/URL

**Graph Relations:**
- Connected to other `document` nodes via `depends_on` edges using SurrealDB's `RELATE` syntax
- Forms a directed acyclic graph (DAG) of document dependencies

**Usage Pattern:**
```rust
// Get document by resource hash
SELECT * FROM document WHERE resource_hash = $hash

// Find all dependencies of a document
SELECT ->depends_on->document FROM document WHERE resource_hash = $hash
```

---

### `depends_on` Table (Edge Table)

**Purpose:** Graph edges representing document dependencies. When document A transcludes/summarizes/consolidates document B, a `depends_on` edge is created from A to B. Enables cascade invalidation when source documents change.

| Field | Type | Description |
|-------|------|-------------|
| `in` | `record<document>` | Source document (the document that depends on something). |
| `out` | `record<document>` | Target document (the document being depended upon). |
| `reference_type` | `string` | Type of reference: `"transclusion"`, `"summarize"`, `"consolidate"`, `"table"`, `"chart"` |
| `required` | `bool` | If `true`, error if target is missing (`!` suffix). If `false`, silently skip if missing (`?` suffix). Default: `false` |

**Indexes:** None (relies on SurrealDB's built-in graph traversal optimization)

**Graph Semantics:**
- Edge direction: `source_document -[depends_on]-> target_document`
- Enables graph traversal queries like "find all documents that depend on X" (cascade invalidation)
- Prevents circular dependencies (validation enforced at application layer)

**Usage Pattern:**
```rust
// Create dependency edge
RELATE (document:abc123)->depends_on->(document:def456)
  SET reference_type = "transclusion", required = true

// Find all documents that depend on a specific document (cascade invalidation)
SELECT <-depends_on<-document FROM document WHERE resource_hash = $hash

// Find all dependencies of a document
SELECT ->depends_on->document FROM document WHERE resource_hash = $hash
```

---

### `image_cache` Table

**Purpose:** Caches image metadata and source information. Stores original dimensions, transparency detection, and source tracking to avoid reprocessing images. The actual image files are stored on disk; this table provides the metadata index.

| Field | Type | Description |
|-------|------|-------------|
| `resource_hash` | `string` | xxHash of resource identifier (image path or URL). Primary key. |
| `content_hash` | `string` | xxHash of image binary content. Used to detect if image has changed. |
| `created_at` | `datetime` | When cache entry was created. Default: `time::now()` |
| `expires_at` | `option<datetime>` | When cache entry expires. `None` for local files (no expiration), set for remote URLs. |
| `source_type` | `string` | Source type: `"local"` (file), `"http"` (remote URL) |
| `source` | `string` | Original source (file path or URL) |
| `has_transparency` | `bool` | Whether image has alpha channel. Determines WebP vs JPEG encoding. |
| `original_width` | `int` | Original image width in pixels |
| `original_height` | `int` | Original image height in pixels |

**Indexes:**
- `idx_image_resource` - UNIQUE on `resource_hash` for fast lookups
- `idx_image_lookup` - Composite on `(resource_hash, content_hash)` for cache validation

**Cache Strategy:**
- **Local files:** No expiration (`expires_at = None`). Invalidate on content_hash change.
- **Remote URLs:** 1-day expiration (configurable per-reference). Re-fetch after expiration.

**Usage Pattern:**
```rust
// Check if image is cached
SELECT * FROM image_cache WHERE resource_hash = $hash

// Validate cache is fresh
SELECT * FROM image_cache
WHERE resource_hash = $hash AND content_hash = $current_hash

// Clean expired remote images
DELETE FROM image_cache WHERE expires_at < $now AND source_type = "http"
```

---

### `llm_cache` Table

**Purpose:** Caches LLM operation results (summarize, consolidate, etc.) to avoid redundant API calls. Results are keyed by operation type, input content hash, and model name.

| Field | Type | Description |
|-------|------|-------------|
| `operation` | `string` | Operation type: `"summarize"`, `"consolidate"`, `"extract"` |
| `input_hash` | `string` | xxHash of input content. Ensures cache hits only on identical input. |
| `model` | `string` | LLM model name (e.g., `"claude-sonnet-4"`, `"gpt-4"`). Different models = different cache entries. |
| `response` | `string` | LLM response text. Stored for reuse. |
| `created_at` | `datetime` | When cache entry was created. Default: `time::now()` |
| `expires_at` | `datetime` | When cache entry expires. Required field (no indefinite caching). |
| `tokens_used` | `option<int>` | Number of tokens consumed by operation (for usage tracking). |

**Indexes:**
- `idx_llm_lookup` - Composite on `(operation, input_hash, model)` for fast cache hits
- `idx_llm_expires` - On `expires_at` for efficient cleanup queries

**Cache Strategy:**
- **Default expiration:** 30 days from creation
- **Cache key:** Combination of operation type + input content + model ensures correct cache hits
- **Cleanup:** Periodic deletion of expired entries via `clean_expired_llm_cache()`

**Usage Pattern:**
```rust
// Check for cached LLM result
SELECT * FROM llm_cache
WHERE operation = $op
  AND input_hash = $hash
  AND model = $model
  AND expires_at > $now

// Clean expired entries
DELETE FROM llm_cache WHERE expires_at < $now RETURN BEFORE
```

---

### `embedding` Table

**Purpose:** Stores vector embeddings for semantic search and content similarity. Each document can have multiple embeddings (different models). HNSW index planned for Phase 6 (requires SurrealDB 2.x).

| Field | Type | Description |
|-------|------|-------------|
| `resource_hash` | `string` | xxHash of resource identifier. Links to `document` table. |
| `content_hash` | `string` | xxHash of content that was embedded. Invalidate on change. |
| `model` | `string` | Embedding model name (e.g., `"text-embedding-ada-002"`, `"voyage-large-2"`). |
| `vector` | `array<float>` | Embedding vector (typically 768 or 1536 dimensions depending on model). |
| `created_at` | `datetime` | When embedding was generated. Default: `time::now()` |

**Indexes:**
- `idx_embedding_resource` - UNIQUE on `resource_hash` for fast lookups
- **Planned:** HNSW vector index for similarity search (requires SurrealDB 2.x)

**Future Enhancement:**
```sql
-- Planned for Phase 6 (SurrealDB 2.x)
DEFINE INDEX idx_embedding_vector ON embedding
FIELDS vector HNSW DIMENSION 1536 DISTANCE COSINE;
```

**Usage Pattern:**
```rust
// Get embedding for document
SELECT * FROM embedding WHERE resource_hash = $hash

// Semantic similarity search (future with HNSW)
SELECT *, vector::similarity::cosine(vector, $query_vector) AS score
FROM embedding
ORDER BY score DESC
LIMIT 10
```

---

### `audio_cache` Table

**Purpose:** Caches audio file metadata (format, duration, bitrate, etc.) to avoid reprocessing. Used for audio player embedding and format detection.

| Field | Type | Description |
|-------|------|-------------|
| `resource_hash` | `string` | xxHash of resource identifier (audio path or URL). Primary key. |
| `content_hash` | `string` | xxHash of audio file binary content. Used to detect changes. |
| `created_at` | `datetime` | When cache entry was created. Default: `time::now()` |
| `source_type` | `string` | Source type: `"local"` (file), `"http"` (remote URL) |
| `source` | `string` | Original source (file path or URL) |
| `format` | `string` | Audio format: `"mp3"`, `"m4a"`, `"wav"`, `"ogg"`, `"flac"` |
| `duration_secs` | `option<float>` | Audio duration in seconds (if available from metadata) |
| `bitrate` | `option<int>` | Audio bitrate in kbps (if available) |
| `sample_rate` | `option<int>` | Sample rate in Hz (e.g., 44100, 48000) |
| `channels` | `option<int>` | Number of audio channels (1 = mono, 2 = stereo) |

**Indexes:**
- `idx_audio_resource` - UNIQUE on `resource_hash` for fast lookups
- `idx_audio_lookup` - Composite on `(resource_hash, content_hash)` for cache validation

**Cache Strategy:**
- Same as `image_cache`: local files never expire, remote URLs expire after 1 day

**Usage Pattern:**
```rust
// Check if audio metadata is cached
SELECT * FROM audio_cache WHERE resource_hash = $hash

// Get audio format and duration for player embedding
SELECT format, duration_secs, bitrate
FROM audio_cache
WHERE resource_hash = $hash
```

---

## Query Performance Notes

### Optimized Queries

**Document dependency traversal** (cascade invalidation):
```sql
-- Efficient graph traversal using indexes
SELECT ->depends_on->document AS dependents
FROM document
WHERE resource_hash = $hash
```
- **Performance:** O(1) lookup via `idx_resource_hash`, then graph edge traversal (optimized by SurrealDB)
- **Use case:** Invalidate all documents that transitively depend on a changed source

**LLM cache hit check**:
```sql
SELECT * FROM llm_cache
WHERE operation = $op
  AND input_hash = $hash
  AND model = $model
  AND expires_at > $now
```
- **Performance:** O(1) lookup via composite `idx_llm_lookup`, then filter on expires_at (indexed)
- **Use case:** Avoid redundant LLM API calls

**Image/Audio cache validation**:
```sql
SELECT * FROM image_cache
WHERE resource_hash = $hash AND content_hash = $current_hash
```
- **Performance:** O(1) lookup via composite `idx_image_lookup`
- **Use case:** Fast cache hit/miss determination

### Slow Queries to Avoid

**Full table scan without index**:
```sql
-- BAD: No index on 'url' field
SELECT * FROM document WHERE url LIKE '%example.com%'
```
- **Issue:** Full table scan, no index to optimize LIKE queries
- **Solution:** Use exact equality on indexed fields, or add full-text index if needed

**Unbounded graph traversal**:
```sql
-- BAD: No depth limit on recursive traversal
SELECT ->depends_on->document->depends_on->document->depends_on->document
FROM document
WHERE resource_hash = $hash
```
- **Issue:** Can explode in time/memory for deeply nested graphs
- **Solution:** Limit traversal depth or use iterative approach with depth tracking

---

## Schema Evolution Strategy

### Current Version: 1.0

The schema is currently at version 1.0 (initial release). Future schema changes will follow this migration strategy:

1. **Additive changes** (new tables, new optional fields):
   - Apply via schema migration scripts
   - No data migration needed

2. **Breaking changes** (field renames, type changes, deletions):
   - Create migration script in `lib/src/cache/migrations/`
   - Version the schema: `v1_to_v2.sql`
   - Store version in metadata table: `DEFINE TABLE schema_version`

3. **Index additions**:
   - Apply immediately (indexes are additive)
   - Monitor query performance before/after

### Planned Changes (Future Versions)

- **v1.1:** Add HNSW vector index for `embedding` table (requires SurrealDB 2.x)
- **v1.2:** Add `schema_version` metadata table for migration tracking
- **v2.0:** Potential multi-namespace support for isolated projects

---

## Development Notes

### Testing

All cache operations are tested via:
- **Unit tests:** `lib/tests/unit/cache/*.rs` (test each table's CRUD operations)
- **Integration tests:** `lib/tests/integration/cache_*.rs` (test graph traversal, cascade invalidation)

### Debugging

Enable SurrealDB query logging:
```rust
// In lib/src/cache/database.rs
use tracing::Level;
tracing::subscriber::set_global_default(
    tracing_subscriber::fmt().with_max_level(Level::DEBUG).finish()
).unwrap();
```

View all queries executed:
```bash
RUST_LOG=surrealdb=debug cargo test
```

### Schema Application

Schema is applied automatically on database initialization:
```rust
// In lib/src/cache/schema.rs
pub async fn apply_schema(db: &Surreal<Db>) -> Result<()> {
    db.query(SCHEMA_SQL).await?;
    Ok(())
}
```

To manually verify schema:
```bash
# Connect to database with surreal CLI
surreal start --log debug file://.composition.db

# In another terminal
surreal sql --endpoint http://localhost:8000 --namespace composition --database composition

# Run queries
INFO FOR TABLE document;
INFO FOR TABLE depends_on;
```
