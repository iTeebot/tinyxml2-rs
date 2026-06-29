# Parser Design

## Overview

The tinyxml2-rs parser is a recursive-descent parser that consumes an XML document as a UTF-8 string and produces a DOM tree stored in a generational arena. The parser is designed for:

- **Single-pass processing**: The input is read once, front-to-back, with no backtracking.
- **Immediate error reporting**: The first syntax error terminates parsing with a descriptive `XmlError::Parse`. No error recovery is attempted, and no partial trees are returned.
- **Streaming construction**: DOM nodes are inserted into the arena as they are parsed, avoiding intermediate AST representations.
- **Configurable behavior**: Entity processing, whitespace handling, and depth limits are controlled via `ParseOptions`.

The parser follows TinyXML2's parsing strategy closely while replacing C-style pointer manipulation with Rust's safe byte-slice indexing.

---

## Token Types

The parser recognizes the following syntactic constructs in the input:

| Token | Pattern | Example |
|---|---|---|
| Element open | `<Name` | `<div` |
| Element close | `</Name>` | `</div>` |
| Self-close | `/>` | `<br/>` |
| Comment | `<!-- ... -->` | `<!-- note -->` |
| CDATA section | `<![CDATA[ ... ]]>` | `<![CDATA[raw]]>` |
| XML declaration | `<?xml ... ?>` | `<?xml version="1.0"?>` |
| Unknown node | `<! ... >` | `<!DOCTYPE html>` |
| Attribute | `Name = "value"` | `id="main"` |
| Text content | Character data | `Hello, world!` |

The parser does not use a separate lexer/tokenizer phase. Instead, it identifies tokens inline during recursive descent by examining the next few bytes of input.

---

## Recursive Descent Grammar

The following production rules define the grammar recognized by the parser. The notation is EBNF-like, with `*` for zero-or-more, `?` for optional, and `|` for alternation.

### Productions

```
document    := prolog? misc* element misc*
prolog      := declaration?
element     := '<' Name attribute* ('/>' | '>' content '</' Name '>')
content     := (element | text | comment | cdata | unknown)*
attribute   := Name '=' ('"' value '"' | "'" value "'")
text        := character_data
comment     := '<!--' comment_content '-->'
cdata       := '<![CDATA[' cdata_content ']]>'
declaration := '<?xml' attribute* '?>'
unknown     := '<!' unknown_content '>'
misc        := comment | unknown | whitespace_text
```

### Terminals

```
Name             := NameStartChar (NameChar)*
NameStartChar    := [a-zA-Z_:] | [#xC0-#xD6] | ...  (XML spec)
NameChar         := NameStartChar | [0-9.\-]
character_data   := [^<]*  (any characters except '<')
comment_content  := (. - '-->')*
cdata_content    := (. - ']]>')*
unknown_content  := (. - '>')*
value            := [^"]*  (for double-quoted) | [^']*  (for single-quoted)
```

> [!NOTE]
> The grammar is intentionally more permissive than the XML 1.0 specification in some areas (e.g., allowing unknown `<!...>` nodes) to match TinyXML2's behavior. This enables parsing of documents containing `<!DOCTYPE>` declarations and other constructs that TinyXML2 preserves as opaque unknown nodes.

---

## Parsing Strategy by Node Type

### Element Parsing

**Entry condition:** Current byte is `<` and the next byte is a `NameStartChar`.

**Algorithm:**

1. Consume the `<` character.
2. Read the element name (scan until whitespace, `/`, or `>`).
3. Parse zero or more attributes (see Attribute Parsing below).
4. Check the next token:
   - `/>`: Self-closing element. Create an element node with no children. Return.
   - `>`: Opening tag complete. Parse content (children).
5. After parsing content, expect `</`:
   - Read the closing tag name.
   - Compare with the opening tag name. If mismatch → `XmlError::MismatchedElement`.
   - Expect `>` after the closing name.
6. Return the element node.

```
Input:  <div class="main">Hello</div>
        ^                            ^
        Start                        End

Steps:  1. '<'
        2. Name = "div"
        3. Attribute: class = "main"
        4. '>' → parse content
        5. Text: "Hello"
        6. '</div>' → verify match
```

### Attribute Parsing

**Entry condition:** Inside an element tag, after the element name, current position is at whitespace or a `NameStartChar`.

**Algorithm:**

1. Skip whitespace.
2. If next byte is `/` or `>` or `?` → no more attributes, return.
3. Read the attribute name (scan until `=`, whitespace, or `>`).
4. Skip whitespace.
5. Expect `=`. If not found → `XmlError::Parse { kind: Attribute }`.
6. Skip whitespace.
7. Read the opening quote character (`"` or `'`). If neither → `XmlError::Parse { kind: Attribute }`.
8. Scan the attribute value until the matching closing quote.
9. If `processEntities` is `true`, decode entity references in the value via `decode_cow()`.
10. Store the attribute name-value pair on the element node.
11. Repeat from step 1.

**Quote matching:** The parser supports both single-quoted (`'value'`) and double-quoted (`"value"`) attribute values. The opening and closing quote characters must match:

```xml
<e attr="valid"/>      ✓ double-quoted
<e attr='valid'/>      ✓ single-quoted
<e attr="invalid'/>    ✗ mismatched quotes → ParseError
```

### Text Parsing

**Entry condition:** Inside element content, current byte is not `<`.

**Algorithm:**

1. Record the start position.
2. Scan forward until `<` is encountered or input ends.
3. Extract the text substring.
4. If `processEntities` is `true`, decode entity references via `decode_cow()`.
5. Apply whitespace normalization based on the `Whitespace` mode:

| Mode | Behavior |
|---|---|
| `Preserve` (default) | Keep all whitespace exactly as-is. |
| `Collapse` | Replace runs of whitespace (space, tab, CR, LF) with a single space. Trim leading/trailing whitespace. |
| `Pedantic` | Normalize line endings (CR+LF → LF, CR → LF) but otherwise preserve whitespace. |

6. If the resulting text is empty after whitespace processing, skip creating a text node.
7. Otherwise, create a text node and add it as a child of the current element.

### Comment Parsing

**Entry condition:** Current position matches `<!--`.

**Algorithm:**

1. Consume the `<!--` delimiter (4 bytes).
2. Scan forward for the `-->` terminator.
3. If end of input is reached without finding `-->` → `XmlError::Parse { kind: Comment }`.
4. Extract the content between `<!--` and `-->`.
5. Create a comment node with the raw content. **No entity processing** is performed on comments.
6. Advance past `-->`.

```
Input:  <!-- This is a comment -->
        ^^^^                   ^^^
        Open                   Close

Content: " This is a comment "
```

> [!IMPORTANT]
> Per the XML specification, the string `--` (double hyphen) should not appear inside a comment. However, following TinyXML2's behavior, the parser does **not** enforce this restriction. It simply scans for the first occurrence of `-->`.

### CDATA Parsing

**Entry condition:** Current position matches `<![CDATA[`.

**Algorithm:**

1. Consume the `<![CDATA[` delimiter (9 bytes).
2. Scan forward for the `]]>` terminator.
3. If end of input is reached without finding `]]>` → `XmlError::Parse { kind: Cdata }`.
4. Extract the content between `<![CDATA[` and `]]>`.
5. Create a text node marked as CDATA with the raw content. **No entity processing** is performed on CDATA sections—this is their defining characteristic.
6. Advance past `]]>`.

```
Input:  <![CDATA[<not>&markup>]]>
        ^^^^^^^^^              ^^^
        Open                   Close

Content: "<not>&markup>"  (stored as-is, no decoding)
```

CDATA text nodes are distinguished from regular text nodes by an internal flag so that the serializer can emit them with proper `<![CDATA[...]]>` delimiters.

### Declaration Parsing

**Entry condition:** Current position matches `<?xml` (case-sensitive).

**Algorithm:**

1. Consume the `<?xml` delimiter.
2. Parse attributes using the same Attribute Parsing logic (version, encoding, standalone).
3. Skip whitespace.
4. Expect `?>`. If not found → `XmlError::Parse { kind: Declaration }`.
5. Create a declaration node with the parsed attributes.

```
Input:  <?xml version="1.0" encoding="UTF-8"?>
        ^^^^^                                ^^
        Open                                 Close

Attributes: version="1.0", encoding="UTF-8"
```

> [!NOTE]
> The parser does not validate the XML declaration's attribute values (e.g., it does not reject `version="2.0"`). This matches TinyXML2's behavior of treating the declaration as a container for key-value pairs.

### Unknown Node Parsing

**Entry condition:** Current position matches `<!` but does **not** match `<!--` (comment) or `<![CDATA[` (CDATA).

**Algorithm:**

1. Consume the `<!` delimiter.
2. Scan forward for the matching `>` terminator, tracking nested angle brackets.
3. If end of input is reached without finding `>` → `XmlError::Parse { kind: Unknown }`.
4. Extract the raw content between `<!` and `>`.
5. Create an unknown node containing the raw content string.

This handles `<!DOCTYPE ...>`, `<!ENTITY ...>`, and any other unrecognized `<!` constructs by preserving them opaquely in the DOM.

---

## Entity Resolution

Entity references are resolved **only** in:
- Text content
- Attribute values

Entity references are **not** resolved in:
- Comments (`<!-- &amp; stays as &amp; -->`)
- CDATA sections (`<![CDATA[&amp; stays as &amp;]]>`)
- Unknown nodes (`<!DOCTYPE &amp; stays as &amp;>`)

Resolution only occurs when `ParseOptions::process_entities` is `true` (the default). When `false`, all text is stored verbatim.

---

## Whitespace Normalization

Whitespace normalization is applied to text nodes based on the `ParseOptions::whitespace` setting. It occurs **after** entity decoding (so `&#x20;` is treated as a space for normalization purposes).

```
Whitespace::Preserve (default):
  Input:  "  hello\n  world  "
  Output: "  hello\n  world  "

Whitespace::Collapse:
  Input:  "  hello\n  world  "
  Output: "hello world"

Whitespace::Pedantic:
  Input:  "  hello\r\n  world  "
  Output: "  hello\n  world  "
```

---

## Self-Closing Elements

Self-closing elements (`<element/>`) create an element node with no children. They are syntactically equivalent to `<element></element>`:

```xml
<br/>               <!-- self-closing: no children -->
<br></br>           <!-- equivalent: empty element -->
<img src="a.png"/>  <!-- self-closing with attribute -->
```

The parser detects self-close by checking for `/>` after attribute parsing. When found, it skips content parsing and immediately returns the element node.

---

## Malformed Input Handling

The parser follows a **fail-fast** strategy:

1. **No error recovery**: The first syntax error terminates parsing immediately.
2. **No partial trees**: On error, no DOM tree is returned. The caller receives `Err(XmlError)`.
3. **No lookahead recovery**: The parser does not attempt to skip malformed sections and resume parsing.
4. **Descriptive errors**: Every error includes the `ParseErrorKind` identifying what was being parsed and the `line` number where the error was detected.

This is consistent with TinyXML2's behavior, which also terminates on the first error.

**Examples of malformed input and resulting errors:**

| Input | Error |
|---|---|
| `<div><span></div>` | `MismatchedElement { line: 1, expected: "span", found: "div" }` |
| `<div attr=value>` | `Parse { kind: Attribute, line: 1, message: "expected quote" }` |
| `<div attr="val` | `Parse { kind: Attribute, line: 1, message: "unterminated attribute value" }` |
| `<!-- no end` | `Parse { kind: Comment, line: 1, message: "unterminated comment" }` |
| *(empty string)* | `EmptyDocument` |

---

## Depth Tracking

The parser maintains a depth counter to prevent stack overflow from maliciously or accidentally deep XML documents:

```
Depth tracking:
  <a>           depth: 0 → 1 ✓
    <b>         depth: 1 → 2 ✓
      <c>       depth: 2 → 3 ✓
      ...
      <deep>    depth: 499 → 500 ✓ (at max_depth)
        <boom>  depth: 500 → 501 ✗ ElementDepthExceeded
```

**Rules:**
- Depth starts at 0.
- Each `<element>` open tag increments depth by 1.
- Each `</element>` close tag (or `/>` self-close) decrements depth by 1.
- Before incrementing, the parser checks `depth < max_depth`. If not → `XmlError::ElementDepthExceeded { line, max_depth }`.
- The default `max_depth` is 500 (from `ParseOptions::max_depth`).
- Self-closing elements (`<e/>`) increment and immediately decrement, so they consume one level of depth momentarily.

> [!WARNING]
> Setting `max_depth` to 0 makes it impossible to parse any document containing elements. This is intentional—it's a valid (if extreme) security configuration.

---

## Parse Entry Point

The top-level parse function signature:

```rust
pub fn parse(input: &str) -> Result<Document>
pub fn parse_with_options(input: &str, options: ParseOptions) -> Result<Document>
```

**Algorithm:**

1. If `input` is empty or contains only whitespace → `Err(XmlError::EmptyDocument)`.
2. Skip optional UTF-8 BOM (byte order mark: `\xEF\xBB\xBF`).
3. Initialize parser state: position = 0, line = 1, depth = 0.
4. Parse according to the `document` production rule.
5. After parsing the root element, skip any trailing whitespace and comments.
6. If unparsed non-whitespace content remains → `Err(XmlError::Parse { kind: General })`.
7. Return `Ok(Document)` containing the populated arena.
