//! Integration tests for iterator adapters.

use tinyxml2::Document;

#[test]
fn children_forward_iteration() {
    let doc = Document::parse("<root><a/><b/><c/></root>").unwrap();
    let root_el = doc.root_element().unwrap();
    let names: Vec<&str> = doc.children(root_el).map(|n| n.value()).collect();
    assert_eq!(names, vec!["a", "b", "c"]);
}

#[test]
fn children_reverse_iteration() {
    let doc = Document::parse("<root><a/><b/><c/></root>").unwrap();
    let root_el = doc.root_element().unwrap();
    let names: Vec<&str> = doc.children(root_el).rev().map(|n| n.value()).collect();
    assert_eq!(names, vec!["c", "b", "a"]);
}

#[test]
fn children_empty_parent() {
    let doc = Document::parse("<root/>").unwrap();
    let root_el = doc.root_element().unwrap();
    let count = doc.children(root_el).count();
    assert_eq!(count, 0);
}

#[test]
fn children_single_child() {
    let doc = Document::parse("<root><only/></root>").unwrap();
    let root_el = doc.root_element().unwrap();
    let names: Vec<&str> = doc.children(root_el).map(|n| n.value()).collect();
    assert_eq!(names, vec!["only"]);
}

#[test]
fn children_fused_after_exhaustion() {
    let doc = Document::parse("<root><a/></root>").unwrap();
    let root_el = doc.root_element().unwrap();
    let mut iter = doc.children(root_el);
    assert!(iter.next().is_some());
    assert!(iter.next().is_none());
    // FusedIterator guarantee
    assert!(iter.next().is_none());
}

#[test]
fn children_mixed_forward_backward() {
    let doc = Document::parse("<root><a/><b/><c/><d/></root>").unwrap();
    let root_el = doc.root_element().unwrap();
    let mut iter = doc.children(root_el);
    assert_eq!(iter.next().unwrap().value(), "a");
    assert_eq!(iter.next_back().unwrap().value(), "d");
    assert_eq!(iter.next().unwrap().value(), "b");
    assert_eq!(iter.next_back().unwrap().value(), "c");
    assert!(iter.next().is_none());
}

#[test]
fn child_elements_no_filter() {
    let doc = Document::parse("<root><a/>text<b/><!-- comment --><c/></root>").unwrap();
    let root_el = doc.root_element().unwrap();
    let names: Vec<&str> = doc
        .child_elements(root_el, None)
        .map(|e| e.name())
        .collect();
    assert_eq!(names, vec!["a", "b", "c"]);
}

#[test]
fn child_elements_name_filter() {
    let doc = Document::parse("<root><item/><other/><item/><item/></root>").unwrap();
    let root_el = doc.root_element().unwrap();
    let count = doc.child_elements(root_el, Some("item")).count();
    assert_eq!(count, 3);
}

#[test]
fn child_elements_name_filter_no_match() {
    let doc = Document::parse("<root><a/><b/></root>").unwrap();
    let root_el = doc.root_element().unwrap();
    let count = doc.child_elements(root_el, Some("missing")).count();
    assert_eq!(count, 0);
}

#[test]
fn siblings_iteration() {
    let doc = Document::parse("<root><a/><b/><c/><d/></root>").unwrap();
    let root_el = doc.root_element().unwrap();
    let first = doc.first_child(root_el).unwrap();
    let sibling_names: Vec<&str> = doc.siblings(first).map(|n| n.value()).collect();
    assert_eq!(sibling_names, vec!["b", "c", "d"]);
}

#[test]
fn siblings_last_child_has_none() {
    let doc = Document::parse("<root><a/><b/></root>").unwrap();
    let root_el = doc.root_element().unwrap();
    // Navigate to the last child
    let b = doc
        .first_child(root_el)
        .and_then(|a| doc.next_sibling(a))
        .unwrap();
    let count = doc.siblings(b).count();
    assert_eq!(count, 0);
}

#[test]
fn attributes_forward() {
    let doc = Document::parse(r#"<root x="1" y="2" z="3"/>"#).unwrap();
    let root_el = doc.root_element().unwrap();
    let attrs: Vec<(&str, &str)> = doc.attributes(root_el).collect();
    assert_eq!(attrs, vec![("x", "1"), ("y", "2"), ("z", "3")]);
}

#[test]
fn attributes_reverse() {
    let doc = Document::parse(r#"<root x="1" y="2" z="3"/>"#).unwrap();
    let root_el = doc.root_element().unwrap();
    let attrs: Vec<(&str, &str)> = doc.attributes(root_el).rev().collect();
    assert_eq!(attrs, vec![("z", "3"), ("y", "2"), ("x", "1")]);
}

#[test]
fn attributes_empty() {
    let doc = Document::parse("<root/>").unwrap();
    let root_el = doc.root_element().unwrap();
    let count = doc.attributes(root_el).count();
    assert_eq!(count, 0);
}

#[test]
fn attributes_exact_size() {
    let doc = Document::parse(r#"<root a="1" b="2"/>"#).unwrap();
    let root_el = doc.root_element().unwrap();
    let iter = doc.attributes(root_el);
    assert_eq!(iter.len(), 2);
}

#[test]
fn descendants_depth_first_order() {
    let doc = Document::parse("<root><a><a1/><a2/></a><b><b1><b1a/></b1></b><c/></root>").unwrap();
    let root_el = doc.root_element().unwrap();
    let names: Vec<&str> = doc.descendants(root_el).map(|n| n.value()).collect();
    assert_eq!(names, vec!["a", "a1", "a2", "b", "b1", "b1a", "c"]);
}

#[test]
fn descendants_empty() {
    let doc = Document::parse("<root/>").unwrap();
    let root_el = doc.root_element().unwrap();
    let count = doc.descendants(root_el).count();
    assert_eq!(count, 0);
}

#[test]
fn descendants_with_text_and_comments() {
    let doc = Document::parse("<root>text<!-- comment --><child/></root>").unwrap();
    let root_el = doc.root_element().unwrap();
    let count = doc.descendants(root_el).count();
    assert_eq!(count, 3); // text node, comment node, child element
}

#[test]
fn child_elements_via_element_ref() {
    let doc = Document::parse("<root><a/><b/><c/></root>").unwrap();
    let root_ref = doc.element_ref(doc.root_element().unwrap()).unwrap();
    let names: Vec<&str> = root_ref.child_elements().map(|e| e.name()).collect();
    assert_eq!(names, vec!["a", "b", "c"]);
}

#[test]
fn child_elements_by_name_via_element_ref() {
    let doc = Document::parse("<root><a/><b/><a/></root>").unwrap();
    let root_ref = doc.element_ref(doc.root_element().unwrap()).unwrap();
    let count = root_ref.child_elements_by_name("a").count();
    assert_eq!(count, 2);
}

#[test]
fn descendants_from_document_root() {
    let doc = Document::parse("<root><a/></root>").unwrap();
    // Descend from the document root (not the root element)
    let count = doc.descendants(doc.root()).count();
    // Should include root element + child a
    assert_eq!(count, 2);
}
