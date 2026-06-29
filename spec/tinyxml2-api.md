# TinyXML2 C++ API Specification

> Complete reference for the TinyXML2 C++ library API surface.
> Based on TinyXML2 v10.x. This document serves as the canonical specification
> for the Rust binding implementation.

---

## Table of Contents

- [Class Hierarchy](#class-hierarchy)
- [Enumerations](#enumerations)
- [XMLNode (Abstract Base)](#xmlnode-abstract-base)
- [XMLDocument](#xmldocument)
- [XMLElement](#xmlelement)
- [XMLAttribute](#xmlattribute)
- [XMLText](#xmltext)
- [XMLComment](#xmlcomment)
- [XMLDeclaration](#xmldeclaration)
- [XMLUnknown](#xmlunknown)
- [XMLVisitor](#xmlvisitor)
- [XMLPrinter](#xmlprinter)
- [XMLHandle](#xmlhandle)
- [XMLConstHandle](#xmlconsthandle)
- [Memory Ownership Model](#memory-ownership-model)
- [Entity Handling](#entity-handling)
- [Whitespace Modes](#whitespace-modes)
- [BOM Handling](#bom-handling)
- [Thread Safety](#thread-safety)

---

## Class Hierarchy

```
XMLNode (abstract)
├── XMLDocument
├── XMLElement
├── XMLText
├── XMLComment
├── XMLDeclaration
└── XMLUnknown

XMLAttribute (standalone, not an XMLNode)

XMLVisitor (abstract)
└── XMLPrinter

XMLHandle (wrapper, non-owning)
XMLConstHandle (wrapper, non-owning, const)
```

---

## Enumerations

### XMLError

Error codes returned by parsing and query operations.

| Enumerant                            | Value | Description                                      |
|--------------------------------------|-------|--------------------------------------------------|
| `XML_SUCCESS`                        | 0     | No error                                         |
| `XML_NO_ERROR`                       | 0     | Alias for `XML_SUCCESS`                          |
| `XML_NO_ATTRIBUTE`                   | 1     | Attribute not found                              |
| `XML_WRONG_ATTRIBUTE_TYPE`           | 2     | Attribute value could not be converted to type   |
| `XML_ERROR_FILE_NOT_FOUND`           | 3     | File not found                                   |
| `XML_ERROR_FILE_COULD_NOT_BE_OPENED` | 4     | File could not be opened                         |
| `XML_ERROR_FILE_READ_ERROR`          | 5     | Error reading file                               |
| `XML_ERROR_PARSING_ELEMENT`          | 6     | Error parsing element                            |
| `XML_ERROR_PARSING_ATTRIBUTE`        | 7     | Error parsing attribute                          |
| `XML_ERROR_PARSING_TEXT`             | 8     | Error parsing text                               |
| `XML_ERROR_PARSING_CDATA`           | 9     | Error parsing CDATA section                      |
| `XML_ERROR_PARSING_COMMENT`         | 10    | Error parsing comment                            |
| `XML_ERROR_PARSING_DECLARATION`     | 11    | Error parsing declaration                        |
| `XML_ERROR_PARSING_UNKNOWN`         | 12    | Error parsing unknown node type                  |
| `XML_ERROR_EMPTY_DOCUMENT`          | 13    | Document is empty                                |
| `XML_ERROR_MISMATCHED_ELEMENT`      | 14    | Mismatched open/close tags                       |
| `XML_ERROR_PARSING`                 | 15    | General parsing error                            |
| `XML_CAN_NOT_CONVERT_TEXT`          | 16    | Text could not be converted to requested type    |
| `XML_NO_TEXT_NODE`                  | 17    | No text node found                               |
| `XML_ELEMENT_DEPTH_EXCEEDED`        | 18    | Maximum element depth exceeded                   |
| `XML_ERROR_COUNT`                   | 19    | Sentinel value (number of error codes)           |

### Whitespace

Controls how whitespace is handled during parsing.

| Enumerant              | Value | Description                                                         |
|------------------------|-------|---------------------------------------------------------------------|
| `PRESERVE_WHITESPACE`  | 0     | Preserve all whitespace exactly as in the source                    |
| `COLLAPSE_WHITESPACE`  | 1     | Collapse contiguous whitespace to a single space, trim leading/trailing |
| `PEDANTIC_WHITESPACE`  | 2     | Preserve whitespace but normalize newlines                          |

### ElementClosingType

Describes how an element is closed in the XML source.

| Enumerant  | Value | Description                              |
|------------|-------|------------------------------------------|
| `OPEN`     | 0     | Element has a separate closing tag `<e>...</e>` |
| `CLOSED`   | 1     | Self-closing element `<e/>`              |
| `CLOSING`  | 2     | The closing tag itself `</e>` (internal) |

---

## XMLNode (Abstract Base)

`XMLNode` is the abstract base class for all nodes in the DOM tree. It cannot be
instantiated directly. All nodes are owned by their parent `XMLDocument`.

### Public Methods

```cpp
// — Identity & Type —
const XMLDocument*  GetDocument() const;
XMLDocument*        GetDocument();
virtual XMLElement*     ToElement();
virtual XMLText*        ToText();
virtual XMLComment*     ToComment();
virtual XMLDocument*    ToDocument();
virtual XMLDeclaration* ToDeclaration();
virtual XMLUnknown*     ToUnknown();
virtual const XMLElement*     ToElement() const;
virtual const XMLText*        ToText() const;
virtual const XMLComment*     ToComment() const;
virtual const XMLDocument*    ToDocument() const;
virtual const XMLDeclaration* ToDeclaration() const;
virtual const XMLUnknown*     ToUnknown() const;

// — Value —
const char* Value() const;
void        SetValue(const char* val, bool staticMem = false);

// — Line Number —
int GetLineNum() const;

// — Tree Traversal —
XMLNode*       Parent();
const XMLNode* Parent() const;
bool           NoChildren() const;
XMLNode*       FirstChild();
const XMLNode* FirstChild() const;
XMLElement*       FirstChildElement(const char* name = 0);
const XMLElement* FirstChildElement(const char* name = 0) const;
XMLNode*       LastChild();
const XMLNode* LastChild() const;
XMLElement*       LastChildElement(const char* name = 0);
const XMLElement* LastChildElement(const char* name = 0) const;
XMLNode*       PreviousSibling();
const XMLNode* PreviousSibling() const;
XMLElement*       PreviousSiblingElement(const char* name = 0);
const XMLElement* PreviousSiblingElement(const char* name = 0) const;
XMLNode*       NextSibling();
const XMLNode* NextSibling() const;
XMLElement*       NextSiblingElement(const char* name = 0);
const XMLElement* NextSiblingElement(const char* name = 0) const;

// — Tree Modification —
XMLNode* InsertFirstChild(XMLNode* addThis);
XMLNode* InsertEndChild(XMLNode* addThis);
XMLNode* InsertAfterChild(XMLNode* afterThis, XMLNode* addThis);
void     DeleteChild(XMLNode* node);
void     DeleteChildren();

// — Utility —
virtual XMLNode* ShallowClone(XMLDocument* document) const = 0;
virtual bool     ShallowEqual(const XMLNode* compare) const = 0;
virtual bool     Accept(XMLVisitor* visitor) const = 0;

// — Deep Copy —
XMLNode* DeepClone(XMLDocument* target) const;
```

---

## XMLDocument

`XMLDocument` is the root of the DOM tree. It owns all nodes and provides
factory methods for creating them. It is also the entry point for parsing.

**Inherits from:** `XMLNode`

### Public Methods

```cpp
// — Construction / Destruction —
XMLDocument(bool processEntities = true, Whitespace whitespaceMode = PRESERVE_WHITESPACE);
~XMLDocument();

// — Document Identity (overrides) —
virtual XMLDocument*       ToDocument() override;
virtual const XMLDocument* ToDocument() const override;

// — Parsing —
XMLError Parse(const char* xml, size_t nBytes = (size_t)(-1));
XMLError LoadFile(const char* filename);
XMLError LoadFile(FILE* fp);

// — Saving —
XMLError SaveFile(const char* filename, bool compact = false);
XMLError SaveFile(FILE* fp, bool compact = false);

// — State Query —
bool        ProcessEntities() const;
Whitespace  WhitespaceMode() const;
bool        HasBOM() const;
void        SetBOM(bool useBOM);

// — Root Element —
XMLElement* RootElement();
const XMLElement* RootElement() const;

// — Printing —
void Print(XMLPrinter* streamer = 0) const;

// — Visitor —
virtual bool Accept(XMLVisitor* visitor) const override;

// — Factory Methods (document owns the created nodes) —
XMLElement*     NewElement(const char* name);
XMLComment*     NewComment(const char* comment);
XMLText*        NewText(const char* text);
XMLDeclaration* NewDeclaration(const char* text = 0);
XMLUnknown*     NewUnknown(const char* text);

// — Error Handling —
bool        Error() const;
XMLError    ErrorID() const;
const char* ErrorStr() const;
int         ErrorLineNum() const;
void        ClearError();
void        PrintError() const;

// — Error Name Lookup —
static const char* ErrorIDToName(XMLError errorID);

// — Deep Copy —
virtual XMLNode* ShallowClone(XMLDocument* document) const override;
virtual bool     ShallowEqual(const XMLNode* compare) const override;

// — Configuration —
void SetMaxElementDepth(unsigned maxDepth);
unsigned MaxElementDepth() const;

// — Clear —
void Clear();
```

---

## XMLElement

`XMLElement` represents an XML element with a tag name, attributes, and child
nodes.

**Inherits from:** `XMLNode`

### Public Methods

```cpp
// — Identity (overrides) —
virtual XMLElement*       ToElement() override;
virtual const XMLElement* ToElement() const override;

// — Name —
const char* Name() const;          // alias for Value()
void        SetName(const char* str, bool staticMem = false);  // alias for SetValue()

// — Visitor —
virtual bool Accept(XMLVisitor* visitor) const override;

// — Attribute Access —
const char*         Attribute(const char* name, const char* value = 0) const;
int                 IntAttribute(const char* name, int defaultValue = 0) const;
unsigned            UnsignedAttribute(const char* name, unsigned defaultValue = 0) const;
int64_t             Int64Attribute(const char* name, int64_t defaultValue = 0) const;
uint64_t            Unsigned64Attribute(const char* name, uint64_t defaultValue = 0) const;
bool                BoolAttribute(const char* name, bool defaultValue = false) const;
double              DoubleAttribute(const char* name, double defaultValue = 0) const;
float               FloatAttribute(const char* name, float defaultValue = 0) const;

// — Attribute Query (with error code) —
XMLError QueryIntAttribute(const char* name, int* value) const;
XMLError QueryUnsignedAttribute(const char* name, unsigned* value) const;
XMLError QueryInt64Attribute(const char* name, int64_t* value) const;
XMLError QueryUnsigned64Attribute(const char* name, uint64_t* value) const;
XMLError QueryBoolAttribute(const char* name, bool* value) const;
XMLError QueryDoubleAttribute(const char* name, double* value) const;
XMLError QueryFloatAttribute(const char* name, float* value) const;
XMLError QueryStringAttribute(const char* name, const char** value) const;

// — Attribute Modification —
void SetAttribute(const char* name, const char* value);
void SetAttribute(const char* name, int value);
void SetAttribute(const char* name, unsigned value);
void SetAttribute(const char* name, int64_t value);
void SetAttribute(const char* name, uint64_t value);
void SetAttribute(const char* name, bool value);
void SetAttribute(const char* name, double value);
void SetAttribute(const char* name, float value);
void DeleteAttribute(const char* name);

// — Attribute Iteration —
const XMLAttribute* FirstAttribute() const;
const XMLAttribute* FindAttribute(const char* name) const;

// — Text Content Convenience —
const char* GetText() const;
void        SetText(const char* inText);
void        SetText(int value);
void        SetText(unsigned value);
void        SetText(int64_t value);
void        SetText(uint64_t value);
void        SetText(bool value);
void        SetText(double value);
void        SetText(float value);

// — Text Content Query —
XMLError QueryIntText(int* ival) const;
XMLError QueryUnsignedText(unsigned* uval) const;
XMLError QueryInt64Text(int64_t* ival) const;
XMLError QueryUnsigned64Text(uint64_t* uval) const;
XMLError QueryBoolText(bool* bval) const;
XMLError QueryDoubleText(double* dval) const;
XMLError QueryFloatText(float* fval) const;

// — Text Content with Default —
int      IntText(int defaultValue = 0) const;
unsigned UnsignedText(unsigned defaultValue = 0) const;
int64_t  Int64Text(int64_t defaultValue = 0) const;
uint64_t Unsigned64Text(uint64_t defaultValue = 0) const;
bool     BoolText(bool defaultValue = false) const;
double   DoubleText(double defaultValue = 0) const;
float    FloatText(float defaultValue = 0) const;

// — Element Closing Type —
ElementClosingType ClosingType() const;

// — Deep Copy —
virtual XMLNode* ShallowClone(XMLDocument* document) const override;
virtual bool     ShallowEqual(const XMLNode* compare) const override;
```

---

## XMLAttribute

`XMLAttribute` represents a single attribute on an `XMLElement`. Attributes are
stored as a singly-linked list on their parent element. They are **not**
`XMLNode` subclasses.

### Public Methods

```cpp
// — Name & Value —
const char* Name() const;
const char* Value() const;

// — Line Number —
int GetLineNum() const;

// — Iteration —
const XMLAttribute* Next() const;

// — Typed Value Access —
int      IntValue(int defaultValue = 0) const;
unsigned UnsignedValue(unsigned defaultValue = 0) const;
int64_t  Int64Value(int64_t defaultValue = 0) const;
uint64_t Unsigned64Value(uint64_t defaultValue = 0) const;
bool     BoolValue(bool defaultValue = false) const;
double   DoubleValue(double defaultValue = 0) const;
float    FloatValue(float defaultValue = 0) const;

// — Typed Value Query (with error code) —
XMLError QueryIntValue(int* value) const;
XMLError QueryUnsignedValue(unsigned* value) const;
XMLError QueryInt64Value(int64_t* value) const;
XMLError QueryUnsigned64Value(uint64_t* value) const;
XMLError QueryBoolValue(bool* value) const;
XMLError QueryDoubleValue(double* value) const;
XMLError QueryFloatValue(float* value) const;

// — Modification —
void SetAttribute(const char* value);
void SetAttribute(int value);
void SetAttribute(unsigned value);
void SetAttribute(int64_t value);
void SetAttribute(uint64_t value);
void SetAttribute(bool value);
void SetAttribute(double value);
void SetAttribute(float value);
```

---

## XMLText

`XMLText` represents a text node within the DOM. It can optionally be marked as
CDATA.

**Inherits from:** `XMLNode`

### Public Methods

```cpp
// — Identity (overrides) —
virtual XMLText*       ToText() override;
virtual const XMLText* ToText() const override;

// — Visitor —
virtual bool Accept(XMLVisitor* visitor) const override;

// — CDATA —
bool CData() const;
void SetCData(bool isCData);

// — Deep Copy —
virtual XMLNode* ShallowClone(XMLDocument* document) const override;
virtual bool     ShallowEqual(const XMLNode* compare) const override;
```

---

## XMLComment

`XMLComment` represents an XML comment (`<!-- ... -->`).

**Inherits from:** `XMLNode`

### Public Methods

```cpp
// — Identity (overrides) —
virtual XMLComment*       ToComment() override;
virtual const XMLComment* ToComment() const override;

// — Visitor —
virtual bool Accept(XMLVisitor* visitor) const override;

// — Deep Copy —
virtual XMLNode* ShallowClone(XMLDocument* document) const override;
virtual bool     ShallowEqual(const XMLNode* compare) const override;
```

---

## XMLDeclaration

`XMLDeclaration` represents the XML declaration (`<?xml version="1.0"?>`).

**Inherits from:** `XMLNode`

### Public Methods

```cpp
// — Identity (overrides) —
virtual XMLDeclaration*       ToDeclaration() override;
virtual const XMLDeclaration* ToDeclaration() const override;

// — Visitor —
virtual bool Accept(XMLVisitor* visitor) const override;

// — Deep Copy —
virtual XMLNode* ShallowClone(XMLDocument* document) const override;
virtual bool     ShallowEqual(const XMLNode* compare) const override;
```

---

## XMLUnknown

`XMLUnknown` represents any XML construct not otherwise recognized (e.g., DTD
declarations like `<!DOCTYPE ...>`).

**Inherits from:** `XMLNode`

### Public Methods

```cpp
// — Identity (overrides) —
virtual XMLUnknown*       ToUnknown() override;
virtual const XMLUnknown* ToUnknown() const override;

// — Visitor —
virtual bool Accept(XMLVisitor* visitor) const override;

// — Deep Copy —
virtual XMLNode* ShallowClone(XMLDocument* document) const override;
virtual bool     ShallowEqual(const XMLNode* compare) const override;
```

---

## XMLVisitor

`XMLVisitor` is the abstract base class for the visitor pattern. Subclass it to
implement custom document traversal.

### Public Methods

```cpp
virtual ~XMLVisitor();

virtual bool VisitEnter(const XMLDocument& doc);
virtual bool VisitExit(const XMLDocument& doc);

virtual bool VisitEnter(const XMLElement& element, const XMLAttribute* firstAttribute);
virtual bool VisitExit(const XMLElement& element);

virtual bool Visit(const XMLDeclaration& declaration);
virtual bool Visit(const XMLText& text);
virtual bool Visit(const XMLComment& comment);
virtual bool Visit(const XMLUnknown& unknown);
```

---

## XMLPrinter

`XMLPrinter` is a concrete `XMLVisitor` that serializes the DOM tree to a string
or to a `FILE*`. It can also be used standalone to construct XML programmatically.

**Inherits from:** `XMLVisitor`

### Public Methods

```cpp
// — Construction —
XMLPrinter(FILE* file = 0, bool compact = false, int depth = 0);
virtual ~XMLPrinter();

// — Programmatic XML Construction —
void PushHeader(bool writeBOM, bool writeDeclaration);
void OpenElement(const char* name, bool compactMode = false);
void PushAttribute(const char* name, const char* value);
void PushAttribute(const char* name, int value);
void PushAttribute(const char* name, unsigned value);
void PushAttribute(const char* name, int64_t value);
void PushAttribute(const char* name, uint64_t value);
void PushAttribute(const char* name, bool value);
void PushAttribute(const char* name, double value);
void PushComment(const char* comment);
void PushText(const char* text, bool cdata = false);
void PushText(int value);
void PushText(unsigned value);
void PushText(int64_t value);
void PushText(uint64_t value);
void PushText(bool value);
void PushText(float value);
void PushText(double value);
void PushUnknown(const char* value);
void CloseElement(bool compactMode = false);

// — Visitor Overrides —
virtual bool VisitEnter(const XMLDocument& doc) override;
virtual bool VisitExit(const XMLDocument& doc) override;
virtual bool VisitEnter(const XMLElement& element, const XMLAttribute* attribute) override;
virtual bool VisitExit(const XMLElement& element) override;
virtual bool Visit(const XMLText& text) override;
virtual bool Visit(const XMLComment& comment) override;
virtual bool Visit(const XMLDeclaration& declaration) override;
virtual bool Visit(const XMLUnknown& unknown) override;

// — Output Access —
const char* CStr() const;
int         CStrSize() const;
void        ClearBuffer(bool resetToFirstElement = true);
```

---

## XMLHandle

`XMLHandle` is a non-owning wrapper around an `XMLNode*` that provides
null-safe traversal. If any traversal step yields `nullptr`, all subsequent
operations return null handles instead of crashing.

### Public Methods

```cpp
// — Construction —
explicit XMLHandle(XMLNode* node);
explicit XMLHandle(XMLNode& node);
XMLHandle(const XMLHandle& ref);
XMLHandle& operator=(const XMLHandle& ref);

// — Traversal —
XMLHandle FirstChild();
XMLHandle FirstChildElement(const char* name = 0);
XMLHandle LastChild();
XMLHandle LastChildElement(const char* name = 0);
XMLHandle PreviousSibling();
XMLHandle PreviousSiblingElement(const char* name = 0);
XMLHandle NextSibling();
XMLHandle NextSiblingElement(const char* name = 0);

// — Conversion —
XMLNode*        ToNode();
XMLElement*     ToElement();
XMLText*        ToText();
XMLUnknown*     ToUnknown();
```

---

## XMLConstHandle

`XMLConstHandle` is the `const` equivalent of `XMLHandle`, wrapping a
`const XMLNode*`.

### Public Methods

```cpp
// — Construction —
explicit XMLConstHandle(const XMLNode* node);
explicit XMLConstHandle(const XMLNode& node);
XMLConstHandle(const XMLConstHandle& ref);
XMLConstHandle& operator=(const XMLConstHandle& ref);

// — Traversal —
XMLConstHandle FirstChild() const;
XMLConstHandle FirstChildElement(const char* name = 0) const;
XMLConstHandle LastChild() const;
XMLConstHandle LastChildElement(const char* name = 0) const;
XMLConstHandle PreviousSibling() const;
XMLConstHandle PreviousSiblingElement(const char* name = 0) const;
XMLConstHandle NextSibling() const;
XMLConstHandle NextSiblingElement(const char* name = 0) const;

// — Conversion —
const XMLNode*    ToNode() const;
const XMLElement* ToElement() const;
const XMLText*    ToText() const;
const XMLUnknown* ToUnknown() const;
```

---

## Memory Ownership Model

TinyXML2 uses a **document-owns-all** memory model:

1. **`XMLDocument` is the memory pool.** All nodes are allocated from and owned
   by a single `XMLDocument` instance.

2. **Factory methods** on `XMLDocument` create new nodes:
   - `NewElement(name)` → `XMLElement*`
   - `NewText(text)` → `XMLText*`
   - `NewComment(text)` → `XMLComment*`
   - `NewDeclaration(text)` → `XMLDeclaration*`
   - `NewUnknown(text)` → `XMLUnknown*`

3. **Inserting** a node into the tree attaches it to a parent:
   - `InsertFirstChild(node)`
   - `InsertEndChild(node)`
   - `InsertAfterChild(afterThis, node)`

4. **A node must only be inserted into the document that created it.**
   Cross-document insertion is undefined behavior.

5. **Deleting nodes:**
   - `DeleteChild(node)` — removes and destroys a specific child
   - `DeleteChildren()` — removes and destroys all children
   - `XMLDocument::Clear()` — destroys the entire tree

6. **`XMLDocument` destructor** destroys all remaining nodes.

7. **Unattached nodes** (created but never inserted) are still owned by the
   document and will be destroyed when the document is destroyed.

8. **Deep copying** across documents is supported via `DeepClone(targetDoc)`.

### Ownership Diagram

```
XMLDocument (memory pool)
├── owns → XMLElement "root"
│   ├── owns → XMLElement "child1"
│   │   └── owns → XMLText "hello"
│   └── owns → XMLComment "a comment"
└── owns → XMLDeclaration
```

---

## Entity Handling

When `processEntities` is `true` (the default), TinyXML2 processes the following
predefined XML entities during parsing and restores them during serialization:

### Predefined Entities

| Entity    | Character | Code Point |
|-----------|-----------|------------|
| `&amp;`   | `&`       | U+0026     |
| `&lt;`    | `<`       | U+003C     |
| `&gt;`    | `>`       | U+003E     |
| `&quot;`  | `"`       | U+0022     |
| `&apos;`  | `'`       | U+0027     |

### Numeric Character References

| Form    | Example   | Result    |
|---------|-----------|-----------|
| `&#N;`  | `&#65;`   | `A` (decimal) |
| `&#xN;` | `&#x41;`  | `A` (hexadecimal) |

### Behavior

- **Parsing:** Entity references are resolved to their character values in the
  in-memory DOM. `&amp;` becomes `&`, etc.
- **Serialization:** Characters that require escaping are written back as entity
  references. `&` becomes `&amp;`, etc.
- **`processEntities = false`:** Entity references are treated as literal text
  and are not resolved. The raw `&amp;` string is preserved in the DOM.

---

## Whitespace Modes

The `Whitespace` enum controls how whitespace is handled during parsing:

### `PRESERVE_WHITESPACE` (default)

All whitespace in the XML source is preserved exactly as written in text nodes.
Newlines, tabs, and spaces are kept verbatim.

```xml
<text>  hello   world  </text>
<!-- GetText() returns "  hello   world  " -->
```

### `COLLAPSE_WHITESPACE`

Contiguous runs of whitespace (spaces, tabs, newlines) are collapsed to a single
space character. Leading and trailing whitespace in text nodes is trimmed.

```xml
<text>  hello   world  </text>
<!-- GetText() returns "hello world" -->
```

### `PEDANTIC_WHITESPACE`

Similar to `PRESERVE_WHITESPACE`, but normalizes line endings:
- `\r\n` → `\n`
- `\r` → `\n`

All other whitespace is preserved as-is.

---

## BOM Handling

TinyXML2 supports UTF-8 Byte Order Marks (BOM):

- **Reading:** If a UTF-8 BOM (`0xEF 0xBB 0xBF`) is present at the start of
  the input, it is consumed and `HasBOM()` returns `true`.
- **Writing:** If `SetBOM(true)` has been called (or if the parsed document had
  a BOM), the BOM is written at the start of serialized output.
- **Encoding:** TinyXML2 **only** supports UTF-8. It does not support UTF-16,
  UTF-32, or other encodings. Non-UTF-8 input produces undefined behavior.

---

## Thread Safety

TinyXML2 provides **no thread safety guarantees**:

- **No global state:** TinyXML2 does not use global or static mutable state.
- **Document isolation:** Each `XMLDocument` is fully self-contained with its
  own memory pool.
- **Rule:** Each thread that needs to work with XML must use its own
  `XMLDocument` instance. Concurrent access to a single `XMLDocument` from
  multiple threads is **undefined behavior**.
- **Safe pattern:** Create, use, and destroy an `XMLDocument` entirely within
  a single thread. Transfer data (not pointers) between threads if needed.
