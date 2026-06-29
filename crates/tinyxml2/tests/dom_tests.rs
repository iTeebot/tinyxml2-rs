#![allow(clippy::approx_constant, clippy::unreadable_literal)]

use tinyxml2::{Attribute, Document, Result, XmlError};

#[test]
fn test_factory_methods() {
    let mut doc = Document::new();
    let root = doc.root();

    // Verify root is a Document node
    // We can't access arena directly from tests, but we can verify it indirectly or test other nodes
    assert_eq!(doc.parent(root), None);

    let el = doc.new_element("test");
    let txt = doc.new_text("hello");
    let cdata = doc.new_cdata("world");
    let comment = doc.new_comment("this is a comment");
    let decl = doc.new_declaration("xml version=\"1.0\"");
    let unknown = doc.new_unknown("DOCTYPE html");

    // Initially all these are detached
    assert_eq!(doc.parent(el), None);
    assert_eq!(doc.parent(txt), None);
    assert_eq!(doc.parent(cdata), None);
    assert_eq!(doc.parent(comment), None);
    assert_eq!(doc.parent(decl), None);
    assert_eq!(doc.parent(unknown), None);
}

#[test]
fn test_tree_mutations_end_child() -> Result<()> {
    let mut doc = Document::new();
    let root = doc.root();

    let el1 = doc.new_element("el1");
    let el2 = doc.new_element("el2");

    doc.insert_end_child(root, el1)?;
    doc.insert_end_child(root, el2)?;

    assert_eq!(doc.parent(el1), Some(root));
    assert_eq!(doc.parent(el2), Some(root));

    assert_eq!(doc.first_child(root), Some(el1));
    assert_eq!(doc.last_child(root), Some(el2));

    assert_eq!(doc.next_sibling(el1), Some(el2));
    assert_eq!(doc.prev_sibling(el2), Some(el1));
    assert_eq!(doc.next_sibling(el2), None);
    assert_eq!(doc.prev_sibling(el1), None);

    Ok(())
}

#[test]
fn test_tree_mutations_first_child() -> Result<()> {
    let mut doc = Document::new();
    let root = doc.root();

    let el1 = doc.new_element("el1");
    let el2 = doc.new_element("el2");

    doc.insert_first_child(root, el1)?;
    doc.insert_first_child(root, el2)?; // should become the first child

    assert_eq!(doc.first_child(root), Some(el2));
    assert_eq!(doc.last_child(root), Some(el1));

    assert_eq!(doc.next_sibling(el2), Some(el1));
    assert_eq!(doc.prev_sibling(el1), Some(el2));

    Ok(())
}

#[test]
fn test_tree_mutations_after_child() -> Result<()> {
    let mut doc = Document::new();
    let root = doc.root();

    let el1 = doc.new_element("el1");
    let el2 = doc.new_element("el2");
    let el3 = doc.new_element("el3");

    doc.insert_end_child(root, el1)?;
    doc.insert_end_child(root, el3)?;
    doc.insert_after_child(el1, el2)?;

    assert_eq!(doc.first_child(root), Some(el1));
    assert_eq!(doc.next_sibling(el1), Some(el2));
    assert_eq!(doc.next_sibling(el2), Some(el3));
    assert_eq!(doc.last_child(root), Some(el3));

    Ok(())
}

#[test]
fn test_tree_mutations_delete_child() -> Result<()> {
    let mut doc = Document::new();
    let root = doc.root();

    let el1 = doc.new_element("el1");
    let el2 = doc.new_element("el2");
    let el3 = doc.new_element("el3");

    doc.insert_end_child(root, el1)?;
    doc.insert_end_child(root, el2)?;
    doc.insert_end_child(root, el3)?;

    doc.delete_child(root, el2)?;

    assert_eq!(doc.first_child(root), Some(el1));
    assert_eq!(doc.next_sibling(el1), Some(el3));
    assert_eq!(doc.prev_sibling(el3), Some(el1));
    assert_eq!(doc.parent(el2), None);

    Ok(())
}

#[test]
fn test_tree_mutations_delete_children() -> Result<()> {
    let mut doc = Document::new();
    let root = doc.root();

    let el1 = doc.new_element("el1");
    let el2 = doc.new_element("el2");

    doc.insert_end_child(root, el1)?;
    doc.insert_end_child(root, el2)?;

    doc.delete_children(root)?;

    assert_eq!(doc.first_child(root), None);
    assert_eq!(doc.last_child(root), None);

    Ok(())
}

#[test]
fn test_tree_mutations_delete_node() -> Result<()> {
    let mut doc = Document::new();
    let root = doc.root();

    let el1 = doc.new_element("el1");
    let el2 = doc.new_element("el2");

    doc.insert_end_child(root, el1)?;
    doc.insert_end_child(root, el2)?;

    doc.delete_node(el1)?;

    assert_eq!(doc.first_child(root), Some(el2));
    assert_eq!(doc.prev_sibling(el2), None);

    // Verify root node deletion is rejected
    assert!(doc.delete_node(root).is_err());

    Ok(())
}

#[test]
fn test_hierarchy_violation_cycles() -> Result<()> {
    let mut doc = Document::new();
    let root = doc.root();

    let el1 = doc.new_element("el1");
    let el2 = doc.new_element("el2");

    doc.insert_end_child(root, el1)?;
    doc.insert_end_child(el1, el2)?;

    // Attempting to make el1 a child of el2 (cycle)
    let res = doc.insert_end_child(el2, el1);
    assert!(res.is_err());

    // Attempting to insert root into a child
    let res = doc.insert_end_child(el1, root);
    assert!(res.is_err());

    Ok(())
}

#[test]
fn test_navigation_element_filters() -> Result<()> {
    let mut doc = Document::new();
    let root = doc.root();

    let el1 = doc.new_element("item");
    let txt = doc.new_text("some text");
    let el2 = doc.new_element("special");
    let el3 = doc.new_element("item");

    doc.insert_end_child(root, el1)?;
    doc.insert_end_child(root, txt)?;
    doc.insert_end_child(root, el2)?;
    doc.insert_end_child(root, el3)?;

    // first_child_element
    assert_eq!(doc.first_child_element(root, None), Some(el1));
    assert_eq!(doc.first_child_element(root, Some("special")), Some(el2));
    assert_eq!(doc.first_child_element(root, Some("missing")), None);

    // last_child_element
    assert_eq!(doc.last_child_element(root, None), Some(el3));
    assert_eq!(doc.last_child_element(root, Some("special")), Some(el2));

    // next_sibling_element
    assert_eq!(doc.next_sibling_element(el1, None), Some(el2)); // skips text
    assert_eq!(doc.next_sibling_element(el1, Some("item")), Some(el3));

    // prev_sibling_element
    assert_eq!(doc.prev_sibling_element(el3, None), Some(el2));
    assert_eq!(doc.prev_sibling_element(el3, Some("item")), Some(el1));

    // root_element
    assert_eq!(doc.root_element(), Some(el1));

    Ok(())
}

#[test]
fn test_attributes() -> Result<()> {
    let mut doc = Document::new();
    let el = doc.new_element("node");

    assert_eq!(doc.attribute(el, "a"), None);
    assert_eq!(doc.attribute_count(el), 0);

    doc.set_attribute(el, "a", "1")?;
    doc.set_attribute(el, "b", "2")?;

    assert_eq!(doc.attribute(el, "a"), Some("1"));
    assert_eq!(doc.attribute(el, "b"), Some("2"));
    assert_eq!(doc.attribute_count(el), 2);

    // Update attribute
    doc.set_attribute(el, "a", "10")?;
    assert_eq!(doc.attribute(el, "a"), Some("10"));
    assert_eq!(doc.attribute_count(el), 2);

    // Order preservation check (first should be "a")
    let first = doc.first_attribute(el).unwrap();
    assert_eq!(first.name, "a");
    assert_eq!(first.value, "10");

    // Find attribute
    assert_eq!(doc.find_attribute(el, "b").unwrap().value, "2");

    // Iterate attributes
    let attrs: Vec<Attribute> = doc.iterate_attributes(el).cloned().collect();
    assert_eq!(attrs.len(), 2);
    assert_eq!(attrs[0].name, "a");
    assert_eq!(attrs[1].name, "b");

    // Delete attribute
    doc.delete_attribute(el, "a")?;
    assert_eq!(doc.attribute(el, "a"), None);
    assert_eq!(doc.attribute_count(el), 1);

    Ok(())
}

#[test]
fn test_typed_attribute_access() -> Result<()> {
    let mut doc = Document::new();
    let el = doc.new_element("node");

    doc.set_attribute(el, "int", "42")?;
    doc.set_attribute(el, "uint", "100")?;
    doc.set_attribute(el, "int64", "-9223372036854775808")?;
    doc.set_attribute(el, "bool_t", "true")?;
    doc.set_attribute(el, "bool_1", "1")?;
    doc.set_attribute(el, "bool_f", "false")?;
    doc.set_attribute(el, "double", "3.14159")?;
    doc.set_attribute(el, "float", "2.718")?;
    doc.set_attribute(el, "bad", "abc")?;

    // Query values
    assert_eq!(doc.query_int_attribute(el, "int")?, 42);
    assert_eq!(doc.query_unsigned_attribute(el, "uint")?, 100);
    assert_eq!(
        doc.query_int64_attribute(el, "int64")?,
        -9223372036854775808
    );
    assert!(doc.query_bool_attribute(el, "bool_t")?);
    assert!(doc.query_bool_attribute(el, "bool_1")?);
    assert!(!doc.query_bool_attribute(el, "bool_f")?);
    assert!((doc.query_double_attribute(el, "double")? - 3.14159).abs() < 1e-5);
    assert!((doc.query_float_attribute(el, "float")? - 2.718).abs() < 1e-5);

    // Wrong type results
    assert!(doc.query_int_attribute(el, "bad").is_err());
    assert!(doc.query_bool_attribute(el, "bad").is_err());

    // Defaults
    assert_eq!(doc.int_attribute(el, "int", 0), 42);
    assert_eq!(doc.int_attribute(el, "bad", 99), 99);
    assert_eq!(doc.int_attribute(el, "missing", -1), -1);

    assert!(doc.bool_attribute(el, "bool_t", false));
    assert!(!doc.bool_attribute(el, "bad", false));
    assert!(doc.bool_attribute(el, "missing", true));

    Ok(())
}

#[test]
fn test_text_helpers() -> Result<()> {
    let mut doc = Document::new();
    let el = doc.new_element("node");

    assert_eq!(doc.get_text(el), None);

    doc.set_text(el, "first text")?;
    assert_eq!(doc.get_text(el), Some("first text"));

    // Overwrite text
    doc.set_text(el, "updated text")?;
    assert_eq!(doc.get_text(el), Some("updated text"));

    // Verify it created exactly one text node child
    let child1 = doc.first_child(el).unwrap();
    assert_eq!(doc.next_sibling(child1), None);

    Ok(())
}

#[test]
fn test_typed_text_access() -> Result<()> {
    let mut doc = Document::new();
    let el = doc.new_element("node");

    doc.set_text(el, "123")?;
    assert_eq!(doc.query_int_text(el)?, 123);
    assert_eq!(doc.int_text(el, 0), 123);

    doc.set_text(el, "true")?;
    assert!(doc.query_bool_text(el)?);
    assert!(doc.bool_text(el, false));

    doc.set_text(el, "3.14")?;
    assert!((doc.query_double_text(el)? - 3.14).abs() < 1e-5);

    doc.set_text(el, "not a number")?;
    assert_eq!(doc.query_int_text(el), Err(XmlError::CanNotConvertText));
    assert_eq!(doc.int_text(el, 999), 999);

    Ok(())
}

#[test]
fn test_cloning() -> Result<()> {
    let mut doc = Document::new();

    let el1 = doc.new_element("parent");
    doc.set_attribute(el1, "attr", "val")?;
    let child = doc.new_element("child");
    doc.insert_end_child(el1, child)?;

    // Shallow clone
    let shallow = doc.shallow_clone(el1)?;
    assert_eq!(doc.attribute(shallow, "attr"), Some("val"));
    assert_eq!(doc.first_child(shallow), None); // no children in shallow clone

    // Deep clone
    let deep = doc.deep_clone(el1)?;
    assert_eq!(doc.attribute(deep, "attr"), Some("val"));
    let deep_child = doc.first_child(deep).unwrap();
    assert_ne!(deep_child, child); // should be a new node ID
    assert_eq!(doc.parent(deep_child), Some(deep));

    // Mutations on original shouldn't affect deep clone
    doc.set_attribute(el1, "attr", "mutated")?;
    assert_eq!(doc.attribute(deep, "attr"), Some("val"));

    Ok(())
}

#[test]
fn test_stale_node_id_checks() -> Result<()> {
    let mut doc = Document::new();
    let root = doc.root();

    let el = doc.new_element("el");
    doc.insert_end_child(root, el)?;

    doc.delete_node(el)?;

    // Now el NodeId is stale, navigating/accessing it should return None or Error
    assert_eq!(doc.parent(el), None);
    assert_eq!(doc.first_child(el), None);
    assert_eq!(doc.attribute(el, "any"), None);
    assert!(doc.set_attribute(el, "any", "val").is_err());
    assert!(doc.delete_node(el).is_err());

    Ok(())
}
