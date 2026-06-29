# Serializer Design

## Overview

The tinyxml2-rs serializer (Printer) converts an in-memory DOM tree back into an XML string. It supports two output modes—pretty-printed (human-readable) and compact (minimal size)—and uses a visitor pattern to walk the DOM tree, emitting XML markup incrementally into an internal buffer.

The serializer is designed to produce output that is:
- **Round-trip safe**: Parsing the output produces a semantically equivalent DOM tree.
- **Compatible**: Output format matches TinyXML2's `XMLPrinter` output.
- **Efficient**: Single-pass traversal with pre-sized buffer allocation.

---

## Printer Struct

```rust
pub struct Printer {
    /// The output buffer accumulating the serialized XML.
    buffer: String,

    /// Whether to produce compact output (no indentation or extra whitespace).
    compact: bool,

    /// Current nesting depth for indentation (0 = root level).
    depth: usize,

    /// Whether the current element has text content (suppresses newlines).
    element_has_text: bool,

    /// Stack tracking whether each open element has children
    /// (used to decide self-closing vs explicit close).
    has_children_stack: Vec<bool>,

    /// Whether the first element has been written (controls leading newline).
    first_element: bool,
}
```

### Construction

```rust
impl Printer {
    /// Create a new pretty-printing Printer.
    pub fn new() -> Self;

    /// Create a new compact Printer (no indentation or newlines).
    pub fn compact() -> Self;

    /// Create a Printer with a pre-allocated buffer of the given capacity.
    pub fn with_capacity(capacity: usize, compact: bool) -> Self;
}
```

---

## Visitor-Based Serialization

The serializer uses the Visitor pattern to decouple DOM traversal from XML output generation. The `Document` drives traversal via an `accept()` method; the `Printer` implements the `Visitor` trait to handle each node type.

### Traversal Flow

```
Document::accept(&self, visitor)
  │
  ├─► visitor.visit_enter_document(document)
  │
  ├─► For each child node:
  │     ├─► Declaration  → visitor.visit_declaration(decl)
  │     ├─► Element      → visitor.visit_enter_element(elem)
  │     │                    ├─► recurse into children
  │     │                    └─► visitor.visit_exit_element(elem)
  │     ├─► Text         → visitor.visit_text(text)
  │     ├─► Comment      → visitor.visit_comment(comment)
  │     └─► Unknown      → visitor.visit_unknown(unknown)
  │
  └─► visitor.visit_exit_document(document)
```

### Visitor Trait

```rust
pub trait Visitor {
    /// Called when entering the document (before any nodes).
    fn visit_enter_document(&mut self, doc: &Document) -> bool;

    /// Called when leaving the document (after all nodes).
    fn visit_exit_document(&mut self, doc: &Document) -> bool;

    /// Called when entering an element (before its children).
    /// The element's attributes are available via `elem.attributes()`.
    fn visit_enter_element(&mut self, elem: &Element) -> bool;

    /// Called when leaving an element (after all its children).
    fn visit_exit_element(&mut self, elem: &Element) -> bool;

    /// Called for text nodes (both regular text and CDATA).
    fn visit_text(&mut self, text: &Text) -> bool;

    /// Called for comment nodes.
    fn visit_comment(&mut self, comment: &Comment) -> bool;

    /// Called for XML declaration nodes.
    fn visit_declaration(&mut self, decl: &Declaration) -> bool;

    /// Called for unknown nodes (<!...>).
    fn visit_unknown(&mut self, unknown: &Unknown) -> bool;
}
```

Each callback returns `bool`:
- `true`: Continue traversal.
- `false`: Stop traversal immediately (early exit).

---

## Streaming API

In addition to visitor-based serialization driven by `Document::accept()`, the Printer provides a streaming (push) API for building XML output programmatically without a DOM:

```rust
impl Printer {
    /// Write the XML declaration: <?xml version="1.0"?>
    pub fn push_header(&mut self, version: &str, encoding: Option<&str>, standalone: Option<bool>);

    /// Open an element: <name
    pub fn open_element(&mut self, name: &str);

    /// Add an attribute to the currently open element: key="value"
    pub fn push_attribute(&mut self, name: &str, value: &str);

    /// Write text content inside the current element.
    pub fn push_text(&mut self, text: &str, cdata: bool);

    /// Write a comment: <!-- text -->
    pub fn push_comment(&mut self, text: &str);

    /// Write an unknown node: <!text>
    pub fn push_unknown(&mut self, text: &str);

    /// Close the most recently opened element: </name> or />
    pub fn close_element(&mut self);
}
```

### Streaming Example

```rust
let mut printer = Printer::new();
printer.push_header("1.0", Some("UTF-8"), None);
printer.open_element("root");
printer.push_attribute("version", "2");
printer.open_element("child");
printer.push_text("Hello, world!", false);
printer.close_element(); // </child>
printer.close_element(); // </root>

// Output:
// <?xml version="1.0" encoding="UTF-8"?>
// <root version="2">
//     <child>Hello, world!</child>
// </root>
```

---

## Pretty-Print Formatting

When `compact` is `false` (the default), the Printer produces human-readable output:

- **Indentation**: 4 spaces per nesting level.
- **Newlines**: Between elements (but not between an element's opening tag and its text content).
- **Depth tracking**: `depth` increments on `visit_enter_element` / `open_element` and decrements on `visit_exit_element` / `close_element`.

### Indentation Rules

```
depth=0:  <root>                     (no indent)
depth=1:      <child>               (4 spaces)
depth=2:          <grandchild/>     (8 spaces)
depth=1:      </child>              (4 spaces)
depth=0:  </root>                    (no indent)
```

### Text Content Handling

When an element contains text, the text is written inline without newlines or indentation:

```xml
<element>text content</element>     <!-- NOT -->
<element>
    text content
</element>
```

This is tracked via the `element_has_text` flag, which suppresses the newline and indent before the closing tag when text was written.

### Mixed Content

When an element contains both text and child elements (mixed content), text nodes are written inline and child elements are indented:

```xml
<parent>
    Some text
    <child/>
    More text
</parent>
```

---

## Compact Mode

When `compact` is `true`, the Printer produces minimal output:

- No indentation.
- No newlines between elements.
- No extra whitespace.

```xml
<?xml version="1.0"?><root version="2"><child>Hello, world!</child></root>
```

Compact mode is useful for:
- Network transmission (minimal bandwidth).
- Machine-to-machine communication (whitespace is irrelevant).
- Comparison/diffing (deterministic output).

---

## Entity Encoding During Output

The serializer applies entity encoding to ensure output is valid XML:

| Context | Encoding Function | Characters Escaped |
|---|---|---|
| Text content | `encode_text()` | `&` `<` `>` |
| Attribute values | `encode_attribute()` | `&` `<` `>` `"` `'` |
| Comments | None | Raw content preserved |
| CDATA sections | None | Raw content preserved |
| Unknown nodes | None | Raw content preserved |

```rust
// During visit_text / push_text:
let encoded = encode_text(text.value());
buffer.push_str(&encoded);

// During attribute serialization:
let encoded = encode_attribute(attr.value());
buffer.push_str(&encoded);
```

> [!IMPORTANT]
> CDATA text nodes are serialized with `<![CDATA[...]]>` delimiters and no entity encoding, preserving their raw content. The serializer distinguishes CDATA text from regular text via an internal flag on the text node.

---

## BOM Output

If the document was parsed from input containing a UTF-8 Byte Order Mark (BOM: `\xEF\xBB\xBF`), the serializer optionally re-emits the BOM at the start of the output:

```rust
fn visit_enter_document(&mut self, doc: &Document) -> bool {
    if doc.has_bom() {
        self.buffer.push_str("\u{FEFF}"); // UTF-8 BOM
    }
    true
}
```

This preserves round-trip fidelity for documents that originally included a BOM.

---

## Buffer Access

```rust
impl Printer {
    /// Returns the serialized XML as a string slice.
    pub fn as_str(&self) -> &str;

    /// Consumes the Printer and returns the serialized XML as a String.
    pub fn into_string(self) -> String;

    /// Returns the current length of the output buffer in bytes.
    pub fn len(&self) -> usize;

    /// Returns true if the output buffer is empty.
    pub fn is_empty(&self) -> bool;
}
```

### Output to Memory (Default)

The primary output target is an in-memory `String` buffer. This is the most common use case and allows the caller to inspect, modify, or write the output to any destination.

### Output to File (Planned)

A future extension will support direct-to-file output via `std::io::Write`:

```rust
// Planned API:
impl Printer {
    pub fn write_to<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()>;
}

// Usage:
let mut file = File::create("output.xml")?;
printer.write_to(&mut file)?;
```

This avoids buffering the entire document in memory for large files.

### CStr for FFI (Planned)

For the optional FFI layer, the Printer will provide a null-terminated C string view:

```rust
// Planned API:
impl Printer {
    pub fn as_c_str(&self) -> &CStr;
}
```

This enables C/C++ consumers to access the output without copying.

---

## Serialization by Node Type

### Self-Closing Elements

Elements with no children are serialized as self-closing tags:

```xml
<element/>
<element attr="value"/>
```

The Printer tracks whether children have been written between `visit_enter_element` and `visit_exit_element`. If no children were written, the element is closed with `/>` instead of emitting `></element>`.

```rust
fn visit_enter_element(&mut self, elem: &Element) -> bool {
    // Write: <name
    // Write attributes: attr="value"
    // Push false onto has_children_stack (no children yet)
    self.has_children_stack.push(false);
    true
}

fn visit_exit_element(&mut self, elem: &Element) -> bool {
    let has_children = self.has_children_stack.pop().unwrap();
    if has_children {
        // Write: </name>
    } else {
        // Write: />
    }
    true
}
```

When any child callback fires (text, comment, nested element), the top of `has_children_stack` is set to `true`, and the pending `>` for the parent element's opening tag is emitted.

### Attribute Serialization

Attributes are serialized as `name="value"` pairs, with the value entity-encoded via `encode_attribute()`:

```
Attribute: class = "my "special" class"
Output:    class="my &quot;special&quot; class"
```

Multiple attributes are separated by a single space:

```xml
<element first="1" second="2" third="3"/>
```

Attribute order matches the order they were parsed or added to the element.

### Declaration Serialization

XML declarations are serialized with the `<?xml ... ?>` syntax:

```xml
<?xml version="1.0"?>
<?xml version="1.0" encoding="UTF-8"?>
<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
```

Only attributes that are present on the declaration node are emitted. The declaration is always emitted before any other nodes.

### Comment Serialization

Comments are serialized with `<!-- ... -->` delimiters:

```xml
<!-- This is a comment -->
```

The comment content is written exactly as stored in the DOM—no entity encoding is applied. In pretty-print mode, comments are indented to match their depth level.

### CDATA Serialization

CDATA sections are serialized with `<![CDATA[ ... ]]>` delimiters:

```xml
<![CDATA[This <is> raw & unencoded content]]>
```

The CDATA content is written exactly as stored—no entity encoding is applied. The serializer identifies CDATA text nodes by their internal CDATA flag and emits the appropriate delimiters.

### Unknown Node Serialization

Unknown nodes are serialized with `<! ... >` delimiters:

```xml
<!DOCTYPE html PUBLIC "-//W3C//DTD HTML 4.01//EN">
```

The content between the delimiters is written exactly as stored in the DOM.

---

## Compatibility with TinyXML2's XMLPrinter

The tinyxml2-rs Printer aims to produce output identical to TinyXML2's `XMLPrinter` in both pretty-print and compact modes:

| Feature | TinyXML2 XMLPrinter | tinyxml2-rs Printer |
|---|---|---|
| Default indentation | 4 spaces | 4 spaces |
| Self-closing elements | `<e/>` (no space before `/>`) | `<e/>` |
| Compact mode | `SetCompact(true)` | `Printer::compact()` |
| Entity encoding (text) | `&` `<` `>` | `&` `<` `>` via `encode_text()` |
| Entity encoding (attr) | `&` `<` `>` `"` `'` | `&` `<` `>` `"` `'` via `encode_attribute()` |
| BOM preservation | Re-emits BOM if present | Re-emits BOM if `has_bom()` |
| Streaming API | `PushHeader`, `OpenElement`, etc. | Matching method names and semantics |
| Visitor interface | `XMLVisitor` with `Visit*` callbacks | `Visitor` trait with `visit_*` callbacks |
| Output access | `CStr()` | `as_str()`, `into_string()` |
| File output | `XMLPrinter(FILE*)` constructor | Planned `write_to()` method |
| Declaration format | `<?xml version="1.0"?>` | `<?xml version="1.0"?>` |
| Comment format | `<!-- text -->` | `<!-- text -->` |
| CDATA format | `<![CDATA[text]]>` | `<![CDATA[text]]>` |

> [!NOTE]
> Minor formatting differences may exist in edge cases (e.g., trailing newlines, whitespace in empty elements). These will be addressed through compatibility testing against TinyXML2's output during implementation.
