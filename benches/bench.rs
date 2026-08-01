use std::{hint::black_box, time::Instant};

use hackathon::{parse, print, print_unformatted};

fn benchmark(name: &str, input: &str) {
    let start = Instant::now();
    let value = parse(input).unwrap();
    let parse_time = start.elapsed();

    let pretty_start = Instant::now();
    let _ = print(&value);
    let pretty_time = pretty_start.elapsed();

    let compact_start = Instant::now();
    let _ = print_unformatted(&value);
    let compact_time = compact_start.elapsed();

    println!(
        "{name}: parse={:?} pretty={:?} compact={:?}",
        parse_time, pretty_time, compact_time
    );
    black_box(value);
}

fn main() {
    benchmark(
        "small",
        r#"{"name":"Ada","active":true,"items":[1,2,3],"meta":{"score":42}}"#,
    );
    benchmark("medium", include_str!("../test_data/medium.json"));
    benchmark("large", include_str!("../test_data/large.json"));
}
