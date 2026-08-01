use std::{fs, path::PathBuf};

fn main() {
    let mut args = std::env::args().skip(1);
    let input = args.next().unwrap_or_else(|| {
        let corpus_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("corpus")
            .join("differential");
        let mut files = fs::read_dir(&corpus_dir)
            .unwrap_or_else(|_| panic!("missing corpus directory: {corpus_dir:?}"))
            .filter_map(Result::ok)
            .collect::<Vec<_>>();
        files.sort_by_key(|entry| entry.path().clone());
        let first = files
            .first()
            .unwrap_or_else(|| panic!("no corpus files found in {corpus_dir:?}"));
        fs::read_to_string(first.path()).unwrap()
    });

    if let Err(reason) = hackathon::compare_against_c(&input) {
        panic!("differential mismatch on {input:?}: {reason}");
    }
}
