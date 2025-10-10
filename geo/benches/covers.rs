use criterion::{Criterion, criterion_group, criterion_main};
use geo::algorithm::{Contains, Covers};
use geo::{Convert, wkt};
use geo::{coord, geometry::*};

fn rect_covers(c: &mut Criterion) {
    c.bench_function("rect covers line", |bencher| {
        let rect = Rect::new(coord! { x: 0., y: 0. }, coord! { x: 10., y: 10. });
        let line = Line::new(coord! { x: 5., y: 5. }, coord! { x: 7., y: 7. });
        bencher.iter(|| {
            assert!(criterion::black_box(&rect).covers(criterion::black_box(&line)));
        });
    });

    c.bench_function("rect contains line", |bencher| {
        let rect = Rect::new(coord! { x: 0., y: 0. }, coord! { x: 10., y: 10. });
        let line = Line::new(coord! { x: 5., y: 5. }, coord! { x: 7., y: 7. });
        bencher.iter(|| {
            assert!(criterion::black_box(&rect).contains(criterion::black_box(&line)));
        });
    });
}

fn linestring_covers_point(c: &mut Criterion) {
    c.bench_function("linestring covers point", |bencher| {
        let ls: LineString<f64> = wkt! {LINESTRING(0 0, 10 0, 10 10, 0 10)}.convert();
        let pt: Point<f64> = wkt! {POINT(5 10)}.convert();
        bencher.iter(|| {
            assert!(criterion::black_box(&ls).covers(criterion::black_box(&pt)));
        });
    });

    c.bench_function("linestring contains point", |bencher| {
        let ls: LineString<f64> = wkt! {LINESTRING(0 0, 10 0, 10 10, 0 10)}.convert();
        let pt: Point<f64> = wkt! {POINT(5 10)}.convert();
        bencher.iter(|| {
            assert!(criterion::black_box(&ls).contains(criterion::black_box(&pt)));
        });
    });
}

criterion_group!(benches, rect_covers, linestring_covers_point,);
criterion_main!(benches);
