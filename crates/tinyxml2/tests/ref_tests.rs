//! Integration tests for `NodeRef` and `ElementRef` typed reference wrappers.

use tinyxml2::{Document, NodeKind};

// --- NodeRef tests ---

#[test]
fn node_ref_kind() {
    let doc = Document::parse("<root/>").unwrap();
    let node = doc.node_ref(doc.root()).unwrap();
    assert!(matches!(node.kind(), NodeKind::Document));
}

#[test]
fn node_ref_value_element() {
    let doc = Document::parse("<root/>").unwrap();
    let root_el = doc.node_ref(doc.root_element().unwrap()).unwrap();
    assert_eq!(root_el.value(), "root");
}

#[test]
fn node_ref_value_text() {
    let doc = Document::parse("<root>hello world</root>").unwrap();
    let root_el = doc.root_element().unwrap();
    let text_node = doc.children(root_el).next().unwrap();
    assert_eq!(text_node.value(), "hello world");
}

#[test]
fn node_ref_value_comment() {
    let doc = Document::parse("<root><!-- my comment --></root>").unwrap();
    let root_el = doc.root_element().unwrap();
    let comment_node = doc.children(root_el).next().unwrap();
    assert_eq!(comment_node.value(), " my comment ");
}

#[test]
fn node_ref_value_document() {
    let doc = Document::parse("<root/>").unwrap();
    let node = doc.node_ref(doc.root()).unwrap();
    assert_eq!(node.value(), "");
}

#[test]
fn node_ref_parent() {
    let doc = Document::parse("<root><child/></root>").unwrap();
    let root_el = doc.root_element().unwrap();
    let child = doc.children(root_el).next().unwrap();
    let parent = child.parent().unwrap();
    assert_eq!(parent.id(), root_el);
}

#[test]
fn node_ref_parent_of_root_is_none() {
    let doc = Document::parse("<root/>").unwrap();
    let doc_node = doc.node_ref(doc.root()).unwrap();
    assert!(doc_node.parent().is_none());
}

#[test]
fn node_ref_children_iteration() {
    let doc = Document::parse("<root><a/><b/><c/></root>").unwrap();
    let root_el = doc.node_ref(doc.root_element().unwrap()).unwrap();
    let names: Vec<&str> = root_el.children().map(|n| n.value()).collect();
    assert_eq!(names, vec!["a", "b", "c"]);
}

#[test]
fn node_ref_line_number() {
    let doc = Document::parse("<root/>").unwrap();
    let root_el = doc.node_ref(doc.root_element().unwrap()).unwrap();
    // Line number should be non-zero for parsed nodes
    assert!(root_el.line() > 0);
}

#[test]
fn node_ref_as_element_success() {
    let doc = Document::parse("<root/>").unwrap();
    let node = doc.node_ref(doc.root_element().unwrap()).unwrap();
    let el = node.as_element();
    assert!(el.is_some());
    assert_eq!(el.unwrap().name(), "root");
}

#[test]
fn node_ref_as_element_failure_on_text() {
    let doc = Document::parse("<root>text</root>").unwrap();
    let root_el = doc.root_element().unwrap();
    let text = doc.children(root_el).next().unwrap();
    assert!(text.as_element().is_none());
}

#[test]
fn node_ref_as_element_failure_on_document() {
    let doc = Document::parse("<root/>").unwrap();
    let doc_node = doc.node_ref(doc.root()).unwrap();
    assert!(doc_node.as_element().is_none());
}

#[test]
fn node_ref_siblings() {
    let doc = Document::parse("<root><a/><b/><c/></root>").unwrap();
    let root_el = doc.root_element().unwrap();
    let a = doc.children(root_el).next().unwrap();
    let sibling_names: Vec<&str> = a.siblings().map(|n| n.value()).collect();
    assert_eq!(sibling_names, vec!["b", "c"]);
}

#[test]
fn node_ref_descendants() {
    let doc = Document::parse("<root><a><a1/></a><b/></root>").unwrap();
    let root_el = doc.node_ref(doc.root_element().unwrap()).unwrap();
    let names: Vec<&str> = root_el.descendants().map(|n| n.value()).collect();
    assert_eq!(names, vec!["a", "a1", "b"]);
}

#[test]
fn node_ref_equality() {
    let doc = Document::parse("<root/>").unwrap();
    let n1 = doc.node_ref(doc.root()).unwrap();
    let n2 = doc.node_ref(doc.root()).unwrap();
    assert_eq!(n1, n2);
}

#[test]
fn node_ref_is_copy() {
    let doc = Document::parse("<root/>").unwrap();
    let n1 = doc.node_ref(doc.root()).unwrap();
    let n2 = n1; // copy
    assert_eq!(n1, n2);
}

#[test]
fn node_ref_document_accessor() {
    let doc = Document::parse("<root/>").unwrap();
    let node = doc.node_ref(doc.root()).unwrap();
    assert_eq!(node.document().root(), doc.root());
}

// --- ElementRef tests ---

#[test]
fn element_ref_name() {
    let doc = Document::parse("<myelem/>").unwrap();
    let el = doc.element_ref(doc.root_element().unwrap()).unwrap();
    assert_eq!(el.name(), "myelem");
}

#[test]
fn element_ref_attribute() {
    let doc = Document::parse(r#"<root key="value"/>"#).unwrap();
    let el = doc.element_ref(doc.root_element().unwrap()).unwrap();
    assert_eq!(el.attribute("key"), Some("value"));
    assert_eq!(el.attribute("missing"), None);
}

#[test]
fn element_ref_attributes_iteration() {
    let doc = Document::parse(r#"<root a="1" b="2" c="3"/>"#).unwrap();
    let el = doc.element_ref(doc.root_element().unwrap()).unwrap();
    let attrs: Vec<(&str, &str)> = el.attributes().collect();
    assert_eq!(attrs, vec![("a", "1"), ("b", "2"), ("c", "3")]);
}

#[test]
fn element_ref_text() {
    let doc = Document::parse("<root>hello</root>").unwrap();
    let el = doc.element_ref(doc.root_element().unwrap()).unwrap();
    assert_eq!(el.text(), Some("hello"));
}

#[test]
fn element_ref_text_none_when_empty() {
    let doc = Document::parse("<root/>").unwrap();
    let el = doc.element_ref(doc.root_element().unwrap()).unwrap();
    assert_eq!(el.text(), None);
}

#[test]
fn element_ref_children() {
    let doc = Document::parse("<root><a/><b/></root>").unwrap();
    let el = doc.element_ref(doc.root_element().unwrap()).unwrap();
    let count = el.children().count();
    assert_eq!(count, 2);
}

#[test]
fn element_ref_child_elements() {
    let doc = Document::parse("<root><a/>text<b/></root>").unwrap();
    let el = doc.element_ref(doc.root_element().unwrap()).unwrap();
    let names: Vec<&str> = el.child_elements().map(|e| e.name()).collect();
    assert_eq!(names, vec!["a", "b"]);
}

#[test]
fn element_ref_child_elements_by_name() {
    let doc = Document::parse("<root><x/><y/><x/></root>").unwrap();
    let el = doc.element_ref(doc.root_element().unwrap()).unwrap();
    let count = el.child_elements_by_name("x").count();
    assert_eq!(count, 2);
}

#[test]
fn element_ref_int_attribute() {
    let doc = Document::parse(r#"<root count="42"/>"#).unwrap();
    let el = doc.element_ref(doc.root_element().unwrap()).unwrap();
    assert_eq!(el.int_attribute("count", 0), 42);
    assert_eq!(el.int_attribute("missing", -1), -1);
}

#[test]
fn element_ref_bool_attribute() {
    let doc = Document::parse(r#"<root enabled="true"/>"#).unwrap();
    let el = doc.element_ref(doc.root_element().unwrap()).unwrap();
    assert!(el.bool_attribute("enabled", false));
    assert!(!el.bool_attribute("missing", false));
}

#[test]
fn element_ref_double_attribute() {
    let doc = Document::parse(r#"<root ratio="1.61803"/>"#).unwrap();
    let el = doc.element_ref(doc.root_element().unwrap()).unwrap();
    let val = el.double_attribute("ratio", 0.0);
    assert!((val - 1.61803).abs() < 1e-10);
    assert!((el.double_attribute("missing", 99.5) - 99.5).abs() < 1e-10);
}

#[test]
fn element_ref_as_node_roundtrip() {
    let doc = Document::parse("<root/>").unwrap();
    let el = doc.element_ref(doc.root_element().unwrap()).unwrap();
    let node = el.as_node();
    let el2 = node.as_element().unwrap();
    assert_eq!(el.id(), el2.id());
}

#[test]
fn element_ref_is_copy() {
    let doc = Document::parse("<root/>").unwrap();
    let el1 = doc.element_ref(doc.root_element().unwrap()).unwrap();
    let el2 = el1; // copy
    assert_eq!(el1, el2);
}

#[test]
fn element_ref_from_non_element_returns_none() {
    let doc = Document::parse("<root/>").unwrap();
    // Document root node is not an Element
    assert!(doc.element_ref(doc.root()).is_none());
}

#[test]
fn element_ref_document_accessor() {
    let doc = Document::parse("<root/>").unwrap();
    let el = doc.element_ref(doc.root_element().unwrap()).unwrap();
    assert_eq!(el.document().root(), doc.root());
}

#[test]
fn element_ref_equality() {
    let doc = Document::parse("<root/>").unwrap();
    let el1 = doc.element_ref(doc.root_element().unwrap()).unwrap();
    let el2 = doc.element_ref(doc.root_element().unwrap()).unwrap();
    assert_eq!(el1, el2);
}

#[test]
fn node_ref_invalid_id_returns_none() {
    let mut doc = Document::parse("<root><child/></root>").unwrap();
    let root_el = doc.root_element().unwrap();
    let child_id = doc.first_child(root_el).unwrap();
    // Delete the child, making the id stale
    doc.delete_child(root_el, child_id).unwrap();
    // node_ref should return None for stale IDs
    assert!(doc.node_ref(child_id).is_none());
}
