# Error Design

## Overview

tinyxml2-rs replaces TinyXML2's error-state polling model with Rust's idiomatic `Result<T, E>` pattern. In TinyXML2 (C++), errors are stored as mutable state on the `XMLDocument` object—callers must remember to check `doc.ErrorID()` after every operation, and forgetting to do so silently swallows failures. This is fundamentally at odds with Rust's philosophy of making errors impossible to ignore.

Our approach:

| Concern | TinyXML2 (C++) | tinyxml2-rs |
|---|---|---|
| Error representation | `XMLError` integer enum on `XMLDocument` | `XmlError` enum returned in `Result` |
| Error propagation | Caller polls `doc.ErrorID()` | `?` operator, pattern matching |
| Error composition | Manual code checks | `From` impls, `Result` combinators |
| Thread safety | Not addressed | `Send + Sync` by construction |
| Error context | `GetErrorStr1()` / `GetErrorStr2()` | Structured fields (line, kind, message) |

Every fallible operation in tinyxml2-rs returns `Result<T>`, making the compiler enforce error handling at every call site.

---

## XmlError Enum

```rust
pub enum XmlError {
    NoAttribute,
    WrongAttributeType,
    Io(std::io::Error),
    Parse {
        kind: ParseErrorKind,
        line: u32,
        message: Option<String>,
    },
    EmptyDocument,
    MismatchedElement {
        line: u32,
        expected: String,
        found: String,
    },
    CanNotConvertText,
    NoTextNode,
    ElementDepthExceeded {
        line: u32,
        max_depth: u32,
    },
    InvalidNodeId,
}
```

### Variant Documentation

| Variant | Fields | Description |
|---|---|---|
| `NoAttribute` | — | Requested attribute does not exist on the element. |
| `WrongAttributeType` | — | Attribute exists but its value cannot be converted to the requested type (e.g., `"abc"` as `i32`). |
| `Io(std::io::Error)` | Wrapped `std::io::Error` | I/O failure during file read/write. Covers `FILE_NOT_FOUND`, `FILE_COULD_NOT_BE_OPENED`, and `FILE_READ_ERROR` from TinyXML2. |
| `Parse { kind, line, message }` | `kind: ParseErrorKind`, `line: u32`, `message: Option<String>` | Syntax error encountered during parsing. The `kind` sub-discriminant identifies what was being parsed, `line` indicates where, and `message` provides optional human-readable detail. |
| `EmptyDocument` | — | The input string or file contained no parseable XML content. |
| `MismatchedElement { line, expected, found }` | `line: u32`, `expected: String`, `found: String` | A closing tag `</found>` did not match the opening tag `<expected>`. |
| `CanNotConvertText` | — | Element text content cannot be converted to the requested type. |
| `NoTextNode` | — | Element has no text child node, but text was requested. |
| `ElementDepthExceeded { line, max_depth }` | `line: u32`, `max_depth: u32` | Nesting depth exceeded `ParseOptions::max_depth`. Protects against stack overflow from maliciously deep documents. |
| `InvalidNodeId` | — | A `NodeId` was used that does not reference a valid, live node in the arena. Unique to tinyxml2-rs—no TinyXML2 equivalent exists because C++ uses raw pointers. |

---

## TinyXML2 Error Code Mapping

Every `XMLError` constant defined in TinyXML2 maps to an `XmlError` variant:

| Code | TinyXML2 Constant | Value | XmlError Variant |
|---|---|---|---|
| 0 | `XML_SUCCESS` | 0 | `Ok(())` — not an error |
| 1 | `XML_NO_ATTRIBUTE` | 1 | `NoAttribute` |
| 2 | `XML_WRONG_ATTRIBUTE_TYPE` | 2 | `WrongAttributeType` |
| 3 | `XML_ERROR_FILE_NOT_FOUND` | 3 | `Io(std::io::Error)` with `ErrorKind::NotFound` |
| 4 | `XML_ERROR_FILE_COULD_NOT_BE_OPENED` | 4 | `Io(std::io::Error)` with `ErrorKind::PermissionDenied` (or other) |
| 5 | `XML_ERROR_FILE_READ_ERROR` | 5 | `Io(std::io::Error)` with relevant `ErrorKind` |
| 6 | `XML_ERROR_PARSING_ELEMENT` | 6 | `Parse { kind: ParseErrorKind::Element, .. }` |
| 7 | `XML_ERROR_PARSING_ATTRIBUTE` | 7 | `Parse { kind: ParseErrorKind::Attribute, .. }` |
| 8 | `XML_ERROR_PARSING_TEXT` | 8 | `Parse { kind: ParseErrorKind::Text, .. }` |
| 9 | `XML_ERROR_PARSING_CDATA` | 9 | `Parse { kind: ParseErrorKind::Cdata, .. }` |
| 10 | `XML_ERROR_PARSING_COMMENT` | 10 | `Parse { kind: ParseErrorKind::Comment, .. }` |
| 11 | `XML_ERROR_PARSING_DECLARATION` | 11 | `Parse { kind: ParseErrorKind::Declaration, .. }` |
| 12 | `XML_ERROR_PARSING_UNKNOWN` | 12 | `Parse { kind: ParseErrorKind::Unknown, .. }` |
| 13 | `XML_ERROR_EMPTY_DOCUMENT` | 13 | `EmptyDocument` |
| 14 | `XML_ERROR_MISMATCHED_ELEMENT` | 14 | `MismatchedElement { line, expected, found }` |
| 15 | `XML_ERROR_PARSING` | 15 | `Parse { kind: ParseErrorKind::General, .. }` |
| 16 | `XML_CAN_NOT_CONVERT_TEXT` | 16 | `CanNotConvertText` |
| 17 | `XML_NO_TEXT_NODE` | 17 | `NoTextNode` |
| 18 | `XML_ELEMENT_DEPTH_EXCEEDED` | 18 | `ElementDepthExceeded { line, max_depth }` |
| — | *(no C++ equivalent)* | — | `InvalidNodeId` |

> [!NOTE]
> TinyXML2 uses three separate error codes for file I/O failures (3, 4, 5). We collapse these into a single `Io` variant because `std::io::Error` already carries the precise `ErrorKind` discriminant, making three separate enum arms redundant.

---

## ParseErrorKind Enum

```rust
pub enum ParseErrorKind {
    Element,
    Attribute,
    Text,
    Cdata,
    Comment,
    Declaration,
    Unknown,
    General,
}
```

| Variant | TinyXML2 Error Name | Description |
|---|---|---|
| `Element` | `XML_ERROR_PARSING_ELEMENT` | Malformed element tag (missing name, invalid characters). |
| `Attribute` | `XML_ERROR_PARSING_ATTRIBUTE` | Malformed attribute (missing `=`, mismatched quotes). |
| `Text` | `XML_ERROR_PARSING_TEXT` | Invalid text content. |
| `Cdata` | `XML_ERROR_PARSING_CDATA` | Malformed CDATA section (missing `]]>` terminator). |
| `Comment` | `XML_ERROR_PARSING_COMMENT` | Malformed comment (missing `-->` terminator). |
| `Declaration` | `XML_ERROR_PARSING_DECLARATION` | Malformed XML declaration (missing `?>` terminator). |
| `Unknown` | `XML_ERROR_PARSING_UNKNOWN` | Malformed unknown node (`<!...>` that isn't comment/CDATA). |
| `General` | `XML_ERROR_PARSING` | Catch-all for parse errors that don't fit a specific category. |

---

## Line Number Tracking

Three error variants carry line number information:

| Variant | Line Field | Semantics |
|---|---|---|
| `Parse { line, .. }` | `line: u32` | Line where the parser detected the syntax error. |
| `MismatchedElement { line, .. }` | `line: u32` | Line of the closing tag that did not match. |
| `ElementDepthExceeded { line, .. }` | `line: u32` | Line of the opening tag that exceeded `max_depth`. |

All other variants (`NoAttribute`, `WrongAttributeType`, `CanNotConvertText`, `NoTextNode`, `EmptyDocument`, `InvalidNodeId`, `Io`) do not carry line information because they occur during DOM manipulation or I/O—not during positional parsing.

The `XmlError::line()` method provides uniform access:

```rust
impl XmlError {
    pub fn line(&self) -> Option<u32> {
        match self {
            Self::Parse { line, .. }
            | Self::MismatchedElement { line, .. }
            | Self::ElementDepthExceeded { line, .. } => Some(*line),
            _ => None,
        }
    }
}
```

---

## Display Implementation

Each variant produces a human-readable error message:

| Variant | Example `Display` Output |
|---|---|
| `NoAttribute` | `"attribute not found"` |
| `WrongAttributeType` | `"wrong attribute type"` |
| `Io(e)` | `"I/O error: No such file or directory (os error 2)"` |
| `Parse { kind: Element, line: 5, message: None }` | `"parse error (element) at line 5"` |
| `Parse { kind: Attribute, line: 12, message: Some("missing '='") }` | `"parse error (attribute) at line 12: missing '='"` |
| `EmptyDocument` | `"empty document"` |
| `MismatchedElement { line: 8, expected: "div", found: "span" }` | `"mismatched element at line 8: expected closing tag 'div', found 'span'"` |
| `CanNotConvertText` | `"cannot convert text"` |
| `NoTextNode` | `"no text node"` |
| `ElementDepthExceeded { line: 100, max_depth: 500 }` | `"element depth exceeded at line 100 (max depth: 500)"` |
| `InvalidNodeId` | `"invalid node id"` |

The `name()` method returns a static string suitable for logging or error categorization:

```rust
impl XmlError {
    pub fn name(&self) -> &'static str {
        match self {
            Self::NoAttribute => "NoAttribute",
            Self::WrongAttributeType => "WrongAttributeType",
            Self::Io(_) => "Io",
            Self::Parse { .. } => "Parse",
            Self::EmptyDocument => "EmptyDocument",
            Self::MismatchedElement { .. } => "MismatchedElement",
            Self::CanNotConvertText => "CanNotConvertText",
            Self::NoTextNode => "NoTextNode",
            Self::ElementDepthExceeded { .. } => "ElementDepthExceeded",
            Self::InvalidNodeId => "InvalidNodeId",
        }
    }
}
```

---

## Trait Implementations

### `std::error::Error`

```rust
impl std::error::Error for XmlError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}
```

Only the `Io` variant wraps an underlying error source. All other variants are self-describing and return `None` from `source()`.

### `Send + Sync`

`XmlError` is `Send + Sync` by construction—all fields are `Send + Sync` (`String`, `u32`, `std::io::Error`). This is verified at compile time with static assertion tests:

```rust
#[cfg(test)]
fn _assert_send_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    assert_send::<XmlError>();
    assert_sync::<XmlError>();
}
```

These tests produce zero-cost compile-time verification. If a future variant breaks `Send` or `Sync`, the build fails immediately.

### `Clone`

`XmlError` implements `Clone` manually because `std::io::Error` does not implement `Clone`. The manual implementation reconstructs the `Io` variant by extracting the `ErrorKind` and converting the error message to a `String`:

```rust
impl Clone for XmlError {
    fn clone(&self) -> Self {
        match self {
            Self::Io(e) => {
                Self::Io(std::io::Error::new(e.kind(), e.to_string()))
            }
            // All other variants derive-clone naturally
            _ => { /* field-by-field clone */ }
        }
    }
}
```

> [!IMPORTANT]
> The `Io` clone is lossy—it preserves the `ErrorKind` and display message but discards any inner `source()` chain or OS-specific error payload. This is an acceptable trade-off because XML parsing errors rarely need to preserve the full I/O error chain across clones.

### `PartialEq`

Structural comparison with special handling for problematic variants:

- **`Io` variant**: Compared by `ErrorKind` only. Two `Io` errors with the same `ErrorKind` are considered equal, regardless of the message string. This is because `std::io::Error` does not implement `PartialEq`.
- **`Parse` variant**: The `message` field is **ignored** during comparison. Two `Parse` errors are equal if they have the same `kind` and `line`. This avoids brittle tests that depend on exact error message wording.
- **All other variants**: Standard structural equality.

```rust
impl PartialEq for XmlError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Io(a), Self::Io(b)) => a.kind() == b.kind(),
            (
                Self::Parse { kind: k1, line: l1, .. },
                Self::Parse { kind: k2, line: l2, .. },
            ) => k1 == k2 && l1 == l2,
            // ... structural equality for remaining variants
            _ => false,
        }
    }
}
```

### `Eq`

`Eq` is implemented unconditionally. Although `PartialEq` for `Io` is technically a partial comparison (different messages, same kind → equal), the equivalence classes are well-defined and reflexive, so `Eq` holds.

### `From<std::io::Error>`

```rust
impl From<std::io::Error> for XmlError {
    fn from(err: std::io::Error) -> Self {
        XmlError::Io(err)
    }
}
```

This enables seamless use of `?` in functions that perform I/O:

```rust
fn load_file(path: &str) -> Result<Document> {
    let content = std::fs::read_to_string(path)?; // io::Error → XmlError::Io
    Document::parse(&content)
}
```

---

## Result Type Alias

```rust
pub type Result<T> = std::result::Result<T, XmlError>;
```

This is the canonical result type for all fallible operations in tinyxml2-rs. It is re-exported from the crate root to avoid callers needing to import `XmlError` separately for basic usage.

---

## InvalidNodeId

`InvalidNodeId` is unique to tinyxml2-rs and has no TinyXML2 equivalent. It arises from the generational arena design:

- In TinyXML2, nodes are heap-allocated C++ objects accessed via raw pointers. A dangling pointer is undefined behavior.
- In tinyxml2-rs, nodes are stored in a generational arena and accessed via `NodeId` handles. A stale `NodeId` (one whose generation doesn't match the arena slot) is detected at runtime and returns `Err(XmlError::InvalidNodeId)`.

This makes use-after-free impossible at the type level—instead of UB, the caller gets a recoverable error.

---

## Design Decisions

### Why an Enum, Not a Struct

An error struct with a `kind` field and optional context fields would be simpler in some ways, but an enum was chosen because:

1. **Exhaustive matching**: Callers can `match` on every error variant and the compiler enforces handling of new variants when they are added.
2. **Zero-cost discrimination**: No runtime overhead for checking error kind—the discriminant is the kind.
3. **Variant-specific fields**: Different errors carry different context. `MismatchedElement` needs `expected` and `found`; `ElementDepthExceeded` needs `max_depth`. An enum makes this type-safe without optional fields.
4. **Alignment with TinyXML2**: TinyXML2's `XMLError` is already an enum. Our enum preserves a clear 1:1 mapping while adding Rust-idiomatic structure.

### Why Line Numbers in Variants, Not Separate

An alternative design would store line numbers in a wrapper struct:

```rust
// NOT our design
struct LocatedError {
    error: XmlError,
    line: Option<u32>,
}
```

We rejected this because:

1. **Not all errors have meaningful line numbers.** `NoAttribute` occurs during DOM queries, not parsing. A wrapper with `line: None` on every non-parse error is noisy.
2. **Variant-specific context is cleaner.** `MismatchedElement` needs `line` but also `expected` and `found`. Splitting location from the error would require two structs or flattening.
3. **Pattern matching is simpler.** Callers destructure one level: `XmlError::Parse { line, kind, .. }` instead of unwrapping a wrapper and then matching the inner error.
4. **Consistency with TinyXML2.** TinyXML2 stores the error line on the document, not on the error code. But since we don't have a mutable document error state, embedding the line in the relevant variants is the natural Rust equivalent.
