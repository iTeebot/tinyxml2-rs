//! XML Visitor pattern for DOM traversal.
//!
//! Mirrors TinyXML2's `XMLVisitor` callbacks.

use crate::arena::NodeId;
use crate::document::Document;

/// Trait for walking the DOM tree.
///
/// Implement this trait to define custom traversal operations. The document
/// drives traversal via [`Document::accept`].
///
/// Returning `false` from any callback halts the traversal immediately.
pub trait XmlVisitor {
    /// Called when entering the Document (before any children).
    fn visit_enter_document(&mut self, _doc: &Document) -> bool {
        true
    }

    /// Called when exiting the Document (after all children).
    fn visit_exit_document(&mut self, _doc: &Document) -> bool {
        true
    }

    /// Called when entering an Element node (before children).
    fn visit_enter_element(&mut self, _doc: &Document, _element: NodeId) -> bool {
        true
    }

    /// Called when leaving an Element node (after children).
    fn visit_exit_element(&mut self, _doc: &Document, _element: NodeId) -> bool {
        true
    }

    /// Called when visiting a Text node.
    fn visit_text(&mut self, _doc: &Document, _text: NodeId) -> bool {
        true
    }

    /// Called when visiting a Comment node.
    fn visit_comment(&mut self, _doc: &Document, _comment: NodeId) -> bool {
        true
    }

    /// Called when visiting a Declaration node.
    fn visit_declaration(&mut self, _doc: &Document, _declaration: NodeId) -> bool {
        true
    }

    /// Called when visiting an Unknown node.
    fn visit_unknown(&mut self, _doc: &Document, _unknown: NodeId) -> bool {
        true
    }
}
