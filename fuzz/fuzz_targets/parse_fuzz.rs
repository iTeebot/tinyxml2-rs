#![no_main]
use libfuzzer_sys::fuzz_target;
use tinyxml2::Document;

fuzz_target!(|data: &[u8]| {
    let _ = Document::parse_bytes(data);
});
