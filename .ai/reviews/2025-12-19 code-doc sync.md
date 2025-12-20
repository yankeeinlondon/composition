# Code-Documentation Sync Report
Date: 2025-12-19
Documents Analyzed: 9/19 (10 interrupted by user)
Total Inconsistencies: 45 (Critical: 14, Major: 10, Minor: 8)

**Status:** Partial - User interrupted before completion

---

## docs/walking-document-tree.md

**Inconsistencies Found:** 3 (Critical: 1, Major: 1, Minor: 1)

### [CRITICAL] Broken documentation reference to caching strategy

**Location:** `docs/walking-document-tree.md:9`

**Issue:** References `./caching.md` which doesn't exist. Found `docs/design/cache-strategy.md` instead, but it's empty.

**Impact:** Users following the design document will encounter a broken link and have no access to the referenced caching strategy documentation.

**Suggested Fix:** Create proper caching documentation at `docs/caching.md` covering SurrealDB-based caching, content hash validation, cache invalidation rules, and performance characteristics.

### [MAJOR] Terminology mismatch - "master document" vs "root"

**Location:** `docs/walking-document-tree.md:5`

**Issue:** Document uses "master document" but implementation consistently uses "root" (DependencyGraph.root, build_graph(root), etc.)

**Impact:** Creates confusion for developers reading both design docs and code.

**Suggested Fix:** Update documentation to use "root" consistently to match implementation.


---

## docs/reference/hashing.md

**Inconsistencies Found:** 1 (Critical: 0, Major: 0, Minor: 1)

### [MINOR] Incorrect relative path to skill document

**Location:** `docs/reference/hashing.md:3`

**Issue:** Uses path `../.claude/skills/xx-hash/SKILL.md` but correct path from doc location should be `../../.claude/skills/xx-hash/SKILL.md`

**Impact:** Broken documentation link in some markdown renderers.

**Suggested Fix:** Use repo-root relative path: `.claude/skills/xx-hash/SKILL.md`

**Note:** Implementation is perfect - all code uses xxhash_rust::xxh3::xxh3_64 as documented. This is purely a documentation link issue.

---

## docs/reference/project-scope.md

**Inconsistencies Found:** 4 (Critical: 2, Major: 1, Minor: 1)

### [CRITICAL] Gitignore filtering not implemented

**Location:** `docs/reference/project-scope.md:10-12`

**Issue:** Documentation claims files excluded by `.gitignore` are removed from project scope, but implementation in `lib/src/graph/utils.rs` directly reads any local file without gitignore validation.

**Impact:** Security risk - sensitive files could be transcluded. Users can reference files that should be excluded (node_modules/, .env, etc.).

**Suggested Fix:** Implement gitignore filtering using the `ignore` crate before production use.

### [MINOR] Database path behavior for provided directory not documented

**Location:** `docs/reference/project-scope.md:13-15`

**Issue:** Implementation distinguishes between explicit `start_dir` provided (uses `start_dir/.composition.db`) vs no start_dir (uses `$HOME/.composition.db`), but documentation only describes the latter case.

**Impact:** Minor documentation ambiguity.

**Suggested Fix:** Update docs to match implementation's more flexible behavior.

---

## docs/design/lsp-technical-strategy.md

**Inconsistencies Found:** 8 (Critical: 1, Major: 5, Minor: 2)


### [MAJOR] Parser architecture differs from design

**Location:** `docs/design/lsp-technical-strategy.md:80-168`

**Issue:** Design describes three-phase parsing (Pre-processor → pulldown-cmark → AST) with source maps. Implementation uses simple line-by-line approach with no pre-processor or position tracking.

**Impact:** LSP features cannot provide accurate diagnostics without source maps.

**Suggested Fix:** Update design to acknowledge current library uses simplified parsing, and pre-processor is LSP-specific requirement.


### [MINOR] pulldown-cmark version mismatch

**Location:** `docs/design/lsp-technical-strategy.md:61`

**Issue:** Design specifies version "0.10" but implementation uses "0.13.0"

**Impact:** Minor version discrepancy.

**Suggested Fix:** Update design doc to reflect actual version (0.13).

### [MINOR] No workspace Cargo.toml

**Location:** Implied by monorepo structure

**Issue:** Design implies workspace structure but no workspace Cargo.toml exists at root.

**Impact:** Less efficient dependency resolution, slower builds.

**Suggested Fix:** Create workspace Cargo.toml for better monorepo management.

### [MINOR] Invalid cargo edition

**Location:** `cli/Cargo.toml:4`

**Issue:** CLI specifies `edition = "2024"` which, while valid in Rust 1.85.1, is very recent. Lib uses "2021".

**Impact:** Edition inconsistency across modules.

**Suggested Fix:** align both to 2024 for consistency.

---

## docs/features/lsp-features.md

**Inconsistencies Found:** 6 (Critical: 3, Major: 2, Minor: 1)


### [CRITICAL] Utility frontmatter variables not implemented

**Location:** `docs/features/lsp-features.md:18-21`

**Issue:** Documentation promises utility variables (today, yesterday, season, etc.) will "always be included", but `lib/src/render/interpolation.rs` only processes custom frontmatter variables, not built-in utilities.

**Impact:** Users cannot use `{{today}}` despite documentation. Core functionality is broken.

**Suggested Fix:** Implement utility variable generation in the library (add function to create HashMap with 14 utility variables).

### [CRITICAL] Broken cross-reference to interpolation section

**Location:** `docs/features/lsp-features.md:21`

**Issue:** References `darkmatter-dsl.md#11-frontmatter-interpolation` but this anchor doesn't exist. Should reference `utility-frontmatter.md`.

**Impact:** Broken documentation link.

**Suggested Fix:** Update link to point to correct document.


### [MINOR] Interpolation scope restriction unclear

**Location:** `docs/features/lsp-features.md:18`

**Issue:** States interpolation autocomplete triggers in `unscoped` document scope, but implementation processes `{{variable}}` everywhere without scope restrictions.

**Impact:** Minor confusion about whether this is autocomplete trigger restriction or interpolation validity restriction.

**Suggested Fix:** Clarify this is about autocomplete trigger, not where interpolation works.

---

## docs/reference/tech-stack.md

**Inconsistencies Found:** 7 (Critical: 2, Major: 3, Minor: 2)

### [CRITICAL] Wrong frontmatter parsing crate

**Location:** `docs/reference/tech-stack.md:15`

**Issue:** Documentation claims `markdown-frontmatter` crate but implementation uses `yaml-rust2` with custom parsing.

**Impact:** Misleading documentation directs developers to wrong crate.

**Suggested Fix:** The implementation SHOULD have used `markdown-frontmatter` as this is much more fit for purpose! Refactor this functionality using `markdown-frontmatter`


### [CRITICAL] OpenTelemetry not integrated

**Location:** `docs/reference/tech-stack.md:6`

**Issue:** Documentation states "tracing and open telemetry" but only basic `tracing` is implemented. No OpenTelemetry crates in dependencies.

**Impact:** Missing critical observability infrastructure if production telemetry is expected.

**Suggested Fix:** implement OpenTelemetry.

### [MAJOR] Criterion benchmarks not implemented

**Location:** `docs/reference/tech-stack.md:8`

**Issue:** Criterion is listed as dev-dependency but no benchmark files exist in `benches/` directory.

**Impact:** Unused dependency, no performance testing infrastructure.

**Suggested Fix:** implement benchmarks.

### [MAJOR] zune-image not used

**Location:** `docs/reference/tech-stack.md:17`

**Issue:** Documentation mentions "some combination of image, zune-image, and possibly other crates" but only `image` crate is used.

**Impact:** Minor documentation inaccuracy.

**Suggested Fix:** Update docs to reflect that only `image` crate is used.



### [MINOR] CLI edition "2024"

**Location:** `cli/Cargo.toml:4`

**Issue:** CLI uses `edition = "2024"` while lib uses "2021". This is technically valid (Rust 2024 edition exists) but worth noting for consistency.

**Impact:** Edition divergence between modules.

**Suggested Fix:** all modules should use "2024".

---

## docs/reference/database.md

**Inconsistencies Found:** 8 (Critical: 5, Major: 2, Minor: 1)

### [CRITICAL] Document map schema completely undefined

**Location:** `docs/reference/database.md:16-18`

**Issue:** States "TBD" but implementation has fully defined schema with document table (5 fields) and depends_on edge table (4 fields) using SurrealDB graph RELATE syntax.

**Impact:** Zero documentation for production schema. Developers have no idea what fields exist or how graph relationships work.

**Suggested Fix:** Document the actual schema from `lib/src/cache/schema.rs`.

### [CRITICAL] Image cache schema completely undefined

**Location:** `docs/reference/database.md:20-22`

**Issue:** States "TBD" but implementation has comprehensive schema with 10 fields including expiration, source type, transparency, dimensions.

**Impact:** No documentation about metadata storage, expiration behavior for remote images, or cache indexes.

**Suggested Fix:** Document the actual image_cache table schema.

### [CRITICAL] Missing LLM cache table

**Location:** Not mentioned in document

**Issue:** Documentation doesn't mention LLM caching, but implementation includes full `llm_cache` table with operation/input_hash/model composite key, expiration, and token tracking.

**Impact:** Documentation misrepresents database scope. Developers unaware of LLM result caching.

**Suggested Fix:** Add LLM Cache section with schema documentation.

### [CRITICAL] Missing vector embedding table

**Location:** Not mentioned in document

**Issue:** Documentation doesn't mention embeddings, but implementation includes `embedding` table with HNSW vector index support.

**Impact:** No visibility into semantic search capabilities.

**Suggested Fix:** Document embedding table and vector similarity features. Then review the new text added to the `docs/features/darkmatter-dsl.md` under the heading `### 16. Vector Embeddings` and make sure that that is implemented for now.

### [CRITICAL] Incorrect image cache description

**Location:** `docs/reference/database.md:8-14`

**Issue:** Describes image cache as "simple KV store" with only hash mapping, but implementation stores 10 fields with expiration, metadata, dual indexing.

**Impact:** Oversimplified description misleads about cache capabilities.

**Suggested Fix:** Update description to reflect actual complexity.

### [MAJOR] Database location logic not documented

**Location:** Not mentioned

**Issue:** Document mentions database paths but doesn't document the complex resolution logic (git detection, explicit dir handling, home fallback).

**Impact:** Users won't understand where database file is located.

**Suggested Fix:** Document `locate_database_path()` behavior with all three cases.

### [MAJOR] Backend technology not specified

**Location:** `docs/reference/database.md:3`

**Issue:** Says "SurrealDB" but doesn't specify RocksDB backend.

**Impact:** Developers don't know it's persisted to disk vs in-memory, or the performance characteristics.

**Suggested Fix:** Specify RocksDB backend and explain why it was chosen.

---

## docs/reference/breakpoints.md

**Inconsistencies Found:** 5 (Critical: 2, Major: 2, Minor: 1)

### [CRITICAL] Missing `xs` and `micro` image size variants

**Location:** `docs/reference/breakpoints.md:13-16`

**Issue:** Documentation describes `xs` (640px) and `micro` (320px) sized images, but implementation only generates 5 breakpoints (sm through 2xl) without xs/micro.

**Impact:** Mobile users will download oversized images. Smallest current variant is 640px.

**Suggested Fix:** Add micro (320px) and xs (640px) to BREAKPOINTS constant and update image processing.

### [CRITICAL] Image width calculation does not match documentation

**Location:** `docs/reference/breakpoints.md:13`

**Issue:** Documentation states images are "no larger than twice the breakpoint size" but implementation uses breakpoint values directly, not doubled.

**Impact:** Images are half the size they should be for retina/HiDPI displays. Quality degradation on modern screens.

**Suggested Fix:** Update implementation to double breakpoint values for retina support (as described in smart-image.md design doc).

### [MAJOR] Naming inconsistency: `2xl` vs `xxl` vs `Xxl`

**Location:** `docs/reference/breakpoints.md:9`

**Issue:** Documentation uses `2xl` (Tailwind convention) but code uses `Xxl` enum variant and `xxl` lowercase.

**Impact:** Confusion mapping docs to code. Violates Rust naming conventions.

**Suggested Fix:** Rename to `xxl` in code and update docs to reference it as "xxl".

### [MAJOR] Breakpoint documentation missing context

**Location:** `docs/reference/breakpoints.md:1-10`

**Issue:** Document lacks critical context about WHERE and HOW breakpoints are used (smart images, frontmatter, HTML generation, DSL types).

**Impact:** Developers don't know which systems respect breakpoint configurations or how to customize them.

**Suggested Fix:** Expand documentation with usage context, examples, and customization instructions.

### [MINOR] Pixel values missing unit consistency explanation

**Location:** `docs/reference/breakpoints.md:5-9`

**Issue:** Shows both pixel and rem values but doesn't explain relationship or which the system uses internally.

**Impact:** Unclear whether system uses px or rem. Missing explanation that 640px ÷ 16px/rem = 40rem.

**Suggested Fix:** Clarify that library uses pixels internally and rem values are for reference only.
