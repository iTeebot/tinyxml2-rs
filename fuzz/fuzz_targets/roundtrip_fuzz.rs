#![no_main]
use libfuzzer_sys::fuzz_target;
use tinyxml2::Document;

fuzz_target!(|data: &[u8]| {
    if let Ok(xml) = std::str::from_utf8(data) {
        if let Ok(doc1) = Document::parse(xml) {
            let output1 = doc1.to_string();
            if let Ok(doc2) = Document::parse(&output1) {
                let output2 = doc2.to_string();
                assert_eq!(output1, output2, "Roundtrip instability");
            }
        }
    }
});
