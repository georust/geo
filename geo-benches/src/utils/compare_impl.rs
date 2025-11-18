/// Macro to run Trait and Relate benchmarks against the same input
/// set up a benchmark group so that Criteron can plot the comparisons
#[macro_export]
macro_rules! trait_vs_relate {
    (
        $c:ident,
        $group_name:expr,
        $arg_a:expr,
        $arg_b:expr,
        $trait_fn:expr,
        $relate_fn:expr,
        $result:expr
    ) => {{
        let mut group = $c.benchmark_group($group_name);

        group.bench_with_input(
            "trait",
            &($arg_a.clone(), $arg_b.clone()),
            |bencher, (arg_a, arg_b)| {
                bencher.iter(|| {
                    assert_eq!(
                        $result,
                        $trait_fn(criterion::black_box(arg_a), criterion::black_box(arg_b))
                    );
                });
            },
        );

        group.bench_with_input(
            "relate",
            &($arg_a.clone(), $arg_b.clone()),
            |bencher, (arg_a, arg_b)| {
                bencher.iter(|| {
                    assert_eq!(
                        $result,
                        $relate_fn(
                            &criterion::black_box(arg_a).relate(criterion::black_box(arg_b))
                        )
                    );
                });
            },
        );
    }};
}
