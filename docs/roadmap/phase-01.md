# Phase 1: Foundation & Infrastructure

> **Status:** ✅ COMPLETED  
> **Estimated Complexity:** MEDIUM (~800 LOC)  
> **Dependencies:** None (bootstrap phase)  
> **Milestone:** `v0.0.1-alpha` internal

---

## Objectives

Establish the project skeleton, configuration, continuous integration pipeline, and
foundational types that every subsequent phase depends on. Phase 1 lays the
ground-truth type system and utility layer that the DOM, parser, and serializer
will build upon.

---

## Deliverables

### 1. Error System — `error.rs`

| Item | Description |
|------|-------------|
| `XmlError` enum | 21 TinyXML2-compatible error codes covering parse errors, I/O errors, and structural violations |
| `Display` impl | Human-readable error messages matching TinyXML2 naming conventions |
| `Error` trait impl | Standard Rust error integration |
| `error_name()` | Returns the C++ enum variant name string for FFI compatibility |
| `error_line()` | Optional line-number attachment for parse-time diagnostics |
| `Result<T>` alias | `Result<T, XmlError>` convenience alias used project-wide |

### 2. Entity Handling — `entity.rs`

| Item | Description |
|------|-------------|
| 5 predefined XML entities | `&amp;` `&lt;` `&gt;` `&quot;` `&apos;` encode/decode |
| Numeric character references | `&#NNN;` (decimal) and `&#xHHHH;` (hexadecimal) resolution |
| `entity_encode()` | Encode special characters for safe XML output |
| `entity_decode()` | Resolve entity references during parse |
| Round-trip guarantee | `decode(encode(s)) == s` for all valid inputs |

### 3. Character Classification Utilities — `util.rs`

| Item | Description |
|------|-------------|
| `is_name_start_char(c)` | XML 1.0 §2.3 NameStartChar production |
| `is_name_char(c)` | XML 1.0 §2.3 NameChar production |
| `is_xml_whitespace(c)` | `#x20`, `#x9`, `#xD`, `#xA` per XML 1.0 §2.3 |
| `skip_whitespace()` | Advance cursor past whitespace |
| `skip_bom()` | Detect and skip UTF-8 BOM (`EF BB BF`) |
| `Whitespace` enum | `Preserve` / `Collapse` / `NormalizeAttribute` modes |
| `ParseOptions` builder | Fluent API for parser configuration (whitespace mode, depth limit, etc.) |

### 4. Generational Arena Allocator — `arena.rs`

| Item | Description |
|------|-------------|
| `Arena<T>` | Generational arena with `Vec<Entry<T>>` backing store |
| `NodeId` | Opaque handle containing `index: u32` + `generation: u32` |
| `alloc(value) -> NodeId` | Allocate a slot, return a generationally-tagged handle |
| `dealloc(id)` | Free a slot, increment generation to invalidate stale handles |
| `get(id) -> Option<&T>` | Bounds + generation check before returning reference |
| `get_mut(id) -> Option<&mut T>` | Mutable variant with identical safety checks |
| Free-list recycling | Deallocated slots are reused via intrusive free list |
| Stale handle detection | Accessing a deallocated `NodeId` returns `None`, never panics |

### 5. Workspace Restructuring

| Item | Description |
|------|-------------|
| Crate consolidation | Reduced from 8 crates to 3: `tinyxml2` (lib), `tinyxml2-ffi` (C API), `tinyxml2-cli` (binary) |
| Cargo workspace | Root `Cargo.toml` with `[workspace]` members |
| Edition 2024 | All crates set to `edition = "2024"` |
| MSRV | `rust-version = "1.85.0"` enforced |
| Lints | `#![warn(missing_docs)]`, `#![deny(unsafe_op_in_unsafe_fn)]` |

### 6. Configuration & CI

| Item | Description |
|------|-------------|
| `Cargo.toml` | Workspace-level metadata, dependencies, features |
| `.github/workflows/ci.yml` | Build, test, clippy, rustfmt, MSRV check |
| `rustfmt.toml` | Project formatting rules |
| `clippy.toml` | Lint configuration |
| `LICENSE` | MIT license |
| `README.md` | Project overview, build instructions, roadmap link |

---

## Key Source Files

| File | Purpose |
|------|---------|
| `tinyxml2/src/error.rs` | Error types and codes |
| `tinyxml2/src/entity.rs` | Entity encode/decode |
| `tinyxml2/src/util.rs` | Character classification, whitespace, BOM |
| `tinyxml2/src/arena.rs` | Generational arena allocator |
| `tinyxml2/src/lib.rs` | Public API re-exports |

---

## Test Coverage

| Category | Tests | Description |
|----------|-------|-------------|
| Entity round-trips | 12 | Encode→decode identity for all entity types |
| Arena lifecycle | 18 | Alloc, dealloc, generation invalidation, reuse |
| Arena stress | 5 | Bulk alloc/dealloc, free-list integrity |
| Character classification | 15 | NameStartChar, NameChar, whitespace boundaries |
| BOM detection | 8 | UTF-8 BOM present/absent, partial BOM, empty input |
| Error Display | 10 | All 21 error variants produce correct messages |
| Error names | 7 | `error_name()` matches TinyXML2 C++ strings |
| Error line numbers | 5 | Line attachment and formatting |
| ParseOptions builder | 7 | Default values, fluent chaining, validation |

**Totals:** 87 unit tests, 15 doc tests — **0 warnings, 0 errors**

---

## Acceptance Criteria — All Met ✅

- [x] `cargo build` succeeds with zero warnings on stable 1.85.0+
- [x] `cargo test` — 87 unit tests + 15 doc tests pass
- [x] `cargo clippy` — zero warnings
- [x] `cargo fmt --check` — no diff
- [x] All 21 `XmlError` variants match TinyXML2 error codes
- [x] Entity encode/decode round-trips for all 5 predefined + numeric references
- [x] Arena generational safety: stale `NodeId` access returns `None`
- [x] `ParseOptions` defaults match TinyXML2 defaults
- [x] Workspace compiles as a single `cargo build --workspace`

---

## Lessons Learned

1. **Generational arenas eliminate use-after-free** — The `NodeId` generation check
   provides memory safety without `Rc`/`RefCell` overhead.
2. **Edition 2024 changes** — Required adjusting `use` statements and some trait
   resolution rules; well worth the forward-compatibility.
3. **Entity round-trip testing** is critical — several edge cases in numeric character
   reference parsing were caught only by property-based round-trip assertions.

---

## Next Phase

→ [Phase 2: DOM Core](./phase-02.md)
