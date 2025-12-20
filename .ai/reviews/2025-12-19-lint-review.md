# Lint Review - 2025-12-19

## Fix Strategy Table

### Source Code Issues (lib/src)

| File:Line | Rule | Severity | Issue | Fix Strategy |
|-----------|------|----------|-------|--------------|
| `lib/src/render/transclusion.rs:136` | `unused_variables` | Warning | Unused parameter `cache` | Prefix with underscore: `_cache` (planned for future caching) |
| `lib/src/render/transclusion.rs:67` | `unused_variables` | Warning | Unused parameter `frontmatter` | Prefix with underscore: `_frontmatter` (planned for frontmatter interpolation) |
| `lib/src/render/interpolation.rs:35` | `unused_variables` | Warning | Unused error variable `e` | Prefix with underscore: `_e` |
| `lib/src/parse/resource.rs:51` | `dead_code` | Warning | Function `parse_resource_with_cache` never used | Remove function (caching handled elsewhere) |
| `lib/src/parse/resource.rs:65` | `dead_code` | Warning | Function `parse_duration` never used | Remove function (moved to cache module) |
| `lib/src/parse/frontmatter.rs:27-28` | `clippy::manual_strip` | Warning | Manually stripping prefix instead of using `strip_prefix` | Use `if let Some(<stripped>) = remaining.strip_prefix("---\n")` |
| `lib/src/parse/mod.rs:59-62` | `clippy::collapsible_match` | Warning | Nested if-let can be collapsed into outer match | Collapse pattern: `DarkMatterNode::Table { source: crate::types::TableSource::External(resource), .. }` |
| `lib/src/image/source.rs:15-21` | `clippy::should_implement_trait` | Warning | Method `from_str` should implement `FromStr` trait | Implement `FromStr` trait instead of custom method |
| `lib/src/image/metadata.rs:132-133` | `clippy::field_reassign_with_default` | Warning | Field assignment outside of Default initializer | Use struct initialization with defaults inline |
| `lib/src/image/metadata.rs:139-140` | `clippy::field_reassign_with_default` | Warning | Field assignment outside of Default initializer | Use struct initialization with defaults inline |

### Test Code Issues (lib/tests)

| File:Line | Rule | Severity | Issue | Fix Strategy |
|-----------|------|----------|-------|--------------|
| `lib/tests/integration_phase1.rs:4` | `unused_imports` | Warning | Unused import `common::*` | Remove wildcard import, add specific imports for fixtures and helpers |
| `lib/tests/integration_phase1.rs:50` | `E0425` | **Error** | Cannot find function `init_test_db` | Add import: `use common::helpers::init_test_db;` |
| `lib/tests/integration_phase1.rs:82` | `E0425` | **Error** | Cannot find function `temp_dir` | Add import: `use common::fixtures::temp_dir;` |
| `lib/tests/integration_phase1.rs:96` | `E0425` | **Error** | Cannot find function `test_frontmatter` | Add import: `use common::fixtures::test_frontmatter;` |
| `lib/tests/integration_phase1.rs:125` | `E0425` | **Error** | Cannot find function `init_test_db` | Already covered by import above |
| `lib/tests/integration_phase1.rs:160` | `E0425` | **Error** | Cannot find function `init_test_db` | Already covered by import above |
| `lib/tests/integration_phase1.rs:196` | `E0425` | **Error** | Cannot find function `init_test_db` | Already covered by import above |
| `lib/tests/integration_phase1.rs:228` | `E0425` | **Error** | Cannot find function `init_test_db` | Already covered by import above |
| `lib/tests/integration_phase1.rs:267` | `E0425` | **Error** | Cannot find function `temp_dir` | Already covered by import above |
| `lib/tests/integration_phase1.rs:301` | `E0425` | **Error** | Cannot find function `test_local_resource` | Add import: `use common::fixtures::test_local_resource;` |
| `lib/tests/integration_phase1.rs:314` | `E0425` | **Error** | Cannot find function `compute_test_hash` | Add import: `use common::helpers::compute_test_hash;` |
| `lib/tests/integration_phase1.rs:315` | `E0425` | **Error** | Cannot find function `compute_test_hash` | Already covered by import above |
| `lib/tests/integration_phase1.rs:316` | `E0425` | **Error** | Cannot find function `compute_test_hash` | Already covered by import above |
| `lib/tests/e2e_workflow.rs:217` | `unused_variables` | Warning | Unused variable `resource` | Prefix with underscore: `_resource` |
| `lib/tests/e2e_workflow.rs:322` | `unused_variables` | Warning | Unused variable `i` in enumerate | Prefix with underscore: `_i` |
| `lib/tests/spike_async_sync.rs:174` | `unused_variables` | Warning | Unused variable `results` | Prefix with underscore: `_results` |
| `lib/tests/common/fixtures.rs:6` | `dead_code` | Warning | Function `temp_dir` never used | Do nothing (used by integration_phase1 after import fix) |
| `lib/tests/common/fixtures.rs:11` | `dead_code` | Warning | Function `test_local_resource` never used | Do nothing (used by integration_phase1 after import fix) |
| `lib/tests/common/fixtures.rs:16` | `dead_code` | Warning | Function `test_remote_resource` never used | Do nothing (will be used in future tests) |
| `lib/tests/common/fixtures.rs:22` | `dead_code` | Warning | Function `test_frontmatter` never used | Do nothing (used by integration_phase1 after import fix) |
| `lib/tests/common/fixtures.rs:30` | `dead_code` | Warning | Function `create_test_markdown_file` never used | Do nothing (will be used in future tests) |
| `lib/tests/common/helpers.rs:8` | `dead_code` | Warning | Static `TEST_DB_COUNTER` never used | Do nothing (used by init_test_db) |
| `lib/tests/common/helpers.rs:11` | `dead_code` | Warning | Function `init_test_db` never used | Do nothing (used by integration_phase1 after import fix) |
| `lib/tests/common/helpers.rs:20` | `dead_code` | Warning | Function `compute_test_hash` never used | Do nothing (used by integration_phase1 after import fix) |

## Summary

- **Errors Found**: 15 (all in test files)
- **Warnings Found**: 23 total
  - Source code: 10 warnings
  - Test code: 13 warnings
- **Warnings marked "do nothing"**: 9
  - 8 dead_code warnings in test helpers/fixtures (will be resolved after import fixes)
  - 1 dead_code warning for TEST_DB_COUNTER (used internally)
- **Auto-Fixed Items**: None (clippy --fix found no auto-fixable issues)

## Implementation Order

1. Fix test imports (resolves all 15 errors + 8 dead_code warnings)
2. Fix clippy warnings in source code (8 warnings)
3. Fix unused variables in tests (3 warnings)
4. Review remaining dead_code warnings in test fixtures (2 warnings - keep for future use)
