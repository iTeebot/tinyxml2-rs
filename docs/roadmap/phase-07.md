# Phase 7: Testing, Fuzzing & Benchmarks

> **Status:** ✅ COMPLETED (2026-07-01)  
> **Estimated Complexity:** MEDIUM-HIGH (~2000 LOC)  
> **Dependencies:** Phase 6 (complete API surface, including C API)  
> **Milestone:** `v0.1.0-rc` release candidate

---

## Objectives

Build a comprehensive quality assurance infrastructure that ensures tinyxml2-rs
is correct, robust, and performant. This phase establishes three pillars:
(1) compatibility testing against TinyXML2 C++, (2) fuzz testing to discover
edge cases, and (3) benchmarking to measure and compare performance.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                    Quality Assurance Infrastructure              │
│                                                                  │
│  ┌──────────────────┐  ┌────────────────┐  ┌─────────────────┐  │
│  │  Compatibility    │  │  Fuzz Testing   │  │  Benchmarks     │  │
│  │  Test Suite       │  │                 │  │                 │  │
│  │                   │  │  cargo-fuzz     │  │  criterion.rs   │  │
│  │  TinyXML2 C++     │  │  libFuzzer      │  │                 │  │
│  │  vs tinyxml2-rs   │  │                 │  │  Parse speed    │  │
│  │  same input →     │  │  parse_fuzz     │  │  Serialize      │  │
│  │  same output?     │  │  roundtrip_fuzz │  │  Traverse       │  │
│  │                   │  │  serialize_fuzz │  │  Memory         │  │
│  │  50+ valid XML    │  │                 │  │                 │  │
│  │  30+ invalid XML  │  │  Property tests │  │  TinyXML2       │  │
│  │  20+ unicode      │  │  (proptest)     │  │  comparison     │  │
│  └──────────────────┘  └────────────────┘  └─────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

---

## Deliverables

### 1. Compatibility Test Suite

The primary goal: given the same XML input, tinyxml2-rs and TinyXML2 C++ must
produce the same results. This suite automates that comparison.

#### Test Harness

```
tests/
├── compat/
│   ├── harness.rs          # Comparison framework
│   ├── cpp_runner.rs       # Subprocess to run TinyXML2 C++ reference
│   ├── mod.rs              # Test registration
│   └── reference/          # Pre-computed TinyXML2 C++ outputs
│       ├── basic_element.json
│       ├── nested_deep.json
│       └── ...
```

The harness:
1. Feeds the same XML input to both libraries
2. Serializes resulting DOMs to a canonical JSON representation
3. Diffs the JSON trees for structural equality
4. Reports any divergences with context

#### Comparison Points

| Aspect | Comparison Method |
|--------|-------------------|
| DOM structure | JSON tree diff (node types, children, ordering) |
| Text content | Byte-exact string comparison |
| Attribute values | Name-value pair comparison, order-sensitive |
| Error codes | Enum value mapping |
| Error line numbers | Exact match |
| Pretty-print output | Byte-exact string comparison |
| Compact output | Byte-exact string comparison |

### 2. XML Test Corpus

A curated corpus of XML documents covering the full spectrum of valid,
invalid, and edge-case inputs.

#### Valid XML (50+ files)

| Category | Count | Examples |
|----------|-------|---------|
| Basic structure | 8 | Single element, nested, self-closing |
| Attributes | 8 | Single/double quotes, entities, empty values |
| Text content | 6 | Plain, entities, mixed content |
| CDATA | 4 | Simple, special chars, empty |
| Comments | 4 | Single-line, multi-line, adjacent |
| Declarations | 4 | Standard, with encoding, standalone |
| Mixed | 6 | Real-world XML snippets |
| Deep nesting | 4 | 10, 50, 99, 100 levels |
| Wide trees | 3 | 100, 1000, 10000 siblings |
| Large files | 3 | 100KB, 1MB, 10MB generated |

#### Invalid XML (30+ files)

| Category | Count | Examples |
|----------|-------|---------|
| Unclosed tags | 5 | Element, comment, CDATA, attribute, declaration |
| Mismatched tags | 4 | Different names, case sensitivity |
| Invalid entities | 4 | Unknown `&foo;`, truncated `&#`, invalid codepoint |
| Malformed attributes | 4 | No quotes, no value, no `=` |
| Depth exceeded | 3 | 101, 200, 1000 levels |
| Invalid characters | 4 | Null byte, control chars, invalid UTF-8 |
| Empty/whitespace | 3 | Empty string, whitespace only, BOM only |
| Structural | 3 | Text before root, multiple roots (valid for TinyXML2) |

#### Unicode XML (20+ files)

| Category | Count | Examples |
|----------|-------|---------|
| CJK content | 4 | Chinese, Japanese, Korean text in elements/attrs |
| Arabic/Hebrew | 3 | RTL text content |
| Emoji | 3 | Emoji in text and attribute values |
| Combining chars | 3 | Diacritical marks, combining sequences |
| BMP boundary | 3 | U+FFFD, U+FFFE, supplementary plane chars |
| Mixed scripts | 4 | Multi-script documents |

### 3. Fuzz Targets

Using `cargo-fuzz` with libFuzzer for continuous fuzzing:

#### Target: `parse_fuzz`

```rust
#![no_main]
use libfuzzer_sys::fuzz_target;
use tinyxml2::Document;

fuzz_target!(|data: &[u8]| {
    if let Ok(xml) = std::str::from_utf8(data) {
        let mut doc = Document::new();
        let _ = doc.parse(xml);
        // Must not panic, must not leak
    }
});
```

#### Target: `roundtrip_fuzz`

```rust
fuzz_target!(|data: &[u8]| {
    if let Ok(xml) = std::str::from_utf8(data) {
        let mut doc = Document::new();
        if doc.parse(xml).is_ok() {
            let output1 = doc.to_string();
            let mut doc2 = Document::new();
            if doc2.parse(&output1).is_ok() {
                let output2 = doc2.to_string();
                assert_eq!(output1, output2, "Round-trip instability");
            }
        }
    }
});
```

#### Target: `serialize_fuzz`

```rust
fuzz_target!(|data: &[u8]| {
    if let Ok(xml) = std::str::from_utf8(data) {
        let mut doc = Document::new();
        if doc.parse(xml).is_ok() {
            // Compact must not panic
            let _ = doc.to_string_compact();
            // Pretty must not panic
            let _ = doc.to_string();
        }
    }
});
```

#### Target: `streaming_fuzz`

Fuzz the streaming Printer API with arbitrary operation sequences.

### 4. Property-Based Testing

Using `proptest` for generative testing:

| Property | Description |
|----------|-------------|
| Round-trip stability | `print(parse(print(parse(xml)))) == print(parse(xml))` |
| Entity idempotence | `decode(encode(s)) == s` for all valid strings |
| Clone independence | Mutations to `deep_clone(n)` don't affect original |
| Navigation consistency | `parent(first_child(n)) == n` when first_child exists |
| Attribute ordering | Attributes maintain insertion order through round-trip |

### 5. Benchmark Suite

Using `criterion.rs` for statistically rigorous benchmarks:

#### Parse Benchmarks

| Benchmark | Input | Measures |
|-----------|-------|----------|
| `parse_small` | 1KB XML | Throughput, latency |
| `parse_medium` | 100KB XML | Throughput, latency |
| `parse_large` | 10MB XML | Throughput, latency |
| `parse_deep` | 100-level nested | Stack/depth performance |
| `parse_wide` | 10K siblings | Allocation performance |
| `parse_attributes` | 100 attributes per element | Attribute parsing speed |

#### Serialize Benchmarks

| Benchmark | Input | Measures |
|-----------|-------|----------|
| `print_pretty_small` | 1KB DOM | Pretty-print throughput |
| `print_pretty_large` | 10MB DOM | Pretty-print throughput |
| `print_compact_small` | 1KB DOM | Compact throughput |
| `print_compact_large` | 10MB DOM | Compact throughput |
| `streaming_build` | 1000 elements | Streaming API throughput |

#### Traversal Benchmarks

| Benchmark | Input | Measures |
|-----------|-------|----------|
| `traverse_children` | Wide tree | Iterator throughput |
| `traverse_descendants` | Deep tree | Recursive traversal |
| `traverse_visitor` | Mixed tree | Visitor pattern overhead |
| `navigate_handle` | Complex tree | Handle chain performance |

#### Memory Benchmarks

| Benchmark | Input | Measures |
|-----------|-------|----------|
| `memory_per_node` | Various sizes | Bytes per node |
| `arena_alloc_dealloc` | 100K cycles | Allocation throughput |
| `memory_peak` | Large document | Peak RSS |

#### TinyXML2 Comparison Benchmarks

| Benchmark | Comparison |
|-----------|------------|
| `vs_tinyxml2_parse` | Parse same file, compare wall time |
| `vs_tinyxml2_print` | Serialize same DOM, compare wall time |
| `vs_tinyxml2_memory` | Same DOM, compare memory usage |

These comparison benchmarks compile and link TinyXML2 C++ via `cc` crate,
running identical operations on both libraries in the same process for
accurate comparison.

---

## CI Integration

### Fuzz Scheduling

```yaml
# .github/workflows/fuzz.yml
name: Fuzz
on:
  schedule:
    - cron: '0 0 * * *'  # Nightly
  workflow_dispatch:

jobs:
  fuzz:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        target: [parse_fuzz, roundtrip_fuzz, serialize_fuzz, streaming_fuzz]
    steps:
      - uses: actions/checkout@v4
      - run: cargo install cargo-fuzz
      - run: cargo fuzz run ${{ matrix.target }} -- -max_total_time=300
```

### Benchmark Tracking

```yaml
# .github/workflows/bench.yml
name: Benchmarks
on:
  push:
    branches: [main]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo bench -- --output-format bencher | tee output.txt
      - uses: benchmark-action/github-action-benchmark@v1
        with:
          tool: cargo
          output-file-path: output.txt
```

---

## Estimated Test Plan

| Category | Est. Tests | Description |
|----------|-----------|-------------|
| Compatibility (valid) | 50 | Each corpus file parsed and compared |
| Compatibility (invalid) | 30 | Each invalid file, error code comparison |
| Compatibility (unicode) | 20 | Unicode handling comparison |
| Property tests | 15 | proptest properties |
| Fuzz targets | 4 | Fuzz harnesses (duration-based) |
| Benchmark baselines | 20 | Initial benchmark measurements |
| Regression tests | 10 | Tests for bugs found by fuzzing |

**Estimated Total:** ~149 test cases + 4 fuzz targets + 20 benchmarks

---

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| TinyXML2 C++ build complexity | CI | Use pre-built reference outputs as fallback |
| Fuzz corpus quality | Coverage | Seed with real-world XML from open-source projects |
| Benchmark noise | Reliability | Use criterion's statistical analysis; run on dedicated CI |
| Unicode edge cases | Compatibility | Extensive Unicode corpus; test on multiple platforms |
| Large file test time | CI speed | Gate large-file tests behind feature flag |

---

## Acceptance Criteria

- [ ] 100% of valid XML corpus produces identical DOM to TinyXML2
- [ ] 100% of invalid XML corpus produces matching error codes
- [ ] All fuzz targets run for 5 minutes with zero crashes
- [ ] All property tests pass for 10,000+ iterations
- [ ] Parse benchmarks show performance within 2× of TinyXML2
- [ ] Memory per node ≤ 2× TinyXML2's per-node overhead
- [ ] CI pipeline runs all tests and benchmarks on every push
- [ ] Zero warnings across entire test infrastructure
- [ ] Benchmark results tracked and visualized over time

---

## File Plan

| File | Responsibility |
|------|---------------|
| `tests/compat/` | Compatibility test harness and runner |
| `tests/corpus/valid/` | Valid XML test files |
| `tests/corpus/invalid/` | Invalid XML test files |
| `tests/corpus/unicode/` | Unicode XML test files |
| `fuzz/fuzz_targets/parse_fuzz.rs` | Parse fuzz target |
| `fuzz/fuzz_targets/roundtrip_fuzz.rs` | Round-trip fuzz target |
| `fuzz/fuzz_targets/serialize_fuzz.rs` | Serialize fuzz target |
| `fuzz/fuzz_targets/streaming_fuzz.rs` | Streaming API fuzz target |
| `benches/parse_bench.rs` | Parse benchmarks |
| `benches/print_bench.rs` | Serialization benchmarks |
| `benches/traverse_bench.rs` | Traversal benchmarks |
| `benches/memory_bench.rs` | Memory benchmarks |
| `benches/comparison_bench.rs` | TinyXML2 vs tinyxml2-rs |
| `.github/workflows/fuzz.yml` | Nightly fuzz CI |
| `.github/workflows/bench.yml` | Benchmark tracking CI |

---

## Previous Phase

← [Phase 6: C API Layer](./phase-06.md)

## Next Phase

→ [Phase 8: Documentation, Examples & Release](./phase-08.md)
