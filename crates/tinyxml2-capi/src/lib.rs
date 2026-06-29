//! C FFI compatibility layer for tinyxml2-rs.
//!
//! This crate provides `extern "C"` functions that expose the tinyxml2 Rust API
//! through a C-compatible ABI. It produces both static and shared libraries.
//!
//! # Safety
//!
//! This crate necessarily uses `unsafe` at the FFI boundary. All public functions
//! validate inputs and catch panics to prevent undefined behavior across the
//! FFI boundary.

// C API implementation will be added in Phase 6.
