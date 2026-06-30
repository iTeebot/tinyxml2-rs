//! Implements a custom `XmlVisitor` to collect nodes or count elements,
//! demonstrating acceptance of the visitor across the document tree.

use tinyxml2::{Document, NodeId, Result, XmlVisitor};

/// A custom XML Visitor that counts node types and collects Element names.
#[derive(Default)]
struct DomStatsCollector {
    element_count: usize,
    text_count: usize,
    comment_count: usize,
    declaration_count: usize,
    element_names: Vec<String>,
}

impl XmlVisitor for DomStatsCollector {
    fn visit_enter_document(&mut self, _doc: &Document) -> bool {
        println!("Starting DOM traversal...");
        true
    }

    fn visit_exit_document(&mut self, _doc: &Document) -> bool {
        println!("Finished DOM traversal.");
        true
    }

    fn visit_enter_element(&mut self, doc: &Document, element: NodeId) -> bool {
        self.element_count += 1;
        if let Some(el_ref) = doc.element_ref(element) {
            self.element_names.push(el_ref.name().to_string());
        }
        true
    }

    fn visit_text(&mut self, _doc: &Document, _text: NodeId) -> bool {
        self.text_count += 1;
        true
    }

    fn visit_comment(&mut self, _doc: &Document, _comment: NodeId) -> bool {
        self.comment_count += 1;
        true
    }

    fn visit_declaration(&mut self, _doc: &Document, _declaration: NodeId) -> bool {
        self.declaration_count += 1;
        true
    }
}

fn main() -> Result<()> {
    // 1. Prepare sample XML
    let xml = r#"<?xml version="1.0"?>
<!-- Project dependencies configuration -->
<project name="tinyxml2-rs">
    <!-- Core library details -->
    <package type="library">
        <name>tinyxml2</name>
        <version>1.0.0</version>
        <description>Rust-native TinyXML2 compatible parser</description>
    </package>
    <!-- FFI layer details -->
    <package type="ffi">
        <name>tinyxml2-capi</name>
        <version>1.0.0</version>
    </package>
</project>
"#;

    let mut doc = Document::new();
    doc.parse_str(xml)?;

    // 2. Instantiate our custom stats collector
    let mut collector = DomStatsCollector::default();

    // 3. Accept the visitor starting from the Document Node
    // Matches C++: doc.Accept(&collector);
    doc.accept(&mut collector);

    // 4. Output the gathered statistics
    println!("\n--- DOM Statistics ---");
    println!("Declarations found: {}", collector.declaration_count);
    println!("Comments found:     {}", collector.comment_count);
    println!("Elements found:     {}", collector.element_count);
    println!("Text nodes found:   {}", collector.text_count);
    println!("\nList of element tags:");
    for tag in &collector.element_names {
        println!("  - <{tag}>");
    }

    Ok(())
}
