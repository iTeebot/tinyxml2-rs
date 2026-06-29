# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
