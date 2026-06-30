use criterion::{Criterion, black_box, criterion_group, criterion_main};
use std::ffi::CString;
use std::os::raw::{c_char, c_void};
use tinyxml2::Document;
use tinyxml2_bench as _;

unsafe extern "C" {
    fn cpp_parse(xml: *const c_char) -> *mut c_void;
    fn cpp_free(doc: *mut c_void);
    fn cpp_print_compact(doc: *mut c_void, out_len: *mut usize) -> *mut c_char;
    fn cpp_free_str(str: *mut c_char);
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

fn comparison_benchmarks(c: &mut Criterion) {
    let xml_str = build_large_xml(1_000); // ~100KB
    let c_xml_str = CString::new(xml_str.clone()).unwrap();

    let mut group = c.benchmark_group("comparison");

    // 1. Parsing
    group.bench_function("parse_rust", |b| {
        b.iter(|| {
            let doc = Document::parse(black_box(&xml_str)).unwrap();
            black_box(doc);
        });
    });

    group.bench_function("parse_cpp", |b| {
        b.iter(|| {
            let doc_ptr = unsafe { cpp_parse(black_box(c_xml_str.as_ptr())) };
            assert!(!doc_ptr.is_null());
            unsafe { cpp_free(doc_ptr) };
        });
    });

    // 2. Printing
    let rust_doc = Document::parse(&xml_str).unwrap();
    let cpp_doc_ptr = unsafe { cpp_parse(c_xml_str.as_ptr()) };
    assert!(!cpp_doc_ptr.is_null());

    group.bench_function("print_compact_rust", |b| {
        b.iter(|| {
            let s = black_box(&rust_doc).to_string_compact();
            black_box(s);
        });
    });

    group.bench_function("print_compact_cpp", |b| {
        b.iter(|| {
            let mut len = 0;
            let ptr = unsafe { cpp_print_compact(black_box(cpp_doc_ptr), std::ptr::addr_of_mut!(len)) };
            assert!(!ptr.is_null());
            unsafe { cpp_free_str(ptr) };
        });
    });

    unsafe { cpp_free(cpp_doc_ptr) };
    group.finish();
}

criterion_group!(benches, comparison_benchmarks);
criterion_main!(benches);
