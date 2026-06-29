# tinyxml2-rs — Development Roadmap

> A ground-up Rust implementation of the [TinyXML2](https://github.com/leethomason/tinyxml2) C++ API.
>
> **Crate Architecture:** `tinyxml2` (core library) · `tinyxml2-capi` (C FFI bindings) · `tinyxml2-bench` (benchmarks & comparisons)

---

## Overview

This roadmap defines eight sequential phases for building a complete, production-grade Rust
replacement for TinyXML2. The implementation uses a **generational arena** for memory-safe tree
storage and a **recursive-descent parser** for XML processing. Each phase builds on the previous
one and is designed to produce a shippable, testable increment.

| Phase | Title                        | Complexity | Status         |
|-------|------------------------------|------------|----------------|
| 1     | Foundation                   | Medium     | ✅ **Completed** |
| 2     | DOM Core                     | High       | ✅ **Completed** |
| 3     | XML Parser                   | High       | ✅ **Completed** |
| 4     | Writer / Serializer          | Medium     | ✅ **Completed** |
| 5     | Visitor Pattern & Ergonomics | Medium     | 🔲 Planned     |
| 6     | C API (`tinyxml2-capi`)      | Medium     | 🔲 Planned     |
| 7     | Testing & Benchmarks         | Medium     | 🔲 Planned     |
| 8     | Documentation & Release      | Low        | 🔲 Planned     |

---

## Phase 1: Foundation ✅ **COMPLETED**

> Establish the core infrastructure, error handling, and build pipeline that every subsequent
> phase depends on.

**Estimated Complexity:** Medium

### Key Deliverables

- [x] **Error types** — `XmlError` enum covering parse errors, I/O errors, missing attributes,
  value conversion failures, and arena allocation faults. Maps to TinyXML2's `XMLError` codes.
- [x] **Entity handling** — Encode/decode the five predefined XML entities (`&amp;`, `&lt;`,
  `&gt;`, `&quot;`, `&apos;`) plus numeric character references (`&#xHHHH;`, `&#DDDD;`).
- [x] **Generational arena allocator** — Arena-based node storage using generational indices
  (`NodeId`) for O(1) access with dangling-reference detection. Supports allocation, deallocation,
  and safe index reuse.
- [x] **Configuration system** — `ParserConfig` struct with builder pattern for whitespace mode,
  entity processing toggle, maximum parse depth, and encoding settings.
- [x] **CI pipeline** — GitHub Actions workflows for `cargo check`, `cargo test`, `cargo clippy`,
  `cargo fmt --check`, and MSRV verification.
- [x] **Documentation infrastructure** — Crate-level docs, module-level docs, and `#[doc(hidden)]`
  annotations for internal APIs.
- [x] **Test suite** — **87 unit tests** covering error construction, entity round-tripping, arena
  operations, and configuration validation. **15 doc tests** embedded in public API documentation.

### Architecture Decisions Locked

| Decision                | Choice                              | Rationale                                       |
|-------------------------|-------------------------------------|-------------------------------------------------|
| Node storage            | Generational arena (`Vec<Entry<T>>`)| Cache-friendly, no `Rc`/`RefCell`, safe reuse   |
| Index type              | `NodeId(u32 index, u32 generation)` | Compact, detects use-after-free                  |
| Error strategy          | `Result<T, XmlError>` everywhere    | Idiomatic Rust, no panics in library code        |
| String storage          | Owned `String` per node             | Simplicity first; interning deferred to Phase 8  |
| Configuration           | Builder pattern on `ParserConfig`   | Ergonomic, extensible, `#[non_exhaustive]`       |

---

## Phase 2: DOM Core

> Implement the full XML Document Object Model tree with all node types, tree manipulation
> operations, and deep cloning.

**Estimated Complexity:** High — this is the largest single phase by line count.

### Key Deliverables

- [x] **Node type enum** — `XmlNode` variants: `Document`, `Element`, `Text`, `Comment`,
  `Declaration`, `ProcessingInstruction`, `Unknown`. Each variant holds type-specific data.
- [x] **`Document` root** — The top-level container. Owns the arena. Provides `new()`,
  `load_file()`, `save_file()`, and the `root_element()` accessor.
- [x] **`Element` operations** — `name()`, `set_name()`, attribute CRUD (`set_attribute`,
  `find_attribute`, `delete_attribute`), child element queries (`first_child_element`,
  `next_sibling_element` with optional name filter).
- [x] **`Attribute` storage** — Linked-list of attributes per element, supporting typed getters
  (`int_value`, `float_value`, `bool_value`) with fallback defaults, mirroring the C++ API.
- [x] **Tree operations** — `insert_first_child`, `insert_end_child`, `insert_after_node`,
  `delete_child`, `delete_children`. All operations maintain parent/child/sibling pointer
  consistency within the arena.
- [x] **Deep cloning** — `deep_clone(node, target_document)` that recursively copies a subtree,
  including across documents (re-allocating into the target arena).
- [x] **Memory management** — `Document::clear()` to reset the arena. Individual node deletion
  with generation bump to invalidate stale `NodeId` handles.

### Estimated Test Count: 120–150 unit tests

### Design Notes

```
Document (arena owner)
  └─ Element "root"
       ├─ Attribute "version" = "1.0"
       ├─ Element "child1"
       │    └─ Text "hello"
       ├─ Comment "<!-- note -->"
       └─ Element "child2"
            └─ Element "nested"
```

All parent ↔ child ↔ sibling links are stored as `Option<NodeId>` inside each node's metadata
struct, enabling O(1) traversal without pointer chasing outside the arena.

---

## Phase 3: XML Parser

> Build the recursive-descent XML parser that converts raw XML text into the DOM tree from Phase 2.

**Estimated Complexity:** High — correctness-critical with many edge cases.

### Key Deliverables

- [x] **Recursive-descent parser** — Hand-written parser (no parser combinators or generated code)
  operating on `&[u8]` input. Matches TinyXML2's parsing strategy for behavioral compatibility.
- [x] **Entity resolution** — Inline expansion of the five predefined entities and numeric
  character references during text and attribute value parsing (leveraging Phase 1 entity module).
- [x] **Whitespace handling** — Two modes mirroring TinyXML2:
  - `PRESERVE_WHITESPACE` — Keep all whitespace as-is.
  - `COLLAPSE_WHITESPACE` — Collapse runs of whitespace to a single space, trim leading/trailing.
- [x] **BOM detection** — Auto-detect and skip UTF-8 BOM (`0xEF 0xBB 0xBF`) at the start of
  input. Reject non-UTF-8 BOMs with a clear error.
- [x] **Depth limits** — Configurable maximum nesting depth (default: 100) to prevent stack
  overflow on malicious input. Returns `XmlError::ElementDepthExceeded` on violation.
- [x] **Error recovery** — Rich error reporting with byte offset, line number, and column number.
  No panic paths.
- [x] **Parse entry points** — `Document::parse(xml: &str)`, `Document::load_file(path)`,
  `Document::parse_bytes(bytes: &[u8])`.

### Parser Architecture

```
parse_document()
  ├─ skip_bom()
  ├─ parse_declaration()?     // <?xml ... ?>
  └─ parse_element()          // recursive
       ├─ parse_attributes()
       ├─ parse_children()
       │    ├─ parse_element()      // recurse
       │    ├─ parse_text()
       │    ├─ parse_comment()      // <!-- ... -->
       │    ├─ parse_cdata()        // <![CDATA[ ... ]]>
       │    └─ parse_pi()           // <?target ... ?>
       └─ parse_close_tag()
```

### Estimated Test Count: 200+ unit tests (including malformed XML, edge cases, encoding)

---

## Phase 4: Writer / Serializer

> Implement XML output with pretty-printing and compact modes, equivalent to TinyXML2's
> `XMLPrinter`.

**Estimated Complexity:** Medium

### Key Deliverables

- [x] **`XmlPrinter` struct** — Stateful writer that traverses the DOM and emits well-formed XML.
  Implements the `XmlVisitor` trait (Phase 5) internally for traversal.
- [x] **Pretty-print mode** — Indented output with configurable indent string (default: 4 spaces).
  Newlines between elements. Mirrors `XMLPrinter(FILE*, true)`.
- [x] **Compact mode** — Minimal whitespace output for network/storage efficiency. Mirrors
  `XMLPrinter(FILE*, false)`.
- [x] **Write targets** —
  - `to_string() -> String` — In-memory serialization.
  - `to_writer(impl Write)` — Streaming output to any `std::io::Write` sink.
  - `to_file(path)` — Convenience wrapper for file output.
- [x] **Streaming API** — Push-based API for building XML without a DOM tree:
  `open_element("tag")`, `push_attribute("key", "value")`, `push_text("content")`,
  `close_element()`. Useful for high-performance serialization.
- [x] **Entity escaping** — Automatic escaping of `<`, `>`, `&`, `"`, `'` in text content and
  attribute values during output.
- [x] **Declaration output** — Optional XML declaration (`<?xml version="1.0" encoding="UTF-8"?>`)
  controlled by configuration.

### Estimated Test Count: 80–100 unit tests

---

## Phase 5: Visitor Pattern & Ergonomics

> Add the Visitor pattern for DOM traversal and ergonomic wrapper types for a Rust-idiomatic API
> surface.

**Estimated Complexity:** Medium

### Key Deliverables

- [ ] **`XmlVisitor` trait** — Mirrors TinyXML2's `XMLVisitor` with `visit_enter` / `visit_exit`
  methods for each node type. Returns `bool` to control traversal continuation.
  ```rust
  pub trait XmlVisitor {
      fn visit_enter_document(&mut self, doc: &Document) -> bool { true }
      fn visit_exit_document(&mut self, doc: &Document) -> bool { true }
      fn visit_enter_element(&mut self, element: ElementRef<'_>, attrs: &[Attribute]) -> bool { true }
      fn visit_exit_element(&mut self, element: ElementRef<'_>) -> bool { true }
      fn visit_text(&mut self, text: &str) -> bool { true }
      fn visit_comment(&mut self, comment: &str) -> bool { true }
      fn visit_declaration(&mut self, decl: &str) -> bool { true }
      fn visit_unknown(&mut self, unknown: &str) -> bool { true }
  }
  ```
- [ ] **`Document::accept(visitor)`** — Depth-first traversal that drives the visitor.
- [ ] **Handle types** — `NodeHandle`, `ElementHandle`, `TextHandle` — typed wrappers around
  `NodeId` that provide safe, ergonomic access without exposing raw arena indices.
- [ ] **`ElementRef<'a>` / `ElementMut<'a>`** — Borrowed references into the arena with
  lifetime-bounded access. Prevents iterator invalidation.
- [ ] **Iterators** — `children()`, `child_elements()`, `attributes()`, `siblings()` returning
  standard Rust iterators (`impl Iterator<Item = ...>`).
- [ ] **Convenience methods** — `element.text()` (get text content), `element.set_text()`,
  `element.query_int_attribute()`, `element.get_or_create_child()`.

### Estimated Test Count: 80–100 unit tests

---

## Phase 6: C API (`tinyxml2-capi` crate)

> Expose the Rust library through a C-compatible FFI for drop-in replacement in C/C++ projects.

**Estimated Complexity:** Medium — mechanically intensive but architecturally straightforward.

### Key Deliverables

- [ ] **`extern "C"` function exports** — One C function per public API method. Naming convention:
  `tinyxml2_document_new`, `tinyxml2_document_parse`, `tinyxml2_element_name`, etc.
- [ ] **Opaque handle types** — C-side pointers (`*mut Document`, `*mut Element`) wrapped in
  opaque structs. All access goes through FFI functions.
- [ ] **Error codes** — C-compatible `enum TinyXml2Error` with integer values matching TinyXML2's
  error enum for maximum compatibility.
- [ ] **Header generation** — Auto-generated `tinyxml2.h` header via [`cbindgen`](https://github.com/eyre-rs/cbindgen).
  Configured in `cbindgen.toml` with C-style output.
- [ ] **Build outputs** —
  - Static library (`libtinyxml2_rs.a` / `tinyxml2_rs.lib`)
  - Shared library (`libtinyxml2_rs.so` / `.dylib` / `.dll`)
  - Configurable via Cargo features: `features = ["static", "shared"]`
- [ ] **Memory safety contract** — All FFI functions validate non-null pointers. Null inputs
  return error codes or no-op. No panics across FFI boundary (`catch_unwind` guards).
- [ ] **Lifetime management** — `tinyxml2_document_free()`, `tinyxml2_string_free()` for
  heap-allocated returns. Clear ownership documentation in header comments.

### Estimated Test Count: 60–80 integration tests (C caller tests via `cc` crate in build script)

---

## Phase 7: Testing & Benchmarks (`tinyxml2-bench` crate)

> Comprehensive compatibility testing against TinyXML2 C++ and performance benchmarking.

**Estimated Complexity:** Medium

### Key Deliverables

- [ ] **Compatibility test suite** — Port TinyXML2's own test cases (`xmltest.cpp`) to Rust.
  Verify identical parse results, error behavior, and output formatting for ~300 test vectors.
- [ ] **Round-trip tests** — Parse → serialize → re-parse cycle for a corpus of real-world XML
  files. Assert structural equality.
- [ ] **Fuzz testing** — `cargo-fuzz` harnesses for:
  - `fuzz_parse` — Random byte sequences → parser (must not panic/crash).
  - `fuzz_roundtrip` — Valid XML → parse → serialize → parse → assert equality.
  - `fuzz_capi` — Random FFI call sequences → C API (must not segfault).
- [ ] **Criterion benchmarks** — Comparative benchmarks against the original TinyXML2 C++ library
  (linked via `cc` crate):
  | Benchmark                  | Measures                                |
  |----------------------------|-----------------------------------------|
  | `parse_small`              | Parse a 1 KB XML document               |
  | `parse_medium`             | Parse a 100 KB XML document             |
  | `parse_large`              | Parse a 10 MB XML document              |
  | `serialize_pretty`         | Pretty-print a parsed DOM               |
  | `serialize_compact`        | Compact-print a parsed DOM              |
  | `dom_traversal`            | Walk all nodes depth-first              |
  | `attribute_lookup`         | Query attributes by name (100K lookups) |
  | `arena_alloc_dealloc`      | Allocate/deallocate 100K nodes          |
- [ ] **Performance targets** — Within 1.5× of TinyXML2 C++ for parsing, within 1.0× for
  serialization (Rust I/O should match or beat C++ `fprintf`).
- [ ] **CI integration** — Benchmark results tracked via `criterion`'s JSON output. Regression
  alerts on >10% slowdown.

### Estimated Test Count: 300+ compatibility tests, 3 fuzz targets, 8 benchmark groups

---

## Phase 8: Documentation & Release

> Polish documentation, write migration guides, and publish the initial release.

**Estimated Complexity:** Low

### Key Deliverables

- [ ] **Migration guide** — `MIGRATION.md` mapping every TinyXML2 C++ class and method to its
  Rust equivalent. Organized by class (`XMLDocument` → `Document`, `XMLElement` → `Element`, etc.)
  with code examples for each.
- [ ] **Example programs** — `examples/` directory with:
  - `parse_file.rs` — Parse an XML file and print element names.
  - `build_dom.rs` — Programmatically construct a DOM and serialize.
  - `visitor.rs` — Implement a custom visitor to extract data.
  - `streaming_writer.rs` — Use the push-based API for large output.
  - `c_interop.rs` — Demonstrate calling the C API from Rust.
- [ ] **API documentation** — 100% documentation coverage on public items. Rich examples in doc
  comments. Cross-linked with `#[doc]` attributes.
- [ ] **`README.md` overhaul** — Feature matrix, quick start, performance comparison table,
  MSRV policy, and contribution guidelines.
- [ ] **`CHANGELOG.md`** — Initial changelog following [Keep a Changelog](https://keepachangelog.com/) format.
- [ ] **Crate metadata** — `Cargo.toml` keywords, categories, repository link, license (MIT/Apache-2.0
  dual license), and `rust-version` field.
- [ ] **0.1.0 release** — Publish `tinyxml2` and `tinyxml2-capi` to [crates.io](https://crates.io).
  Tag `v0.1.0` in git.

---

## Future Work

> Items beyond the 0.1.0 release, tracked for future planning.

### SIMD-Accelerated Parsing

Leverage SIMD instructions (`SSE2`/`AVX2`/`NEON`) for hot-path operations:
- Fast scanning for `<`, `>`, `&`, `"` delimiters using `_mm256_cmpeq_epi8`.
- Bulk whitespace detection and skipping.
- Expected 2–4× speedup on large documents.
- Use the [`memchr`](https://crates.io/crates/memchr) crate for portable SIMD-accelerated byte search as a starting point.

### Parallel DOM Utilities

- **Parallel visitor** — Rayon-based parallel traversal for read-only visitors on large DOMs.
- **Parallel parsing** — Experimental chunked parsing for multi-gigabyte XML files (requires
  speculative parsing with rollback).
- **Thread-safe Document** — `Arc<RwLock<Arena>>` variant for concurrent read access.

### XML Namespace Support

- Namespace-aware parsing: `xmlns` attribute handling, prefix resolution.
- Namespace-qualified element and attribute lookup: `element.find_attribute_ns("uri", "local")`.
- Namespace context propagation through the DOM tree.
- *Note:* TinyXML2 intentionally omits namespace support; this would be a Rust-only extension.

### Serde Integration

- `tinyxml2-serde` crate providing `Serialize`/`Deserialize` implementations.
- Derive macros for automatic XML ↔ struct mapping.
- Attribute vs. element field annotations (`#[xml(attribute)]`, `#[xml(child)]`).
- Compatibility with existing serde XML crates' conventions where possible.

### `no_std` Support

- Feature-gated `no_std` mode for embedded/WASM targets.
- Replace `std::io::Write` with a trait abstraction.
- Replace `String`/`Vec` with `alloc` crate equivalents.
- Remove file I/O APIs under `no_std`; retain in-memory parsing and serialization.
- Target: `#![no_std]` with `extern crate alloc`.

### Additional Future Items

- **XPath subset** — Basic XPath 1.0 expression evaluation for element queries.
- **Schema validation** — Optional DTD/XSD validation layer.
- **String interning** — Deduplicate tag names and attribute names in the arena for reduced memory
  usage on large documents with repetitive structure.
- **Memory-mapped parsing** — `mmap`-based input for zero-copy parsing of large files.
- **WASM bindings** — `wasm-bindgen` wrapper for browser/Node.js usage.

---

## Contributing

Contributions are welcome at any phase. Please see the issue tracker for phase-specific tasks
labeled `phase-N`. Each phase has a tracking issue with a checklist of deliverables.

**Priority for contributors:**
1. Phase 2 (DOM Core) — Most impactful, enables all downstream phases.
2. Phase 3 (Parser) — Second most impactful, enables real-world usage.
3. Phase 7 (Testing) — Can begin in parallel once Phase 3 is complete.

---

*Last updated: 2025-06-29*
