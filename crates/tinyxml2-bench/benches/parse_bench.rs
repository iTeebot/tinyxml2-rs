use criterion::{Criterion, Throughput, black_box, criterion_group, criterion_main};
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

fn parse_benchmarks(c: &mut Criterion) {
    let small_xml = "<root><child attr=\"val\">hello world</child></root>"; // ~1KB
    let medium_xml = build_large_xml(1_000); // ~100KB
    let large_xml = build_large_xml(100_000); // ~10MB

    let mut group = c.benchmark_group("parse");

    group.throughput(Throughput::Bytes(small_xml.len() as u64));
    group.bench_function("small_1kb", |b| {
        b.iter(|| {
            let _ = Document::parse(black_box(small_xml)).unwrap();
        });
    });

    group.throughput(Throughput::Bytes(medium_xml.len() as u64));
    group.bench_function("medium_100kb", |b| {
        b.iter(|| {
            let _ = Document::parse(black_box(&medium_xml)).unwrap();
        });
    });

    group.throughput(Throughput::Bytes(large_xml.len() as u64));
    group.bench_function("large_10mb", |b| {
        b.iter(|| {
            let _ = Document::parse(black_box(&large_xml)).unwrap();
        });
    });

    group.finish();
}

criterion_group!(benches, parse_benchmarks);
criterion_main!(benches);
