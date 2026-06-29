# ADR-0001: Project Goals

| Field   | Value                |
|---------|----------------------|
| Status  | **Accepted**         |
| Date    | 2026-06-01           |
| Authors | tinyxml2-rs maintainers |

---

## Context

TinyXML2 is a widely-used lightweight XML parser in the C++ ecosystem. It provides a
simple DOM-based API for parsing, manipulating, and serializing XML documents. Many
projects depend on its specific parsing behavior, error handling, and output format.

The Rust ecosystem lacks a direct equivalent — a safe, lightweight XML DOM parser that
is behaviorally compatible with TinyXML2. Projects migrating from C++ to Rust, or Rust
projects interfacing with systems that rely on TinyXML2's behavior, need a drop-in
behavioral replacement written in idiomatic Rust.

## Problem Statement

How do we provide TinyXML2 compatibility in Rust while maintaining safety, idiomatic
design, and zero C++ dependencies?

## Alternatives Considered

### Alternative 1: Wrap TinyXML2 via FFI (bindgen)

Generate Rust bindings to the C++ TinyXML2 library using `bindgen` or a hand-written
FFI layer.

**Pros:**
- Fastest path to API coverage
- Guaranteed behavioral compatibility (it *is* TinyXML2)
- Low development effort for initial version

**Cons:**
- Requires a C++ toolchain to build (cmake, g++/clang++)
- Inherits all memory-safety issues from the C++ implementation
- Links against the C++ runtime (libstdc++ / libc++)
- Difficult to distribute as a pure Rust crate
- FFI boundary imposes overhead and ergonomic friction
- Harder to extend or customize behavior
- Cross-compilation becomes significantly more complex

### Alternative 2: Mechanical Translation (c2rust style)

Use `c2rust` or manual translation to convert the C++ source line-by-line into Rust.

**Pros:**
- Preserves original algorithm structure
- High behavioral fidelity

**Cons:**
- Produces `unsafe` Rust throughout — not meaningfully safer than C++
- Generated code is unidiomatic and hard to read
- Maintenance nightmare: changes must be re-translated
- Does not leverage Rust's type system or ownership model
- Error handling remains C-style (error codes, null checks)

### Alternative 3: Write from Scratch in Idiomatic Rust

Implement a new XML parser and DOM library in safe, idiomatic Rust that is
*behaviorally compatible* with TinyXML2.

**Pros:**
- Memory safety by construction (no `unsafe` in core paths)
- Idiomatic Rust API (Result types, iterators, trait implementations)
- No C++ toolchain or runtime dependency
- Clean, maintainable codebase
- Full control over architecture and design decisions
- Easily testable and fuzzable
- Compatible with the broader Rust ecosystem (serde, no_std potential)

**Cons:**
- Significantly more upfront development work
- Risk of behavioral drift from the reference C++ implementation
- Must write and maintain our own parser
- Must independently discover and handle edge cases

## Decision

**Option 3 — Write from scratch in idiomatic Rust with behavioral compatibility.**

## Reasoning

The decision is driven by the project's priority ordering:

> **correctness > compatibility > safety > maintainability > performance**

1. **Safety first.** The primary motivation for a Rust implementation is memory safety.
   Wrapping C++ (Alt 1) or mechanically translating it (Alt 2) defeats this purpose.

2. **Maintainability.** A clean, idiomatic Rust codebase is far easier to maintain,
   review, and extend than either FFI bindings or translated code.

3. **Rust ecosystem compatibility.** A pure Rust crate with no C dependencies integrates
   seamlessly with `cargo`, cross-compilation, WebAssembly targets, and the broader
   ecosystem.

4. **No C++ dependency.** Eliminating the C++ toolchain requirement makes the crate
   accessible to all Rust developers regardless of their system configuration.

5. **Testability.** A ground-up implementation allows us to build comprehensive test
   infrastructure from day one, including conformance tests, round-trip tests, and fuzz
   targets.

## Consequences

### Positive

- **Memory safety**: No `unsafe` code in core paths; ownership and lifetimes enforced
  at compile time.
- **Idiomatic API**: Rust-native error handling (`Result<T, XmlError>`), iterators,
  builder patterns, and trait implementations.
- **No C++ toolchain required**: Pure `cargo build` with no external dependencies.
- **Testable**: Unit tests, integration tests, conformance tests, and fuzz targets from
  day one.
- **Maintainable**: Clean architecture with clear module boundaries and documentation.
- **Portable**: Compiles to any Rust target including WebAssembly and embedded.

### Negative

- **More upfront development work**: Must implement parser, DOM, printer, and all
  supporting infrastructure from scratch.
- **Risk of behavioral drift**: Without rigorous testing, behavior may diverge from the
  reference C++ implementation in subtle ways.
- **Must maintain own parser**: No upstream parser to inherit bug fixes from; all parser
  bugs are our responsibility.

### Mitigations

| Risk | Mitigation |
|------|------------|
| Behavioral drift | Extensive conformance test suite ported from TinyXML2's own tests |
| Missing edge cases | Behavioral specification document (`spec/behavior.md`) catalogs all observable behaviors |
| Parser bugs | Fuzz testing with `cargo-fuzz` / `libFuzzer` to discover parsing issues |
| Compatibility gaps | Cross-validation testing against the C++ reference implementation |
| Incomplete coverage | Phased development plan with clear milestones and coverage tracking |

## References

- [TinyXML2 GitHub Repository](https://github.com/leethomason/tinyxml2)
- [TinyXML2 Documentation](http://www.grinninglizard.com/tinyxml2/)
- [XML 1.0 Specification (Fifth Edition)](https://www.w3.org/TR/xml/)
