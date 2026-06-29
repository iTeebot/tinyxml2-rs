# ADR-0002: Memory Model

| Field   | Value                |
|---------|----------------------|
| Status  | **Accepted**         |
| Date    | 2026-06-01           |
| Authors | tinyxml2-rs maintainers |

---

## Context

Rust's ownership model — single owner, no aliasing with mutation — makes traditional
pointer-based tree structures (as used in C++ TinyXML2) impossible to express directly
in safe Rust. The C++ TinyXML2 uses a document-owns-all pattern where:

- The `XMLDocument` owns all memory for all nodes
- Nodes contain raw pointers to parent, first/last child, prev/next sibling
- Nodes are allocated from a memory pool within the document
- Deleting the document frees all nodes

We need a tree structure for the DOM that is:

1. **Safe** — no `unsafe` code required for core operations
2. **Efficient** — O(1) node access, cache-friendly layout
3. **Semantically correct** — matches TinyXML2's document-owns-all ownership model
4. **Stale-reference safe** — detects use of removed/invalid node references

## Problem Statement

How do we implement a tree structure for the DOM in safe Rust that preserves TinyXML2's
document-owns-all semantics?

## Alternatives Considered

### Alternative 1: `Rc<RefCell<T>>`

Use reference-counted smart pointers with interior mutability for each node.

```rust
type NodeRef = Rc<RefCell<Node>>;

struct Node {
    parent: Option<Weak<RefCell<Node>>>,
    children: Vec<Rc<RefCell<Node>>>,
    // ...
}
```

**Pros:**
- Fully safe, no `unsafe` required
- Familiar pattern in Rust
- Supports shared ownership naturally

**Cons:**
- High overhead: reference count increment/decrement on every clone/drop
- Runtime borrow checking: `borrow()` / `borrow_mut()` panics if violated
- Poor cache locality: nodes scattered across the heap
- Doesn't match document-owns-all: ownership is distributed across `Rc` pointers
- `Weak` references for parent links add complexity
- Difficult to iterate efficiently
- Memory fragmentation over time

### Alternative 2: Raw Pointers with `unsafe`

Mirror the C++ approach directly using raw pointers.

```rust
struct Node {
    parent: *mut Node,
    first_child: *mut Node,
    next_sibling: *mut Node,
    // ...
}
```

**Pros:**
- Fastest possible performance
- Most flexible — can directly mirror C++ structure
- Minimal memory overhead

**Cons:**
- Requires pervasive `unsafe` — defeats the purpose of using Rust
- Use-after-free risk if pointers dangle
- No stale reference detection
- Doesn't compose well with safe Rust code
- Burden of proof for soundness falls entirely on the developer
- Extremely difficult to audit and maintain

### Alternative 3: Generational Arena with Indices

Store all nodes in a contiguous `Vec` (arena). Reference nodes by `NodeId` — a
lightweight index paired with a generation counter.

```rust
struct NodeId {
    index: u32,
    generation: u32,
}

struct Arena<T> {
    entries: Vec<Slot<T>>,
    generations: Vec<u32>,
    free_list: Vec<u32>,
}
```

**Pros:**
- Fully safe — no `unsafe` required
- O(1) node access by index
- Stale ID detection via generation mismatch
- Contiguous memory → excellent cache locality
- Matches document-owns-all perfectly: arena (document) owns all data
- `NodeId` is `Copy` — lightweight 8-byte handle
- Deterministic deallocation (drop the arena, everything is freed)
- Simple to implement and reason about

**Cons:**
- Slots are not immediately reclaimed (reused via free list)
- Extra 4 bytes per slot for generation counter
- IDs are not globally unique across different arenas (documents)
- Slightly more indirection than raw pointers

### Alternative 4: External Crate (indextree, petgraph)

Use a pre-built arena or graph library from the Rust ecosystem.

**Pros:**
- No implementation effort for the data structure itself
- Battle-tested code

**Cons:**
- May not match TinyXML2's exact semantics (e.g., insertion order, sibling navigation)
- Adds a dependency — conflicts with zero-dependency goal
- Limited control over memory layout and allocation strategy
- API may not support all operations needed for TinyXML2 compatibility
- Version churn risk for a foundational component

## Decision

**Option 3 — Custom generational arena with index-based node IDs.**

## Reasoning

The generational arena is the ideal fit for this project:

1. **Document-owns-all alignment.** TinyXML2's memory model is fundamentally arena-based:
   the document owns a pool, all nodes live in that pool, and destroying the document
   frees everything. A generational arena maps directly to this model.

2. **Safety without overhead.** Unlike `Rc<RefCell<T>>`, there are no reference counts to
   maintain and no runtime borrow checks to panic on. Unlike raw pointers, there is no
   `unsafe` code. The generation counter provides use-after-free detection at negligible
   cost.

3. **Cache friendliness.** Nodes are stored contiguously in a `Vec`, which is optimal for
   modern CPU cache hierarchies. Tree traversals touch sequential memory rather than
   chasing heap pointers.

4. **Zero dependencies.** A custom implementation avoids external crate dependencies,
   keeping the dependency tree clean and the crate easy to audit.

5. **Full control.** A custom arena allows us to tailor the API exactly to TinyXML2's
   needs — specific insertion patterns, removal semantics, and traversal orders.

## Consequences

### Arena Structure

```
Arena<T>
├── entries: Vec<Slot<T>>    // Contiguous node storage
│   ├── Slot::Occupied(T)    // Live node
│   └── Slot::Vacant(next)   // Free slot → next free index
├── generations: Vec<u32>    // Generation counter per slot
└── free_list_head: Option<u32>  // Head of free list
```

### NodeId Properties

| Property | Value |
|----------|-------|
| Size | 8 bytes (u32 index + u32 generation) |
| Traits | `Copy`, `Clone`, `Eq`, `Hash`, `Debug` |
| Lifetime | Valid until the node is removed from the arena |
| Staleness | Detected by generation mismatch → returns `None` |
| Scope | Valid only within the arena that created it |

### API Implications

- **Document-centric mutation.** All tree modifications go through `Document` methods
  that hold `&mut` access to the arena. This naturally enforces Rust's aliasing rules.

  ```rust
  // All mutation is document-centric
  let elem_id = doc.new_element("name");
  doc.insert_first_child(parent_id, elem_id);
  doc.set_attribute(elem_id, "key", "value");
  ```

- **Stale ID safety.** Using a `NodeId` after the node has been removed returns `None`
  (or an appropriate error), never undefined behavior.

  ```rust
  doc.remove(node_id);
  assert!(doc.get(node_id).is_none()); // Generation mismatch
  ```

- **Cross-document operations.** A `NodeId` from one document is meaningless in another.
  Cross-document copying requires explicit `deep_clone` operations.

- **No dangling references.** Because `NodeId` is a value type (not a reference), there
  are no lifetime concerns. IDs can be stored, passed around, and used freely.

### Memory Layout

```
Index:       [0]     [1]     [2]     [3]     [4]
Entries:   [ Elem  | Text  | FREE  | Elem  | FREE  ]
Gens:      [  1    |  1    |  2    |  1    |  3    ]
Free list: 4 → 2 → None
```

- Slot 2 was used (gen 0 → occupied, gen 1 → freed → gen 2) and is now free
- Slot 4 has been reused multiple times (generation 3)
- Any `NodeId { index: 2, generation: 1 }` will fail the generation check

### Trade-offs Accepted

| Trade-off | Justification |
|-----------|---------------|
| Memory not immediately reclaimed | Free list reuse is sufficient; DOM trees are short-lived |
| 4 extra bytes per slot (generation) | Trivial cost for use-after-free detection |
| IDs not globally unique | Document-scoping is sufficient; cross-doc ops are explicit |
| Custom implementation | ~200 lines of straightforward code; worth the control |

## References

- [Generational Indices (Catherine West, RustConf 2018)](https://kyren.github.io/2018/09/14/rustconf-talk.html)
- [TinyXML2 MemPool implementation](https://github.com/leethomason/tinyxml2/blob/master/tinyxml2.h)
- [Arena Allocation Pattern](https://rust-unofficial.github.io/patterns/idioms/ffi/accepting-strings.html)
