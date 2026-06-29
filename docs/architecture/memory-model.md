# Memory Model

## Overview

The tinyxml2-rs memory model is centered on a **generational arena allocator** — a flat, `Vec`-backed data structure that provides `O(1)` allocation, deallocation, and lookup with compile-time memory safety (no `unsafe`) and runtime stale-reference detection via generation counters.

The arena replaces TinyXML2's custom `MemPool` with a Rust-idiomatic equivalent that preserves the performance characteristics (O(1) operations, cache-friendly layout) while adding safety guarantees that C++ cannot provide.

---

## Data Structures

### Arena

```rust
pub struct Arena<T> {
    /// Node storage. Each slot is either occupied with a live value or vacant
    /// with a pointer to the next free slot.
    slots: Vec<Slot<T>>,

    /// Per-slot generation counters. `generations[i]` is incremented every time
    /// slot `i` is deallocated. Used to detect stale NodeId references.
    generations: Vec<u32>,

    /// Index of the first free slot, or `u32::MAX` if the free list is empty.
    free_head: u32,

    /// Number of currently occupied slots.
    len: usize,
}
```

### Slot

```rust
pub(crate) enum Slot<T> {
    /// A live value stored in this slot.
    Occupied(T),

    /// This slot is vacant. The `u32` is the index of the next free slot
    /// in the free list, or `u32::MAX` if this is the last free slot.
    Vacant(u32),
}
```

### NodeId

```rust
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct NodeId {
    /// Index into the `slots` and `generations` vectors.
    index: u32,

    /// Expected generation. Must match `generations[index]` for the ID to be valid.
    generation: u32,
}
```

---

## Generation Counters

Each slot has an independent generation counter stored in `generations[i]`:

- **Initial value:** `0` when the slot is first created.
- **Incremented on dealloc:** When slot `i` is deallocated, `generations[i]` is bumped via `wrapping_add(1)`.
- **Checked on access:** When a `NodeId { index: i, generation: g }` is used to look up a value, the arena checks `generations[i] == g`. A mismatch means the ID is stale.

### Wrapping behavior

Generation counters use `wrapping_add` so they wrap from `u32::MAX` back to `0`. In practice, a single slot would need to be allocated and deallocated 2³² times before a generation collision could occur — this is astronomically unlikely for an XML DOM.

```
Slot 5:  gen=0 → alloc → gen=0 (live)
                  dealloc → gen=1 (free)
                  alloc → gen=1 (live, new value)
                  dealloc → gen=2 (free)
                  ...

Old NodeId { index: 5, generation: 0 } → mismatch with gen=2 → None
```

---

## Free List

The free list is a **LIFO stack** (last-in, first-out) threaded through the `Vacant` slots:

```
free_head = 3

slots:  [ Occupied(A), Vacant(7), Occupied(B), Vacant(1), Occupied(C), ..., Vacant(MAX) ]
           index 0       index 1     index 2     index 3     index 4         index 7

Free list chain:  free_head → 3 → 1 → 7 → u32::MAX (end)
```

### Sentinel value

`u32::MAX` serves as the "null pointer" for the free list. When `free_head == u32::MAX`, the free list is empty and the next allocation must grow the `slots` vector.

### Why LIFO?

LIFO ordering means recently freed slots are reused first. This improves cache locality because recently used memory is more likely to be in CPU cache.

---

## O(1) Operations

### `alloc(value: T) → NodeId`

Allocates a new slot and returns a `NodeId` handle.

```
fn alloc(&mut self, value: T) -> NodeId {
    if self.free_head == u32::MAX {
        // Free list empty — push a new slot at the end
        let index = self.slots.len() as u32;
        self.slots.push(Slot::Occupied(value));
        self.generations.push(0);
        self.len += 1;
        NodeId { index, generation: 0 }
    } else {
        // Pop from free list
        let index = self.free_head;
        let next_free = match self.slots[index as usize] {
            Slot::Vacant(next) => next,
            _ => unreachable!(),
        };
        self.free_head = next_free;
        self.slots[index as usize] = Slot::Occupied(value);
        self.len += 1;
        let generation = self.generations[index as usize];
        NodeId { index, generation }
    }
}
```

**Amortized O(1):** When the free list is empty, `Vec::push` is amortized O(1) due to geometric growth. When the free list has entries, it's strict O(1).

### `dealloc(id: NodeId) → Option<T>`

Deallocates a slot and returns the stored value (if the ID is valid).

```
fn dealloc(&mut self, id: NodeId) -> Option<T> {
    // Bounds check
    if id.index as usize >= self.slots.len() { return None; }

    // Generation check
    if self.generations[id.index as usize] != id.generation { return None; }

    // Swap Occupied → Vacant
    let old_slot = std::mem::replace(
        &mut self.slots[id.index as usize],
        Slot::Vacant(self.free_head),
    );

    match old_slot {
        Slot::Occupied(value) => {
            // Push to free list head
            self.free_head = id.index;
            // Bump generation
            self.generations[id.index as usize] =
                self.generations[id.index as usize].wrapping_add(1);
            self.len -= 1;
            Some(value)
        }
        Slot::Vacant(_) => {
            // Already vacant — restore and return None
            self.slots[id.index as usize] = old_slot;
            None
        }
    }
}
```

**Strict O(1):** No search, no compaction, no system allocator call.

### `get(id: NodeId) → Option<&T>`

Looks up a value by its `NodeId`.

```
fn get(&self, id: NodeId) -> Option<&T> {
    // 1. Bounds check
    let slot = self.slots.get(id.index as usize)?;

    // 2. Generation check
    if self.generations[id.index as usize] != id.generation {
        return None;
    }

    // 3. Match Occupied
    match slot {
        Slot::Occupied(value) => Some(value),
        Slot::Vacant(_) => None,
    }
}
```

**Strict O(1):** One bounds check, one `u32` comparison, one enum match. No indirection, no pointer chasing.

### `get_mut(id: NodeId) → Option<&mut T>`

Identical to `get` but returns `&mut T`. Follows the same three-step validation.

---

## Cache-Friendly Contiguous Storage

All node data lives in a single `Vec<Slot<T>>`:

```
Memory layout:

slots: [ Slot<T> | Slot<T> | Slot<T> | Slot<T> | Slot<T> | ... ]
         ▲                                                   ▲
         │                                                   │
    contiguous in memory ──────────────────────────────────────

generations: [ u32 | u32 | u32 | u32 | u32 | ... ]
               ▲                              ▲
               │                              │
          also contiguous ─────────────────────
```

### Why this matters:

1. **Sequential traversal** (e.g., iterating all children of an element) accesses slots at nearby indices, which are contiguous in memory. The CPU prefetcher handles this efficiently.

2. **No pointer chasing.** In an `Rc<RefCell<T>>` tree, following a child pointer requires dereferencing a heap pointer to an arbitrary address. In the arena, following a child `NodeId` is an array index operation.

3. **Cache line utilization.** A typical cache line is 64 bytes. Multiple `Slot<NodeData>` values may fit in a single cache line, meaning a single cache fetch brings multiple nodes into L1.

4. **Bulk deallocation.** Dropping the `Arena` drops the entire `Vec` in one operation — no per-node destructor traversal needed (unless `T` has drop glue, but `NodeData` drops are simple).

---

## NodeId as a Copy Type

`NodeId` is 8 bytes — two `u32` fields — and implements `Copy`:

```
┌─────────────────────────────────┐
│        NodeId (8 bytes)         │
├────────────────┬────────────────┤
│  index: u32    │ generation: u32│
│  (4 bytes)     │  (4 bytes)     │
└────────────────┴────────────────┘
```

### Implications:

- **Pass by value:** `NodeId` is always passed by value, never by reference. No lifetime annotations needed.
- **Store freely:** Multiple data structures can hold copies of the same `NodeId` without ownership concerns.
- **Compare cheaply:** `PartialEq` compares 8 bytes — a single 64-bit comparison on most architectures.
- **Hash cheaply:** `Hash` hashes 8 bytes — suitable for `HashMap<NodeId, V>` usage.
- **No `Drop`:** Copying and discarding `NodeId` values has zero runtime cost.

---

## Stale ID Detection

The generation mechanism provides **safe stale-reference detection** without `unsafe` code:

### Scenario: Use-After-Free (Prevented)

```rust
let id = arena.alloc(NodeData::new_text("hello"));  // id = { index: 5, gen: 0 }
arena.dealloc(id);                                    // slot 5: gen bumped to 1
let new_id = arena.alloc(NodeData::new_text("world")); // reuses slot 5, gen: 1

// Old id still exists but is stale:
assert_eq!(arena.get(id), None);       // gen 0 ≠ gen 1 → None
assert_eq!(arena.get(new_id), Some(&NodeData::new_text("world")));  // gen 1 == gen 1 → ✅
```

### Guarantees:

| Property | Guarantee |
|---|---|
| No panic | Stale ID lookups return `None`, never panic |
| No UB | All operations are safe Rust — no undefined behavior possible |
| No silent corruption | A stale ID cannot accidentally access a different node's data (generation mismatch prevents it) |
| No memory leak | Deallocated slots are immediately available for reuse |

---

## Comparison with TinyXML2's MemPool

TinyXML2 uses a custom `MemPool` allocator for its DOM nodes:

| Aspect | TinyXML2 `MemPool` | tinyxml2-rs `Arena<T>` |
|---|---|---|
| **Storage** | Linked list of fixed-size pages, each page is an array of fixed-size blocks | Single `Vec<Slot<T>>` with geometric growth |
| **Block size** | Fixed at compile time per pool (one pool per node type) | Dynamic — `Slot<T>` size determined by `T` |
| **Allocation** | O(1): pop from free list within current page, or allocate new page | O(1) amortized: pop from free list, or `Vec::push` |
| **Deallocation** | O(1): push to free list head | O(1): swap to `Vacant`, push index to free list, bump generation |
| **Safety** | None — raw pointer returned, caller responsible for lifetime | Generation-checked `NodeId` — stale access returns `None` |
| **Type safety** | Separate pool per type, `void*` casts | Generic `Arena<T>`, fully type-safe |
| **Bulk free** | `MemPool::Clear()` resets all pages | `Arena::clear()` or `drop(arena)` |
| **Growth** | Page-at-a-time (linked list of pages) | Geometric doubling (standard `Vec` behavior) |
| **Cache behavior** | Good within a page; cross-page access may cause cache misses | Excellent — single contiguous `Vec` |
| **Memory return** | Pages not returned to OS until pool destruction | `Vec` not returned to OS until arena drop (standard `Vec` behavior) |

### Key insight:

Both systems achieve O(1) alloc/dealloc via free lists. The fundamental difference is **safety**: TinyXML2's `MemPool` returns raw pointers with no lifetime tracking; the Rust arena returns `NodeId` handles with generation-based validity checks. The Rust version catches use-after-free at runtime (returns `None`) rather than silently producing undefined behavior.

---

## Memory Overhead Analysis

### Per-slot overhead

| Component | Size | Notes |
|---|---|---|
| `Slot<T>` discriminant | 0–8 bytes | Enum discriminant; may be optimized by niche |
| `T` (payload) | `size_of::<T>()` | The actual node data |
| Generation counter | 4 bytes | One `u32` in the `generations` vec |

For a typical `NodeData` with 5 `Option<NodeId>` tree links (40 bytes), a `String` value (24 bytes), a `NodeKind` enum (1 byte + padding), and a `u32` line number:

```
NodeData ≈ 80–96 bytes (estimated, depends on padding)
Slot<NodeData> ≈ 80–104 bytes (enum discriminant + payload)
Generation ≈ 4 bytes
Total per slot ≈ 84–108 bytes
```

### Free list overhead

Vacant slots store a single `u32` (next-free index) inside the `Slot::Vacant` variant. This reuses the same memory as `Slot::Occupied` — no additional allocation.

### Comparison with `Rc<RefCell<T>>`

| | Arena | `Rc<RefCell<T>>` |
|---|---|---|
| Per-node overhead | ~4 bytes (generation) | ~16 bytes (strong count + weak count + borrow flag + heap header) |
| Pointer/handle size | 8 bytes (`NodeId`) | 8 bytes (`Rc` pointer) |
| Allocation | `Vec::push` or free-list pop | `Box::new` → system allocator |
| Fragmentation | None (contiguous) | Potential heap fragmentation |

### Vec growth strategy

`Vec` uses geometric doubling: capacity doubles when full. This means at most 50% wasted capacity at any time. For a document with `N` nodes, the arena uses at most `2N × slot_size` bytes — competitive with any allocator strategy.
