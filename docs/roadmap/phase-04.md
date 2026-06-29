# Phase 4: Writer/Serializer

> **Status:** 🔲 NOT STARTED  
> **Estimated Complexity:** MEDIUM (~1500 LOC)  
> **Dependencies:** Phase 3 (parser — needed for round-trip testing)  
> **Milestone:** `v0.0.4-alpha` internal

---

## Objectives

Serialize DOM trees back to XML text, matching TinyXML2's output format
precisely. Provide both DOM-driven serialization (traverse the tree and emit
XML) and a streaming API (manually construct XML output without building a
DOM first). The writer completes the read-write cycle and enables round-trip
testing.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                    Serialization Paths                    │
│                                                          │
│  Path A: DOM → Text              Path B: Streaming API   │
│  ┌──────────────┐                ┌────────────────────┐  │
│  │  Document     │                │   Printer          │  │
│  │  .to_string() │                │   .open_element()  │  │
│  │  .save_file() │                │   .push_attribute()│  │
│  │  .save_writer()│               │   .push_text()     │  │
│  └──────┬───────┘                │   .close_element() │  │
│         │                        └────────┬───────────┘  │
│         ▼                                 │              │
│  ┌──────────────┐                         │              │
│  │  Printer      │◄───────────────────────┘              │
│  │  (internal)   │                                       │
│  └──────┬───────┘                                       │
│         │                                                │
│         ▼                                                │
│  ┌──────────────┐                                       │
│  │  Output Sink  │  → String, File, Write trait          │
│  └──────────────┘                                       │
└─────────────────────────────────────────────────────────┘
```

---

## Deliverables

### 1. Printer Struct — `printer.rs`

The `Printer` is the central serialization engine, used by both DOM
serialization methods and the streaming API.

```rust
pub struct Printer {
    buffer: String,          // Output buffer
    compact: bool,           // true = no whitespace, false = pretty-print
    indent: u32,             // Current indent depth
    indent_str: String,      // Indent unit (default: 4 spaces)
    element_open: bool,      // Tracking if an element tag is still open
    first_element: bool,     // Is this the first element (for BOM)
    write_bom: bool,         // Whether to emit UTF-8 BOM
    entity_flag: bool,       // Whether to encode entities on output
}
```

### 2. Pretty-Print Mode

Formatted output with indentation for human readability:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<root>
    <child attr="value">
        <grandchild/>
        Text content
    </child>
    <!-- A comment -->
</root>
```

| Feature | Behavior |
|---------|----------|
| Indent unit | 4 spaces (configurable) |
| Element children | Each on its own line, indented |
| Text-only elements | `<tag>text</tag>` on single line (TinyXML2 behavior) |
| Attributes | On same line as opening tag |
| Comments | Indented to current level |
| Declarations | No indentation (top-level) |
| Empty elements | Self-closing `<tag/>` |
| Trailing newline | Final newline after last element |

### 3. Compact Mode

Minimized output with no extraneous whitespace:

```xml
<?xml version="1.0" encoding="UTF-8"?><root><child attr="value"><grandchild/>Text content</child><!-- A comment --></root>
```

| Feature | Behavior |
|---------|----------|
| No indentation | Zero whitespace between elements |
| No newlines | Entire document on one logical line |
| Same entity encoding | Entities still encoded |
| Text preserved | Text content unchanged |

### 4. DOM Serialization Methods

| Method | Description |
|--------|-------------|
| `Document::to_string() -> String` | Pretty-print the entire DOM |
| `Document::to_string_compact() -> String` | Compact-print the entire DOM |
| `Document::save_file(path: &Path) -> Result<()>` | Pretty-print to file |
| `Document::save_file_compact(path: &Path) -> Result<()>` | Compact-print to file |
| `Document::save_writer(writer: impl Write) -> Result<()>` | Pretty-print to any `Write` sink |
| `Document::save_writer_compact(writer: impl Write) -> Result<()>` | Compact-print to any `Write` sink |
| `impl Display for Document` | Delegates to `to_string()` |

### 5. Streaming API

The streaming API allows constructing XML output imperatively without building
a DOM tree first. Useful for report generation, logging, and high-performance
serialization.

| Method | Description |
|--------|-------------|
| `Printer::new() -> Printer` | Create a new pretty-print printer |
| `Printer::new_compact() -> Printer` | Create a new compact printer |
| `open_element(name: &str)` | Emit `<name` (tag left open for attributes) |
| `push_attribute(name: &str, value: &str)` | Emit ` name="value"` |
| `close_element()` | Emit `</name>` or `/>` if empty |
| `push_text(text: &str)` | Emit entity-encoded text content |
| `push_text_raw(text: &str)` | Emit text without entity encoding |
| `push_cdata(text: &str)` | Emit `<![CDATA[text]]>` |
| `push_comment(text: &str)` | Emit `<!--text-->` |
| `push_declaration(text: &str)` | Emit `<?xml text?>` |
| `push_unknown(text: &str)` | Emit `<!text>` |
| `result() -> &str` | Get accumulated output |
| `into_string() -> String` | Consume printer, return output |
| `clear()` | Reset printer for reuse |

### 6. Entity Encoding on Output

| Character | Encoded As | Context |
|-----------|-----------|---------|
| `&` | `&amp;` | Text and attribute values |
| `<` | `&lt;` | Text and attribute values |
| `>` | `&gt;` | Text only (TinyXML2 behavior) |
| `"` | `&quot;` | Attribute values only |
| `'` | `&apos;` | Not encoded (TinyXML2 behavior) |

### 7. BOM Output

| Feature | Description |
|---------|-------------|
| `Printer::set_bom(write: bool)` | Configure BOM output |
| UTF-8 BOM | `EF BB BF` prepended to output when enabled |
| Default | BOM disabled (TinyXML2 default) |

---

## Node-Type Serialization Rules

| Node Type | Pretty Format | Compact Format |
|-----------|--------------|----------------|
| Document | Children only (no wrapper tag) | Same |
| Element (empty) | `<tag/>` | `<tag/>` |
| Element (text only) | `<tag>text</tag>` (single line) | `<tag>text</tag>` |
| Element (children) | Open tag, newline, indented children, close tag | No whitespace |
| Text | Inline with parent element | Same |
| CData | `<![CDATA[content]]>` | Same |
| Comment | `<!--content-->` | Same |
| Declaration | `<?xml attrs?>` | Same |
| Unknown | `<!content>` | Same |

---

## Estimated Test Plan

| Category | Est. Tests | Description |
|----------|-----------|-------------|
| Pretty-print basic | 10 | Elements, text, comments, declarations |
| Pretty-print indent | 8 | Nested elements at various depths |
| Compact-print | 8 | Same inputs as pretty, verify no whitespace |
| Entity encoding | 10 | All special characters in text and attributes |
| Streaming API | 15 | Sequential open/close/text/comment operations |
| Round-trip | 12 | Parse → print → parse → print identity |
| File I/O | 6 | Save/load with pretty and compact modes |
| BOM output | 4 | With/without BOM, verify bytes |
| Display trait | 3 | `format!("{}", doc)` |
| Edge cases | 8 | Empty document, deeply nested, very long text |
| Custom indent | 4 | Tab indent, 2-space, 8-space |

**Estimated Total:** ~88 tests

---

## Round-Trip Testing Strategy

Round-trip testing is the primary validation mechanism for the writer:

```
Input XML → parse() → DOM₁ → to_string() → Output₁
Output₁   → parse() → DOM₂ → to_string() → Output₂

Assert: Output₁ == Output₂  (idempotent serialization)
Assert: DOM₁ ≅ DOM₂         (structurally equivalent)
```

> **Note:** `Input XML ≠ Output₁` is expected — TinyXML2 normalizes formatting.
> The key property is that the **second** round-trip is stable.

---

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Pretty-print format mismatch | Compatibility | Diff test output against TinyXML2 for 50+ inputs |
| Text-only element detection | Formatting bug | Match TinyXML2's heuristic exactly |
| Streaming API state machine | Correctness | Exhaustive invalid-sequence tests |
| Large output performance | UX | Pre-allocate buffer; benchmark against TinyXML2 |

---

## Acceptance Criteria

- [ ] Pretty-print output matches TinyXML2 format exactly
- [ ] Compact output matches TinyXML2 compact format exactly
- [ ] Round-trip parse→print→parse→print produces identical output (idempotent)
- [ ] Streaming API produces valid, well-formed XML
- [ ] Entity encoding is correct for all special characters
- [ ] File I/O works for both pretty and compact modes
- [ ] BOM output matches TinyXML2 behavior
- [ ] `Display` trait produces same output as `to_string()`
- [ ] All tests pass with zero warnings

---

## File Plan

| File | Responsibility |
|------|---------------|
| `tinyxml2/src/printer.rs` | `Printer` struct, streaming API, serialization logic |
| `tinyxml2/src/document.rs` | `to_string()`, `save_file()`, `save_writer()` (additions) |
| `tinyxml2/src/tests/printer_tests.rs` | Writer/printer test suite |
| `tinyxml2/src/tests/roundtrip_tests.rs` | Round-trip test suite |

---

## Previous Phase

← [Phase 3: XML Parser](./phase-03.md)

## Next Phase

→ [Phase 5: Visitor Pattern & Ergonomic API](./phase-05.md)
