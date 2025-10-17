#![no_main]

use libfuzzer_sys::fuzz_target;
use wos_shared::parser::tokenize;

fuzz_target!(|data: &[u8]| {
    // Convert random bytes to UTF-8 string (or skip invalid UTF-8)
    if let Ok(s) = std::str::from_utf8(data) {
        // Fuzz the parser with arbitrary input
        let _result = tokenize(s);
        // Parser should never panic, even on malformed input
    }
});
