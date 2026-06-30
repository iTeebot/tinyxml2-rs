use criterion::{Criterion, black_box, criterion_group, criterion_main};
use tinyxml2::Document;

fn build_large_xml(elements: usize) -> String {
    let mut content = String::new();
    content.push_str("<root>\n");
    for i in 0..elements {
        content.push_str("  <item id=\"");
        content.push_str(&i.to_string());
        content.push_str("\">\n");
        content.push_str("    <name>Item ");
        content.push_str(&i.to_string());
        content.push_str("</name>\n");
        content.push_str("    <description>This is details for item ");
        content.push_str(&i.to_string());
        content.push_str("</description>\n");
        content.push_str("    <active>true</active>\n");
        content.push_str("  </item>\n");
    }
    content.push_str("</root>");
    content
}

fn print_benchmarks(c: &mut Criterion) {
    let small_xml = "<root><child attr=\"val\">hello world</child></root>";
    let medium_xml = build_large_xml(1000); // ~100KB

    let small_doc = Document::parse(small_xml).unwrap();
    let medium_doc = Document::parse(&medium_xml).unwrap();

    let mut group = c.benchmark_group("print");

    group.bench_function("pretty_small", |b| {
        b.iter(|| {
            let _ = black_box(&small_doc).to_string();
        });
    });

    group.bench_function("compact_small", |b| {
        b.iter(|| {
            let _ = black_box(&small_doc).to_string_compact();
        });
    });

    group.bench_function("pretty_medium", |b| {
        b.iter(|| {
            let _ = black_box(&medium_doc).to_string();
        });
    });

    group.bench_function("compact_medium", |b| {
        b.iter(|| {
            let _ = black_box(&medium_doc).to_string_compact();
        });
    });

    group.finish();
}

criterion_group!(benches, print_benchmarks);
criterion_main!(benches);
