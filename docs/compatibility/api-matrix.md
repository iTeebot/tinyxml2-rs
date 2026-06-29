# API Compatibility Matrix

This document maps every public method in the TinyXML2 C++ library to its planned
Rust equivalent in `tinyxml2-rs`. The matrix covers all classes and indicates
whether each method is directly planned, adapted to Rust idioms, or not applicable.

## Legend

| Symbol | Meaning |
|--------|---------|
| 🟢 | **Planned** — Direct 1:1 equivalent with the same semantics |
| 🔵 | **Adapted** — Rust-idiomatic equivalent with changed signature or behavior |
| ⚪ | **N/A** — Not applicable in Rust (e.g., raw-pointer APIs, C-specific patterns) |

---

## XMLDocument

The root of the DOM tree. Owns all nodes via a generational arena.

| TinyXML2 Method | Rust Equivalent | Status | Notes |
|---|---|---|---|
| `Parse(const char*, size_t)` | `document.parse(xml)` | 🔵 | Accepts `&str`; returns `Result<(), XmlError>` instead of `XMLError` enum |
| `LoadFile(const char*)` | `document.load_file(path)` | 🔵 | Accepts `impl AsRef<Path>`; returns `Result<(), XmlError>` |
| `LoadFile(FILE*)` | — | ⚪ | Rust uses `std::io::Read`; see `parse_reader()` |
| `SaveFile(const char*)` | `document.save_file(path)` | 🔵 | Accepts `impl AsRef<Path>`; returns `Result<(), XmlError>` |
| `SaveFile(FILE*)` | — | ⚪ | Rust uses `std::io::Write`; see `write_to()` |
| `ProcessEntities()` | `document.process_entities()` | 🟢 | Returns `bool` |
| `WhitespaceMode()` | `document.whitespace_mode()` | 🟢 | Returns `WhitespaceMode` enum |
| `HasBOM()` | `document.has_bom()` | 🟢 | Returns `bool` |
| `SetBOM(bool)` | `document.set_bom(bom)` | 🟢 | |
| `RootElement()` | `document.root_element()` | 🔵 | Returns `Option<NodeId>` instead of nullable pointer |
| `Print(XMLPrinter*)` | `document.print(printer)` | 🔵 | Accepts `&mut XmlPrinter` |
| `Accept(XMLVisitor*)` | `document.accept(visitor)` | 🔵 | Accepts `&mut impl XmlVisitor` |
| `NewElement(const char*)` | `document.new_element(name)` | 🔵 | Returns `NodeId` (arena-allocated) |
| `NewComment(const char*)` | `document.new_comment(text)` | 🔵 | Returns `NodeId` |
| `NewText(const char*)` | `document.new_text(text)` | 🔵 | Returns `NodeId` |
| `NewDeclaration(const char*)` | `document.new_declaration(text)` | 🔵 | Returns `NodeId`; default value `"xml version=\"1.0\" encoding=\"UTF-8\""` |
| `NewUnknown(const char*)` | `document.new_unknown(text)` | 🔵 | Returns `NodeId` |
| `Error()` | — | ⚪ | Replaced by `Result`-based error handling |
| `ErrorID()` | — | ⚪ | Errors are returned as `XmlError` variants, not polled |
| `ErrorStr()` | `XmlError::Display` impl | 🔵 | Use `format!("{}", error)` or `.to_string()` |
| `ErrorLineNum()` | `XmlError::line_num()` | 🔵 | Available on parse-error variants |
| `ClearError()` | — | ⚪ | No error state to clear; errors are values |
| `PrintError()` | `eprintln!("{}", error)` | 🔵 | Standard Rust `Display` formatting |
| `ErrorIDToName(XMLError)` | `XmlError::name()` | 🔵 | Returns `&'static str` |
| `ShallowClone(XMLDocument*)` | `document.shallow_clone()` | 🟢 | Returns new `XmlDocument` |
| `ShallowEqual(const XMLNode*)` | `document.shallow_equal(other)` | 🟢 | Returns `bool` |
| `SetMaxElementDepth(int)` | `ParseOptions::max_element_depth(n)` | 🔵 | Builder-style configuration |
| `MaxElementDepth()` | `document.max_element_depth()` | 🟢 | Returns `usize` |
| `Clear()` | `document.clear()` | 🟢 | Resets the document and arena |

---

## XMLNode

Base class for all DOM nodes. In Rust, node operations are methods on `XmlDocument`
that accept a `NodeId`.

| TinyXML2 Method | Rust Equivalent | Status | Notes |
|---|---|---|---|
| `GetDocument()` | — | ⚪ | Nodes are always accessed through their owning `XmlDocument` |
| `ToElement()` | `document.node_as_element(id)` | 🔵 | Returns `Option<&Element>` instead of nullable pointer |
| `ToText()` | `document.node_as_text(id)` | 🔵 | Returns `Option<&Text>` |
| `ToComment()` | `document.node_as_comment(id)` | 🔵 | Returns `Option<&Comment>` |
| `ToDocument()` | — | ⚪ | Document is the owner, not a node variant |
| `ToDeclaration()` | `document.node_as_declaration(id)` | 🔵 | Returns `Option<&Declaration>` |
| `ToUnknown()` | `document.node_as_unknown(id)` | 🔵 | Returns `Option<&Unknown>` |
| `Value()` | `document.node_value(id)` | 🔵 | Returns `Result<&str, XmlError>` |
| `SetValue(const char*)` | `document.set_node_value(id, value)` | 🔵 | Returns `Result<(), XmlError>` |
| `GetLineNum()` | `document.node_line_num(id)` | 🔵 | Returns `Result<u32, XmlError>` |
| `Parent()` | `document.parent(id)` | 🔵 | Returns `Option<NodeId>` |
| `NoChildren()` | `document.no_children(id)` | 🔵 | Returns `Result<bool, XmlError>` |
| `FirstChild()` | `document.first_child(id)` | 🔵 | Returns `Option<NodeId>` |
| `FirstChildElement(const char*)` | `document.first_child_element(id, name)` | 🔵 | `name: Option<&str>` for optional filtering |
| `LastChild()` | `document.last_child(id)` | 🔵 | Returns `Option<NodeId>` |
| `LastChildElement(const char*)` | `document.last_child_element(id, name)` | 🔵 | `name: Option<&str>` for optional filtering |
| `PreviousSibling()` | `document.previous_sibling(id)` | 🔵 | Returns `Option<NodeId>` |
| `PreviousSiblingElement(const char*)` | `document.previous_sibling_element(id, name)` | 🔵 | `name: Option<&str>` |
| `NextSibling()` | `document.next_sibling(id)` | 🔵 | Returns `Option<NodeId>` |
| `NextSiblingElement(const char*)` | `document.next_sibling_element(id, name)` | 🔵 | `name: Option<&str>` |
| `InsertFirstChild(XMLNode*)` | `document.insert_first_child(parent, child)` | 🔵 | Accepts `NodeId` pair; returns `Result<NodeId, XmlError>` |
| `InsertEndChild(XMLNode*)` | `document.insert_end_child(parent, child)` | 🔵 | Accepts `NodeId` pair; returns `Result<NodeId, XmlError>` |
| `InsertAfterChild(XMLNode*, XMLNode*)` | `document.insert_after_child(parent, after, child)` | 🔵 | Three `NodeId` args; returns `Result<NodeId, XmlError>` |
| `DeleteChild(XMLNode*)` | `document.delete_child(parent, child)` | 🔵 | Accepts `NodeId` pair; returns `Result<(), XmlError>` |
| `DeleteChildren()` | `document.delete_children(id)` | 🔵 | Accepts `NodeId`; returns `Result<(), XmlError>` |
| `ShallowClone(XMLDocument*)` | `document.shallow_clone_node(id)` | 🔵 | Returns `Result<NodeId, XmlError>` |
| `ShallowEqual(const XMLNode*)` | `document.shallow_equal_nodes(a, b)` | 🔵 | Returns `Result<bool, XmlError>` |
| `Accept(XMLVisitor*)` | `document.accept_node(id, visitor)` | 🔵 | Accepts `&mut impl XmlVisitor` |
| `DeepClone(XMLDocument*)` | `document.deep_clone_node(id)` | 🔵 | Returns `Result<NodeId, XmlError>` |

---

## XMLElement

Element nodes with a tag name and attributes. Accessed via `NodeId` on `XmlDocument`.

| TinyXML2 Method | Rust Equivalent | Status | Notes |
|---|---|---|---|
| `ToElement()` | — | ⚪ | Already accessing as element via typed methods |
| `Name()` | `document.element_name(id)` | 🔵 | Returns `Result<&str, XmlError>` |
| `SetName(const char*)` | `document.set_element_name(id, name)` | 🔵 | Returns `Result<(), XmlError>` |
| `Accept(XMLVisitor*)` | `document.accept_node(id, visitor)` | 🔵 | Shared with XMLNode |
| `Attribute(const char*, const char*)` | `document.attribute(id, name)` | 🔵 | Returns `Result<Option<&str>, XmlError>`; no default param |
| `IntAttribute(const char*, int)` | `document.int_attribute(id, name)` | 🔵 | Returns `Result<i32, XmlError>` instead of default-on-error |
| `UnsignedAttribute(const char*, unsigned)` | `document.unsigned_attribute(id, name)` | 🔵 | Returns `Result<u32, XmlError>` |
| `Int64Attribute(const char*, int64_t)` | `document.int64_attribute(id, name)` | 🔵 | Returns `Result<i64, XmlError>` |
| `Unsigned64Attribute(const char*, uint64_t)` | `document.unsigned64_attribute(id, name)` | 🔵 | Returns `Result<u64, XmlError>` |
| `BoolAttribute(const char*, bool)` | `document.bool_attribute(id, name)` | 🔵 | Returns `Result<bool, XmlError>` |
| `DoubleAttribute(const char*, double)` | `document.double_attribute(id, name)` | 🔵 | Returns `Result<f64, XmlError>` |
| `FloatAttribute(const char*, float)` | `document.float_attribute(id, name)` | 🔵 | Returns `Result<f32, XmlError>` |
| `QueryIntAttribute(const char*, int*)` | `document.query_int_attribute(id, name)` | 🔵 | Returns `Result<i32, XmlError>` (merged with typed getter) |
| `QueryUnsignedAttribute(const char*, unsigned*)` | `document.query_unsigned_attribute(id, name)` | 🔵 | Returns `Result<u32, XmlError>` |
| `QueryInt64Attribute(const char*, int64_t*)` | `document.query_int64_attribute(id, name)` | 🔵 | Returns `Result<i64, XmlError>` |
| `QueryUnsigned64Attribute(const char*, uint64_t*)` | `document.query_unsigned64_attribute(id, name)` | 🔵 | Returns `Result<u64, XmlError>` |
| `QueryBoolAttribute(const char*, bool*)` | `document.query_bool_attribute(id, name)` | 🔵 | Returns `Result<bool, XmlError>` |
| `QueryDoubleAttribute(const char*, double*)` | `document.query_double_attribute(id, name)` | 🔵 | Returns `Result<f64, XmlError>` |
| `QueryFloatAttribute(const char*, float*)` | `document.query_float_attribute(id, name)` | 🔵 | Returns `Result<f32, XmlError>` |
| `QueryStringAttribute(const char*, const char**)` | `document.query_string_attribute(id, name)` | 🔵 | Returns `Result<&str, XmlError>` |
| `SetAttribute(const char*, const char*)` | `document.set_attribute(id, name, value)` | 🔵 | Generic over `impl Into<AttrValue>` covers all 8 C++ overloads |
| `SetAttribute(const char*, int)` | `document.set_attribute(id, name, value)` | 🔵 | Same generic method; `i32` implements `Into<AttrValue>` |
| `SetAttribute(const char*, unsigned)` | `document.set_attribute(id, name, value)` | 🔵 | `u32` implements `Into<AttrValue>` |
| `SetAttribute(const char*, int64_t)` | `document.set_attribute(id, name, value)` | 🔵 | `i64` implements `Into<AttrValue>` |
| `SetAttribute(const char*, uint64_t)` | `document.set_attribute(id, name, value)` | 🔵 | `u64` implements `Into<AttrValue>` |
| `SetAttribute(const char*, bool)` | `document.set_attribute(id, name, value)` | 🔵 | `bool` implements `Into<AttrValue>` |
| `SetAttribute(const char*, double)` | `document.set_attribute(id, name, value)` | 🔵 | `f64` implements `Into<AttrValue>` |
| `SetAttribute(const char*, float)` | `document.set_attribute(id, name, value)` | 🔵 | `f32` implements `Into<AttrValue>` |
| `DeleteAttribute(const char*)` | `document.delete_attribute(id, name)` | 🔵 | Returns `Result<(), XmlError>` |
| `FirstAttribute()` | `document.first_attribute(id)` | 🔵 | Returns `Result<Option<&XmlAttribute>, XmlError>` |
| `FindAttribute(const char*)` | `document.find_attribute(id, name)` | 🔵 | Returns `Result<Option<&XmlAttribute>, XmlError>` |
| `GetText()` | `document.get_text(id)` | 🔵 | Returns `Result<Option<&str>, XmlError>` |
| `SetText(const char*)` | `document.set_text(id, value)` | 🔵 | Generic over `impl Into<TextValue>` covers all 8 overloads |
| `SetText(int)` | `document.set_text(id, value)` | 🔵 | `i32` implements `Into<TextValue>` |
| `SetText(unsigned)` | `document.set_text(id, value)` | 🔵 | `u32` implements `Into<TextValue>` |
| `SetText(int64_t)` | `document.set_text(id, value)` | 🔵 | `i64` implements `Into<TextValue>` |
| `SetText(uint64_t)` | `document.set_text(id, value)` | 🔵 | `u64` implements `Into<TextValue>` |
| `SetText(bool)` | `document.set_text(id, value)` | 🔵 | `bool` implements `Into<TextValue>` |
| `SetText(double)` | `document.set_text(id, value)` | 🔵 | `f64` implements `Into<TextValue>` |
| `SetText(float)` | `document.set_text(id, value)` | 🔵 | `f32` implements `Into<TextValue>` |
| `QueryIntText(int*)` | `document.query_int_text(id)` | 🔵 | Returns `Result<i32, XmlError>` |
| `QueryUnsignedText(unsigned*)` | `document.query_unsigned_text(id)` | 🔵 | Returns `Result<u32, XmlError>` |
| `QueryInt64Text(int64_t*)` | `document.query_int64_text(id)` | 🔵 | Returns `Result<i64, XmlError>` |
| `QueryUnsigned64Text(uint64_t*)` | `document.query_unsigned64_text(id)` | 🔵 | Returns `Result<u64, XmlError>` |
| `QueryBoolText(bool*)` | `document.query_bool_text(id)` | 🔵 | Returns `Result<bool, XmlError>` |
| `QueryDoubleText(double*)` | `document.query_double_text(id)` | 🔵 | Returns `Result<f64, XmlError>` |
| `QueryFloatText(float*)` | `document.query_float_text(id)` | 🔵 | Returns `Result<f32, XmlError>` |
| `IntText(int)` | `document.int_text(id)` | 🔵 | Returns `Result<i32, XmlError>` instead of default-on-error |
| `UnsignedText(unsigned)` | `document.unsigned_text(id)` | 🔵 | Returns `Result<u32, XmlError>` |
| `Int64Text(int64_t)` | `document.int64_text(id)` | 🔵 | Returns `Result<i64, XmlError>` |
| `Unsigned64Text(uint64_t)` | `document.unsigned64_text(id)` | 🔵 | Returns `Result<u64, XmlError>` |
| `BoolText(bool)` | `document.bool_text(id)` | 🔵 | Returns `Result<bool, XmlError>` |
| `DoubleText(double)` | `document.double_text(id)` | 🔵 | Returns `Result<f64, XmlError>` |
| `FloatText(float)` | `document.float_text(id)` | 🔵 | Returns `Result<f32, XmlError>` |
| `ClosingType()` | `document.closing_type(id)` | 🟢 | Returns `ClosingType` enum |
| `ShallowClone(XMLDocument*)` | `document.shallow_clone_node(id)` | 🔵 | Shared with XMLNode |
| `ShallowEqual(const XMLNode*)` | `document.shallow_equal_nodes(a, b)` | 🔵 | Shared with XMLNode |

---

## XMLAttribute

Key-value pairs on elements. In Rust, attributes are value types returned by
reference from element queries.

| TinyXML2 Method | Rust Equivalent | Status | Notes |
|---|---|---|---|
| `Name()` | `attr.name()` | 🟢 | Returns `&str` |
| `Value()` | `attr.value()` | 🟢 | Returns `&str` |
| `GetLineNum()` | `attr.line_num()` | 🟢 | Returns `u32` |
| `Next()` | `attr.next()` / iterator | 🔵 | Attributes implement `Iterator` for chaining |
| `IntValue()` | `attr.int_value()` | 🔵 | Returns `Result<i32, XmlError>` |
| `UnsignedValue()` | `attr.unsigned_value()` | 🔵 | Returns `Result<u32, XmlError>` |
| `Int64Value()` | `attr.int64_value()` | 🔵 | Returns `Result<i64, XmlError>` |
| `Unsigned64Value()` | `attr.unsigned64_value()` | 🔵 | Returns `Result<u64, XmlError>` |
| `BoolValue()` | `attr.bool_value()` | 🔵 | Returns `Result<bool, XmlError>` |
| `DoubleValue()` | `attr.double_value()` | 🔵 | Returns `Result<f64, XmlError>` |
| `FloatValue()` | `attr.float_value()` | 🔵 | Returns `Result<f32, XmlError>` |
| `QueryIntValue(int*)` | `attr.query_int_value()` | 🔵 | Returns `Result<i32, XmlError>` (same as `int_value()`) |
| `QueryUnsignedValue(unsigned*)` | `attr.query_unsigned_value()` | 🔵 | Returns `Result<u32, XmlError>` |
| `QueryInt64Value(int64_t*)` | `attr.query_int64_value()` | 🔵 | Returns `Result<i64, XmlError>` |
| `QueryUnsigned64Value(uint64_t*)` | `attr.query_unsigned64_value()` | 🔵 | Returns `Result<u64, XmlError>` |
| `QueryBoolValue(bool*)` | `attr.query_bool_value()` | 🔵 | Returns `Result<bool, XmlError>` |
| `QueryDoubleValue(double*)` | `attr.query_double_value()` | 🔵 | Returns `Result<f64, XmlError>` |
| `QueryFloatValue(float*)` | `attr.query_float_value()` | 🔵 | Returns `Result<f32, XmlError>` |
| `SetAttribute(const char*)` | `document.set_attribute(id, name, value)` | 🔵 | Attribute mutation goes through `XmlDocument` |
| `SetAttribute(int)` | `document.set_attribute(id, name, value)` | 🔵 | Generic via `Into<AttrValue>` |
| `SetAttribute(unsigned)` | `document.set_attribute(id, name, value)` | 🔵 | Generic via `Into<AttrValue>` |
| `SetAttribute(int64_t)` | `document.set_attribute(id, name, value)` | 🔵 | Generic via `Into<AttrValue>` |
| `SetAttribute(uint64_t)` | `document.set_attribute(id, name, value)` | 🔵 | Generic via `Into<AttrValue>` |
| `SetAttribute(bool)` | `document.set_attribute(id, name, value)` | 🔵 | Generic via `Into<AttrValue>` |
| `SetAttribute(double)` | `document.set_attribute(id, name, value)` | 🔵 | Generic via `Into<AttrValue>` |
| `SetAttribute(float)` | `document.set_attribute(id, name, value)` | 🔵 | Generic via `Into<AttrValue>` |

---

## XMLText

Text content nodes (both regular text and CDATA sections).

| TinyXML2 Method | Rust Equivalent | Status | Notes |
|---|---|---|---|
| `ToText()` | `document.node_as_text(id)` | 🔵 | Returns `Option<&Text>` |
| `Accept(XMLVisitor*)` | `document.accept_node(id, visitor)` | 🔵 | Shared with XMLNode |
| `CData()` | `document.text_is_cdata(id)` | 🔵 | Returns `Result<bool, XmlError>` |
| `SetCData(bool)` | `document.set_text_cdata(id, is_cdata)` | 🔵 | Returns `Result<(), XmlError>` |
| `ShallowClone(XMLDocument*)` | `document.shallow_clone_node(id)` | 🔵 | Shared with XMLNode |
| `ShallowEqual(const XMLNode*)` | `document.shallow_equal_nodes(a, b)` | 🔵 | Shared with XMLNode |

---

## XMLComment

Comment nodes (`<!-- ... -->`).

| TinyXML2 Method | Rust Equivalent | Status | Notes |
|---|---|---|---|
| `ToComment()` | `document.node_as_comment(id)` | 🔵 | Returns `Option<&Comment>` |
| `Accept(XMLVisitor*)` | `document.accept_node(id, visitor)` | 🔵 | Shared with XMLNode |
| `ShallowClone(XMLDocument*)` | `document.shallow_clone_node(id)` | 🔵 | Shared with XMLNode |
| `ShallowEqual(const XMLNode*)` | `document.shallow_equal_nodes(a, b)` | 🔵 | Shared with XMLNode |

---

## XMLDeclaration

XML declaration nodes (`<?xml ... ?>`).

| TinyXML2 Method | Rust Equivalent | Status | Notes |
|---|---|---|---|
| `ToDeclaration()` | `document.node_as_declaration(id)` | 🔵 | Returns `Option<&Declaration>` |
| `Accept(XMLVisitor*)` | `document.accept_node(id, visitor)` | 🔵 | Shared with XMLNode |
| `ShallowClone(XMLDocument*)` | `document.shallow_clone_node(id)` | 🔵 | Shared with XMLNode |
| `ShallowEqual(const XMLNode*)` | `document.shallow_equal_nodes(a, b)` | 🔵 | Shared with XMLNode |

---

## XMLUnknown

Unknown / unrecognized markup nodes (e.g., `<!DOCTYPE ...>`).

| TinyXML2 Method | Rust Equivalent | Status | Notes |
|---|---|---|---|
| `ToUnknown()` | `document.node_as_unknown(id)` | 🔵 | Returns `Option<&Unknown>` |
| `Accept(XMLVisitor*)` | `document.accept_node(id, visitor)` | 🔵 | Shared with XMLNode |
| `ShallowClone(XMLDocument*)` | `document.shallow_clone_node(id)` | 🔵 | Shared with XMLNode |
| `ShallowEqual(const XMLNode*)` | `document.shallow_equal_nodes(a, b)` | 🔵 | Shared with XMLNode |

---

## XMLVisitor

SAX-like visitor pattern for document traversal. In Rust, implemented as a trait.

| TinyXML2 Method | Rust Equivalent | Status | Notes |
|---|---|---|---|
| `VisitEnter(const XMLDocument&)` | `fn visit_enter_document(&mut self, doc) -> bool` | 🔵 | Takes `&XmlDocument` reference |
| `VisitExit(const XMLDocument&)` | `fn visit_exit_document(&mut self, doc) -> bool` | 🔵 | Takes `&XmlDocument` reference |
| `VisitEnter(const XMLElement&, const XMLAttribute*)` | `fn visit_enter_element(&mut self, doc, id, first_attr) -> bool` | 🔵 | Takes `NodeId` + `Option<&XmlAttribute>` |
| `VisitExit(const XMLElement&)` | `fn visit_exit_element(&mut self, doc, id) -> bool` | 🔵 | Takes `NodeId` |
| `Visit(const XMLDeclaration&)` | `fn visit_declaration(&mut self, doc, id) -> bool` | 🔵 | Takes `NodeId` |
| `Visit(const XMLText&)` | `fn visit_text(&mut self, doc, id) -> bool` | 🔵 | Takes `NodeId` |
| `Visit(const XMLComment&)` | `fn visit_comment(&mut self, doc, id) -> bool` | 🔵 | Takes `NodeId` |
| `Visit(const XMLUnknown&)` | `fn visit_unknown(&mut self, doc, id) -> bool` | 🔵 | Takes `NodeId` |

> **Note:** All visitor trait methods have default implementations that return `true`
> (continue traversal), matching TinyXML2's default behavior. Implementors only need
> to override the methods they care about.

---

## XMLPrinter

Dual-mode printer (compact / pretty-print) that also implements the `XmlVisitor` trait.

| TinyXML2 Method | Rust Equivalent | Status | Notes |
|---|---|---|---|
| `PushHeader(bool, bool)` | `printer.push_header(write_bom, write_declaration)` | 🟢 | |
| `OpenElement(const char*, bool)` | `printer.open_element(name, compact)` | 🔵 | `compact` is `bool` (default `false`) |
| `PushAttribute(const char*, const char*)` | `printer.push_attribute(name, value)` | 🔵 | Generic over `impl Into<AttrValue>` covers all 7 overloads |
| `PushAttribute(const char*, int)` | `printer.push_attribute(name, value)` | 🔵 | `i32` implements `Into<AttrValue>` |
| `PushAttribute(const char*, unsigned)` | `printer.push_attribute(name, value)` | 🔵 | `u32` implements `Into<AttrValue>` |
| `PushAttribute(const char*, int64_t)` | `printer.push_attribute(name, value)` | 🔵 | `i64` implements `Into<AttrValue>` |
| `PushAttribute(const char*, uint64_t)` | `printer.push_attribute(name, value)` | 🔵 | `u64` implements `Into<AttrValue>` |
| `PushAttribute(const char*, bool)` | `printer.push_attribute(name, value)` | 🔵 | `bool` implements `Into<AttrValue>` |
| `PushAttribute(const char*, double)` | `printer.push_attribute(name, value)` | 🔵 | `f64` implements `Into<AttrValue>` |
| `PushComment(const char*)` | `printer.push_comment(text)` | 🟢 | |
| `PushText(const char*, bool)` | `printer.push_text(value, cdata)` | 🔵 | Generic over `impl Into<TextValue>` covers all 7 overloads |
| `PushText(int)` | `printer.push_text(value, false)` | 🔵 | `i32` implements `Into<TextValue>` |
| `PushText(unsigned)` | `printer.push_text(value, false)` | 🔵 | `u32` implements `Into<TextValue>` |
| `PushText(int64_t)` | `printer.push_text(value, false)` | 🔵 | `i64` implements `Into<TextValue>` |
| `PushText(uint64_t)` | `printer.push_text(value, false)` | 🔵 | `u64` implements `Into<TextValue>` |
| `PushText(bool)` | `printer.push_text(value, false)` | 🔵 | `bool` implements `Into<TextValue>` |
| `PushText(float)` | `printer.push_text(value, false)` | 🔵 | `f32` implements `Into<TextValue>` |
| `PushUnknown(const char*)` | `printer.push_unknown(text)` | 🟢 | |
| `CloseElement(bool)` | `printer.close_element(compact)` | 🔵 | `compact` is `bool` (default `false`) |
| `CStr()` | `printer.as_str()` | 🔵 | Returns `&str` (UTF-8 guaranteed) |
| `CStrSize()` | `printer.len()` | 🔵 | Returns `usize` |
| `ClearBuffer()` | `printer.clear()` | 🟢 | |
| All visitor overrides | Implemented via `impl XmlVisitor for XmlPrinter` | 🔵 | See XMLVisitor table |

> **Note:** The C++ `XMLPrinter` can write directly to a `FILE*`. In Rust, use
> `printer.write_to(writer)` where `writer: &mut impl std::io::Write`.

---

## XMLHandle

Null-safe navigation wrapper. In Rust, this operates on `Option<NodeId>` with
method chaining.

| TinyXML2 Method | Rust Equivalent | Status | Notes |
|---|---|---|---|
| `FirstChild()` | `handle.first_child()` | 🔵 | Returns `XmlHandle` (chainable) |
| `FirstChildElement(const char*)` | `handle.first_child_element(name)` | 🔵 | `name: Option<&str>` |
| `LastChild()` | `handle.last_child()` | 🔵 | Returns `XmlHandle` |
| `LastChildElement(const char*)` | `handle.last_child_element(name)` | 🔵 | `name: Option<&str>` |
| `PreviousSibling()` | `handle.previous_sibling()` | 🔵 | Returns `XmlHandle` |
| `PreviousSiblingElement(const char*)` | `handle.previous_sibling_element(name)` | 🔵 | `name: Option<&str>` |
| `NextSibling()` | `handle.next_sibling()` | 🔵 | Returns `XmlHandle` |
| `NextSiblingElement(const char*)` | `handle.next_sibling_element(name)` | 🔵 | `name: Option<&str>` |
| `ToNode()` | `handle.to_node_id()` | 🔵 | Returns `Option<NodeId>` |
| `ToElement()` | `handle.to_element()` | 🔵 | Returns `Option<NodeId>` (validated as element) |
| `ToText()` | `handle.to_text()` | 🔵 | Returns `Option<NodeId>` (validated as text) |
| `ToUnknown()` | `handle.to_unknown()` | 🔵 | Returns `Option<NodeId>` (validated as unknown) |

---

## XMLConstHandle

Const-correct version of `XMLHandle` in C++.

| TinyXML2 Method | Rust Equivalent | Status | Notes |
|---|---|---|---|
| All methods (same as XMLHandle) | — | ⚪ | Rust's borrow checker makes const/mut distinction automatic. A single `XmlHandle` type with `&XmlDocument` covers all const handle use cases. No separate type needed. |

> **Note:** In C++, `XMLHandle` and `XMLConstHandle` exist to provide const-correct
> and non-const navigation wrappers. In Rust, this distinction is unnecessary because
> the borrow checker enforces immutability at the reference level. A single `XmlHandle`
> type parameterized over `&XmlDocument` provides the same safety guarantees.

---

## Summary Statistics

| Category | Count |
|---|---|
| Total C++ methods mapped | ~200 |
| 🟢 Planned (direct equivalent) | ~15 |
| 🔵 Adapted (Rust idiomatic) | ~175 |
| ⚪ N/A (not applicable) | ~10 |

The overwhelming majority of methods are **adapted** rather than directly ported,
reflecting Rust's stronger type system, `Result`-based error handling, and
ownership model. Despite the signature differences, the **semantic behavior** of
each method is preserved — see [behavior.md](./behavior.md) for details.
