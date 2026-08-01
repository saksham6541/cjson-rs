use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use hackathon::compare_against_c;
use serde_json::{json, Value as JsonValue};
use sha2::{Digest, Sha256};

const ORIGINAL_CJSON_TEST_DIRS: &[&str] = &["../cJSON/tests", "original/cJSON/tests", "../original/cJSON/tests"];
const HASH_FILE_NAME: &str = "test_hashes.txt";
const RESULTS_FILE_NAME: &str = "test_results.json";

fn resolve_test_root() -> PathBuf {
    if let Ok(override_path) = env::var("CJSON_TEST_ROOT") {
        let path = PathBuf::from(override_path);
        if path.is_dir() {
            return path;
        }
        panic!("CJSON_TEST_ROOT is set but is not a directory: {}", path.display());
    }

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    for candidate in ORIGINAL_CJSON_TEST_DIRS {
        let path = manifest_dir.join(candidate);
        if path.is_dir() {
            return path;
        }
    }

    panic!(
        "could not find upstream cJSON tests directory. checked: {:?}" ,
        ORIGINAL_CJSON_TEST_DIRS
    );
}

fn collect_files(dir: &Path, extensions: &[&str]) -> std::io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            files.extend(collect_files(&path, extensions)?);
        } else if let Some(ext) = path.extension().and_then(OsStr::to_str) {
            if extensions.iter().any(|allowed| allowed == &ext) {
                files.push(path);
            }
        }
    }
    files.sort();
    Ok(files)
}

fn compute_sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

fn relative_to_manifest(path: &Path) -> String {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string()
}

fn write_hashes(path: &Path, hashes: &[(String, String)]) {
    let mut file = fs::File::create(path).unwrap_or_else(|err| {
        panic!("failed to create {}: {err}", path.display())
    });

    for (relative_path, sha) in hashes {
        writeln!(file, "{sha}  {relative_path}").unwrap();
    }
}

fn write_results(path: &Path, payload: &JsonValue) {
    let file = fs::File::create(path).unwrap_or_else(|err| {
        panic!("failed to create {}: {err}", path.display())
    });
    serde_json::to_writer_pretty(file, payload).unwrap();
}

#[test]
fn cjson_compatibility_test_harness() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir.clone();
    let test_root = resolve_test_root();

    let hash_files = collect_files(&test_root, &["c", "json"]).unwrap_or_else(|err| {
        panic!("failed to collect hash files from {}: {err}", test_root.display())
    });
    assert!(!hash_files.is_empty(), "no .c or .json files found in {test_root:?}");

    let json_cases = collect_files(&test_root, &["json"]).unwrap_or_else(|err| {
        panic!("failed to collect JSON test cases from {}: {err}", test_root.display())
    });
    assert!(!json_cases.is_empty(), "no .json test case files found in {test_root:?}");

    println!("cJSON compatibility test harness starting");
    println!("test root: {}", test_root.display());

    let mut hashes = Vec::new();
    for file_path in &hash_files {
        let bytes = fs::read(file_path).unwrap_or_else(|err| {
            panic!("failed to read {}: {err}", file_path.display())
        });
        let sha = compute_sha256(&bytes);
        println!("hash [{}] = {}", relative_to_manifest(file_path), sha);
        hashes.push((relative_to_manifest(file_path), sha));
    }

    let mut cases = Vec::new();
    let mut passed = 0;
    let mut failed = 0;

    for case_path in &json_cases {
        let bytes = fs::read(case_path).unwrap_or_else(|err| {
            panic!("failed to read {}: {err}", case_path.display())
        });
        let input = String::from_utf8_lossy(&bytes).to_string();
        let result = compare_against_c(&input);

        let status = if result.is_ok() { "passed" } else { "failed" };
        if status == "passed" {
            passed += 1;
        } else {
            failed += 1;
        }

        cases.push(json!({
            "path": relative_to_manifest(case_path),
            "sha256": compute_sha256(&bytes),
            "status": status,
            "details": match result {
                Ok(output) => json!({"output": output}),
                Err(error) => json!({"error": error}),
            },
        }));
    }

    let hashes_path = repo_root.join(HASH_FILE_NAME);
    write_hashes(&hashes_path, &hashes);

    let results_path = repo_root.join(RESULTS_FILE_NAME);
    let summary = json!({
        "total_files": hash_files.len(),
        "executed_cases": json_cases.len(),
        "passed": passed,
        "failed": failed,
        "expected_passed": 45,
        "hashes_file": hashes_path.to_string_lossy().to_string(),
        "results_file": results_path.to_string_lossy().to_string(),
    });
    let payload = json!({
        "summary": summary,
        "cases": cases,
    });
    write_results(&results_path, &payload);

    println!("cJSON compatibility harness summary:");
    println!("  test root: {}", test_root.display());
    println!("  total hashed files: {}", hash_files.len());
    println!("  executed JSON cases: {}", json_cases.len());
    println!("  passed: {}", passed);
    println!("  failed: {}", failed);
    println!("  hashes: {}", hashes_path.display());
    println!("  results: {}", results_path.display());

    if failed > 0 {
        println!("\nfailed cases:");
        for case in payload["cases"].as_array().unwrap().iter().filter(|case| case["status"] == "failed") {
            println!("- {}", case["path"]);
            println!("  details: {}", case["details"]);
        }
    }

    assert_eq!(failed, 0, "{} compatibility cases failed", failed);
}
