# Phase 6: C API Layer

> **Status:** ✅ COMPLETED  
> **Completed:** 2026-06-30  
> **Estimated Complexity:** MEDIUM (~1500 LOC)  
> **Dependencies:** Phase 5 (complete Rust API)  
> **Milestone:** `v0.0.6-alpha` internal

---

## Objectives

Provide a C-compatible FFI layer that allows existing C and C++ projects to
use tinyxml2-rs as a drop-in replacement for TinyXML2 without rewriting
application code. The C API exposes opaque handle types, `extern "C"` functions
for all core operations, C-compatible error codes, and a generated C header
file.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                      C Application                           │
│  #include "tinyxml2.h"                                       │
│  TxDocument* doc = tx_document_new();                        │
│  tx_document_parse(doc, "<root>hello</root>");               │
│  TxNodeId root = tx_document_root_element(doc);              │
│  ...                                                         │
└────────────────────────┬────────────────────────────────────┘
                         │ extern "C" calls
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                  tinyxml2-ffi crate                           │
│  ┌───────────────────────────────────────────────────────┐   │
│  │  Opaque types:  TxDocument (Box<Document>)            │   │
│  │                 TxNodeId   (repr(C) struct)           │   │
│  │                 TxPrinter  (Box<Printer>)             │   │
│  │                                                       │   │
│  │  Panic safety:  catch_unwind on all extern "C" fns    │   │
│  │  Error codes:   C-compatible i32 enum                 │   │
│  └───────────────────────────────────────────────────────┘   │
│                         │                                    │
│                         ▼                                    │
│  ┌───────────────────────────────────────────────────────┐   │
│  │              tinyxml2 core crate (Rust API)           │   │
│  └───────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

---

## Deliverables

### 1. Opaque Handle Types

| C Type | Rust Backing | Description |
|--------|-------------|-------------|
| `TxDocument*` | `*mut Document` (boxed) | Opaque pointer to a document |
| `TxNodeId` | `#[repr(C)] struct { index: u32, generation: u32 }` | Value type for node handles |
| `TxPrinter*` | `*mut Printer` (boxed) | Opaque pointer to a printer |
| `TX_NULL_NODE` | `TxNodeId { index: u32::MAX, generation: 0 }` | Sentinel for "no node" |

### 2. Document Lifecycle Functions

```c
// Creation and destruction
TxDocument*  tx_document_new(void);
void         tx_document_free(TxDocument* doc);
void         tx_document_clear(TxDocument* doc);

// Parsing
TxError      tx_document_parse(TxDocument* doc, const char* xml);
TxError      tx_document_load_file(TxDocument* doc, const char* path);

// Serialization
TxError      tx_document_save_file(const TxDocument* doc, const char* path);
const char*  tx_document_to_string(const TxDocument* doc);
const char*  tx_document_to_string_compact(const TxDocument* doc);

// Error state
TxError      tx_document_error(const TxDocument* doc);
int          tx_document_error_line(const TxDocument* doc);
const char*  tx_document_error_name(const TxDocument* doc);
```

### 3. Node Factory Functions

```c
TxNodeId  tx_new_element(TxDocument* doc, const char* name);
TxNodeId  tx_new_text(TxDocument* doc, const char* text);
TxNodeId  tx_new_comment(TxDocument* doc, const char* text);
TxNodeId  tx_new_declaration(TxDocument* doc, const char* text);
TxNodeId  tx_new_unknown(TxDocument* doc, const char* text);
```

### 4. Tree Modification Functions

```c
void      tx_insert_end_child(TxDocument* doc, TxNodeId parent, TxNodeId child);
void      tx_insert_first_child(TxDocument* doc, TxNodeId parent, TxNodeId child);
void      tx_insert_after_child(TxDocument* doc, TxNodeId after, TxNodeId child);
void      tx_delete_child(TxDocument* doc, TxNodeId parent, TxNodeId child);
void      tx_delete_children(TxDocument* doc, TxNodeId parent);
void      tx_delete_node(TxDocument* doc, TxNodeId node);
```

### 5. Navigation Functions

```c
TxNodeId  tx_parent(const TxDocument* doc, TxNodeId node);
TxNodeId  tx_first_child(const TxDocument* doc, TxNodeId node);
TxNodeId  tx_last_child(const TxDocument* doc, TxNodeId node);
TxNodeId  tx_prev_sibling(const TxDocument* doc, TxNodeId node);
TxNodeId  tx_next_sibling(const TxDocument* doc, TxNodeId node);
TxNodeId  tx_first_child_element(const TxDocument* doc, TxNodeId node, const char* name);
TxNodeId  tx_next_sibling_element(const TxDocument* doc, TxNodeId node, const char* name);
TxNodeId  tx_root_element(const TxDocument* doc);
```

### 6. Element Functions

```c
const char*  tx_element_name(const TxDocument* doc, TxNodeId element);
const char*  tx_element_attribute(const TxDocument* doc, TxNodeId el, const char* name);
void         tx_element_set_attribute(TxDocument* doc, TxNodeId el,
                                       const char* name, const char* value);
void         tx_element_delete_attribute(TxDocument* doc, TxNodeId el, const char* name);
const char*  tx_element_get_text(const TxDocument* doc, TxNodeId element);
void         tx_element_set_text(TxDocument* doc, TxNodeId element, const char* text);
```

### 7. Typed Attribute Access

```c
TxError  tx_query_int_attribute(const TxDocument* doc, TxNodeId el,
                                 const char* name, int* value);
TxError  tx_query_bool_attribute(const TxDocument* doc, TxNodeId el,
                                  const char* name, bool* value);
TxError  tx_query_double_attribute(const TxDocument* doc, TxNodeId el,
                                    const char* name, double* value);
int      tx_int_attribute(const TxDocument* doc, TxNodeId el,
                           const char* name, int default_val);
bool     tx_bool_attribute(const TxDocument* doc, TxNodeId el,
                            const char* name, bool default_val);
double   tx_double_attribute(const TxDocument* doc, TxNodeId el,
                              const char* name, double default_val);
```

### 8. Printer / Streaming API

```c
TxPrinter*   tx_printer_new(void);
TxPrinter*   tx_printer_new_compact(void);
void         tx_printer_free(TxPrinter* printer);
void         tx_printer_open_element(TxPrinter* p, const char* name);
void         tx_printer_push_attribute(TxPrinter* p, const char* name, const char* value);
void         tx_printer_close_element(TxPrinter* p);
void         tx_printer_push_text(TxPrinter* p, const char* text);
void         tx_printer_push_comment(TxPrinter* p, const char* text);
const char*  tx_printer_result(const TxPrinter* p);
void         tx_printer_clear(TxPrinter* p);
```

### 9. Node Type Inspection

```c
typedef enum {
    TX_NODE_DOCUMENT    = 0,
    TX_NODE_ELEMENT     = 1,
    TX_NODE_TEXT        = 2,
    TX_NODE_COMMENT     = 3,
    TX_NODE_DECLARATION = 4,
    TX_NODE_UNKNOWN     = 5,
} TxNodeType;

TxNodeType  tx_node_type(const TxDocument* doc, TxNodeId node);
bool        tx_node_is_null(TxNodeId node);
const char* tx_node_value(const TxDocument* doc, TxNodeId node);
int         tx_node_line(const TxDocument* doc, TxNodeId node);
```

### 10. C-Compatible Error Codes

```c
typedef enum {
    TX_SUCCESS                   = 0,
    TX_ERROR_EMPTY_DOCUMENT      = 1,
    TX_ERROR_PARSING_ELEMENT     = 2,
    TX_ERROR_PARSING_ATTRIBUTE   = 3,
    TX_ERROR_PARSING_TEXT        = 4,
    TX_ERROR_PARSING_CDATA       = 5,
    TX_ERROR_PARSING_COMMENT     = 6,
    TX_ERROR_PARSING_DECLARATION = 7,
    TX_ERROR_PARSING_UNKNOWN     = 8,
    TX_ERROR_MISMATCHED_ELEMENT  = 9,
    TX_ERROR_FILE_NOT_FOUND      = 10,
    TX_ERROR_FILE_READ           = 11,
    TX_ERROR_DEPTH_EXCEEDED      = 12,
    // ... remaining TinyXML2-compatible codes
    TX_ERROR_COUNT               = 21,
} TxError;
```

---

## Safety Strategy

### Panic Safety

Every `extern "C"` function is wrapped in `std::panic::catch_unwind`:

```rust
#[no_mangle]
pub extern "C" fn tx_document_parse(doc: *mut Document, xml: *const c_char) -> TxError {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let doc = unsafe { &mut *doc };
        let xml = unsafe { CStr::from_ptr(xml) }.to_str().unwrap();
        doc.parse(xml)
    }));
    match result {
        Ok(Ok(())) => TxError::TX_SUCCESS,
        Ok(Err(e)) => e.into(),
        Err(_) => TxError::TX_ERROR_PARSING_UNKNOWN, // panic fallback
    }
}
```

### Null Pointer Checks

All functions accepting pointers validate non-null before dereferencing.
Null `TxDocument*` returns a sentinel error code.

### Lifetime Management

- `TxDocument*` is heap-allocated via `Box::new()` and freed via `Box::from_raw()`
- `const char*` return values point to Rust-owned strings; valid until the next
  mutating operation on the same document
- Callers must not free `const char*` return values

---

## Build Configuration

### Cargo.toml (tinyxml2-ffi)

```toml
[lib]
crate-type = ["staticlib", "cdylib"]

[dependencies]
tinyxml2 = { path = "../tinyxml2" }

[build-dependencies]
cbindgen = "0.27"
```

### Header Generation

`cbindgen` generates `tinyxml2.h` from the Rust source:

```rust
// build.rs
fn main() {
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_language(cbindgen::Language::C)
        .with_include_guard("TINYXML2_RS_H")
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file("include/tinyxml2.h");
}
```

---

## Estimated Test Plan

| Category | Est. Tests | Description |
|----------|-----------|-------------|
| Document lifecycle | 6 | new/free/clear, double-free safety |
| Parse via C API | 10 | Valid/invalid XML, error code mapping |
| Node factory | 6 | All node types via C functions |
| Tree modification | 12 | Insert/delete via C functions |
| Navigation | 10 | All navigation functions |
| Attributes | 10 | Get/set/delete/typed via C functions |
| Printer | 8 | Streaming API via C functions |
| Null safety | 8 | Null pointers, null NodeId, invalid NodeId |
| Panic safety | 4 | Deliberate panic triggers |
| Header compilation | 2 | Compile C example with generated header |

**Estimated Total:** ~76 tests

---

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| String lifetime management | Memory safety | Document all ownership rules; use `const char*` consistently |
| Panic across FFI boundary | UB | `catch_unwind` on every extern function |
| cbindgen compatibility | Build | Pin cbindgen version; test header compilation |
| NodeId repr(C) correctness | Interop | Static assertion on size/alignment |
| Thread safety | Concurrency | Document single-threaded usage requirement |

---

## Acceptance Criteria

- [ ] All `extern "C"` functions are callable from C
- [ ] Generated `tinyxml2.h` compiles with `gcc` and `clang`
- [ ] C example programs build and run correctly
- [ ] Error codes map 1:1 to TinyXML2 error codes
- [ ] Null pointer inputs produce defined behavior (not UB)
- [ ] Panics do not propagate across FFI boundary
- [ ] `staticlib` and `cdylib` both build successfully
- [ ] NodeId round-trips correctly through C API
- [ ] All tests pass with zero warnings

---

## File Plan

| File | Responsibility |
|------|---------------|
| `tinyxml2-ffi/src/lib.rs` | All `extern "C"` functions |
| `tinyxml2-ffi/src/types.rs` | `TxNodeId`, `TxError`, `TxNodeType` C-compatible types |
| `tinyxml2-ffi/build.rs` | cbindgen header generation |
| `tinyxml2-ffi/include/tinyxml2.h` | Generated C header |
| `tinyxml2-ffi/Cargo.toml` | Crate config with staticlib + cdylib |
| `tinyxml2-ffi/examples/basic.c` | C example: parse and traverse |
| `tinyxml2-ffi/examples/create.c` | C example: build DOM and serialize |
| `tinyxml2-ffi/tests/ffi_tests.rs` | Rust-side FFI tests |

---

## Previous Phase

← [Phase 5: Visitor Pattern & Ergonomic API](./phase-05.md)

## Next Phase

→ [Phase 7: Testing, Fuzzing & Benchmarks](./phase-07.md)
