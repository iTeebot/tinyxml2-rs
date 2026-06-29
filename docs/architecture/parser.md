# Parser Architecture

## Overview

The tinyxml2-rs parser is a **recursive-descent, single-pass** XML parser that builds the DOM directly into the generational arena. There is no intermediate AST, token stream, or event layer — parsed nodes are allocated in the arena and linked into the tree as they are recognized.

This approach matches TinyXML2's parsing strategy: each grammar production corresponds to a function that consumes input, creates a node, and returns the remaining input (or an error).

---

## Pipeline

```
┌──────────┐    ┌──────────────┐    ┌─────────┐    ┌───────────────────┐    ┌───────────┐
│  Input   │───▶│ BOM Detection│───▶│ Scanner │───▶│ Recursive Descent │───▶│   Arena   │
│  &str    │    │ strip_bom()  │    │  (util) │    │     Parser        │    │   (DOM)   │
└──────────┘    └──────────────┘    └─────────┘    └───────────────────┘    └───────────┘
```

### Stage Details

| Stage | Responsibility | Module |
|---|---|---|
| **Input** | Raw XML source as a `&str` slice. UTF-8 is required (Rust strings are always UTF-8). | caller |
| **BOM Detection** | Checks for and strips a UTF-8 BOM (`0xEF 0xBB 0xBF`) at the start of input. Sets `Document.has_bom` flag for round-trip fidelity. | `util::strip_bom()` |
| **Scanner** | Low-level character classification, whitespace skipping, name reading, entity resolution. These are stateless utility functions, not a separate lexer pass. | `util`, `entity` |
| **Recursive Descent Parser** | Consumes scanner output to recognize XML grammar productions. Creates `NodeData` in the arena and links nodes into the tree. Tracks line numbers and enforces depth limits. | `parser` (planned) |
| **Arena (DOM)** | Final output. A fully-linked DOM tree stored in the `Document`'s `Arena<NodeData>`. | `arena`, `dom` |

### No Intermediate AST

Unlike many XML parsers that produce a stream of SAX events or an intermediate AST, tinyxml2-rs writes nodes **directly into the arena** during parsing. This eliminates an entire allocation+copy pass and keeps the parser simple:

```
Traditional:  Input → Tokens → AST → DOM
tinyxml2-rs:  Input → DOM (direct)
```

---

## Scanner Responsibilities

The scanner is not a separate component but a collection of utility functions in `util.rs` and `entity.rs` that the parser calls as needed:

### Character Classification (`util.rs`)

| Function | Purpose |
|---|---|
| `is_whitespace(ch)` | Returns `true` for XML whitespace: space, tab, newline, carriage return (`\x20`, `\x09`, `\x0A`, `\x0D`). |
| `is_whitespace_byte(b)` | Byte-level version for ASCII fast paths. |
| `is_name_start_char(ch)` | XML NameStartChar: letters, `_`, `:`, and extended Unicode ranges per XML spec §2.3. |
| `is_name_char(ch)` | XML NameChar: NameStartChar plus digits, `-`, `.`, combining chars, extenders. |

### Whitespace and Name Reading (`util.rs`)

| Function | Purpose |
|---|---|
| `skip_whitespace(input)` | Advances past whitespace, returns `(remaining, newline_count)` for line tracking. |
| `collapse_whitespace(input)` | Replaces runs of whitespace with a single space, trims leading/trailing. Used in `Whitespace::Collapse` mode. |
| `read_name(input)` | Reads an XML name (NameStartChar followed by NameChar*), returns `(name, remaining)`. |

### Entity Resolution (`entity.rs`)

| Function | Purpose |
|---|---|
| `decode(input)` | Resolves a single entity reference at the current position. Returns the decoded character and remaining input. |
| `decode_cow(input)` | Decodes all entities in a string, returning `Cow::Borrowed` if no entities were present (zero-allocation fast path) or `Cow::Owned` if substitutions were made. |
| `encode_text(input)` | Escapes `&`, `<`, `>` for text content output. |
| `encode_attribute(input)` | Escapes `&`, `<`, `>`, `"`, `'` for attribute value output. |

---

## Parser Responsibilities

The parser module contains one function per grammar production:

### Element Parsing

```rust
fn parse_element(input: &str, parent: NodeId, doc: &mut Document, depth: u32)
    -> Result<&str, XmlError>
```

1. Match `<` + name via `read_name`.
2. Parse attributes in a loop until `>` or `/>`.
3. If self-closing (`/>`), return.
4. Otherwise, parse children recursively until `</name>`.
5. Verify closing tag name matches opening tag — mismatch produces `XmlError::MismatchedElement`.

### Attribute Parsing

```rust
fn parse_attributes(input: &str, element: NodeId, doc: &mut Document)
    -> Result<&str, XmlError>
```

1. Loop: `skip_whitespace`, attempt `read_name`.
2. If no name found, done (return to element parser).
3. Expect `=`, then quoted value (`"..."` or `'...'`).
4. Decode entities in value if `process_entities` is true.
5. Store attribute on the element node.

### Other Node Types

| Production | Trigger | Handler |
|---|---|---|
| Text | Any character data not starting with `<` | `parse_text()` |
| CDATA | `<![CDATA[` | `parse_cdata()` |
| Comment | `<!--` | `parse_comment()` |
| Declaration | `<?xml` | `parse_declaration()` |
| Unknown | `<!` (not comment or CDATA) | `parse_unknown()` |

---

## Line Tracking

The parser maintains a running line counter for error reporting:

```rust
struct ParserState<'a> {
    input: &'a str,
    line: u32,        // current line number, 1-indexed
    doc: &'a mut Document,
}
```

- `skip_whitespace()` returns the number of newlines consumed. The parser adds this to its `line` counter after each call.
- When a node is created, `NodeData.line_num` is set to the current line.
- When an error occurs, `XmlError::Parse { line, .. }` or `XmlError::MismatchedElement { line, .. }` captures the line for diagnostics.

---

## Entity Resolution

Entity resolution is controlled by `ParseOptions.process_entities` (default: `true`):

### When `process_entities = true`:

- Text content and attribute values are passed through `decode_cow()` during parsing.
- The 5 named entities are resolved: `&amp;` → `&`, `&lt;` → `<`, `&gt;` → `>`, `&quot;` → `"`, `&apos;` → `'`.
- Numeric character references are resolved: `&#65;` → `A`, `&#x41;` → `A`.
- Unrecognized entity references (e.g., `&foo;`) are passed through literally, matching TinyXML2 behavior.

### When `process_entities = false`:

- Entity references are left as-is in the stored text. The raw `&amp;` string appears in `NodeData.value`.
- This mode is useful for round-tripping XML without modifying entity encoding.

---

## Whitespace Modes

The `ParseOptions.whitespace` field controls text node whitespace handling:

| Mode | Behavior | Example Input | Stored Text |
|---|---|---|---|
| `Preserve` | All whitespace is kept exactly as-is. | `"  hello\n  world  "` | `"  hello\n  world  "` |
| `Collapse` | Leading/trailing whitespace is trimmed. Internal runs of whitespace are collapsed to a single space. | `"  hello\n  world  "` | `"hello world"` |
| `Pedantic` | Whitespace-only text nodes between elements are preserved. Acts like `Preserve` but affects whether whitespace-only nodes are created. | `"  "` between elements | Node created with `"  "` |

### Text node creation rules:

- In `Collapse` mode, if collapsing produces an empty string, no text node is created.
- In `Preserve` mode, all text content (including whitespace-only) produces a text node.
- In `Pedantic` mode, whitespace-only text between elements produces a text node (unlike TinyXML2's default which would skip it).

---

## BOM Handling

At the very start of parsing:

```rust
let (input, has_bom) = if starts_with_bom(input) {
    (strip_bom(input), true)
} else {
    (input, false)
};
doc.has_bom = has_bom;
```

- Only UTF-8 BOM is supported (Rust strings are UTF-8).
- The `has_bom` flag is stored on the `Document` so the printer can re-emit the BOM for round-trip fidelity.
- BOM detection uses `starts_with_bom()` which checks for the 3-byte sequence `0xEF 0xBB 0xBF` (encoded as the Unicode character U+FEFF in UTF-8).

---

## Depth Limiting

To prevent stack overflow from deeply nested XML (or malicious input), the parser enforces a maximum element nesting depth:

```rust
fn parse_element(..., depth: u32) -> Result<&str, XmlError> {
    if depth > self.options.max_depth {
        return Err(XmlError::ElementDepthExceeded {
            line: self.line,
            max_depth: self.options.max_depth,
        });
    }
    // ... parse children with depth + 1
}
```

- Default `max_depth`: **500** (matching TinyXML2's default).
- The depth counter is passed as a parameter to `parse_element`, incremented on each recursive call.
- Non-element nodes (text, comments, etc.) do not increment the depth counter.
- When exceeded, returns `XmlError::ElementDepthExceeded { line, max_depth }` immediately.

---

## Error Recovery

**There is no error recovery.** The parser uses a **fail-fast** strategy:

1. The first error encountered causes an immediate `return Err(XmlError::...)`.
2. No partial DOM is returned — the `Document` is left in an undefined state on error.
3. This matches TinyXML2's behavior: `XMLDocument::Parse()` returns an error code and sets error state; the document should not be used after a parse error.

### Error variants produced by the parser:

| Error | Cause |
|---|---|
| `Parse { kind: Element, .. }` | Malformed element tag, missing `>`, invalid name. |
| `Parse { kind: Attribute, .. }` | Malformed attribute: missing `=`, missing quotes, invalid name. |
| `Parse { kind: Text, .. }` | Invalid character in text content. |
| `Parse { kind: Cdata, .. }` | Unterminated `<![CDATA[` section. |
| `Parse { kind: Comment, .. }` | Unterminated `<!--` comment or `--` inside comment body. |
| `Parse { kind: Declaration, .. }` | Malformed `<?xml` declaration. |
| `Parse { kind: Unknown, .. }` | Unterminated `<!` unknown construct. |
| `Parse { kind: General, .. }` | General parse failure (unexpected EOF, invalid characters). |
| `MismatchedElement { line, expected, found }` | Closing tag `</found>` doesn't match opening tag `<expected>`. |
| `EmptyDocument` | Input contains no root element. |
| `ElementDepthExceeded { line, max_depth }` | Nesting exceeds `max_depth`. |

---

## Recursive Descent Grammar Sketch

The parser implements the following simplified grammar (not a formal XML spec grammar, but the subset that TinyXML2 supports):

```
document     ::= bom? misc* element misc*
misc         ::= comment | declaration | unknown | whitespace
element      ::= '<' name attribute* '/>'
               | '<' name attribute* '>' content* '</' name '>'
attribute    ::= name '=' quoted_value
quoted_value ::= '"' attr_chars '"' | "'" attr_chars "'"
content      ::= element | text | comment | cdata | declaration | unknown
text         ::= char_data+
cdata        ::= '<![CDATA[' cdata_chars ']]>'
comment      ::= '<!--' comment_chars '-->'
declaration  ::= '<?xml' attribute* '?>'
unknown      ::= '<!' unknown_chars '>'
```

Each production maps to a `parse_*` function:

| Production | Function |
|---|---|
| `document` | `parse_document()` — entry point |
| `element` | `parse_element()` — recursive |
| `attribute` | `parse_attributes()` — loop within element |
| `content` | `parse_children()` — dispatches based on next character |
| `text` | `parse_text()` |
| `cdata` | `parse_cdata()` |
| `comment` | `parse_comment()` |
| `declaration` | `parse_declaration()` |
| `unknown` | `parse_unknown()` |

---

## Comparison with TinyXML2's Parsing Approach

| Aspect | TinyXML2 (C++) | tinyxml2-rs (Rust) |
|---|---|---|
| **Strategy** | Recursive descent, manual character scanning | Recursive descent, `&str` slice advancing |
| **Input type** | `const char*` with manual pointer arithmetic | `&str` with slice operations and pattern matching |
| **Memory** | `MemPool` allocates node objects | Arena allocates `NodeData` values |
| **Error handling** | Sets error state on `XMLDocument`, returns null | Returns `Result<T, XmlError>`, no mutable error state |
| **String storage** | `StrPair` with lazy evaluation and in-place mutation | `String` / `Cow<str>` with eager evaluation |
| **Entity handling** | `StrPair` flags control lazy entity resolution | `decode_cow()` resolves during parse when enabled |
| **Depth limit** | Configurable, checked on element open | Same: `max_depth` checked on `parse_element` entry |
| **Whitespace** | `PRESERVE_WHITESPACE` / `COLLAPSE_WHITESPACE` enum | `Whitespace::Preserve` / `Collapse` / `Pedantic` enum |
| **BOM** | Detected and stripped; `_writeBOM` flag stored | `strip_bom()` + `has_bom` flag on `Document` |
| **Intermediate repr.** | None — direct DOM construction | Same — no intermediate AST |

### Key differences:

1. **No `StrPair`:** TinyXML2's `StrPair` is a lazy string class that defers entity resolution and normalization. tinyxml2-rs resolves entities eagerly during parsing using `Cow<str>` to avoid allocation when no entities are present.

2. **No in-place mutation:** TinyXML2 modifies the input buffer in place for performance (e.g., null-terminating strings). Rust's `&str` is immutable, so tinyxml2-rs uses slice operations and allocates `String` values for node content.

3. **Structured errors:** TinyXML2 stores a single error code + line number. tinyxml2-rs uses a rich `XmlError` enum with structured variants that carry context (expected tag name, found tag name, error kind, line number).

4. **Additional whitespace mode:** The `Pedantic` mode is a tinyxml2-rs addition that provides finer control over whitespace-only text node creation.
