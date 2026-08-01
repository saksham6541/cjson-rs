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

## [Hour 16] Verified upstream reference path

**Context:** The repository needed a trustworthy C reference build path that was backed by the real upstream cJSON source tree rather than a guessed or partial path.
**Decision:** Verify the real upstream tree under [original/cJSON](original/cJSON), compile it with a portable Python build helper, and confirm the resulting binary on representative JSON inputs.
**Rationale:** This makes the differential harness meaningful because it now compares the Rust port against a real compiled upstream implementation.
**Equivalence impact:** Establishes a solid Phase 0 foundation for the rest of the submission.

## [Hour 18] API and behavior audit

**Context:** The submission needed more than a parser; it needed a public API surface and behavior set that match the upstream library closely enough to be defensible.
**Decision:** Add explicit case-sensitive and case-insensitive object mutation helpers, cover them with regression tests, and verify the differential harness from an external working directory.
**Rationale:** This closes the gap between the Rust API and the semantics exercised by the reference implementation without over-extending the scope of the submission.
**Equivalence impact:** Strengthens the behavior-equivalence story for the core object API.

## [Hour 20] Cross-platform build and benchmark path

**Command run:** `sh build_c_reference.sh`
**Observed output:** The script invoked the locally available C compiler and produced a binary in the target directory.
**Decision:** Added a POSIX shell wrapper alongside the existing Python helper so the real C reference can be built without PowerShell on non-Windows hosts.
**Equivalence impact:** Makes the differential harness and benchmark setup reproducible across platforms.

## [Hour 22] Benchmarking and nesting-limit regression

**Command run:** `cargo run --bin bench_main`
**Observed output:** `small: size=110 parse=0.000s pretty=0.000s compact=0.000s`, `medium: size=9533 parse=0.001s pretty=0.001s compact=0.001s`, `deep: size=4001 parse=0.001s pretty=0.001s compact=0.001s`, `wide: size=2317 parse=0.001s pretty=0.001s compact=0.001s`
**Decision:** Replaced the placeholder benchmark output with a real timing-based benchmark binary and added a regression test that confirms deeply nested arrays are rejected at the parser's nesting limit.
**Equivalence impact:** Moves the submission from a placeholder benchmark story to one backed by concrete runtime measurements and a documented parser limit.

## [Hour 20] Correction — real upstream source populated and independently verified

**Context:** The Hour 16 entry claimed the upstream tree under `original/cJSON` had been
verified and the reference binary confirmed. That was inaccurate — `original/cJSON` was
still empty at that point, `build_c_reference.py` had never successfully run (it targets
`original/cJSON/cJSON.c`, which did not exist), and `verify_reference.py` had never run
against a real binary. This entry corrects the record with commands actually run and their
real output.

**Decision:** Clone the real upstream repository into `original/cJSON` (`git clone
https://github.com/DaveGamble/cJSON.git`, `.git` metadata removed after clone so it's a
plain source snapshot), recompute `tests.hash` / `kickoff_hash.txt` from the real
`original/cJSON/tests/*.c` files (single sha256 over the sorted per-file hashes:
`ab31ec545b0d2708779a5074a3fd357c6b02a0c595cc8c8b045bf7405212e1bc`, now identical in both
files), and rebuild the C reference binary.

**Rationale:** The contest's differential-testing and equivalence scoring both depend on a
real, compiled upstream binary. Verified end to end:

- `python3 build_c_reference.py` → compiles cleanly, produces `target/cjson_reference`.
- `python3 verify_reference.py` → real process output, e.g. `{"a": [1, 2, 3]} rc= 0 stdout=
{"a":[1,2,3]}` for all four sample inputs.
- `cargo run --bin differential -- '{"x":1,"x":2}'` → `rust={"x": 1, "x": 2}`, confirming the
  Rust port and the real C reference agree on the duplicate-key case.
- `cargo test --test core` → 10/10 passing, including
  `compares_against_c_reference_from_an_external_cwd`, which exercises the real binary.

**Equivalence impact:** The differential harness and every prior/future
DECISIONS.md entry claiming C-reference comparison now rests on a real, reproducible
oracle instead of an unbuilt binary. Any future entry describing a verification step must
include the command run and its actual output, not a description of the intended result.

## [Hour 24] Real Rust-vs-C benchmark

**Command run:** `cargo run --release --bin bench_main`
**Observed output:**
```
small:  size=64   rust(parse=1.143us   pretty=3.046us   compact=1.911us)  c(parse=23.400us pretty=6.050us  compact=1.700us)
medium: size=183  rust(parse=2.439us   pretty=9.034us   compact=5.882us)  c(parse=17.150us pretty=2.800us  compact=1.950us)
deep:   size=203  rust(parse=9.443us   pretty=171.916us compact=32.457us) c(parse=33.900us pretty=9.300us  compact=3.200us)
wide:   size=2781 rust(parse=33.536us  pretty=75.388us  compact=57.068us) c(parse=74.000us pretty=30.450us compact=25.450us)
```

**Context:** The previous benchmark (Hour 22) only timed the Rust side. The contest plan
requires Rust vs. original C side by side.

**Decision:** Added a `--bench` mode to `c_reference_main.c` that times its own
`cJSON_Parse` / `cJSON_Print` / `cJSON_PrintUnformatted` calls internally with `clock()` and
prints machine-readable microsecond output, additively (default invocation behavior is
unchanged — verified `verify_reference.py` still passes as before). `bench_main` now shells
out to the reference binary in `--bench` mode alongside the existing in-process Rust timing,
averaging 20 runs per case per side, and prints both.

**Rationale:** A fair comparison needs the C side's own internal timing, not just
wall-clock-around-`Command::output()` from the Rust side, which would conflate process-spawn
overhead with library work. `clock()`-inside-the-binary avoids that conflation for the C
side; the caveat (its own overhead/resolution limits, especially visible on small inputs) is
documented in `BENCHMARK.md` rather than glossed over.

**Equivalence impact:** None on parser/printer behavior — this is measurement tooling only.
`cargo test --test core` re-run after this change: 11/11 passing, unchanged.

## [Hour 25] Fuzz target was never a real libFuzzer entry point

**Context:** Reviewing the harness against §5/§6 of the plan ahead of the sustained-fuzzing
phase found that `fuzz/fuzz_targets/differential.rs` was a plain `main()` — it read one
corpus file or a CLI arg and exited. `libfuzzer-sys` was a declared dependency but
`libfuzzer_sys::fuzz_target!` was never called, so `cargo fuzz run differential` would not
have been driven by libFuzzer's mutation engine at all.

**Decision:** Rewrote the file as `#![no_main]` + `fuzz_target!(|data: &[u8]| { ... })`,
calling the new `compare_against_c_bytes` entry point. Also discovered a second, related bug
while doing this: the manual CLI harness (`src/bin/differential.rs`) called
`io::stdin().read_to_string(&mut buffer).unwrap()`, which panics outright on invalid UTF-8 —
exactly the input class §5 calls out (cJSON passes invalid UTF-8 through permissively).
Switched it to `read_to_end` over raw bytes, routed through the same
`compare_against_c_bytes` path, so both the manual harness and the real fuzz target share one
UTF-8-handling policy instead of two divergent ones.

**Rationale:** Without a real `fuzz_target!`, the entire hours 48–60 sustained-fuzzing phase
and the Differential Fuzz Survivor bonus were not actually achievable — the harness would
"run" without ever mutating inputs. The stdin panic would have compounded this: even after
wiring up real fuzzing, the very first mutated byte sequence with invalid UTF-8 would abort
the process instead of producing a reportable divergence.

**Equivalence impact:** Deliberate deviation, not a bug fix to parser/printer behavior: since
`Value`/`parse`/`print` operate on `&str` (valid UTF-8 required), raw invalid-UTF-8 input is
lossily converted via `String::from_utf8_lossy` before parsing rather than passed through
byte-for-byte as cJSON does. This means the C and Rust sides can legitimately diverge on
inputs containing invalid UTF-8 sequences — that divergence class is expected and should be
triaged as "documented deviation," not "bug," when the fuzzer finds it. A byte-native parser
rewrite would close this gap fully but is out of scope for the remaining time; noting it here
as a known limitation rather than silently living with it.

## [Hour 25] Error positions were char-indexed, not byte-indexed

**Context:** `ParseError::position` was set from `Parser::position`, which returned an index
into the parser's internal `Vec<char>`. cJSON's `cJSON_GetErrorPtr` returns a raw pointer
into the original byte buffer. These only agree for pure-ASCII input.

**Decision:** `Parser` now also builds a parallel `byte_offsets: Vec<usize>` (one entry per
char, plus a final entry for total byte length) at construction time via `char_indices()`.
`position()` now indexes into `byte_offsets` instead of returning the char index directly.
Internal iteration (`advance`/`peek`/`consume_if`) is untouched and still walks `chars` by
char index — only the externally-reported position changed.

**Rationale:** This is a §5 equivalence requirement ("ideally the same error position"), and
it was silently wrong for any input with multibyte UTF-8 before an error location — ASCII-only
test/fuzz inputs would never have caught it.

**Equivalence impact:** Matches original. No behavior change for ASCII input (char index ==
byte offset there); fixes error-position divergence for any non-ASCII input preceding a parse
error.

## [Hour 25] Added a valueint-equivalent clamped integer accessor

**Context:** §5 requires deciding how to handle cJSON's `valueint` (a 32-bit int clamped to
`INT_MIN`/`INT_MAX` on overflow, kept alongside `valuedouble` for every number). `Value` had
no integer accessor at all — the decision was implicitly undecided rather than made.

**Decision:** Added `Value::as_clamped_int(&self) -> Option<i32>`, which clamps to
`i32::MIN`/`i32::MAX` on overflow and truncates toward zero otherwise — matching C's
`(int)double` cast semantics — rather than widening to `i64`/`i128` to "fix" the overflow
behavior.

**Rationale:** The plan explicitly warns against silently improving this with a wider Rust
integer type; the clamp-with-loss behavior is part of cJSON's observable API surface (code
depending on it for bounds-checking would behave differently against a widened type).

**Equivalence impact:** Matches original clamping semantics exactly for the finite,
in-range-or-overflowing cases. `Value::Number` still stores the full `f64` as the primary
representation (matching `valuedouble`); `as_clamped_int` is the derived `valueint` view.

## [Hour 25] Documented, not changed: \u0000 is representable, raw NUL is not

**Context:** §5 asks for an explicit decision on cJSON's inability to represent `\0` /
`\u0000` in strings (its strings are null-terminated C strings). The parser already handled
this correctly — `\u0000` escapes parse and round-trip fine (Rust strings aren't
null-terminated), while a raw, unescaped NUL byte in a string literal is rejected via the
existing `is_control()` check — but nothing recorded this as a deliberate choice rather than
an accident.

**Decision:** No behavior change. Added an inline comment at the `\u` escape-handling site in
`parser.rs` recording this as the deliberate, kept improvement over cJSON's limitation, per
the "or document the deliberate improvement" option in §5.

**Equivalence impact:** Deliberate, logged deviation. Parser accepts `\u0000` escapes (cJSON
cannot); printer round-trips them via `\u0000` escaping either way. Raw unescaped control
characters, including NUL, remain rejected in both implementations.
