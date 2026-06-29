// Benchmark stubs — will be implemented in Phase 7.
use criterion::{criterion_group, criterion_main};

fn parse_benchmarks(_c: &mut criterion::Criterion) {
    // TODO: Add parse benchmarks
}

criterion_group!(benches, parse_benchmarks);
criterion_main!(benches);
