#[macro_use]
extern crate criterion;
extern crate wkt;

use std::str::FromStr;

fn criterion_benchmark(c: &mut criterion::Criterion) {
    c.bench_function("parse small", |bencher| {
        let s = include_str!("./small.wkt");
        bencher.iter(|| {
            let _ = wkt::Wkt::<f64>::from_str(s).unwrap();
        });
    });

    c.bench_function("parse big", |bencher| {
        let s = include_str!("./big.wkt");
        bencher.iter(|| {
            let _ = wkt::Wkt::<f64>::from_str(s).unwrap();
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
