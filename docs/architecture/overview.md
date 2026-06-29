# Architecture Overview

## Project Description

**tinyxml2-rs** is a ground-up Rust implementation of the [TinyXML2](https://github.com/leethomason/tinyxml2) C++ XML library. It provides a lightweight, efficient DOM-based XML parser and printer that faithfully reproduces TinyXML2's API semantics while leveraging Rust's type system and ownership model for memory safety.

### Goals

1. **API Compatibility** — Mirror TinyXML2's public API as closely as idiomatic Rust permits, so that users familiar with TinyXML2 can transition with minimal friction.
2. **Memory Safety** — Eliminate the classes of bugs (use-after-free, double-free, buffer overflows) that plague C++ XML parsers, without sacrificing performance.
3. **Zero `unsafe` in Core** — The core `tinyxml2` crate enforces `#![forbid(unsafe_code)]`. All `unsafe` is isolated in the FFI boundary crate.
4. **Drop-in C Replacement** — The `tinyxml2-capi` crate exposes a C-compatible shared/static library so C and C++ projects can swap in the Rust implementation.
5. **Correctness First** — Every feature ships with comprehensive tests; correctness is never traded for performance.

---

## Workspace Layout

The project is organized as a three-crate Cargo workspace:

```
tinyxml2-rs/
├── Cargo.toml              # Workspace root
├── tinyxml2/               # Core library crate
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs           # Crate root, public API re-exports
│       ├── error.rs         # XmlError enum, ParseErrorKind
│       ├── entity.rs        # XML entity encode/decode
│       ├── arena.rs         # Generational arena allocator
│       └── util.rs          # Character classification, whitespace, BOM
├── tinyxml2-capi/           # C FFI binding crate
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs           # extern "C" functions, opaque handles
├── tinyxml2-bench/          # Benchmark crate
│   ├── Cargo.toml
│   └── benches/
│       └── ...
└── docs/
    └── architecture/        # This documentation
```

| Crate | Type | Purpose |
|---|---|---|
| `tinyxml2` | `lib` | Core XML parser, DOM, printer. `#![forbid(unsafe_code)]`. |
| `tinyxml2-capi` | `cdylib` + `staticlib` | C-compatible FFI layer. `unsafe` allowed for raw pointers and `extern "C"`. |
| `tinyxml2-bench` | `bin` / bench harness | Performance benchmarks comparing against reference implementations. |

---

## Module Responsibilities

| Module | Status | Responsibility |
|---|---|---|
| `error.rs` | ✅ Complete | `XmlError` enum with structured variants for parse errors, I/O errors, DOM query errors, and depth limits. `ParseErrorKind` sub-enum for parser error classification. |
| `entity.rs` | ✅ Complete | XML entity encoding (`encode_text`, `encode_attribute`) and decoding (`decode`, `decode_cow`). Handles the 5 named entities plus decimal `&#N;` and hex `&#xN;` character references. |
| `arena.rs` | ✅ Complete | Generational arena allocator. Provides `O(1)` alloc, dealloc, and lookup with stale-ID detection via generation counters. Core data structure for the DOM tree. |
| `util.rs` | ✅ Complete | Character classification (`is_name_start_char`, `is_name_char`, `is_whitespace`), whitespace handling (`skip_whitespace`, `collapse_whitespace`), name reading (`read_name`), BOM detection (`starts_with_bom`, `strip_bom`). |
| `dom.rs` | 🔲 Planned | DOM tree built on top of the arena. `NodeData`, `NodeKind`, tree traversal, `Document` as the arena owner with factory methods. |
| `parser.rs` | 🔲 Planned | Recursive-descent XML parser. Single-pass, builds DOM directly into the arena. Handles elements, attributes, text, CDATA, comments, declarations, and unknown nodes. |
| `printer.rs` | 🔲 Planned | Dual-mode XML printer (compact and pretty-printed). Implements the `XmlPrinter` / `XmlVisitor` interface. |
| `visitor.rs` | 🔲 Planned | Visitor pattern trait for DOM traversal, mirroring TinyXML2's `XMLVisitor`. |

---

## Module Dependency Graph

```
                    ┌─────────────┐
                    │   lib.rs    │  (crate root, re-exports)
                    └──────┬──────┘
                           │
              ┌────────────┼────────────┐
              │            │            │
              ▼            ▼            ▼
        ┌──────────┐ ┌──────────┐ ┌──────────┐
        │ parser   │ │   dom    │ │ printer  │
        │ (planned)│ │ (planned)│ │ (planned)│
        └────┬─────┘ └────┬─────┘ └────┬─────┘
             │            │            │
             │      ┌─────┴─────┐      │
             │      │           │      │
             ▼      ▼           ▼      ▼
        ┌──────────┐       ┌──────────┐
        │  entity  │       │ visitor  │
        │          │       │ (planned)│
        └──────────┘       └──────────┘
             │
             ▼
        ┌──────────┐
        │   util   │
        └──────────┘

        ┌──────────┐
        │  arena   │  ← used by dom, parser
        └──────────┘

        ┌──────────┐
        │  error   │  ← used by all modules
        └──────────┘


   External crate dependency:

        ┌──────────────┐      ┌──────────┐
        │ tinyxml2-capi│─────▶│ tinyxml2 │
        └──────────────┘      └──────────┘

        ┌──────────────┐      ┌──────────┐
        │tinyxml2-bench│─────▶│ tinyxml2 │
        └──────────────┘      └──────────┘
```

**Key relationships:**

- `error` is a leaf module with no internal dependencies — every other module depends on it.
- `util` depends only on `error` and provides low-level character/string operations.
- `entity` depends on `util` for character classification during decode.
- `arena` is self-contained — it has no dependencies on other internal modules.
- `dom` (planned) depends on `arena` for storage, `error` for result types.
- `parser` (planned) depends on `dom`, `entity`, `util`, and `error`.
- `printer` (planned) depends on `dom`, `entity`, and `visitor`.

---

## Design Philosophy

The project follows a strict priority ordering for design decisions:

### 1. Correctness (Highest Priority)

Every behavior must match the XML specification and TinyXML2's documented semantics. The test suite (87 tests in Phase 1) validates all edge cases including malformed input, entity boundaries, and whitespace modes. No optimization or convenience feature may compromise correctness.

**Justification:** An XML parser that produces wrong output is worse than useless — it silently corrupts data. Correctness is non-negotiable.

### 2. Compatibility

API surface, error semantics, and behavioral quirks should match TinyXML2 as closely as idiomatic Rust allows. Users migrating from TinyXML2 (C++) should find familiar patterns: document-owns-all, factory methods, visitor traversal.

**Justification:** The primary value proposition is being a safe Rust replacement for TinyXML2. Compatibility reduces migration cost and ensures the FFI layer can faithfully replicate TinyXML2's C API.

### 3. Safety

Zero `unsafe` in the core crate (`#![forbid(unsafe_code)]`). All `unsafe` is confined to the FFI boundary in `tinyxml2-capi`. The generational arena eliminates use-after-free without raw pointers. Error handling uses `Result<T, XmlError>` — no panics in normal operation.

**Justification:** Memory safety is Rust's defining feature. By forbidding `unsafe` in the core, we get compiler-verified guarantees that TinyXML2 (C++) cannot provide.

### 4. Maintainability

Clean module boundaries, comprehensive documentation, explicit error types (no stringly-typed errors), and a test suite that serves as living documentation. Each module has a single, well-defined responsibility.

**Justification:** An XML parser has a long tail of edge cases. The codebase must be approachable for contributors and auditable for security-sensitive users.

### 5. Performance (Lowest Priority)

The arena provides cache-friendly storage and `O(1)` operations. Entity decoding uses `Cow<str>` to avoid allocation when no entities are present. But performance is never prioritized over the four concerns above.

**Justification:** TinyXML2 targets "good enough" performance for configuration files, small documents, and embedded systems — not high-throughput streaming. The same holds for tinyxml2-rs.

---

## Key Architectural Decisions

### Arena-Based DOM

The DOM tree is stored in a generational arena (`Vec<Slot<T>>`) rather than a graph of heap-allocated nodes. This provides:

- **Cache locality:** Nodes are contiguous in memory.
- **O(1) alloc/dealloc:** Free-list management, no system allocator pressure per node.
- **Safe stale-ID detection:** Generation counters catch use-after-free at runtime without `unsafe`.
- **Document-owns-all semantics:** The arena (owned by `Document`) is the single owner of all nodes, matching TinyXML2's ownership model.

### Recursive Descent Parser

The parser uses recursive descent (one function per grammar production) rather than a state machine or parser generator. This mirrors TinyXML2's parsing strategy and provides:

- **Readability:** Each grammar rule maps to a named function.
- **Direct DOM construction:** Nodes are allocated in the arena as they are parsed — no intermediate AST.
- **Natural depth limiting:** Recursion depth maps directly to element nesting depth, making `max_depth` enforcement trivial.

### No `unsafe` in Core

The `#![forbid(unsafe_code)]` attribute on the `tinyxml2` crate ensures that the entire DOM, parser, and printer are verified safe by the Rust compiler. This is a hard constraint, not a guideline — it cannot be bypassed with `#[allow]`.

---

## Mapping to TinyXML2's Architecture

| TinyXML2 (C++) | tinyxml2-rs (Rust) | Notes |
|---|---|---|
| `XMLDocument` owns `MemPool` | `Document` owns `Arena<NodeData>` | Document-owns-all pattern preserved |
| `XMLNode*` raw pointers | `NodeId` (Copy, 8 bytes) | Safe handle replaces raw pointer |
| `MemPool` fixed-block allocator | `Arena<T>` generational free-list | Both O(1) alloc/dealloc |
| `XMLNode::InsertEndChild(node*)` | `document.insert_end_child(parent, child)` | Factory + insertion via Document methods |
| `XMLVisitor` virtual class | `Visitor` trait (planned) | Trait objects replace vtable inheritance |
| `XMLPrinter` (inherits `XMLVisitor`) | `Printer` struct implementing `Visitor` (planned) | Composition over inheritance |
| `XMLElement::QueryIntAttribute()` | Typed attribute query methods returning `Result` | `Result` replaces error-code out-params |
| `XMLDocument::Parse(const char*)` | `Document::parse(&str)` → `Result<(), XmlError>` | Rust error handling replaces error codes |
| `SetError()` stores error state | `Err(XmlError)` returned immediately | No mutable error state; errors are values |
