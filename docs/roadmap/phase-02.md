# Phase 2: DOM Core

> **Status:** ✅ COMPLETED  
> **Estimated Complexity:** HIGH (~3000 LOC)  
> **Dependencies:** Phase 1 (arena, error, entity)  
> **Milestone:** `v0.0.2-alpha` internal

---

## Objectives

Implement the complete Document Object Model tree structure. This is the
backbone of the library — every other phase (parser, writer, visitor, FFI)
operates on the DOM. The API surface must mirror TinyXML2's `XMLDocument`,
`XMLElement`, `XMLText`, `XMLComment`, `XMLDeclaration`, and `XMLUnknown`
classes with Rust-idiomatic adaptations.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────┐
│                  Document                        │
│  ┌─────────┐  ┌───────────────────────────────┐ │
│  │  Arena   │  │  Root NodeId (Document node)  │ │
│  │<NodeData>│  └───────────────────────────────┘ │
│  └─────────┘                                     │
│       │                                          │
│       ▼                                          │
│  ┌──────────────────────────────────────────┐    │
│  │ NodeData {                                │    │
│  │   kind: NodeKind,                         │    │
│  │   parent: Option<NodeId>,                 │    │
│  │   first_child: Option<NodeId>,            │    │
│  │   last_child: Option<NodeId>,             │    │
│  │   prev_sibling: Option<NodeId>,           │    │
│  │   next_sibling: Option<NodeId>,           │    │
│  │   line_num: u32,                          │    │
│  │ }                                         │    │
│  └──────────────────────────────────────────┘    │
└─────────────────────────────────────────────────┘
```

---

## Deliverables

### 1. Node Types — `node.rs`

| Type | Description |
|------|-------------|
| `NodeKind` enum | `Document`, `Element(ElementData)`, `Text(TextData)`, `Comment(String)`, `Declaration(ElementData)`, `Unknown(String)` |
| `NodeData` struct | Tree linkage fields + `NodeKind` + source line number |
| `ElementData` struct | Tag name (`String`) + attribute list (`Vec<Attribute>`) |
| `TextData` struct | Text content (`String`) + `is_cdata: bool` flag |
| `Attribute` struct | Name/value pair (`String`, `String`) |

### 2. Document Struct — `document.rs`

| Item | Description |
|------|-------------|
| `Document` | Owns `Arena<NodeData>` + root `NodeId` + parse state |
| `Document::new()` | Create empty document with root Document node |
| `Document::root()` | Returns `NodeId` of the document root |
| `Document::error()` | Returns current `XmlError` state |
| `Document::error_line()` | Line number where last error occurred |
| `Document::clear()` | Reset document to empty state, deallocate all nodes |

### 3. Factory Methods

All factory methods allocate in the document's arena and return `NodeId`:

| Method | Description |
|--------|-------------|
| `new_element(name: &str) -> NodeId` | Create a detached Element node |
| `new_text(text: &str) -> NodeId` | Create a detached Text node |
| `new_comment(text: &str) -> NodeId` | Create a detached Comment node |
| `new_declaration(decl: &str) -> NodeId` | Create a detached Declaration node |
| `new_unknown(text: &str) -> NodeId` | Create a detached Unknown node |
| `new_cdata(text: &str) -> NodeId` | Create a detached CData Text node |

### 4. Tree Modification Methods

These methods maintain all invariants (parent/child/sibling linkage):

| Method | Description |
|--------|-------------|
| `insert_end_child(parent, child)` | Append child as last child of parent |
| `insert_first_child(parent, child)` | Insert child as first child of parent |
| `insert_after_child(after, child)` | Insert child as next sibling of `after` |
| `delete_child(parent, child)` | Remove child from parent, deallocate recursively |
| `delete_children(parent)` | Remove and deallocate all children of parent |
| `delete_node(node)` | Unlink from parent (if any) and deallocate recursively |

**Invariants maintained by all mutations:**
- `parent.first_child` and `parent.last_child` stay consistent
- Sibling chain is doubly-linked and cycle-free
- Detached nodes have `parent = None`
- Deallocated nodes' `NodeId` generations are incremented

### 5. Navigation Methods

| Method | Returns | Description |
|--------|---------|-------------|
| `parent(node)` | `Option<NodeId>` | Parent node |
| `first_child(node)` | `Option<NodeId>` | First child |
| `last_child(node)` | `Option<NodeId>` | Last child |
| `prev_sibling(node)` | `Option<NodeId>` | Previous sibling |
| `next_sibling(node)` | `Option<NodeId>` | Next sibling |
| `first_child_element(node, name?)` | `Option<NodeId>` | First child that is an Element, optionally filtered by tag name |
| `last_child_element(node, name?)` | `Option<NodeId>` | Last child Element |
| `next_sibling_element(node, name?)` | `Option<NodeId>` | Next sibling Element |
| `prev_sibling_element(node, name?)` | `Option<NodeId>` | Previous sibling Element |
| `root_element()` | `Option<NodeId>` | First Element child of the Document root |

### 6. Attribute API

| Method | Description |
|--------|-------------|
| `attribute(element, name) -> Option<&str>` | Get attribute value by name |
| `set_attribute(element, name, value)` | Set attribute (insert or update) |
| `delete_attribute(element, name)` | Remove attribute by name |
| `first_attribute(element) -> Option<&Attribute>` | First attribute in order |
| `attribute_count(element) -> usize` | Number of attributes |
| `find_attribute(element, name) -> Option<&Attribute>` | Find by name |
| `iterate_attributes(element) -> impl Iterator` | Iterate all attributes |

### 7. Typed Value Access

Mirrors TinyXML2's `QueryIntValue`, `QueryBoolValue`, etc.:

| Method | Return | Description |
|--------|--------|-------------|
| `query_int_attribute(el, name)` | `Result<i32>` | Parse attribute as `i32` |
| `query_unsigned_attribute(el, name)` | `Result<u32>` | Parse as `u32` |
| `query_int64_attribute(el, name)` | `Result<i64>` | Parse as `i64` |
| `query_bool_attribute(el, name)` | `Result<bool>` | Parse as `bool` (`"true"`/`"1"` → true) |
| `query_double_attribute(el, name)` | `Result<f64>` | Parse as `f64` |
| `query_float_attribute(el, name)` | `Result<f32>` | Parse as `f32` |
| `int_attribute(el, name, default)` | `i32` | With default fallback |
| `bool_attribute(el, name, default)` | `bool` | With default fallback |
| `double_attribute(el, name, default)` | `f64` | With default fallback |
| `float_attribute(el, name, default)` | `f32` | With default fallback |

**Boolean parsing rules** (matching TinyXML2):
- `"true"` and `"1"` → `true`
- `"false"` and `"0"` → `false`
- Anything else → `Err(XmlError::XmlErrorParsingAttribute)`

### 8. Text Content Access

| Method | Description |
|--------|-------------|
| `get_text(element) -> Option<&str>` | Text content of first Text child |
| `set_text(element, text)` | Set/replace first Text child |
| `query_int_text(element)` | Parse text content as `i32` |
| `query_bool_text(element)` | Parse text content as `bool` |
| `query_double_text(element)` | Parse text content as `f64` |
| `int_text(element, default)` | With default fallback |
| `bool_text(element, default)` | With default fallback |

### 9. Clone Operations

| Method | Description |
|--------|-------------|
| `deep_clone(node) -> NodeId` | Recursive clone of node and all descendants |
| `shallow_clone(node) -> NodeId` | Clone node only (no children) |

---

## Design Decisions

### Why `NodeId` handles instead of `&Node` references?

Rust's borrow checker prevents holding `&mut` references to tree nodes while
modifying the tree. The `NodeId` indirection through the arena solves this
without `unsafe` code. All operations go through `Document` methods that
mediate arena access.

### Why `Vec<Attribute>` instead of `HashMap`?

- XML attribute order is significant for serialization fidelity
- Most elements have ≤5 attributes — linear scan is faster than hash lookup
- Matches TinyXML2's linked-list attribute storage semantics
- Preserves insertion order for round-trip printing

### Why separate `TextData.is_cdata`?

TinyXML2 tracks whether text was originally in a `<![CDATA[...]]>` section
to preserve it on output. A single `Text` node kind with a flag is simpler
than a separate `CData` variant and matches TinyXML2's design.

---

## Estimated Test Plan

| Category | Est. Tests | Description |
|----------|-----------|-------------|
| Factory methods | 12 | One per node type, verify kind and content |
| Tree structure | 25 | Insert/delete in various orders, verify linkage |
| Navigation | 20 | All navigation methods, empty/single/multi-child trees |
| Attributes | 18 | Set/get/delete/iterate, duplicate handling |
| Typed values | 24 | All types × valid/invalid/edge-case inputs |
| Deep clone | 8 | Clone trees of depth 1..5, verify independence |
| Clear | 4 | Clear non-empty document, verify arena state |
| Invariants | 10 | Deliberate misuse attempts, verify no panics |

**Estimated Total:** ~120 tests

---

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Attribute storage design | API ergonomics | Benchmark `Vec` vs `SmallVec` vs `HashMap`; start with `Vec`, optimize later |
| Typed conversion semantics | Compatibility | Port TinyXML2's exact parsing logic; test against C++ output |
| Tree invariant bugs | Correctness | Extensive insert/delete sequences with invariant assertions |
| Arena memory growth | Performance | Monitor with benchmarks; consider `clear()` + arena reset |

---

## Acceptance Criteria

- [x] All 6 factory methods create correct node types
- [x] Tree invariants hold after any sequence of insert/delete operations
- [x] Navigation methods return correct nodes for all tree topologies
- [x] Typed attribute conversions match TinyXML2 for all edge cases
- [x] `deep_clone` produces independent subtrees (mutations don't cross)
- [x] Generation checks reject stale `NodeId` after `delete_node`
- [x] `clear()` resets document to initial state
- [x] All tests pass with zero warnings

---

## File Plan

| File | Responsibility |
|------|---------------|
| `tinyxml2/src/node.rs` | `NodeKind`, `NodeData`, `ElementData`, `TextData`, `Attribute` |
| `tinyxml2/src/document.rs` | `Document` struct, factory methods, tree modification |
| `tinyxml2/src/navigate.rs` | Navigation methods (may be in `document.rs`) |
| `tinyxml2/src/typed.rs` | Typed value access helpers |
| `tinyxml2/src/tests/dom_tests.rs` | DOM test suite |

---

## Previous Phase

← [Phase 1: Foundation & Infrastructure](./phase-01.md)

## Next Phase

→ [Phase 3: XML Parser](./phase-03.md)
