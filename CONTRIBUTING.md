# Contributing to tinyxml2-rs

Thank you for your interest in contributing to **tinyxml2-rs**! Every contribution matters — whether it's a bug report, a feature suggestion, a documentation improvement, or a code change.

---

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [How to Contribute](#how-to-contribute)
  - [Reporting Bugs](#reporting-bugs)
  - [Suggesting Features](#suggesting-features)
  - [Contributing Code](#contributing-code)
- [Development Setup](#development-setup)
- [Code Style](#code-style)
- [Testing](#testing)
- [Pull Request Process](#pull-request-process)
- [Commit Convention](#commit-convention)
- [Architecture Reference](#architecture-reference)
- [Getting Help](#getting-help)

---

## Code of Conduct

This project follows the [Contributor Covenant Code of Conduct](CODE_OF_CONDUCT.md). By participating, you agree to uphold a welcoming, inclusive, and harassment-free environment for everyone.

---

## How to Contribute

### Reporting Bugs

Found a bug? We appreciate you taking the time to report it.

1. **Search existing issues** to check if it's already been reported.
2. **Open a new issue** using the bug report template.
3. Include the following information:
   - A clear, descriptive title
   - Steps to reproduce the issue
   - Expected behavior vs. actual behavior
   - The XML input that triggers the problem (if applicable)
   - Your Rust version (`rustc --version`) and OS
   - Any relevant error messages or stack traces

> **Security vulnerabilities** should be reported privately. See [SECURITY.md](SECURITY.md).

### Suggesting Features

Have an idea for an improvement?

1. **Check the [ROADMAP.md](ROADMAP.md)** to see if it's already planned.
2. **Search existing issues** for similar suggestions.
3. **Open a new issue** using the feature request template.
4. Describe:
   - The problem your feature would solve
   - Your proposed solution or API sketch
   - Any alternatives you've considered
   - How it relates to TinyXML2 behavioral compatibility

### Contributing Code

Ready to write code? Great! Here's the workflow:

1. Fork the repository.
2. Create a feature branch from `main` (`git checkout -b feat/my-feature`).
3. Make your changes following the guidelines below.
4. Commit using [conventional commits](#commit-convention).
5. Push to your fork and open a pull request.

---

## Development Setup

### Prerequisites

- **Rust** (stable toolchain — managed via `rust-toolchain.toml`)
- **Git**

### Clone and Build

```bash
# Clone your fork
git clone https://github.com/<your-username>/tinyxml2-rs.git
cd tinyxml2-rs

# Build the entire workspace
cargo build

# Run all tests
cargo test --workspace

# Run clippy lints
cargo clippy --workspace --all-targets -- -D warnings

# Check formatting
cargo fmt --all -- --check
```

### Useful Commands

| Command | Description |
|---------|-------------|
| `cargo build` | Build all crates |
| `cargo test --workspace` | Run all tests across the workspace |
| `cargo clippy --workspace --all-targets -- -D warnings` | Run lint checks |
| `cargo fmt --all` | Format all code |
| `cargo fmt --all -- --check` | Check formatting without modifying files |
| `cargo doc --workspace --no-deps --open` | Build and open documentation |
| `cargo bench -p tinyxml2-bench` | Run benchmarks |

---

## Code Style

We enforce consistent code style across the project:

### Formatting

- All code must be formatted with **`rustfmt`** using the project's [rustfmt.toml](rustfmt.toml).
- Run `cargo fmt --all` before committing.
- CI will reject PRs with formatting issues.

### Linting

- All code must pass **`clippy`** with zero warnings.
- Run `cargo clippy --workspace --all-targets -- -D warnings`.
- If you believe a clippy lint is a false positive, discuss it in your PR before adding an `#[allow(...)]`.

### General Guidelines

- **Prefer safe Rust.** The core `tinyxml2` crate must contain no `unsafe` code. The `tinyxml2-capi` crate may use `unsafe` where required for FFI boundaries.
- **Write idiomatic Rust.** Leverage the type system, use `Result` for fallible operations, and prefer iterators over manual loops.
- **Document public APIs.** Every public type, function, and method must have a doc comment (`///`).
- **Keep functions focused.** Each function should do one thing well.
- **Use meaningful names.** Variable and function names should be descriptive and self-documenting.
- **Preserve existing comments.** Don't remove or alter comments unrelated to your change.

---

## Testing

Testing is critical for a library that targets behavioral compatibility. We maintain several layers of tests:

### Test Categories

| Category | Location | Purpose |
|----------|----------|---------|
| Unit tests | `#[cfg(test)]` modules in source files | Test individual functions and types |
| Integration tests | `tests/` | Test cross-module behavior |
| Conformance tests | `spec/` | Verify behavioral parity with TinyXML2 |
| Fuzz tests | `fuzz/` | Find crashes and edge cases via random input |

### Requirements

- **All new code must have tests.** Aim for comprehensive coverage of both success and failure paths.
- **All existing tests must pass.** Run `cargo test --workspace` and ensure zero failures.
- **Bug fixes must include a regression test.** Add a test that fails without your fix and passes with it.
- **Conformance tests** are particularly important. If your change affects parsing or output behavior, add or update conformance tests in `spec/`.

### Running Tests

```bash
# Run all tests
cargo test --workspace

# Run tests for a specific crate
cargo test -p tinyxml2

# Run a specific test by name
cargo test --workspace -- test_name

# Run tests with output
cargo test --workspace -- --nocapture
```

---

## Pull Request Process

1. **Ensure all checks pass locally** before opening a PR:
   - `cargo build --workspace`
   - `cargo test --workspace`
   - `cargo clippy --workspace --all-targets -- -D warnings`
   - `cargo fmt --all -- --check`

2. **Fill out the PR template** completely:
   - Describe what the PR does and why
   - Reference any related issues (`Fixes #123`, `Closes #456`)
   - Note any breaking changes

3. **Keep PRs focused.** One logical change per PR. If you find unrelated issues, open separate PRs.

4. **Respond to review feedback** promptly. Discussions are how we improve the code together.

5. **Squash commits** if requested. We prefer a clean commit history on `main`.

### PR Checklist

- [ ] Code compiles without warnings
- [ ] All tests pass (`cargo test --workspace`)
- [ ] Clippy passes with no warnings
- [ ] Code is formatted with `rustfmt`
- [ ] Public APIs have doc comments
- [ ] New functionality has tests
- [ ] Relevant documentation is updated
- [ ] Commit messages follow conventional commit format

---

## Commit Convention

We follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
<type>(<scope>): <description>

[optional body]

[optional footer(s)]
```

### Types

| Type | When to Use |
|------|-------------|
| `feat` | A new feature |
| `fix` | A bug fix |
| `docs` | Documentation changes only |
| `style` | Formatting, whitespace — no code logic changes |
| `refactor` | Code change that neither fixes a bug nor adds a feature |
| `perf` | A code change that improves performance |
| `test` | Adding or updating tests |
| `build` | Changes to the build system or dependencies |
| `ci` | Changes to CI configuration |
| `chore` | Other changes that don't modify `src` or `test` files |

### Scopes

Use the crate name as the scope: `core`, `capi`, `bench`, or `workspace` for cross-cutting changes.

### Examples

```
feat(core): implement arena-based node allocation
fix(core): handle self-closing tags with whitespace
docs: update README with quick start example
test(core): add conformance tests for entity decoding
refactor(capi): simplify FFI error conversion
```

---

## Architecture Reference

Before contributing, familiarize yourself with the project's architecture:

- **[ARCHITECTURE.md](ARCHITECTURE.md)** — Detailed overview of the crate structure, module layout, and design decisions.
- **[ROADMAP.md](ROADMAP.md)** — Current development phase and planned milestones.

### Key Concepts

- **Generational Arena**: The DOM uses a generational arena for node storage. Nodes are referenced by `NodeId` handles rather than pointers.
- **Behavioral Compatibility**: We treat TinyXML2 as a specification. When in doubt about behavior, match what TinyXML2 does.
- **Recursive Descent Parser**: The parser is hand-written, not generated. It mirrors TinyXML2's parsing strategy.
- **Safe FFI Boundary**: The `tinyxml2-capi` crate contains the `unsafe` boundary. Everything else stays safe.

---

## Getting Help

- **Questions?** Open a [Discussion](https://github.com/teebot/tinyxml2-rs/discussions) on GitHub.
- **Stuck on a contribution?** Mention it in your PR or issue — we're happy to help.
- **Security concerns?** See [SECURITY.md](SECURITY.md).

---

Thank you for helping make tinyxml2-rs better! 🦀
