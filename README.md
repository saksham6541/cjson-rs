# hackathon

A Rust reimplementation of a core subset of cJSON focused on JSON parsing, formatting, differential testing, and benchmark reporting.

## Build

```bash
cargo build
```

## Test

```bash
cargo test --test core -- --nocapture
```

## Differential harness

Build the C reference from the real upstream tree first:

```bash
python3 build_c_reference.py
# or, on POSIX shells:
sh build_c_reference.sh
```

Then run the Rust-vs-C comparison:

```bash
cargo run --bin differential -- '{"a": [1, 2, 3]}'
```

The build helpers use the upstream sources under [original/cJSON](original/cJSON) and produce the reference binary in [target](target).

## Fuzzing

The differential harness is wired to the shared comparison helper in [src/compare.rs](src/compare.rs) and runs against a small corpus under [fuzz/corpus/differential](fuzz/corpus/differential). Any parse/reject mismatch or semantic output mismatch is treated as a finding and causes the harness to panic.

Run it with:

```bash
cargo run --manifest-path fuzz/Cargo.toml --bin differential
```

## Benchmarking

A timing-oriented benchmark binary is available via:

```bash
cargo run --bin bench_main
```

The current benchmark output is recorded in [BENCHMARK.md](BENCHMARK.md) and uses the real upstream C reference build path as the comparison oracle.

The kickoff hash for the original upstream test-suite snapshot used for the submission is stored in [kickoff_hash.txt](kickoff_hash.txt).
