#![no_main]

use libfuzzer_sys::fuzz_target;

// Real libFuzzer entry point. Previously this file was a plain `main()` that
// only ever read one corpus file and exited — `libfuzzer-sys` was a declared
// dependency but never actually invoked, so `cargo fuzz run differential`
// wasn't getting libFuzzer's mutation engine at all. See DECISIONS.md.
//
// Takes raw bytes (not `&str`) so libFuzzer's mutator, which has no notion of
// UTF-8, can hand us arbitrary byte sequences the way it would to a C target.
// `compare_against_c_bytes` handles the UTF-8-validity gap between Rust
// strings and cJSON's raw-byte model (see compare.rs doc comment).
fuzz_target!(|data: &[u8]| {
    if let Err(reason) = hackathon::compare_against_c_bytes(data) {
        panic!("differential mismatch on {data:?}: {reason}");
    }
});
