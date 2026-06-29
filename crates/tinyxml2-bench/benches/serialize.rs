use criterion::{criterion_group, criterion_main};

fn serialize_benchmarks(_c: &mut criterion::Criterion) {
    // TODO: Add serialization benchmarks
}

criterion_group!(benches, serialize_benchmarks);
criterion_main!(benches);
