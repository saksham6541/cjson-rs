use std::{
    env,
    io::{self, Read},
};

use hackathon::compare_against_c_bytes;

fn main() {
    // Read raw bytes, not `read_to_string`. The previous version called
    // `read_to_string(..).unwrap()`, which panics outright on invalid UTF-8 —
    // exactly the class of input (cJSON passes invalid UTF-8 through
    // permissively, per §5) this harness exists to compare. See DECISIONS.md.
    let input: Vec<u8> = match env::args().nth(1) {
        Some(arg) => arg.into_bytes(),
        None => {
            let mut buffer = Vec::new();
            io::stdin().read_to_end(&mut buffer).unwrap();
            buffer
        }
    };

    let mut input = input;
    trim_ascii_whitespace(&mut input);
    if input.len() >= 2 {
        let first = input[0];
        let last = input[input.len() - 1];
        if (first == b'"' && last == b'"') || (first == b'\'' && last == b'\'') {
            input.remove(0);
            input.pop();
        }
    }

    match compare_against_c_bytes(&input) {
        Ok(output) => {
            println!("rust={output}");
        }
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(1);
        }
    }
}

fn trim_ascii_whitespace(buffer: &mut Vec<u8>) {
    while matches!(buffer.first(), Some(b) if b.is_ascii_whitespace()) {
        buffer.remove(0);
    }
    while matches!(buffer.last(), Some(b) if b.is_ascii_whitespace()) {
        buffer.pop();
    }
}
