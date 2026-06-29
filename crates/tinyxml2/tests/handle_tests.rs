//! Integration tests for `Handle` and `HandleMut` null-safe navigation.

use tinyxml2::Document;
use tinyxml2::Handle;

// --- Handle (immutable) tests ---

#[test]
fn handle_navigation_chain() {
    let doc = Document::parse(
        r#"<root><settings><resolution width="1920">1080p</resolution></settings></root>"#,
    )
    .unwrap();

    let text = doc
        .handle(doc.root())
        .first_child_element(Some("root"))
        .first_child_element(Some("settings"))
        .first_child_element(Some("resolution"))
        .text();
    assert_eq!(text, Some("1080p"));
}

#[test]
fn handle_attribute_extraction() {
    let doc = Document::parse(r#"<item id="42" enabled="true"/>"#).unwrap();

    let h = doc.handle(doc.root()).first_child_element(None);
    assert_eq!(h.attribute("id"), Some("42"));
    assert_eq!(h.attribute("enabled"), Some("true"));
    assert_eq!(h.attribute("missing"), None);
}

#[test]
fn handle_null_propagation() {
    let doc = Document::parse("<root/>").unwrap();

    let h = doc
        .handle(doc.root())
        .first_child_element(Some("nonexistent"))
        .first_child()
        .next_sibling();
    assert!(h.is_null());
    assert_eq!(h.to_node(), None);
    assert_eq!(h.to_element(), None);
    assert_eq!(h.text(), None);
    assert_eq!(h.attribute("x"), None);
}

#[test]
fn handle_null_constructor() {
    let doc = Document::parse("<root/>").unwrap();
    let h = Handle::null(&doc);
    assert!(h.is_null());
    assert_eq!(h.first_child().to_node(), None);
}

#[test]
fn handle_to_element_returns_none_for_non_element() {
    let doc = Document::parse("<root>hello</root>").unwrap();
    // first_child of root is the <root> element, first_child of root_element is text "hello"
    let h = doc.handle(doc.root()).first_child().first_child();
    // This should be the text node, not an element
    assert!(h.to_node().is_some());
    assert!(h.to_element().is_none());
}

#[test]
fn handle_sibling_navigation() {
    let doc = Document::parse("<root><a/><b/><c/></root>").unwrap();
    let root_el = doc.handle(doc.root()).first_child_element(None);

    let a = root_el.first_child_element(None);
    assert_eq!(
        a.to_element(),
        doc.first_child_element(doc.root_element().unwrap(), Some("a"))
    );

    let b = a.next_sibling_element(None);
    assert!(b.to_element().is_some());

    let c = b.next_sibling_element(None);
    assert!(c.to_element().is_some());

    let end = c.next_sibling_element(None);
    assert!(end.is_null());
}

#[test]
fn handle_last_child_and_prev_sibling() {
    let doc = Document::parse("<root><a/><b/><c/></root>").unwrap();
    let root_el = doc.handle(doc.root()).first_child_element(None);

    let last = root_el.last_child();
    assert!(last.to_element().is_some());

    let prev = last.prev_sibling();
    assert!(prev.to_element().is_some());
}

#[test]
fn handle_parent_navigation() {
    let doc = Document::parse("<root><child><nested/></child></root>").unwrap();
    let nested = doc
        .handle(doc.root())
        .first_child_element(None)
        .first_child_element(None)
        .first_child_element(None);
    assert!(nested.to_element().is_some());

    let child = nested.parent();
    assert!(child.to_element().is_some());

    let root = child.parent();
    assert!(root.to_element().is_some());
}

#[test]
fn handle_is_copy() {
    let doc = Document::parse("<root><a/></root>").unwrap();
    let h1 = doc.handle(doc.root()).first_child_element(None);
    let h2 = h1; // copy
    assert_eq!(h1.to_node(), h2.to_node());
}

// --- HandleMut tests ---

#[test]
fn handle_mut_navigation_chain() {
    let mut doc = Document::parse("<root><a><b/></a></root>").unwrap();
    let root = doc.root();
    let node = doc.handle_mut(root).first_child().first_child().to_node();
    assert!(node.is_some());
}

#[test]
fn handle_mut_null_propagation() {
    let mut doc = Document::parse("<root/>").unwrap();
    let root = doc.root();
    let h = doc
        .handle_mut(root)
        .first_child_element(Some("nope"))
        .first_child();
    assert!(h.is_null());
    assert_eq!(h.to_node(), None);
}

#[test]
fn handle_mut_into_doc() {
    let mut doc = Document::parse("<root/>").unwrap();
    let root = doc.root();
    let h = doc.handle_mut(root).first_child_element(None);
    let doc_ref = h.into_doc();
    // Verify we can still mutate the document after reclaiming the reference
    let new_el = doc_ref.new_element("added");
    doc_ref.insert_end_child(doc_ref.root(), new_el).unwrap();
    assert!(doc_ref.root_element().is_some());
}

#[test]
fn handle_mut_text_and_attribute() {
    let mut doc = Document::parse(r#"<root><item key="val">hello</item></root>"#).unwrap();
    let root = doc.root();
    let h = doc
        .handle_mut(root)
        .first_child_element(None)
        .first_child_element(None);
    assert_eq!(h.text(), Some("hello"));
    assert_eq!(h.attribute("key"), Some("val"));
}
