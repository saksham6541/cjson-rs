use std::process::Command;

use crate::{parse, print_unformatted};

pub fn compare_against_c(input: &str) -> Result<String, String> {
    let rust_output = match parse(input) {
        Ok(value) => print_unformatted(&value),
        Err(err) => return Err(format!("rust parse error: {err}")),
    };

    let output = Command::new(reference_binary_path())
        .arg(input)
        .output()
        .map_err(|e| format!("failed to run C reference binary: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "c reference rejected input: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let c_output = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if rust_output != c_output {
        return Err(format!("mismatch: rust={rust_output} c={c_output}"));
    }

    Ok(rust_output)
}

fn reference_binary_path() -> &'static str {
    if cfg!(windows) {
        r".\target\cjson_reference.exe"
    } else {
        "./target/cjson_reference"
    }
}