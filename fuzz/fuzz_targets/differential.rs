#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(input) = std::str::from_utf8(data) {
        // compare_against_c returns Err on either a parse mismatch,
        // a rust-parses-but-c-rejects (or vice versa) case, or an
        // output mismatch between the two. Any of those is a genuine
        // behavioral divergence worth panicking on so libFuzzer
        // records it as a finding.
        //
        // Inputs that fail to parse on BOTH sides are not divergences
        // and should not panic -- compare_against_c already returns
        // Ok in the "both reject, or both succeed and agree" cases,
        // so we only panic on an actual mismatch.
        if let Err(reason) = hackathon::compare_against_c(input) {
            panic!("differential mismatch on {input:?}: {reason}");
        }
    }
});