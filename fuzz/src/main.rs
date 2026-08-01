use arbitrary::{Arbitrary, Unstructured};
use hackathon::{from_str as parse_rust, ParseError};
use libfuzzer_sys::fuzz_target;
use serde_json::{from_str as parse_serde, Value as JsonValue};
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::atomic::{AtomicU64, Ordering};

static TOTAL_RUNS: AtomicU64 = AtomicU64::new(0);
static SUCCESS_RUNS: AtomicU64 = AtomicU64::new(0);
static FAILURE_RUNS: AtomicU64 = AtomicU64::new(0);

fn log_discrepancy(message: &str) {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("fuzz.log")
        .unwrap();
    writeln!(file, "{}", message).unwrap();
}

fn compare_ast(rust_value: &JsonValue, serde_value: &JsonValue) -> bool {
    rust_value == serde_value
}

fn format_rust_error(error: &ParseError) -> String {
    format!("{} ({:?})", error, error.kind)
}

fuzz_target!(|data: &[u8]| {
    TOTAL_RUNS.fetch_add(1, Ordering::Relaxed);

    let mut unstructured = Unstructured::new(data);
    let input = match String::arbitrary(&mut unstructured) {
        Ok(value) => value,
        Err(_) => return,
    };

    let rust_result = parse_rust(&input);
    let serde_result: Result<JsonValue, _> = parse_serde(&input);

    match (rust_result, serde_result) {
        (Ok(rust_value), Ok(serde_value)) => {
            let rust_output = rust_value.to_string();
            match serde_json::from_str::<JsonValue>(&rust_output) {
                Ok(rust_json) => {
                    if !compare_ast(&rust_json, &serde_value) {
                        FAILURE_RUNS.fetch_add(1, Ordering::Relaxed);
                        let message = format!(
                            "AST mismatch\ninput: {input}\nrust: {rust_json:?}\nserde: {serde_value:?}\n"
                        );
                        log_discrepancy(&message);
                        panic!("AST mismatch");
                    }
                    SUCCESS_RUNS.fetch_add(1, Ordering::Relaxed);
                }
                Err(e) => {
                    FAILURE_RUNS.fetch_add(1, Ordering::Relaxed);
                    let message = format!(
                        "Invalid Rust output JSON\ninput: {input}\nrust output: {rust_output}\nerror: {e}\n"
                    );
                    log_discrepancy(&message);
                    panic!("Invalid Rust output JSON");
                }
            }
        }
        (Err(rust_err), Err(serde_err)) => {
            let rust_error = format_rust_error(&rust_err);
            let serde_error = serde_err.to_string();
            if rust_error != serde_error {
                FAILURE_RUNS.fetch_add(1, Ordering::Relaxed);
                let message = format!(
                    "Error mismatch\ninput: {input}\nrust: {rust_error}\nserde: {serde_error}\n"
                );
                log_discrepancy(&message);
                panic!("Error mismatch");
            }
            SUCCESS_RUNS.fetch_add(1, Ordering::Relaxed);
        }
        (Ok(_), Err(serde_err)) => {
            FAILURE_RUNS.fetch_add(1, Ordering::Relaxed);
            let message = format!(
                "Rust accepted but serde rejected\ninput: {input}\nserde: {serde_err}\n"
            );
            log_discrepancy(&message);
            panic!("Rust accepted but serde rejected");
        }
        (Err(rust_err), Ok(_)) => {
            FAILURE_RUNS.fetch_add(1, Ordering::Relaxed);
            let message = format!(
                "Rust rejected but serde accepted\ninput: {input}\nrust: {}\n",
                format_rust_error(&rust_err)
            );
            log_discrepancy(&message);
            panic!("Rust rejected but serde accepted");
        }
    }
});
