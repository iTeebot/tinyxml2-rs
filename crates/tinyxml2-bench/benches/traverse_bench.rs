use criterion::{Criterion, black_box, criterion_group, criterion_main};
use tinyxml2::{Document, NodeId, visitor::XmlVisitor};

struct NullVisitor {
    count: usize,
}

impl XmlVisitor for NullVisitor {
    fn visit_enter_element(&mut self, _doc: &Document, _element: NodeId) -> bool {
        self.count += 1;
        true
    }
}

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

fn traverse_benchmarks(c: &mut Criterion) {
    let xml = build_large_xml(1000); // ~100KB document
    let doc = Document::parse(&xml).unwrap();
    let root = doc.root();

    let mut group = c.benchmark_group("traverse");

    group.bench_function("descendants", |b| {
        b.iter(|| {
            let mut count = 0;
            for _node in doc.descendants(root) {
                count += 1;
            }
            black_box(count);
        });
    });

    group.bench_function("children", |b| {
        b.iter(|| {
            let mut count = 0;
            for _node in doc.children(root) {
                count += 1;
            }
            black_box(count);
        });
    });

    group.bench_function("child_elements", |b| {
        b.iter(|| {
            let mut count = 0;
            for _node in doc.child_elements(root, None) {
                count += 1;
            }
            black_box(count);
        });
    });

    group.bench_function("visitor", |b| {
        b.iter(|| {
            let mut visitor = NullVisitor { count: 0 };
            doc.accept(&mut visitor);
            black_box(visitor.count);
        });
    });

    group.finish();
}

criterion_group!(benches, traverse_benchmarks);
criterion_main!(benches);
