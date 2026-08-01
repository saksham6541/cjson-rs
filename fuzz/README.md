# cjson-rs Differential Fuzzing

This fuzz target exercises the Rust cJSON parser against `serde_json` as the reference implementation.

## Setup

```bash
cargo install cargo-fuzz
``` 

## Run

```bash
cargo fuzz run fuzz_target -- -max_total_time=28800
```

## Behavior

- Generates arbitrary string inputs using the `arbitrary` crate.
- Parses the input with `hackathon::from_str`.
- Parses the same input with `serde_json`.
- If both sides succeed, compares ASTs for equality.
- If both sides fail, compares error messages for exact equivalence.
- Logs any discrepancy to `fuzz.log`.
- Tracks total runs, successful comparisons, and failures.

## Goal

- Run for at least 8 hours.
- Achieve zero discrepancies for a valid differential fuzzing proof.
- Claim the +5 Differential Fuzz Survivor bonus.
