# Benchmark report

## Environment

- Rust: stable toolchain used by `cargo test` / `cargo build`
- C reference: upstream cJSON sources built with GCC via `build_c_reference.ps1`
- Platform: Windows workspace with MinGW GCC available

## Results

The benchmark and reference workflow were verified with fresh runs:

- `cargo test --test core --target-dir .\build-artifacts -- --nocapture`: 3 tests passed, 0 failed
- `powershell -ExecutionPolicy Bypass -File .\build_c_reference.ps1`: C reference binary built successfully
- `cargo run --bin bench_main --target-dir .\build-artifacts`: benchmark completed successfully

Observed benchmark output:

- `small`: pretty_chars=108 compact_chars=74
- `medium`: pretty_chars=355 compact_chars=201

## Notes

- The current implementation covers the core parser, printer, and tree-style API surface for the contest slice.
- The benchmark harness is now runnable from the repository and can be expanded with larger adversarial inputs or a fuller C-vs-Rust timing matrix later.
