# TinyXML2 C++ to Rust Migration Guide

This guide is designed for developers transitioning from the C++ `tinyxml2` library to the Rust `tinyxml2-rs` implementation. While `tinyxml2-rs` maintains behavioral equivalence and matches XML parsing/printing semantics, it leverages Rust's type-safety, memory safety, and ownership models.

---

## 1. Class Mappings

TinyXML2 C++ utilizes an object-oriented inheritance tree under raw pointers. `tinyxml2-rs` represents this via `NodeId` handles into a generational arena managed by a `Document`, and provides safe references (`NodeRef` and `ElementRef`) to traverse the DOM.

| TinyXML2 C++ Class | Rust Equivalent | Notes |
|:---|:---|:---|
| `tinyxml2::XMLDocument` | [`Document`](crate::Document) | Owns the entire DOM arena and all string cache allocations. |
| `tinyxml2::XMLElement` | [`ElementRef`](crate::ElementRef) / [`ElementRefMut`](crate::ElementRefMut) | References to element nodes. Can also interact via a generic `NodeId`. |
| `tinyxml2::XMLNode` | [`NodeRef`](crate::NodeRef) / [`NodeRefMut`](crate::NodeRefMut) | References to any node (Elements, Comments, Text, CDATA, etc.). |
| `tinyxml2::XMLAttribute` | [`Attribute`](crate::Attribute) / [`Attributes` iterator](crate::Attributes) | Represents name-value attribute pairs. Accessed via `Attributes` iterator. |
| `tinyxml2::XMLPrinter` | [`XmlPrinter`](crate::XmlPrinter) | Used for serializing documents or elements to strings/writers. |
| `tinyxml2::XMLVisitor` | [`XmlVisitor`](crate::XmlVisitor) | Trait for implementing custom DOM visitors. |

---

## 2. Core Differences & Architecture

### Memory & Ownership: Raw Pointers vs. Arena Allocator
- **C++:** Node lifecycles are tied to the `XMLDocument` but managed through raw pointers (e.g., `XMLElement*`). Memory corruption can occur if pointers are held after deletion.
- **Rust:** The `Document` owns the DOM inside a **generational arena**. Nodes are identified by `NodeId` structs. Use-after-free and dangling pointers are impossible: accessing a deleted or invalid `NodeId` returns `None`.

### Error Handling: Error Codes vs. Result
- **C++:** Operations return `XMLError` codes or set error flags on the document. Developers must check `doc.Error()` and `doc.ErrorID()`.
- **Rust:** Operations return a standard `Result<T, XmlError>` which forces developers to handle errors cleanly using the `?` operator.

---

## 3. Side-by-Side Method Equivalents

### Loading & Parsing Files

#### C++
```cpp
#include "tinyxml2.h"
#include <iostream>

tinyxml2::XMLDocument doc;
if (doc.LoadFile("config.xml") != tinyxml2::XML_SUCCESS) {
    std::cerr << "Error: " << doc.ErrorStr() << std::endl;
}

const char* xml = "<root><child/></root>";
if (doc.Parse(xml) != tinyxml2::XML_SUCCESS) {
    std::cerr << "Parse error: " << doc.ErrorStr() << std::endl;
}
```

#### Rust
```rust
use tinyxml2::Document;

let mut doc = Document::new();
// Load file
doc.load_file_mut("config.xml")?;

// Parse string
let xml = "<root><child/></root>";
doc.parse_str(xml)?;
```

---

### Querying Attributes

#### C++
```cpp
tinyxml2::XMLElement* el = doc.FirstChildElement("item");
if (el) {
    // String attribute
    const char* val = el->Attribute("name");
    
    // Typed attribute parsing
    int id = 0;
    if (el->QueryIntAttribute("id", &id) == tinyxml2::XML_SUCCESS) {
        std::cout << "ID: " << id << std::endl;
    }
    
    double price = 0.0;
    el->QueryDoubleAttribute("price", &price);

    bool active = false;
    el->QueryBoolAttribute("active", &active);
}
```

#### Rust
```rust
if let Some(el) = doc.first_child_element(doc.root_node(), Some("item")) {
    let el_ref = doc.element(el).unwrap();
    
    // String attribute
    let name: Option<&str> = el_ref.attribute("name");

    // Typed attribute parsing returns Result
    let id: i32 = doc.query_int_attribute(el, "id")?;
    let price: f64 = doc.query_double_attribute(el, "price")?;
    let active: bool = doc.query_bool_attribute(el, "active")?;
}
```

---

### Programmatic DOM Insertion & Mutation

#### C++
```cpp
tinyxml2::XMLDocument doc;
tinyxml2::XMLNode* root = doc.NewElement("root");
doc.InsertFirstChild(root);

tinyxml2::XMLElement* child = doc.NewElement("child");
child->SetAttribute("key", "value");
child->SetText("Hello World");
root->InsertEndChild(child);

// Delete node
root->DeleteChild(child);
```

#### Rust
```rust
use tinyxml2::Document;

let mut doc = Document::new();
let root = doc.new_element("root");
doc.insert_first_child(doc.document_node(), root)?;

let child = doc.new_element("child");
doc.set_attribute(child, "key", "value")?;
doc.set_text(child, "Hello World")?;
doc.insert_end_child(root, child)?;

// Delete node
doc.delete_node(child);
```

---

### Traversing Parent/Sibling/Child Nodes (Iterators & Handles)

#### C++ (Manual Pointer Traversal)
```cpp
tinyxml2::XMLElement* root = doc.RootElement();
for (tinyxml2::XMLElement* child = root->FirstChildElement("item");
     child;
     child = child->NextSiblingElement("item")) {
    std::cout << child->Attribute("name") << std::endl;
}

// Fluent Handle Navigation
tinyxml2::XMLHandle docHandle(&doc);
tinyxml2::XMLElement* subChild = docHandle.FirstChildElement("root")
                                           .FirstChildElement("group")
                                           .FirstChildElement("item")
                                           .ToElement();
```

#### Rust (Idiomatic Iterators & Fluent Handles)
```rust
let root = doc.root_element().unwrap();

// Idiomatic Iterator pattern
for child_id in doc.child_elements_by_name(root.id(), "item") {
    let el = doc.element(child_id).unwrap();
    println!("{:?}", el.attribute("name"));
}

// Fluent Handle Navigation (Matches C++ Handle functionality)
let sub_child = doc.handle(doc.document_node())
    .first_child_element(Some("root"))
    .first_child_element(Some("group"))
    .first_child_element(Some("item"))
    .to_element();
```

---

## 4. FFI C-Layer Integration

For applications written in C or C++ that wish to integrate the `tinyxml2-rs` implementation as a drop-in binary replacement, the `tinyxml2-capi` crate exposes matching FFI C functions.

### Linkage & Headers
Add `tinyxml2-capi` to your Rust library configurations to compile it as `staticlib` or `cdylib`. The FFI builds a header file `tinyxml2_capi.h` automatically.

| TinyXML2 C++ / C API | FFI equivalent |
|:---|:---|
| `XMLDocument* doc = new XMLDocument();` | `TxDocument* doc = tx_document_new();` |
| `doc->Parse(xml)` | `tx_document_parse(doc, xml)` |
| `doc->SaveFile(path)` | `tx_document_save_file(doc, path)` |
| `doc->FirstChildElement(name)` | `TxNodeId node = tx_document_first_child_element(doc, parent, name)` |
| `node.ToElement()->Attribute(name)` | `const char* val = tx_element_attribute(doc, node, name)` |
| `delete doc;` | `tx_document_free(doc);` |

---

## 5. Summary Cheat Sheet

| Operation | C++ TinyXML2 | Rust `tinyxml2-rs` |
|:---|:---|:---|
| **Root Element** | `doc.RootElement()` | `doc.root_element()` |
| **Get Text** | `el->GetText()` | `doc.get_text(el_id)` / `el_ref.text()` |
| **Set Text** | `el->SetText(str)` | `doc.set_text(el_id, str)` |
| **Add Comment** | `doc.NewComment(str)` | `doc.new_comment(str)` |
| **Parent Node** | `node->Parent()` | `doc.parent(node_id)` |
| **Sibling Iter** | `node->NextSibling()` | `doc.next_sibling(node_id)` |
| **Delete All** | `doc.Clear()` | `doc.clear()` |
