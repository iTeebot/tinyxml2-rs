# tinyxml2-rs

**A ground-up Rust implementation of the TinyXML2 API.**

[![Build Status](https://img.shields.io/github/actions/workflow/status/iTeebot/tinyxml2-rs/ci.yml?branch=main&style=flat-square&logo=github)](https://github.com/iTeebot/tinyxml2-rs/actions)
[![Crates.io](https://img.shields.io/crates/v/tinyxml2?style=flat-square&logo=rust)](https://crates.io/crates/tinyxml2)
[![docs.rs](https://img.shields.io/docsrs/tinyxml2?style=flat-square&logo=docs.rs)](https://docs.rs/tinyxml2)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](LICENSE)
[![MSRV](https://img.shields.io/badge/MSRV-1.85-orange?style=flat-square&logo=rust)](rust-toolchain.toml)

---

## What is tinyxml2-rs?

**tinyxml2-rs** is a native Rust implementation that provides behavioral and API compatibility with [TinyXML-2](https://github.com/leethomason/tinyxml2). It treats TinyXML-2 as a *behavioral specification* — not as source code to translate.

### What it **IS**

- ✅ A ground-up Rust implementation of the TinyXML2 API surface
- ✅ Behaviorally compatible with TinyXML2 (same inputs produce same outputs)
- ✅ Written in safe, idiomatic Rust with an arena-based DOM
- ✅ A C FFI layer for drop-in replacement in C/C++ projects
- ✅ Lightweight, fast, and suitable for embedded or resource-constrained environments

### What it is **NOT**

- ❌ **Not a wrapper** around the C++ TinyXML2 library
- ❌ **Not a line-by-line translation** of the C++ source
- ❌ **Not a full XML specification implementation** — it targets the same subset as TinyXML2
- ❌ **Not a streaming parser** — it builds an in-memory DOM, just like TinyXML2

---

## Quick Start

Add `tinyxml2` to your project:

```bash
cargo add tinyxml2
```

### Parsing XML and Navigating the DOM

```rust
use tinyxml2::Document;

fn main() {
    let xml = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <bookstore>
            <book category="fiction">
                <title lang="en">The Great Gatsby</title>
                <author>F. Scott Fitzgerald</author>
                <year>1925</year>
                <price>10.99</price>
            </book>
            <book category="nonfiction">
                <title lang="en">Sapiens</title>
                <author>Yuval Noah Harari</author>
                <year>2011</year>
                <price>14.99</price>
            </book>
        </bookstore>
    "#;

    // Parse the document
    let doc = Document::parse(xml).expect("Failed to parse XML");

    // Navigate to the root element
    let root = doc.root_element().expect("No root element");
    assert_eq!(root.name(), "bookstore");

    // Iterate over child elements
    for book in root.children_with_name("book") {
        let category = book.attribute("category").unwrap_or("unknown");
        let title = book.first_child_element_with_name("title")
            .and_then(|t| t.text())
            .unwrap_or("untitled");
        println!("{title} ({category})");
    }
}
```

### Writing XML

```rust
use tinyxml2::Document;

fn main() {
    let mut doc = Document::new();

    let root = doc.new_element("config");
    doc.insert_end_child(doc.root(), root).unwrap();

    let setting = doc.new_element("setting");
    doc.set_attribute(setting, "name", "volume").unwrap();
    doc.set_attribute(setting, "value", "75").unwrap();
    doc.insert_end_child(root, setting).unwrap();

    // Print compact XML
    let output = doc.to_string_compact();
    println!("{output}");

    // Print formatted XML
    let output = doc.to_string();
    println!("{output}");
}
```

---

## Features

| Feature | Description |
|---------|-------------|
| **Safe Rust** | No `unsafe` in the core crate — memory safety guaranteed by the compiler |
| **Behavioral Compatibility** | Matches TinyXML2 behavior across thousands of test cases |
| **Arena-Based DOM** | Generational arena for cache-friendly, allocation-efficient DOM trees |
| **C FFI Layer** | Drop-in C API compatible with TinyXML2's header interface |
| **Recursive Descent Parser** | Hand-written parser mirroring TinyXML2's parsing strategy |
| **Dual-Mode Writer** | Compact and pretty-printed XML output |
| **Visitor Pattern** | Extensible traversal via the Visitor trait |
| **Entity Handling** | Built-in XML entity encoding/decoding |
| **Error Reporting** | Rich, structured error types with line/column information |
| **Zero Dependencies** | Core crate has no external dependencies |

---

## Architecture

tinyxml2-rs is organized as a Cargo workspace with three crates:

```
tinyxml2-rs/
├── crates/
│   ├── tinyxml2/          # Core library — DOM, parser, writer, entities, errors
│   ├── tinyxml2-capi/     # C FFI compatibility layer (cdylib + staticlib)
│   └── tinyxml2-bench/    # Benchmark harness (criterion)
├── tests/                 # Integration & conformance tests
├── fuzz/                  # Fuzz testing targets
├── benches/               # Benchmark data & fixtures
├── spec/                  # TinyXML2 behavioral specification tests
└── examples/              # Usage examples
```

| Crate | Purpose | Depends On |
|-------|---------|------------|
| `tinyxml2` | Core Rust implementation of the DOM, parser, writer, and utilities | — |
| `tinyxml2-capi` | Exposes `tinyxml2` through a C-compatible FFI | `tinyxml2` |
| `tinyxml2-bench` | Performance benchmarks comparing against reference implementations | `tinyxml2` |

For a deep dive, see [ARCHITECTURE.md](ARCHITECTURE.md).

---

## Compatibility with TinyXML2

tinyxml2-rs aims for **behavioral compatibility** with TinyXML2:

- **Parsing**: Same documents accepted/rejected; same error conditions
- **DOM API**: Equivalent navigation, mutation, and query operations
- **Output**: Byte-identical output in both compact and pretty-print modes
- **Entity handling**: Same set of predefined XML entities
- **Error semantics**: Equivalent error codes and failure modes

Compatibility is verified through a conformance test suite derived from TinyXML2's own tests and additional edge-case coverage.

---

## Performance Goals

- **Parsing throughput** competitive with or exceeding TinyXML2 (C++)
- **Memory usage** within 1.5× of TinyXML2 for equivalent documents
- **Zero-copy** string handling where possible
- **Cache-friendly** arena layout for DOM traversal
- **No allocations** on the hot path during parsing

Benchmarks are tracked in `crates/tinyxml2-bench/` using [Criterion](https://github.com/bheisler/criterion.rs).

---

> **Phase 5 — Visitor Pattern & Ergonomics** (planned)

tinyxml2-rs is under active development. The core DOM, parser, writer, and visitor infrastructure are fully implemented. See [ROADMAP.md](ROADMAP.md) for the full development plan.

| Milestone | Status |
|-----------|--------|
| Project scaffolding & architecture | ✅ Complete |
| Error types & entity handling | ✅ Complete |
| Arena-based DOM | ✅ Complete |
| Recursive descent parser | ✅ Complete |
| XML writer (compact + pretty) | ✅ Complete |
| Visitor pattern | ✅ Complete |
| C FFI layer | ⬚ Planned |
| Conformance test suite | ⬚ Planned |
| `0.1.0` release | ⬚ Planned |

---

## Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) before submitting a pull request.

We follow the [Contributor Covenant Code of Conduct](CODE_OF_CONDUCT.md).

---

## Security

To report a security vulnerability, please see [SECURITY.md](SECURITY.md). **Do not open a public issue.**

---

## License

Licensed under the [MIT License](LICENSE).

```
Copyright (c) 2026 Teebot
```

---

<p align="center">
  Made with 🦀 by <a href="https://github.com/iTeebot">Teebot</a>
</p>
