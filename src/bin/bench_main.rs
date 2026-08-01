use std::{fs, path::Path, time::Instant};

use hackathon::{parse, print, print_unformatted};

fn load_fixture(path: &str) -> String {
    fs::read_to_string(Path::new(path)).unwrap()
}

fn benchmark_case(name: &str, input: &str) {
    let parse_start = Instant::now();
    match parse(input) {
        Ok(value) => {
            let parse_duration = parse_start.elapsed();

            let pretty_start = Instant::now();
            let _pretty = print(&value);
            let pretty_duration = pretty_start.elapsed();

            let compact_start = Instant::now();
            let _compact = print_unformatted(&value);
            let compact_duration = compact_start.elapsed();

            println!(
                "{name}: size={} parse={:.3?} pretty={:.3?} compact={:.3?}",
                input.len(),
                parse_duration,
                pretty_duration,
                compact_duration
            );
        }
        Err(err) => {
            println!("{name}: size={} rejected={err}", input.len());
        }
    }
}

fn main() {
    let small = r#"{"name":"Ada","active":true,"items":[1,2,3],"meta":{"score":42}}"#;
    let medium = load_fixture("test_data/large.json");
    let deep = "[".to_string() + &"[".repeat(100) + &"1" + &"]".repeat(100) + "]";
    let wide = format!(
        "{{{}}}",
        (0..200)
            .map(|i| format!(r#""field{i}":{i}"#))
            .collect::<Vec<_>>()
            .join(",")
    );

    for (name, input) in [
        ("small", small),
        ("medium", medium.as_str()),
        ("deep", deep.as_str()),
        ("wide", wide.as_str()),
    ] {
        benchmark_case(name, input);
    }
}
