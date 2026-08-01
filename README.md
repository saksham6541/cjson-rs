# cjson-rs — cJSON Ported to Rust

Team VILTRUMITES: Saksham Kaushik, Saksham Mishra, Ayush Rawat

## Project Overview

This repository ports the cJSON C library into Rust with a focus on behavioral equivalence, memory safety, and zero unsafe code.

## Status

- ✅ Core parser complete
- ✅ All unit tests passing
- ✅ Compatibility harness in place
- ✅ Differential fuzzing target configured
- ✅ Benchmarks documented

## Build

```bash
cargo build --release
```

## Test

```bash
cargo test -- --nocapture
```

## Fuzz

```bash
cd fuzz
cargo install cargo-fuzz
cargo fuzz run fuzz_target -- -max_total_time=28800
```

## Benchmarks

See [BENCHMARK.md](BENCHMARK.md)

## Proof of Equivalence

- Core unit tests: see `cargo test`
- Differential fuzzing: stopgap driver found real divergences (control chars, trailing input) which were fixed to match cJSON — see DECISIONS.md [Task 2]/[Task 4]
- Real `cargo fuzz run differential` should still be run on nightly before claiming the Differential Fuzz Survivor bonus
- Test hashes: `tests.hash` / `kickoff_hash.txt`

## Team and Track

- Team: VILTRUMITES
- Track: C → Rust
- Hardware: ASUS TUF 15, Ryzen 7, 16GB DDR5, RTX 3050
- OS: Windows 11
- Repository: https://github.com/saksham6541/cjson-rs

## How to Run

```bash
cargo test -- --nocapture
cargo bench -- --verbose
```

For fuzzing:

```bash
cd fuzz
cargo fuzz run fuzz_target -- -max_total_time=28800
```

## Notes

- The parser preserves cJSON-compatible behavior for objects, arrays, strings, numbers, booleans, and null.
- The Rust port uses a safe `Value` enum and avoids unsafe memory operations entirely.

## License

Same license as cJSON (MIT).
