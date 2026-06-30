//! FFI integration tests for the `tinyxml2-capi` C API.
//!
//! Each test calls the `extern "C"` functions directly from Rust and validates
//! correctness of the C-compatible API surface.

use std::ffi::{CStr, CString};
use std::ptr;

use tinyxml2_capi::{
    TX_NULL_NODE, TxDocument, TxError, TxNodeId, TxNodeType, tx_bool_attribute, tx_document_clear,
    tx_document_error, tx_document_error_line, tx_document_error_name, tx_document_free,
    tx_document_load_file, tx_document_new, tx_document_parse, tx_document_save_file,
    tx_document_to_string, tx_document_to_string_compact, tx_double_attribute,
    tx_element_attribute, tx_element_delete_attribute, tx_element_get_text, tx_element_name,
    tx_element_set_attribute, tx_element_set_text, tx_first_child, tx_first_child_element,
    tx_insert_after_child, tx_insert_end_child, tx_insert_first_child, tx_int_attribute,
    tx_last_child, tx_new_comment, tx_new_declaration, tx_new_element, tx_new_text, tx_new_unknown,
    tx_next_sibling, tx_next_sibling_element, tx_node_is_null, tx_node_line, tx_node_type,
    tx_node_value, tx_parent, tx_prev_sibling, tx_printer_clear, tx_printer_close_element,
    tx_printer_free, tx_printer_new, tx_printer_new_compact, tx_printer_open_element,
    tx_printer_push_attribute, tx_printer_push_comment, tx_printer_push_text, tx_printer_result,
    tx_query_bool_attribute, tx_query_double_attribute, tx_query_int_attribute, tx_root_element,
};

// ============================================================
// Helpers
// ============================================================

/// Shorthand: create a `CString` from a `&str`.
fn c(s: &str) -> CString {
    CString::new(s).unwrap()
}

/// Shorthand: read a `*const c_char` into a `&str`.
///
/// # Safety
///
/// The pointer must be non-null and point to a valid, null-terminated UTF-8
/// C string.
unsafe fn str_from_ptr(ptr: *const std::ffi::c_char) -> &'static str {
    assert!(!ptr.is_null(), "unexpected null C string pointer");
    unsafe { CStr::from_ptr(ptr) }.to_str().unwrap()
}

// ============================================================
// 1. Document Lifecycle
// ============================================================

#[test]
fn doc_create_and_free() {
    let doc = tx_document_new();
    assert!(!doc.is_null());
    // Fresh document has no error.
    let err = unsafe { tx_document_error(doc) };
    assert_eq!(err, TxError::TxSuccess);
    unsafe { tx_document_free(doc) };
}

#[test]
fn doc_parse_valid_xml() {
    let doc = tx_document_new();
    let xml = c("<root><child/></root>");
    let err = unsafe { tx_document_parse(doc, xml.as_ptr()) };
    assert_eq!(err, TxError::TxSuccess);

    // Root element should be accessible.
    let root = unsafe { tx_root_element(doc) };
    assert!(!tx_node_is_null(root));

    let name_ptr = unsafe { tx_element_name(doc, root) };
    assert_eq!(unsafe { str_from_ptr(name_ptr) }, "root");

    unsafe { tx_document_free(doc) };
}

#[test]
fn doc_parse_invalid_xml() {
    let doc = tx_document_new();
    let xml = c("<root><unclosed>");
    let err = unsafe { tx_document_parse(doc, xml.as_ptr()) };
    assert_ne!(err, TxError::TxSuccess);

    // Error state should be reflected.
    let err2 = unsafe { tx_document_error(doc) };
    assert_ne!(err2, TxError::TxSuccess);

    unsafe { tx_document_free(doc) };
}

#[test]
fn doc_clear_and_reparse() {
    let doc = tx_document_new();

    // Parse initial XML.
    let xml1 = c("<first/>");
    let err = unsafe { tx_document_parse(doc, xml1.as_ptr()) };
    assert_eq!(err, TxError::TxSuccess);

    let root = unsafe { tx_root_element(doc) };
    let name_ptr = unsafe { tx_element_name(doc, root) };
    assert_eq!(unsafe { str_from_ptr(name_ptr) }, "first");

    // Clear and reparse different XML.
    unsafe { tx_document_clear(doc) };
    let xml2 = c("<second/>");
    let err2 = unsafe { tx_document_parse(doc, xml2.as_ptr()) };
    assert_eq!(err2, TxError::TxSuccess);

    let root2 = unsafe { tx_root_element(doc) };
    let name_ptr2 = unsafe { tx_element_name(doc, root2) };
    assert_eq!(unsafe { str_from_ptr(name_ptr2) }, "second");

    unsafe { tx_document_free(doc) };
}

// ============================================================
// 2. Factory + Tree Building
// ============================================================

#[test]
fn factory_create_elements_and_insert() {
    let doc = tx_document_new();

    let root_name = c("root");
    let root = unsafe { tx_new_element(doc, root_name.as_ptr()) };
    assert!(!tx_node_is_null(root));

    // Insert root as child of document node.
    let _doc_node = unsafe { tx_first_child(doc, TX_NULL_NODE) };
    // The document node is the implicit parent; use insert_end_child on the
    // document's own node id. We need to get the document node id. We can
    // parse an empty doc or build from scratch. Let's try inserting root
    // as a child of the document directly — tx_insert_end_child with the
    // document root.

    // Actually, for tinyxml2, the document itself is a node. The document
    // node has index 0, generation 0 — let's construct it.
    let doc_id = TxNodeId {
        index: 0,
        generation: 0,
    };

    let err = unsafe { tx_insert_end_child(doc, doc_id, root) };
    assert_eq!(err, TxError::TxSuccess);

    // Create children.
    let child1_name = c("alpha");
    let child1 = unsafe { tx_new_element(doc, child1_name.as_ptr()) };
    let err = unsafe { tx_insert_end_child(doc, root, child1) };
    assert_eq!(err, TxError::TxSuccess);

    let child2_name = c("beta");
    let child2 = unsafe { tx_new_element(doc, child2_name.as_ptr()) };
    let err = unsafe { tx_insert_first_child(doc, root, child2) };
    assert_eq!(err, TxError::TxSuccess);

    // child2 ("beta") should now be the first child of root.
    let first = unsafe { tx_first_child(doc, root) };
    let first_name = unsafe { tx_element_name(doc, first) };
    assert_eq!(unsafe { str_from_ptr(first_name) }, "beta");

    // child1 ("alpha") should be the last child.
    let last = unsafe { tx_last_child(doc, root) };
    let last_name = unsafe { tx_element_name(doc, last) };
    assert_eq!(unsafe { str_from_ptr(last_name) }, "alpha");

    // Insert a third element after child2 (beta), before child1 (alpha).
    let child3_name = c("gamma");
    let child3 = unsafe { tx_new_element(doc, child3_name.as_ptr()) };
    let err = unsafe { tx_insert_after_child(doc, first, child3) };
    assert_eq!(err, TxError::TxSuccess);

    // Order should now be: beta, gamma, alpha.
    let mid = unsafe { tx_next_sibling(doc, first) };
    let mid_name = unsafe { tx_element_name(doc, mid) };
    assert_eq!(unsafe { str_from_ptr(mid_name) }, "gamma");

    unsafe { tx_document_free(doc) };
}

#[test]
fn factory_create_text_comment_declaration_unknown() {
    let doc = tx_document_new();

    let doc_id = tinyxml2_capi::TxNodeId {
        index: 0,
        generation: 0,
    };

    // Create a root element to hold things.
    let root_name = c("root");
    let root = unsafe { tx_new_element(doc, root_name.as_ptr()) };
    let err = unsafe { tx_insert_end_child(doc, doc_id, root) };
    assert_eq!(err, TxError::TxSuccess);

    // Text node.
    let text_content = c("Hello World");
    let text = unsafe { tx_new_text(doc, text_content.as_ptr()) };
    assert!(!tx_node_is_null(text));
    let err = unsafe { tx_insert_end_child(doc, root, text) };
    assert_eq!(err, TxError::TxSuccess);

    // Comment node.
    let comment_content = c("this is a comment");
    let comment = unsafe { tx_new_comment(doc, comment_content.as_ptr()) };
    assert!(!tx_node_is_null(comment));
    let err = unsafe { tx_insert_end_child(doc, root, comment) };
    assert_eq!(err, TxError::TxSuccess);

    // Declaration node.
    let decl_content = c("xml version=\"1.0\"");
    let decl = unsafe { tx_new_declaration(doc, decl_content.as_ptr()) };
    assert!(!tx_node_is_null(decl));

    // Unknown node.
    let unknown_content = c("!ENTITY test");
    let unknown = unsafe { tx_new_unknown(doc, unknown_content.as_ptr()) };
    assert!(!tx_node_is_null(unknown));

    // Verify node types.
    let text_type = unsafe { tx_node_type(doc, text) };
    assert_eq!(text_type, TxNodeType::TxNodeText);

    let comment_type = unsafe { tx_node_type(doc, comment) };
    assert_eq!(comment_type, TxNodeType::TxNodeComment);

    let decl_type = unsafe { tx_node_type(doc, decl) };
    assert_eq!(decl_type, TxNodeType::TxNodeDeclaration);

    let unknown_type = unsafe { tx_node_type(doc, unknown) };
    assert_eq!(unknown_type, TxNodeType::TxNodeUnknown);

    unsafe { tx_document_free(doc) };
}

// ============================================================
// 3. DOM Navigation
// ============================================================

#[test]
fn nav_parent_child_sibling() {
    let doc = tx_document_new();
    let xml = c("<root><a/><b/><c/></root>");
    let err = unsafe { tx_document_parse(doc, xml.as_ptr()) };
    assert_eq!(err, TxError::TxSuccess);

    let root = unsafe { tx_root_element(doc) };

    // first_child → "a"
    let a = unsafe { tx_first_child(doc, root) };
    assert!(!tx_node_is_null(a));
    let a_name = unsafe { tx_element_name(doc, a) };
    assert_eq!(unsafe { str_from_ptr(a_name) }, "a");

    // last_child → "c"
    let c_node = unsafe { tx_last_child(doc, root) };
    let c_name = unsafe { tx_element_name(doc, c_node) };
    assert_eq!(unsafe { str_from_ptr(c_name) }, "c");

    // next_sibling(a) → "b"
    let b = unsafe { tx_next_sibling(doc, a) };
    let b_name = unsafe { tx_element_name(doc, b) };
    assert_eq!(unsafe { str_from_ptr(b_name) }, "b");

    // prev_sibling(c) → "b"
    let prev = unsafe { tx_prev_sibling(doc, c_node) };
    let prev_name = unsafe { tx_element_name(doc, prev) };
    assert_eq!(unsafe { str_from_ptr(prev_name) }, "b");

    // parent(a) → root
    let par = unsafe { tx_parent(doc, a) };
    let par_name = unsafe { tx_element_name(doc, par) };
    assert_eq!(unsafe { str_from_ptr(par_name) }, "root");

    // prev_sibling(a) → null (no previous)
    let none = unsafe { tx_prev_sibling(doc, a) };
    assert!(tx_node_is_null(none));

    // next_sibling(c) → null (no next)
    let none2 = unsafe { tx_next_sibling(doc, c_node) };
    assert!(tx_node_is_null(none2));

    unsafe { tx_document_free(doc) };
}

#[test]
fn nav_first_child_element_with_name_filter() {
    let doc = tx_document_new();
    let xml = c("<root><item id=\"1\"/><other/><item id=\"2\"/></root>");
    let err = unsafe { tx_document_parse(doc, xml.as_ptr()) };
    assert_eq!(err, TxError::TxSuccess);

    let root = unsafe { tx_root_element(doc) };

    // No name filter — returns first child element.
    let first_any = unsafe { tx_first_child_element(doc, root, ptr::null()) };
    assert!(!tx_node_is_null(first_any));
    let first_name = unsafe { tx_element_name(doc, first_any) };
    assert_eq!(unsafe { str_from_ptr(first_name) }, "item");

    // Filter by "other".
    let filter = c("other");
    let other = unsafe { tx_first_child_element(doc, root, filter.as_ptr()) };
    assert!(!tx_node_is_null(other));
    let other_name = unsafe { tx_element_name(doc, other) };
    assert_eq!(unsafe { str_from_ptr(other_name) }, "other");

    // next_sibling_element with name filter "item" from first item → second item.
    let item_filter = c("item");
    let second_item = unsafe { tx_next_sibling_element(doc, first_any, item_filter.as_ptr()) };
    assert!(!tx_node_is_null(second_item));

    // Verify it's the second item by checking the "id" attribute.
    let id_attr = c("id");
    let id_ptr = unsafe { tx_element_attribute(doc, second_item, id_attr.as_ptr()) };
    assert_eq!(unsafe { str_from_ptr(id_ptr) }, "2");

    // Filter by non-existent name.
    let nope = c("nonexistent");
    let missing = unsafe { tx_first_child_element(doc, root, nope.as_ptr()) };
    assert!(tx_node_is_null(missing));

    unsafe { tx_document_free(doc) };
}

#[test]
fn nav_root_element() {
    let doc = tx_document_new();

    // Empty document → null root element.
    let root = unsafe { tx_root_element(doc) };
    assert!(tx_node_is_null(root));

    // Parse something → root is available.
    let xml = c("<hello/>");
    let _ = unsafe { tx_document_parse(doc, xml.as_ptr()) };
    let root = unsafe { tx_root_element(doc) };
    assert!(!tx_node_is_null(root));

    let name_ptr = unsafe { tx_element_name(doc, root) };
    assert_eq!(unsafe { str_from_ptr(name_ptr) }, "hello");

    unsafe { tx_document_free(doc) };
}

// ============================================================
// 4. Attribute Operations
// ============================================================

#[test]
fn attr_set_and_get() {
    let doc = tx_document_new();
    let xml = c("<root/>");
    let _ = unsafe { tx_document_parse(doc, xml.as_ptr()) };
    let root = unsafe { tx_root_element(doc) };

    let attr_name = c("color");
    let attr_val = c("blue");
    let err = unsafe { tx_element_set_attribute(doc, root, attr_name.as_ptr(), attr_val.as_ptr()) };
    assert_eq!(err, TxError::TxSuccess);

    // Read it back.
    let ptr = unsafe { tx_element_attribute(doc, root, attr_name.as_ptr()) };
    assert_eq!(unsafe { str_from_ptr(ptr) }, "blue");

    // Overwrite with a new value.
    let attr_val2 = c("red");
    let err =
        unsafe { tx_element_set_attribute(doc, root, attr_name.as_ptr(), attr_val2.as_ptr()) };
    assert_eq!(err, TxError::TxSuccess);

    let ptr2 = unsafe { tx_element_attribute(doc, root, attr_name.as_ptr()) };
    assert_eq!(unsafe { str_from_ptr(ptr2) }, "red");

    // Non-existent attribute → null.
    let missing = c("nope");
    let ptr3 = unsafe { tx_element_attribute(doc, root, missing.as_ptr()) };
    assert!(ptr3.is_null());

    unsafe { tx_document_free(doc) };
}

#[test]
fn attr_delete() {
    let doc = tx_document_new();
    let xml = c("<root keep=\"yes\" remove=\"me\"/>");
    let _ = unsafe { tx_document_parse(doc, xml.as_ptr()) };
    let root = unsafe { tx_root_element(doc) };

    let remove_name = c("remove");
    let err = unsafe { tx_element_delete_attribute(doc, root, remove_name.as_ptr()) };
    assert_eq!(err, TxError::TxSuccess);

    // Deleted attribute should be null.
    let ptr = unsafe { tx_element_attribute(doc, root, remove_name.as_ptr()) };
    assert!(ptr.is_null());

    // "keep" should still be there.
    let keep_name = c("keep");
    let ptr2 = unsafe { tx_element_attribute(doc, root, keep_name.as_ptr()) };
    assert_eq!(unsafe { str_from_ptr(ptr2) }, "yes");

    unsafe { tx_document_free(doc) };
}

#[test]
fn attr_typed_int_bool_double() {
    let doc = tx_document_new();
    let xml = c("<item count=\"42\" active=\"true\" ratio=\"1.25\"/>");
    let _ = unsafe { tx_document_parse(doc, xml.as_ptr()) };
    let root = unsafe { tx_root_element(doc) };

    // Query int attribute.
    let count_name = c("count");
    let mut int_val: std::ffi::c_int = 0;
    let err = unsafe { tx_query_int_attribute(doc, root, count_name.as_ptr(), &raw mut int_val) };
    assert_eq!(err, TxError::TxSuccess);
    assert_eq!(int_val, 42);

    // Query bool attribute.
    let active_name = c("active");
    let mut bool_val: bool = false;
    let err =
        unsafe { tx_query_bool_attribute(doc, root, active_name.as_ptr(), &raw mut bool_val) };
    assert_eq!(err, TxError::TxSuccess);
    assert!(bool_val);

    // Query double attribute.
    let ratio_name = c("ratio");
    let mut dbl_val: std::ffi::c_double = 0.0;
    let err =
        unsafe { tx_query_double_attribute(doc, root, ratio_name.as_ptr(), &raw mut dbl_val) };
    assert_eq!(err, TxError::TxSuccess);
    assert!((dbl_val - 1.25).abs() < 1e-10);

    // Convenience accessors with defaults.
    let int_v = unsafe { tx_int_attribute(doc, root, count_name.as_ptr(), -1) };
    assert_eq!(int_v, 42);

    let bool_v = unsafe { tx_bool_attribute(doc, root, active_name.as_ptr(), false) };
    assert!(bool_v);

    let dbl_v = unsafe { tx_double_attribute(doc, root, ratio_name.as_ptr(), 0.0) };
    assert!((dbl_v - 1.25).abs() < 1e-10);

    // Default for missing attribute.
    let missing = c("missing");
    let def_int = unsafe { tx_int_attribute(doc, root, missing.as_ptr(), -99) };
    assert_eq!(def_int, -99);

    let def_bool = unsafe { tx_bool_attribute(doc, root, missing.as_ptr(), true) };
    assert!(def_bool);

    let def_dbl = unsafe { tx_double_attribute(doc, root, missing.as_ptr(), 9.9) };
    assert!((def_dbl - 9.9).abs() < f64::EPSILON);

    unsafe { tx_document_free(doc) };
}

// ============================================================
// 5. Element Text
// ============================================================

#[test]
fn text_get_and_set() {
    let doc = tx_document_new();
    let xml = c("<msg>Hello</msg>");
    let _ = unsafe { tx_document_parse(doc, xml.as_ptr()) };
    let root = unsafe { tx_root_element(doc) };

    // Get existing text.
    let ptr = unsafe { tx_element_get_text(doc, root) };
    assert_eq!(unsafe { str_from_ptr(ptr) }, "Hello");

    // Set new text.
    let new_text = c("Goodbye");
    let err = unsafe { tx_element_set_text(doc, root, new_text.as_ptr()) };
    assert_eq!(err, TxError::TxSuccess);

    let ptr2 = unsafe { tx_element_get_text(doc, root) };
    assert_eq!(unsafe { str_from_ptr(ptr2) }, "Goodbye");

    unsafe { tx_document_free(doc) };
}

#[test]
fn text_get_null_for_no_text() {
    let doc = tx_document_new();
    let xml = c("<empty/>");
    let _ = unsafe { tx_document_parse(doc, xml.as_ptr()) };
    let root = unsafe { tx_root_element(doc) };

    // Element with no text child → null.
    let ptr = unsafe { tx_element_get_text(doc, root) };
    assert!(ptr.is_null());

    unsafe { tx_document_free(doc) };
}

// ============================================================
// 6. Printer
// ============================================================

#[test]
fn printer_pretty() {
    let printer = tx_printer_new();
    assert!(!printer.is_null());

    let root_name = c("book");
    unsafe { tx_printer_open_element(printer, root_name.as_ptr()) };

    let attr_name = c("lang");
    let attr_val = c("en");
    unsafe { tx_printer_push_attribute(printer, attr_name.as_ptr(), attr_val.as_ptr()) };

    let text = c("Title");
    unsafe { tx_printer_push_text(printer, text.as_ptr()) };

    unsafe { tx_printer_close_element(printer) };

    let result_ptr = unsafe { tx_printer_result(printer) };
    let result = unsafe { str_from_ptr(result_ptr) };
    assert!(result.contains("<book"));
    assert!(result.contains("lang=\"en\""));
    assert!(result.contains("Title"));
    assert!(result.contains("</book>"));

    // Test clear.
    unsafe { tx_printer_clear(printer) };
    let cleared_ptr = unsafe { tx_printer_result(printer) };
    let cleared = unsafe { str_from_ptr(cleared_ptr) };
    assert!(
        cleared.is_empty(),
        "printer should be empty after clear, got: {cleared:?}"
    );

    unsafe { tx_printer_free(printer) };
}

#[test]
fn printer_compact() {
    let printer = tx_printer_new_compact();
    assert!(!printer.is_null());

    let elem = c("item");
    unsafe { tx_printer_open_element(printer, elem.as_ptr()) };

    let comment = c("note");
    unsafe { tx_printer_push_comment(printer, comment.as_ptr()) };

    let text = c("data");
    unsafe { tx_printer_push_text(printer, text.as_ptr()) };

    unsafe { tx_printer_close_element(printer) };

    let result_ptr = unsafe { tx_printer_result(printer) };
    let result = unsafe { str_from_ptr(result_ptr) };

    // Compact output should have no indentation/newlines between tags.
    assert!(result.contains("<item>"));
    assert!(result.contains("<!--note-->"));
    assert!(result.contains("data"));
    assert!(result.contains("</item>"));
    // Compact: no leading whitespace/newlines between elements.
    assert!(
        !result.contains("\n    "),
        "compact output should not have indentation"
    );

    unsafe { tx_printer_free(printer) };
}

// ============================================================
// 7. Node Inspection
// ============================================================

#[test]
fn node_type_and_value() {
    let doc = tx_document_new();
    let xml = c("<root>text<!-- comment --></root>");
    let _ = unsafe { tx_document_parse(doc, xml.as_ptr()) };

    let root = unsafe { tx_root_element(doc) };

    // Root element type.
    let root_type = unsafe { tx_node_type(doc, root) };
    assert_eq!(root_type, TxNodeType::TxNodeElement);

    // Root element value (tag name).
    let root_val = unsafe { tx_node_value(doc, root) };
    assert_eq!(unsafe { str_from_ptr(root_val) }, "root");

    // Text child.
    let text_node = unsafe { tx_first_child(doc, root) };
    let text_type = unsafe { tx_node_type(doc, text_node) };
    assert_eq!(text_type, TxNodeType::TxNodeText);
    let text_val = unsafe { tx_node_value(doc, text_node) };
    assert_eq!(unsafe { str_from_ptr(text_val) }, "text");

    // Comment child.
    let comment_node = unsafe { tx_next_sibling(doc, text_node) };
    let comment_type = unsafe { tx_node_type(doc, comment_node) };
    assert_eq!(comment_type, TxNodeType::TxNodeComment);
    let comment_val = unsafe { tx_node_value(doc, comment_node) };
    assert_eq!(unsafe { str_from_ptr(comment_val) }, " comment ");

    unsafe { tx_document_free(doc) };
}

#[test]
fn node_is_null_and_line() {
    // tx_node_is_null on the sentinel.
    assert!(tx_node_is_null(TX_NULL_NODE));

    // A valid node should not be null.
    let doc = tx_document_new();
    let xml = c("<root/>");
    let _ = unsafe { tx_document_parse(doc, xml.as_ptr()) };
    let root = unsafe { tx_root_element(doc) };
    assert!(!tx_node_is_null(root));

    // Line number (parsed from line 1).
    let line = unsafe { tx_node_line(doc, root) };
    assert!(line >= 1, "line should be >= 1 for a parsed node");

    unsafe { tx_document_free(doc) };
}

// ============================================================
// 8. Null Safety
// ============================================================

#[test]
fn null_document_pointer() {
    let null_doc: *mut TxDocument = ptr::null_mut();

    // All functions should handle null doc gracefully.
    let err = unsafe { tx_document_error(null_doc) };
    assert_eq!(err, TxError::TxErrorInvalidNodeId);

    let line = unsafe { tx_document_error_line(null_doc) };
    assert_eq!(line, 0);

    let name = unsafe { tx_document_error_name(null_doc) };
    assert!(name.is_null());

    let root = unsafe { tx_root_element(null_doc) };
    assert!(tx_node_is_null(root));

    let to_str = unsafe { tx_document_to_string(null_doc) };
    assert!(to_str.is_null());

    let to_str_c = unsafe { tx_document_to_string_compact(null_doc) };
    assert!(to_str_c.is_null());

    // Free null should be a no-op (no crash).
    unsafe { tx_document_free(null_doc) };

    // Clear null should be a no-op (no crash).
    unsafe { tx_document_clear(null_doc) };

    // Parse with null doc.
    let xml = c("<x/>");
    let err = unsafe { tx_document_parse(null_doc, xml.as_ptr()) };
    assert_ne!(err, TxError::TxSuccess);
}

#[test]
fn null_string_pointer() {
    let doc = tx_document_new();

    // Parse with null xml string.
    let err = unsafe { tx_document_parse(doc, ptr::null()) };
    assert_ne!(err, TxError::TxSuccess);

    // Factory with null name.
    let node = unsafe { tx_new_element(doc, ptr::null()) };
    assert!(tx_node_is_null(node));

    let node = unsafe { tx_new_text(doc, ptr::null()) };
    assert!(tx_node_is_null(node));

    let node = unsafe { tx_new_comment(doc, ptr::null()) };
    assert!(tx_node_is_null(node));

    // Load file with null path.
    let err = unsafe { tx_document_load_file(doc, ptr::null()) };
    assert_ne!(err, TxError::TxSuccess);

    // Save file with null path.
    let err = unsafe { tx_document_save_file(doc, ptr::null()) };
    assert_ne!(err, TxError::TxSuccess);

    unsafe { tx_document_free(doc) };
}

// ============================================================
// 9. Serialization
// ============================================================

#[test]
fn serialization_to_string() {
    let doc = tx_document_new();
    let xml = c("<root><child/></root>");
    let _ = unsafe { tx_document_parse(doc, xml.as_ptr()) };

    let ptr = unsafe { tx_document_to_string(doc) };
    assert!(!ptr.is_null());
    let result = unsafe { str_from_ptr(ptr) };

    // Pretty-printed output should contain newlines/indentation.
    assert!(result.contains("<root>"));
    assert!(result.contains("<child/>"));
    assert!(result.contains("</root>"));

    unsafe { tx_document_free(doc) };
}

#[test]
fn serialization_to_string_compact() {
    let doc = tx_document_new();
    let xml = c("<root><a/><b/></root>");
    let _ = unsafe { tx_document_parse(doc, xml.as_ptr()) };

    let ptr = unsafe { tx_document_to_string_compact(doc) };
    assert!(!ptr.is_null());
    let result = unsafe { str_from_ptr(ptr) };

    // Compact output — everything on one line, no extra whitespace.
    assert!(result.contains("<root>"));
    assert!(result.contains("<a/>"));
    assert!(result.contains("<b/>"));
    assert!(result.contains("</root>"));

    // Should NOT have indentation newlines.
    assert!(
        !result.contains("\n    "),
        "compact output should not have indentation, got: {result:?}"
    );

    unsafe { tx_document_free(doc) };
}
