# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.14] - 2026-06-30

### Added

- `XmlVisitor` trait defining standard DOM traversal callbacks (`visit_enter_document`, `visit_exit_document`, `visit_enter_element`, `visit_exit_element`, `visit_text`, `visit_comment`, `visit_declaration`, `visit_unknown`)
- `XmlPrinter` struct supporting pretty-printed (indented, newline-formatted) and compact XML serialization
- Stateful streaming (push) API on `XmlPrinter` (`open_element`, `push_attribute`, `close_element`, `push_text`, `push_text_raw`, `push_cdata`, `push_comment`, `push_declaration`, `push_unknown`, `push_header`)
- DOM-driven serialization methods on `Document` (`to_string`, `to_string_compact`, `save_file`, `save_file_compact`, `save_writer`, `save_writer_compact`)
- `std::fmt::Display` implementation for `Document`
- Comprehensive unit and round-trip integration test suites verifying printer correctness and parse-print-parse equivalence
- Automated release publication GitHub Actions workflow (`publish.yml`) to publish crates to crates.io on tag push

## [0.1.13] - 2026-06-29

### Added

- Project scaffolding with 3-crate workspace structure
- `XmlError` enum with all TinyXML2-compatible error codes
- `ParseErrorKind` enum for parse error classification
- XML entity encoding and decoding (5 predefined entities + numeric character references)
- Zero-allocation fast paths for entity handling (`Cow<str>`)
- Character classification utilities (XML name characters, whitespace)
- UTF-8 BOM detection and stripping
- Whitespace collapsing utility
- XML name parsing utility
- Generational arena allocator with use-after-free detection
- `NodeId` type with generation-checked access
- Arena iterators with `IntoIterator` implementations
- `Whitespace` enum (Preserve, Collapse, Pedantic)
- `ParseOptions` builder struct
- Workspace-level lint configuration (unsafe_code = forbid, clippy pedantic)
- CI pipeline (GitHub Actions: test matrix, lint, MSRV)
- Complete TinyXML2 API specification reference
- Project documentation (README, CONTRIBUTING, ARCHITECTURE, ROADMAP, SECURITY, CODE_OF_CONDUCT)
- Issue templates and PR template
- Editor and formatter configuration (.editorconfig, rustfmt.toml, .clippy.toml)
