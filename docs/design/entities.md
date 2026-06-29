# Entity Design

## Overview

XML entities are escape sequences that represent characters which would otherwise be interpreted as markup. tinyxml2-rs provides complete entity handling compatible with TinyXML2, supporting the five predefined XML entities, decimal numeric character references (`&#N;`), and hexadecimal numeric character references (`&#xN;`).

Entity processing is split into two complementary operations:

- **Decoding** (input): Converting entity references (`&amp;`) back to characters (`&`) during parsing.
- **Encoding** (output): Converting special characters (`&`) to entity references (`&amp;`) during serialization.

Both operations are designed for zero-allocation fast paths when no transformation is needed.

---

## Predefined XML Entities

The five XML predefined entities, as defined in the XML 1.0 specification:

| Entity | Character | Code Point | Description |
|---|---|---|---|
| `&amp;` | `&` | U+0026 | Ampersand |
| `&lt;` | `<` | U+003C | Less-than sign |
| `&gt;` | `>` | U+003E | Greater-than sign |
| `&quot;` | `"` | U+0022 | Double quotation mark |
| `&apos;` | `'` | U+0027 | Apostrophe |

These are the only named entities supported. Unlike HTML, XML does not define entities like `&nbsp;` or `&copy;`. Unrecognized named entities are left unchanged in the output.

---

## Numeric Character References

### Decimal References: `&#N;`

A decimal numeric character reference consists of `&#` followed by one or more ASCII decimal digits followed by `;`. The digit sequence is interpreted as a decimal number representing a Unicode code point.

**Examples:**
- `&#65;` → `A` (U+0041)
- `&#8364;` → `€` (U+20AC)
- `&#128512;` → `😀` (U+1F600)

### Hexadecimal References: `&#xN;`

A hexadecimal numeric character reference consists of `&#x` (or `&#X`—the `x`/`X` is **case-insensitive**) followed by one or more hexadecimal digits followed by `;`.

**Examples:**
- `&#x41;` → `A` (U+0041)
- `&#x20AC;` → `€` (U+20AC)
- `&#X1f600;` → `😀` (U+1F600, note uppercase X)

### Invalid Reference Handling

Invalid numeric references are **left unchanged** in the output. This matches TinyXML2's behavior exactly. A reference is invalid if:

| Invalid Input | Reason | Output |
|---|---|---|
| `&#0;` | U+0000 (null) is not a valid XML character | `&#0;` (unchanged) |
| `&#x110000;` | Value exceeds Unicode maximum (U+10FFFF) | `&#x110000;` (unchanged) |
| `&#;` | Empty digit sequence | `&#;` (unchanged) |
| `&#xG;` | Invalid hex digit | `&#xG;` (unchanged) |
| `&#99999999;` | Overflows `u32` or exceeds U+10FFFF | `&#99999999;` (unchanged) |

> [!NOTE]
> The decision to leave invalid references unchanged (rather than returning an error) is deliberate. This provides maximum compatibility with real-world XML documents that may contain technically invalid references, and it matches TinyXML2's behavior precisely.

---

## Public API

### `decode(input: &str) -> String`

Decodes all entity references and numeric character references in the input string. **Always allocates** a new `String`, even if no entities are present.

```rust
use tinyxml2::entity::decode;

assert_eq!(decode("no entities"), "no entities");
assert_eq!(decode("&amp; &lt; &gt;"), "& < >");
assert_eq!(decode("&#65;&#x42;"), "AB");
assert_eq!(decode("&#0;"), "&#0;"); // invalid, unchanged
```

**When to use:** When you need a guaranteed `String` and don't care about the allocation in the no-entity case.

### `decode_cow(input: &str) -> Cow<'_, str>`

Zero-allocation fast path: if the input contains no `&` character, returns `Cow::Borrowed(input)` without any allocation. Otherwise, decodes all entities and returns `Cow::Owned(String)`.

```rust
use tinyxml2::entity::decode_cow;
use std::borrow::Cow;

// Fast path: no & found, zero allocation
let result = decode_cow("hello world");
assert!(matches!(result, Cow::Borrowed(_)));

// Slow path: entities found, allocates
let result = decode_cow("hello &amp; world");
assert!(matches!(result, Cow::Owned(_)));
assert_eq!(result, "hello & world");
```

**When to use:** During parsing, where most text and attribute values contain no entities. The `Cow` return type allows the parser to avoid allocations for the common case.

### `encode_text(input: &str) -> Cow<'_, str>`

Encodes characters that are special in **element text content**. Returns `Cow::Borrowed` when no special characters are present.

**Characters escaped:**

| Character | Replacement |
|---|---|
| `&` | `&amp;` |
| `<` | `&lt;` |
| `>` | `&gt;` |

```rust
use tinyxml2::entity::encode_text;

// No escaping needed
assert_eq!(encode_text("hello"), "hello");

// Escaping applied
assert_eq!(encode_text("a < b & c > d"), "a &lt; b &amp; c &gt; d");

// Quotes are NOT escaped in text content
assert_eq!(encode_text(r#"say "hello""#), r#"say "hello""#);
```

### `encode_attribute(input: &str) -> Cow<'_, str>`

Encodes characters that are special in **attribute values**. Returns `Cow::Borrowed` when no special characters are present.

**Characters escaped:**

| Character | Replacement |
|---|---|
| `&` | `&amp;` |
| `<` | `&lt;` |
| `>` | `&gt;` |
| `"` | `&quot;` |
| `'` | `&apos;` |

```rust
use tinyxml2::entity::encode_attribute;

// No escaping needed
assert_eq!(encode_attribute("hello"), "hello");

// Quotes must be escaped in attribute values
assert_eq!(encode_attribute(r#"say "hello""#), "say &quot;hello&quot;");
assert_eq!(encode_attribute("it's"), "it&apos;s");
```

---

## Why Different Escaping Rules

Text content and attribute values have different escaping requirements due to their syntactic context in XML:

**Text content** appears between tags:
```xml
<element>text content here</element>
```
- `&` must be escaped (otherwise starts an entity reference)
- `<` must be escaped (otherwise starts a new tag)
- `>` must be escaped (for symmetry and to avoid `]]>` ambiguity in text)
- `"` and `'` are **safe** — they have no special meaning between tags

**Attribute values** appear inside quotes:
```xml
<element attr="attribute value here"/>
```
- `&` must be escaped (otherwise starts an entity reference)
- `<` must be escaped (XML spec requirement for attribute values)
- `>` must be escaped (for consistency)
- `"` **must** be escaped (otherwise terminates a double-quoted attribute value)
- `'` **must** be escaped (otherwise terminates a single-quoted attribute value)

> [!IMPORTANT]
> We escape both `"` and `'` in attribute values regardless of which quote character is actually used as the delimiter. This ensures that encoded attribute values are safe to embed in either single-quoted or double-quoted attributes.

---

## Internal: `try_decode_entity`

```rust
fn try_decode_entity(bytes: &[u8]) -> Option<(char, usize)>
```

This is an internal (non-public) function that performs byte-level parsing of a single entity reference starting at an `&` byte. It is the core decoding primitive used by both `decode()` and `decode_cow()`.

**Parameters:**
- `bytes`: A byte slice starting at the `&` character of the entity reference.

**Returns:**
- `Some((char, usize))`: The decoded character and the total number of bytes consumed (including the `&` and `;`).
- `None`: The byte sequence is not a valid entity reference. The caller should emit the `&` literally and advance by one byte.

**Parsing logic:**

1. Check if `bytes[0] == b'&'`. If not, return `None`.
2. Check `bytes[1]`:
   - If `b'#'`: Numeric reference.
     - If `bytes[2]` is `b'x'` or `b'X'`: Hex reference. Scan hex digits until `;`.
     - Otherwise: Decimal reference. Scan decimal digits until `;`.
     - Parse the digit sequence to `u32`. Attempt `char::from_u32()`. Return `None` if invalid.
   - Otherwise: Named reference. Match against the five predefined entities (`amp`, `lt`, `gt`, `quot`, `apos`).
3. Verify the reference ends with `;`. If not, return `None`.

---

## Round-Trip Correctness

Encoding and then decoding returns the original string. This invariant is verified by tests:

```rust
// For any string s:
assert_eq!(decode(&encode_text(s)), s);
assert_eq!(decode(&encode_attribute(s)), s);
```

The reverse is **not** always true: decoding and then encoding may produce a different (but semantically equivalent) representation. For example, `&#65;` decodes to `A`, and encoding `A` returns `A` (not `&#65;`), because `A` is not a special character.

---

## processEntities Flag

The `ParseOptions::process_entities` flag (default: `true`) controls whether entity decoding occurs during parsing:

| `process_entities` | Behavior |
|---|---|
| `true` (default) | Entity references in text content and attribute values are decoded. `&amp;` → `&`. |
| `false` | Entity references are preserved as raw text. `&amp;` → `&amp;`. |

When `process_entities` is `false`:
- `decode_cow()` is not called on text content or attribute values.
- The raw entity text is stored in the DOM exactly as it appeared in the source.
- Serialization with `encode_text()` / `encode_attribute()` would then double-encode entities (`&amp;` → `&amp;amp;`), so the serializer must also skip encoding when entities were not processed.

This flag exists for compatibility with TinyXML2's `processEntities` setting and for use cases where the raw XML text needs to be preserved (e.g., XML editors).

---

## Performance Considerations

### Zero-Allocation Fast Paths

Both `decode_cow()` and the `encode_*()` functions use `Cow<'_, str>` to avoid allocation when no transformation is needed:

```
decode_cow():
  1. Scan bytes for '&'
  2. If no '&' found → return Cow::Borrowed(input)  // zero alloc
  3. If '&' found → allocate String, decode entities  // one alloc

encode_text() / encode_attribute():
  1. Scan bytes for special characters
  2. If none found → return Cow::Borrowed(input)      // zero alloc
  3. If found → allocate String, encode characters     // one alloc
```

In typical XML documents, the vast majority of text and attribute values contain no special characters, so the fast path (zero allocation) is the common case.

### Byte-Level Scanning

Entity scanning operates at the byte level (not character level) because:
- `&`, `<`, `>`, `"`, `'` are all single-byte ASCII characters.
- UTF-8 guarantees that these byte values never appear as part of a multi-byte sequence.
- Byte-level scanning avoids the overhead of UTF-8 character boundary checking.

### Pre-Sized Allocation

When the slow path triggers, the output `String` is pre-allocated with `String::with_capacity(input.len())` because the encoded form is always ≥ the decoded form in length (for encoding) and ≤ for decoding. This avoids repeated reallocations.

---

## Compatibility with TinyXML2

tinyxml2-rs matches TinyXML2's entity handling exactly:

| Feature | TinyXML2 | tinyxml2-rs |
|---|---|---|
| Predefined entities | `&amp;` `&lt;` `&gt;` `&quot;` `&apos;` | Same 5 entities |
| Decimal references | `&#N;` | `&#N;` |
| Hex references | `&#xN;` (case-insensitive x) | `&#xN;` (case-insensitive x) |
| Invalid references | Left unchanged | Left unchanged |
| HTML entities | Not supported | Not supported |
| `processEntities` flag | Configurable | Configurable via `ParseOptions` |
| Text encoding | `&` `<` `>` | `&` `<` `>` |
| Attribute encoding | `&` `<` `>` `"` `'` | `&` `<` `>` `"` `'` |
