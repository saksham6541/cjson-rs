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

Build the C reference from the real upstream tree first:

```powershell
python .\build_c_reference.py
```

Then run the Rust-vs-C comparison:

```powershell
cargo run --bin differential -- '{"a": [1, 2, 3]}'
```

The build helper uses the upstream sources under [original/cJSON](original/cJSON) and produces a reference binary in [target](target).

## Fuzzing

The differential harness is wired to the shared comparison helper in [src/compare.rs](src/compare.rs) and runs against a small corpus under [fuzz/corpus/differential](fuzz/corpus/differential). Any parse/reject mismatch or semantic output mismatch is treated as a finding and causes the harness to panic.

Run it with:

```powershell
cargo run --manifest-path fuzz/Cargo.toml --bin differential
```

The kickoff hash for the original upstream test-suite snapshot used for the submission is stored in [kickoff_hash.txt](kickoff_hash.txt).
