# Benchmark report

## Environment

- Rust: current toolchain used by `cargo test` / `cargo build`
- C reference: upstream cJSON sources built with GCC via the portable build helpers in [build_c_reference.py](build_c_reference.py) and [build_c_reference.sh](build_c_reference.sh)
- Platform: current workspace

## Results

The benchmark path was verified with a fresh run of:

- `cargo run --bin bench_main`

Observed output from the current workspace:

```text
small: size=64 parse=49.000µs pretty=47.900µs compact=16.500µs
medium: size=183 parse=33.200µs pretty=71.000µs compact=43.700µs
deep: size=203 parse=199.100µs pretty=2.574ms compact=121.900µs
wide: size=2781 parse=344.200µs pretty=476.800µs compact=406.500µs
```

## Notes

- The benchmark binary now measures parse time, formatted-print time, and unformatted-print time directly rather than reporting only character counts.
- The current implementation covers the core parser, printer, and tree-style API surface for the contest slice.
- The numbers above are from one local run and are intended as a baseline, not a statistically rigorous performance study.
