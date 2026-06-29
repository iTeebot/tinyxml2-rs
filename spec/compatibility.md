# Compatibility Specification

> **Purpose**: This document defines what "compatible with TinyXML2" means for the
> tinyxml2-rs project. It establishes compatibility tiers, clarifies what is and is not
> in scope, and outlines the test strategy for verifying compatibility.

---

## Table of Contents

1. [Definition of Compatibility](#1-definition-of-compatibility)
2. [What IS Compatible](#2-what-is-compatible)
3. [What is NOT Compatible](#3-what-is-not-compatible)
4. [Compatibility Tiers](#4-compatibility-tiers)
5. [Test Strategy](#5-test-strategy)
6. [Compatibility Exceptions](#6-compatibility-exceptions)

---

## 1. Definition of Compatibility

tinyxml2-rs aims for **behavioral compatibility** with TinyXML2: given the same XML
input and configuration, tinyxml2-rs produces the same observable results. This means
the same DOM structure, the same serialized output, and the same error conditions.

Behavioral compatibility is distinct from source compatibility (same API signatures),
binary compatibility (same ABI), or implementation compatibility (same algorithms).

---

## 2. What IS Compatible

### 2.1 Behavioral Compatibility

The most critical form of compatibility. Given the same XML input:

- **Same parse results**: The DOM tree has the same structure — same elements, same
  attributes, same text content, same node types, same parent-child relationships
- **Same values**: Element names, attribute names, attribute values, text content, and
  comment text are byte-identical after entity processing
- **Same error conditions**: If TinyXML2 reports an error for a given input, tinyxml2-rs
  reports the corresponding error. If TinyXML2 succeeds, tinyxml2-rs succeeds.

```
TinyXML2 input  ──→  TinyXML2 DOM structure  ══  tinyxml2-rs DOM structure
                                                  (identical)
```

### 2.2 Output Compatibility

Serialized output should be byte-identical:

- **Formatted mode**: Same indentation, same newline placement, same entity encoding
- **Compact mode**: Identical byte output
- **BOM handling**: Same BOM presence/absence in output
- **Entity encoding**: Same characters encoded, same entity references used

```
TinyXML2 Print()  ──→  output bytes  ══  tinyxml2-rs print()  ──→  output bytes
                                         (byte-identical)
```

### 2.3 Error Compatibility

Error conditions map between the two implementations:

| TinyXML2 Error Code | tinyxml2-rs Error |
|---------------------|-------------------|
| `XML_SUCCESS` | `Ok(())` |
| `XML_NO_ATTRIBUTE` | `XmlError::NoAttribute` |
| `XML_WRONG_ATTRIBUTE_TYPE` | `XmlError::WrongAttributeType` |
| `XML_ERROR_FILE_NOT_FOUND` | `XmlError::FileNotFound` |
| `XML_ERROR_FILE_COULD_NOT_BE_OPENED` | `XmlError::FileCouldNotBeOpened` |
| `XML_ERROR_FILE_READ_ERROR` | `XmlError::FileReadError` |
| `XML_ERROR_PARSING_ELEMENT` | `XmlError::ParsingElement` |
| `XML_ERROR_PARSING_ATTRIBUTE` | `XmlError::ParsingAttribute` |
| `XML_ERROR_PARSING_TEXT` | `XmlError::ParsingText` |
| `XML_ERROR_PARSING_CDATA` | `XmlError::ParsingCData` |
| `XML_ERROR_PARSING_COMMENT` | `XmlError::ParsingComment` |
| `XML_ERROR_PARSING_DECLARATION` | `XmlError::ParsingDeclaration` |
| `XML_ERROR_PARSING_UNKNOWN` | `XmlError::ParsingUnknown` |
| `XML_ERROR_EMPTY_DOCUMENT` | `XmlError::EmptyDocument` |
| `XML_ERROR_MISMATCHED_ELEMENT` | `XmlError::MismatchedElement` |
| `XML_ERROR_PARSING` | `XmlError::Parsing` |
| `XML_ERROR_COUNT` | `XmlError::CountMismatch` |
| `XML_NO_TEXT_NODE` | `XmlError::NoTextNode` |
| `XML_NO_EXISTING_ATTRIBUTE` | `XmlError::NoExistingAttribute` |
| `XML_ERROR_ELEMENT_DEPTH_EXCEEDED` | `XmlError::ElementDepthExceeded` |

### 2.4 Semantic Compatibility

The DOM structure is semantically equivalent:

- Same node types at the same positions
- Same parent-child and sibling relationships
- Same attribute sets on each element
- Same text content (after entity processing)
- Same CDATA vs. text distinction
- Same comment content
- Same declaration attributes

---

## 3. What is NOT Compatible

### 3.1 Source Compatibility

tinyxml2-rs is written in Rust, not C++. The API uses Rust idioms:

| TinyXML2 (C++) | tinyxml2-rs (Rust) |
|-----------------|-------------------|
| `XMLDocument doc;` | `let doc = Document::new();` |
| `doc.Parse(xml);` | `doc.parse(xml)?;` |
| `XMLElement* elem = doc.FirstChildElement();` | `let elem_id = doc.first_child_element(None);` |
| Returns `nullptr` for "not found" | Returns `Option<NodeId>` |
| Error codes via `ErrorID()` | `Result<T, XmlError>` |
| `const char*` string values | `&str` / `String` |

### 3.2 Binary Compatibility

- tinyxml2-rs is NOT ABI-compatible with the C++ TinyXML2 library
- Cannot be used as a drop-in replacement for the shared library
- Different memory layout, different calling conventions, different name mangling

### 3.3 Internal Algorithm Compatibility

- Different data structures (generational arena vs. memory pool with raw pointers)
- Different memory allocation patterns
- Different internal iteration strategies
- Implementation details are free to differ as long as observable behavior matches

### 3.4 Performance Characteristics

- Performance may differ in both directions:
  - **Potentially faster**: Cache-friendly arena, no virtual dispatch, Rust optimizations
  - **Potentially slower**: Generation checks, different allocation patterns
- Performance parity is a non-goal; correctness and safety come first
- Performance should be "in the same ballpark" — no order-of-magnitude regressions

---

## 4. Compatibility Tiers

### Tier 1: Must Match ⚠️

These behaviors MUST be identical. Any deviation is a bug.

| Behavior | Description |
|----------|-------------|
| Parsing behavior | Same DOM structure for same input |
| Entity handling | Same encoding/decoding for all entity types |
| Error conditions | Same errors for same inputs |
| Serialization output | Byte-identical formatted and compact output |
| Whitespace handling | Same behavior in all three modes |
| BOM handling | Same detection and output behavior |
| Self-closing tags | Same treatment of `<element/>` |
| Attribute parsing | Same attribute handling including quoting |
| CDATA sections | Same parsing and serialization |
| Comment handling | Same parsing and serialization |
| Element depth limits | Same default, same error condition |

### Tier 2: Should Match ⚡

These behaviors SHOULD be identical. Deviations require documentation and justification.

| Behavior | Description |
|----------|-------------|
| API surface coverage | All public TinyXML2 methods have Rust equivalents |
| Default values | Same default configurations |
| Edge case behavior | Same behavior for unusual but valid inputs |
| Attribute query methods | Same type conversion behavior (`IntAttribute`, etc.) |
| Node traversal order | Same iteration order for children and siblings |
| Deep clone behavior | Same subtree copying semantics |

### Tier 3: Best Effort 💡

These behaviors are matched on a best-effort basis. Deviations are acceptable.

| Behavior | Description |
|----------|-------------|
| Error message text | Messages should be helpful but need not be identical |
| Performance characteristics | Same order of magnitude but may differ |
| Memory usage | Similar but not identical (different data structures) |
| Stack depth under recursion | May differ due to different call frames |
| Thread safety model | Rust's ownership model provides different guarantees |

---

## 5. Test Strategy

### 5.1 Conformance Test Suite

Port TinyXML2's own test cases from `xmltest.cpp` to Rust:

- Each C++ test case becomes a Rust `#[test]` function
- Tests verify the same assertions as the original
- Coverage goal: 100% of TinyXML2's test cases ported

```rust
#[test]
fn test_parse_simple_element() {
    // Port of TinyXML2 test case
    let mut doc = Document::new();
    doc.parse("<element/>").unwrap();
    let root = doc.root_element().unwrap();
    assert_eq!(doc.element_name(root), "element");
}
```

### 5.2 Round-Trip Testing

Verify that parse → serialize → parse produces identical DOM structures:

```
Input XML → parse → DOM₁ → serialize → XML₂ → parse → DOM₂
                     │                                   │
                     └───────── assert_eq ───────────────┘
```

- Tests structural equivalence of DOM₁ and DOM₂
- Tests byte equality of serialized output
- Covers both formatted and compact modes

### 5.3 Fuzz Testing

Use `cargo-fuzz` with `libFuzzer` to discover parsing issues:

- **Structure-aware fuzzing**: Generate semi-valid XML inputs
- **Mutation-based fuzzing**: Mutate valid XML documents
- **Comparison fuzzing**: Feed same input to both C++ TinyXML2 and tinyxml2-rs,
  compare outputs
- **Crash detection**: Ensure no panics, no infinite loops, no excessive memory use

### 5.4 Edge Case Catalog

Maintain a catalog of known edge cases with expected behavior:

| # | Input | Expected Behavior | Tier |
|---|-------|-------------------|------|
| 1 | Empty string | `EmptyDocument` error | T1 |
| 2 | `<a><b></a>` | `MismatchedElement` error | T1 |
| 3 | `<a attr="v" attr="v2"/>` | `ParsingAttribute` error (duplicate) | T1 |
| 4 | `<a/>` | Self-closing element, no children | T1 |
| 5 | `&#0;` | Invalid numeric ref, left as literal | T1 |
| 6 | `&unknown;` | Left as literal text | T1 |
| 7 | Very deep nesting (>500) | `ElementDepthExceeded` error | T1 |
| 8 | UTF-8 BOM + content | BOM consumed, `HasBOM() = true` | T1 |
| ... | ... | ... | ... |

### 5.5 Cross-Validation

Run the same test inputs against both implementations and compare:

```
                    ┌──→  C++ TinyXML2  ──→  C++ output  ──┐
Test Input  ──────┤                                         ├──→  Compare
                    └──→  tinyxml2-rs   ──→  Rust output  ──┘
```

- Automated CI pipeline that builds both implementations
- Feeds shared test corpus to both
- Compares DOM structure and serialized output
- Reports any divergences

---

## 6. Compatibility Exceptions

### 6.1 Intentional Deviations

Some deviations from TinyXML2 are intentional and documented:

| Deviation | Rationale |
|-----------|-----------|
| Non-UTF-8 input produces `Err` instead of UB | Safety: Rust cannot have undefined behavior |
| `Result<T, XmlError>` instead of error codes | Idiomatic Rust error handling |
| `Option<NodeId>` instead of `nullptr` | Null safety via type system |
| No mutable string pointers | Rust ownership model |

### 6.2 Discovered Deviations

Any unintentional deviation discovered during testing should be:

1. Documented with a test case that demonstrates the difference
2. Triaged against the compatibility tier
3. Fixed if Tier 1, evaluated if Tier 2, noted if Tier 3
4. Added to the edge case catalog
