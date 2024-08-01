// This needs a public export for the stitchtriangles trait. For now we decided to make it private
// so this benchmark is commented out. In case you need it and the trait still isn't public yet,
// you need to temporarily change that to make this benchmark work again.
//
// use criterion::{criterion_group, criterion_main, criterion};
// use geo::stitch::StitchTriangles;
// use geo::TriangulateSpade;
//
// fn criterion_benchmark(c: &mut criterion) {
//     c.bench_function("stitch", |bencher| {
//         let p = geo_test_fixtures::east_baton_rouge::<f32>();
//         let tris = p.unconstrained_triangulation().unwrap();
//
//         bencher.iter(|| {
//             criterion::black_box(criterion::black_box(&tris).stitch_triangulation().unwrap());
//         });
//     });
// }
//
// criterion_group!(benches, criterion_benchmark);
// criterion_main!(benches);
fn main() {
    println!("Placeholder");
}
