# cjson-rs — cJSON Ported to Rust

Team VILTRUMITES: Saksham Kaushik, Saksham Mishra, Ayush Rawat

## Project Overview

Safe Rust reimplementation of [cJSON](https://github.com/DaveGamble/cJSON) focused on behavioral equivalence, memory safety, and zero `unsafe` code.

## Status

- Core parser and printer complete (`#![deny(unsafe_code)]`)
- Unit + compatibility tests passing
- Differential comparison harness against compiled upstream cJSON
- Side-by-side benchmarks documented in [BENCHMARK.md](BENCHMARK.md)
- Decision log in [DECISIONS.md](DECISIONS.md)

## Build

```bash
cargo build --release
```

## Test

```bash
cargo test -- --nocapture
```

Requires a C compiler (`gcc` / `cc`) so the harness can build `original/cJSON` into `target/cjson_reference`.

## Benchmark

```bash
cargo run --release --bin bench_main
```

See [BENCHMARK.md](BENCHMARK.md) for environment, methodology, and measured numbers.

## Fuzz

Stopgap differential driver (no nightly required):

```bash
cargo run --release --bin fuzz_driver 5000
```

Real libFuzzer target (needs nightly + `cargo-fuzz`). **Run from the repo root**, not from inside `fuzz/`:

```bash
cargo install cargo-fuzz
cargo +nightly fuzz run differential -- -max_total_time=3600
```

## Proof of equivalence (what we can defend)

1. **Unit / regression tests** — `tests/core.rs` (objects, arrays, numbers, Unicode, helpers, nesting limit, control characters, trailing content).
2. **Differential harness** — `src/compare.rs` runs the same input through Rust and the compiled upstream C reference; both-accept + matching canonical output, or both-reject, counts as pass.
3. **Fuzz findings fixed to match cJSON** — raw C0 controls in strings and trailing content after a complete value were found by the differential driver and fixed (see DECISIONS.md Task 2 / Task 4).
4. **Upstream tree** — `original/cJSON/` holds the real DaveGamble/cJSON sources used by the reference binary.

We do **not** claim multi-hour zero-discrepancy libFuzzer runs until `cargo +nightly fuzz run differential` has been executed for the required duration on a nightly toolchain.

## Known limitations

- Invalid UTF-8 is lossily converted (`from_utf8_lossy`) before the Rust parser sees it; the C side receives raw bytes. Some differential mismatches can be harness artifacts, not parser bugs.
- Pretty-print is slower than cJSON on deep nesting (allocation vs. C’s lighter printer). Buffer-based printer reduces this; numbers are in BENCHMARK.md.
- Public C-style wrappers use non-idiomatic names deliberately for API familiarity.

## Team

- Track: C → Rust
- Hardware: ASUS TUF 15, Ryzen 7, 16GB DDR5, RTX 3050 (dev); benchmark machine: Intel i5-1135G7 (see BENCHMARK.md)
- Repository: https://github.com/saksham6541/cjson-rs

## License

MIT (same as cJSON).
