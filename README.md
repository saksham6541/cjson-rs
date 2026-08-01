# hackathon

A Rust reimplementation of a core subset of cJSON focused on JSON parsing, formatting, differential testing, and benchmark reporting.

## Build

```powershell
cargo build
```

## Test

```powershell
cargo test --test core -- --nocapture
```

## Differential harness

Build the C reference first:

```powershell
powershell -ExecutionPolicy Bypass -File .\build_c_reference.ps1
```

Then run the Rust-vs-C comparison:

```powershell
cargo run --bin differential -- '{"a": [1, 2, 3]}'
```

## Fuzzing

The differential fuzz target is seeded from the upstream cJSON corpus under [fuzz/corpus/differential](fuzz/corpus/differential) and compares the same input against the compiled C reference and the Rust port. Any parse/reject mismatch or output mismatch is treated as a finding and causes the fuzz target to panic.

Run it with:

```powershell
cargo fuzz run differential
```

The kickoff hash for the original upstream test-suite snapshot used for the submission is stored in [kickoff_hash.txt](kickoff_hash.txt).
