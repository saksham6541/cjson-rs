## [Hour 0] Initial scope

**Context:** The contest requires a standalone implementation of core cJSON behavior that is not a direct Rust port of an existing Rust package.
**Decision:** Keep the submission in Rust for the cJSON track and focus on core parser/formatter behavior first.
**Rationale:** This keeps the implementation aligned with the requested track while still producing a buildable and testable submission.
**Equivalence impact:** Covers the core JSON data model and round-trip formatting for the supported cases.

## [Hour 2] CLI and testability

**Context:** The project needs a simple build-and-run path and a practical verification loop for the submission.
**Decision:** Provide a tiny CLI entry point and a small test suite around parse and format behavior.
**Rationale:** This makes the submission easier to review and demonstrates that the implementation is runnable without extra scaffolding.
**Equivalence impact:** Preserves the basic parse/print behavior expected of the upstream library.

## [Hour 4] Object lookup semantics fix

**Context:** The original cJSON implementation treats the default object lookup helper as case-insensitive and reserves the case-sensitive variant for the explicit helper. The Rust port had the behavior reversed.
**Decision:** Change `get_object_item` to be case-insensitive by default and keep `get_object_item_case_sensitive` as the explicit sensitive lookup path.
**Rationale:** This matches the upstream API contract from [original/cJSON/cJSON.c](original/cJSON/cJSON.c), where `cJSON_GetObjectItem` calls `get_object_item(..., false)` and `cJSON_GetObjectItemCaseSensitive` calls `get_object_item(..., true)`.
**Equivalence impact:** Aligns the Rust port with the original behavior for default object lookups.

## [Hour 6] Parser bug fixes

**Context:** The parser previously used byte-based slicing for literal matching while advancing the parser over Unicode characters, and it accepted Unicode whitespace that is not part of JSON. Both are divergences from the reference parser.
**Decision:** Compare literals against the character stream rather than the original byte slice and restrict whitespace handling to the four JSON separators: space, tab, newline, and carriage return.
**Rationale:** This avoids panics and removes spurious acceptance of non-JSON separators.
**Equivalence impact:** Brings the parser closer to the upstream behavior for non-ASCII literals and JSON whitespace.

## [Hour 8] Differential fuzzing

**Context:** The fuzz target previously only checked that the Rust parser did not panic. The submission needs a genuine differential harness.
**Decision:** Route the fuzz target through the same comparison helper used by the CLI so it compares Rust output against the compiled C reference binary on each input.
**Rationale:** This turns the fuzz harness into a true differential tool and makes it useful for finding real divergences.
**Equivalence impact:** Provides a real comparison path for the forked fuzzing workflow.

## [Hour 10] Benchmark approach

**Context:** The repository already had a manual `std::time::Instant` benchmark harness, while `criterion` was declared as a dev-dependency but not wired up.
**Decision:** Keep the manual timing harness for now rather than refactoring to criterion, because the submission already has an end-to-end timing loop and the parser correctness fixes took priority.
**Rationale:** This avoids introducing more moving parts into the submission while retaining a reproducible, comparative performance report path.
**Equivalence impact:** Keeps the benchmark output simple and stable for the report without affecting core parser behavior.

## [Hour 12] Differential fuzz harness

**Context:** The submission required a real differential fuzz target rather than a simple panic-only parse smoke test.
**Decision:** Implement the fuzz target so it feeds each input through the same comparison helper used by the CLI and panics on any real Rust-vs-C divergence.
**Rationale:** This provides a genuine differential harness and makes the behavior easier to explain in the submission package.
**Equivalence impact:** Gives the repository a meaningful fuzzing path aligned with the original differential-testing requirement.

## [Hour 14] Kickoff hash

**Context:** The submission package benefits from a clear, self-generated snapshot hash of the original upstream test suite at the time the work began.
**Decision:** Record the SHA-256 hash of the upstream cJSON test-suite files in [kickoff_hash.txt](kickoff_hash.txt).
**Rationale:** This is easy for reviewers to verify and demonstrates that the reference snapshot was captured independently at kickoff.
**Equivalence impact:** Adds traceability without changing parser behavior.
