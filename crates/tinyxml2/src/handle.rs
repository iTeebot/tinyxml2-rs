//! Null-safe DOM navigation handles.
//!
//! [`Handle`] and [`HandleMut`] wrap a `Document` reference and an
//! `Option<NodeId>`, providing fluent navigation chains that propagate
//! `None` gracefully instead of panicking.
//!
//! # Example
//!
//! ```ignore
//! let text = doc.handle(doc.root())
//!     .first_child_element(Some("settings"))
//!     .first_child_element(Some("resolution"))
//!     .text();
//! ```

use crate::arena::NodeId;
use crate::document::Document;
use crate::node::NodeKind;

/// An immutable, null-safe DOM navigation handle.
///
/// Wraps a `&Document` and an `Option<NodeId>`. Navigation methods
/// return a new `Handle`, propagating `None` if the current node is null.
/// This enables fluent chains without nested `Option` matching.
#[derive(Debug, Clone, Copy)]
pub struct Handle<'a> {
    doc: &'a Document,
    node: Option<NodeId>,
}

impl<'a> Handle<'a> {
    /// Creates a new `Handle` referencing the given node.
    pub fn new(doc: &'a Document, node: NodeId) -> Self {
        Self {
            doc,
            node: Some(node),
        }
    }

    /// Creates a null `Handle` (no node).
    pub fn null(doc: &'a Document) -> Self {
        Self { doc, node: None }
    }

    /// Returns `true` if this handle references no node.
    pub fn is_null(&self) -> bool {
        self.node.is_none()
    }

    /// Returns the underlying `NodeId`, if the handle is non-null.
    pub fn to_node(&self) -> Option<NodeId> {
        self.node
    }

    /// Returns the underlying `NodeId` if the handle references an Element node.
    pub fn to_element(&self) -> Option<NodeId> {
        let id = self.node?;
        let data = self.doc.arena.get(id)?;
        match &data.kind {
            NodeKind::Element(_) => Some(id),
            _ => None,
        }
    }

    /// Returns the text content of the first child Text node, if the
    /// handle references an Element.
    pub fn text(&self) -> Option<&'a str> {
        let id = self.to_element()?;
        self.doc.get_text(id)
    }

    /// Returns the string value of the named attribute, if the handle
    /// references an Element with that attribute.
    pub fn attribute(&self, name: &str) -> Option<&'a str> {
        let id = self.to_element()?;
        self.doc.attribute(id, name)
    }

    // --- Navigation (returns new Handle, propagating None) ---

    /// Navigates to the parent of the current node.
    #[must_use]
    pub fn parent(&self) -> Handle<'a> {
        Handle {
            doc: self.doc,
            node: self.node.and_then(|id| self.doc.parent(id)),
        }
    }

    /// Navigates to the first child of the current node.
    #[must_use]
    pub fn first_child(&self) -> Handle<'a> {
        Handle {
            doc: self.doc,
            node: self.node.and_then(|id| self.doc.first_child(id)),
        }
    }

    /// Navigates to the last child of the current node.
    #[must_use]
    pub fn last_child(&self) -> Handle<'a> {
        Handle {
            doc: self.doc,
            node: self.node.and_then(|id| self.doc.last_child(id)),
        }
    }

    /// Navigates to the next sibling of the current node.
    #[must_use]
    pub fn next_sibling(&self) -> Handle<'a> {
        Handle {
            doc: self.doc,
            node: self.node.and_then(|id| self.doc.next_sibling(id)),
        }
    }

    /// Navigates to the previous sibling of the current node.
    #[must_use]
    pub fn prev_sibling(&self) -> Handle<'a> {
        Handle {
            doc: self.doc,
            node: self.node.and_then(|id| self.doc.prev_sibling(id)),
        }
    }

    /// Navigates to the first child element, optionally matching a tag name.
    #[must_use]
    pub fn first_child_element(&self, name: Option<&str>) -> Handle<'a> {
        Handle {
            doc: self.doc,
            node: self
                .node
                .and_then(|id| self.doc.first_child_element(id, name)),
        }
    }

    /// Navigates to the next sibling element, optionally matching a tag name.
    #[must_use]
    pub fn next_sibling_element(&self, name: Option<&str>) -> Handle<'a> {
        Handle {
            doc: self.doc,
            node: self
                .node
                .and_then(|id| self.doc.next_sibling_element(id, name)),
        }
    }
}

/// A mutable, null-safe DOM navigation handle.
///
/// Wraps a `&mut Document` and an `Option<NodeId>`. Navigation methods
/// **consume** `self` (take by value) to comply with Rust's `&mut`
/// aliasing rules, enabling fluent chains.
///
/// # Example
///
/// ```ignore
/// let node_id = doc.handle_mut(doc.root())
///     .first_child()
///     .first_child()
///     .to_node();
/// ```
#[derive(Debug)]
pub struct HandleMut<'a> {
    doc: &'a mut Document,
    node: Option<NodeId>,
}

impl<'a> HandleMut<'a> {
    /// Creates a new `HandleMut` referencing the given node.
    pub fn new(doc: &'a mut Document, node: NodeId) -> Self {
        Self {
            doc,
            node: Some(node),
        }
    }

    /// Creates a null `HandleMut` (no node).
    pub fn null(doc: &'a mut Document) -> Self {
        Self { doc, node: None }
    }

    /// Returns `true` if this handle references no node.
    pub fn is_null(&self) -> bool {
        self.node.is_none()
    }

    /// Returns the underlying `NodeId`, if the handle is non-null.
    pub fn to_node(&self) -> Option<NodeId> {
        self.node
    }

    /// Returns the underlying `NodeId` if the handle references an Element node.
    pub fn to_element(&self) -> Option<NodeId> {
        let id = self.node?;
        let data = self.doc.arena.get(id)?;
        match &data.kind {
            NodeKind::Element(_) => Some(id),
            _ => None,
        }
    }

    /// Returns the text content of the first child Text node, if the
    /// handle references an Element.
    pub fn text(&self) -> Option<&str> {
        let id = self.to_element()?;
        self.doc.get_text(id)
    }

    /// Returns the string value of the named attribute, if the handle
    /// references an Element with that attribute.
    pub fn attribute(&self, name: &str) -> Option<&str> {
        let id = self.to_element()?;
        self.doc.attribute(id, name)
    }

    /// Consumes this handle and returns the underlying mutable `Document` reference.
    pub fn into_doc(self) -> &'a mut Document {
        self.doc
    }

    // --- Navigation (consumes self, returns new HandleMut) ---

    /// Navigates to the parent of the current node.
    #[must_use]
    pub fn parent(self) -> HandleMut<'a> {
        let node = self.node.and_then(|id| self.doc.parent(id));
        HandleMut {
            doc: self.doc,
            node,
        }
    }

    /// Navigates to the first child of the current node.
    #[must_use]
    pub fn first_child(self) -> HandleMut<'a> {
        let node = self.node.and_then(|id| self.doc.first_child(id));
        HandleMut {
            doc: self.doc,
            node,
        }
    }

    /// Navigates to the last child of the current node.
    #[must_use]
    pub fn last_child(self) -> HandleMut<'a> {
        let node = self.node.and_then(|id| self.doc.last_child(id));
        HandleMut {
            doc: self.doc,
            node,
        }
    }

    /// Navigates to the next sibling of the current node.
    #[must_use]
    pub fn next_sibling(self) -> HandleMut<'a> {
        let node = self.node.and_then(|id| self.doc.next_sibling(id));
        HandleMut {
            doc: self.doc,
            node,
        }
    }

    /// Navigates to the previous sibling of the current node.
    #[must_use]
    pub fn prev_sibling(self) -> HandleMut<'a> {
        let node = self.node.and_then(|id| self.doc.prev_sibling(id));
        HandleMut {
            doc: self.doc,
            node,
        }
    }

    /// Navigates to the first child element, optionally matching a tag name.
    #[must_use]
    pub fn first_child_element(self, name: Option<&str>) -> HandleMut<'a> {
        let node = self
            .node
            .and_then(|id| self.doc.first_child_element(id, name));
        HandleMut {
            doc: self.doc,
            node,
        }
    }

    /// Navigates to the next sibling element, optionally matching a tag name.
    #[must_use]
    pub fn next_sibling_element(self, name: Option<&str>) -> HandleMut<'a> {
        let node = self
            .node
            .and_then(|id| self.doc.next_sibling_element(id, name));
        HandleMut {
            doc: self.doc,
            node,
        }
    }
}
