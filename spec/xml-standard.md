# XML Standard Reference

> **Purpose**: This document maps TinyXML2's feature support against the XML 1.0
> specification (Fifth Edition, W3C Recommendation). It clarifies which XML features
> are supported, partially supported, or unsupported.

> [!IMPORTANT]
> **TinyXML2 is NOT a full XML 1.0 parser.** It is a lightweight parser that supports
> the most commonly used subset of XML. This document explicitly catalogs what is and
> is not supported so that users have clear expectations.

---

## Table of Contents

1. [Supported Features](#1-supported-features)
2. [Unsupported Features](#2-unsupported-features)
3. [XML 1.0 Spec Section Mapping](#3-xml-10-spec-section-mapping)
4. [Character Encoding](#4-character-encoding)
5. [Well-Formedness](#5-well-formedness)

---

## 1. Supported Features

### 1.1 Elements

Full support for XML elements in both forms:

```xml
<!-- Start and end tags -->
<element>content</element>

<!-- Self-closing (empty element) tags -->
<element/>
<element />
```

- Element names follow XML naming rules (letters, digits, hyphens, underscores, periods,
  colons)
- Colons are allowed in element names but are treated as literal characters — no
  namespace resolution
- Elements can be nested to arbitrary depth (configurable limit, default 500)

### 1.2 Attributes

Full support for element attributes:

```xml
<element name="value"/>
<element name='value'/>
<element a="1" b="2" c='3'/>
```

- Both single-quote (`'`) and double-quote (`"`) delimiters supported
- Opening and closing quotes must match
- Multiple attributes per element supported
- Duplicate attribute names on the same element produce an error
- Attribute values undergo entity processing (when enabled)
- Attribute names follow XML naming rules

### 1.3 Text Content

Character data between element tags:

```xml
<element>This is text content</element>
<element>Text with &amp; entities &lt;decoded&gt;</element>
```

- Text is subject to entity processing (when enabled)
- Text is subject to whitespace handling (per configured mode)
- Adjacent text nodes may be merged (verify TinyXML2 behavior)

### 1.4 Comments

XML comments in standard syntax:

```xml
<!-- This is a comment -->
<!---->
<!-- Multi-line
     comment -->
```

- Content between `<!--` and `-->` is stored verbatim
- No entity processing within comments
- Comments can appear at any level in the document

### 1.5 CDATA Sections

CDATA sections for literal text content:

```xml
<![CDATA[This is <literal> text & not processed]]>
```

- Content between `<![CDATA[` and `]]>` is stored verbatim
- No entity processing within CDATA sections
- CDATA nodes are a type of text node with a CDATA flag

### 1.6 XML Declarations

XML declaration (prolog):

```xml
<?xml version="1.0"?>
<?xml version="1.0" encoding="UTF-8"?>
<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
```

- Parsed as a declaration node
- Attributes (`version`, `encoding`, `standalone`) are accessible
- Typically the first node in the document

### 1.7 Predefined Entities

The 5 XML predefined entities are fully supported:

| Entity Reference | Character | Unicode Code Point |
|------------------|-----------|-------------------|
| `&amp;`          | `&`       | U+0026            |
| `&lt;`           | `<`       | U+003C            |
| `&gt;`           | `>`       | U+003E            |
| `&quot;`         | `"`       | U+0022            |
| `&apos;`         | `'`       | U+0027            |

- Decoded during parsing (when `processEntities = true`)
- Encoded during serialization as appropriate for context (text vs. attribute)

### 1.8 Numeric Character References

Both decimal and hexadecimal forms:

| Form | Example | Result |
|------|---------|--------|
| Decimal | `&#65;` | `A` |
| Hexadecimal | `&#x41;` | `A` |
| Hexadecimal (uppercase) | `&#X41;` | `A` |
| High Unicode | `&#x1F600;` | `😀` |

- Must decode to a valid Unicode scalar value
- Invalid values are left as literal text

### 1.9 UTF-8 Encoding

- UTF-8 is the only supported encoding
- Optional UTF-8 BOM (`EF BB BF`) is detected and consumed
- All string content is valid UTF-8

### 1.10 Element Nesting

- Elements can contain other elements, text, comments, CDATA, and other nodes
- Proper nesting is enforced: `<a><b></b></a>` ✓, `<a><b></a></b>` ✗
- Nesting depth is configurable (default limit: 500)

### 1.11 Multiple Attributes

- Elements can have any number of attributes
- Attributes are stored in document order
- Each attribute name must be unique within its element

---

## 2. Unsupported Features

### 2.1 Document Type Definitions (DTD)

```xml
<!-- NOT SUPPORTED — parsed as Unknown node -->
<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.0//EN" "...">
<!DOCTYPE root [
  <!ELEMENT root (child)>
  <!ATTLIST root attr CDATA #REQUIRED>
]>
```

- `<!DOCTYPE ...>` is parsed as an Unknown node
- The content is stored but not interpreted
- No element/attribute validation against DTD rules
- Internal DTD subsets are not processed

### 2.2 External Entities

```xml
<!-- NOT SUPPORTED -->
<!ENTITY chapter SYSTEM "chapter1.xml">
<!ENTITY logo SYSTEM "http://example.com/logo.png" NDATA png>
```

- No file or URL resolution for external entities
- No external entity expansion
- External entity declarations in DTDs are stored as Unknown nodes

### 2.3 Internal Entity Definitions

```xml
<!-- NOT SUPPORTED -->
<!DOCTYPE doc [
  <!ENTITY myEntity "replacement text">
]>
<doc>&myEntity;</doc>
```

- Only the 5 predefined entities are recognized
- User-defined entities via `<!ENTITY ...>` are not processed
- References to undefined entities are left as literal text

### 2.4 Namespaces

```xml
<!-- Colons allowed but NOT interpreted as namespaces -->
<ns:element xmlns:ns="http://example.com/ns">
  <ns:child ns:attr="value"/>
</ns:element>
```

- Colons in element and attribute names are treated as literal characters
- No namespace URI resolution
- No namespace-aware element/attribute lookup
- `xmlns` attributes are parsed like any other attribute
- No prefix-to-namespace mapping

### 2.5 Processing Instructions

```xml
<!-- Partial support — parsed but treated as declaration/unknown -->
<?xml-stylesheet type="text/xsl" href="style.xsl"?>
<?php echo "Hello"; ?>
```

- The XML declaration (`<?xml ...?>`) is handled specially
- Other processing instructions may be parsed as declaration or unknown nodes
- No processing instruction handler or callback mechanism

### 2.6 XML Schema Validation

- No support for XML Schema (XSD) validation
- No schema-aware parsing
- No type checking against schema definitions

### 2.7 XPath

- No XPath query support
- Node lookup is via DOM traversal only

### 2.8 XSLT

- No XSLT transformation support
- No stylesheet processing

### 2.9 XInclude

```xml
<!-- NOT SUPPORTED -->
<xi:include href="other.xml" xmlns:xi="http://www.w3.org/2001/XInclude"/>
```

- No XInclude processing
- `xi:include` elements are parsed as regular elements

### 2.10 `xml:space` Attribute Processing

```xml
<!-- NOT SUPPORTED as a special attribute -->
<element xml:space="preserve">  content  </element>
```

- `xml:space` is parsed as a regular attribute
- Its value does not affect whitespace handling
- Whitespace mode is a document-level configuration, not per-element

### 2.11 Conditional Sections

```xml
<!-- NOT SUPPORTED -->
<![INCLUDE[ ... ]]>
<![IGNORE[ ... ]]>
```

- Conditional sections (used within DTDs) are not supported
- May be misinterpreted as CDATA if starting with `<![`

### 2.12 Notation Declarations

```xml
<!-- NOT SUPPORTED -->
<!NOTATION png SYSTEM "image/png">
```

- Notation declarations in DTDs are not processed
- Stored as Unknown nodes if encountered

### 2.13 Parameter Entities

```xml
<!-- NOT SUPPORTED -->
<!ENTITY % common "id ID #REQUIRED">
<!ATTLIST element %common;>
```

- Parameter entities (used within DTDs) are not supported
- DTD content is stored as-is in Unknown nodes

---

## 3. XML 1.0 Spec Section Mapping

The following table maps sections of the
[XML 1.0 Specification (Fifth Edition)](https://www.w3.org/TR/xml/) to TinyXML2's
support level.

| Spec Section | Title | Support Level | Notes |
|--------------|-------|---------------|-------|
| §2.1 | Well-Formed XML Documents | ⚠️ Partial | Basic well-formedness checks |
| §2.2 | Characters | ✅ Supported | UTF-8 characters |
| §2.3 | Common Syntactic Constructs | ✅ Supported | Names, tokens, whitespace |
| §2.4 | Character Data and Markup | ✅ Supported | Text content, entity refs |
| §2.5 | Comments | ✅ Supported | `<!-- ... -->` |
| §2.6 | Processing Instructions | ⚠️ Partial | XML decl only; others as unknown |
| §2.7 | CDATA Sections | ✅ Supported | `<![CDATA[ ... ]]>` |
| §2.8 | Prolog and DTD | ⚠️ Partial | XML decl supported; DTD as unknown |
| §2.9 | Standalone Document Declaration | ⚠️ Partial | Parsed as attribute, not enforced |
| §2.10 | White Space Handling | ⚠️ Partial | Custom modes, not `xml:space` |
| §2.11 | End-of-Line Handling | ✅ Supported | Via PEDANTIC whitespace mode |
| §2.12 | Language Identification | ❌ Not supported | `xml:lang` parsed as regular attr |
| §3.1 | Start-Tags, End-Tags, Empty-Element Tags | ✅ Supported | Full support |
| §3.2 | Element Type Declarations | ❌ Not supported | DTD feature |
| §3.3 | Attribute-List Declarations | ❌ Not supported | DTD feature |
| §3.3.1 | Attribute Types | ❌ Not supported | DTD feature |
| §3.3.2 | Attribute Defaults | ❌ Not supported | DTD feature |
| §3.3.3 | Attribute-Value Normalization | ⚠️ Partial | Whitespace modes, not XML spec rules |
| §3.4 | Conditional Sections | ❌ Not supported | DTD feature |
| §4.1 | Character and Entity References | ⚠️ Partial | Predefined + numeric only |
| §4.2 | Entity Declarations | ❌ Not supported | DTD feature |
| §4.3 | Parsed Entities | ❌ Not supported | No external entity resolution |
| §4.4 | XML Processor Treatment of Entities | ⚠️ Partial | 5 predefined + numeric refs |
| §4.5 | Construction of Entity Replacement Text | ❌ Not supported | No entity definitions |
| §4.6 | Predefined Entities | ✅ Supported | All 5 predefined entities |
| §4.7 | Notation Declarations | ❌ Not supported | DTD feature |

**Legend:**
- ✅ **Supported**: Feature is fully implemented
- ⚠️ **Partial**: Some aspects supported, others not
- ❌ **Not supported**: Feature is not implemented

---

## 4. Character Encoding

### 4.1 Supported Encoding

| Encoding | Status |
|----------|--------|
| UTF-8 | ✅ Supported (only encoding) |
| UTF-8 with BOM | ✅ Supported (BOM consumed) |
| UTF-16 | ❌ Not supported |
| UTF-32 | ❌ Not supported |
| ISO-8859-1 | ❌ Not supported |
| Other encodings | ❌ Not supported |

### 4.2 Encoding Detection

- If a UTF-8 BOM (`EF BB BF`) is present, it is consumed
- No `encoding` attribute processing — the declared encoding is stored but not used for
  transcoding
- Input is assumed to be valid UTF-8
- Invalid UTF-8 sequences:
  - In C++ TinyXML2: undefined behavior
  - In tinyxml2-rs: returns an error (intentional deviation for safety)

### 4.3 Character Range

Per the XML 1.0 specification, the following Unicode code points are valid in XML
documents:

```
#x9 | #xA | #xD | [#x20-#xD7FF] | [#xE000-#xFFFD] | [#x10000-#x10FFFF]
```

TinyXML2 does NOT validate character ranges — it accepts any UTF-8 byte sequence.
tinyxml2-rs should match this permissive behavior (Tier 2 compatibility).

---

## 5. Well-Formedness

### 5.1 Checks Performed

TinyXML2 performs the following well-formedness checks:

| Check | Description | Error |
|-------|-------------|-------|
| Matching tags | Start and end tag names must match | `MismatchedElement` |
| Proper nesting | Elements must be properly nested | `MismatchedElement` |
| Valid names | Element and attribute names must start with letter/underscore | `ParsingElement` |
| Unique attributes | No duplicate attribute names per element | `ParsingAttribute` |
| Quote matching | Attribute quotes must match (single or double) | `ParsingAttribute` |
| Non-empty document | Document must contain at least one node | `EmptyDocument` |
| Depth limit | Nesting depth must not exceed configured maximum | `ElementDepthExceeded` |

### 5.2 Checks NOT Performed

TinyXML2 does NOT perform these XML 1.0 well-formedness checks:

| Check | XML 1.0 Requirement |
|-------|---------------------|
| Single root element | Document must have exactly one root element |
| Character range validation | Characters must be in allowed Unicode ranges |
| `--` in comments | The string `--` must not occur within comments |
| Name character validation | Full XML 1.0 name character rules |
| Attribute value restrictions | No `<` in attribute values |
| Entity declaration requirement | Referenced entities must be declared (except predefined) |
| Standalone declaration enforcement | Standalone="yes" constraints |

### 5.3 Implications

Because TinyXML2 accepts inputs that are not well-formed per XML 1.0, tinyxml2-rs must
also accept these inputs (Tier 1 compatibility). Some examples:

```xml
<!-- Multiple root elements — accepted by TinyXML2 -->
<a/><b/>

<!-- Characters that XML 1.0 forbids — accepted by TinyXML2 -->
<element>&#x1;</element>

<!-- Loose name validation — may be accepted by TinyXML2 -->
<123element/>
```

The guiding principle is: **match TinyXML2's behavior, not the XML 1.0 specification.**

---

## References

- [XML 1.0 Specification (Fifth Edition)](https://www.w3.org/TR/xml/)
- [Namespaces in XML 1.0 (Third Edition)](https://www.w3.org/TR/xml-names/)
- [TinyXML2 GitHub Repository](https://github.com/leethomason/tinyxml2)
- [TinyXML2 Documentation](http://www.grinninglizard.com/tinyxml2/)
