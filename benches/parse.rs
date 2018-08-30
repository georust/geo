#[macro_use]
extern crate criterion;
extern crate wkt;

use criterion::Criterion;

fn criterion_benchmark(c: &mut criterion::Criterion) {
    c.bench_function("parse", |bencher| {
        let s = include_str!("./terrang_coverage.wkt");
        bencher.iter(|| {
            let _ = wkt::Wkt::from_str(s).unwrap();
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);