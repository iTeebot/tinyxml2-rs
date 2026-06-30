//! Programmatically builds a complex XML document with elements, CDATA, comments,
//! declarations, and nested attributes, and outputs the result to stdout.

use tinyxml2::{Document, Result};

fn main() -> Result<()> {
    let mut doc = Document::new();

    // 1. Add XML Declaration
    // Matches C++: tinyxml2::XMLDeclaration* decl = doc.NewDeclaration(nullptr);
    let decl = doc.new_declaration("xml version=\"1.0\" encoding=\"UTF-8\"");
    doc.insert_first_child(doc.root(), decl)?;

    // 2. Add a comment
    let comment = doc.new_comment("Configuration generated automatically — DO NOT EDIT");
    doc.insert_end_child(doc.root(), comment)?;

    // 3. Create Root Element
    let root = doc.new_element("system-config");
    doc.insert_end_child(doc.root(), root)?;
    doc.set_attribute(root, "version", "1.0.0")?;
    doc.set_attribute(root, "status", "production")?;

    // 4. Create and Nest Server Configuration Element
    let server = doc.new_element("server");
    doc.insert_end_child(root, server)?;
    doc.set_attribute(server, "id", "srv-west-01")?;

    let host = doc.new_element("host");
    doc.set_text(host, "127.0.0.1")?;
    doc.insert_end_child(server, host)?;

    let port = doc.new_element("port");
    // Set attribute
    doc.set_attribute(port, "value", "8080")?;
    doc.insert_end_child(server, port)?;

    // 5. Create Database Configuration Element with CDATA
    let database = doc.new_element("database");
    doc.insert_end_child(root, database)?;
    doc.set_attribute(database, "type", "postgresql")?;

    let query_template = doc.new_element("init-query");
    doc.insert_end_child(database, query_template)?;

    // Add a CDATA section to escape SQL characters safely
    let sql_cdata = doc.new_cdata("SELECT * FROM users WHERE active = true AND role = 'admin';");
    doc.insert_end_child(query_template, sql_cdata)?;

    // 6. Serialize the programmatically built XML to stdout
    println!("--- Built XML Document ---");
    let xml_output = doc.to_string();
    println!("{xml_output}");

    Ok(())
}
