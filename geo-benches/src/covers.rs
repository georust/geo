use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
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

// bench a method derived from intersects
fn rect_linestring_scaling(c: &mut Criterion) {
    fn make_outer_rect(n: i32) -> Rect<f64> {
        Rect::new(coord! {x:0,y:0}, coord! {x:n,y:3}).convert()
    }
    fn make_inner_ls(n: i32) -> LineString<f64> {
        LineString::new((1..n).map(|i: i32| coord! {x:i,y:(1+i%2)}).collect()).convert()
    }

    {
        // create two polygons, both of  of n+2 sides and no holes
        let mut group = c.benchmark_group("covers rect linestring scaling");

        // trait is faster for small polygons, but relate overtakes from around 700*700 boundary segment checks
        for i in [10, 1_000, 100_000] {
            group.throughput(Throughput::Elements(i as u64));

            let inner_poly = make_inner_ls(i);
            let outer_poly = make_outer_rect(i);

            group.bench_with_input(
                BenchmarkId::new("covers trait", i),
                &(&outer_poly, &inner_poly),
                |bencher, &(a, b)| {
                    bencher.iter(|| assert!(a.covers(b)));
                },
            );

            // contains delegates this to relate
            group.bench_with_input(
                BenchmarkId::new("contains trait", i),
                &(&outer_poly, &inner_poly),
                |bencher, &(a, b)| {
                    bencher.iter(|| assert!(a.contains(b)));
                },
            );
        }
        group.finish();
    }
}

criterion_group!(
    benches,
    rect_covers,
    linestring_covers_point,
    rect_linestring_scaling
);
criterion_main!(benches);
