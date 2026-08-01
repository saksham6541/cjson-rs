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

The differential fuzz target is seeded from the upstream cJSON corpus under [fuzz/corpus/differential](fuzz/corpus/differential).
