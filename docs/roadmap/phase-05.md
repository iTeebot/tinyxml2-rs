# Phase 5: Visitor Pattern & Ergonomic API

> **Status:** ✅ COMPLETED  
> **Estimated Complexity:** MEDIUM (~1200 LOC)  
> **Dependencies:** Phase 4 (writer — Printer as Visitor implementation)  
> **Milestone:** `v0.1.15`

---

## Objectives

Implement the Visitor pattern for DOM traversal (matching TinyXML2's
`XMLVisitor`) and provide Rust-idiomatic ergonomic wrappers — `Handle` types
for null-safe navigation chains, `Ref` wrappers for convenient access, and
iterator adapters for `for`-loop traversal. This phase bridges TinyXML2's
C++ patterns with Rust's type system and iterator protocol.

---

## Architecture Overview

```
┌──────────────────────────────────────────────────────────────┐
│                     Ergonomic API Layers                      │
│                                                              │
│  Layer 3: Iterators                                          │
│  ┌────────────────────────────────────────────────────────┐  │
│  │  Children(node)  ChildElements(node)  Siblings(node)   │  │
│  │  Attributes(element)  Descendants(node)                │  │
│  └────────────────────────────────────────────────────────┘  │
│                                                              │
│  Layer 2: Handle / Ref Wrappers                              │
│  ┌────────────────────────────────────────────────────────┐  │
│  │  Handle<'a>  HandleMut<'a>  ElementRef<'a>  NodeRef<'a>│  │
│  └────────────────────────────────────────────────────────┘  │
│                                                              │
│  Layer 1: Visitor Trait                                       │
│  ┌────────────────────────────────────────────────────────┐  │
│  │  Visitor trait  +  Document::accept()  traversal        │  │
│  └────────────────────────────────────────────────────────┘  │
│                                                              │
│  Layer 0: Raw DOM (Phase 2)                                  │
│  ┌────────────────────────────────────────────────────────┐  │
│  │  Document + Arena + NodeId + NodeData                   │  │
│  └────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
```

---

## Deliverables

### 1. Visitor Trait — `visitor.rs`

The `Visitor` trait mirrors TinyXML2's `XMLVisitor` with 8 callback methods.
All methods have default implementations that return `true` (continue
traversal).

```rust
pub trait Visitor {
    /// Called when visiting a Document node.
    fn visit_document(&mut self, doc: &Document) -> bool { true }

    /// Called when entering an Element node (before children).
    fn visit_enter_element(&mut self, doc: &Document, element: NodeId) -> bool { true }

    /// Called when leaving an Element node (after children).
    fn visit_exit_element(&mut self, doc: &Document, element: NodeId) -> bool { true }

    /// Called when visiting a Text node.
    fn visit_text(&mut self, doc: &Document, text: NodeId) -> bool { true }

    /// Called when visiting a Comment node.
    fn visit_comment(&mut self, doc: &Document, comment: NodeId) -> bool { true }

    /// Called when visiting a Declaration node.
    fn visit_declaration(&mut self, doc: &Document, declaration: NodeId) -> bool { true }

    /// Called when visiting an Unknown node.
    fn visit_unknown(&mut self, doc: &Document, unknown: NodeId) -> bool { true }

    /// Called when entering the Document (before any children).
    fn visit_enter_document(&mut self, doc: &Document) -> bool { true }

    /// Called when exiting the Document (after all children).
    fn visit_exit_document(&mut self, doc: &Document) -> bool { true }
}
```

**Traversal semantics:**
- Depth-first, pre-order traversal
- Returning `false` from any method stops traversal immediately
- `visit_enter_element` is called before children; `visit_exit_element` after
- Leaf nodes (Text, Comment, Declaration, Unknown) get a single visit call

### 2. Document Accept Method

| Method | Description |
|--------|-------------|
| `Document::accept(visitor: &mut dyn Visitor) -> bool` | Traverse the entire DOM, dispatching to visitor callbacks |
| `Document::accept_node(node: NodeId, visitor: &mut dyn Visitor) -> bool` | Traverse a subtree starting from a specific node |

### 3. Printer as Visitor

Re-implement `Printer` DOM serialization as a `Visitor` implementation. This
demonstrates the pattern and ensures the Visitor API is sufficient for
real-world use:

```rust
impl Visitor for Printer {
    fn visit_enter_element(&mut self, doc: &Document, el: NodeId) -> bool {
        // Emit opening tag with attributes
    }
    fn visit_exit_element(&mut self, doc: &Document, el: NodeId) -> bool {
        // Emit closing tag
    }
    fn visit_text(&mut self, doc: &Document, text: NodeId) -> bool {
        // Emit text content
    }
    // ... etc
}
```

### 4. Handle Types — `handle.rs`

Null-safe navigation wrappers that enable fluent chaining without explicit
`Option` matching. Inspired by TinyXML2's handle API.

```rust
/// Immutable handle for fluent DOM navigation.
/// All navigation methods return another Handle, enabling chaining.
/// A "null" handle (wrapping None) propagates through the chain.
pub struct Handle<'a> {
    doc: &'a Document,
    node: Option<NodeId>,
}

/// Mutable handle for fluent DOM modification.
pub struct HandleMut<'a> {
    doc: &'a mut Document,
    node: Option<NodeId>,
}
```

| Method | Description |
|--------|-------------|
| `Handle::new(doc, node) -> Handle` | Create from document + optional NodeId |
| `.first_child() -> Handle` | Navigate to first child |
| `.last_child() -> Handle` | Navigate to last child |
| `.parent() -> Handle` | Navigate to parent |
| `.next_sibling() -> Handle` | Navigate to next sibling |
| `.prev_sibling() -> Handle` | Navigate to previous sibling |
| `.first_child_element(name?) -> Handle` | First child Element |
| `.next_sibling_element(name?) -> Handle` | Next sibling Element |
| `.to_element() -> Option<NodeId>` | Extract NodeId if it's an Element |
| `.to_text() -> Option<NodeId>` | Extract NodeId if it's a Text node |
| `.to_node() -> Option<NodeId>` | Extract raw NodeId |
| `.is_null() -> bool` | Check if the handle is null |
| `.text() -> Option<&str>` | Get text content directly |
| `.attribute(name) -> Option<&str>` | Get attribute value directly |

**Key property:** Calling navigation on a null Handle returns another null
Handle — the chain never panics.

### 5. Ref Wrappers — `refs.rs`

Typed, lifetime-bound references for convenient access to specific node types:

```rust
/// A borrowed reference to a node in the DOM.
pub struct NodeRef<'a> {
    doc: &'a Document,
    id: NodeId,
}

/// A borrowed reference to an Element node.
pub struct ElementRef<'a> {
    doc: &'a Document,
    id: NodeId,
}
```

| `NodeRef` Methods | Description |
|-------------------|-------------|
| `.kind() -> &NodeKind` | Node type |
| `.parent() -> Option<NodeRef>` | Parent as NodeRef |
| `.children() -> Children<'a>` | Iterator over children |
| `.as_element() -> Option<ElementRef>` | Downcast to ElementRef |
| `.value() -> &str` | Node value (text content, comment text, etc.) |
| `.line() -> u32` | Source line number |

| `ElementRef` Methods | Description |
|----------------------|-------------|
| `.name() -> &str` | Tag name |
| `.attribute(name) -> Option<&str>` | Get attribute |
| `.attributes() -> Attributes<'a>` | Iterate attributes |
| `.children() -> Children<'a>` | Iterate children |
| `.child_elements() -> ChildElements<'a>` | Iterate child elements |
| `.text() -> Option<&str>` | First text child content |
| `.int_attribute(name, default) -> i32` | Typed attribute access |
| `.bool_attribute(name, default) -> bool` | Typed attribute access |
| `.double_attribute(name, default) -> f64` | Typed attribute access |

### 6. Iterator Adapters — `iter.rs`

Standard Rust iterators for DOM traversal:

| Iterator | Yields | Description |
|----------|--------|-------------|
| `Children<'a>` | `NodeRef<'a>` | All direct children of a node |
| `ChildElements<'a>` | `ElementRef<'a>` | All direct child Elements (optionally name-filtered) |
| `Siblings<'a>` | `NodeRef<'a>` | All following siblings |
| `SiblingElements<'a>` | `ElementRef<'a>` | All following sibling Elements |
| `Attributes<'a>` | `(&'a str, &'a str)` | Name-value pairs of an Element's attributes |
| `Descendants<'a>` | `NodeRef<'a>` | Depth-first traversal of all descendants |

All iterators implement `Iterator`, `DoubleEndedIterator` (where possible),
and `FusedIterator`.

---

## Usage Examples

### Visitor: Extract All Text Content

```rust
struct TextExtractor {
    texts: Vec<String>,
}

impl Visitor for TextExtractor {
    fn visit_text(&mut self, doc: &Document, text: NodeId) -> bool {
        if let Some(content) = doc.get_text_content(text) {
            self.texts.push(content.to_string());
        }
        true
    }
}
```

### Handle: Fluent Navigation

```rust
let value = Handle::new(&doc, doc.root())
    .first_child_element(Some("config"))
    .first_child_element(Some("database"))
    .attribute("host");
// value is Some("localhost") or None — never panics
```

### Iterators: Process Elements

```rust
let root = doc.root_element_ref().unwrap();
for child in root.child_elements() {
    println!("{}: {}", child.name(), child.text().unwrap_or(""));
}
```

---

## Estimated Test Plan

| Category | Est. Tests | Description |
|----------|-----------|-------------|
| Visitor traversal | 12 | Full document, subtree, early termination |
| Visitor dispatch | 8 | Correct callback for each node type |
| Printer-as-Visitor | 6 | Output matches direct serialization |
| Handle navigation | 15 | All navigation methods, null propagation |
| Handle extraction | 8 | `to_element`, `to_text`, `text()`, `attribute()` |
| NodeRef | 10 | All methods, type casting |
| ElementRef | 12 | Attributes, children, typed access |
| Children iterator | 8 | Empty, single, multiple, forward + reverse |
| ChildElements iterator | 6 | Name filtering, mixed node types |
| Siblings iterator | 6 | From various positions |
| Attributes iterator | 5 | Empty, single, multiple attributes |
| Descendants iterator | 6 | Deep trees, wide trees |

**Estimated Total:** ~102 tests

---

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Borrow checker conflicts in Handle | API design | Handle borrows `&Document`, not individual nodes |
| Iterator invalidation on mutation | Safety | Iterators hold `&Document` — mutation requires separate scope |
| Visitor API sufficiency | Completeness | Validate by implementing Printer as Visitor |
| Performance of Ref wrappers | Overhead | Zero-cost — just `(doc_ref, NodeId)` tuples |

---

## Acceptance Criteria

- [x] Visitor trait correctly dispatches to all 8 callback methods
- [x] `Document::accept()` performs correct depth-first traversal
- [x] Returning `false` from any visitor method halts traversal
- [x] Printer-as-Visitor produces identical output to direct serialization
- [x] Handle null-propagation works for all navigation chains
- [x] All iterators yield elements in correct order
- [x] `DoubleEndedIterator` works for Children and Attributes
- [x] ElementRef typed access matches Document-level methods
- [x] All tests pass with zero warnings

---

## File Plan

| File | Responsibility |
|------|---------------|
| `tinyxml2/src/visitor.rs` | `Visitor` trait, `accept()` traversal engine |
| `tinyxml2/src/handle.rs` | `Handle<'a>`, `HandleMut<'a>` |
| `tinyxml2/src/refs.rs` | `NodeRef<'a>`, `ElementRef<'a>` |
| `tinyxml2/src/iter.rs` | All iterator types |
| `tinyxml2/src/printer.rs` | `Visitor` impl for `Printer` (addition) |
| `tinyxml2/src/tests/visitor_tests.rs` | Visitor test suite |
| `tinyxml2/src/tests/handle_tests.rs` | Handle test suite |
| `tinyxml2/src/tests/iter_tests.rs` | Iterator test suite |

---

## Previous Phase

← [Phase 4: Writer/Serializer](./phase-04.md)

## Next Phase

→ [Phase 6: C API Layer](./phase-06.md)
