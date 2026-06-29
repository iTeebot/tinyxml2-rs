# DOM Architecture

## Overview

The tinyxml2-rs DOM (Document Object Model) is built on a **generational arena** — a contiguous `Vec`-backed allocator that stores all XML nodes in a single flat array. Tree structure is encoded via explicit link fields (`parent`, `first_child`, `last_child`, `prev_sibling`, `next_sibling`) stored as `Option<NodeId>` inside each node.

This design mirrors TinyXML2's **document-owns-all** model: the `Document` struct owns the arena and is the sole authority for creating, linking, and destroying nodes. External code interacts with nodes exclusively through lightweight `NodeId` handles — never through references with complex lifetimes or shared ownership.

---

## NodeId

`NodeId` is the fundamental handle type for addressing nodes in the arena.

```rust
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct NodeId {
    index: u32,
    generation: u32,
}
```

### Properties

| Property | Detail |
|---|---|
| Size | 8 bytes (two `u32` fields) |
| Traits | `Copy`, `Clone`, `PartialEq`, `Eq`, `Hash` |
| Debug format | `NodeId(5g3)` — index 5, generation 3 |
| Default | No `Default` impl — `NodeId` is never "null" |
| Niche | `Option<NodeId>` is used for nullable references (parent, siblings) |

### Why `Copy`?

`NodeId` is intentionally `Copy` so it can be passed around freely, stored in multiple places, and compared cheaply. It is a value type — copying a `NodeId` does not copy the node it points to. This matches TinyXML2's use of raw pointers as handles, but without the safety hazards.

### Stale IDs

A `NodeId` becomes **stale** when the node it referenced is deallocated. The arena bumps the slot's generation counter on deallocation, so any subsequent lookup with the old `NodeId` will see a generation mismatch and return `None` — no panic, no undefined behavior.

```
Before dealloc:  slot[5].generation = 3   NodeId { index: 5, generation: 3 } → ✅ match
After dealloc:   slot[5].generation = 4   NodeId { index: 5, generation: 3 } → ❌ mismatch → None
```

---

## NodeData (Planned)

Each slot in the arena stores a `NodeData` value:

```rust
pub struct NodeData {
    /// What kind of XML node this is.
    pub kind: NodeKind,

    /// The node's string value (element name, text content, comment body, etc.).
    pub value: String,

    /// Source line number (1-indexed) where this node was parsed, or 0 if programmatically created.
    pub line_num: u32,

    // ── Tree links ──────────────────────────────────────────
    pub parent:       Option<NodeId>,
    pub first_child:  Option<NodeId>,
    pub last_child:   Option<NodeId>,
    pub prev_sibling: Option<NodeId>,
    pub next_sibling: Option<NodeId>,
}
```

---

## NodeKind (Planned)

```rust
pub enum NodeKind {
    /// The root document node. Exactly one per arena.
    Document,

    /// An XML element: `<name attr="val">children</name>`.
    Element,

    /// A text node (character data between elements).
    Text,

    /// A comment: `<!-- body -->`.
    Comment,

    /// An XML declaration: `<?xml version="1.0"?>`.
    Declaration,

    /// An unknown or unrecognized construct (e.g., `<!DOCTYPE ...>`).
    Unknown,
}
```

This enum mirrors TinyXML2's node type hierarchy (`XMLDocument`, `XMLElement`, `XMLText`, `XMLComment`, `XMLDeclaration`, `XMLUnknown`) but uses a flat enum + data struct instead of an inheritance tree.

---

## Tree Linkage Model

The DOM tree is encoded as an **intrusive linked structure** using five `Option<NodeId>` fields per node:

```
                    ┌─────────┐
                    │ Document│
                    │ (root)  │
                    └────┬────┘
                         │ first_child / last_child
                         ▼
                    ┌─────────┐   next_sibling   ┌─────────┐
                    │ Element │ ───────────────▶  │ Comment │
                    │  "root" │ ◀───────────────  │  "..."  │
                    └────┬────┘   prev_sibling    └─────────┘
                         │
            ┌────────────┼────────────┐
            ▼            │            ▼
       ┌─────────┐      │       ┌─────────┐
       │  Text   │      │       │ Element │
       │ "hello" │      │       │ "child" │
       └─────────┘      │       └─────────┘
                         │
                    parent (all children
                     point back up)
```

### Link semantics

| Link | Meaning |
|---|---|
| `parent` | The containing node. `None` only for the root `Document` node. |
| `first_child` | The first child in document order. `None` if the node has no children. |
| `last_child` | The last child in document order. Enables O(1) `insert_end_child`. |
| `prev_sibling` | The preceding sibling. `None` for the first child. |
| `next_sibling` | The following sibling. `None` for the last child. |

### Invariants

1. If `A.first_child == Some(B)`, then `B.parent == Some(A)`.
2. If `A.first_child == Some(B)`, then `B.prev_sibling == None`.
3. If `A.last_child == Some(B)`, then `B.next_sibling == None`.
4. Siblings form a doubly-linked list: `B.next_sibling == Some(C)` ⟺ `C.prev_sibling == Some(B)`.
5. The `Document` node's `parent` is always `None`.

---

## Document as Owner

The `Document` struct owns the arena and provides factory methods for creating nodes:

```rust
pub struct Document {
    arena: Arena<NodeData>,
    root: NodeId,              // The Document node — always at index 0
    has_bom: bool,
    options: ParseOptions,
}
```

### Factory Methods (Planned)

```rust
impl Document {
    pub fn new_element(&mut self, name: &str) -> NodeId;
    pub fn new_text(&mut self, text: &str) -> NodeId;
    pub fn new_comment(&mut self, comment: &str) -> NodeId;
    pub fn new_declaration(&mut self) -> NodeId;
    pub fn new_unknown(&mut self, content: &str) -> NodeId;
}
```

**Key design rule:** Nodes can only be created through the `Document`'s factory methods. There is no public constructor for `NodeData`. This ensures:

1. Every node is allocated in the arena.
2. The document always knows about every node.
3. `NodeKind`-specific initialization logic is centralized.

### Tree Manipulation (Planned)

```rust
impl Document {
    pub fn insert_first_child(&mut self, parent: NodeId, child: NodeId) -> Result<(), XmlError>;
    pub fn insert_end_child(&mut self, parent: NodeId, child: NodeId) -> Result<(), XmlError>;
    pub fn insert_after(&mut self, after: NodeId, child: NodeId) -> Result<(), XmlError>;
    pub fn delete_node(&mut self, node: NodeId);
}
```

---

## Deletion Semantics

When a node is deleted:

1. **Unlink from tree:** Remove from parent's child list and sibling chain.
2. **Recursive child deletion:** All descendants are also deleted (matching TinyXML2 behavior).
3. **Arena dealloc:** The slot is swapped to `Vacant`, generation is bumped, index is pushed to free list.
4. **Stale IDs:** Any `NodeId` values still held by external code will now fail generation checks and return `None` on lookup.

```
delete_node(id):
    for each child of id:
        delete_node(child)       // recursive
    unlink(id)                   // fix sibling/parent pointers
    arena.dealloc(id)            // Occupied → Vacant, gen++, push free list
```

---

## Why Not `Rc<RefCell<T>>`?

A common Rust approach for tree structures is `Rc<RefCell<Node>>` with `Weak` back-references. This was rejected for several reasons:

| Concern | `Rc<RefCell<T>>` | Arena |
|---|---|---|
| **Runtime overhead** | Reference counting on every clone/drop. `RefCell` borrow checks on every access. | Direct `Vec` indexing. Generation check is a single `u32` comparison. |
| **Cache locality** | Nodes scattered across heap. Traversal causes cache misses. | Nodes contiguous in `Vec`. Sequential traversal is cache-friendly. |
| **Ref-count churn** | Tree manipulation (insert/remove) requires careful `Rc`/`Weak` bookkeeping. Cycles with `Weak` are fragile. | `NodeId` is `Copy`. No reference counting at all. |
| **Ownership model** | Shared ownership — any `Rc` holder can keep a node alive. | Document-owns-all — the arena is the single owner, matching TinyXML2. |
| **Bulk deallocation** | Must traverse and drop each `Rc` individually. | Drop the arena → all nodes freed in one `Vec::drop`. |

---

## Why Not Raw Pointers?

Raw pointers (`*mut Node`) would replicate TinyXML2's C++ approach directly. This was rejected because:

1. **Requires `unsafe`** — Every pointer dereference, every tree traversal, every mutation would need `unsafe` blocks, defeating the purpose of a Rust rewrite.
2. **Use-after-free risk** — Deleting a node and then accessing a stored pointer is undefined behavior. The generational arena detects this safely.
3. **No compiler verification** — The borrow checker cannot reason about raw pointer lifetimes. Bugs become runtime UB instead of compile-time errors.
4. **Ecosystem friction** — Safe Rust libraries cannot call into `unsafe`-heavy APIs without wrapping them. An arena-based API composes naturally with the Rust ecosystem.

---

## Why Generational Arena?

The generational arena is the best fit for this project's requirements:

| Requirement | How the Arena Satisfies It |
|---|---|
| **Safe** | No `unsafe` code. Stale IDs return `None`, not UB. |
| **Cache-friendly** | `Vec<Slot<T>>` is contiguous memory. |
| **O(1) operations** | Alloc, dealloc, and lookup are all O(1). |
| **Document-owns-all** | The `Document` owns the `Arena`. Dropping the document drops all nodes. |
| **Stale ID detection** | Generation mismatch → `None`. External code can safely hold old `NodeId` values. |
| **No lifetimes in API** | `NodeId` is `Copy + 'static`. No lifetime parameters leak into the public API. |
| **Matches TinyXML2** | TinyXML2's `MemPool` provides similar O(1) fixed-block allocation; the arena is the Rust-idiomatic equivalent. |

---

## Attribute Storage (Planned)

Element attributes will be stored as a **singly-linked list** per element, matching TinyXML2's approach:

```rust
pub struct Attribute {
    pub name: String,
    pub value: String,
    pub next: Option<AttributeId>,  // link to next attribute on same element
}
```

Design considerations:

- **Singly-linked list** is sufficient because attributes are typically accessed sequentially or by name scan. Random access by index is not a common operation.
- Attributes may be stored in the same arena as nodes (using a separate `Arena<Attribute>`) or inline in the `NodeData` via a `Vec<Attribute>`. The final design will be determined by benchmarking.
- **Order preservation:** Attributes maintain insertion order, matching TinyXML2 and XML specification requirements.
- **Typed queries:** Methods like `query_int_attribute("width")` will parse the string value and return `Result<i32, XmlError>`, mirroring TinyXML2's `QueryIntAttribute`.
