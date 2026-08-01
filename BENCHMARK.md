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
small: size=110 parse=0.000s pretty=0.000s compact=0.000s
medium: size=9533 parse=0.001s pretty=0.001s compact=0.001s
deep: size=4001 parse=0.001s pretty=0.001s compact=0.001s
wide: size=2317 parse=0.001s pretty=0.001s compact=0.001s
```

## Notes

- The benchmark binary now measures parse time, formatted-print time, and unformatted-print time directly rather than reporting only character counts.
- The current implementation covers the core parser, printer, and tree-style API surface for the contest slice.
- The numbers above are from one local run and are intended as a baseline, not a statistically rigorous performance study.
