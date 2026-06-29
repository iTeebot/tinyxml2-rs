use tinyxml2::{Document, NodeId};

fn assert_dom_equal(doc1: &Document, node1: NodeId, doc2: &Document, node2: NodeId) {
    let k1 = doc1.node_kind(node1).expect("Node not found in doc1");
    let k2 = doc2.node_kind(node2).expect("Node not found in doc2");
    assert_eq!(k1, k2);

    let mut c1 = doc1.first_child(node1);
    let mut c2 = doc2.first_child(node2);
    while let (Some(child1), Some(child2)) = (c1, c2) {
        assert_dom_equal(doc1, child1, doc2, child2);
        c1 = doc1.next_sibling(child1);
        c2 = doc2.next_sibling(child2);
    }
    assert!(c1.is_none(), "doc1 has extra children");
    assert!(c2.is_none(), "doc2 has extra children");
}

fn verify_roundtrip(xml: &str) {
    // 1. Parse original XML
    let doc1 = Document::parse(xml).expect("Failed to parse original XML");

    // 2. Serialize to string
    let serialized1 = doc1.to_string();

    // 3. Parse serialized XML
    let doc2 = Document::parse(&serialized1).expect("Failed to parse serialized XML");

    // 4. Serialize again
    let serialized2 = doc2.to_string();

    // 5. Assert string stability
    assert_eq!(serialized1, serialized2);

    // 6. Assert DOM equality
    assert_dom_equal(&doc1, doc1.root(), &doc2, doc2.root());
}

#[test]
fn test_roundtrip_simple() {
    let xml = "<root><child attr=\"val\">hello</child></root>";
    verify_roundtrip(xml);
}

#[test]
fn test_roundtrip_empty_elements() {
    let xml = "<root><empty/><empty_with_attr a=\"1\" b=\"2\"/></root>";
    verify_roundtrip(xml);
}

#[test]
fn test_roundtrip_comments_and_decls() {
    let xml = "\
<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<!--A top level comment-->
<root>
    <!--A comment-->
    <child>content</child>
</root>
";
    verify_roundtrip(xml);
}

#[test]
fn test_roundtrip_escaped_entities() {
    let xml = "<root attr=\"&lt;&gt;&amp;&quot;&apos;\">escape text: &amp; &lt; &gt;</root>";
    verify_roundtrip(xml);
}

#[test]
fn test_roundtrip_cdata() {
    let xml = "<root><![CDATA[some raw <xml> & text]]></root>";
    verify_roundtrip(xml);
}

#[test]
fn test_roundtrip_complex() {
    let xml = "\
<?xml version=\"1.0\"?>
<catalog>
    <book id=\"bk101\">
        <author>Gambardella, Matthew</author>
        <title>XML Developer's Guide</title>
        <genre>Computer</genre>
        <price>44.95</price>
        <publish_date>2000-10-01</publish_date>
        <description>An in-depth look at creating applications with XML.</description>
    </book>
    <book id=\"bk102\">
        <author>Ralls, Kim</author>
        <title>Midnight Rain</title>
        <genre>Fantasy</genre>
        <price>5.95</price>
        <publish_date>2000-12-16</publish_date>
        <description>A former journalist finds themselves in a fantasy realm.</description>
    </book>
</catalog>
";
    verify_roundtrip(xml);
}
