# Behavior Compatibility

This document describes how `tinyxml2-rs` preserves behavioral compatibility with the
original TinyXML2 C++ library. The goal is **semantic equivalence**: given the same
input, both libraries should produce the same parse tree and the same serialized output,
with deviations only where Rust's type system and idioms provide a strictly better
alternative.

---

## What "Behavioral Compatibility" Means

Behavioral compatibility is stronger than API compatibility. It means:

1. **Same documents accepted** — any XML that TinyXML2 parses successfully, `tinyxml2-rs`
   also parses successfully, producing an equivalent DOM tree.
2. **Same documents rejected** — any XML that TinyXML2 rejects with an error,
   `tinyxml2-rs` also rejects with the corresponding `XmlError` variant.
3. **Same output** — serializing equivalent DOM trees produces byte-identical output
   in both compact and pretty-print modes.
4. **Same edge-case behavior** — entity expansion, whitespace handling, BOM processing,
   and error conditions match the C++ reference implementation.

---

## Parsing

### Accepted Documents

`tinyxml2-rs` accepts the same set of XML documents as TinyXML2. This includes:

- Well-formed XML 1.0 documents
- Documents with XML declarations (`<?xml version="1.0"?>`)
- Documents with comments, CDATA sections, and processing instructions
- Documents with the 5 predefined XML entities
- Documents with numeric character references (decimal and hexadecimal)
- Documents with mixed content (elements containing both text and child elements)
- Documents with UTF-8 BOM (detected and stripped)
- Documents with empty elements (`<br/>` and `<br></br>`)

### Rejected Documents

The same malformed inputs produce errors:

| Condition | TinyXML2 Error | XmlError Variant |
|---|---|---|
| Empty document | `XML_ERROR_EMPTY_DOCUMENT` | `XmlError::EmptyDocument` |
| Mismatched tags | `XML_ERROR_MISMATCHED_ELEMENT` | `XmlError::MismatchedElement` |
| Missing closing tag | `XML_ERROR_PARSING` | `XmlError::Parsing` |
| Duplicate attributes | `XML_ERROR_PARSING_ATTRIBUTE` | `XmlError::ParsingAttribute` |
| Malformed declaration | `XML_ERROR_PARSING_DECLARATION` | `XmlError::ParsingDeclaration` |
| Malformed comment | `XML_ERROR_PARSING_COMMENT` | `XmlError::ParsingComment` |
| Malformed CDATA | `XML_ERROR_PARSING_CDATA` | `XmlError::ParsingCData` |
| Malformed element | `XML_ERROR_PARSING_ELEMENT` | `XmlError::ParsingElement` |
| Malformed text | `XML_ERROR_PARSING_TEXT` | `XmlError::ParsingText` |
| Malformed unknown | `XML_ERROR_PARSING_UNKNOWN` | `XmlError::ParsingUnknown` |
| Max depth exceeded | `XML_ERROR_ELEMENT_DEPTH_EXCEEDED` | `XmlError::ElementDepthExceeded` |

### Parse Results

Given the same input and the same configuration (`ProcessEntities`, `WhitespaceMode`),
both libraries produce DOM trees with:

- The same node types in the same order
- The same element names and attribute key-value pairs
- The same text content (after entity expansion and whitespace processing)
- The same CDATA content (verbatim, never entity-processed)
- The same line numbers on all nodes

---

## Serialization

### Compact Mode

Both libraries produce identical compact output:

- No whitespace between elements
- No newlines
- Single space between attributes
- Self-closing tags for empty elements (`<br/>`)
- Entities re-escaped on output (`<` → `&lt;`, etc.)

### Pretty-Print Mode

Both libraries produce identical pretty-printed output:

- 4-space indentation (configurable, same default)
- Newline after each element
- Text content inline with its parent element when it's the only child
- Mixed content preserves text positioning
- Declaration and comments at their natural indentation level

### Output Encoding

- UTF-8 output in both libraries
- BOM emitted if `SetBOM(true)` / `set_bom(true)` is set
- Attribute values quoted with double quotes
- Special characters in attribute values are escaped

---

## Entity Handling

### Predefined Entities

Both libraries recognize and process the same 5 predefined XML entities:

| Entity | Character | Decimal | Hex |
|---|---|---|---|
| `&amp;` | `&` | `&#38;` | `&#x26;` |
| `&lt;` | `<` | `&#60;` | `&#x3c;` |
| `&gt;` | `>` | `&#62;` | `&#x3e;` |
| `&apos;` | `'` | `&#39;` | `&#x27;` |
| `&quot;` | `"` | `&#34;` | `&#x22;` |

### Numeric Character References

- Decimal: `&#NNN;` — both libraries parse the same range of values
- Hexadecimal: `&#xHHH;` — case-insensitive hex digits, same behavior
- Out-of-range values: both libraries produce the same error or replacement behavior
- Leading zeros: accepted in both libraries

### Entity Processing Control

- When `ProcessEntities` is `true` (default): entities are expanded on parse, re-escaped
  on serialize
- When `ProcessEntities` is `false`: entity-like sequences are treated as literal text
- Both libraries have identical behavior in both modes

### Invalid Entity Handling

Unrecognized named entities (e.g., `&foo;`) are handled identically:

- Both libraries pass them through without modification when `ProcessEntities` is `true`
- Both libraries treat them as literal text when `ProcessEntities` is `false`

---

## Whitespace Handling

Both libraries support the same three whitespace modes with identical semantics:

### Preserve Mode (Default)

- All whitespace in text content is preserved exactly as written
- Leading and trailing whitespace in text nodes is kept
- Newlines within text are preserved
- This is the default mode in both libraries

### Collapse Mode

- Contiguous runs of whitespace (spaces, tabs, newlines, carriage returns) are
  collapsed to a single space
- Leading and trailing whitespace in text nodes is trimmed
- Whitespace-only text nodes may be eliminated

### Pedantic Mode

- Whitespace is preserved exactly as written (like Preserve mode)
- Additionally, whitespace between markup is also preserved as text nodes
- Even whitespace-only text nodes between elements are kept
- Most strictly preserves the original document's whitespace

### Edge Cases

The following edge cases behave identically in both libraries:

- `\r\n` → `\n` normalization before whitespace mode processing
- `\r` (standalone) → `\n` normalization
- Whitespace inside CDATA sections is always preserved regardless of mode
- Whitespace in attribute values follows XML 1.0 normalization rules

---

## BOM Handling

### Detection on Parse

- Both libraries detect a UTF-8 BOM (`0xEF 0xBB 0xBF`) at the start of input
- The BOM is stripped and does not appear in the parse tree
- `HasBOM()` / `has_bom()` returns `true` if a BOM was detected

### Emission on Serialize

- `SetBOM(true)` / `set_bom(true)` causes the BOM to be emitted at the start of output
- `SetBOM(false)` / `set_bom(false)` (default) omits the BOM
- BOM detection on parse does **not** automatically enable BOM emission on serialize
  (same behavior in both libraries)

---

## Error Codes

Every TinyXML2 `XMLError` code maps 1:1 to an `XmlError` variant:

| TinyXML2 Constant | Value | XmlError Variant |
|---|---|---|
| `XML_SUCCESS` | 0 | (no error — `Ok(())`) |
| `XML_NO_ATTRIBUTE` | 1 | `XmlError::NoAttribute` |
| `XML_WRONG_ATTRIBUTE_TYPE` | 2 | `XmlError::WrongAttributeType` |
| `XML_ERROR_FILE_NOT_FOUND` | 3 | `XmlError::FileNotFound` |
| `XML_ERROR_FILE_COULD_NOT_BE_OPENED` | 4 | `XmlError::FileCouldNotBeOpened` |
| `XML_ERROR_FILE_READ_ERROR` | 5 | `XmlError::FileReadError` |
| `XML_ERROR_PARSING_ELEMENT` | 6 | `XmlError::ParsingElement` |
| `XML_ERROR_PARSING_ATTRIBUTE` | 7 | `XmlError::ParsingAttribute` |
| `XML_ERROR_PARSING_TEXT` | 8 | `XmlError::ParsingText` |
| `XML_ERROR_PARSING_CDATA` | 9 | `XmlError::ParsingCData` |
| `XML_ERROR_PARSING_COMMENT` | 10 | `XmlError::ParsingComment` |
| `XML_ERROR_PARSING_DECLARATION` | 11 | `XmlError::ParsingDeclaration` |
| `XML_ERROR_PARSING_UNKNOWN` | 12 | `XmlError::ParsingUnknown` |
| `XML_ERROR_EMPTY_DOCUMENT` | 13 | `XmlError::EmptyDocument` |
| `XML_ERROR_MISMATCHED_ELEMENT` | 14 | `XmlError::MismatchedElement` |
| `XML_ERROR_PARSING` | 15 | `XmlError::Parsing` |
| `XML_CAN_NOT_CONVERT_TEXT` | 16 | `XmlError::CanNotConvertText` |
| `XML_NO_TEXT_NODE` | 17 | `XmlError::NoTextNode` |
| `XML_ELEMENT_DEPTH_EXCEEDED` | 18 | `XmlError::ElementDepthExceeded` |

### Additional Rust-Only Errors

| XmlError Variant | Description |
|---|---|
| `XmlError::InvalidNodeId` | The `NodeId` does not refer to a valid, live node in the arena. This error has no C++ equivalent because C++ uses raw pointers (which would segfault on invalid access). |
| `XmlError::Io(std::io::Error)` | Wraps Rust `std::io::Error` for file/stream operations. Replaces the C++ `FILE*`-based error reporting. |

---

## Memory Ownership

### Document-Owns-All Model

Both libraries use the same ownership model:

- **TinyXML2 (C++):** `XMLDocument` allocates all nodes from an internal memory pool.
  Nodes are freed when the document is destroyed or `Clear()` is called.
- **tinyxml2-rs (Rust):** `XmlDocument` allocates all nodes in a generational arena.
  Nodes are freed when the document is dropped or `clear()` is called.

### Key Equivalences

| Concept | TinyXML2 (C++) | tinyxml2-rs (Rust) |
|---|---|---|
| Node allocation | `doc.NewElement("x")` returns `XMLElement*` | `doc.new_element("x")` returns `NodeId` |
| Node ownership | Document owns all nodes | Document owns all nodes |
| Node lifetime | Valid until `DeleteChild()` or `Clear()` | Valid until `delete_child()` or `clear()` |
| Cross-document | Nodes cannot move between documents | Nodes cannot move between documents |
| Cleanup | `Clear()` frees all nodes | `clear()` frees all nodes |
| Destruction | `~XMLDocument()` frees all nodes | `Drop` frees all nodes |

### Safety Improvement

In C++, using a pointer to a deleted node is undefined behavior. In Rust, using a
`NodeId` that refers to a deleted node returns `Err(XmlError::InvalidNodeId)` — a
safe, recoverable error.

---

## Thread Safety

Both libraries provide the same threading guarantees:

### Shared Document (Neither Library is Safe)

- Neither TinyXML2 nor `tinyxml2-rs` provides internal synchronization
- Concurrent reads and writes to the same document from multiple threads are
  **unsafe** in C++ and **prevented by the borrow checker** in Rust
- In Rust, `XmlDocument` is `Send` but not `Sync` — it can be moved between threads
  but not shared without external synchronization

### Separate Documents (Always Safe)

- Each thread can safely own and operate on its own `XmlDocument`
- No global state is shared between documents in either library
- This is the recommended usage pattern for multi-threaded applications

### Rust Advantage

The Rust borrow checker enforces at compile time what is merely documented convention
in the C++ library. Data races on a shared `XmlDocument` are compile-time errors in
Rust rather than runtime undefined behavior.

---

## Intentional Deviations

The following deviations from TinyXML2's behavior are **intentional** and represent
improvements enabled by Rust's type system and idioms.

### 1. Result-Based Errors Instead of Error-State Polling

**TinyXML2 (C++):**
```cpp
doc.Parse(xml);
if (doc.Error()) {
    printf("Error: %s\n", doc.ErrorStr());
}
```

**tinyxml2-rs (Rust):**
```rust
doc.parse(xml)?;  // Error propagated via Result
```

**Rationale:** Rust's `Result` type makes error handling explicit and composable.
It is impossible to forget to check for errors when the return type is `Result`.
The error-state polling pattern (`Error()`, `ErrorID()`, `ClearError()`) is not
needed and is not provided.

### 2. NodeId Instead of Raw Pointers

**TinyXML2 (C++):**
```cpp
XMLElement* elem = doc.NewElement("child");
parent->InsertEndChild(elem);
// elem is a raw pointer — dangling if parent or doc is destroyed
```

**tinyxml2-rs (Rust):**
```rust
let elem = doc.new_element("child");
doc.insert_end_child(parent, elem)?;
// elem is a NodeId — validated on every use
```

**Rationale:** `NodeId` is a lightweight handle (index + generation) that is validated
on access. Using a stale `NodeId` returns `Err(XmlError::InvalidNodeId)` instead of
causing undefined behavior.

### 3. InvalidNodeId Error (Unique to Arena Model)

The `XmlError::InvalidNodeId` variant has no C++ equivalent. It is returned when
code attempts to use a `NodeId` that:

- Refers to a node that has been deleted
- Was created by a different `XmlDocument`
- Has an out-of-range index

This error replaces the undefined behavior that would occur in C++ when using a
dangling pointer.

### 4. Builder-Style ParseOptions

**TinyXML2 (C++):**
```cpp
XMLDocument doc(true, COLLAPSE_WHITESPACE);
```

**tinyxml2-rs (Rust):**
```rust
let mut doc = XmlDocument::new();
doc.parse_with_options(
    xml,
    ParseOptions::new()
        .process_entities(true)
        .whitespace_mode(WhitespaceMode::Collapse)
        .max_element_depth(500),
)?;
```

**Rationale:** Builder pattern is more readable, self-documenting, and extensible
than positional constructor parameters. New options can be added without breaking
existing call sites.

### 5. No FILE* Support

**TinyXML2 (C++):**
```cpp
FILE* fp = fopen("output.xml", "w");
doc.SaveFile(fp);
fclose(fp);
```

**tinyxml2-rs (Rust):**
```rust
let file = File::create("output.xml")?;
let mut writer = BufWriter::new(file);
doc.write_to(&mut writer)?;
```

**Rationale:** Rust's `std::io::Read` and `std::io::Write` traits are more general
and safer than C `FILE*` pointers. They support any I/O source (files, network
streams, in-memory buffers, etc.) without unsafe code.

### 6. Rust Naming Conventions

All public API surfaces follow Rust naming conventions:

| C++ Convention | Rust Convention | Example |
|---|---|---|
| `PascalCase` methods | `snake_case` methods | `FirstChildElement` → `first_child_element` |
| `PascalCase` types | `PascalCase` types | `XMLDocument` → `XmlDocument` |
| `SCREAMING_CASE` enums | `PascalCase` variants | `XML_SUCCESS` → `XmlError::Success` |
| `Get` prefix | No prefix | `GetText()` → `get_text()` (kept for clarity) |
| `Set` prefix | `set_` prefix | `SetName()` → `set_element_name()` |

---

## Verification

Behavioral compatibility is verified through:

1. **Port of TinyXML2's test suite** — All tests from the C++ `xmltest.cpp` are
   ported and expected to produce identical results.
2. **Roundtrip testing** — Parse → serialize → parse produces identical DOM trees.
3. **Fuzz testing** — Shared corpus between C++ and Rust fuzzers to ensure identical
   accept/reject decisions.
4. **Output comparison** — Byte-for-byte comparison of serialized output between
   the C++ and Rust implementations for a corpus of test documents.
