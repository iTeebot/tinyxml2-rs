#![no_main]
use libfuzzer_sys::fuzz_target;
use tinyxml2::XmlPrinter;

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }
    let mut printer = XmlPrinter::new();
    let mut i = 0;
    while i < data.len() {
        let op = data[i] % 10;
        i += 1;
        match op {
            0 => {
                // open_element
                if i + 3 <= data.len() {
                    let len = (data[i] as usize) % 10 + 1;
                    i += 1;
                    if i + len <= data.len() {
                        if let Ok(name) = std::str::from_utf8(&data[i..i+len]) {
                            if !name.trim().is_empty() {
                                printer.open_element(name);
                            }
                        }
                        i += len;
                    }
                }
            }
            1 => {
                // close_element
                printer.close_element();
            }
            2 => {
                // push_attribute
                if i + 4 <= data.len() {
                    let key_len = (data[i] as usize) % 5 + 1;
                    let val_len = (data[i+1] as usize) % 10 + 1;
                    i += 2;
                    if i + key_len + val_len <= data.len() {
                        if let (Ok(key), Ok(val)) = (
                            std::str::from_utf8(&data[i..i+key_len]),
                            std::str::from_utf8(&data[i+key_len..i+key_len+val_len]),
                        ) {
                            if !key.trim().is_empty() {
                                printer.push_attribute(key, val);
                            }
                        }
                        i += key_len + val_len;
                    }
                }
            }
            3 => {
                // push_text
                if i + 2 <= data.len() {
                    let len = (data[i] as usize) % 20 + 1;
                    i += 1;
                    if i + len <= data.len() {
                        if let Ok(text) = std::str::from_utf8(&data[i..i+len]) {
                            printer.push_text(text);
                        }
                        i += len;
                    }
                }
            }
            4 => {
                // push_comment
                if i + 2 <= data.len() {
                    let len = (data[i] as usize) % 20 + 1;
                    i += 1;
                    if i + len <= data.len() {
                        if let Ok(comment) = std::str::from_utf8(&data[i..i+len]) {
                            printer.push_comment(comment);
                        }
                        i += len;
                    }
                }
            }
            5 => {
                // push_cdata
                if i + 2 <= data.len() {
                    let len = (data[i] as usize) % 20 + 1;
                    i += 1;
                    if i + len <= data.len() {
                        if let Ok(cdata) = std::str::from_utf8(&data[i..i+len]) {
                            printer.push_cdata(cdata);
                        }
                        i += len;
                    }
                }
            }
            6 => {
                // push_unknown
                if i + 2 <= data.len() {
                    let len = (data[i] as usize) % 15 + 1;
                    i += 1;
                    if i + len <= data.len() {
                        if let Ok(unknown) = std::str::from_utf8(&data[i..i+len]) {
                            printer.push_unknown(unknown);
                        }
                        i += len;
                    }
                }
            }
            7 => {
                // push_declaration
                if i + 2 <= data.len() {
                    let len = (data[i] as usize) % 20 + 1;
                    i += 1;
                    if i + len <= data.len() {
                        if let Ok(decl) = std::str::from_utf8(&data[i..i+len]) {
                            printer.push_declaration(decl);
                        }
                        i += len;
                    }
                }
            }
            8 => {
                // push_header
                printer.push_header("1.0", Some("UTF-8"), Some(true));
            }
            _ => {
                let _ = printer.as_str();
            }
        }
    }
    let _ = printer.as_str();
});
