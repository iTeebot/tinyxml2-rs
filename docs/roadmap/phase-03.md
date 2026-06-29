# Phase 3: XML Parser

> **Status:** 🔲 NOT STARTED  
> **Estimated Complexity:** HIGH (~2500 LOC)  
> **Dependencies:** Phase 2 (DOM Core)  
> **Milestone:** `v0.0.3-alpha` internal

---

## Objectives

Implement a recursive-descent XML parser that constructs a DOM tree from XML
text input. The parser must match TinyXML2's behavior exactly — accepting the
same inputs, rejecting the same malformed inputs, and producing structurally
identical DOM trees. This is the critical compatibility layer that determines
whether tinyxml2-rs can serve as a drop-in replacement.

---

## Parser Architecture

```
                    Input: &str / &[u8] / File / Read
                              │
                              ▼
                    ┌──────────────────┐
                    │   BOM Detection  │
                    │   (skip UTF-8)   │
                    └────────┬─────────┘
                             │
                             ▼
                    ┌──────────────────┐
                    │  Parse Document  │◄── entry point
                    │  (declaration?,  │
                    │   children*)     │
                    └────────┬─────────┘
                             │
              ┌──────────────┼──────────────┐
              ▼              ▼              ▼
     ┌───────────────┐ ┌──────────┐ ┌──────────────┐
     │ Parse Element │ │Parse Text│ │ Parse Comment│
     │ (attrs, body) │ │ / CDATA  │ │ / PI / Unkn  │
     └───────┬───────┘ └──────────┘ └──────────────┘
             │
    ┌────────┼────────┐
    ▼        ▼        ▼
  attrs   children  close-tag
```

The parser is a single-pass, recursive-descent parser operating on a `&str`
slice with a cursor position. It does **not** use a separate tokenizer/lexer
stage — character-level parsing is performed inline, matching TinyXML2's
approach.

---

## Deliverables

### 1. Core Parse Entry Points

| Method | Description |
|--------|-------------|
| `Document::parse(xml: &str) -> Result<()>` | Parse XML string into this document's DOM |
| `Document::load_file(path: &Path) -> Result<()>` | Read file to string, then parse |
| `Document::load_reader(reader: impl Read) -> Result<()>` | Read from any `std::io::Read` source |

All entry points clear any existing DOM content before parsing.

### 2. Recursive-Descent Parse Functions

| Function | Parses | Termination |
|----------|--------|-------------|
| `parse_document()` | Top-level content | EOF |
| `parse_element()` | `<name attrs...>children</name>` or `<name attrs.../>` | Close tag or self-close |
| `parse_attributes()` | `name="value"` pairs | `>` or `/>` |
| `parse_text()` | Character data between tags | `<` |
| `parse_cdata()` | `<![CDATA[...]]>` | `]]>` |
| `parse_comment()` | `<!--...-->` | `-->` |
| `parse_declaration()` | `<?xml ...?>` | `?>` |
| `parse_unknown()` | `<!...>` (DTD, etc.) | `>` |

### 3. Entity Resolution

| Feature | Description |
|---------|-------------|
| Predefined entities | `&amp;` `&lt;` `&gt;` `&quot;` `&apos;` resolved during parse |
| Decimal char refs | `&#NNN;` resolved to UTF-8 |
| Hex char refs | `&#xHHHH;` resolved to UTF-8 |
| Invalid refs | Produce `XmlError::XmlErrorParsingText` |
| Attribute values | Entities resolved within attribute values |
| Text content | Entities resolved within text nodes |

### 4. Whitespace Handling

Three modes controlled by `ParseOptions::whitespace`:

| Mode | Behavior |
|------|----------|
| `Preserve` | All whitespace kept as-is (default for TinyXML2 compatibility) |
| `Collapse` | Leading/trailing whitespace trimmed; internal runs collapsed to single space |
| `NormalizeAttribute` | Attribute-specific normalization per XML 1.0 §3.3.3 |

### 5. Input Preprocessing

| Feature | Description |
|---------|-------------|
| BOM detection | UTF-8 BOM (`EF BB BF`) detected and skipped |
| Line tracking | Line number incremented on `\n`; stored on each parsed node |
| Cursor management | Internal `Parser` struct tracks position, line, and depth |

### 6. Safety & Limits

| Feature | Description |
|---------|-------------|
| Depth limiting | Configurable max nesting depth (default: 100) to prevent stack overflow |
| Unterminated checks | Detect unclosed tags, comments, CDATA, attribute values |
| Duplicate attributes | Detected and reported as `XmlError::XmlErrorParsingAttribute` |
| Empty document | Valid — produces Document with no children |

---

## Error Reporting

The parser sets detailed error information on the `Document`:

| Error Code | Trigger |
|------------|---------|
| `XmlErrorEmptyDocument` | Input is empty or whitespace-only |
| `XmlErrorParsingElement` | Malformed element open/close tag |
| `XmlErrorParsingAttribute` | Malformed attribute name or value |
| `XmlErrorParsingText` | Invalid entity reference in text |
| `XmlErrorParsingCdata` | Unterminated `<![CDATA[` |
| `XmlErrorParsingComment` | Unterminated `<!--` |
| `XmlErrorParsingDeclaration` | Malformed `<?xml ...?>` |
| `XmlErrorParsingUnknown` | Unterminated `<!...>` |
| `XmlErrorMismatchedElement` | Close tag doesn't match open tag |
| `XmlErrorParsingDepthExceeded` | Nesting exceeds configured limit |

All error codes include the line number where the error was detected.

---

## Parser State Machine

```
struct Parser<'a> {
    input: &'a str,       // Full input text
    cursor: usize,        // Current byte position
    line: u32,            // Current line number (1-based)
    depth: u32,           // Current nesting depth
    options: ParseOptions, // Configuration
}
```

The parser is not stored on `Document` — it is a local struct created for the
duration of a single `parse()` call. This ensures the parser state cannot leak
or be accidentally reused.

---

## Edge Cases & TinyXML2 Compatibility

| Edge Case | TinyXML2 Behavior | Our Behavior |
|-----------|-------------------|--------------|
| Self-closing `<br/>` | Creates empty Element | Same |
| Space before `/>` | `<br />` is valid | Same |
| Single-quoted attrs | `attr='value'` accepted | Same |
| Unquoted attrs | **Rejected** | Same — error |
| `<![CDATA[` in text | Creates Text node with `is_cdata=true` | Same |
| Multiple root elements | Accepted (non-conforming but TinyXML2 allows it) | Same |
| BOM + declaration | BOM skipped, declaration parsed | Same |
| Empty attribute value | `attr=""` is valid | Same |
| CR/LF normalization | `\r\n` → `\n`, lone `\r` → `\n` | Same |
| Nested comments | **Rejected** (`--` inside comment) | Same |

---

## Estimated Test Plan

| Category | Est. Tests | Description |
|----------|-----------|-------------|
| Basic elements | 15 | Single, nested, self-closing, with content |
| Attributes | 18 | Single/double quotes, entities in values, empty values |
| Text content | 12 | Plain text, entities, whitespace modes |
| CDATA | 8 | Simple, nested-looking, empty, with special chars |
| Comments | 8 | Simple, multi-line, edge cases |
| Declarations | 6 | Standard XML decl, with encoding, standalone |
| Unknown nodes | 5 | DOCTYPE, processing instructions |
| Error cases | 20 | All error codes with minimal reproduction |
| Whitespace modes | 10 | Preserve vs collapse for same input |
| BOM handling | 4 | With/without BOM, BOM + declaration |
| Depth limiting | 4 | At limit, over limit |
| Entity resolution | 12 | All predefined, decimal, hex, invalid |
| Round-trip prep | 8 | Parse and verify tree structure for Phase 4 tests |

**Estimated Total:** ~130 tests

---

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Entity parsing edge cases | Compatibility | Port TinyXML2's exact entity parsing; diff-test outputs |
| Whitespace normalization in attrs | Subtle bugs | Test against TinyXML2 with attribute-heavy XML |
| Self-closing tag handling | Compatibility | Explicit test matrix for `<br/>`, `<br />`, `<br></br>` |
| Stack overflow on deep XML | Crash | Configurable depth limit with clear error |
| Performance on large files | UX | Benchmark against TinyXML2; optimize hot paths |

---

## Acceptance Criteria

- [ ] Parses all valid XML that TinyXML2 accepts
- [ ] Rejects all malformed XML that TinyXML2 rejects
- [ ] Produces identical DOM trees (same structure, values, node types)
- [ ] Error codes match TinyXML2 for all error cases
- [ ] Line numbers are accurate for all error reports
- [ ] BOM handling matches TinyXML2
- [ ] All three whitespace modes produce correct output
- [ ] Depth limit prevents stack overflow
- [ ] Performance within 2× of TinyXML2 for typical inputs
- [ ] All tests pass with zero warnings

---

## File Plan

| File | Responsibility |
|------|---------------|
| `tinyxml2/src/parser.rs` | `Parser` struct, all parse functions |
| `tinyxml2/src/document.rs` | `parse()`, `load_file()`, `load_reader()` entry points (additions) |
| `tinyxml2/src/tests/parser_tests.rs` | Parser test suite |
| `tinyxml2/src/tests/fixtures/` | XML test fixtures |

---

## Previous Phase

← [Phase 2: DOM Core](./phase-02.md)

## Next Phase

→ [Phase 4: Writer/Serializer](./phase-04.md)
