use std::process::Command;

use crate::{parse, print_unformatted};

pub fn compare_against_c(input: &str) -> Result<String, String> {
    let rust_result = parse(input).map(|value| print_unformatted(&value));

    let output = Command::new(reference_binary_path())
        .arg(input)
        .output()
        .map_err(|e| format!("failed to run C reference binary: {e}"))?;

    let c_succeeded = output.status.success();
    let c_output = String::from_utf8_lossy(&output.stdout).trim().to_string();

    match (rust_result, c_succeeded) {
        (Ok(rust_output), true) => {
            if rust_output == c_output {
                Ok(rust_output)
            } else {
                Err(format!("mismatch: rust={rust_output} c={c_output}"))
            }
        }
        (Err(rust_err), false) => {
            // both sides reject -- not a divergence
            Ok(format!("both rejected (rust: {rust_err})"))
        }
        (Ok(rust_output), false) => {
            Err(format!("rust accepted but c rejected: rust={rust_output}"))
        }
        (Err(rust_err), true) => Err(format!(
            "c accepted but rust rejected: rust_err={rust_err}, c={c_output}"
        )),
    }
}

fn reference_binary_path() -> &'static str {
    if cfg!(windows) {
        r".\target\cjson_reference.exe"
    } else {
        "./target/cjson_reference"
    }
}
