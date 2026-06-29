# Migration Guide: TinyXML2 C++ to tinyxml2-rs

## Overview

This guide helps developers migrate from the TinyXML2 C++ library to `tinyxml2-rs`,
the Rust implementation. It provides side-by-side code examples for every common
operation, explains key conceptual differences, and highlights common pitfalls.

### Target Audience

- C++ developers familiar with TinyXML2 who are porting code to Rust
- Rust developers building new projects who want to understand how `tinyxml2-rs`
  relates to the original C++ API
- Teams maintaining both C++ and Rust codebases that need interop via the C FFI crate

### Prerequisites

Add `tinyxml2` to your `Cargo.toml`:

```toml
[dependencies]
tinyxml2 = "0.1"
```

---

## Side-by-Side Examples

### 1. Creating a Document

**C++:**
```cpp
#include "tinyxml2.h"
using namespace tinyxml2;

XMLDocument doc;
// doc is ready to use — stack allocated, no heap setup needed
```

**Rust:**
```rust
use tinyxml2::XmlDocument;

let mut doc = XmlDocument::new();
// doc is ready to use — arena is allocated internally
```

> **Key difference:** Both create an empty document. In C++, configuration is passed
> via constructor parameters. In Rust, configuration is passed via `ParseOptions`
> at parse time (see Example 10).

---

### 2. Parsing XML from a String

**C++:**
```cpp
XMLDocument doc;
XMLError err = doc.Parse("<root><child>Hello</child></root>");
if (err != XML_SUCCESS) {
    printf("Parse error: %s\n", doc.ErrorStr());
    return;
}
// doc is now populated
```

**Rust:**
```rust
use tinyxml2::XmlDocument;

let mut doc = XmlDocument::new();
doc.parse("<root><child>Hello</child></root>")?;
// doc is now populated
// The ? operator propagates XmlError on failure
```

> **Key difference:** Rust uses `Result`-based error handling. The `?` operator
> replaces manual error checking. There is no error state to poll.

---

### 3. Navigating the DOM

**C++:**
```cpp
XMLElement* root = doc.RootElement();
if (!root) return;

// First child element
XMLElement* child = root->FirstChildElement();

// First child element with a specific name
XMLElement* named = root->FirstChildElement("item");

// Iterate siblings
for (XMLElement* e = root->FirstChildElement(); e != nullptr;
     e = e->NextSiblingElement()) {
    printf("Element: %s\n", e->Name());
}

// Navigate up
XMLNode* parent = child->Parent();
```

**Rust:**
```rust
let root = doc.root_element().ok_or("no root element")?;

// First child element
let child = doc.first_child_element(root, None);

// First child element with a specific name
let named = doc.first_child_element(root, Some("item"));

// Iterate siblings
let mut current = doc.first_child_element(root, None);
while let Some(elem) = current {
    let name = doc.element_name(elem)?;
    println!("Element: {}", name);
    current = doc.next_sibling_element(elem, None);
}

// Navigate up
let parent = doc.parent(child.unwrap());
```

> **Key difference:** Navigation returns `Option<NodeId>` instead of nullable pointers.
> Pattern matching (`if let`, `while let`) replaces null checks.

---

### 4. Reading Attributes

**C++:**
```cpp
XMLElement* elem = root->FirstChildElement("player");

// String attribute
const char* name = elem->Attribute("name");

// Int attribute (with default value)
int score = elem->IntAttribute("score", 0);

// Bool attribute
bool active = elem->BoolAttribute("active", false);

// Float attribute
float speed = elem->FloatAttribute("speed", 1.0f);

// Query-style (explicit error checking)
int health;
XMLError err = elem->QueryIntAttribute("health", &health);
if (err == XML_SUCCESS) {
    printf("Health: %d\n", health);
}
```

**Rust:**
```rust
let elem = doc.first_child_element(root, Some("player"))
    .ok_or("player not found")?;

// String attribute
let name: Option<&str> = doc.attribute(elem, "name")?;

// Int attribute (returns Result, no silent defaults)
let score: i32 = doc.int_attribute(elem, "score")?;

// Bool attribute
let active: bool = doc.bool_attribute(elem, "active")?;

// Float attribute
let speed: f32 = doc.float_attribute(elem, "speed")?;

// Query-style (same as typed getter in Rust — both return Result)
match doc.query_int_attribute(elem, "health") {
    Ok(health) => println!("Health: {}", health),
    Err(e) => eprintln!("Could not read health: {}", e),
}

// Using unwrap_or for default values
let score: i32 = doc.int_attribute(elem, "score").unwrap_or(0);
```

> **Key difference:** C++ returns a default value on missing/invalid attributes.
> Rust returns `Result`, forcing explicit handling. Use `.unwrap_or(default)` to
> replicate the C++ default-value behavior.

---

### 5. Modifying the Tree

**C++:**
```cpp
// Add a new element
XMLElement* item = doc.NewElement("item");
item->SetAttribute("id", 42);
item->SetAttribute("name", "sword");
root->InsertEndChild(item);

// Set text content
XMLElement* desc = doc.NewElement("description");
desc->SetText("A sharp blade.");
item->InsertEndChild(desc);

// Delete a child
root->DeleteChild(item);

// Delete all children
root->DeleteChildren();
```

**Rust:**
```rust
// Add a new element
let item = doc.new_element("item");
doc.set_attribute(item, "id", 42)?;
doc.set_attribute(item, "name", "sword")?;
doc.insert_end_child(root, item)?;

// Set text content
let desc = doc.new_element("description");
doc.set_text(desc, "A sharp blade.")?;
doc.insert_end_child(item, desc)?;

// Delete a child
doc.delete_child(root, item)?;

// Delete all children
doc.delete_children(root)?;
```

> **Key difference:** In C++, you call methods on the node pointer. In Rust, all
> mutations go through `XmlDocument` because the document owns the arena. The
> pattern is always `doc.method(node_id, ...)`.

---

### 6. Serializing to String

**C++:**
```cpp
// Pretty-print (default)
XMLPrinter printer;
doc.Print(&printer);
printf("%s", printer.CStr());

// Compact mode
XMLPrinter compactPrinter(nullptr, true);
doc.Print(&compactPrinter);
printf("%s", compactPrinter.CStr());

// Save to file
doc.SaveFile("output.xml");
```

**Rust:**
```rust
use tinyxml2::XmlPrinter;
use std::fs::File;
use std::io::BufWriter;

// Pretty-print (default)
let mut printer = XmlPrinter::new(false); // compact = false
doc.print(&mut printer);
println!("{}", printer.as_str());

// Compact mode
let mut compact_printer = XmlPrinter::new(true); // compact = true
doc.print(&mut compact_printer);
println!("{}", compact_printer.as_str());

// Save to file
doc.save_file("output.xml")?;

// Or use any std::io::Write implementor
let file = File::create("output.xml")?;
let mut writer = BufWriter::new(file);
doc.write_to(&mut writer)?;
```

> **Key difference:** The Rust printer uses `as_str()` instead of `CStr()` and
> returns a proper `&str`. File output supports any `std::io::Write` implementor,
> not just file paths and `FILE*` pointers.

---

### 7. Error Handling

**C++:**
```cpp
XMLDocument doc;
doc.Parse(xml);

// Check error state
if (doc.Error()) {
    printf("Error ID: %d\n", doc.ErrorID());
    printf("Error: %s\n", doc.ErrorStr());
    printf("Line: %d\n", doc.ErrorLineNum());
    doc.PrintError();     // Print to stderr
    doc.ClearError();     // Reset error state
}

// Error name lookup
const char* name = XMLDocument::ErrorIDToName(XML_ERROR_PARSING);
```

**Rust:**
```rust
use tinyxml2::{XmlDocument, XmlError};

let mut doc = XmlDocument::new();

// Pattern matching on errors
match doc.parse(xml) {
    Ok(()) => println!("Parse successful"),
    Err(XmlError::EmptyDocument) => {
        eprintln!("The document was empty");
    }
    Err(XmlError::MismatchedElement) => {
        eprintln!("Mismatched opening/closing tags");
    }
    Err(e) => {
        // Display trait provides human-readable message
        eprintln!("Parse error: {}", e);

        // Access line number on parse errors
        if let Some(line) = e.line_num() {
            eprintln!("  at line {}", line);
        }

        // Error name
        eprintln!("  error name: {}", e.name());
    }
}

// Idiomatic: use ? operator for propagation
fn process_xml(xml: &str) -> Result<String, XmlError> {
    let mut doc = XmlDocument::new();
    doc.parse(xml)?;
    let root = doc.root_element().ok_or(XmlError::EmptyDocument)?;
    let name = doc.element_name(root)?;
    Ok(name.to_string())
}
```

> **Key difference:** No error state to poll or clear. Errors are values returned
> from functions. The `?` operator, `match`, and combinators (`map`, `and_then`,
> `unwrap_or`) provide more flexible error handling than state-based checking.

---

### 8. Visitor Pattern

**C++:**
```cpp
class MyVisitor : public XMLVisitor {
public:
    bool VisitEnter(const XMLElement& elem, const XMLAttribute* attr) override {
        printf("Enter: %s\n", elem.Name());
        return true; // continue traversal
    }

    bool VisitExit(const XMLElement& elem) override {
        printf("Exit: %s\n", elem.Name());
        return true;
    }

    bool Visit(const XMLText& text) override {
        printf("Text: %s\n", text.Value());
        return true;
    }

    bool Visit(const XMLComment& comment) override {
        printf("Comment: %s\n", comment.Value());
        return true;
    }
};

MyVisitor visitor;
doc.Accept(&visitor);
```

**Rust:**
```rust
use tinyxml2::{XmlDocument, XmlVisitor, XmlAttribute, NodeId};

struct MyVisitor;

impl XmlVisitor for MyVisitor {
    fn visit_enter_element(
        &mut self,
        doc: &XmlDocument,
        id: NodeId,
        _first_attr: Option<&XmlAttribute>,
    ) -> bool {
        if let Ok(name) = doc.element_name(id) {
            println!("Enter: {}", name);
        }
        true // continue traversal
    }

    fn visit_exit_element(
        &mut self,
        doc: &XmlDocument,
        id: NodeId,
    ) -> bool {
        if let Ok(name) = doc.element_name(id) {
            println!("Exit: {}", name);
        }
        true
    }

    fn visit_text(
        &mut self,
        doc: &XmlDocument,
        id: NodeId,
    ) -> bool {
        if let Ok(value) = doc.node_value(id) {
            println!("Text: {}", value);
        }
        true
    }

    fn visit_comment(
        &mut self,
        doc: &XmlDocument,
        id: NodeId,
    ) -> bool {
        if let Ok(value) = doc.node_value(id) {
            println!("Comment: {}", value);
        }
        true
    }
}

let mut visitor = MyVisitor;
doc.accept(&mut visitor);
```

> **Key difference:** Rust uses a trait instead of virtual inheritance. Node data
> is accessed through the `&XmlDocument` reference passed to each method, rather
> than through the node object itself. Default implementations return `true`, so
> you only need to override the methods you care about.

---

### 9. Entity Handling

**C++:**
```cpp
// Entities processed by default
XMLDocument doc;
doc.Parse("<root attr=\"1 &amp; 2\">&lt;hello&gt;</root>");

XMLElement* root = doc.RootElement();
printf("Text: %s\n", root->GetText());     // prints: <hello>
printf("Attr: %s\n", root->Attribute("attr")); // prints: 1 & 2

// Disable entity processing
XMLDocument doc2(false); // processEntities = false
doc2.Parse("<root>&lt;hello&gt;</root>");
root = doc2.RootElement();
printf("Text: %s\n", root->GetText());     // prints: &lt;hello&gt;

// Numeric character references
XMLDocument doc3;
doc3.Parse("<root>&#65;&#x42;</root>");
root = doc3.RootElement();
printf("Text: %s\n", root->GetText());     // prints: AB
```

**Rust:**
```rust
use tinyxml2::{XmlDocument, ParseOptions};

// Entities processed by default
let mut doc = XmlDocument::new();
doc.parse("<root attr=\"1 &amp; 2\">&lt;hello&gt;</root>")?;

let root = doc.root_element().unwrap();
let text = doc.get_text(root)?;                // Some("<hello>")
let attr = doc.attribute(root, "attr")?;       // Some("1 & 2")

// Disable entity processing
let mut doc2 = XmlDocument::new();
doc2.parse_with_options(
    "<root>&lt;hello&gt;</root>",
    ParseOptions::new().process_entities(false),
)?;
let root = doc2.root_element().unwrap();
let text = doc2.get_text(root)?;               // Some("&lt;hello&gt;")

// Numeric character references
let mut doc3 = XmlDocument::new();
doc3.parse("<root>&#65;&#x42;</root>")?;
let root = doc3.root_element().unwrap();
let text = doc3.get_text(root)?;               // Some("AB")
```

> **Key difference:** Entity processing configuration moves from the constructor
> to `ParseOptions`. The behavior is identical: same 5 predefined entities, same
> numeric character reference support.

---

### 10. Whitespace Configuration

**C++:**
```cpp
// Default: preserve whitespace
XMLDocument doc1;
doc1.Parse("<root>  hello  world  </root>");
// Text: "  hello  world  "

// Collapse whitespace
XMLDocument doc2(true, COLLAPSE_WHITESPACE);
doc2.Parse("<root>  hello  world  </root>");
// Text: "hello world"

// Pedantic whitespace
XMLDocument doc3(true, PEDANTIC_WHITESPACE);
doc3.Parse("<root>  hello  world  </root>");
// Text: "  hello  world  "
// (also preserves whitespace-only text nodes between elements)
```

**Rust:**
```rust
use tinyxml2::{XmlDocument, ParseOptions, WhitespaceMode};

// Default: preserve whitespace
let mut doc1 = XmlDocument::new();
doc1.parse("<root>  hello  world  </root>")?;
// Text: "  hello  world  "

// Collapse whitespace
let mut doc2 = XmlDocument::new();
doc2.parse_with_options(
    "<root>  hello  world  </root>",
    ParseOptions::new().whitespace_mode(WhitespaceMode::Collapse),
)?;
// Text: "hello world"

// Pedantic whitespace
let mut doc3 = XmlDocument::new();
doc3.parse_with_options(
    "<root>  hello  world  </root>",
    ParseOptions::new().whitespace_mode(WhitespaceMode::Pedantic),
)?;
// Text: "  hello  world  "
// (also preserves whitespace-only text nodes between elements)
```

> **Key difference:** Whitespace mode is configured via `ParseOptions` builder
> instead of constructor parameters. The `WhitespaceMode` enum replaces the C++
> enum values.

---

## Key Differences Summary

| Concept | TinyXML2 (C++) | tinyxml2-rs (Rust) |
|---|---|---|
| **Error handling** | Error state on document; poll with `Error()`, `ErrorID()` | `Result<T, XmlError>`; use `?` operator |
| **Node references** | Raw pointers (`XMLElement*`) | `NodeId` (index + generation) |
| **Null safety** | Null pointers, manual checks | `Option<NodeId>`, pattern matching |
| **Memory safety** | Dangling pointers → UB | Stale `NodeId` → `Err(InvalidNodeId)` |
| **Node mutation** | Methods on node pointer (`elem->SetName(...)`) | Methods on document (`doc.set_element_name(id, ...)`) |
| **Type overloading** | Function overloading (`SetAttribute` × 8) | Generics (`impl Into<AttrValue>`) |
| **Configuration** | Constructor parameters | Builder-style `ParseOptions` |
| **File I/O** | `FILE*` and `const char*` paths | `std::io::Read/Write` traits and `AsRef<Path>` |
| **Naming** | `PascalCase` methods | `snake_case` methods |
| **Const correctness** | `XMLHandle` / `XMLConstHandle` | Single `XmlHandle` type; borrow checker enforces const |
| **Iteration** | Manual `for` loop with null check | `while let` or iterator adapters |
| **Visitor** | Virtual class inheritance | Trait with default methods |

---

## Common Pitfalls

### 1. Forgetting That Mutations Go Through the Document

❌ **Wrong** (thinking like C++):
```rust
// This won't compile — there's no method on NodeId
// node_id.set_name("new_name");
```

✅ **Correct:**
```rust
doc.set_element_name(node_id, "new_name")?;
```

**Why:** The document owns the arena. All node access and mutation must go through
the `XmlDocument` instance.

### 2. Using Stale NodeIds After Deletion

❌ **Dangerous** (C++ equivalent would be UB):
```rust
let child = doc.first_child_element(root, None).unwrap();
doc.delete_child(root, child)?;

// child is now a stale NodeId
let name = doc.element_name(child); // Returns Err(InvalidNodeId)
```

✅ **Safe pattern:**
```rust
let child = doc.first_child_element(root, None).unwrap();
doc.delete_child(root, child)?;

// Don't use child anymore — rebind if needed
if let Some(new_first) = doc.first_child_element(root, None) {
    let name = doc.element_name(new_first)?;
}
```

**Why:** Generational arena detects stale references, but your code should still
avoid using `NodeId`s after the corresponding node is deleted.

### 3. Expecting Default Values from Attribute Getters

❌ **Wrong** (expecting C++ default-value behavior):
```rust
// C++ returns 0 by default:  elem->IntAttribute("missing", 0)
// Rust returns an error:
let value = doc.int_attribute(elem, "missing")?; // Error: NoAttribute
```

✅ **Correct:**
```rust
// Use unwrap_or for defaults
let value = doc.int_attribute(elem, "missing").unwrap_or(0);

// Or handle explicitly
let value = match doc.int_attribute(elem, "missing") {
    Ok(v) => v,
    Err(XmlError::NoAttribute) => 0,
    Err(e) => return Err(e),
};
```

### 4. Forgetting to Handle Option for Navigation

❌ **Wrong** (assuming nodes exist):
```rust
let child = doc.first_child_element(root, Some("item")).unwrap(); // panics if None
```

✅ **Correct:**
```rust
if let Some(child) = doc.first_child_element(root, Some("item")) {
    let name = doc.element_name(child)?;
    println!("Found: {}", name);
} else {
    println!("No 'item' element found");
}
```

### 5. Mixing Up NodeIds from Different Documents

❌ **Wrong:**
```rust
let mut doc1 = XmlDocument::new();
let mut doc2 = XmlDocument::new();
doc1.parse("<a/>")?;
doc2.parse("<b/>")?;

let root1 = doc1.root_element().unwrap();
// Using root1 with doc2 — wrong document!
let name = doc2.element_name(root1); // Error: InvalidNodeId
```

✅ **Correct:**
```rust
let root1 = doc1.root_element().unwrap();
let root2 = doc2.root_element().unwrap();

// Always use NodeIds with their owning document
let name1 = doc1.element_name(root1)?;
let name2 = doc2.element_name(root2)?;
```

### 6. Not Using the ? Operator

❌ **Verbose:**
```rust
let result = doc.parse(xml);
if result.is_err() {
    return Err(result.unwrap_err());
}
let root = doc.root_element();
if root.is_none() {
    return Err(XmlError::EmptyDocument);
}
let root = root.unwrap();
```

✅ **Idiomatic:**
```rust
doc.parse(xml)?;
let root = doc.root_element().ok_or(XmlError::EmptyDocument)?;
```

### 7. Assuming Mutable Borrow Is Always Needed

❌ **Overly restrictive:**
```rust
fn count_elements(doc: &mut XmlDocument, id: NodeId) -> usize { ... }
```

✅ **Correct — read-only operations only need &:**
```rust
fn count_elements(doc: &XmlDocument, id: NodeId) -> usize { ... }
```

**Why:** Navigation and reading operations only require `&XmlDocument`. Reserve
`&mut XmlDocument` for operations that modify the tree.

---

## Quick Reference: Method Name Mapping

For developers searching for a specific C++ method, here are the most commonly used
mappings:

| C++ | Rust |
|---|---|
| `doc.Parse(xml)` | `doc.parse(xml)?` |
| `doc.RootElement()` | `doc.root_element()` |
| `doc.NewElement("x")` | `doc.new_element("x")` |
| `elem->Name()` | `doc.element_name(id)?` |
| `elem->Attribute("x")` | `doc.attribute(id, "x")?` |
| `elem->SetAttribute("x", val)` | `doc.set_attribute(id, "x", val)?` |
| `elem->GetText()` | `doc.get_text(id)?` |
| `elem->SetText(val)` | `doc.set_text(id, val)?` |
| `elem->FirstChildElement("x")` | `doc.first_child_element(id, Some("x"))` |
| `elem->NextSiblingElement()` | `doc.next_sibling_element(id, None)` |
| `parent->InsertEndChild(child)` | `doc.insert_end_child(parent, child)?` |
| `parent->DeleteChild(child)` | `doc.delete_child(parent, child)?` |
| `doc.Print(&printer)` | `doc.print(&mut printer)` |
| `doc.SaveFile("x.xml")` | `doc.save_file("x.xml")?` |
| `doc.Error()` | (use `Result` from parse) |
| `doc.Accept(&visitor)` | `doc.accept(&mut visitor)` |

For the complete mapping of all methods, see the
[API Compatibility Matrix](./api-matrix.md).
