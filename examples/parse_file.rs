//! Reads an XML file on disk, extracts metadata, performs basic error handling,
//! and prints formatted DOM content.

use std::fs;
use std::path::Path;
use tinyxml2::{Document, Result};

fn main() -> Result<()> {
    // 1. Prepare a sample XML file on disk
    let path = Path::new("example_sample.xml");
    let sample_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<bookstore name="Science &amp; Fiction">
    <book category="physics" price="14.99">
        <title lang="en">Brief History of Time</title>
        <author>Stephen Hawking</author>
        <year>1988</year>
    </book>
    <book category="fiction" price="9.99">
        <title lang="en">Dune</title>
        <author>Frank Herbert</author>
        <year>1965</year>
    </book>
</bookstore>
"#;
    fs::write(path, sample_content).expect("Failed to write sample XML file");

    println!("Created sample XML file: {}", path.display());

    // 2. Load and parse the XML file
    let mut doc = Document::new();
    if let Err(e) = doc.load_file_mut(path.to_str().unwrap()) {
        eprintln!("Failed to load/parse XML file: {e:?}");
        // Clean up and return the error
        let _ = fs::remove_file(path);
        return Err(e);
    }

    // 3. Extract and display metadata
    let root_id = doc.root();
    if let Some(bookstore_id) = doc.first_child_element(root_id, Some("bookstore")) {
        let bookstore = doc.element_ref(bookstore_id).unwrap();
        // Resolve attribute with entity reference decoding ("Science & Fiction")
        let store_name = bookstore.attribute("name").unwrap_or("Unknown");
        println!("\nBookstore Name: {store_name}");

        println!("\nBooks listing:");
        // Iterate through all "book" child elements
        for book in doc.child_elements(bookstore_id, Some("book")) {
            let book_id = book.id();
            let category = book.attribute("category").unwrap_or("general");

            let title_id = doc.first_child_element(book_id, Some("title")).unwrap();
            let title_ref = doc.element_ref(title_id).unwrap();
            let title = title_ref.text().unwrap_or("");
            let lang = title_ref.attribute("lang").unwrap_or("en");

            let author_id = doc.first_child_element(book_id, Some("author")).unwrap();
            let author = doc.get_text(author_id).unwrap_or("");

            let price: f64 = doc.query_double_attribute(book_id, "price").unwrap_or(0.0);

            println!("- [{category}] \"{title}\" by {author} (${price}) (Language: {lang})");
        }
    } else {
        println!("No <bookstore> root element found.");
    }

    // 4. Output the formatted (pretty-printed) XML representation of the DOM
    println!("\n--- Pretty-printed DOM output ---");
    let formatted_xml = doc.to_string();
    println!("{formatted_xml}");

    // Clean up temporary file
    let _ = fs::remove_file(path);
    Ok(())
}
