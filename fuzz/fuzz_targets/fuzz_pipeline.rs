#![no_main]

use libfuzzer_sys::fuzz_target;
use wos_shared::pipeline::parse_pipeline;

fuzz_target!(|data: &[u8]| {
    // Convert random bytes to UTF-8 string (or skip invalid UTF-8)
    if let Ok(s) = std::str::from_utf8(data) {
        // Fuzz the pipeline parser with arbitrary input
        let _result = parse_pipeline(s);
        // Parser should never panic, even on malformed input
    }
});
