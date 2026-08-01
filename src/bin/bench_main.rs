use std::{fs, path::Path};

use hackathon::{parse, print, print_unformatted};

fn load_fixture(path: &str) -> String {
    fs::read_to_string(Path::new(path)).unwrap()
}

fn main() {
    let small = r#"{"name":"Ada","active":true,"items":[1,2,3],"meta":{"score":42}}"#;
    let medium = load_fixture("test_data/large.json");

    for (name, input) in [("small", small), ("medium", medium.as_str())] {
        let value = parse(input).unwrap();
        let pretty = print(&value);
        let compact = print_unformatted(&value);
        println!(
            "{name}: pretty_chars={} compact_chars={}",
            pretty.len(),
            compact.len()
        );
    }
}
