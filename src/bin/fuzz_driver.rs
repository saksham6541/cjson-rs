//! A stopgap differential fuzz driver, NOT a replacement for `cargo fuzz run
//! differential`.
//!
//! `cargo fuzz` requires a nightly Rust toolchain. In an environment without
//! nightly available, this binary still exercises the exact same
//! `compare_against_c_bytes` entry point the real libFuzzer target
//! (`fuzz/fuzz_targets/differential.rs`) uses, via simple corpus-seeded
//! mutation instead of libFuzzer's coverage-guided engine. It is weaker at
//! finding deep bugs (no coverage feedback) but every run it reports is real:
//! this binary actually executes the Rust parser and the real compiled C
//! reference binary on every generated input.
//!
//! Run `cargo fuzz run differential` on a machine with a nightly toolchain
//! before final submission -- that is the real, scored fuzzing harness.

use std::{
    fs, io,
    path::Path,
    time::{Duration, Instant},
};

use hackathon::compare_against_c_bytes;

// Minimal xorshift PRNG -- avoids pulling in a `rand` dependency for a
// stopgap tool.
struct Rng(u64);
impl Rng {
    fn new(seed: u64) -> Self {
        Rng(seed | 1)
    }
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    fn next_usize(&mut self, bound: usize) -> usize {
        if bound == 0 {
            0
        } else {
            (self.next_u64() as usize) % bound
        }
    }
}

fn load_corpus(dir: &Path) -> Vec<Vec<u8>> {
    let mut seeds = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Ok(bytes) = fs::read(&path) {
                    if !bytes.is_empty() {
                        seeds.push(bytes);
                    }
                }
            }
        }
    }
    seeds
}

/// Applies 1-3 small random mutations to a seed: byte flip, byte insert, or
/// byte delete. Deliberately dumb -- this is what a coverage-guided fuzzer
/// starts from, not a substitute for one.
fn mutate(rng: &mut Rng, seed: &[u8]) -> Vec<u8> {
    let mut data = seed.to_vec();
    let mutations = 1 + rng.next_usize(3);
    for _ in 0..mutations {
        if data.is_empty() {
            data.push(rng.next_u64() as u8);
            continue;
        }
        match rng.next_usize(3) {
            0 => {
                // flip a random byte
                let i = rng.next_usize(data.len());
                data[i] = rng.next_u64() as u8;
            }
            1 => {
                // insert a random byte at a random position
                let i = rng.next_usize(data.len() + 1);
                data.insert(i, rng.next_u64() as u8);
            }
            _ => {
                // delete a random byte
                let i = rng.next_usize(data.len());
                data.remove(i);
            }
        }
    }
    data
}

fn main() -> io::Result<()> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let corpus_dir = Path::new(manifest_dir).join("fuzz/corpus/differential");
    let seeds = load_corpus(&corpus_dir);
    if seeds.is_empty() {
        eprintln!("no corpus seeds found at {}", corpus_dir.display());
        std::process::exit(1);
    }

    let iterations: usize = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(2000);

    let mut rng = Rng::new(0x5EED_u64);
    let start = Instant::now();

    let mut mismatches: Vec<(Vec<u8>, String)> = Vec::new();
    let mut both_rejected = 0usize;
    let mut both_accepted_matched = 0usize;
    let mut harness_errors = 0usize; // e.g. reference binary itself failed to run

    for i in 0..iterations {
        let seed = &seeds[rng.next_usize(seeds.len())];
        let input = mutate(&mut rng, seed);

        match compare_against_c_bytes(&input) {
            Ok(msg) if msg.starts_with("both rejected") => both_rejected += 1,
            Ok(_) => both_accepted_matched += 1,
            Err(reason) => {
                if reason.starts_with("failed to run C reference binary")
                    || reason.starts_with("failed to build C reference binary")
                {
                    harness_errors += 1;
                    if harness_errors <= 3 {
                        eprintln!("harness error on iteration {i}: {reason}");
                    }
                } else {
                    mismatches.push((input, reason));
                }
            }
        }

        if i % 500 == 0 && i > 0 {
            eprintln!("...{i}/{iterations} iterations, {} mismatches so far", mismatches.len());
        }
    }

    let elapsed: Duration = start.elapsed();

    println!("=== fuzz_driver summary ===");
    println!("iterations executed: {iterations}");
    println!("elapsed: {:.2}s", elapsed.as_secs_f64());
    println!(
        "throughput: {:.1} inputs/sec",
        iterations as f64 / elapsed.as_secs_f64().max(0.001)
    );
    println!("corpus seeds used: {}", seeds.len());
    println!("both rejected (no divergence): {both_rejected}");
    println!("both accepted and matched (no divergence): {both_accepted_matched}");
    println!("harness errors (binary missing/failed to run, not a real divergence): {harness_errors}");
    println!("REAL DIVERGENCES FOUND: {}", mismatches.len());

    if !mismatches.is_empty() {
        let out_dir = Path::new(manifest_dir).join("fuzz/found_divergences");
        fs::create_dir_all(&out_dir)?;
        for (idx, (input, reason)) in mismatches.iter().enumerate() {
            let path = out_dir.join(format!("divergence_{idx}"));
            fs::write(&path, input)?;
            println!("  [{idx}] saved to {} -- {reason}", path.display());
        }
    }

    Ok(())
}
