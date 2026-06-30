#![no_main]
use libfuzzer_sys::fuzz_target;
use tinyxml2::Document;

fuzz_target!(|data: &[u8]| {
    if let Ok(xml) = std::str::from_utf8(data) {
        if let Ok(doc) = Document::parse(xml) {
            let _ = doc.to_string();
            let _ = doc.to_string_compact();
        }
    }
});
