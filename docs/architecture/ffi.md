# FFI Architecture

## Overview

The `tinyxml2-capi` crate provides a **C-compatible Foreign Function Interface (FFI)** for the core `tinyxml2` Rust library. It produces both a shared library (`cdylib`) and a static library (`staticlib`), enabling C and C++ projects to use the Rust XML parser as a drop-in replacement for TinyXML2.

The FFI layer is a thin translation shim: it converts between C types and Rust types, manages ownership boundaries, and ensures that Rust panics never escape into C code. All `unsafe` code in the project is confined to this crate.

---

## Opaque Handle Design

C callers interact with the Rust `Document` through an **opaque pointer handle**:

```rust
/// Opaque handle to a Rust Document. C callers treat this as `void*`.
pub type TxmlDocument = *mut std::ffi::c_void;
```

### Lifecycle

```
C caller                          tinyxml2-capi                      tinyxml2 (core)
────────                          ─────────────                      ───────────────
txml2_document_new()  ──────────▶ Box::new(Document::new())
                                  Box::into_raw() as *mut c_void
                      ◀────────── return opaque handle
                                  
txml2_document_parse(doc, xml) ──▶ unsafe { &mut *(doc as *mut Document) }
                                   doc.parse(xml)
                      ◀────────── return error code

txml2_document_free(doc) ────────▶ unsafe { drop(Box::from_raw(doc as *mut Document)) }
                                   Document and all nodes freed
```

### Ownership rules

1. `txml2_document_new()` creates a `Box<Document>`, converts it to a raw pointer, and returns it to C. **Ownership transfers to C.**
2. C passes the handle to every API function. The FFI layer borrows it (unsafe dereference) for the duration of the call.
3. `txml2_document_free()` reclaims ownership via `Box::from_raw` and drops the `Document`, freeing all memory.
4. **Double-free protection:** Calling `txml2_document_free()` twice on the same handle is undefined behavior (same as C's `free()`). Documentation will warn callers.

---

## Naming Conventions

All exported `extern "C"` functions follow a naming convention designed to match TinyXML2's C++ API:

| Pattern | Example | Maps to |
|---|---|---|
| `txml2_document_*` | `txml2_document_new()` | `XMLDocument::XMLDocument()` |
| `txml2_document_parse` | `txml2_document_parse(doc, xml)` | `XMLDocument::Parse()` |
| `txml2_element_*` | `txml2_element_name(elem)` | `XMLElement::Name()` |
| `txml2_node_*` | `txml2_node_first_child(node)` | `XMLNode::FirstChild()` |
| `txml2_printer_*` | `txml2_printer_new()` | `XMLPrinter::XMLPrinter()` |

### Convention rules

- Prefix: `txml2_` (short for tinyxml2, avoids collisions with other libraries).
- Object type: `document_`, `element_`, `node_`, `printer_`, etc.
- Method name: lowercase with underscores, matching TinyXML2's method name (e.g., `FirstChild` → `first_child`).
- First argument: always the opaque handle (self pointer equivalent).

---

## Panic Safety

Rust panics that unwind across an `extern "C"` boundary cause **undefined behavior**. Every FFI function is wrapped in `std::panic::catch_unwind` to prevent this:

```rust
#[no_mangle]
pub extern "C" fn txml2_document_parse(
    doc: TxmlDocument,
    xml: *const std::os::raw::c_char,
) -> i32 {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        // Unsafe: dereference opaque handle
        let doc = unsafe { &mut *(doc as *mut Document) };
        // Unsafe: convert C string to Rust &str
        let xml = unsafe { std::ffi::CStr::from_ptr(xml) }
            .to_str()
            .map_err(|_| /* UTF-8 error */)?;
        
        doc.parse(xml)
    }));

    match result {
        Ok(Ok(())) => 0,                           // TXML2_SUCCESS
        Ok(Err(e)) => error_to_code(&e),           // Mapped error code
        Err(_panic) => TXML2_ERROR_INTERNAL,        // Panic caught — return generic error
    }
}
```

### Guarantees

- **No panic escapes.** Even if the Rust core has a bug that panics, the FFI layer catches it and returns an error code.
- **No stack unwinding through C frames.** `catch_unwind` stops the unwind before it crosses the FFI boundary.
- **AssertUnwindSafe.** The closure is wrapped in `AssertUnwindSafe` because FFI functions inherently deal with raw pointers that are not `UnwindSafe`. This is acceptable because the panic handler returns an error code rather than continuing to use potentially-inconsistent state.

---

## String Handling

### Incoming strings (C → Rust)

C strings are null-terminated `const char*`. The FFI layer converts them using `CStr`:

```rust
let c_str = unsafe { std::ffi::CStr::from_ptr(ptr) };
let rust_str: &str = c_str.to_str()?;  // Validates UTF-8
```

- **Null check:** All functions validate that the pointer is not null before dereferencing.
- **UTF-8 validation:** `CStr::to_str()` validates UTF-8. Invalid UTF-8 returns an error code to C.
- **Lifetime:** The `&str` borrows from the C caller's buffer. It is only valid for the duration of the FFI call.

### Outgoing strings (Rust → C)

Rust strings must be converted to null-terminated `CString` for C callers:

```rust
let rust_string: &str = element.name();
let c_string = std::ffi::CString::new(rust_string)?;
// Option A: Return pointer to internally-stored string (valid until next mutation)
// Option B: Return CString that caller must free
```

**Strategy:** For strings owned by the DOM (element names, text content, attribute values), the FFI layer returns a pointer into the Rust-owned string data. This pointer is valid until the next mutation of the document. This matches TinyXML2's behavior where `const char*` returns point into document-owned memory.

### Null termination

- Rust `&str` is **not** null-terminated. Direct return is unsafe.
- `CString::new()` adds null termination and checks for interior nulls.
- For performance-critical paths, the DOM may store strings with a trailing null byte to avoid re-allocation during FFI returns.

---

## Lifetime Management

The FFI layer follows TinyXML2's ownership model:

```
┌────────────────────────────────────────┐
│              C Caller                  │
│                                        │
│  doc = txml2_document_new();           │  ← allocates Document
│  txml2_document_parse(doc, xml);       │  ← populates DOM
│  elem = txml2_document_root(doc);      │  ← borrows into DOM
│  name = txml2_element_name(elem);      │  ← borrows string from DOM
│  // 'name' valid until doc is mutated  │
│  txml2_document_free(doc);             │  ← frees everything
│  // doc, elem, name all now invalid    │
└────────────────────────────────────────┘
```

### Rules

1. **Document owns everything.** All nodes, strings, and attributes are owned by the `Document` and freed when it is freed.
2. **No individual node free.** C callers cannot free individual nodes — they must use `txml2_document_delete_node(doc, node)` which goes through the `Document`'s deletion logic.
3. **Returned pointers are borrows.** String pointers (`const char*`) returned by the FFI are valid until the next mutating operation on the document. This matches TinyXML2's semantics.
4. **Node handles are opaque integers.** `NodeId` values may be exposed to C as opaque 64-bit integers (index + generation packed into a `uint64_t`). Stale handles return error codes rather than causing undefined behavior.

---

## Header Generation (Planned)

The project plans to use [cbindgen](https://github.com/eqrion/cbindgen) to auto-generate a C header file from the Rust FFI source:

```toml
# cbindgen.toml
language = "C"
header = "/* Auto-generated by cbindgen. Do not edit. */"
include_guard = "TINYXML2_RS_H"
autogen_warning = "/* Warning: this file is auto-generated by cbindgen. */"

[export]
prefix = "txml2"

[fn]
rename_args = "SnakeCase"
```

### Expected output (`tinyxml2.h`)

```c
#ifndef TINYXML2_RS_H
#define TINYXML2_RS_H

#include <stdint.h>

/* Opaque handle to a Document */
typedef void* TxmlDocument;

/* Error codes */
#define TXML2_SUCCESS                0
#define TXML2_ERROR_FILE_NOT_FOUND   1
#define TXML2_ERROR_FILE_COULD_NOT_BE_OPENED  2
#define TXML2_ERROR_FILE_READ_ERROR  3
#define TXML2_ERROR_PARSING_ELEMENT  4
/* ... */

/* Document API */
TxmlDocument txml2_document_new(void);
void txml2_document_free(TxmlDocument doc);
int32_t txml2_document_parse(TxmlDocument doc, const char* xml);
/* ... */

#endif /* TINYXML2_RS_H */
```

---

## Library Output

The `tinyxml2-capi` crate produces both shared and static libraries:

```toml
# tinyxml2-capi/Cargo.toml
[lib]
crate-type = ["cdylib", "staticlib"]
```

| Output | File (Linux) | File (macOS) | File (Windows) | Usage |
|---|---|---|---|---|
| `cdylib` | `libtinyxml2_capi.so` | `libtinyxml2_capi.dylib` | `tinyxml2_capi.dll` | Dynamic linking at runtime |
| `staticlib` | `libtinyxml2_capi.a` | `libtinyxml2_capi.a` | `tinyxml2_capi.lib` | Static linking at compile time |

### Linking from C

```bash
# Dynamic linking
gcc -o myapp myapp.c -L target/release -ltinyxml2_capi

# Static linking (must also link Rust runtime dependencies)
gcc -o myapp myapp.c -L target/release -ltinyxml2_capi -lpthread -ldl -lm
```

---

## Error Code Translation

Rust's `XmlError` enum is translated to C integer error codes matching TinyXML2's `XMLError` enum values:

| `XmlError` (Rust) | C Error Code | Value | TinyXML2 Equivalent |
|---|---|---|---|
| (success) | `TXML2_SUCCESS` | `0` | `XML_SUCCESS` |
| `Parse { kind: Element, .. }` | `TXML2_ERROR_PARSING_ELEMENT` | `4` | `XML_ERROR_PARSING_ELEMENT` |
| `Parse { kind: Attribute, .. }` | `TXML2_ERROR_PARSING_ATTRIBUTE` | `5` | `XML_ERROR_PARSING_ATTRIBUTE` |
| `Parse { kind: Text, .. }` | `TXML2_ERROR_PARSING_TEXT` | `6` | `XML_ERROR_PARSING_TEXT` |
| `Parse { kind: Cdata, .. }` | `TXML2_ERROR_PARSING_CDATA` | `7` | `XML_ERROR_PARSING_CDATA` |
| `Parse { kind: Comment, .. }` | `TXML2_ERROR_PARSING_COMMENT` | `8` | `XML_ERROR_PARSING_COMMENT` |
| `Parse { kind: Declaration, .. }` | `TXML2_ERROR_PARSING_DECLARATION` | `9` | `XML_ERROR_PARSING_DECLARATION` |
| `Parse { kind: Unknown, .. }` | `TXML2_ERROR_PARSING_UNKNOWN` | `10` | `XML_ERROR_PARSING_UNKNOWN` |
| `MismatchedElement { .. }` | `TXML2_ERROR_MISMATCHED_ELEMENT` | `11` | `XML_ERROR_MISMATCHED_ELEMENT` |
| `EmptyDocument` | `TXML2_ERROR_EMPTY_DOCUMENT` | `13` | `XML_ERROR_EMPTY_DOCUMENT` |
| `Io(_)` | `TXML2_ERROR_FILE_READ_ERROR` | `3` | `XML_ERROR_FILE_READ_ERROR` |
| `NoAttribute` | `TXML2_NO_ATTRIBUTE` | `14` | `XML_NO_ATTRIBUTE` |
| `WrongAttributeType` | `TXML2_WRONG_ATTRIBUTE_TYPE` | `15` | `XML_WRONG_ATTRIBUTE_TYPE` |
| `CanNotConvertText` | `TXML2_CAN_NOT_CONVERT_TEXT` | `16` | `XML_CAN_NOT_CONVERT_TEXT` |
| `NoTextNode` | `TXML2_NO_TEXT_NODE` | `17` | `XML_NO_TEXT_NODE` |
| `ElementDepthExceeded { .. }` | `TXML2_ELEMENT_DEPTH_EXCEEDED` | `18` | `XML_ELEMENT_DEPTH_EXCEEDED` |
| `InvalidNodeId` | `TXML2_ERROR_INTERNAL` | `99` | (no equivalent) |

### Translation function

```rust
fn error_to_code(err: &XmlError) -> i32 {
    match err {
        XmlError::Parse { kind: ParseErrorKind::Element, .. } => 4,
        XmlError::Parse { kind: ParseErrorKind::Attribute, .. } => 5,
        XmlError::Parse { kind: ParseErrorKind::Text, .. } => 6,
        XmlError::Parse { kind: ParseErrorKind::Cdata, .. } => 7,
        XmlError::Parse { kind: ParseErrorKind::Comment, .. } => 8,
        XmlError::Parse { kind: ParseErrorKind::Declaration, .. } => 9,
        XmlError::Parse { kind: ParseErrorKind::Unknown, .. } => 10,
        XmlError::MismatchedElement { .. } => 11,
        XmlError::EmptyDocument => 13,
        XmlError::Io(_) => 3,
        XmlError::NoAttribute => 14,
        XmlError::WrongAttributeType => 15,
        XmlError::CanNotConvertText => 16,
        XmlError::NoTextNode => 17,
        XmlError::ElementDepthExceeded { .. } => 18,
        XmlError::InvalidNodeId => 99,
        _ => 99, // Unknown/internal error
    }
}
```

---

## Thread Safety

The FFI layer provides the **same thread-safety guarantees as TinyXML2**:

- **Each `Document` is independent.** Different documents can be used from different threads concurrently without synchronization.
- **A single `Document` is NOT thread-safe.** Concurrent access to the same document from multiple threads requires external synchronization (e.g., a mutex).
- **No global mutable state.** The FFI layer has no global variables, static mutable state, or thread-local storage.

### C caller responsibility

```c
// ✅ Safe: different documents on different threads
// Thread 1:
TxmlDocument doc1 = txml2_document_new();
txml2_document_parse(doc1, xml1);

// Thread 2:
TxmlDocument doc2 = txml2_document_new();
txml2_document_parse(doc2, xml2);

// ❌ Unsafe: same document on different threads without synchronization
// Thread 1: txml2_document_parse(doc, xml);
// Thread 2: txml2_element_name(txml2_document_root(doc));  // DATA RACE
```

---

## Example C Usage Patterns

### Basic parsing and traversal

```c
#include "tinyxml2.h"
#include <stdio.h>

int main(void) {
    // Create a new document
    TxmlDocument doc = txml2_document_new();
    if (!doc) {
        fprintf(stderr, "Failed to allocate document\n");
        return 1;
    }

    // Parse XML from a string
    const char* xml = "<root><item id=\"1\">Hello</item><item id=\"2\">World</item></root>";
    int32_t err = txml2_document_parse(doc, xml);
    if (err != TXML2_SUCCESS) {
        fprintf(stderr, "Parse error: %d\n", err);
        txml2_document_free(doc);
        return 1;
    }

    // Get root element
    TxmlNode root = txml2_document_root_element(doc);

    // Iterate children
    TxmlNode child = txml2_node_first_child_element(root, NULL);
    while (child) {
        const char* name = txml2_element_name(child);
        const char* id = txml2_element_attribute(child, "id", NULL);
        const char* text = txml2_element_get_text(child);

        printf("Element: %s, id=%s, text=%s\n", name, id ? id : "(none)", text ? text : "(none)");

        child = txml2_node_next_sibling_element(child, NULL);
    }

    // Free the document (and all nodes)
    txml2_document_free(doc);
    return 0;
}
```

### Creating XML programmatically

```c
#include "tinyxml2.h"
#include <stdio.h>

int main(void) {
    TxmlDocument doc = txml2_document_new();

    // Create declaration
    TxmlNode decl = txml2_document_new_declaration(doc, NULL);
    txml2_document_insert_first_child(doc, decl);

    // Create root element
    TxmlNode root = txml2_document_new_element(doc, "config");
    txml2_document_insert_end_child(doc, root);

    // Add child elements
    TxmlNode setting = txml2_document_new_element(doc, "setting");
    txml2_element_set_attribute(setting, "name", "volume");
    txml2_element_set_attribute_int(setting, "value", 75);
    txml2_node_insert_end_child(root, setting);

    // Print to string
    TxmlPrinter printer = txml2_printer_new(0);  // 0 = not compact
    txml2_document_accept(doc, printer);
    const char* xml_output = txml2_printer_cstr(printer);
    printf("%s\n", xml_output);

    // Cleanup
    txml2_printer_free(printer);
    txml2_document_free(doc);
    return 0;
}
```

### Error handling

```c
#include "tinyxml2.h"
#include <stdio.h>

const char* error_string(int32_t code) {
    switch (code) {
        case TXML2_SUCCESS: return "Success";
        case TXML2_ERROR_PARSING_ELEMENT: return "Error parsing element";
        case TXML2_ERROR_MISMATCHED_ELEMENT: return "Mismatched element";
        case TXML2_ERROR_EMPTY_DOCUMENT: return "Empty document";
        case TXML2_ELEMENT_DEPTH_EXCEEDED: return "Element depth exceeded";
        default: return "Unknown error";
    }
}

int main(void) {
    TxmlDocument doc = txml2_document_new();

    // Intentionally malformed XML
    const char* bad_xml = "<root><unclosed>";
    int32_t err = txml2_document_parse(doc, bad_xml);
    if (err != TXML2_SUCCESS) {
        fprintf(stderr, "Parse failed: %s (code %d)\n", error_string(err), err);
        // Document should not be used after parse failure
    }

    txml2_document_free(doc);
    return (err != TXML2_SUCCESS) ? 1 : 0;
}
```
