# Behavioral Specification

> **Purpose**: This document catalogs TinyXML2's observable behaviors that tinyxml2-rs
> MUST faithfully reproduce. Each behavior is derived from the TinyXML2 C++ source code
> and test suite. Deviations from this specification are bugs.

---

## Table of Contents

1. [Entity Handling](#1-entity-handling)
2. [Whitespace Modes](#2-whitespace-modes)
3. [BOM Handling](#3-bom-handling)
4. [Error Behavior](#4-error-behavior)
5. [Element Depth Limits](#5-element-depth-limits)
6. [Self-Closing Tags](#6-self-closing-tags)
7. [Attribute Handling](#7-attribute-handling)
8. [CDATA Sections](#8-cdata-sections)
9. [Comments](#9-comments)
10. [Declarations](#10-declarations)
11. [Unknown Nodes](#11-unknown-nodes)
12. [Memory Ownership Semantics](#12-memory-ownership-semantics)
13. [Printing / Serialization](#13-printing--serialization)

---

## 1. Entity Handling

### 1.1 Predefined Entity Decoding

When `processEntities = true` (the default), the following 5 predefined XML entities
are decoded during parsing:

| Entity   | Decoded Character | Unicode |
|----------|-------------------|---------|
| `&amp;`  | `&`               | U+0026  |
| `&lt;`   | `<`               | U+003C  |
| `&gt;`   | `>`               | U+003E  |
| `&quot;` | `"`               | U+0022  |
| `&apos;` | `'`               | U+0027  |

### 1.2 Numeric Character References

Numeric character references are decoded regardless of the `processEntities` flag:

| Format   | Example     | Decoded |
|----------|-------------|---------|
| `&#N;`   | `&#65;`     | `A`     |
| `&#xN;`  | `&#x41;`    | `A`     |
| `&#xN;`  | `&#x1F600;` | `😀`    |

**Rules:**
- Decimal (`&#N;`) and hexadecimal (`&#xN;` / `&#XN;`) forms are both supported
- The decoded value must be a valid Unicode scalar value
- Invalid numeric references (e.g., `&#xFFFFFFFF;`, `&#0;`) are left as literal text
- Leading zeros are permitted: `&#065;` → `A`

### 1.3 Invalid / Unknown Entity References

Entity references that are NOT one of the 5 predefined entities are left as literal
text in the output:

```
Input:  &foo;
Output: &foo;     (literal text, not decoded)
```

This applies to:
- Unknown named entities: `&nbsp;`, `&copy;`, etc.
- Malformed entities: `&amp` (missing semicolon), `&#;` (empty), `&#x;` (empty hex)

### 1.4 Entity Processing Disabled

When `processEntities = false`:
- The 5 predefined entities are NOT decoded
- `&amp;` remains `&amp;` in the DOM text value
- Numeric character references are still decoded (this matches TinyXML2 behavior)

### 1.5 Entity Encoding on Output

When serializing (printing), certain characters are encoded to entity references:

**In text content:**

| Character | Encoded As |
|-----------|------------|
| `&`       | `&amp;`    |
| `<`       | `&lt;`     |
| `>`       | `&gt;`     |

**In attribute values:**

| Character | Encoded As |
|-----------|------------|
| `&`       | `&amp;`    |
| `<`       | `&lt;`     |
| `>`       | `&gt;`     |
| `"`       | `&quot;`   |
| `'`       | `&apos;`   |

> **Note:** The additional `"` and `'` encoding in attributes is required because
> attribute values are enclosed in quotes.

---

## 2. Whitespace Modes

TinyXML2 provides three whitespace handling modes that control how whitespace in text
content is processed during parsing.

### 2.1 PRESERVE Mode

**Behavior:** All whitespace characters are kept exactly as they appear in the source.

- No stripping, collapsing, or normalization
- Tabs, spaces, newlines, carriage returns are all preserved
- This is the most literal mode

```
Input:  <e>  hello   world  \n</e>
Value:  "  hello   world  \n"
```

### 2.2 COLLAPSE Mode

**Behavior:** Whitespace is aggressively normalized.

Rules (applied in order):
1. All whitespace characters (`\t`, `\n`, `\r`, ` `) are converted to space (U+0020)
2. Leading whitespace is stripped
3. Trailing whitespace is stripped
4. Internal runs of whitespace are collapsed to a single space

```
Input:  <e>  hello \t  world  \n</e>
Value:  "hello world"
```

### 2.3 PEDANTIC Mode

**Behavior:** Like PRESERVE, but with line-ending normalization.

Rules:
1. `\r\n` sequences are normalized to `\n`
2. Standalone `\r` is normalized to `\n`
3. All other whitespace is preserved as-is

```
Input:  <e>hello\r\nworld\rfoo</e>
Value:  "hello\nworld\nfoo"
```

### 2.4 Whitespace Mode Summary

| Behavior          | PRESERVE | COLLAPSE | PEDANTIC |
|-------------------|----------|----------|----------|
| Strip leading     | No       | Yes      | No       |
| Strip trailing    | No       | Yes      | No       |
| Collapse runs     | No       | Yes      | No       |
| `\r\n` → `\n`    | No       | Yes*     | Yes      |
| `\r` → `\n`      | No       | Yes*     | Yes      |
| `\t` → ` `       | No       | Yes      | No       |

*In COLLAPSE mode, `\r\n` and `\r` are first converted to spaces, then collapsed.

---

## 3. BOM Handling

### 3.1 BOM Detection

- The UTF-8 Byte Order Mark (BOM) is the 3-byte sequence `EF BB BF`
- On parsing, if the input begins with a UTF-8 BOM, it is consumed (removed from the
  parse stream)
- The `HasBOM()` method returns `true` if a BOM was detected during parsing

### 3.2 BOM on Output

- When `SetBOM(true)` is called, the BOM is prepended to serialized output
- When the original parsed document had a BOM, the BOM is preserved on output (unless
  explicitly disabled)
- Default: no BOM on output for programmatically-created documents

### 3.3 Encoding

- **Only UTF-8 is supported**
- No encoding detection beyond BOM presence
- No encoding conversion
- Non-UTF-8 input produces undefined behavior (in C++); in tinyxml2-rs, it should
  produce an error

---

## 4. Error Behavior

### 4.1 Error Precedence

- The **first error** encountered during parsing stops all further parsing
- No error recovery is attempted
- The document may be partially constructed up to the point of the error
- Subsequent parse calls on the same document replace the previous state

### 4.2 Error State

- `Error()` returns `true` if an error occurred
- `ErrorID()` returns the specific `XmlError` variant
- `ErrorStr()` returns a human-readable error message
- `ErrorLineNum()` returns the line number where the error was detected (1-based)
- After an error, the document should not be used for further DOM operations

### 4.3 Error Conditions

| Condition | Error |
|-----------|-------|
| Empty input | `EmptyDocument` |
| Missing closing tag | `MismatchedElement` |
| Mismatched tag names | `MismatchedElement` |
| Malformed element syntax | `ParsingElement` |
| Malformed attribute syntax | `ParsingAttribute` |
| Malformed CDATA syntax | `ParsingCData` |
| Malformed comment syntax | `ParsingComment` |
| Malformed declaration syntax | `ParsingDeclaration` |
| Depth limit exceeded | `ElementDepthExceeded` |
| Duplicate attribute names | `ParsingAttribute` |

---

## 5. Element Depth Limits

### 5.1 Configuration

- Maximum element nesting depth is configurable
- Default maximum depth: **500**
- Set via `SetMaxElementDepth(depth)` (0 = unlimited? or minimum 1 — verify against C++)

### 5.2 Behavior

- The depth counter increments when an opening tag is encountered
- If the depth exceeds the maximum, parsing stops with `ElementDepthExceeded`
- The depth counter decrements when a closing tag is processed
- The root element is at depth 1

### 5.3 Purpose

- Prevents stack overflow from deeply nested XML
- Defends against XML "billion laughs" style attacks with deep nesting
- Provides a safety limit for recursive descent parsing

---

## 6. Self-Closing Tags

### 6.1 Syntax

```xml
<element/>
<element />
<element attr="value"/>
```

### 6.2 Behavior

- Self-closing tags create an element with **no children**
- The element's closing type is `CLOSED` (vs. `OPEN` for `<e>...</e>`)
- On serialization, self-closing elements are printed as `<element/>`
- Self-closing elements with attributes: `<element attr="value"/>`
- Whitespace before `/>` is permitted: `<element />` → `<element/>`

---

## 7. Attribute Handling

### 7.1 Quoting

- Both single quotes (`'`) and double quotes (`"`) are valid delimiters
- Opening and closing quotes **must match**: `attr="value"` or `attr='value'`
- Mismatched quotes produce a parsing error

### 7.2 Entity Processing in Attributes

- Entity processing applies to attribute values (when enabled)
- All 5 predefined entities are decoded in attribute values
- Numeric character references are decoded in attribute values

### 7.3 Attribute Order

- Attributes are stored in insertion order
- On serialization, attributes are printed in the order they were parsed or added

### 7.4 Duplicate Attributes

- Duplicate attribute names on the same element produce a parsing error
- This matches XML 1.0's well-formedness constraint

---

## 8. CDATA Sections

### 8.1 Syntax

```xml
<![CDATA[ content goes here ]]>
```

### 8.2 Behavior

- Content between `<![CDATA[` and `]]>` is treated as literal text
- **No entity processing** occurs within CDATA sections
- The `CData()` method returns `true` for CDATA text nodes
- CDATA sections are a type of text node in the DOM
- On serialization, CDATA content is wrapped in `<![CDATA[` ... `]]>`

### 8.3 Edge Cases

- Empty CDATA: `<![CDATA[]]>` → empty text node with `CData() = true`
- CDATA cannot contain the sequence `]]>` (this ends the section)
- Nested CDATA is not possible

---

## 9. Comments

### 9.1 Syntax

```xml
<!-- This is a comment -->
```

### 9.2 Behavior

- Comments are delimited by `<!--` and `-->`
- **No entity processing** occurs within comments
- The value/text of a comment is everything between `<!--` and `-->`
- Comments are represented as comment nodes in the DOM
- Comments can appear at any level: before/after root element, within elements

### 9.3 Edge Cases

- Empty comment: `<!---->` → comment with empty value
- The sequence `--` is technically not allowed within comments per XML 1.0, but
  TinyXML2's behavior for this case should be matched (verify: does it error or accept?)
- Comments cannot be nested

---

## 10. Declarations

### 10.1 Syntax

```xml
<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
```

### 10.2 Behavior

- Declarations are delimited by `<?xml` and `?>`
- Typically contain `version`, `encoding`, and `standalone` attributes
- The declaration is stored as a declaration node in the DOM
- Attributes on declarations are parsed and accessible
- Usually appears as the first node in the document (before the root element)

### 10.3 Edge Cases

- Multiple declarations: TinyXML2 behavior should be matched
- Processing instructions (`<?target data?>`) — verify how TinyXML2 handles these

---

## 11. Unknown Nodes

### 11.1 Syntax

```xml
<!DOCTYPE html>
<!ENTITY name "value">
```

### 11.2 Behavior

- Any construct starting with `<!` that is not a comment (`<!--`) or CDATA
  (`<![CDATA[`) is parsed as an Unknown node
- The content between `<!` and `>` is stored as the node's value
- No further parsing or interpretation of the content is performed
- This is how TinyXML2 handles DTDs, entity declarations, and other constructs it
  doesn't fully support

---

## 12. Memory Ownership Semantics

### 12.1 Document Owns All Nodes

- The `Document` is the owner and factory for all nodes
- Nodes are created via factory methods on the Document:
  - `new_element(name)` → `NodeId`
  - `new_text(content)` → `NodeId`
  - `new_comment(content)` → `NodeId`
  - `new_declaration()` → `NodeId`
  - `new_unknown(content)` → `NodeId`

### 12.2 Node Insertion

- Nodes must be inserted into the tree via Document methods:
  - `insert_first_child(parent, child)`
  - `insert_end_child(parent, child)`
  - `insert_after_child(after, child)`
- A node can only be inserted into the Document that created it
- Inserting an already-inserted node should detach it first (or error — verify C++
  behavior)

### 12.3 Unattached Nodes

- Nodes that are created but never inserted into the tree are "unattached"
- Unattached nodes are still owned by the Document
- Unattached nodes are destroyed when the Document is dropped
- In tinyxml2-rs, unattached nodes remain in the arena until the Document is dropped

### 12.4 Cross-Document Operations

- Node IDs from one Document are invalid in another Document
- To copy nodes between documents, use `DeepClone`:
  - Creates a deep copy of a subtree in the target document
  - All children, attributes, and text are recursively cloned
  - The cloned nodes get new NodeIds in the target document

### 12.5 Node Removal

- Removing a node from the tree detaches it and its entire subtree
- The removed node's arena slot is returned to the free list
- Any NodeIds pointing to removed nodes become stale (generation mismatch)
- Children of removed nodes are also removed recursively

---

## 13. Printing / Serialization

### 13.1 Dual-Mode Printer

TinyXML2 provides two printing modes:

**Formatted (Pretty-Print):**
- Indentation with configurable depth (default: 4 spaces per level)
- Newlines between elements
- Human-readable output

**Compact:**
- No indentation
- No extra whitespace
- Minimal output size

### 13.2 Serialization Rules

- Elements with children: `<name>...</name>`
- Self-closing elements (no children): `<name/>`
- Attributes: `name="value"` (always double-quoted)
- Text content: entity-encoded as per §1.5
- CDATA: `<![CDATA[content]]>`
- Comments: `<!--content-->`
- Declarations: `<?xml attributes?>`
- Unknown: `<!content>`

### 13.3 Output Encoding

- Output is always UTF-8
- BOM is prepended if `SetBOM(true)` or original had BOM (see §3.2)
- Entity encoding as specified in §1.5
