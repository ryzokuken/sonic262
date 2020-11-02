use criterion::{black_box, criterion_group, criterion_main, Criterion};

use sonic262::run_all;
use sonic262::Diagnostics;

use std::path::PathBuf;

fn benchmark(c: &mut Criterion) {
    c.bench_function("single", |b| {
        b.iter(|| {
            run_all(
                PathBuf::from(black_box("./benches/fixtures/single.js")),
                PathBuf::from(black_box("./benches/fixtures/harness")),
                Diagnostics::default()
            )
        })
    });
    let mut group = c.benchmark_group("bigger");
    group.sample_size(25);
    group.bench_function("multiple", |b| {
        b.iter(|| {
            run_all(
                PathBuf::from(black_box("./benches/fixtures/multiple")),
                PathBuf::from(black_box("./benches/fixtures/harness")),
                Diagnostics::default()
            )
        })
    });
    group.finish();
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
