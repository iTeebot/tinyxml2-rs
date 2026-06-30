//! C FFI compatibility layer for tinyxml2-rs.
//!
//! This crate provides `extern "C"` functions that expose the tinyxml2 Rust API
//! through a C-compatible ABI. It produces both static and shared libraries.
//!
//! # Safety
//!
//! This crate necessarily uses `unsafe` at the FFI boundary. All public functions
//! validate inputs and catch panics to prevent undefined behavior across the
//! FFI boundary.
//!
//! # String Lifetimes
//!
//! Functions returning `*const c_char` return pointers to strings cached inside
//! the `TxDocument` or `TxPrinter` wrapper. These pointers remain valid until:
//! - The wrapper is freed (`tx_document_free` / `tx_printer_free`).
//! - A mutating operation is called on the wrapper (e.g., parse, insert, delete).

// Line numbers are always small positive u32 values; the cast to c_int (i32)
// will not wrap in any realistic scenario.
#![allow(clippy::cast_possible_wrap)]

mod types;

pub use types::{TX_NULL_NODE, TxDocument, TxError, TxNodeId, TxNodeType, TxPrinter};

use std::ffi::{CStr, CString, c_char, c_double, c_int};
use std::ptr;

// ============================================================
// Internal helpers
// ============================================================

/// Converts a C string pointer to a Rust `&str`.
///
/// Returns `None` if the pointer is null or the string is not valid UTF-8.
///
/// # Safety
///
/// The caller must ensure that `s` points to a valid, null-terminated C string
/// (or is null). The returned `&str` borrows the C string's memory.
unsafe fn cstr_to_str<'a>(s: *const c_char) -> Option<&'a str> {
    if s.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(s) }.to_str().ok()
}

/// Caches a Rust string slice in the document's string cache and returns
/// a pointer to the cached C string.
///
/// Returns null if the string contains interior null bytes.
fn cache_str(doc: &mut TxDocument, s: &str) -> *const c_char {
    match CString::new(s) {
        Ok(cs) => {
            let ptr = cs.as_ptr();
            doc.string_cache.push(cs);
            ptr
        }
        Err(_) => ptr::null(),
    }
}

/// Wraps a closure in `catch_unwind` with `AssertUnwindSafe`, returning a
/// default value if a panic occurs.
macro_rules! ffi_catch {
    ($default:expr, $body:expr) => {
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| $body)) {
            Ok(val) => val,
            Err(_) => $default,
        }
    };
}

// ============================================================
// Document Lifecycle
// ============================================================

/// Creates a new, empty XML document.
///
/// The returned pointer must eventually be freed with [`tx_document_free`].
///
/// # Safety
///
/// The caller must free the returned pointer with [`tx_document_free`] when
/// done. Returns null on allocation failure.
#[unsafe(no_mangle)]
pub extern "C" fn tx_document_new() -> *mut TxDocument {
    ffi_catch!(ptr::null_mut(), {
        Box::into_raw(Box::new(TxDocument::new()))
    })
}

/// Frees a document previously created with [`tx_document_new`].
///
/// # Safety
///
/// `doc` must be a valid pointer returned by [`tx_document_new`], or null.
/// After this call, the pointer must not be used again.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tx_document_free(doc: *mut TxDocument) {
    if doc.is_null() {
        return;
    }
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        drop(unsafe { Box::from_raw(doc) });
    }));
}

/// Clears the document, removing all nodes and resetting to an empty state.
///
/// # Safety
///
/// `doc` must be a valid, non-null pointer to a `TxDocument`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tx_document_clear(doc: *mut TxDocument) {
    if doc.is_null() {
        return;
    }
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let doc = unsafe { &mut *doc };
        doc.invalidate_caches();
        doc.doc.clear();
    }));
}

/// Parses an XML string into the document, replacing any existing content.
///
/// # Safety
///
/// - `doc` must be a valid, non-null pointer to a `TxDocument`.
/// - `xml` must be a valid, non-null pointer to a null-terminated UTF-8 C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tx_document_parse(doc: *mut TxDocument, xml: *const c_char) -> TxError {
    if doc.is_null() {
        return TxError::TxErrorInvalidNodeId;
    }
    ffi_catch!(TxError::TxErrorInvalidNodeId, {
        let doc = unsafe { &mut *doc };
        let Some(xml_str) = (unsafe { cstr_to_str(xml) }) else {
            return TxError::TxErrorEmptyDocument;
        };
        doc.invalidate_caches();
        TxError::from_result(&doc.doc.parse_str(xml_str))
    })
}

/// Loads and parses an XML file.
///
/// # Safety
///
/// - `doc` must be a valid, non-null pointer to a `TxDocument`.
/// - `path` must be a valid, non-null pointer to a null-terminated UTF-8 C string
///   containing a filesystem path.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tx_document_load_file(
    doc: *mut TxDocument,
    path: *const c_char,
) -> TxError {
    if doc.is_null() {
        return TxError::TxErrorInvalidNodeId;
    }
    ffi_catch!(TxError::TxErrorInvalidNodeId, {
        let doc = unsafe { &mut *doc };
        let Some(path_str) = (unsafe { cstr_to_str(path) }) else {
            return TxError::TxErrorFileNotFound;
        };
        doc.invalidate_caches();
        TxError::from_result(&doc.doc.load_file_mut(path_str))
    })
}

/// Saves the document to a file (pretty-printed).
///
/// # Safety
///
/// - `doc` must be a valid, non-null pointer to a `TxDocument`.
/// - `path` must be a valid, non-null pointer to a null-terminated UTF-8 C string
///   containing a filesystem path.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tx_document_save_file(
    doc: *const TxDocument,
    path: *const c_char,
) -> TxError {
    if doc.is_null() {
        return TxError::TxErrorInvalidNodeId;
    }
    ffi_catch!(TxError::TxErrorInvalidNodeId, {
        let doc = unsafe { &*doc };
        let Some(path_str) = (unsafe { cstr_to_str(path) }) else {
            return TxError::TxErrorFileNotFound;
        };
        TxError::from_result(&doc.doc.save_file(path_str))
    })
}

/// Returns the pretty-printed XML string for the document.
///
/// The returned pointer is valid until the next mutating operation on the
/// document or until the document is freed.
///
/// # Safety
///
/// `doc` must be a valid, non-null pointer to a `TxDocument`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tx_document_to_string(doc: *mut TxDocument) -> *const c_char {
    if doc.is_null() {
        return ptr::null();
    }
    ffi_catch!(ptr::null(), {
        let doc = unsafe { &mut *doc };
        let s = doc.doc.to_string();
        match CString::new(s) {
            Ok(cs) => {
                let ptr = cs.as_ptr();
                doc.cached_to_string = Some(cs);
                ptr
            }
            Err(_) => ptr::null(),
        }
    })
}

/// Returns the compact XML string for the document.
///
/// The returned pointer is valid until the next mutating operation on the
/// document or until the document is freed.
///
/// # Safety
///
/// `doc` must be a valid, non-null pointer to a `TxDocument`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tx_document_to_string_compact(doc: *mut TxDocument) -> *const c_char {
    if doc.is_null() {
        return ptr::null();
    }
    ffi_catch!(ptr::null(), {
        let doc = unsafe { &mut *doc };
        let s = doc.doc.to_string_compact();
        match CString::new(s) {
            Ok(cs) => {
                let ptr = cs.as_ptr();
                doc.cached_to_string_compact = Some(cs);
                ptr
            }
            Err(_) => ptr::null(),
        }
    })
}

/// Returns the current error code of the document.
///
/// Returns [`TxError::TxSuccess`] if no error has occurred.
///
/// # Safety
///
/// `doc` must be a valid, non-null pointer to a `TxDocument`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tx_document_error(doc: *const TxDocument) -> TxError {
    if doc.is_null() {
        return TxError::TxErrorInvalidNodeId;
    }
    ffi_catch!(TxError::TxErrorInvalidNodeId, {
        let doc = unsafe { &*doc };
        match doc.doc.error() {
            Some(e) => TxError::from_xml_error(&e),
            None => TxError::TxSuccess,
        }
    })
}

/// Returns the line number of the current error, or 0 if no error.
///
/// # Safety
///
/// `doc` must be a valid, non-null pointer to a `TxDocument`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tx_document_error_line(doc: *const TxDocument) -> c_int {
    if doc.is_null() {
        return 0;
    }
    ffi_catch!(0, {
        let doc = unsafe { &*doc };
        doc.doc.error_line().unwrap_or(0) as c_int
    })
}

/// Returns the error name string, or null if no error.
///
/// The returned pointer is valid until the next mutating operation on the
/// document or until the document is freed.
///
/// # Safety
///
/// `doc` must be a valid, non-null pointer to a `TxDocument`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tx_document_error_name(doc: *mut TxDocument) -> *const c_char {
    if doc.is_null() {
        return ptr::null();
    }
    ffi_catch!(ptr::null(), {
        let doc = unsafe { &mut *doc };
        match doc.doc.error() {
            Some(e) => {
                let name = e.name();
                match CString::new(name) {
                    Ok(cs) => {
                        let ptr = cs.as_ptr();
                        doc.cached_error_name = Some(cs);
                        ptr
                    }
                    Err(_) => ptr::null(),
                }
            }
            None => ptr::null(),
        }
    })
}

// ============================================================
// Factory Functions
// ============================================================

/// Creates a new element node with the given tag name.
///
/// # Safety
///
/// - `doc` must be a valid, non-null pointer to a `TxDocument`.
/// - `name` must be a valid, non-null pointer to a null-terminated UTF-8 string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tx_new_element(doc: *mut TxDocument, name: *const c_char) -> TxNodeId {
    if doc.is_null() {
        return TX_NULL_NODE;
    }
    ffi_catch!(TX_NULL_NODE, {
        let doc = unsafe { &mut *doc };
        let Some(name_str) = (unsafe { cstr_to_str(name) }) else {
            return TX_NULL_NODE;
        };
        doc.invalidate_caches();
        TxNodeId::from_node_id(doc.doc.new_element(name_str))
    })
}

/// Creates a new text node with the given content.
///
/// # Safety
///
/// - `doc` must be a valid, non-null pointer to a `TxDocument`.
/// - `text` must be a valid, non-null pointer to a null-terminated UTF-8 string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tx_new_text(doc: *mut TxDocument, text: *const c_char) -> TxNodeId {
    if doc.is_null() {
        return TX_NULL_NODE;
    }
    ffi_catch!(TX_NULL_NODE, {
        let doc = unsafe { &mut *doc };
        let Some(text_str) = (unsafe { cstr_to_str(text) }) else {
            return TX_NULL_NODE;
        };
        doc.invalidate_caches();
        TxNodeId::from_node_id(doc.doc.new_text(text_str))
    })
}

/// Creates a new comment node.
///
/// # Safety
///
/// - `doc` must be a valid, non-null pointer to a `TxDocument`.
/// - `text` must be a valid, non-null pointer to a null-terminated UTF-8 string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tx_new_comment(doc: *mut TxDocument, text: *const c_char) -> TxNodeId {
    if doc.is_null() {
        return TX_NULL_NODE;
    }
    ffi_catch!(TX_NULL_NODE, {
        let doc = unsafe { &mut *doc };
        let Some(text_str) = (unsafe { cstr_to_str(text) }) else {
            return TX_NULL_NODE;
        };
        doc.invalidate_caches();
        TxNodeId::from_node_id(doc.doc.new_comment(text_str))
    })
}

/// Creates a new declaration node.
///
/// # Safety
///
/// - `doc` must be a valid, non-null pointer to a `TxDocument`.
/// - `text` must be a valid, non-null pointer to a null-terminated UTF-8 string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tx_new_declaration(doc: *mut TxDocument, text: *const c_char) -> TxNodeId {
    if doc.is_null() {
        return TX_NULL_NODE;
    }
    ffi_catch!(TX_NULL_NODE, {
        let doc = unsafe { &mut *doc };
        let Some(text_str) = (unsafe { cstr_to_str(text) }) else {
            return TX_NULL_NODE;
        };
        doc.invalidate_caches();
        TxNodeId::from_node_id(doc.doc.new_declaration(text_str))
    })
}

/// Creates a new "unknown" node.
///
/// # Safety
///
/// - `doc` must be a valid, non-null pointer to a `TxDocument`.
/// - `text` must be a valid, non-null pointer to a null-terminated UTF-8 string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tx_new_unknown(doc: *mut TxDocument, text: *const c_char) -> TxNodeId {
    if doc.is_null() {
        return TX_NULL_NODE;
    }
    ffi_catch!(TX_NULL_NODE, {
        let doc = unsafe { &mut *doc };
        let Some(text_str) = (unsafe { cstr_to_str(text) }) else {
            return TX_NULL_NODE;
        };
        doc.invalidate_caches();
        TxNodeId::from_node_id(doc.doc.new_unknown(text_str))
    })
}

// ============================================================
// DOM Tree Mutations
// ============================================================

/// Inserts `child` as the last child of `parent`.
///
/// # Safety
///
/// `doc` must be a valid, non-null pointer to a `TxDocument`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tx_insert_end_child(
    doc: *mut TxDocument,
    parent: TxNodeId,
    child: TxNodeId,
) -> TxError {
    if doc.is_null() {
        return TxError::TxErrorInvalidNodeId;
    }
    ffi_catch!(TxError::TxErrorInvalidNodeId, {
        let doc = unsafe { &mut *doc };
        doc.invalidate_caches();
        let result = doc
            .doc
            .insert_end_child(parent.to_node_id(), child.to_node_id());
        TxError::from_result(&result)
    })
}

/// Inserts `child` as the first child of `parent`.
///
/// # Safety
///
/// `doc` must be a valid, non-null pointer to a `TxDocument`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tx_insert_first_child(
    doc: *mut TxDocument,
    parent: TxNodeId,
    child: TxNodeId,
) -> TxError {
    if doc.is_null() {
        return TxError::TxErrorInvalidNodeId;
    }
    ffi_catch!(TxError::TxErrorInvalidNodeId, {
        let doc = unsafe { &mut *doc };
        doc.invalidate_caches();
        let result = doc
            .doc
            .insert_first_child(parent.to_node_id(), child.to_node_id());
        TxError::from_result(&result)
    })
}

/// Inserts `child` as the next sibling after `after`.
///
/// # Safety
///
/// `doc` must be a valid, non-null pointer to a `TxDocument`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tx_insert_after_child(
    doc: *mut TxDocument,
    after: TxNodeId,
    child: TxNodeId,
) -> TxError {
    if doc.is_null() {
        return TxError::TxErrorInvalidNodeId;
    }
    ffi_catch!(TxError::TxErrorInvalidNodeId, {
        let doc = unsafe { &mut *doc };
        doc.invalidate_caches();
        let result = doc
            .doc
            .insert_after_child(after.to_node_id(), child.to_node_id());
        TxError::from_result(&result)
    })
}

/// Deletes `child` from `parent`.
///
/// # Safety
///
/// `doc` must be a valid, non-null pointer to a `TxDocument`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tx_delete_child(
    doc: *mut TxDocument,
    parent: TxNodeId,
    child: TxNodeId,
) -> TxError {
    if doc.is_null() {
        return TxError::TxErrorInvalidNodeId;
    }
    ffi_catch!(TxError::TxErrorInvalidNodeId, {
        let doc = unsafe { &mut *doc };
        doc.invalidate_caches();
        let result = doc
            .doc
            .delete_child(parent.to_node_id(), child.to_node_id());
        TxError::from_result(&result)
    })
}

/// Deletes all children of `parent`.
///
/// # Safety
///
/// `doc` must be a valid, non-null pointer to a `TxDocument`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tx_delete_children(doc: *mut TxDocument, parent: TxNodeId) -> TxError {
    if doc.is_null() {
        return TxError::TxErrorInvalidNodeId;
    }
    ffi_catch!(TxError::TxErrorInvalidNodeId, {
        let doc = unsafe { &mut *doc };
        doc.invalidate_caches();
        TxError::from_result(&doc.doc.delete_children(parent.to_node_id()))
    })
}

/// Deletes a node and all its descendants from the document.
///
/// # Safety
///
/// `doc` must be a valid, non-null pointer to a `TxDocument`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tx_delete_node(doc: *mut TxDocument, node: TxNodeId) -> TxError {
    if doc.is_null() {
        return TxError::TxErrorInvalidNodeId;
    }
    ffi_catch!(TxError::TxErrorInvalidNodeId, {
        let doc = unsafe { &mut *doc };
        doc.invalidate_caches();
        TxError::from_result(&doc.doc.delete_node(node.to_node_id()))
    })
}

// ============================================================
// DOM Navigation
// ============================================================

/// Returns the parent of the given node, or `TX_NULL_NODE` if none.
///
/// # Safety
///
/// `doc` must be a valid, non-null pointer to a `TxDocument`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tx_parent(doc: *const TxDocument, node: TxNodeId) -> TxNodeId {
    if doc.is_null() {
        return TX_NULL_NODE;
    }
    ffi_catch!(TX_NULL_NODE, {
        let doc = unsafe { &*doc };
        TxNodeId::from_option(doc.doc.parent(node.to_node_id()))
    })
}

/// Returns the first child of the given node, or `TX_NULL_NODE` if none.
///
/// # Safety
///
/// `doc` must be a valid, non-null pointer to a `TxDocument`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tx_first_child(doc: *const TxDocument, node: TxNodeId) -> TxNodeId {
    if doc.is_null() {
        return TX_NULL_NODE;
    }
    ffi_catch!(TX_NULL_NODE, {
        let doc = unsafe { &*doc };
        TxNodeId::from_option(doc.doc.first_child(node.to_node_id()))
    })
}

/// Returns the last child of the given node, or `TX_NULL_NODE` if none.
///
/// # Safety
///
/// `doc` must be a valid, non-null pointer to a `TxDocument`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tx_last_child(doc: *const TxDocument, node: TxNodeId) -> TxNodeId {
    if doc.is_null() {
        return TX_NULL_NODE;
    }
    ffi_catch!(TX_NULL_NODE, {
        let doc = unsafe { &*doc };
        TxNodeId::from_option(doc.doc.last_child(node.to_node_id()))
    })
}

/// Returns the previous sibling of the given node, or `TX_NULL_NODE` if none.
///
/// # Safety
///
/// `doc` must be a valid, non-null pointer to a `TxDocument`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tx_prev_sibling(doc: *const TxDocument, node: TxNodeId) -> TxNodeId {
    if doc.is_null() {
        return TX_NULL_NODE;
    }
    ffi_catch!(TX_NULL_NODE, {
        let doc = unsafe { &*doc };
        TxNodeId::from_option(doc.doc.prev_sibling(node.to_node_id()))
    })
}

/// Returns the next sibling of the given node, or `TX_NULL_NODE` if none.
///
/// # Safety
///
/// `doc` must be a valid, non-null pointer to a `TxDocument`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tx_next_sibling(doc: *const TxDocument, node: TxNodeId) -> TxNodeId {
    if doc.is_null() {
        return TX_NULL_NODE;
    }
    ffi_catch!(TX_NULL_NODE, {
        let doc = unsafe { &*doc };
        TxNodeId::from_option(doc.doc.next_sibling(node.to_node_id()))
    })
}

/// Returns the first child element, optionally filtered by tag name.
///
/// If `name` is null, returns the first child element regardless of its name.
///
/// # Safety
///
/// - `doc` must be a valid, non-null pointer to a `TxDocument`.
/// - `name`, if non-null, must be a valid null-terminated UTF-8 string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tx_first_child_element(
    doc: *const TxDocument,
    node: TxNodeId,
    name: *const c_char,
) -> TxNodeId {
    if doc.is_null() {
        return TX_NULL_NODE;
    }
    ffi_catch!(TX_NULL_NODE, {
        let doc = unsafe { &*doc };
        let name_opt = unsafe { cstr_to_str(name) };
        TxNodeId::from_option(doc.doc.first_child_element(node.to_node_id(), name_opt))
    })
}

/// Returns the next sibling element, optionally filtered by tag name.
///
/// If `name` is null, returns the next sibling element regardless of its name.
///
/// # Safety
///
/// - `doc` must be a valid, non-null pointer to a `TxDocument`.
/// - `name`, if non-null, must be a valid null-terminated UTF-8 string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tx_next_sibling_element(
    doc: *const TxDocument,
    node: TxNodeId,
    name: *const c_char,
) -> TxNodeId {
    if doc.is_null() {
        return TX_NULL_NODE;
    }
    ffi_catch!(TX_NULL_NODE, {
        let doc = unsafe { &*doc };
        let name_opt = unsafe { cstr_to_str(name) };
        TxNodeId::from_option(doc.doc.next_sibling_element(node.to_node_id(), name_opt))
    })
}

/// Returns the root element of the document, or `TX_NULL_NODE` if the
/// document is empty.
///
/// # Safety
///
/// `doc` must be a valid, non-null pointer to a `TxDocument`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tx_root_element(doc: *const TxDocument) -> TxNodeId {
    if doc.is_null() {
        return TX_NULL_NODE;
    }
    ffi_catch!(TX_NULL_NODE, {
        let doc = unsafe { &*doc };
        TxNodeId::from_option(doc.doc.root_element())
    })
}

// ============================================================
// Element & Attribute Helpers
// ============================================================

/// Returns the tag name of an element node.
///
/// The returned pointer is valid until the next mutating operation on the
/// document or until the document is freed.
///
/// # Safety
///
/// - `doc` must be a valid, non-null pointer to a `TxDocument`.
/// - `element` must identify an element node.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tx_element_name(doc: *mut TxDocument, element: TxNodeId) -> *const c_char {
    if doc.is_null() {
        return ptr::null();
    }
    ffi_catch!(ptr::null(), {
        let doc = unsafe { &mut *doc };
        let val = doc
            .doc
            .node_ref(element.to_node_id())
            .map(|nr| nr.value().to_owned());
        match val {
            Some(s) => cache_str(doc, &s),
            None => ptr::null(),
        }
    })
}

/// Returns the value of the named attribute on an element.
///
/// Returns null if the element or attribute does not exist.
///
/// # Safety
///
/// - `doc` must be a valid, non-null pointer to a `TxDocument`.
/// - `name` must be a valid, non-null pointer to a null-terminated UTF-8 string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tx_element_attribute(
    doc: *mut TxDocument,
    el: TxNodeId,
    name: *const c_char,
) -> *const c_char {
    if doc.is_null() {
        return ptr::null();
    }
    ffi_catch!(ptr::null(), {
        let doc = unsafe { &mut *doc };
        let Some(name_str) = (unsafe { cstr_to_str(name) }) else {
            return ptr::null();
        };
        let val = doc
            .doc
            .attribute(el.to_node_id(), name_str)
            .map(str::to_owned);
        match val {
            Some(s) => cache_str(doc, &s),
            None => ptr::null(),
        }
    })
}

/// Sets an attribute on an element. Creates the attribute if it doesn't exist,
/// or updates its value if it does.
///
/// # Safety
///
/// - `doc` must be a valid, non-null pointer to a `TxDocument`.
/// - `name` and `value` must be valid, non-null pointers to null-terminated
///   UTF-8 strings.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tx_element_set_attribute(
    doc: *mut TxDocument,
    el: TxNodeId,
    name: *const c_char,
    value: *const c_char,
) -> TxError {
    if doc.is_null() {
        return TxError::TxErrorInvalidNodeId;
    }
    ffi_catch!(TxError::TxErrorInvalidNodeId, {
        let doc = unsafe { &mut *doc };
        let Some(name_str) = (unsafe { cstr_to_str(name) }) else {
            return TxError::TxErrorNoAttribute;
        };
        let Some(val_str) = (unsafe { cstr_to_str(value) }) else {
            return TxError::TxErrorNoAttribute;
        };
        doc.invalidate_caches();
        TxError::from_result(&doc.doc.set_attribute(el.to_node_id(), name_str, val_str))
    })
}

/// Deletes an attribute from an element by name.
///
/// # Safety
///
/// - `doc` must be a valid, non-null pointer to a `TxDocument`.
/// - `name` must be a valid, non-null pointer to a null-terminated UTF-8 string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tx_element_delete_attribute(
    doc: *mut TxDocument,
    el: TxNodeId,
    name: *const c_char,
) -> TxError {
    if doc.is_null() {
        return TxError::TxErrorInvalidNodeId;
    }
    ffi_catch!(TxError::TxErrorInvalidNodeId, {
        let doc = unsafe { &mut *doc };
        let Some(name_str) = (unsafe { cstr_to_str(name) }) else {
            return TxError::TxErrorNoAttribute;
        };
        doc.invalidate_caches();
        TxError::from_result(&doc.doc.delete_attribute(el.to_node_id(), name_str))
    })
}

/// Returns the text content of an element's first child text node.
///
/// Returns null if no text child exists.
///
/// # Safety
///
/// - `doc` must be a valid, non-null pointer to a `TxDocument`.
/// - `element` must identify an element node.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tx_element_get_text(
    doc: *mut TxDocument,
    element: TxNodeId,
) -> *const c_char {
    if doc.is_null() {
        return ptr::null();
    }
    ffi_catch!(ptr::null(), {
        let doc = unsafe { &mut *doc };
        let val = doc.doc.get_text(element.to_node_id()).map(str::to_owned);
        match val {
            Some(s) => cache_str(doc, &s),
            None => ptr::null(),
        }
    })
}

/// Sets the text content of an element.
///
/// If a child text node exists, its content is replaced. Otherwise, a new
/// text node is created and inserted as the first child.
///
/// # Safety
///
/// - `doc` must be a valid, non-null pointer to a `TxDocument`.
/// - `text` must be a valid, non-null pointer to a null-terminated UTF-8 string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tx_element_set_text(
    doc: *mut TxDocument,
    element: TxNodeId,
    text: *const c_char,
) -> TxError {
    if doc.is_null() {
        return TxError::TxErrorInvalidNodeId;
    }
    ffi_catch!(TxError::TxErrorInvalidNodeId, {
        let doc = unsafe { &mut *doc };
        let Some(text_str) = (unsafe { cstr_to_str(text) }) else {
            return TxError::TxErrorNoTextNode;
        };
        doc.invalidate_caches();
        TxError::from_result(&doc.doc.set_text(element.to_node_id(), text_str))
    })
}

// ============================================================
// Typed Attribute Accessors
// ============================================================

/// Queries an integer attribute value.
///
/// On success, writes the value to `*value` and returns `TxSuccess`.
///
/// # Safety
///
/// - `doc` must be a valid, non-null pointer to a `TxDocument`.
/// - `name` must be a valid, non-null pointer to a null-terminated UTF-8 string.
/// - `value` must be a valid, non-null pointer to a `c_int`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tx_query_int_attribute(
    doc: *const TxDocument,
    el: TxNodeId,
    name: *const c_char,
    value: *mut c_int,
) -> TxError {
    if doc.is_null() || value.is_null() {
        return TxError::TxErrorInvalidNodeId;
    }
    ffi_catch!(TxError::TxErrorInvalidNodeId, {
        let doc = unsafe { &*doc };
        let Some(name_str) = (unsafe { cstr_to_str(name) }) else {
            return TxError::TxErrorNoAttribute;
        };
        match doc.doc.query_int_attribute(el.to_node_id(), name_str) {
            Ok(v) => {
                unsafe { *value = v as c_int };
                TxError::TxSuccess
            }
            Err(e) => TxError::from_xml_error(&e),
        }
    })
}

/// Queries a boolean attribute value.
///
/// On success, writes the value to `*value` and returns `TxSuccess`.
///
/// # Safety
///
/// - `doc` must be a valid, non-null pointer to a `TxDocument`.
/// - `name` must be a valid, non-null pointer to a null-terminated UTF-8 string.
/// - `value` must be a valid, non-null pointer to a `bool`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tx_query_bool_attribute(
    doc: *const TxDocument,
    el: TxNodeId,
    name: *const c_char,
    value: *mut bool,
) -> TxError {
    if doc.is_null() || value.is_null() {
        return TxError::TxErrorInvalidNodeId;
    }
    ffi_catch!(TxError::TxErrorInvalidNodeId, {
        let doc = unsafe { &*doc };
        let Some(name_str) = (unsafe { cstr_to_str(name) }) else {
            return TxError::TxErrorNoAttribute;
        };
        match doc.doc.query_bool_attribute(el.to_node_id(), name_str) {
            Ok(v) => {
                unsafe { *value = v };
                TxError::TxSuccess
            }
            Err(e) => TxError::from_xml_error(&e),
        }
    })
}

/// Queries a double (f64) attribute value.
///
/// On success, writes the value to `*value` and returns `TxSuccess`.
///
/// # Safety
///
/// - `doc` must be a valid, non-null pointer to a `TxDocument`.
/// - `name` must be a valid, non-null pointer to a null-terminated UTF-8 string.
/// - `value` must be a valid, non-null pointer to a `c_double`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tx_query_double_attribute(
    doc: *const TxDocument,
    el: TxNodeId,
    name: *const c_char,
    value: *mut c_double,
) -> TxError {
    if doc.is_null() || value.is_null() {
        return TxError::TxErrorInvalidNodeId;
    }
    ffi_catch!(TxError::TxErrorInvalidNodeId, {
        let doc = unsafe { &*doc };
        let Some(name_str) = (unsafe { cstr_to_str(name) }) else {
            return TxError::TxErrorNoAttribute;
        };
        match doc.doc.query_double_attribute(el.to_node_id(), name_str) {
            Ok(v) => {
                unsafe { *value = v };
                TxError::TxSuccess
            }
            Err(e) => TxError::from_xml_error(&e),
        }
    })
}

/// Returns an integer attribute value, or `default_val` if the attribute
/// does not exist or cannot be parsed.
///
/// # Safety
///
/// - `doc` must be a valid, non-null pointer to a `TxDocument`.
/// - `name` must be a valid, non-null pointer to a null-terminated UTF-8 string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tx_int_attribute(
    doc: *const TxDocument,
    el: TxNodeId,
    name: *const c_char,
    default_val: c_int,
) -> c_int {
    if doc.is_null() {
        return default_val;
    }
    ffi_catch!(default_val, {
        let doc = unsafe { &*doc };
        let Some(name_str) = (unsafe { cstr_to_str(name) }) else {
            return default_val;
        };
        doc.doc
            .int_attribute(el.to_node_id(), name_str, default_val)
    })
}

/// Returns a boolean attribute value, or `default_val` if the attribute
/// does not exist or cannot be parsed.
///
/// # Safety
///
/// - `doc` must be a valid, non-null pointer to a `TxDocument`.
/// - `name` must be a valid, non-null pointer to a null-terminated UTF-8 string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tx_bool_attribute(
    doc: *const TxDocument,
    el: TxNodeId,
    name: *const c_char,
    default_val: bool,
) -> bool {
    if doc.is_null() {
        return default_val;
    }
    ffi_catch!(default_val, {
        let doc = unsafe { &*doc };
        let Some(name_str) = (unsafe { cstr_to_str(name) }) else {
            return default_val;
        };
        doc.doc
            .bool_attribute(el.to_node_id(), name_str, default_val)
    })
}

/// Returns a double (f64) attribute value, or `default_val` if the attribute
/// does not exist or cannot be parsed.
///
/// # Safety
///
/// - `doc` must be a valid, non-null pointer to a `TxDocument`.
/// - `name` must be a valid, non-null pointer to a null-terminated UTF-8 string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tx_double_attribute(
    doc: *const TxDocument,
    el: TxNodeId,
    name: *const c_char,
    default_val: c_double,
) -> c_double {
    if doc.is_null() {
        return default_val;
    }
    ffi_catch!(default_val, {
        let doc = unsafe { &*doc };
        let Some(name_str) = (unsafe { cstr_to_str(name) }) else {
            return default_val;
        };
        doc.doc
            .double_attribute(el.to_node_id(), name_str, default_val)
    })
}

// ============================================================
// Printer / Streaming API
// ============================================================

/// Creates a new XML printer (pretty-print mode).
///
/// # Safety
///
/// The returned pointer must eventually be freed with [`tx_printer_free`].
#[unsafe(no_mangle)]
pub extern "C" fn tx_printer_new() -> *mut TxPrinter {
    ffi_catch!(ptr::null_mut(), {
        Box::into_raw(Box::new(TxPrinter::new(false)))
    })
}

/// Creates a new XML printer (compact mode, no whitespace).
///
/// # Safety
///
/// The returned pointer must eventually be freed with [`tx_printer_free`].
#[unsafe(no_mangle)]
pub extern "C" fn tx_printer_new_compact() -> *mut TxPrinter {
    ffi_catch!(ptr::null_mut(), {
        Box::into_raw(Box::new(TxPrinter::new(true)))
    })
}

/// Frees a printer previously created with [`tx_printer_new`] or
/// [`tx_printer_new_compact`].
///
/// # Safety
///
/// `printer` must be a valid pointer returned by a printer constructor, or null.
/// After this call, the pointer must not be used again.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tx_printer_free(printer: *mut TxPrinter) {
    if printer.is_null() {
        return;
    }
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        drop(unsafe { Box::from_raw(printer) });
    }));
}

/// Opens an element tag in the printer output.
///
/// # Safety
///
/// - `printer` must be a valid, non-null pointer to a `TxPrinter`.
/// - `name` must be a valid, non-null pointer to a null-terminated UTF-8 string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tx_printer_open_element(printer: *mut TxPrinter, name: *const c_char) {
    if printer.is_null() {
        return;
    }
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let p = unsafe { &mut *printer };
        if let Some(name_str) = unsafe { cstr_to_str(name) } {
            p.cached_result = None;
            p.printer.open_element(name_str);
        }
    }));
}

/// Pushes an attribute onto the currently open element.
///
/// # Safety
///
/// - `printer` must be a valid, non-null pointer to a `TxPrinter`.
/// - `name` and `value` must be valid, non-null pointers to null-terminated
///   UTF-8 strings.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tx_printer_push_attribute(
    printer: *mut TxPrinter,
    name: *const c_char,
    value: *const c_char,
) {
    if printer.is_null() {
        return;
    }
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let p = unsafe { &mut *printer };
        if let (Some(n), Some(v)) = (unsafe { cstr_to_str(name) }, unsafe { cstr_to_str(value) }) {
            p.cached_result = None;
            p.printer.push_attribute(n, v);
        }
    }));
}

/// Closes the most recently opened element.
///
/// # Safety
///
/// `printer` must be a valid, non-null pointer to a `TxPrinter`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tx_printer_close_element(printer: *mut TxPrinter) {
    if printer.is_null() {
        return;
    }
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let p = unsafe { &mut *printer };
        p.cached_result = None;
        p.printer.close_element();
    }));
}

/// Pushes text content into the current element.
///
/// # Safety
///
/// - `printer` must be a valid, non-null pointer to a `TxPrinter`.
/// - `text` must be a valid, non-null pointer to a null-terminated UTF-8 string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tx_printer_push_text(printer: *mut TxPrinter, text: *const c_char) {
    if printer.is_null() {
        return;
    }
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let p = unsafe { &mut *printer };
        if let Some(text_str) = unsafe { cstr_to_str(text) } {
            p.cached_result = None;
            p.printer.push_text(text_str);
        }
    }));
}

/// Pushes a comment into the printer output.
///
/// # Safety
///
/// - `printer` must be a valid, non-null pointer to a `TxPrinter`.
/// - `text` must be a valid, non-null pointer to a null-terminated UTF-8 string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tx_printer_push_comment(printer: *mut TxPrinter, text: *const c_char) {
    if printer.is_null() {
        return;
    }
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let p = unsafe { &mut *printer };
        if let Some(text_str) = unsafe { cstr_to_str(text) } {
            p.cached_result = None;
            p.printer.push_comment(text_str);
        }
    }));
}

/// Returns the accumulated printer output as a C string.
///
/// The returned pointer is valid until the printer is modified or freed.
///
/// # Safety
///
/// `printer` must be a valid, non-null pointer to a `TxPrinter`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tx_printer_result(printer: *mut TxPrinter) -> *const c_char {
    if printer.is_null() {
        return ptr::null();
    }
    ffi_catch!(ptr::null(), {
        let p = unsafe { &mut *printer };
        let s = p.printer.result();
        match CString::new(s) {
            Ok(cs) => {
                let ptr = cs.as_ptr();
                p.cached_result = Some(cs);
                ptr
            }
            Err(_) => ptr::null(),
        }
    })
}

/// Clears the printer output, resetting it to empty.
///
/// # Safety
///
/// `printer` must be a valid, non-null pointer to a `TxPrinter`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tx_printer_clear(printer: *mut TxPrinter) {
    if printer.is_null() {
        return;
    }
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let p = unsafe { &mut *printer };
        p.cached_result = None;
        p.printer.clear();
    }));
}

// ============================================================
// Node Type Inspection
// ============================================================

/// Returns the type of the given node.
///
/// # Safety
///
/// `doc` must be a valid, non-null pointer to a `TxDocument`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tx_node_type(doc: *const TxDocument, node: TxNodeId) -> TxNodeType {
    if doc.is_null() {
        return TxNodeType::TxNodeUnknown;
    }
    ffi_catch!(TxNodeType::TxNodeUnknown, {
        let doc = unsafe { &*doc };
        match doc.doc.node_kind(node.to_node_id()) {
            Some(kind) => TxNodeType::from_node_kind(kind),
            None => TxNodeType::TxNodeUnknown,
        }
    })
}

/// Returns `true` if the given node ID is the null sentinel.
///
/// This function is safe to call without a document pointer.
#[unsafe(no_mangle)]
pub extern "C" fn tx_node_is_null(node: TxNodeId) -> bool {
    node.is_null()
}

/// Returns the "value" of a node based on its type:
///
/// - Element → tag name
/// - Text → text content
/// - Comment → comment text
/// - Declaration → declaration name
/// - Unknown → raw content
/// - Document → empty string
///
/// # Safety
///
/// `doc` must be a valid, non-null pointer to a `TxDocument`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tx_node_value(doc: *mut TxDocument, node: TxNodeId) -> *const c_char {
    if doc.is_null() {
        return ptr::null();
    }
    ffi_catch!(ptr::null(), {
        let doc = unsafe { &mut *doc };
        let val = doc
            .doc
            .node_ref(node.to_node_id())
            .map(|nr| nr.value().to_owned());
        match val {
            Some(s) => cache_str(doc, &s),
            None => ptr::null(),
        }
    })
}

/// Returns the 1-based source line number where the node was parsed,
/// or 0 if the node was not created by parsing.
///
/// # Safety
///
/// `doc` must be a valid, non-null pointer to a `TxDocument`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tx_node_line(doc: *const TxDocument, node: TxNodeId) -> c_int {
    if doc.is_null() {
        return 0;
    }
    ffi_catch!(0, {
        let doc = unsafe { &*doc };
        doc.doc.line_num(node.to_node_id()).unwrap_or(0) as c_int
    })
}
