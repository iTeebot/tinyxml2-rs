//! Demonstrates the use of the `tinyxml2-capi` FFI functions from Rust
//! to show how a host C/C++ application interacts with the C API.

use std::ffi::{CStr, CString};
use tinyxml2_capi::{
    TxError, tx_document_free, tx_document_new, tx_document_parse, tx_element_attribute,
    tx_element_get_text, tx_first_child_element, tx_root_element,
};

fn main() {
    println!("--- Running C Interop Example via tinyxml2-capi FFI ---");

    // 1. Create a new document via C API
    // C++ equivalent: XMLDocument* doc = new XMLDocument();
    let doc_ptr = tx_document_new();
    assert!(!doc_ptr.is_null(), "Failed to allocate document via FFI");

    unsafe {
        // 2. Parse XML string via C API
        // C++ equivalent: doc->Parse(xml);
        let xml = CString::new(
            r#"<?xml version="1.0" encoding="utf-8"?>
            <app-settings>
                <connection-string provider="mssql">Server=localhost;Database=prod;</connection-string>
                <theme>dark</theme>
            </app-settings>"#
        ).unwrap();

        let parse_result = tx_document_parse(doc_ptr, xml.as_ptr());
        if parse_result != TxError::TxSuccess {
            eprintln!("FFI Parse Error: {parse_result:?}");
            tx_document_free(doc_ptr);
            return;
        }
        println!("XML successfully parsed via FFI.");

        // 3. Navigate elements via C API
        // C++ equivalent: XMLElement* root = doc->RootElement();
        let root_id = tx_root_element(doc_ptr);
        assert!(!root_id.is_null(), "Root element is null");

        // Find <connection-string> child element
        let conn_name = CString::new("connection-string").unwrap();
        let conn_id = tx_first_child_element(doc_ptr, root_id, conn_name.as_ptr());
        assert!(!conn_id.is_null(), "Connection-string element not found");

        // 4. Retrieve attribute via C API
        // C++ equivalent: const char* provider = conn->Attribute("provider");
        let provider_name = CString::new("provider").unwrap();
        let prov_ptr = tx_element_attribute(doc_ptr, conn_id, provider_name.as_ptr());
        if !prov_ptr.is_null() {
            let provider_str = CStr::from_ptr(prov_ptr).to_str().unwrap();
            println!("Connection Provider (Attribute): {provider_str}");
        }

        // 5. Retrieve text content via C API
        // C++ equivalent: const char* conn_text = conn->GetText();
        let text_ptr = tx_element_get_text(doc_ptr, conn_id);
        if !text_ptr.is_null() {
            let conn_text = CStr::from_ptr(text_ptr).to_str().unwrap();
            println!("Connection String (Text):       {conn_text}");
        }

        // Find <theme> child element
        let theme_name = CString::new("theme").unwrap();
        let theme_id = tx_first_child_element(doc_ptr, root_id, theme_name.as_ptr());
        if !theme_id.is_null() {
            let theme_text_ptr = tx_element_get_text(doc_ptr, theme_id);
            if !theme_text_ptr.is_null() {
                let theme_text = CStr::from_ptr(theme_text_ptr).to_str().unwrap();
                println!("App Theme (Text):               {theme_text}");
            }
        }

        // 6. Free the document via FFI
        // C++ equivalent: delete doc;
        tx_document_free(doc_ptr);
    }

    println!("FFI document successfully freed. C Interop completed.");
}
