use std::{
    env,
    io::{self, Read},
};

use hackathon::compare_against_c;

fn main() {
    let input = env::args().nth(1).unwrap_or_else(|| {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer).unwrap();
        buffer
    });

    let mut input = input.trim().to_string();
    if input.len() >= 2 {
        if (input.starts_with('"') && input.ends_with('"'))
            || (input.starts_with('\'') && input.ends_with('\''))
        {
            input.remove(0);
            input.pop();
        }
    }

    match compare_against_c(&input) {
        Ok(output) => {
            println!("rust={output}");
        }
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(1);
        }
    }
}
