use criterion::{criterion_group, criterion_main};

fn dom_benchmarks(_c: &mut criterion::Criterion) {
    // TODO: Add DOM traversal/modification benchmarks
}

criterion_group!(benches, dom_benchmarks);
criterion_main!(benches);
