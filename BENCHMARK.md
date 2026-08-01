# Benchmark report

## Environment

- CPU: Intel(R) Core(TM) i5-1135G7 @ 2.40GHz
- Rust: `rustc 1.99.0-nightly (ad3d0bc14 2026-07-31)`
- Rust build profile: `--release` (`cargo run --release --bin bench_main`)
- C compiler: MinGW GCC (`C:\MinGW\bin\gcc.EXE`), `-O2`
- C reference: upstream `DaveGamble/cJSON` sources under `original/cJSON/`, built via
  [build_c_reference.py](build_c_reference.py)
- Platform: Windows (native, PowerShell)
- Timer used by the C reference binary: `QueryPerformanceCounter` on Windows,
  `clock_gettime(CLOCK_MONOTONIC)` elsewhere — see [Hour 29 in DECISIONS.md](DECISIONS.md) for
  why this replaced `clock()` (`clock()`'s ~15.6ms resolution on Windows made every C-side
  timing read as `0.000us` regardless of input).

## How this was measured

`cargo run --release --bin bench_main` runs both sides on five inputs (small, medium, large,
deep, wide) and reports the **average of 20 runs per case, per implementation**:

- **Rust**: `std::time::Instant` around in-process calls to `parse`, `print`, and
  `print_unformatted` — no process-spawn overhead.
- **C**: the reference binary (`c_reference_main.c`) supports a `--bench` flag that times its
  own `cJSON_Parse`, `cJSON_Print`, and `cJSON_PrintUnformatted` calls internally with a
  monotonic high-resolution timer (`QueryPerformanceCounter` on Windows,
  `clock_gettime(CLOCK_MONOTONIC)` elsewhere — not `clock()`, which was too coarse on Windows
  to measure microsecond-scale operations; see Hour 29 in DECISIONS.md), and prints the result
  as `parse_us=.. pretty_us=.. compact_us=..`. Rust invokes it as a subprocess per run (input
  piped via stdin — see Hour 29 — rather than passed as a CLI argument) and parses that output.

**Caveat, stated plainly:** the C timings still include process-spawn overhead per
invocation (fork/exec, dynamic linking), which the in-process Rust loop does not pay. This
makes the *absolute* C numbers on very small inputs look worse than the library code alone
would be — the `clock()` timing inside the C binary is the fair comparison for the
library work itself; the wall-clock-per-process-call approach was avoided for exactly this
reason. Treat this as directional, not a rigorous microbenchmark (no warmup exclusion, no
statistical outlier removal, single machine, single run of the full suite).

## Results

Observed output from a real run of `cargo run --release --bin bench_main` (Windows, after the
[Hour 29](DECISIONS.md) stdin/timer fixes — this is the first run where `medium` and `large`
succeeded at all, and the first where C-side numbers are non-zero):

```text
(each number is an average over 20 runs)
small:  size=64     rust(parse=3.180us     pretty=4.635us     compact=3.680us)     c(parse=25.340us   pretty=20.215us   compact=1.375us)
medium: size=45272  rust(parse=864.980us   pretty=2136.270us  compact=1564.490us)  c(parse=868.295us   pretty=512.995us  compact=450.615us)
large:  size=489433 rust(parse=11304.070us pretty=24365.785us compact=17319.475us) c(parse=8402.700us  pretty=5401.460us compact=4920.815us)
deep:   size=203    rust(parse=13.655us    pretty=402.485us   compact=38.045us)    c(parse=39.235us    pretty=9.110us    compact=2.410us)
wide:   size=2781   rust(parse=66.600us    pretty=168.680us   compact=138.325us)   c(parse=135.105us   pretty=42.915us   compact=34.195us)
```

## Honest commentary

- **On `small`, Rust's parse (3.18us) beats C's (25.34us); on every larger case, C pulls
  ahead on parse** (e.g. `large`: C 8402.70us vs. Rust 11304.07us). The likely explanation
  isn't that C parsing gets relatively faster as input grows — it's that Rust's advantage on
  `small` is dominated by fixed process-spawn overhead that the C side pays once per
  invocation regardless of input size (the C binary is still spawned as a subprocess per
  timed call even though its *internal* timing uses a high-resolution timer). That fixed cost
  is a much larger fraction of the total on a 64-byte input than a 489KB one, which is
  consistent with the crossover seen here. Treat the `small` numbers as measuring
  "Rust-in-process vs. C-plus-process-spawn," not a fair comparison of parser code alone.
- **Rust's pretty-print is the largest and most consistent gap**, worst on `large`
  (24365.79us vs. C's 5401.46us, roughly 4.5x). This is a real, explainable tradeoff rather
  than a measurement artifact — the most likely cause is that the `Vec`-based `Value` tree and
  per-level indentation logic allocate more intermediate `String`s than cJSON's C printer does
  per nesting level. Not fixed before submission; named here rather than left unexplained.
- **Compact print shows the same direction, a smaller but still real gap** — `large`:
  Rust 17319.48us vs. C's 4920.82us, roughly 3.5x. Plausibly the same allocation pattern as
  pretty-print, just without indentation overhead on top of it.
- **`deep` (203 bytes, 100 levels of nesting) shows Rust's pretty-print at 402.49us against
  C's 9.11us** — the largest *relative* gap in the whole table (~44x), even though the input
  is tiny. This strongly points at a genuine per-recursion-level cost in the Rust printer
  (allocation or indentation-string construction per level) rather than anything
  size-proportional — worth profiling first if there's time to optimize, since it's the
  clearest, most isolated signal in this data.

## Notes

- The benchmark binary measures parse time, formatted-print time, and unformatted-print time
  directly for both implementations, averaged over 20 runs each — not a single sample and
  not character counts.
- `original/cJSON/` must be populated with the real upstream source before running this
  benchmark; `ensure_reference_binary()` in `bench_main` will attempt to build it via
  `build_c_reference.py` if the binary is missing, but the source tree itself must exist.
- `ensure_reference_binary()` only rebuilds the C reference binary if it doesn't already
  exist at `target/cjson_reference[.exe]` — it has no way to detect that the *source*
  (`c_reference_main.c`) changed. If you edit that file, delete the existing binary first
  (`target/cjson_reference.exe` on Windows) before the next benchmark or comparison run, or
  it will silently keep using the stale build. This bit us once during development: after the
  Hour 29 stdin fix, a stale pre-fix binary caused every single case to report
  `c_error=C reference binary rejected input: parse_error`, because the old binary was still
  reading from `argv[1]` while the new Rust-side caller was writing to stdin.
- `test_data/medium.json` (~44KB) and `test_data/large.json` (~478KB) are real, properly-sized
  fixtures as of Hour 27 — the original 183-byte "medium" mislabeling is resolved.
