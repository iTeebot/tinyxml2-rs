//! C-compatible types for the tinyxml2 FFI layer.
//!
//! This module defines all `#[repr(C)]` types, opaque wrapper structs, and
//! conversion utilities used by the `extern "C"` functions in [`crate`].

use std::ffi::CString;

use tinyxml2::{Document, NodeId, NodeKind, ParseErrorKind, XmlError, XmlPrinter};

// ============================================================
// TxNodeId — C-compatible node identifier
// ============================================================

/// A C-compatible node identifier.
///
/// This is a `#[repr(C)]` mirror of the internal `NodeId` type. It carries
/// both an arena index and a generation counter for use-after-free detection.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TxNodeId {
    /// Index into the document's node arena.
    pub index: u32,
    /// Generation counter for validity checking.
    pub generation: u32,
}

/// Sentinel value representing a null / invalid node.
///
/// Any function that would return "no node" returns this sentinel.
/// Use [`tx_node_is_null`](crate::tx_node_is_null) to test for it.
pub const TX_NULL_NODE: TxNodeId = TxNodeId {
    index: u32::MAX,
    generation: 0,
};

impl TxNodeId {
    /// Converts a Rust `NodeId` into a C-compatible `TxNodeId`.
    #[inline]
    pub fn from_node_id(id: NodeId) -> Self {
        let (index, generation) = id.raw_parts();
        Self { index, generation }
    }

    /// Converts this C-compatible `TxNodeId` back into a Rust `NodeId`.
    #[inline]
    pub fn to_node_id(self) -> NodeId {
        NodeId::from_raw_parts(self.index, self.generation)
    }

    /// Returns `true` if this node ID is the null sentinel.
    #[inline]
    pub fn is_null(self) -> bool {
        self == TX_NULL_NODE
    }

    /// Converts an `Option<NodeId>` to a `TxNodeId`, mapping `None` to
    /// [`TX_NULL_NODE`].
    #[inline]
    pub fn from_option(opt: Option<NodeId>) -> Self {
        match opt {
            Some(id) => Self::from_node_id(id),
            None => TX_NULL_NODE,
        }
    }
}

// ============================================================
// TxNodeType — C-compatible node kind enum
// ============================================================

/// C-compatible representation of the XML node type.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TxNodeType {
    /// The document root node.
    TxNodeDocument = 0,
    /// An XML element (tag).
    TxNodeElement = 1,
    /// A text content node.
    TxNodeText = 2,
    /// An XML comment (`<!-- ... -->`).
    TxNodeComment = 3,
    /// An XML declaration (`<?xml ... ?>`).
    TxNodeDeclaration = 4,
    /// An unrecognized XML construct.
    TxNodeUnknown = 5,
}

impl TxNodeType {
    /// Converts a Rust `NodeKind` reference into a `TxNodeType`.
    pub fn from_node_kind(kind: &NodeKind) -> Self {
        match kind {
            NodeKind::Document => Self::TxNodeDocument,
            NodeKind::Element(_) => Self::TxNodeElement,
            NodeKind::Text(_) => Self::TxNodeText,
            NodeKind::Comment(_) => Self::TxNodeComment,
            NodeKind::Declaration(_) => Self::TxNodeDeclaration,
            NodeKind::Unknown(_) => Self::TxNodeUnknown,
        }
    }
}

// ============================================================
// TxError — C-compatible error code enum
// ============================================================

/// C-compatible error code enum matching `TinyXML2` error values.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TxError {
    /// Operation completed successfully.
    TxSuccess = 0,
    /// The document was empty or contained no XML content.
    TxErrorEmptyDocument = 1,
    /// Error while parsing an element.
    TxErrorParsingElement = 2,
    /// Error while parsing an attribute.
    TxErrorParsingAttribute = 3,
    /// Error while parsing text content.
    TxErrorParsingText = 4,
    /// Error while parsing a CDATA section.
    TxErrorParsingCdata = 5,
    /// Error while parsing a comment.
    TxErrorParsingComment = 6,
    /// Error while parsing a declaration.
    TxErrorParsingDeclaration = 7,
    /// Error while parsing an unknown construct.
    TxErrorParsingUnknown = 8,
    /// An element's closing tag did not match its opening tag.
    TxErrorMismatchedElement = 9,
    /// The specified file was not found.
    TxErrorFileNotFound = 10,
    /// An error occurred while reading/writing a file.
    TxErrorFileRead = 11,
    /// The maximum element nesting depth was exceeded.
    TxErrorElementDepthExceeded = 12,
    /// The requested attribute does not exist.
    TxErrorNoAttribute = 13,
    /// The attribute value could not be converted to the requested type.
    TxErrorWrongAttributeType = 14,
    /// Text content could not be converted to the requested type.
    TxErrorCanNotConvertText = 15,
    /// No text node was found as a child of the element.
    TxErrorNoTextNode = 16,
    /// The provided node ID is invalid or refers to a freed node.
    TxErrorInvalidNodeId = 17,
}

impl TxError {
    /// Converts a Rust `XmlError` into a C-compatible `TxError` code.
    pub fn from_xml_error(err: &XmlError) -> Self {
        match err {
            XmlError::EmptyDocument => Self::TxErrorEmptyDocument,
            XmlError::Parse { kind, .. } => match kind {
                ParseErrorKind::Element | ParseErrorKind::General => Self::TxErrorParsingElement,
                ParseErrorKind::Attribute => Self::TxErrorParsingAttribute,
                ParseErrorKind::Text => Self::TxErrorParsingText,
                ParseErrorKind::Cdata => Self::TxErrorParsingCdata,
                ParseErrorKind::Comment => Self::TxErrorParsingComment,
                ParseErrorKind::Declaration => Self::TxErrorParsingDeclaration,
                ParseErrorKind::Unknown => Self::TxErrorParsingUnknown,
            },
            XmlError::MismatchedElement { .. } => Self::TxErrorMismatchedElement,
            XmlError::Io(io_err) => {
                if io_err.kind() == std::io::ErrorKind::NotFound {
                    Self::TxErrorFileNotFound
                } else {
                    Self::TxErrorFileRead
                }
            }
            XmlError::ElementDepthExceeded { .. } => Self::TxErrorElementDepthExceeded,
            XmlError::NoAttribute => Self::TxErrorNoAttribute,
            XmlError::WrongAttributeType => Self::TxErrorWrongAttributeType,
            XmlError::CanNotConvertText => Self::TxErrorCanNotConvertText,
            XmlError::NoTextNode => Self::TxErrorNoTextNode,
            XmlError::InvalidNodeId => Self::TxErrorInvalidNodeId,
        }
    }

    /// Converts a `Result<T>` into a `TxError`, mapping `Ok` to `TxSuccess`.
    pub fn from_result<T>(result: &tinyxml2::Result<T>) -> Self {
        match result {
            Ok(_) => Self::TxSuccess,
            Err(e) => Self::from_xml_error(e),
        }
    }
}

// ============================================================
// TxDocument — Opaque document wrapper
// ============================================================

/// Opaque wrapper around a `Document` for FFI use.
///
/// This struct holds the actual document plus cached `CString` values so that
/// `const char*` pointers returned across the FFI boundary remain valid until
/// the document is mutated or freed.
pub struct TxDocument {
    /// The inner tinyxml2 document.
    pub(crate) doc: Document,
    /// Cached pretty-printed serialization.
    pub(crate) cached_to_string: Option<CString>,
    /// Cached compact serialization.
    pub(crate) cached_to_string_compact: Option<CString>,
    /// Cached error name string.
    pub(crate) cached_error_name: Option<CString>,
    /// Accumulated node-level string cache. Pointers into these `CString`
    /// values remain valid until the next mutating operation or until the
    /// document is freed.
    pub(crate) string_cache: Vec<CString>,
}

impl TxDocument {
    /// Creates a new wrapper around a fresh, empty `Document`.
    pub fn new() -> Self {
        Self {
            doc: Document::new(),
            cached_to_string: None,
            cached_to_string_compact: None,
            cached_error_name: None,
            string_cache: Vec::new(),
        }
    }

    /// Invalidates all cached strings. Called before any mutating operation.
    pub fn invalidate_caches(&mut self) {
        self.cached_to_string = None;
        self.cached_to_string_compact = None;
        self.cached_error_name = None;
        self.string_cache.clear();
    }
}

impl Default for TxDocument {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================
// TxPrinter — Opaque printer wrapper
// ============================================================

/// Opaque wrapper around an `XmlPrinter` for FFI use.
///
/// Caches the result string so that the `const char*` pointer returned by
/// [`tx_printer_result`](crate::tx_printer_result) remains valid.
pub struct TxPrinter {
    /// The inner tinyxml2 printer.
    pub(crate) printer: XmlPrinter,
    /// Cached result string for FFI return.
    pub(crate) cached_result: Option<CString>,
}

impl TxPrinter {
    /// Creates a new printer wrapper.
    pub fn new(compact: bool) -> Self {
        let printer = if compact {
            XmlPrinter::new_compact()
        } else {
            XmlPrinter::new()
        };
        Self {
            printer,
            cached_result: None,
        }
    }
}
