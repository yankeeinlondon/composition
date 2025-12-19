---
description: Execute a phase of a detailed TDD plan from .ai/plans/
argument-hint: [phase-number]
---

# Execute Phase Using TDD Workflow

You are now in **TDD Execution Mode**. Your goal is to execute a single phase from a plan following the rigorous 5-Step TDD Workflow.

## Prerequisites

Before starting, ensure:

1. A plan exists in `.ai/plans/` directory
2. You know which phase to execute
3. All previous phases are complete (if applicable)

## Step 0: Load the Testing Skill

Before proceeding, invoke the `rust-testing` skill to load Rust testing patterns, cargo test, and nextest best practices.

Use the Skill tool to activate the skill.

## Step 1: Identify the Phase

Ask the user:

1. **Which plan should we execute?**
   - List available plans in `.ai/plans/`
   - Or ask for the plan filename

2. **Which phase should we execute?**
   - Show available phases from the plan
   - Suggest the next incomplete phase
   - Confirm phase number with the user

3. **Read the plan file:**
   - Use Read tool to load `.ai/plans/[planName].md`
   - Extract the details for the specified phase
   - **Extract the blast radius** for this phase (test scope pattern)
   - If blast radius is empty string `""`, tests will run against entire test suite

## Step 1.5: EXPLORE EXISTING CODE - MANDATORY

**🚨 CRITICAL: Before doing ANYTHING else, understand what code already exists! 🚨**

**Purpose:** Prevent architectural misunderstandings by examining existing code structure BEFORE implementation.

**Actions:**

1. **Identify files mentioned in the plan:**

   From the phase description, note ALL files that will be created or modified.

2. **Search for existing related files:**

   ```bash
   # Search for files with similar names
   find lib -name "*[keyword]*" -type f
   find cli -name "*[keyword]*" -type f

   # Or use Glob
   Glob: lib/**/*[keyword]*.rs
   Glob: cli/**/*[keyword]*.rs
   ```

   For example, if implementing "markdown parsing", search for:
   - Existing files with "markdown", "parse", "md" in the name
   - Related module stubs
   - Similar utilities

3. **Read existing stubs/interfaces:**

   If files already exist:
   - **Read them COMPLETELY** using the Read tool
   - Understand the existing structure
   - Note TODOs or incomplete sections
   - Check if you're meant to COMPLETE existing code, not create new files

4. **Understand the architecture:**

   Before writing code, understand:
   - What patterns does the existing code use?
   - Are there type definitions you need to follow?
   - Are there existing traits or base structs?
   - How do similar features work in the codebase?

5. **Use Grep to find usage patterns:**

   ```bash
   # Find how existing code uses similar features
   Grep: "similar pattern"
   Grep: "use.*types::"
   ```

6. **Document findings in log file:**

   ```markdown
   ### Existing Code Exploration

   **Files found:**
   - `lib/src/parser/markdown.rs` - EXISTS as stub (needs completion)
   - `lib/src/types/document.rs` - Defines Document type pattern

   **Architecture notes:**
   - Parser modules return Result<T, ParserError>
   - Pattern: public parse() function calls private helpers
   - Uses pulldown-cmark for markdown event stream

   **Decision:** Complete existing stub, don't create new utility
   ```

7. **Validate plan against reality:**

   Ask yourself:
   - Does the plan match the existing code structure?
   - Am I creating something that already exists?
   - Am I understanding the architecture correctly?
   - Should I complete an existing stub instead of creating new files?

**If you discover a mismatch between the plan and existing code, STOP and inform the user before proceeding.**

**DO NOT SKIP THIS STEP.** Most architectural mistakes happen because this exploration was skipped.

---

## Step 2: SNAPSHOT - Capture Current Test State

**Purpose:** Establish a baseline so you can detect regressions and measure progress within the blast radius.

**Actions:**

1. **Run tests within the blast radius:**

   ```bash
   # If blast radius is empty string, run all tests:
   cargo nextest run  # or cargo test

   # If blast radius is a pattern, run scoped tests:
   cargo nextest run -E 'test([pattern])'
   # or
   cargo test [pattern]
   ```

2. **Run doc tests:**

   ```bash
   cargo test --doc
   ```

3. **Create XML snapshot:**

   Create a simple XML representation of test results:

   ```xml
   <test-snapshot date="YYYY-MM-DD">
     <blast-radius>[pattern or "all"]</blast-radius>
     <suite name="runtime-tests" total="X" passed="Y" failed="Z" />
     <suite name="doc-tests" total="X" passed="Y" failed="Z" />
     <starting-failures>
       <failure test="module::test_name" />
     </starting-failures>
   </test-snapshot>
   ```

4. **Document starting failures within blast radius** - these are your baseline, don't fix them yet

## Step 3: CREATE LOG - Document Starting Position

**Purpose:** Create a detailed record for debugging and tracking progress.

**Actions:**

1. **Create log file:**
   - Path: `.ai/logs/YYYY-MM-[planName]-phase[N]-log.md`
   - Example: `.ai/logs/2025-12-composition-phase1-log.md`
   - Create `.ai/logs/` directory if it doesn't exist

2. **Write log file with starting state:**

   ```markdown
   # Phase [N]: [Phase Name]

   **Plan:** [planName]
   **Phase:** [N]
   **Started:** [Date and Time]
   **Blast Radius:** [test scope pattern or "all"]

   ## Phase Overview

   [Copy phase overview from plan]

   ## Starting Test Position

       <test-snapshot date="YYYY-MM-DD">
         <blast-radius>[pattern or "all"]</blast-radius>
         <suite name="runtime-tests" total="X" passed="Y" failed="Z" />
         <suite name="doc-tests" total="X" passed="Y" failed="Z" />
         <starting-failures>
           <failure test="module::test_name" />
         </starting-failures>
       </test-snapshot>

   ## Repo Starting Position

   **Last local commit:** [git log -1 --format="%H"]
   **Last remote commit:** [git log origin/main -1 --format="%H" 2>/dev/null || echo "N/A"]
   **Branch:** [git branch --show-current]
   **Dirty files:** [git status --short || echo "None"]

   ## Work Log

   [This section will be updated as work progresses]
   ```

3. **Save the log file**

4. **IMPORTANT:** Verify the markdown file has NO linting errors - proper formatting makes logs readable and professional

## Step 4: WRITE TESTS - Create Tests FIRST

**Purpose:** Tests define the contract and expected behavior before any code is written.

**🚨 CRITICAL: This is TRUE Test-Driven Development - tests MUST be written BEFORE implementation! 🚨**

**Actions:**

1. **Create WIP directory if needed:**

   ```bash
   mkdir -p tests/wip
   ```

2. **Review test requirements from plan:**

   - Happy path tests
   - Edge case tests
   - Error condition tests
   - Doc tests

3. **Create test files:**

   For **unit tests:**
   - Add `#[cfg(test)] mod tests` blocks within the source file
   - File location: Same file as the code being tested (e.g., `lib/src/parser.rs`)
   - Use `use super::*;` to access private functions

   For **integration tests:**
   - File naming: `phase[N]-[description].rs`
   - Example: `tests/wip/phase1-markdown-parsing.rs`
   - Each file is a separate crate (can't access private items)

4. **Write comprehensive tests:**

   ```rust
   // Unit tests (in lib/src/module.rs)
   #[cfg(test)]
   mod tests {
       use super::*;

       #[test]
       fn it_returns_expected_output() {
           let result = parse_input("valid").unwrap();
           assert_eq!(result.value, "expected");
       }

       #[test]
       fn it_rejects_empty_input() {
           let result = parse_input("");
           assert!(result.is_err());
       }

       #[test]
       #[should_panic(expected = "invalid format")]
       fn it_panics_on_invalid_format() {
           parse_input("!!!").unwrap();
       }

       // Test with Result return for cleaner error handling
       #[test]
       fn it_handles_complex_input() -> Result<(), Box<dyn std::error::Error>> {
           let result = parse_input("complex")?;
           assert_eq!(result.count, 42);
           Ok(())
       }
   }
   ```

   ```rust
   // Integration tests (in tests/wip/phase1-feature.rs)
   use composition::Feature;

   #[test]
   fn feature_end_to_end() {
       let feature = Feature::new();
       let result = feature.process_workflow();
       assert!(result.is_ok());
   }
   ```

5. **Consider property-based tests for complex logic:**

   ```rust
   use proptest::prelude::*;

   proptest! {
       #[test]
       fn roundtrip_serialization(input: String) {
           let serialized = serialize(&input);
           let deserialized = deserialize(&serialized)?;
           prop_assert_eq!(input, deserialized);
       }
   }
   ```

6. **Verify tests FAIL initially:**

   ```bash
   cargo test --no-run  # Just compile
   cargo nextest run    # Run tests (should fail)
   ```

   - Confirm tests fail (no implementation exists yet)
   - This verifies the tests are checking for real functionality, not trivially passing

7. **Think critically about test completeness:**

   - Review each test and ask: **If the functionality were built, would this test be meaningful?**
   - Consider all variants the function/module can express:
     - Different input types and combinations
     - Boundary conditions and edge cases
     - Error states and failure modes
     - Return value variations
   - **Think hardest here** - missing variants now means gaps in coverage later
   - Are you testing behavior, not just implementation details?
   - Would these tests catch regressions if someone changed the code?

8. **Update log file with test creation:**

   Add to "Work Log" section:

   ```markdown
   ### Tests Created

   **Unit tests:**
   - `lib/src/parser.rs` - 5 tests in `#[cfg(test)] mod tests`
   - `lib/src/types.rs` - 3 tests in `#[cfg(test)] mod tests`

   **Integration tests:**
   - `tests/wip/phase1-markdown-parsing.rs` - 7 end-to-end tests

   **Initial test run:** All tests fail as expected (no implementation yet)
   ```

## Step 4.5: VALIDATE TESTS - Critical Checkpoint

**⚠️ MANDATORY: Before proceeding to implementation, validate your tests are correct**

**Purpose:** Catch testing pattern mistakes NOW, before they're baked into implementation. This checkpoint prevents hours of rework.

**Actions:**

1. **Open the Rust testing skill:**

   Open `~/.claude/skills/rust-testing/SKILL.md` for testing patterns

2. **Validate test structure:**

   - Unit tests in `#[cfg(test)] mod tests` blocks within source files
   - Integration tests in `tests/` directory
   - `use super::*;` present for accessing private items
   - Descriptive test names: `fn it_returns_error_for_invalid_input()`

3. **Validate test patterns:**

   - Using `assert_eq!`, `assert_ne!`, `assert!` correctly
   - `#[should_panic]` for expected panics
   - Result return type for fallible tests: `fn test() -> Result<(), Box<dyn Error>>`

4. **Check for property tests (if applicable):**

   - Complex logic should have proptest invariants
   - Roundtrip tests for serialization

5. **Run the tests:**

   ```bash
   cargo nextest run     # Better output
   cargo test            # Standard runner
   cargo test --no-run   # Just compile, verify tests build
   ```

6. **Update log file with validation:**

   ```markdown
   ### Test Validation

   - Completed Rust testing checklist ✅
   - Unit tests in correct location ✅
   - Integration tests in tests/ directory ✅
   - Tests ready for implementation ✅
   ```

**🚨 DO NOT PROCEED TO IMPLEMENTATION IF ANY CHECKLIST ITEM FAILS 🚨**

Testing mistakes caught here save hours of debugging and rework later. If you're unsure about any pattern, re-read the skill guide sections.

---

## Step 5: IMPLEMENTATION - Build to Pass Tests

**Purpose:** Let tests drive the implementation, ensuring you build exactly what's needed.

**Actions:**

1. **Implement minimal code to pass each test:**
   - Start with one test or small group of related tests
   - Write the simplest code that makes tests pass
   - Don't over-engineer or add features not covered by tests

2. **Follow the plan's implementation details:**
   - Create files specified in the plan
   - Modify files specified in the plan
   - Implement key functions/structs as planned

3. **Iterate rapidly:**
   - Run tests frequently: `cargo nextest run` or `cargo test`
   - Fix failures immediately
   - Keep the feedback loop tight

4. **Continue until all phase tests pass:**
   - All unit tests must be green
   - All integration tests in `tests/wip/` must be green
   - No shortcuts - every test must pass

5. **Run quality checks:**
   ```bash
   cargo clippy -- -D warnings
   cargo fmt --check
   cargo test --doc
   ```

6. **Refactor with confidence:**
   - Once tests pass, improve code quality
   - Tests act as a safety net
   - Re-run tests after each refactor

7. **Update log file during implementation:**

   Add to "Work Log" section as you go:

   ```markdown
   ### Implementation Progress

   **[Timestamp]** - Created `lib/src/parser.rs`
   - Implemented `parse()` function
   - Tests passing: X/Y

   **[Timestamp]** - Modified `lib/src/types.rs`
   - Added integration with parser
   - Tests passing: Y/Y ✅

   **[Timestamp]** - Refactored for better readability
   - All tests still passing ✅
   ```

## Step 6: CLOSE OUT - Verify and Document

**Purpose:** Ensure quality, prevent regressions, and properly document completion.

### CRITICAL WARNING: DO NOT MIGRATE TESTS AUTOMATICALLY

**Tests MUST remain in `tests/wip/` until the user explicitly reviews and approves them!**

**Actions:**

1. **Run tests within blast radius:**

   ```bash
   # If blast radius is empty string, run all tests:
   cargo nextest run  # or cargo test
   cargo test --doc   # doc tests

   # If blast radius is a pattern, run scoped tests:
   cargo nextest run -E 'test([pattern])'
   cargo test [pattern]
   ```

2. **Run quality checks:**

   ```bash
   cargo clippy -- -D warnings
   cargo fmt --check
   cargo build --release
   ```

3. **Check for regressions within blast radius:**

   Compare ending test failures against starting failures:

   - **Capture ending failures:** Run tests and note all failures
   - **Compare against starting failures:** Identify NEW failures
   - **New regressions = ending failures - starting failures**

   If NEW regressions exist:

   - **STOP and think deeply** - understand WHY, not just the error message
   - Add a "Regressions Found" section to log file with test name, failure message, root cause analysis, and resolution
   - Determine root cause:
     - Is your implementation incorrect?
     - Does the existing test need updating? (only if requirements changed)
     - Is there a side effect you didn't anticipate?
   - Fix the root cause, not just the symptom
   - Re-run tests within blast radius to confirm fix

4. **Update log file with completion:**

   Add `## Phase Completion` section:

   ```markdown
   ## Phase Completion

   **Completed:** [Date and Time]
   **Duration:** [Time taken]
   **Blast Radius:** [test scope pattern or "all"]

   ### Final Test Results (within blast radius)

   - WIP tests: X/X passing ✅
   - Blast radius tests: Y/Y passing ✅
   - Doc tests: Z/Z passing ✅

   ### Quality Checks

   - Clippy: ✅ 0 warnings
   - Formatting: ✅ Pass
   - Build: ✅ Success

   ### Regression Analysis

   **Starting failures:** [count] tests
   - [list from starting snapshot]

   **Ending failures:** [count] tests
   - [list from final run]

   **New regressions:** [None / list any new failures]

   ### Files Changed

   **Created:**
   - `lib/src/new-module.rs`

   **Modified:**
   - `lib/src/existing-module.rs`

   ### Tests Location

   **IMPORTANT:** Tests remain in `tests/wip/` awaiting user review.

   The user must review and approve tests before they are migrated to their permanent location.
   ```

   **Verify markdown quality:** Ensure log file has no linting errors

5. **Update plan status:**

   - Read the plan file
   - Mark this phase as complete
   - Update the plan's status section
   - Save the updated plan
   - **Verify markdown quality:** Ensure updated plan has no linting errors

6. **Report completion to user:**

   Provide a clear summary:

   ```text
   ✅ Phase [N] Complete: [Phase Name]

   **What was implemented:**
   - [Summary of implementation]

   **Test coverage added:**
   - [Number] unit tests
   - [Number] integration tests
   - All tests passing
   - No regressions

   **Quality checks:**
   - Clippy: 0 warnings
   - Formatting: Pass
   - Build: Success

   **Tests location:**
   🔍 Tests are in `tests/wip/` awaiting your review

   **Next steps:**
   1. Review tests in `tests/wip/phase[N]-*.rs`
   2. When satisfied, tell me to "migrate the tests"
   3. Or run `/execute-phase [N+1]` to continue to next phase

   **Log file:** `.ai/logs/YYYY-MM-[planName]-phase[N]-log.md`
   ```

## After User Reviews Tests

**Only after explicit user approval:**

1. **Ask where tests should be migrated:**
   - Think carefully about the right permanent location
   - Consider if a new subdirectory is needed
   - Follow project's test organization patterns
   - Unit tests stay in source files (`#[cfg(test)]`)
   - Integration tests move to `tests/` directory

2. **Migrate integration tests:**

   ```bash
   mv tests/wip/phase[N]-*.rs tests/integration/
   # or
   mv tests/wip/phase[N]-*.rs tests/
   ```

3. **Verify tests still pass in new location:**

   ```bash
   cargo nextest run
   cargo test --doc
   ```

4. **Delete WIP directory (if empty):**

   ```bash
   rmdir tests/wip
   ```

5. **Update log file with final locations:**

   ```markdown
   ### Tests Migrated

   - `tests/wip/phase1-markdown.rs` → `tests/integration/markdown.rs`

   ✅ All tests passing in new location
   ```

   **Verify markdown quality:** Ensure log file updates have no linting errors

## Important Reminders

- **Tests FIRST** - Always write tests before implementation
- **WIP directory** - All new integration tests go in `tests/wip/`
- **Unit tests inline** - Unit tests stay in source files with `#[cfg(test)]`
- **No auto-migration** - Tests remain in WIP until user approves
- **Log everything** - Keep the log file updated throughout
- **Understand failures** - Don't just fix symptoms, understand root causes
- **Blast radius testing** - Run tests within blast radius, not necessarily entire suite
- **Track regressions properly** - Compare ending failures against starting failures; only NEW failures are regressions
- **Test location** - Unit tests inline with `#[cfg(test)]`, integration tests in `tests/`
- **Property tests** - Use proptest for complex invariants and roundtrip testing
- **Quality checks** - Run clippy, fmt, and build to ensure high code quality
- **Markdown quality** - ALL markdown files (logs, plan updates) MUST be lint-free; linting errors make documents very hard to read

## Phase Execution Checklist

Use this checklist to ensure you don't miss any steps:

- [ ] Rust testing skill loaded
- [ ] Plan and phase identified
- [ ] **Blast radius extracted from plan**
- [ ] **Existing code explored (Step 1.5)**
- [ ] SNAPSHOT captured (baseline test state within blast radius)
- [ ] **Starting failures documented**
- [ ] LOG created in `.ai/logs/`
- [ ] Starting position documented
- [ ] Tests written (unit tests in source, integration in `tests/wip/`)
- [ ] Tests initially failed (proving validity)
- [ ] Implementation completed
- [ ] All WIP tests passing
- [ ] **Blast radius tests run**
- [ ] **Quality checks passed (clippy, fmt, build)**
- [ ] **Ending failures documented**
- [ ] **No NEW regressions** (ending - starting = 0 new failures)
- [ ] Log file updated with completion
- [ ] Plan status updated
- [ ] User notified with summary
- [ ] **Tests remain in WIP awaiting review**

**After user review:**

- [ ] User approved tests
- [ ] Integration tests migrated to permanent location
- [ ] Tests verified in new location
- [ ] WIP directory removed (if empty)
- [ ] Log file updated with final locations
