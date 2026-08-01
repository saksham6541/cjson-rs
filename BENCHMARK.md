# Benchmark report

> **⚠️ Numbers below are stale.** They were captured against the old `test_data/large.json`
> fixture, which was only 183 bytes despite being labeled "medium." As of [Hour 27 in
> DECISIONS.md](DECISIONS.md), `test_data/medium.json` (~44KB) and a properly resized
> `test_data/large.json` (~478KB) now exist, and both benchmark binaries have been updated to
> use them, including a new `large` case. **Re-run `cargo run --release --bin bench_main`
> before treating this report as final** — the numbers below have not been regenerated against
> the corrected fixtures yet.

## Environment

- Rust: verified against rustc/cargo (edition-2021-compatible build; see note below)
- C compiler: GCC 13.3.0, `-O2`
- C reference: upstream `DaveGamble/cJSON` sources under `original/cJSON/`, built via
  [build_c_reference.py](build_c_reference.py) / [build_c_reference.sh](build_c_reference.sh)
- Platform: Linux x86_64

## How this was measured

`cargo run --release --bin bench_main` runs both sides on four inputs (small, medium, deep,
wide) and reports the **average of 20 runs per case, per implementation**:

- **Rust**: `std::time::Instant` around in-process calls to `parse`, `print`, and
  `print_unformatted` — no process-spawn overhead.
- **C**: the reference binary (`c_reference_main.c`) now supports a `--bench` flag that times
  its own `cJSON_Parse`, `cJSON_Print`, and `cJSON_PrintUnformatted` calls internally with
  `clock()`, and prints the result as `parse_us=.. pretty_us=.. compact_us=..`. Rust invokes
  it as a subprocess per run and parses that output.

**Caveat, stated plainly:** the C timings still include process-spawn overhead per
invocation (fork/exec, dynamic linking), which the in-process Rust loop does not pay. This
makes the *absolute* C numbers on very small inputs look worse than the library code alone
would be — the `clock()` timing inside the C binary is the fair comparison for the
library work itself; the wall-clock-per-process-call approach was avoided for exactly this
reason. Treat this as directional, not a rigorous microbenchmark (no warmup exclusion, no
statistical outlier removal, single machine, single run of the full suite).

## Results

Observed output from a real run of `cargo run --release --bin bench_main`:

```text
(each number is an average over 20 runs)
small:  size=64   rust(parse=1.143us   pretty=3.046us   compact=1.911us)  c(parse=23.400us pretty=6.050us  compact=1.700us)
medium: size=183  rust(parse=2.439us   pretty=9.034us   compact=5.882us)  c(parse=17.150us pretty=2.800us  compact=1.950us)
deep:   size=203  rust(parse=9.443us   pretty=171.916us compact=32.457us) c(parse=33.900us pretty=9.300us  compact=3.200us)
wide:   size=2781 rust(parse=33.536us  pretty=75.388us  compact=57.068us) c(parse=74.000us pretty=30.450us compact=25.450us)
```

("medium" here is the `test_data/large.json` fixture, which is 183 bytes despite the name —
worth renaming the fixture or swapping in an actual ~100KB file if a true "medium" size
category is wanted for the final submission; see Notes.)

## Honest commentary

- **C's `clock()`-measured parse is consistently slower than Rust's `Instant`-measured parse**
  in this run, which on its face looks surprising for hand-rolled C vs. a safety-checked Rust
  parser. This is very likely an artifact of `clock()` resolution/overhead on very short
  operations (single-digit-to-tens of microseconds) rather than a real performance
  difference — `clock()` measures CPU time with coarser granularity than `Instant`, and the
  first call in a freshly-exec'd process can include page-fault/cache-cold effects that
  `Instant`'s in-process loop avoids entirely after the first iteration. This should be
  called out as a measurement-methodology caveat, not presented as "Rust beats C at
  parsing" without qualification.
- **Rust's pretty-print on the deeply nested case (171.9us) is the clearest case where Rust
  is genuinely slower** — nested `Vec`-of-`Value` indentation likely allocates more
  intermediate `String`s than cJSON's C printer does per level. This is a legitimate,
  explainable tradeoff (safety/ergonomics vs. allocation count) worth stating as such rather
  than avoiding.
- On the "wide" case (200 keys), Rust's compact print (57us) is noticeably slower than C's
  (25us) — plausibly `Vec<(String, Value)>`-based object lookup/iteration overhead vs. C's
  linked-list walk; worth profiling if there's time before submission, but not blocking.

## Notes

- The benchmark binary measures parse time, formatted-print time, and unformatted-print time
  directly for both implementations, averaged over 20 runs each — not a single sample and
  not character counts.
- `original/cJSON/` must be populated with the real upstream source before running this
  benchmark; `ensure_reference_binary()` in `bench_main` will attempt to build it via
  `build_c_reference.py` if the binary is missing, but the source tree itself must exist.
- Before the final submission: run this on the actual target machine (not this sandbox), with
  a properly-sized "medium" (~100KB) and "large" (~10MB) fixture per the contest plan — the
  current `test_data/large.json` is 183 bytes and does not represent a real medium-size case.
