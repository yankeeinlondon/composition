---
description: Create a detailed multi-phase plan with sub-agent ownership and parallel reviews for Rust development
---

# Multi-Phase Planning with Sub-Agent Review (Rust Edition)

You have been asked to create a comprehensive implementation plan for the Composition project. This command orchestrates a sophisticated planning workflow that leverages specialized sub-agents for domain expertise and parallel review.

**IMPORTANT:** Use the TodoWrite tool to track your progress through these steps.

## Overview

This planning workflow:

1. Gathers requirements and analyzes the task
2. Creates a detailed plan with phases and assigns principal owners
3. Launches parallel reviews by domain specialists
4. Consolidates feedback and identifies parallelization opportunities
5. Produces a final, implementation-ready plan

## Available Sub-Agents (Principal Owners)

| Sub-Agent | Domain | Assign When |
|-----------|--------|-------------|
| **Rust Developer** | Core library, CLI, LSP, performance-critical code | Systems programming, pulldown-cmark integration, trait design |
| **Database Expert** | SurrealDB schema, caching, queries | Database design, optimization, migration strategy |
| **Schema Architect** | Data modeling, type systems, validation | Data contracts, Rust type systems, DSL node types |
| **Feature Tester (Rust)** | Rust testing strategy, cargo test, proptest, nextest | Testing strategy, TDD workflow, property-based tests |

---

## Prerequisites

Before starting:

1. **Ensure required directories exist:**

   ```bash
   mkdir -p .ai/plans .ai/logs
   ```

2. **Verify sub-agent definitions are accessible:**

   These commands require sub-agent definitions in one of these locations:
   - `.claude/agents/` (project-level, preferred)
   - `~/.claude/agents/` (user-level, fallback)

   Required agent files:
   - `agents/rust-developer.md`
   - `agents/database-expert.md`
   - `agents/schema-architect.md`
   - `agents/feature-tester-rust.md`

---

## Step 1: Requirements Gathering

### 1.1 Understand the Task

Ask the user clarifying questions to fully understand what needs to be built:

1. **What is being built?**
   - Feature name and description
   - Primary goal and business value
   - Which module(s) affected: `/lib`, `/cli`, `/lsp`

2. **Who are the stakeholders?**
   - End users (CLI users, LSP consumers, library integrators)
   - Other systems/integrations (LLM providers, image processors)

3. **What are the constraints?**
   - Performance requirements (markdown parsing speed, cache latency)
   - Compatibility needs (CommonMark/GFM compliance, editor support)
   - Timeline expectations (scope, not duration)

### 1.2 Identify Requirements

Document both functional and non-functional requirements:

**Functional Requirements (FR):**

- What the system should DO
- DSL features and syntax
- Data inputs and outputs (markdown → HTML, DSL nodes → resolved content)
- Business rules and logic (caching strategy, LLM integration)

**Non-Functional Requirements (NFR):**

- Performance (parsing speed, cache hit ratio, LLM response time)
- Security (remote content validation, LLM prompt injection prevention)
- Scalability (large document handling, concurrent processing)
- Maintainability (error handling with thiserror, logging with tracing)
- Reliability (error recovery, graceful degradation)

### 1.3 Codebase Analysis

Use the Task tool with `subagent_type=Explore` to understand the current codebase:

```
Explore the codebase to understand:
1. Existing architecture and module structure (/lib, /cli, /lsp)
2. Relevant files and Rust modules
3. Testing infrastructure (cargo test setup, integration tests)
4. Build and CI processes
5. pulldown-cmark event processing patterns
6. SurrealDB schema and caching strategy
```

---

## Step 2: Create the Initial Plan

### 2.1 Plan Structure

Create a plan document at `.ai/plans/YYYY-MM-DD.plan-name.md`:

```markdown
# [Plan Name]

**Created:** [Date]
**Status:** Draft - Awaiting Review

## Executive Summary

[2-3 sentence overview of what this plan accomplishes]

## Requirements

### Functional Requirements

| ID | Requirement | Priority | Owner |
|----|-------------|----------|-------|
| FR-1 | [requirement] | High/Med/Low | [sub-agent] |
| FR-2 | [requirement] | High/Med/Low | [sub-agent] |

### Non-Functional Requirements

| ID | Requirement | Target | Owner |
|----|-------------|--------|-------|
| NFR-1 | [requirement] | [metric] | [sub-agent] |
| NFR-2 | [requirement] | [metric] | [sub-agent] |

## Architecture Overview

[High-level architecture description]

### Component Diagram

[ASCII or description of component relationships]

### Data Flow

[How data moves through the system - markdown parsing → DSL resolution → HTML generation]

## Phases

### Phase 1: [Phase Name]

**Principal Owner:** [Rust Developer/Database Expert/Schema Architect/Feature Tester]

**Goal:** [What this phase accomplishes]

**Dependencies:** None / [list dependencies]

**Blast Radius:** [Test scope - glob pattern or empty string for all tests]

**Deliverables:**
- [Deliverable 1]
- [Deliverable 2]

**Technical Details:**
- Crates to create/modify
- Key traits/structs/enums
- pulldown-cmark event handling patterns
- Integration points

**Acceptance Criteria:**
- [ ] [Criterion 1]
- [ ] [Criterion 2]

---

### Phase 2: [Phase Name]

[Repeat structure - include Blast Radius field]

---

## Blast Radius Analysis

For each phase, determine the **blast radius** - the scope of tests that should be run to verify both new functionality AND detect unintended downstream effects.

### How to Determine Blast Radius

1. **Identify direct test files:**
   - Tests for modules being created/modified
   - Example: `cargo test --lib parse` for parsing changes

2. **Identify downstream dependencies:**
   - What modules import/depend on the code being changed?
   - What tests cover those dependent modules?

3. **Construct the test command:**
   - Use `cargo test --lib [module]` for library changes
   - Use `cargo test --test [integration_test]` for integration tests
   - Use `cargo test` (empty string equivalent) for full coverage

4. **Use full test suite for foundational changes:**
   - If changes affect core traits, error types, or shared utilities
   - If unsure about impact scope
   - Empty string `""` or `cargo test` runs ALL tests

### Blast Radius Examples

| Change Type | Blast Radius |
|-------------|--------------|
| New DSL node type | `cargo test --lib dsl` |
| pulldown-cmark event processing | `cargo test --lib parse` |
| SurrealDB schema changes | `cargo test --lib cache` + integration tests |
| Core trait changes | `cargo test` (full suite) |
| Error type modifications | `cargo test` (full suite - errors affect everything) |
| CLI argument parsing | `cargo test --bin compose` |

---

## Cross-Cutting Concerns

### Testing Strategy
- Unit tests: `#[cfg(test)] mod tests` blocks in each module
- Integration tests: `tests/` directory for end-to-end workflows
- Property-based tests: proptest for DSL parsing, markdown roundtrips
- Doc tests: Examples in rustdoc comments
- Benchmarks: criterion for performance-critical paths

### Security Considerations
- Remote content validation (HTTP response size limits, content-type checks)
- LLM prompt injection prevention
- File path traversal prevention in transclusion
- Safe HTML generation (XSS prevention)

### Performance Considerations
- pulldown-cmark event stream efficiency (avoid unnecessary allocations)
- SurrealDB query optimization (indexes, prepared queries)
- Async/await patterns for LLM and HTTP requests
- Image processing pipeline (parallel processing with rayon)
- Caching strategy (cache hit ratio monitoring)

### Project-Specific Concerns

**pulldown-cmark Integration:**
- Event stream processing patterns (push vs pull)
- Custom event types for DSL nodes
- Offset tracking for LSP features

**SurrealDB Schema Design:**
- Cache key structure (content hash + config)
- TTL and eviction policies
- Query patterns for retrieval

**AI/LLM Integration:**
- Provider abstraction (trait-based design)
- Rate limiting and retry logic
- Response caching and validation
- Token counting and cost tracking

**Image Processing Pipeline:**
- Format detection and conversion
- Optimization strategies (lossless vs lossy)
- Parallel processing with rayon
- Hash-based deduplication

**Error Handling:**
- thiserror for library errors
- color-eyre for CLI pretty errors
- LSP error reporting (Diagnostic messages)

## Parallelization Opportunities

[Phases that can be executed in parallel]

| Parallel Group | Phases | Reason |
|----------------|--------|--------|
| Group A | Phase 1, Phase 2 | No dependencies |
| Group B | Phase 3 | Depends on Group A |

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| [Risk 1] | High/Med/Low | [Mitigation strategy] |

## Open Questions

- [ ] [Question 1]
- [ ] [Question 2]
```

### 2.2 Assign Principal Owners

For each phase and requirement, assign a principal owner based on:

| Content Type | Primary Owner | Secondary |
|--------------|---------------|-----------|
| Core library code (parsing, DSL, resolution) | Rust Developer | - |
| CLI implementation (Clap, argument parsing) | Rust Developer | - |
| LSP implementation (language server features) | Rust Developer | - |
| Data schemas, type design, validation | Schema Architect | Rust Developer |
| SurrealDB schema, queries, caching | Database Expert | - |
| Testing strategy, TDD, property-based tests | Feature Tester (Rust) | Rust Developer |
| Performance optimization, benchmarking | Rust Developer | - |

---

## Step 3: Parallel Sub-Agent Reviews

**CRITICAL:** Launch ALL reviews in PARALLEL using multiple Task tool calls in a single message.

### 3.1 Review Prompts

For each sub-agent with assigned ownership, create a review task:

**Rust Developer Review:**

```typescript
Task({
    subagent_type: "general-purpose",
    description: "Rust review of [plan-name]",
    model: "sonnet",
    run_in_background: true,
    prompt: `You are the Rust Developer sub-agent reviewing a plan.

## First: Activate Skills
Activate relevant Rust skills based on task type:
- \`rust-testing\` - For testing-related work
- \`rust-logging\` - For logging/observability
- \`rust-devops\` - For builds/deployment
- \`gfm\` or \`markdown-rs\` - For markdown parsing work
- \`thiserror\` or \`color-eyre\` - For error handling
- \`clap\` - For CLI work
- \`rust-lsp\` - For LSP work

## Context
Read your expertise guidelines in: .claude/agents/rust-developer.md

## Plan to Review
Read the plan at: .ai/plans/YYYY-MM-DD.plan-name.md

## Your Review Focus
Review ALL sections where Rust Developer is assigned as owner, plus:

1. **Architecture**
   - Is the module structure appropriate?
   - Are trait boundaries well-defined?
   - Is the error type design sound (thiserror)?
   - Are pulldown-cmark event patterns correct?

2. **Performance**
   - Are allocation patterns optimized?
   - Are hot paths identified (parsing, DSL resolution)?
   - Is async/await used appropriately (LLM, HTTP)?
   - Is rayon used for parallelizable work (image processing)?

3. **Safety**
   - Are ownership patterns correct?
   - Is unsafe code minimized and justified?
   - Are lifetimes handled properly?
   - Are file path operations safe (no traversal)?

4. **Testing**
   - Are unit tests planned in \`#[cfg(test)] mod tests\` blocks?
   - Are integration tests planned in \`tests/\` directory?
   - Is property-based testing (proptest) considered for DSL parsing?
   - Are doc tests planned for public APIs?
   - Is TDD workflow incorporated?

5. **Observability**
   - Is tracing integrated for debugging?
   - Are spans and log levels appropriate?
   - Is error context preserved (thiserror sources)?

6. **DarkMatter DSL Specifics**
   - Are DSL node types well-modeled?
   - Is transclusion logic correct?
   - Are LLM operations (summarize, consolidate) properly abstracted?
   - Is interpolation ({{variable}}) handled correctly?

7. **pulldown-cmark Integration**
   - Are event streams processed efficiently?
   - Are custom events for DSL nodes well-designed?
   - Is offset tracking correct for LSP features?

## Output Format
Return your review as:

### Rust Developer Review

**Overall Assessment:** [Approve / Approve with Changes / Request Revision]

**Strengths:**
- [strength 1]
- [strength 2]

**Concerns:**
- [concern 1 with suggested fix]
- [concern 2 with suggested fix]

**Suggested Changes:**
1. [specific change to plan]
2. [specific change to plan]

**Parallelization Notes:**
- [which Rust phases can run in parallel]
- [dependencies to be aware of]

**Missing Considerations:**
- [anything overlooked]`
})
```

**Schema Architect Review:**

```typescript
Task({
    subagent_type: "general-purpose",
    description: "Schema/data modeling review of [plan-name]",
    model: "sonnet",
    run_in_background: true,
    prompt: `You are the Schema Architect sub-agent reviewing a plan.

## Context
Read your expertise guidelines in: .claude/agents/schema-architect.md

## Plan to Review
Read the plan at: .ai/plans/YYYY-MM-DD.plan-name.md

## Your Review Focus
Review ALL sections where Schema Architect is assigned as owner, plus:

1. **Data Modeling**
   - Are Rust types well-defined (structs, enums)?
   - Are DSL node types correctly modeled?
   - Are invariants enforced by the type system?
   - Can invalid states be represented? (they shouldn't be)

2. **Type System Design**
   - Are discriminated unions (Rust enums) used appropriately?
   - Are newtypes used for IDs and domain concepts?
   - Is serde integration correct for serialization?
   - Are lifetimes minimized where possible?

3. **Validation Strategy**
   - Is validation at system boundaries (CLI input, remote content)?
   - Are business rules encoded in types vs runtime checks?
   - Is error reporting informative (thiserror messages)?

4. **DSL Schema Specifics**
   - Are frontmatter types well-defined?
   - Are DSL node variants (transclusion, summarize, consolidate) complete?
   - Is the document graph structure correct (DAG validation)?
   - Are cache keys properly typed?

5. **SurrealDB Schema Integration**
   - Are Rust types compatible with SurrealDB schema?
   - Is serde serialization correctly configured?
   - Are query result types properly modeled?

6. **Evolution & Versioning**
   - How will schemas evolve?
   - Is backwards compatibility addressed?
   - Are migrations planned for breaking changes?

## Output Format
Return your review as:

### Schema Architect Review

**Overall Assessment:** [Approve / Approve with Changes / Request Revision]

**Strengths:**
- [strength 1]
- [strength 2]

**Concerns:**
- [concern 1 with suggested fix]
- [concern 2 with suggested fix]

**Suggested Changes:**
1. [specific change to plan]
2. [specific change to plan]

**Parallelization Notes:**
- [which schema/data phases can run in parallel]
- [dependencies to be aware of]

**Missing Considerations:**
- [anything overlooked]`
})
```

**Database Expert Review:**

```typescript
Task({
    subagent_type: "general-purpose",
    description: "Database design/optimization review of [plan-name]",
    model: "sonnet",
    run_in_background: true,
    prompt: `You are the Database Expert sub-agent reviewing a plan.

## First: Activate Skills
Activate the \`surrealdb\` and \`database\` skills before proceeding with the review.

## Context
Read your expertise guidelines in: .claude/agents/database-expert.md

## Plan to Review
Read the plan at: .ai/plans/YYYY-MM-DD.plan-name.md

## Your Review Focus
Review ALL sections where Database Expert is assigned as owner, plus:

1. **SurrealDB Schema Design**
   - Are tables/records well-defined?
   - Are relationships (graph edges) appropriate?
   - Are field types optimal?
   - Are constraints (ASSERT, UNIQUE) used effectively?

2. **Caching Strategy**
   - Is cache key structure correct (content hash + config)?
   - Are TTL policies appropriate (default 1 day for HTTP)?
   - Is eviction strategy defined?
   - Are cache misses handled gracefully?

3. **Performance Considerations**
   - Are appropriate indexes planned?
   - Are query patterns optimized?
   - Is connection pooling configured?
   - Are bulk operations used where appropriate?

4. **SurrealDB-Specific Features**
   - Are record IDs properly structured?
   - Are graph traversals efficient?
   - Is SurrealQL query syntax correct?
   - Are transactions used appropriately?

5. **Migration Strategy**
   - Are schema migrations planned?
   - Is database initialization handled (init() function)?
   - Is database location configurable ($HOME vs git root)?
   - Is rollback strategy defined?

6. **Data Integrity**
   - Are integrity constraints sufficient?
   - Is validation at database level vs application level balanced?
   - Are LLM response caches immutable?
   - Are embeddings stored efficiently?

7. **Composition-Specific Concerns**
   - Cache storage location (_composition-cache.db)
   - LLM response caching (summarize, consolidate, topics)
   - Embedding storage and similarity search
   - Remote content caching (HTTP responses)

## Output Format
Return your review as:

### Database Expert Review

**Overall Assessment:** [Approve / Approve with Changes / Request Revision]

**Strengths:**
- [strength 1]
- [strength 2]

**Concerns:**
- [concern 1 with suggested fix]
- [concern 2 with suggested fix]

**Suggested Changes:**
1. [specific change to plan]
2. [specific change to plan]

**Parallelization Notes:**
- [which database phases can run in parallel]
- [dependencies to be aware of]

**Missing Considerations:**
- [anything overlooked regarding database design/performance]`
})
```

**Feature Tester Review (Rust):**

```typescript
Task({
    subagent_type: "general-purpose",
    description: "Rust testing strategy review of [plan-name]",
    model: "sonnet",
    run_in_background: true,
    prompt: `You are the Feature Tester (Rust) sub-agent reviewing a plan.

## First: Activate Skills
Activate the \`rust-testing\` skill before proceeding with the review.

## Context
Read your expertise guidelines in: .claude/agents/feature-tester-rust.md

## Plan to Review
Read the plan at: .ai/plans/YYYY-MM-DD.plan-name.md

## Your Review Focus
Review the Rust testing strategy and ensure comprehensive test coverage:

1. **Test Strategy Completeness**
   - Are unit tests planned in \`#[cfg(test)] mod tests\` blocks?
   - Are integration tests planned in \`tests/\` directory?
   - Is TDD workflow incorporated appropriately?
   - Are doc tests planned for public APIs?

2. **Test Coverage**
   - Are happy paths covered?
   - Are edge cases and error conditions addressed?
   - Are property-based tests (proptest) planned for:
     - DSL parsing (markdown → AST roundtrips)
     - Document graph validation (DAG properties)
     - Cache key generation (determinism)
   - Are snapshot tests (insta) considered for HTML output?

3. **Test Organization**
   - Are unit tests in same file as implementation?
   - Are integration tests testing public API only?
   - Are test utilities in \`tests/common/mod.rs\`?
   - Is test data organized (fixture markdown files)?

4. **Acceptance Criteria Testability**
   - Can each acceptance criterion be verified by a test?
   - Are there missing criteria that should be added?
   - Are DSL features testable (transclusion, summarize, etc.)?

5. **Testing Dependencies**
   - Are mockall traits used for LLM provider isolation?
   - Are HTTP mocks used for remote content testing?
   - Is SurrealDB isolated in tests (in-memory mode)?
   - Are external dependencies properly abstracted?

6. **Rust-Specific Testing**
   - Is cargo-nextest configured for better output?
   - Are benchmarks (criterion) planned for:
     - Markdown parsing (pulldown-cmark event processing)
     - DSL resolution
     - Cache lookups
     - Image processing
   - Are compilation tests planned for type constraints?

7. **Composition-Specific Testing**
   - Are DarkMatter DSL features thoroughly tested?
   - Is pulldown-cmark event stream processing verified?
   - Are SurrealDB cache operations tested?
   - Are LLM integrations mocked appropriately?
   - Is image processing tested with sample files?

## Output Format
Return your review as:

### Feature Tester (Rust) Review

**Overall Assessment:** [Approve / Approve with Changes / Request Revision]

**Strengths:**
- [strength 1]
- [strength 2]

**Concerns:**
- [concern 1 with suggested fix]
- [concern 2 with suggested fix]

**Suggested Changes:**
1. [specific change to plan]
2. [specific change to plan]

**Test Scenarios to Add:**
- [missing test scenario 1]
- [missing test scenario 2]

**Missing Considerations:**
- [anything overlooked]`
})
```

### 3.2 Launch Reviews in Parallel

**IMPORTANT:** Send ALL relevant Task calls in a SINGLE message to run them in parallel.

Only invoke sub-agents that have assigned ownership in the plan.

Example parallel invocation:

```typescript
// All in ONE message for parallel execution
Task({ /* Rust Developer review */ run_in_background: true })
Task({ /* Database Expert review */ run_in_background: true })
Task({ /* Schema Architect review */ run_in_background: true })
Task({ /* Feature Tester (Rust) review */ run_in_background: true })
```

### 3.3 Collect Review Results

Use TaskOutput to collect results from all background tasks:

```typescript
TaskOutput({ task_id: "rust-review-id", block: true })
TaskOutput({ task_id: "database-review-id", block: true })
TaskOutput({ task_id: "schema-review-id", block: true })
TaskOutput({ task_id: "tester-review-id", block: true })
```

---

## Step 4: Consolidation and Optimization

After all reviews complete, perform a final consolidation pass:

### 4.1 Synthesize Feedback

1. **Aggregate Concerns:** Group similar concerns across reviews
2. **Resolve Conflicts:** If reviewers disagree, determine the best path
3. **Prioritize Changes:** Order suggested changes by impact

### 4.2 Update the Plan

Incorporate review feedback into the plan:

1. Update requirement assignments if suggested
2. Modify phase details based on concerns
3. Add missing considerations identified by reviewers
4. Update acceptance criteria
5. Add project-specific considerations:
   - pulldown-cmark event processing patterns
   - SurrealDB schema refinements
   - LLM integration error handling
   - Image processing optimizations

### 4.3 Finalize Parallelization Analysis

Based on all reviews, create the final parallelization strategy:

```markdown
## Implementation Parallelization Strategy

### Parallel Execution Groups

| Group | Phases | Can Start After | Assignees |
|-------|--------|-----------------|-----------|
| A | 1, 2 | Plan approval | Rust Dev, Schema |
| B | 3 | Group A complete | Database, Rust Dev |
| C | 4, 5 | Phase 3 complete | Rust Dev, Tester |

### Parallelization Diagram
```

```text
Timeline:
─────────────────────────────────────────────────────►

Group A: ████████████ (Phase 1 + Phase 2 in parallel)
                     │
Group B:             └──████████ (Phase 3)
                              │
Group C:                      └──████████████ (Phase 4 + Phase 5)
```

### Synchronization Points

1. **After Group A:** Core types and traits must be finalized
2. **After Group B:** SurrealDB schema must be deployed
3. **Final:** Integration testing across lib/cli/lsp modules

### 4.4 Update Plan Status

Change the plan status and add the review summary:

```markdown
**Status:** Reviewed - Ready for Implementation

## Review Summary

**Reviews Completed:** [Date]

**Reviewers:**
- Rust Developer: [Approve/Approve with Changes/Request Revision]
- Database Expert: [Approve/Approve with Changes/Request Revision]
- Schema Architect: [Approve/Approve with Changes/Request Revision]
- Feature Tester (Rust): [Approve/Approve with Changes/Request Revision]

**Key Changes from Review:**
1. [Change 1]
2. [Change 2]
3. [Change 3]

**Resolved Concerns:**
- [Concern] → [Resolution]
```

---

## Step 5: Present to User

### 5.1 Summary Report

Present the final plan to the user with:

1. **Executive Summary** - What will be built
2. **Phase Overview** - High-level view of all phases
3. **Owner Assignments** - Who owns what
4. **Parallelization Strategy** - How to maximize efficiency
5. **Key Risks** - Top risks and mitigations
6. **Open Questions** - Items needing user input

### 5.2 Request Approval

Ask the user to:

1. Review the plan at `.ai/plans/YYYY-MM-DD.plan-name.md`
2. Answer any open questions
3. Approve or request changes

---

## Output Artifacts

This command produces:

| Artifact | Location | Purpose |
|----------|----------|---------|
| Plan Document | `.ai/plans/YYYY-MM-DD.plan-name.md` | Complete implementation plan |
| Review Log | Embedded in plan | Sub-agent feedback |

---

## Example Workflow

```text
User: Create a plan for adding vector similarity search for related documents

Main Thread:
├── Step 1: Gather requirements
│   ├── Ask clarifying questions
│   ├── Document FR and NFR
│   └── Explore codebase (SurrealDB schema, DSL resolution)
│
├── Step 2: Create initial plan
│   ├── Draft plan with phases
│   ├── Assign principal owners:
│   │   ├── Rust Developer: Embedding generation, similarity search API
│   │   ├── Database Expert: SurrealDB vector storage and queries
│   │   ├── Schema Architect: Embedding types, document metadata
│   │   └── Feature Tester: Test strategy for similarity algorithms
│   └── Save to .ai/plans/
│
├── Step 3: Parallel reviews (ALL AT ONCE)
│   ├── Rust Developer ──────┐
│   ├── Database Expert ──────├── Running in parallel
│   ├── Schema Architect ─────┤
│   └── Feature Tester (Rust) ┘
│
├── Step 4: Consolidation
│   ├── Synthesize feedback
│   ├── Update plan
│   ├── Finalize parallelization:
│   │   ├── Group A: Schema + DB setup (parallel)
│   │   ├── Group B: Embedding generation (after Group A)
│   │   └── Group C: Similarity search + tests (after Group B)
│   └── Mark as reviewed
│
└── Step 5: Present to user
    └── Request approval
```

---

## Tips for Success

1. **Be thorough in Step 1** - Good requirements lead to good plans
2. **Assign owners carefully** - Match expertise to tasks
3. **Always run reviews in parallel** - This is the key efficiency gain
4. **Don't skip consolidation** - Cross-cutting concerns emerge in review
5. **Document parallelization clearly** - Implementation teams need this
6. **Keep the plan living** - Update as implementation reveals new information
7. **Consider Composition specifics:**
   - pulldown-cmark event stream patterns
   - SurrealDB schema design for caching
   - LLM provider abstractions
   - Image processing pipelines
   - LSP offset tracking

---

## Next Steps After Planning

Once the plan is approved:

1. **For TDD workflow:** Use `/execute-phase` to implement each phase
2. **For feature workflow:** Use `/add-feature` with the plan as context
3. **For parallel implementation:** Coordinate sub-agents based on parallelization groups
4. **Run tests:** `cargo test` or `cargo nextest run`
5. **Check benchmarks:** `cargo bench` for performance-critical changes
