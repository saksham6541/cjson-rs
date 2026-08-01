use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
    time::Instant,
};

use hackathon::{parse, print, print_unformatted};

const ITERATIONS: u32 = 20;

fn load_fixture(path: &str) -> String {
    fs::read_to_string(Path::new(path)).unwrap()
}

struct Timings {
    parse_us: f64,
    pretty_us: f64,
    compact_us: f64,
}

fn rust_timings(input: &str) -> Result<Timings, String> {
    // Average over several iterations: a single `Instant` sample on tiny
    // inputs is dominated by noise, not by the work being measured.
    let mut parse_total = 0.0;
    let mut pretty_total = 0.0;
    let mut compact_total = 0.0;

    for _ in 0..ITERATIONS {
        let parse_start = Instant::now();
        let value = parse(input).map_err(|e| e.to_string())?;
        parse_total += parse_start.elapsed().as_secs_f64() * 1_000_000.0;

        let pretty_start = Instant::now();
        let _pretty = print(&value);
        pretty_total += pretty_start.elapsed().as_secs_f64() * 1_000_000.0;

        let compact_start = Instant::now();
        let _compact = print_unformatted(&value);
        compact_total += compact_start.elapsed().as_secs_f64() * 1_000_000.0;
    }

    Ok(Timings {
        parse_us: parse_total / ITERATIONS as f64,
        pretty_us: pretty_total / ITERATIONS as f64,
        compact_us: compact_total / ITERATIONS as f64,
    })
}

fn reference_binary_path() -> PathBuf {
    if let Ok(path) = env::var("HACKATHON_REFERENCE_BINARY") {
        return PathBuf::from(path);
    }
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    if cfg!(windows) {
        manifest_dir.join("target").join("cjson_reference.exe")
    } else {
        manifest_dir.join("target").join("cjson_reference")
    }
}

fn ensure_reference_binary(reference_binary: &PathBuf) -> Result<(), String> {
    if reference_binary.exists() {
        return Ok(());
    }
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let python = env::var("PYTHON").unwrap_or_else(|_| "python3".to_string());
    let build_script = manifest_dir.join("build_c_reference.py");
    let status = Command::new(&python)
        .arg(&build_script)
        .current_dir(&manifest_dir)
        .status()
        .map_err(|e| format!("failed to launch reference build script: {e}"))?;
    if !status.success() || !reference_binary.exists() {
        return Err(format!(
            "failed to build C reference binary via {}",
            build_script.display()
        ));
    }
    Ok(())
}

/// Runs the C reference binary in `--bench` mode, which times its own
/// parse/print calls internally with `clock()` and prints
/// `parse_us=.. pretty_us=.. compact_us=..`. We average several process
/// invocations; this still includes process-spawn overhead per call
/// (unlike the Rust side, which loops in-process), so treat the C
/// parse/print *proportions* as more reliable than the absolute numbers
/// on very small inputs. This is called out in BENCHMARK.md.
fn c_timings(input: &str) -> Result<Timings, String> {
    let reference_binary = reference_binary_path();
    ensure_reference_binary(&reference_binary)?;

    let mut parse_total = 0.0;
    let mut pretty_total = 0.0;
    let mut compact_total = 0.0;

    for _ in 0..ITERATIONS {
        let output = Command::new(&reference_binary)
            .arg(input)
            .arg("--bench")
            .output()
            .map_err(|e| format!("failed to run C reference binary: {e}"))?;

        if !output.status.success() {
            return Err(format!(
                "C reference binary rejected input: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut parse_us = None;
        let mut pretty_us = None;
        let mut compact_us = None;
        for field in stdout.split_whitespace() {
            if let Some(v) = field.strip_prefix("parse_us=") {
                parse_us = v.parse::<f64>().ok();
            } else if let Some(v) = field.strip_prefix("pretty_us=") {
                pretty_us = v.parse::<f64>().ok();
            } else if let Some(v) = field.strip_prefix("compact_us=") {
                compact_us = v.parse::<f64>().ok();
            }
        }
        parse_total += parse_us.ok_or("missing parse_us in C reference output")?;
        pretty_total += pretty_us.ok_or("missing pretty_us in C reference output")?;
        compact_total += compact_us.ok_or("missing compact_us in C reference output")?;
    }

    Ok(Timings {
        parse_us: parse_total / ITERATIONS as f64,
        pretty_us: pretty_total / ITERATIONS as f64,
        compact_us: compact_total / ITERATIONS as f64,
    })
}

fn benchmark_case(name: &str, input: &str) {
    let size = input.len();

    let rust = match rust_timings(input) {
        Ok(t) => t,
        Err(err) => {
            println!("{name}: size={size} rust_rejected={err}");
            return;
        }
    };

    match c_timings(input) {
        Ok(c) => {
            println!(
                "{name}: size={size} \
                 rust(parse={:.3}us pretty={:.3}us compact={:.3}us) \
                 c(parse={:.3}us pretty={:.3}us compact={:.3}us)",
                rust.parse_us,
                rust.pretty_us,
                rust.compact_us,
                c.parse_us,
                c.pretty_us,
                c.compact_us,
            );
        }
        Err(err) => {
            println!(
                "{name}: size={size} \
                 rust(parse={:.3}us pretty={:.3}us compact={:.3}us) c_error={err}",
                rust.parse_us, rust.pretty_us, rust.compact_us,
            );
        }
    }
}

fn main() {
    let small = r#"{"name":"Ada","active":true,"items":[1,2,3],"meta":{"score":42}}"#;
    let medium = load_fixture("test_data/medium.json");
    let large = load_fixture("test_data/large.json");
    let deep = "[".to_string() + &"[".repeat(100) + &"1" + &"]".repeat(100) + "]";
    let wide = format!(
        "{{{}}}",
        (0..200)
            .map(|i| format!(r#""field{i}":{i}"#))
            .collect::<Vec<_>>()
            .join(",")
    );

    println!("(each number is an average over {ITERATIONS} runs)");
    for (name, input) in [
        ("small", small),
        ("medium", medium.as_str()),
        ("large", large.as_str()),
        ("deep", deep.as_str()),
        ("wide", wide.as_str()),
    ] {
        benchmark_case(name, input);
    }
}
