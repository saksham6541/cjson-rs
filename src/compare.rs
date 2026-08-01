use std::{env, path::PathBuf, process::Command};

use crate::{parse, print_unformatted};

pub fn compare_against_c(input: &str) -> Result<String, String> {
    let rust_result = parse(input).map(|value| print_unformatted(&value));
    let reference_binary = reference_binary_path();
    ensure_reference_binary(&reference_binary)?;

    let output = Command::new(&reference_binary)
        .arg(input)
        .output()
        .map_err(|e| {
            format!(
                "failed to run C reference binary at {}: {e}",
                reference_binary.display()
            )
        })?;

    let c_succeeded = output.status.success();
    let c_output = String::from_utf8_lossy(&output.stdout).trim().to_string();

    match (rust_result, c_succeeded) {
        (Ok(rust_output), true) => {
            if canonical_json(&rust_output) == canonical_json(&c_output) {
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

fn canonical_json(input: &str) -> String {
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(input) {
        serde_json::to_string(&value).unwrap_or_else(|_| input.to_string())
    } else {
        input.to_string()
    }
}

fn ensure_reference_binary(reference_binary: &PathBuf) -> Result<(), String> {
    if reference_binary.exists() {
        return Ok(());
    }

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let python = env::var("PYTHON").unwrap_or_else(|_| "python".to_string());
    let build_script = manifest_dir.join("build_c_reference.py");
    let status = Command::new(&python)
        .arg(&build_script)
        .current_dir(&manifest_dir)
        .status()
        .map_err(|e| format!("failed to launch reference build script: {e}"))?;

    if !status.success() {
        return Err(format!(
            "failed to build C reference binary via {}",
            build_script.display()
        ));
    }

    if !reference_binary.exists() {
        return Err(format!(
            "reference build completed but binary is missing at {}",
            reference_binary.display()
        ));
    }

    Ok(())
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
