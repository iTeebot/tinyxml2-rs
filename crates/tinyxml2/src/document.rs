//! XML document container and DOM tree manipulation.

use crate::arena::{Arena, NodeId};
use crate::error::{Result, XmlError};
use crate::node::{Attribute, ElementData, NodeData, NodeKind, TextData};

/// The main XML document container.
///
/// Owns the node arena, root node, and tracks the current parse state/error.
#[derive(Debug)]
pub struct Document {
    pub(crate) arena: Arena<NodeData>,
    root: NodeId,
    error: Option<XmlError>,
}

impl Document {
    /// Creates a new, empty XML document containing only a root Document node.
    #[must_use]
    pub fn new() -> Self {
        let mut arena = Arena::new();
        let root = arena.alloc(NodeData::new(NodeKind::Document, 1));
        Self {
            arena,
            root,
            error: None,
        }
    }

    /// Returns the root `NodeId` of the document.
    #[must_use]
    pub const fn root(&self) -> NodeId {
        self.root
    }

    /// Returns the current error state of the document, if any.
    #[must_use]
    pub fn error(&self) -> Option<XmlError> {
        self.error.clone()
    }

    /// Returns the line number of the last error, if available.
    #[must_use]
    pub fn error_line(&self) -> Option<u32> {
        self.error.as_ref().and_then(XmlError::line)
    }

    /// Sets the error state of the document.
    #[allow(dead_code)]
    pub(crate) fn set_error(&mut self, err: XmlError) {
        self.error = Some(err);
    }

    /// Resets the document to an empty state, invalidating all existing `NodeId`s.
    pub fn clear(&mut self) {
        self.arena.clear();
        self.error = None;
        self.root = self.arena.alloc(NodeData::new(NodeKind::Document, 1));
    }

    // --- Factory Methods ---

    /// Creates a new detached Element node in the document's arena.
    pub fn new_element(&mut self, name: &str) -> NodeId {
        let kind = NodeKind::Element(ElementData {
            name: name.to_string(),
            attributes: Vec::new(),
        });
        self.arena.alloc(NodeData::new(kind, 1))
    }

    /// Creates a new detached Text node in the document's arena.
    pub fn new_text(&mut self, text: &str) -> NodeId {
        let kind = NodeKind::Text(TextData {
            content: text.to_string(),
            is_cdata: false,
        });
        self.arena.alloc(NodeData::new(kind, 1))
    }

    /// Creates a new detached CDATA Text node in the document's arena.
    pub fn new_cdata(&mut self, text: &str) -> NodeId {
        let kind = NodeKind::Text(TextData {
            content: text.to_string(),
            is_cdata: true,
        });
        self.arena.alloc(NodeData::new(kind, 1))
    }

    /// Creates a new detached Comment node in the document's arena.
    pub fn new_comment(&mut self, text: &str) -> NodeId {
        let kind = NodeKind::Comment(text.to_string());
        self.arena.alloc(NodeData::new(kind, 1))
    }

    /// Creates a new detached Declaration node in the document's arena.
    pub fn new_declaration(&mut self, decl: &str) -> NodeId {
        let kind = NodeKind::Declaration(ElementData {
            name: decl.to_string(),
            attributes: Vec::new(),
        });
        self.arena.alloc(NodeData::new(kind, 1))
    }

    /// Creates a new detached Unknown node in the document's arena.
    pub fn new_unknown(&mut self, text: &str) -> NodeId {
        let kind = NodeKind::Unknown(text.to_string());
        self.arena.alloc(NodeData::new(kind, 1))
    }

    // --- Navigation APIs ---

    /// Returns the parent of the specified node, if it exists.
    #[must_use]
    pub fn parent(&self, node: NodeId) -> Option<NodeId> {
        self.arena.get(node).and_then(|d| d.parent)
    }

    /// Returns the first child of the specified node, if it exists.
    #[must_use]
    pub fn first_child(&self, node: NodeId) -> Option<NodeId> {
        self.arena.get(node).and_then(|d| d.first_child)
    }

    /// Returns the last child of the specified node, if it exists.
    #[must_use]
    pub fn last_child(&self, node: NodeId) -> Option<NodeId> {
        self.arena.get(node).and_then(|d| d.last_child)
    }

    /// Returns the previous sibling of the specified node, if it exists.
    #[must_use]
    pub fn prev_sibling(&self, node: NodeId) -> Option<NodeId> {
        self.arena.get(node).and_then(|d| d.prev_sibling)
    }

    /// Returns the next sibling of the specified node, if it exists.
    #[must_use]
    pub fn next_sibling(&self, node: NodeId) -> Option<NodeId> {
        self.arena.get(node).and_then(|d| d.next_sibling)
    }

    /// Returns the first child Element of the specified node, optionally matching a tag name.
    #[must_use]
    pub fn first_child_element(&self, node: NodeId, name: Option<&str>) -> Option<NodeId> {
        let mut current = self.first_child(node);
        while let Some(curr) = current {
            if let Some(data) = self.arena.get(curr) {
                if let NodeKind::Element(el_data) = &data.kind {
                    if name.is_none_or(|n| el_data.name == n) {
                        return Some(curr);
                    }
                }
            }
            current = self.next_sibling(curr);
        }
        None
    }

    /// Returns the last child Element of the specified node, optionally matching a tag name.
    #[must_use]
    pub fn last_child_element(&self, node: NodeId, name: Option<&str>) -> Option<NodeId> {
        let mut current = self.last_child(node);
        while let Some(curr) = current {
            if let Some(data) = self.arena.get(curr) {
                if let NodeKind::Element(el_data) = &data.kind {
                    if name.is_none_or(|n| el_data.name == n) {
                        return Some(curr);
                    }
                }
            }
            current = self.prev_sibling(curr);
        }
        None
    }

    /// Returns the next sibling Element of the specified node, optionally matching a tag name.
    #[must_use]
    pub fn next_sibling_element(&self, node: NodeId, name: Option<&str>) -> Option<NodeId> {
        let mut current = self.next_sibling(node);
        while let Some(curr) = current {
            if let Some(data) = self.arena.get(curr) {
                if let NodeKind::Element(el_data) = &data.kind {
                    if name.is_none_or(|n| el_data.name == n) {
                        return Some(curr);
                    }
                }
            }
            current = self.next_sibling(curr);
        }
        None
    }

    /// Returns the previous sibling Element of the specified node, optionally matching a tag name.
    #[must_use]
    pub fn prev_sibling_element(&self, node: NodeId, name: Option<&str>) -> Option<NodeId> {
        let mut current = self.prev_sibling(node);
        while let Some(curr) = current {
            if let Some(data) = self.arena.get(curr) {
                if let NodeKind::Element(el_data) = &data.kind {
                    if name.is_none_or(|n| el_data.name == n) {
                        return Some(curr);
                    }
                }
            }
            current = self.prev_sibling(curr);
        }
        None
    }

    /// Returns the root Element of the document (the first element child of the root document node).
    #[must_use]
    pub fn root_element(&self) -> Option<NodeId> {
        self.first_child_element(self.root, None)
    }

    // --- Invariant Checks & Tree Linkage Helpers ---

    /// Returns `true` if `ancestor` is an ancestor of `descendant` (or the same node).
    fn is_ancestor(&self, ancestor: NodeId, mut descendant: NodeId) -> bool {
        if ancestor == descendant {
            return true;
        }
        while let Some(parent) = self.parent(descendant) {
            if parent == ancestor {
                return true;
            }
            descendant = parent;
        }
        false
    }

    /// Unlinks a node from its parent and siblings.
    fn unlink(&mut self, node: NodeId) -> Result<()> {
        let data = self.arena.get(node).ok_or(XmlError::InvalidNodeId)?.clone();
        if let Some(parent) = data.parent {
            let p_data = self.arena.get_mut(parent).ok_or(XmlError::InvalidNodeId)?;
            if p_data.first_child == Some(node) {
                p_data.first_child = data.next_sibling;
            }
            if p_data.last_child == Some(node) {
                p_data.last_child = data.prev_sibling;
            }
        }
        if let Some(prev) = data.prev_sibling {
            if let Some(prev_node) = self.arena.get_mut(prev) {
                prev_node.next_sibling = data.next_sibling;
            }
        }
        if let Some(next) = data.next_sibling {
            if let Some(next_node) = self.arena.get_mut(next) {
                next_node.prev_sibling = data.prev_sibling;
            }
        }

        let node_mut = self.arena.get_mut(node).ok_or(XmlError::InvalidNodeId)?;
        node_mut.parent = None;
        node_mut.prev_sibling = None;
        node_mut.next_sibling = None;
        Ok(())
    }

    // --- Tree Mutation APIs ---

    /// Inserts `child` as the last child of `parent`.
    pub fn insert_end_child(&mut self, parent: NodeId, child: NodeId) -> Result<NodeId> {
        if !self.arena.contains(parent) || !self.arena.contains(child) {
            return Err(XmlError::InvalidNodeId);
        }
        if self.is_ancestor(child, parent) {
            return Err(XmlError::InvalidNodeId);
        }

        self.unlink(child)?;

        let parent_data = self.arena.get(parent).ok_or(XmlError::InvalidNodeId)?;
        let old_last = parent_data.last_child;

        if let Some(last) = old_last {
            let last_node = self.arena.get_mut(last).ok_or(XmlError::InvalidNodeId)?;
            last_node.next_sibling = Some(child);
        }

        let child_node = self.arena.get_mut(child).ok_or(XmlError::InvalidNodeId)?;
        child_node.parent = Some(parent);
        child_node.prev_sibling = old_last;
        child_node.next_sibling = None;

        let parent_node = self.arena.get_mut(parent).ok_or(XmlError::InvalidNodeId)?;
        if parent_node.first_child.is_none() {
            parent_node.first_child = Some(child);
        }
        parent_node.last_child = Some(child);

        Ok(child)
    }

    /// Inserts `child` as the first child of `parent`.
    pub fn insert_first_child(&mut self, parent: NodeId, child: NodeId) -> Result<NodeId> {
        if !self.arena.contains(parent) || !self.arena.contains(child) {
            return Err(XmlError::InvalidNodeId);
        }
        if self.is_ancestor(child, parent) {
            return Err(XmlError::InvalidNodeId);
        }

        self.unlink(child)?;

        let parent_data = self.arena.get(parent).ok_or(XmlError::InvalidNodeId)?;
        let old_first = parent_data.first_child;

        if let Some(first) = old_first {
            let first_node = self.arena.get_mut(first).ok_or(XmlError::InvalidNodeId)?;
            first_node.prev_sibling = Some(child);
        }

        let child_node = self.arena.get_mut(child).ok_or(XmlError::InvalidNodeId)?;
        child_node.parent = Some(parent);
        child_node.prev_sibling = None;
        child_node.next_sibling = old_first;

        let parent_node = self.arena.get_mut(parent).ok_or(XmlError::InvalidNodeId)?;
        if parent_node.last_child.is_none() {
            parent_node.last_child = Some(child);
        }
        parent_node.first_child = Some(child);

        Ok(child)
    }

    /// Inserts `child` immediately after `after`.
    pub fn insert_after_child(&mut self, after: NodeId, child: NodeId) -> Result<NodeId> {
        if !self.arena.contains(after) || !self.arena.contains(child) {
            return Err(XmlError::InvalidNodeId);
        }
        let parent = self.parent(after).ok_or(XmlError::InvalidNodeId)?;
        if self.is_ancestor(child, parent) {
            return Err(XmlError::InvalidNodeId);
        }

        self.unlink(child)?;

        let after_data = self.arena.get(after).ok_or(XmlError::InvalidNodeId)?;
        let old_next = after_data.next_sibling;

        if let Some(next) = old_next {
            let next_node = self.arena.get_mut(next).ok_or(XmlError::InvalidNodeId)?;
            next_node.prev_sibling = Some(child);
        }

        let child_node = self.arena.get_mut(child).ok_or(XmlError::InvalidNodeId)?;
        child_node.parent = Some(parent);
        child_node.prev_sibling = Some(after);
        child_node.next_sibling = old_next;

        let after_node = self.arena.get_mut(after).ok_or(XmlError::InvalidNodeId)?;
        after_node.next_sibling = Some(child);

        let parent_node = self.arena.get_mut(parent).ok_or(XmlError::InvalidNodeId)?;
        if parent_node.last_child == Some(after) {
            parent_node.last_child = Some(child);
        }

        Ok(child)
    }

    /// Helper for recursive deallocation of a node and all of its descendants.
    fn delete_recursive(&mut self, node: NodeId) {
        let mut next_child = self.first_child(node);
        while let Some(child) = next_child {
            let sibling = self.next_sibling(child);
            self.delete_recursive(child);
            next_child = sibling;
        }
        self.arena.dealloc(node);
    }

    /// Removes `child` from `parent` and deallocates it recursively.
    pub fn delete_child(&mut self, parent: NodeId, child: NodeId) -> Result<()> {
        if !self.arena.contains(parent) || !self.arena.contains(child) {
            return Err(XmlError::InvalidNodeId);
        }
        if self.parent(child) != Some(parent) {
            return Err(XmlError::InvalidNodeId);
        }

        self.unlink(child)?;
        self.delete_recursive(child);
        Ok(())
    }

    /// Removes and deallocates all children of `parent`.
    pub fn delete_children(&mut self, parent: NodeId) -> Result<()> {
        if !self.arena.contains(parent) {
            return Err(XmlError::InvalidNodeId);
        }

        let mut next_child = self.first_child(parent);
        while let Some(child) = next_child {
            let sibling = self.next_sibling(child);
            self.unlink(child)?;
            self.delete_recursive(child);
            next_child = sibling;
        }

        let parent_node = self.arena.get_mut(parent).ok_or(XmlError::InvalidNodeId)?;
        parent_node.first_child = None;
        parent_node.last_child = None;
        Ok(())
    }

    /// Unlinks `node` from its parent and recursively deallocates it.
    pub fn delete_node(&mut self, node: NodeId) -> Result<()> {
        if !self.arena.contains(node) {
            return Err(XmlError::InvalidNodeId);
        }
        if node == self.root {
            return Err(XmlError::InvalidNodeId);
        }

        self.unlink(node)?;
        self.delete_recursive(node);
        Ok(())
    }

    // --- Clone Operations ---

    /// Creates a shallow clone of the node (clones type & data only; no children, detached).
    pub fn shallow_clone(&mut self, node: NodeId) -> Result<NodeId> {
        let data = self.arena.get(node).ok_or(XmlError::InvalidNodeId)?.clone();
        let cloned_kind = match &data.kind {
            NodeKind::Document => NodeKind::Document,
            NodeKind::Element(el) => NodeKind::Element(el.clone()),
            NodeKind::Text(txt) => NodeKind::Text(txt.clone()),
            NodeKind::Comment(c) => NodeKind::Comment(c.clone()),
            NodeKind::Declaration(d) => NodeKind::Declaration(d.clone()),
            NodeKind::Unknown(u) => NodeKind::Unknown(u.clone()),
        };
        let cloned_data = NodeData::new(cloned_kind, data.line_num);
        let cloned_id = self.arena.alloc(cloned_data);
        Ok(cloned_id)
    }

    /// Recursively clones a node and all of its descendants.
    pub fn deep_clone(&mut self, node: NodeId) -> Result<NodeId> {
        let cloned_id = self.shallow_clone(node)?;
        let mut next_child = self.first_child(node);
        while let Some(child) = next_child {
            let cloned_child = self.deep_clone(child)?;
            self.insert_end_child(cloned_id, cloned_child)?;
            next_child = self.next_sibling(child);
        }
        Ok(cloned_id)
    }

    // --- Attribute Manipulation ---

    /// Returns the string value of the attribute on element `el` if it exists.
    #[must_use]
    pub fn attribute(&self, el: NodeId, name: &str) -> Option<&str> {
        let data = self.arena.get(el)?;
        match &data.kind {
            NodeKind::Element(el_data) | NodeKind::Declaration(el_data) => el_data
                .attributes
                .iter()
                .find(|attr| attr.name == name)
                .map(|attr| attr.value.as_str()),
            _ => None,
        }
    }

    /// Sets the value of an attribute on element `el` (inserts or updates).
    pub fn set_attribute(&mut self, el: NodeId, name: &str, value: &str) -> Result<()> {
        let data = self.arena.get_mut(el).ok_or(XmlError::InvalidNodeId)?;
        match &mut data.kind {
            NodeKind::Element(el_data) | NodeKind::Declaration(el_data) => {
                if let Some(attr) = el_data.attributes.iter_mut().find(|attr| attr.name == name) {
                    attr.value = value.to_string();
                } else {
                    el_data.attributes.push(Attribute {
                        name: name.to_string(),
                        value: value.to_string(),
                    });
                }
                Ok(())
            }
            _ => Err(XmlError::InvalidNodeId),
        }
    }

    /// Removes an attribute from element `el`.
    pub fn delete_attribute(&mut self, el: NodeId, name: &str) -> Result<()> {
        let data = self.arena.get_mut(el).ok_or(XmlError::InvalidNodeId)?;
        match &mut data.kind {
            NodeKind::Element(el_data) | NodeKind::Declaration(el_data) => {
                if let Some(pos) = el_data.attributes.iter().position(|attr| attr.name == name) {
                    el_data.attributes.remove(pos);
                    Ok(())
                } else {
                    Err(XmlError::NoAttribute)
                }
            }
            _ => Err(XmlError::InvalidNodeId),
        }
    }

    /// Returns a reference to the first Attribute on element `el`.
    #[must_use]
    pub fn first_attribute(&self, el: NodeId) -> Option<&Attribute> {
        let data = self.arena.get(el)?;
        match &data.kind {
            NodeKind::Element(el_data) | NodeKind::Declaration(el_data) => {
                el_data.attributes.first()
            }
            _ => None,
        }
    }

    /// Returns the number of attributes on element `el`.
    #[must_use]
    pub fn attribute_count(&self, el: NodeId) -> usize {
        let Some(data) = self.arena.get(el) else {
            return 0;
        };
        match &data.kind {
            NodeKind::Element(el_data) | NodeKind::Declaration(el_data) => el_data.attributes.len(),
            _ => 0,
        }
    }

    /// Finds a reference to the Attribute with the specified name.
    #[must_use]
    pub fn find_attribute(&self, el: NodeId, name: &str) -> Option<&Attribute> {
        let data = self.arena.get(el)?;
        match &data.kind {
            NodeKind::Element(el_data) | NodeKind::Declaration(el_data) => {
                el_data.attributes.iter().find(|attr| attr.name == name)
            }
            _ => None,
        }
    }

    /// Returns an iterator over all attributes of element `el`.
    pub fn iterate_attributes(&self, el: NodeId) -> impl Iterator<Item = &Attribute> {
        let attrs = match self.arena.get(el) {
            Some(data) => match &data.kind {
                NodeKind::Element(el_data) | NodeKind::Declaration(el_data) => {
                    &el_data.attributes[..]
                }
                _ => &[],
            },
            None => &[],
        };
        attrs.iter()
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}
